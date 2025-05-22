use crate::slaspec::instructions::{
    core::{InstrBuilder, InstrFactory, InstrFamilyBuilder},
    pattern::{FieldType, ProtoField, ProtoPattern},
};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "NOP32",
        "32-bit Slot Nop",
        "mnop",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sigH", FieldType::Mask(0x18), 5),
                ProtoField::new("x3", FieldType::Mask(0x003), 11),
            ]),
            ProtoPattern::new(vec![ProtoField::new("sigL", FieldType::Mask(0x1800), 16)]),
        ],
    );

    ifam.add_instrs(&NOPFactory());

    ifam
}

struct NOPFactory();

impl InstrFactory for NOPFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![InstrBuilder::new(ifam).name("NOP32")]
    }
}
