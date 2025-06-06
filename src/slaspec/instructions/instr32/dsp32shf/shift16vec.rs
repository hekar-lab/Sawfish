use super::common::*;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sop {
    AShft = 0x0,
    AShftS = 0x1,
    LShft = 0x2,
}

impl Sop {
    fn sat(&self) -> bool {
        match self {
            Self::AShftS => true,
            _ => false,
        }
    }

    fn arithm(&self) -> bool {
        match self {
            Self::LShft => false,
            _ => true,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::AShft | Self::AShftS => "AShift16Vec",
            Self::LShft => "LShift16Vec",
        }
    }

    fn display(&self) -> String {
        let sat_str = if self.sat() { "V, S" } else { "V" };
        let shft_str = if *self == Sop::LShft {
            "LSHIFT"
        } else {
            "ASHIFT"
        };
        format!("{{dst}} = {shft_str} {{src0}} BY {{src1}} ({sat_str})")
    }
}

pub struct Shift16VecFactory();

impl Shift16VecFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(sop.name())
            .display(sop.display())
            .set_field_type("sopc", FieldType::Mask(0x1))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DRegL))
            .add_pcode(e_copy(e_local("src_vecL", 2), b_size(e_rfield("src0"), 2)))
            .add_pcode(e_copy(e_local("src_vecH", 2), b_trunc(e_rfield("src0"), 2)))
            .add_pcode(e_local("res_vecL", 2))
            .add_pcode(e_local("res_vecH", 2))
            .add_pcode(shift(
                b_var("res_vecL"),
                b_var("src_vecL"),
                2,
                sop.arithm(),
                sop.sat(),
                &format!("{}L", sop.name()),
            ))
            .add_pcode(shift(
                b_var("res_vecH"),
                b_var("src_vecH"),
                2,
                sop.arithm(),
                sop.sat(),
                &format!("{}H", sop.name()),
            ))
            .add_pcode(e_copy(
                e_rfield("dst"),
                e_bit_or(
                    e_lshft(e_zext(b_var("res_vecH")), b_num(16)),
                    e_zext(b_var("res_vecL")),
                ),
            ))
    }
}

impl InstrFactory for Shift16VecFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [Sop::AShft, Sop::AShftS, Sop::LShft]
            .into_iter()
            .map(|sop| Self::base_instr(ifam, sop))
            .collect()
    }
}
