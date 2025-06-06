mod common;
mod shift16;
mod shift16vec;
mod shift32;
mod shiftacc;

use shift16::Shift16Factory;
use shift16vec::Shift16VecFactory;
use shift32::Shift32Factory;
use shiftacc::ShiftAccFactory;

use crate::slaspec::instructions::core::InstrFamilyBuilder;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "Dsp32ShfImm",
        "Shift Immediate",
        "dsi",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sigDsp", FieldType::Mask(0xc), 4),
                ProtoField::new("m", FieldType::Blank, 1),
                ProtoField::new("sig", FieldType::Mask(0xd), 4),
                ProtoField::new("dead", FieldType::Blank, 2),
                ProtoField::new("sopc", FieldType::Blank, 5),
            ]),
            ProtoPattern::new(vec![
                ProtoField::new("sop", FieldType::Blank, 2),
                ProtoField::new("hls", FieldType::Blank, 2),
                ProtoField::new("dst", FieldType::Blank, 3),
                ProtoField::new("imm", FieldType::Blank, 6),
                ProtoField::new("src", FieldType::Blank, 3),
            ]),
        ],
    );

    ifam.set_multi(true);
    ifam.add_instrs(&Shift16Factory());
    ifam.add_instrs(&Shift16VecFactory());
    ifam.add_instrs(&Shift32Factory());
    ifam.add_instrs(&ShiftAccFactory());

    ifam
}
