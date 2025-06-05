use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "DAGModIm",
        "DAG Arithmetic",
        "dmm",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x9e), 8),
            ProtoField::new("br", FieldType::Blank, 1),
            ProtoField::new("mask2", FieldType::Mask(0x3), 2),
            ProtoField::new("op", FieldType::Blank, 1),
            ProtoField::new("m", FieldType::Variable(RegisterSet::MReg), 2),
            ProtoField::new("i", FieldType::Variable(RegisterSet::IReg), 2),
        ]),
    );

    ifam.add_instrs(&DagAddFactory());

    ifam
}

struct DagAddFactory();

impl DagAddFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, brev: bool, dec: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("DagAdd32")
            .display(format!(
                "{{i}} {}= {{m}}{}",
                if dec { "-" } else { "+" },
                if brev { " (BREV)" } else { "" }
            ))
            .set_field_type("br", FieldType::Mask(brev as u16))
            .set_field_type("op", FieldType::Mask(dec as u16))
            .add_pcode(cs_assign_by(
                if dec { e_sub } else { e_add },
                e_rfield("i"),
                e_rfield("m"),
            ))
    }
}

impl InstrFactory for DagAddFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::base_instr(ifam, false, false),
            Self::base_instr(ifam, true, false),
            Self::base_instr(ifam, false, true),
        ]
    }
}
