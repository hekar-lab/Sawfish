mod cmplxmac;
mod tradmac;

use cmplxmac::CmplxMacFactory;
use tradmac::TradMacFactory;

use crate::slaspec::instructions::core::InstrFamilyBuilder;
use crate::slaspec::instructions::instr32::common32::Mmode;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "Dsp32Mac",
        "Multiply Accumulate",
        "dmc",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sigDsp", FieldType::Mask(0xc), 4),
                ProtoField::new("m", FieldType::Blank, 1),
                ProtoField::new("sig", FieldType::Mask(0x0), 2),
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
    ifam.add_id_instrs("Cplx", &CmplxMacFactory());

    let mmod0 = Mmode::mmod0();
    let mmod1 = Mmode::mmod1();
    let mmode = Mmode::mmode();

    for mode in mmod0 {
        ifam.add_id_instrs(
            &format!("TradM0{:?}", mode),
            &TradMacFactory::new(false, false, mode),
        );
    }

    for mode in mmod1 {
        ifam.add_id_instrs(
            &format!("TradM1{:?}", mode),
            &TradMacFactory::new(true, false, mode),
        );
    }

    for mode in mmode {
        ifam.add_id_instrs(
            &format!("TradME{:?}", mode),
            &TradMacFactory::new(true, true, mode),
        );
    }

    ifam
}
