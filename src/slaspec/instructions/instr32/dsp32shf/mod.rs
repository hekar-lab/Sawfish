mod align;
mod bitmux;
mod bxor;
mod common;
mod deposit;
mod expadj;
mod extract;
mod packvec;
mod shift16;
mod shift16vec;
mod shift32;
mod shiftacc;
mod signbits;
mod vitmax;

use align::AlignFactory;
use bitmux::BitMuxFactory;
use bxor::BXORFactory;
use deposit::DepositFactory;
use expadj::ExpAdjFactory;
use extract::ExtractFactory;
use packvec::PackVecFactory;
use shift16::Shift16Factory;
use shift16vec::Shift16VecFactory;
use shift32::Shift32Factory;
use shiftacc::ShiftAccFactory;
use signbits::SignBitsFactory;
use vitmax::VitMaxFactory;

use crate::slaspec::instructions::core::InstrFamilyBuilder;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "Dsp32Shf",
        "Shift",
        "dsh",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sigDsp", FieldType::Mask(0xc), 4),
                ProtoField::new("m", FieldType::Blank, 1),
                ProtoField::new("sig", FieldType::Mask(0xc), 4),
                ProtoField::new("dead", FieldType::Blank, 2),
                ProtoField::new("sopc", FieldType::Blank, 5),
            ]),
            ProtoPattern::new(vec![
                ProtoField::new("sop", FieldType::Blank, 2),
                ProtoField::new("hls", FieldType::Blank, 2),
                ProtoField::new("dst", FieldType::Blank, 3),
                ProtoField::new("mask3", FieldType::Mask(0x0), 3),
                ProtoField::new("src0", FieldType::Blank, 3),
                ProtoField::new("src1", FieldType::Blank, 3),
            ]),
        ],
    );

    ifam.set_multi(true);
    ifam.add_instrs(&Shift16Factory());
    ifam.add_instrs(&Shift16VecFactory());
    ifam.add_instrs(&Shift32Factory());
    ifam.add_instrs(&ShiftAccFactory());
    ifam.add_instrs(&PackVecFactory());
    ifam.add_instrs(&SignBitsFactory());
    ifam.add_instrs(&ExpAdjFactory());
    ifam.add_instrs(&BitMuxFactory());
    ifam.add_instrs(&VitMaxFactory());
    ifam.add_instrs(&ExtractFactory());
    ifam.add_instrs(&DepositFactory());
    ifam.add_instrs(&BXORFactory());
    ifam.add_instrs(&AlignFactory());

    ifam
}
