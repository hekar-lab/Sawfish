use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "DAGModIk",
        "DAG Arithmetic",
        "dmk",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x9f6), 12),
            ProtoField::new("opc", FieldType::Blank, 2),
            ProtoField::new("i", FieldType::Variable(RegisterSet::IReg), 2),
        ]),
    );

    ifam.add_instrs(&DagAddImmFactory());

    ifam
}

struct DagAddImmFactory();

impl DagAddImmFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, op: u16) -> InstrBuilder {
        let inc = op % 2 == 0;
        let val = 2 + 2 * ((op >= 2) as i128);

        InstrBuilder::new(ifam)
            .name("DagAddImm")
            .display(format!("{{i}} {}= {}", if inc { "+" } else { "-" }, val))
            .set_field_type("opc", FieldType::Mask(op))
            .add_pcode(cs_assign_by(
                if inc { e_add } else { e_sub },
                e_rfield("i"),
                b_num(val),
            ))
    }
}

impl InstrFactory for DagAddImmFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        (0..4).map(|op| Self::base_instr(ifam, op)).collect()
    }
}
