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
                ProtoField::new("sigDsp", FieldType::Mask(0xc), 4),
                ProtoField::new("m", FieldType::Blank, 1),
                ProtoField::new("sigH", FieldType::Mask(0x003), 11),
            ]),
            ProtoPattern::new(vec![ProtoField::new("sigL", FieldType::Mask(0x1800), 16)]),
        ],
    );

    ifam.set_multi(true);
    ifam.add_instrs(&NOPFactory());

    ifam
}

struct NOPFactory();

impl InstrFactory for NOPFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![InstrBuilder::new(ifam).name("NOP32")]
    }
}
