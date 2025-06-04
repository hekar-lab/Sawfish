use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Aop {
    LL = 0x0,
    LH = 0x1,
    HL = 0x2,
    HH = 0x3,
}

impl Aop {
    fn src_regsets(&self) -> (RegisterSet, RegisterSet) {
        match self {
            Aop::LL => (RegisterSet::DRegL, RegisterSet::DRegL),
            Aop::LH => (RegisterSet::DRegL, RegisterSet::DRegH),
            Aop::HL => (RegisterSet::DRegH, RegisterSet::DRegL),
            Aop::HH => (RegisterSet::DRegH, RegisterSet::DRegH),
        }
    }
}

pub struct AddSubFactory();

impl AddSubFactory {
    fn disp_sat(sat: bool) -> &'static str {
        if sat { "S" } else { "NS" }
    }

    fn disp_simple(opt: &str, sub: bool) -> String {
        format!(
            "{{dst0}} = {{src0}} {} {{src1}} ({opt})",
            if sub { "-" } else { "+" }
        )
    }

    fn expr(dst_id: &str, sat: bool, sub: bool, half: bool) -> Expr {
        match (sat, sub) {
            (false, false) => e_copy(e_rfield(dst_id), e_add(e_rfield("src0"), e_rfield("src1"))),
            (false, true) => e_copy(e_rfield(dst_id), e_sub(e_rfield("src0"), e_rfield("src1"))),
            (true, false) => cs_sadd_sat(
                e_rfield(dst_id),
                e_rfield("src0"),
                e_rfield("src1"),
                if half { 2 } else { 4 },
                dst_id,
            ),
            (true, true) => cs_ssub_sat(
                e_rfield(dst_id),
                e_rfield("src0"),
                e_rfield("src1"),
                if half { 2 } else { 4 },
                dst_id,
            ),
        }
    }

    fn half_instr(instr: InstrBuilder, hl: bool) -> InstrBuilder {
        instr
            .set_field_type("hl", FieldType::Mask(hl as u16))
            .set_field_type(
                "dst0",
                FieldType::Variable(if hl {
                    RegisterSet::DRegH
                } else {
                    RegisterSet::DRegL
                }),
            )
    }

    fn as16_instr(
        ifam: &InstrFamilyBuilder,
        sat: bool,
        sub: bool,
        hl: bool,
        aop: Aop,
    ) -> InstrBuilder {
        let (src0var, src1var) = aop.src_regsets();
        Self::half_instr(InstrBuilder::new(ifam), hl)
            .name("AddSub16")
            .display(Self::disp_simple(Self::disp_sat(sat), sub))
            .set_field_type("aopc", FieldType::Mask(0x2 + sub as u16))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("src0", FieldType::Variable(src0var))
            .set_field_type("src1", FieldType::Variable(src1var))
            .add_pcode(Self::expr("dst0", sat, sub, true))
    }

    fn as32_instr(ifam: &InstrFamilyBuilder, sat: bool, aop: Aop) -> InstrBuilder {
        let sub = aop as u16 == 0x1;
        let dual = aop as u16 == 0x2;
        InstrBuilder::new(ifam)
            .name(if dual { "AddSub32Dual" } else { "AddSub32" })
            .display(if dual {
                format!(
                    "{{dst0}} = {{src0}} + {{src1}}, {{dst1}} = {{src0}} - {{src1}} ({})",
                    Self::disp_sat(sat)
                )
            } else {
                Self::disp_simple(Self::disp_sat(sat), sub)
            })
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0x4))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type_opt(dual, "dst1", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(if dual {
                cs_mline(vec![
                    Self::expr("dst0", sat, false, true),
                    Self::expr("dst1", sat, true, true),
                ])
            } else {
                Self::expr("dst0", sat, sub, true)
            })
    }

    fn asrnd_instr(ifam: &InstrFamilyBuilder, hl: bool, aop: Aop, down: bool) -> InstrBuilder {
        let sub = aop as u16 == 0x1;
        let op = if sub { e_sub } else { e_add };
        let shft_op = if down { e_arshft } else { e_lshft };

        Self::half_instr(InstrBuilder::new(ifam), hl)
            .name(if down { "AddSubRnd20" } else { "AddSubRnd12" })
            .display(Self::disp_simple(if down { "RND20" } else { "RND12" }, sub))
            .set_field_type("aopc", FieldType::Mask(0x5))
            .set_field_type("aop", FieldType::Mask(2 * down as u16 + aop as u16))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(down as u16))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_copy(
                e_local("asr_res", 5),
                op(
                    e_sext(shft_op(e_rfield("src0"), b_num(4))),
                    e_sext(shft_op(e_rfield("src1"), b_num(4))),
                ),
            ))
            .add_pcode(e_local("asr_res_rnd", 3))
            .add_pcode(cs_round_biased(
                b_var("asr_res_rnd"),
                3,
                b_var("asr_res"),
                5,
                "asr",
            ))
            .add_pcode(if down {
                e_copy(e_rfield("dst0"), b_size(b_var("asr_res_rnd"), 2))
            } else {
                cs_strunc_sat(e_rfield("dst0"), b_var("asr_res_rnd"), 2, "asr")
            })
    }
}

impl InstrFactory for AddSubFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut as16_instrs: Vec<InstrBuilder> = [false, true]
            .into_iter()
            .cartesian_product([Aop::LL, Aop::LH, Aop::HL, Aop::HH])
            .cartesian_product([false, true])
            .cartesian_product([false, true])
            .map(|(((sub, aop), hl), sat)| Self::as16_instr(ifam, sat, sub, hl, aop))
            .collect();

        let mut as32_instrs: Vec<InstrBuilder> = [Aop::LL, Aop::LH, Aop::HL]
            .into_iter()
            .cartesian_product([false, true])
            .map(|(aop, sat)| Self::as32_instr(ifam, sat, aop))
            .collect();

        let mut asrnd_instrs: Vec<InstrBuilder> = [false, true]
            .into_iter()
            .cartesian_product([Aop::LL, Aop::LH])
            .cartesian_product([false, true])
            .map(|((down, aop), hl)| Self::asrnd_instr(ifam, hl, aop, down))
            .collect();

        as16_instrs.append(&mut as32_instrs);
        as16_instrs.append(&mut asrnd_instrs);

        as16_instrs
    }
}
