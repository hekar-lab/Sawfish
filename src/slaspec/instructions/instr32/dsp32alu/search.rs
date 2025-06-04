use std::fmt;

use crate::slaspec::instructions::common::BinOp;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Aop {
    GT = 0x0,
    GE = 0x1,
    LT = 0x2,
    LE = 0x3,
}

impl Aop {
    fn comp_op(&self) -> BinOp {
        match self {
            Aop::GT => e_gts,
            Aop::GE => e_ges,
            Aop::LT => e_lts,
            Aop::LE => e_les,
        }
    }
}

impl fmt::Display for Aop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct SearchFactory();

impl SearchFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, aop: Aop) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("Search")
            .display(format!("({{dst1}}, {{dst0}}) = SEARCH {{src0}} ({aop})"))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0xd))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("dst1", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_local("comp_var", 2))
            .add_pcode(cs_mline(vec![
                e_copy(b_var("comp_var"), b_size(e_rfield("src0"), 2)),
                b_ifgoto(
                    e_not(b_grp(aop.comp_op()(
                        b_var("comp_var"),
                        b_size(b_reg("A0"), 2),
                    ))),
                    b_label("search_end_A0"),
                ),
                e_copy(b_reg("A0"), e_sext(b_var("comp_var"))),
                e_copy(e_rfield("dst0"), b_reg("P0")),
                b_label("search_end_A0"),
            ]))
            .add_pcode(cs_mline(vec![
                e_copy(b_var("comp_var"), b_trunc(e_rfield("src1"), 2)),
                b_ifgoto(
                    e_not(b_grp(aop.comp_op()(
                        b_var("comp_var"),
                        b_size(b_reg("A1"), 2),
                    ))),
                    b_label("search_end_A1"),
                ),
                e_copy(b_reg("A1"), e_sext(b_var("comp_var"))),
                e_copy(e_rfield("dst1"), b_reg("P0")),
                b_label("search_end_A1"),
            ]))
    }
}

impl InstrFactory for SearchFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::base_instr(ifam, Aop::GT),
            Self::base_instr(ifam, Aop::GE),
            Self::base_instr(ifam, Aop::LT),
            Self::base_instr(ifam, Aop::LE),
        ]
    }
}
