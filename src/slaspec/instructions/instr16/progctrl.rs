use crate::slaspec::instructions::{
    core::{InstrBuilder, InstrFamilyBuilder},
    pattern::{FieldType, ProtoField, ProtoPattern},
    util::quote,
};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "ProgCtrl",
        "Basic Program Sequencer Control Functions",
        "pgc",
        ProtoPattern {
            fields: vec![
                ProtoField::new("sig", FieldType::Mask(0x00), 8),
                ProtoField::new("opc", FieldType::Blank, 4),
                ProtoField::new("reg", FieldType::Blank, 4),
            ],
        },
    );

    let retregs = "SIXNE";
    let mut regmask = 0x0;
    for c in retregs.chars() {
        ifam.add_instr(instr_rt(&ifam, c, regmask));
        regmask += 1;
    }

    ifam
}

fn instr_rt(ifam: &InstrFamilyBuilder, retreg: char, regmask: u16) -> InstrBuilder {
    let mut instr = InstrBuilder::new("Return", ifam);

    instr.set_field_type("opc", FieldType::Mask(0x01));
    instr.set_field_type("reg", FieldType::Mask(regmask));
    instr.display = quote(&format!("RET{retreg}"));

    instr
}
