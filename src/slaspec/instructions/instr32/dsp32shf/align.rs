use std::fmt;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sop {
    ALIGN8 = 0x0,
    ALIGN16 = 0x1,
    ALIGN24 = 0x2,
}

impl Sop {
    fn shifts(&self) -> (i128, i128) {
        match self {
            Sop::ALIGN8 => (24, 8),
            Sop::ALIGN16 => (16, 16),
            Sop::ALIGN24 => (8, 24),
        }
    }
}

impl fmt::Display for Sop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
pub struct AlignFactory();

impl AlignFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop) -> InstrBuilder {
        let (hi_shft, lo_shft) = sop.shifts();
        InstrBuilder::new(ifam)
            .name("Align")
            .display(format!("{{dst}} = {sop} ({{src1}}, {{src0}})"))
            .set_field_type("sopc", FieldType::Mask(0xd))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_copy(
                e_rfield("dst"),
                e_bit_or(
                    b_grp(e_lshft(e_rfield("src1"), b_num(hi_shft))),
                    b_grp(e_rshft(e_rfield("src0"), b_num(lo_shft))),
                ),
            ))
    }
}

impl InstrFactory for AlignFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [Sop::ALIGN8, Sop::ALIGN16, Sop::ALIGN24]
            .into_iter()
            .map(|sop| Self::base_instr(ifam, sop))
            .collect()
    }
}
