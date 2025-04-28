use crate::slaspec::instructions::{
    core::{InstrBuilder, InstrFamilyBuilder},
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

    ifam.add_instr(instr_nop(&ifam));

    ifam
}

fn instr_nop(ifam: &InstrFamilyBuilder) -> InstrBuilder {
    InstrBuilder::new("NOP", ifam)
}
