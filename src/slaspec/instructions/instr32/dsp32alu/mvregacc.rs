use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Aop {
    RegA0 = 0x0,
    RegLA0X = 0x1,
    RegA1 = 0x2,
    RegLA1X = 0x3,
}

impl Aop {
    fn name(&self, sat: bool) -> &'static str {
        match self {
            Aop::RegA0 | Aop::RegA1 => {
                if sat {
                    "MvDregToAx"
                } else {
                    "MvDregHLToAxHL"
                }
            }
            Aop::RegLA0X | Aop::RegLA1X => "MvDregLToAxX",
        }
    }

    fn acc(&self) -> usize {
        match self {
            Aop::RegA0 | Aop::RegLA0X => 0,
            _ => 1,
        }
    }

    fn display(&self, sat: bool, ext: bool, hl: bool) -> String {
        let acc_str = format!("A{}", self.acc());
        let half_str = if hl { ".H" } else { ".L" };
        let ext_str = if ext { "Z" } else { "X" };

        match self {
            Aop::RegA0 | Aop::RegA1 => {
                if sat {
                    format!("{acc_str} = {{src0}} ({ext_str})")
                } else {
                    format!("{acc_str}{half_str} = {{src0}}")
                }
            }
            Aop::RegLA0X | Aop::RegLA1X => format!("{acc_str}.X = {{src0}}"),
        }
    }

    fn src_reg(&self, sat: bool, hl: bool) -> RegisterSet {
        match self {
            Aop::RegA0 | Aop::RegA1 => {
                if sat {
                    RegisterSet::DReg
                } else {
                    if hl {
                        RegisterSet::DRegH
                    } else {
                        RegisterSet::DRegL
                    }
                }
            }
            Aop::RegLA0X | Aop::RegLA1X => RegisterSet::DRegL,
        }
    }

    fn expr(&self, sat: bool, ext: bool, hl: bool) -> Expr {
        let acc_str = format!("A{}", self.acc());
        let half_str = if hl { ".H" } else { ".L" };
        let ext_op = if ext { e_zext } else { e_sext };

        match self {
            Aop::RegA0 | Aop::RegA1 => {
                if sat {
                    e_copy(b_reg(&acc_str), ext_op(e_rfield("src0")))
                } else {
                    e_copy(b_reg(&format!("{acc_str}{half_str}")), e_rfield("src0"))
                }
            }
            Aop::RegLA0X | Aop::RegLA1X => {
                e_copy(b_reg(&format!("{acc_str}.X")), b_size(e_rfield("src0"), 2))
            }
        }
    }
}

pub struct MvRegAccFactory();

impl MvRegAccFactory {
    fn base_instr(
        ifam: &InstrFamilyBuilder,
        aop: Aop,
        sat: bool,
        ext: bool,
        hl: bool,
    ) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(aop.name(sat))
            .display(aop.display(sat, ext, hl))
            .set_field_type("hl", FieldType::Mask(hl as u16))
            .set_field_type("aopc", FieldType::Mask(0x9))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(ext as u16))
            .set_field_type("src0", FieldType::Variable(aop.src_reg(sat, hl)))
            .add_pcode(aop.expr(sat, ext, hl))
    }
}

impl InstrFactory for MvRegAccFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::base_instr(ifam, Aop::RegA0, false, false, false),
            Self::base_instr(ifam, Aop::RegA0, false, false, true),
            Self::base_instr(ifam, Aop::RegA0, true, false, false),
            Self::base_instr(ifam, Aop::RegA0, true, true, false),
            Self::base_instr(ifam, Aop::RegLA0X, false, false, false),
            Self::base_instr(ifam, Aop::RegA1, false, false, false),
            Self::base_instr(ifam, Aop::RegA1, false, false, true),
            Self::base_instr(ifam, Aop::RegA1, true, false, false),
            Self::base_instr(ifam, Aop::RegA1, true, true, false),
            Self::base_instr(ifam, Aop::RegLA1X, false, false, false),
        ]
    }
}
