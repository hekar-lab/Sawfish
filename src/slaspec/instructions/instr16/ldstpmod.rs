use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "LdStPmod",
        "Load/Store postmodify addressing, pregister based",
        "lsp",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x8), 4),
            ProtoField::new("w", FieldType::Blank, 1),
            ProtoField::new("aop", FieldType::Blank, 2),
            ProtoField::new("reg", FieldType::Blank, 3),
            ProtoField::new("idx", FieldType::Variable(RegisterSet::PReg), 3),
            ProtoField::new("ptr", FieldType::Variable(RegisterSet::PReg), 3),
        ]),
    );

    ifam.add_instrs(&LdStFactory());
    ifam.add_instrs(&BitExtFactory());

    ifam
}

struct LdStFactory();

impl LdStFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, store: bool, aop: u16) -> InstrBuilder {
        let reg_str = &format!(
            "Dreg{}",
            match aop {
                1 => "L",
                2 => "H",
                _ => "",
            }
        );
        let ptr_str = if aop == 0 { "M32bit" } else { "M16bit" };
        let name = if store {
            format!("St{}To{}", reg_str, ptr_str)
        } else {
            format!("Ld{}To{}", ptr_str, reg_str)
        };

        let addr_str = format!(
            "{}[{{ptr}}++{{idx}}]",
            match aop {
                1 | 2 => "W",
                _ => "",
            }
        );
        let display = if store {
            format!("{addr_str} = {{reg}}")
        } else {
            format!("{{reg}} = {addr_str}")
        };

        let addr_expr = e_ptr(e_rfield("ptr"), if aop == 0 { 4 } else { 2 });

        InstrBuilder::new(ifam)
            .name(&name)
            .display(display)
            .set_field_type("w", FieldType::Mask(store as u16))
            .set_field_type("aop", FieldType::Mask(aop))
            .set_field_type(
                "reg",
                FieldType::Variable(match aop {
                    1 => RegisterSet::DRegL,
                    2 => RegisterSet::DRegH,
                    _ => RegisterSet::DReg,
                }),
            )
            .add_pcode(if store {
                e_copy(addr_expr, e_rfield("reg"))
            } else {
                e_copy(e_rfield("reg"), addr_expr)
            })
            .add_pcode(e_copy(
                e_rfield("ptr"),
                e_add(e_rfield("ptr"), e_rfield("idx")),
            ))
    }
}

impl InstrFactory for LdStFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [false, true]
            .iter()
            .cartesian_product([0, 1, 2].iter())
            .map(|(store, aop)| Self::base_instr(ifam, *store, *aop))
            .collect()
    }
}

struct BitExtFactory();

impl BitExtFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sext: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("LdM16bitToDreg")
            .display(format!(
                "{{reg}} = W[{{ptr}}++{{idx}}] ({})",
                if sext { "X" } else { "Z" }
            ))
            .set_field_type("w", FieldType::Mask(sext as u16))
            .set_field_type("aop", FieldType::Mask(0x3))
            .set_field_type("reg", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_copy(
                e_rfield("reg"),
                e_macp(
                    if sext { "sext" } else { "zext" },
                    e_ptr(e_rfield("ptr"), 2),
                ),
            ))
            .add_pcode(e_copy(
                e_rfield("ptr"),
                e_add(e_rfield("ptr"), e_rfield("idx")),
            ))
    }
}

impl InstrFactory for BitExtFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [false, true]
            .into_iter()
            .map(|sext| Self::base_instr(ifam, sext))
            .collect()
    }
}
