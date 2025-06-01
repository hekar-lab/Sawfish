mod addsub;
mod addsubvec;

use addsub::AddSubFactory;
use addsubvec::AddSubVecFactory;

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

    ifam.add_instrs(&AddSubVecFactory());
    ifam.add_instrs(&AddSubFactory());

    ifam
}
