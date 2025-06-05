use super::common::*;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sop {
    AShft = 0x0,
    AShftS = 0x1,
    LShft = 0x2,
    RotShft = 0x3,
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
            Self::AShft | Self::AShftS => "AShift32",
            Self::LShft => "LShift32",
            Self::RotShft => "Rot32",
        }
    }

    fn display(&self) -> String {
        let sat_str = if self.sat() { " (S)" } else { "" };
        let shft_str = match self {
            Self::AShft | Self::AShftS => "ASHIFT",
            Self::LShft => "LSHIFT",
            Self::RotShft => "ROT",
        };

        format!("{{dst}} = {shft_str} {{src0}} BY {{src1}}{sat_str}")
    }
}

pub struct Shift32Factory();

impl Shift32Factory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(sop.name())
            .display(sop.display())
            .set_field_type("sopc", FieldType::Mask(0x2))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DRegL))
            .add_pcode(if sop == Sop::RotShft {
                rot(e_rfield("dst"), e_rfield("src0"), 4, &sop.name())
            } else {
                shift(
                    e_rfield("dst"),
                    e_rfield("src0"),
                    4,
                    sop.arithm(),
                    sop.sat(),
                    &sop.name(),
                )
            })
    }
}

impl InstrFactory for Shift32Factory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [Sop::AShft, Sop::AShftS, Sop::LShft, Sop::RotShft]
            .into_iter()
            .map(|sop| Self::base_instr(ifam, sop))
            .collect()
    }
}
