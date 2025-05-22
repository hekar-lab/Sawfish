use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "CompI2op",
        "Destructive Binary Operations, dreg/preg with 7bit immediate",
        "ci2",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x6), 4),
            ProtoField::new("r", FieldType::Blank, 1),
            ProtoField::new("opc", FieldType::Blank, 1),
            ProtoField::new("src", FieldType::SImmVal, 7),
            ProtoField::new("dst", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&BinOpFactory());

    ifam
}

struct BinOpFactory();

impl BinOpFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, preg: bool, add: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(&format!(
                "{}{}",
                if preg & add { "Dag" } else { "" },
                if add { "AddImm" } else { "LdImmToReg" }
            ))
            .display(format!(
                "{{dst}} {}= {{src}}{}",
                if add { "+" } else { "" },
                if add { "" } else { " (X)" }
            ))
            .set_field_type("r", FieldType::Mask(preg as u16))
            .set_field_type("opc", FieldType::Mask(add as u16))
            .set_field_type(
                "dst",
                FieldType::Variable(if preg {
                    RegisterSet::PReg
                } else {
                    RegisterSet::DReg
                }),
            )
            .add_pcode(e_copy(
                e_rfield("dst"),
                if add {
                    e_add(e_rfield("dst"), e_field("src"))
                } else {
                    e_macp("sext", b_size(e_field("src"), 1))
                },
            ))
    }
}

impl InstrFactory for BinOpFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let params = [false, true];

        params
            .iter()
            .cartesian_product(params.iter())
            .map(|(preg, add)| Self::base_instr(ifam, *preg, *add))
            .collect()
    }
}
