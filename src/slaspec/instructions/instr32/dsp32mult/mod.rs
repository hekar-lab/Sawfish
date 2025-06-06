mod mac32;
mod mult16;
mod mult32;

use mac32::Mac32Factory;
use mult16::Mult16Factory;
use mult32::Mult32Factory;

use crate::slaspec::instructions::core::InstrFamilyBuilder;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "Dsp32Mult",
        "Multiply with 3 operands",
        "dmt",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sigDsp", FieldType::Mask(0xc), 4),
                ProtoField::new("m", FieldType::Blank, 1),
                ProtoField::new("sig", FieldType::Mask(0x1), 2),
                ProtoField::new("mmod", FieldType::Blank, 4),
                ProtoField::new("mm", FieldType::Blank, 1),
                ProtoField::new("p", FieldType::Blank, 1),
                ProtoField::new("w1", FieldType::Blank, 1),
                ProtoField::new("op1", FieldType::Blank, 2),
            ]),
            ProtoPattern::new(vec![
                ProtoField::new("h01", FieldType::Blank, 1),
                ProtoField::new("h11", FieldType::Blank, 1),
                ProtoField::new("w0", FieldType::Blank, 1),
                ProtoField::new("op0", FieldType::Blank, 2),
                ProtoField::new("h00", FieldType::Blank, 1),
                ProtoField::new("h10", FieldType::Blank, 1),
                ProtoField::new("dst", FieldType::Blank, 3),
                ProtoField::new("src0", FieldType::Blank, 3),
                ProtoField::new("src1", FieldType::Blank, 3),
            ]),
        ],
    );

    ifam.set_multi(true);
    ifam.add_instrs(&Mult16Factory());
    ifam.add_instrs(&Mac32Factory());
    ifam.add_instrs(&Mult32Factory());

    ifam
}
