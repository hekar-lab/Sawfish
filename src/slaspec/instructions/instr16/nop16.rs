use crate::slaspec::instructions::{
    core::{InstrBuilder, InstrFactory, InstrFamilyBuilder},
    pattern::{FieldType, ProtoField, ProtoPattern},
};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "NOP16",
        "16-bit Slot Nop",
        "nop",
        ProtoPattern {
            fields: vec![ProtoField::new("sig", FieldType::Mask(0x0000), 16)],
        },
    );

    ifam.add_instrs(&NOPFactory());

    ifam
}

struct NOPFactory();

impl InstrFactory for NOPFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![InstrBuilder::new(ifam).name("NOP")]
    }
}
