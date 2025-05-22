use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam: InstrFamilyBuilder = InstrFamilyBuilder::new_16(
        "LdStIIFP",
        "Load/Store indexed with small immediate offset FP",
        "lsfp",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x2e), 6),
            ProtoField::new("w", FieldType::Blank, 1),
            ProtoField::new("off", FieldType::UImmVal, 5),
            ProtoField::new("g", FieldType::Blank, 1),
            ProtoField::new("reg", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&LdStImmFPFactory());

    ifam
}

struct LdStImmFPFactory();

impl LdStImmFPFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, preg: bool, store: bool) -> InstrBuilder {
        let addr_str = "M32bit";
        let reg_str = if preg { "Preg" } else { "Dreg" };
        let name = if store {
            format!("St{}To{}", reg_str, addr_str)
        } else {
            format!("Ld{}To{}", addr_str, reg_str)
        };

        let display = if store {
            format!("[FP - {{$imm}}] = {{reg}}")
        } else {
            format!("{{reg}} = [FP - {{$imm}}]")
        };

        let addr_expr = b_ptr(b_grp(e_sub(b_reg("FP"), b_var("imm"))), 4);
        let reg_expr = e_rfield("reg");

        InstrBuilder::new(ifam)
            .name(&name)
            .display(display)
            .set_field_type("w", FieldType::Mask(store as u16))
            .set_field_type("g", FieldType::Mask(preg as u16))
            .set_field_type(
                "reg",
                FieldType::Variable(if preg {
                    RegisterSet::PReg
                } else {
                    RegisterSet::DReg
                }),
            )
            .add_action(e_copy(
                b_var("imm"),
                e_sub(b_num(0x80), b_grp(e_lshft(e_field("off"), b_num(2)))),
            ))
            .add_pcode(if store {
                e_copy(addr_expr, reg_expr)
            } else {
                e_copy(reg_expr, addr_expr)
            })
    }
}

impl InstrFactory for LdStImmFPFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let params = [false, true];

        params
            .iter()
            .cartesian_product(params.iter())
            .map(|(store, preg)| Self::base_instr(ifam, *preg, *store))
            .collect()
    }
}
