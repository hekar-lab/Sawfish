use std::fmt;

use crate::slaspec::instructions::common::UnOp;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Aop {
    A0 = 0x0,
    A1 = 0x1,
    Dual = 0x3,
}

impl Aop {
    fn name(&self) -> &'static str {
        match self {
            Aop::A0 => "Acc0",
            Aop::A1 => "Acc1",
            Aop::Dual => "AccDual",
        }
    }
}

impl fmt::Display for Aop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct AccOpFactory();

impl AccOpFactory {
    fn neg_instr(ifam: &InstrFamilyBuilder, aop: Aop, hl: bool) -> InstrBuilder {
        let hl_acc = if hl { "A1" } else { "A0" };
        InstrBuilder::new(ifam)
            .name(&format!("Neg{}", aop.name()))
            .display(if aop == Aop::Dual {
                "A1 = -A1, A0 = -A0".to_string()
            } else {
                format!("{hl_acc} = -{aop}")
            })
            .set_field_type("hl", FieldType::Mask(hl as u16))
            .set_field_type("aopc", FieldType::Mask(0xe))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(0x0))
            .add_pcode(if aop == Aop::Dual {
                cs_mline(vec![
                    cs_neg_sat(b_reg("A1"), b_reg("A1"), true, 5, "A1"),
                    cs_neg_sat(b_reg("A0"), b_reg("A0"), true, 5, "A0"),
                ])
            } else {
                cs_neg_sat(
                    b_reg(hl_acc),
                    b_reg(&format!("{aop}")),
                    true,
                    5,
                    &format!("{hl_acc}{aop}"),
                )
            })
    }

    fn abs_instr(ifam: &InstrFamilyBuilder, aop: Aop, hl: bool) -> InstrBuilder {
        let hl_acc = if hl { "A1" } else { "A0" };
        InstrBuilder::new(ifam)
            .name(&format!("Abs{}", aop.name()))
            .display(if aop == Aop::Dual {
                "A1 = ABS A1, A0 = ABS A0".to_string()
            } else {
                format!("{hl_acc} = ABS {aop}")
            })
            .set_field_type("hl", FieldType::Mask(hl as u16))
            .set_field_type("aopc", FieldType::Mask(0x10))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(0x0))
            .add_pcode(if aop == Aop::Dual {
                cs_mline(vec![
                    cs_abs_sat(b_reg("A1"), b_reg("A1"), 5, "A1"),
                    cs_abs_sat(b_reg("A0"), b_reg("A0"), 5, "A0"),
                ])
            } else {
                cs_abs_sat(
                    b_reg(hl_acc),
                    b_reg(&format!("{aop}")),
                    5,
                    &format!("{hl_acc}{aop}"),
                )
            })
    }

    fn weird_instr(ifam: &InstrFamilyBuilder, smode: bool, xmode: bool) -> InstrBuilder {
        fn mode_str(m: bool) -> &'static str {
            if m { "Z" } else { "X" }
        }

        fn mode_op(m: bool) -> UnOp {
            if m { e_zext } else { e_sext }
        }

        InstrBuilder::new(ifam)
            .name("MvDregToAxDual")
            .display(format!(
                "A1 = {{src1}} ({}), A0 = {{src0}} ({})",
                mode_str(smode),
                mode_str(xmode)
            ))
            .set_field_type("hl", FieldType::Mask(0x1))
            .set_field_type("aopc", FieldType::Mask(0x10))
            .set_field_type("aop", FieldType::Mask(0x3))
            .set_field_type("s", FieldType::Mask(smode as u16))
            .set_field_type("x", FieldType::Mask(xmode as u16))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_copy(b_reg("A1"), mode_op(smode)(e_rfield("src1"))))
            .add_pcode(e_copy(b_reg("A0"), mode_op(xmode)(e_rfield("src0"))))
    }
}

impl InstrFactory for AccOpFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::neg_instr(ifam, Aop::A0, false),
            Self::neg_instr(ifam, Aop::A0, true),
            Self::neg_instr(ifam, Aop::A1, false),
            Self::neg_instr(ifam, Aop::A1, true),
            Self::neg_instr(ifam, Aop::Dual, false),
            Self::abs_instr(ifam, Aop::A0, false),
            Self::abs_instr(ifam, Aop::A0, true),
            Self::abs_instr(ifam, Aop::A1, false),
            Self::abs_instr(ifam, Aop::A1, true),
            Self::abs_instr(ifam, Aop::Dual, false),
            Self::weird_instr(ifam, false, false),
            Self::weird_instr(ifam, false, true),
            Self::weird_instr(ifam, true, false),
            Self::weird_instr(ifam, true, true),
        ]
    }
}
