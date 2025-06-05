use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sop {
    ASL = 0x0,
    ASR = 0x1,
    ASLDual = 0x2,
    ASRDual = 0x3,
}

impl Sop {
    fn dual(&self) -> bool {
        match self {
            Sop::ASLDual | Sop::ASRDual => true,
            _ => false,
        }
    }

    fn asr(&self) -> bool {
        match self {
            Sop::ASR | Sop::ASRDual => true,
            _ => false,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Sop::ASLDual | Sop::ASRDual => "DualVitMax",
            Sop::ASL | Sop::ASR => "VitMax",
        }
    }

    fn display(&self) -> String {
        format!(
            "{{dst}} = VIT_MAX ({{src0}}{}) ({})",
            if self.dual() { ", {src1}" } else { "" },
            if self.asr() { "ASR" } else { "ASL" }
        )
    }

    fn dst_reg(&self) -> RegisterSet {
        match self {
            Sop::ASLDual | Sop::ASRDual => RegisterSet::DReg,
            Sop::ASL | Sop::ASR => RegisterSet::DRegL,
        }
    }
}

pub struct VitMaxFactory();

impl VitMaxFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop) -> InstrBuilder {
        let lhs_var = "lhs_srcH";
        let rhs_var = "rhs_srcL";

        fn get_oper(src: &str, lhs: &str, rhs: &str) -> Expr {
            cs_mline(vec![
                e_copy(b_var(lhs), b_trunc(e_rfield(src), 2)),
                e_copy(b_var(rhs), b_size(e_rfield(src), 2)),
            ])
        }

        fn set_bit(val: bool, asr: bool) -> Expr {
            match (asr, val) {
                (false, false) => cs_assign_by(e_lshft, b_reg("A0.W"), b_num(1)),
                (false, true) => e_copy(
                    b_reg("A0.W"),
                    e_bit_or(b_grp(e_lshft(b_reg("A0.W"), b_num(1))), b_num(0x1)),
                ),
                (true, false) => cs_assign_by(e_rshft, b_reg("A0.W"), b_num(1)),
                (true, true) => e_copy(
                    b_reg("A0.W"),
                    e_bit_or(b_grp(e_rshft(b_reg("A0.W"), b_num(1))), b_num(0x80000000)),
                ),
            }
        }

        fn set_dst(hs: &str, dual: bool) -> Expr {
            if dual {
                cs_mline(vec![
                    cs_assign_by(e_rshft, e_rfield("dst"), b_num(16)),
                    cs_assign_by(
                        e_bit_or,
                        e_rfield("dst"),
                        b_grp(e_lshft(e_zext(b_var(hs)), b_num(16))),
                    ),
                ])
            } else {
                e_copy(e_rfield("dst"), b_var(hs))
            }
        }

        fn vitmax(src: &str, lhs: &str, rhs: &str, asr: bool, dual: bool) -> Expr {
            cs_mline(vec![
                get_oper(src, lhs, rhs),
                b_ifgoto(
                    e_ges(b_grp(e_sub(b_var(lhs), b_var(rhs))), b_num(0)),
                    b_label("vitmax_greater"),
                ),
                set_bit(false, asr),
                set_dst(rhs, dual),
                b_goto(b_label("vitmax_end")),
                b_label("vitmax_greater"),
                set_bit(true, asr),
                set_dst(lhs, dual),
                b_label("vitmax_end"),
            ])
        }

        InstrBuilder::new(ifam)
            .name(sop.name())
            .display(sop.display())
            .set_field_type("sopc", FieldType::Mask(0x9))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(sop.dst_reg()))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type_opt(sop.dual(), "src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(cs_mline(vec![e_local(lhs_var, 2), e_local(rhs_var, 2)]))
            .add_pcode(if sop.dual() {
                cs_mline(vec![
                    vitmax("src1", lhs_var, rhs_var, sop.asr(), true),
                    vitmax("src0", lhs_var, rhs_var, sop.asr(), true),
                ])
            } else {
                vitmax("src0", lhs_var, rhs_var, sop.asr(), false)
            })
    }
}

impl InstrFactory for VitMaxFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [Sop::ASL, Sop::ASR, Sop::ASLDual, Sop::ASRDual]
            .into_iter()
            .map(|sop| Self::base_instr(ifam, sop))
            .collect()
    }
}
