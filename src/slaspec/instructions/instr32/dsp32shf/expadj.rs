use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sop {
    Reg = 0x0,
    Vec = 0x1,
    RegL = 0x2,
    RegH = 0x3,
}

impl Sop {
    fn src_reg(&self) -> RegisterSet {
        match self {
            Sop::Reg | Sop::Vec => RegisterSet::DReg,
            Sop::RegL => RegisterSet::DRegL,
            Sop::RegH => RegisterSet::DRegH,
        }
    }

    fn display(&self) -> String {
        format!(
            "{{dst}} = EXPADJ ({{src0}}, {{src1}}){}",
            if *self == Sop::Vec { " (V)" } else { "" }
        )
    }
}

pub struct ExpAdjFactory();

impl ExpAdjFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop) -> InstrBuilder {
        let res_id = "res_expadj";
        let exp_id = "exp_expadj";

        fn get_exp(dst: Expr, src: Expr, id: &str) -> Expr {
            let endl = &format!("sign_end{id}");
            let negl = &format!("sign_neg{id}");
            cs_mline(vec![
                b_ifgoto(e_lts(src.clone(), b_num(0)), b_label(negl)),
                e_copy(dst.clone(), e_sub(e_macp("lzcount", src.clone()), b_num(1))),
                b_goto(b_label(endl)),
                b_label(negl),
                e_copy(dst, e_sub(e_macp("lzcount", e_bit_not(src)), b_num(1))),
                b_label(endl),
            ])
        }

        fn min(dst: Expr, src: Expr, id: &str) -> Expr {
            let minl = &format!("min_end{id}");
            cs_mline(vec![
                b_ifgoto(e_ge(src.clone(), dst.clone()), b_label(minl)),
                e_copy(dst, src),
                b_label(minl),
            ])
        }

        InstrBuilder::new(ifam)
            .name("ExpAdj32")
            .display(sop.display())
            .set_field_type("sopc", FieldType::Mask(0x7))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DRegL))
            .set_field_type("src0", FieldType::Variable(sop.src_reg()))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DRegL))
            .add_pcode(e_copy(e_local(res_id, 2), e_rfield("src1")))
            .add_pcode(e_local(exp_id, 2))
            .add_pcode(if sop == Sop::Vec {
                cs_mline(vec![
                    e_copy(e_local("tmp_src", 2), b_size(e_rfield("src0"), 2)),
                    get_exp(b_var(exp_id), b_var("tmp_src"), "L"),
                    min(b_var(res_id), b_var(exp_id), "L"),
                    e_copy(b_var("tmp_src"), b_trunc(e_rfield("src0"), 2)),
                    get_exp(b_var(exp_id), b_var("tmp_src"), "H"),
                    min(b_var(res_id), b_var(exp_id), "H"),
                ])
            } else {
                cs_mline(vec![
                    get_exp(b_var(exp_id), e_rfield("src0"), ""),
                    min(b_var(res_id), b_var(exp_id), ""),
                ])
            })
            .add_pcode(e_copy(e_rfield("dst"), b_var(res_id)))
    }
}

impl InstrFactory for ExpAdjFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [Sop::Reg, Sop::Vec, Sop::RegL, Sop::RegH]
            .into_iter()
            .map(|sop| Self::base_instr(ifam, sop))
            .collect()
    }
}
