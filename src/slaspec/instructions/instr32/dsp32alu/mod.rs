mod accop;
mod addsub;
mod addsubac0;
mod addsubaccext;
mod addsubvec;
mod maxmin;
mod misc;
mod mvacc;
mod mvaccreg;
mod mvregacc;
mod packop;
mod search;
mod videoop;
mod vidopmisc;

use accop::AccOpFactory;
use addsub::AddSubFactory;
use addsubac0::AddSubAc0Factory;
use addsubaccext::AddSubAccExtFactory;
use addsubvec::AddSubVecFactory;
use maxmin::MMANFactory;
use misc::MiscFactory;
use mvacc::MvAccFactory;
use mvaccreg::MvAccRegFactory;
use mvregacc::MvRegAccFactory;
use packop::PackOpFactory;
use search::SearchFactory;
use videoop::VideoOpFactory;
use vidopmisc::VidOpMiscFactory;

use crate::slaspec::instructions::core::InstrFamilyBuilder;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "Dsp32Alu",
        "ALU Operations",
        "dau",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sigDsp", FieldType::Mask(0xc), 4),
                ProtoField::new("m", FieldType::Blank, 1),
                ProtoField::new("sig", FieldType::Mask(0x2), 2),
                ProtoField::new("dead", FieldType::Blank, 3),
                ProtoField::new("hl", FieldType::Blank, 1),
                ProtoField::new("aopc", FieldType::Blank, 5),
            ]),
            ProtoPattern::new(vec![
                ProtoField::new("aop", FieldType::Blank, 2),
                ProtoField::new("s", FieldType::Blank, 1),
                ProtoField::new("x", FieldType::Blank, 1),
                ProtoField::new("dst0", FieldType::Blank, 3),
                ProtoField::new("dst1", FieldType::Blank, 3),
                ProtoField::new("src0", FieldType::Blank, 3),
                ProtoField::new("src1", FieldType::Blank, 3),
            ]),
        ],
    );

    ifam.set_multi(true);
    ifam.add_pcodeop("disalgnexcpt");

    ifam.add_instrs(&AddSubVecFactory());
    ifam.add_instrs(&AddSubFactory());
    ifam.add_instrs(&MMANFactory());
    ifam.add_instrs(&MvAccFactory());
    ifam.add_instrs(&MvRegAccFactory());
    ifam.add_instrs(&MvAccRegFactory());
    ifam.add_instrs(&AddSubAccExtFactory());
    ifam.add_instrs(&MiscFactory());
    ifam.add_instrs(&SearchFactory());
    ifam.add_instrs(&AccOpFactory());
    ifam.add_instrs(&VidOpMiscFactory());
    ifam.add_instrs(&VideoOpFactory());
    ifam.add_instrs(&PackOpFactory());
    ifam.add_instrs(&AddSubAc0Factory());

    ifam
}
