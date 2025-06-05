use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sop {
    LL = 0x0,
    LH = 0x1,
    HL = 0x2,
    HH = 0x3,
}

impl Sop {
    fn lhs_reg(&self) -> RegisterSet {
        match self {
            Sop::LL | Sop::LH => RegisterSet::DRegL,
            Sop::HL | Sop::HH => RegisterSet::DRegH,
        }
    }

    fn rhs_reg(&self) -> RegisterSet {
        match self {
            Sop::LL | Sop::HL => RegisterSet::DRegL,
            Sop::LH | Sop::HH => RegisterSet::DRegH,
        }
    }
}

pub struct PackVecFactory();

impl PackVecFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("Pack16Vec")
            .display("{dst} = PACK ({src0}, {src1})".to_string())
            .set_field_type("sopc", FieldType::Mask(0x4))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(sop.lhs_reg()))
            .set_field_type("src1", FieldType::Variable(sop.rhs_reg()))
            .add_pcode(e_copy(
                e_rfield("dst"),
                e_bit_or(
                    b_grp(e_lshft(e_zext(e_rfield("src0")), b_num(16))),
                    e_zext(e_rfield("src1")),
                ),
            ))
    }
}

impl InstrFactory for PackVecFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [Sop::LL, Sop::LH, Sop::HL, Sop::HH]
            .into_iter()
            .map(|sop| Self::base_instr(ifam, sop))
            .collect()
    }
}
