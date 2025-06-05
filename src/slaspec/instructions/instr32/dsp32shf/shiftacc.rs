use itertools::Itertools;

use super::common::*;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sop {
    AShft = 0x0,
    LShft = 0x1,
    RotShft = 0x2,
}

impl Sop {
    fn arithm(&self) -> bool {
        match self {
            Self::LShft => false,
            _ => true,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::AShft => "AShiftAcc",
            Self::LShft => "LShiftAcc",
            Self::RotShft => "ShiftRotAcc",
        }
    }

    fn display(&self, acc: bool) -> String {
        let acc_str = if acc { "A1" } else { "A0" };
        let shft_str = match self {
            Self::AShft => "ASHIFT",
            Self::LShft => "LSHIFT",
            Self::RotShft => "ROT",
        };

        format!("{acc_str} = {shft_str} {acc_str} BY {{src1}}")
    }
}

pub struct ShiftAccFactory();

impl ShiftAccFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop, acc: bool) -> InstrBuilder {
        let acc_id = if acc { "A1" } else { "A0" };
        InstrBuilder::new(ifam)
            .name(sop.name())
            .display(sop.display(acc))
            .set_field_type("sopc", FieldType::Mask(0x3))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(acc as u16))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DRegL))
            .add_pcode(if sop == Sop::RotShft {
                rot(b_reg(acc_id), b_reg(acc_id), 5, &sop.name())
            } else {
                shift(
                    b_reg(acc_id),
                    b_reg(acc_id),
                    5,
                    sop.arithm(),
                    false,
                    &sop.name(),
                )
            })
    }
}

impl InstrFactory for ShiftAccFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [Sop::AShft, Sop::LShft, Sop::RotShft]
            .into_iter()
            .cartesian_product([false, true])
            .map(|(sop, acc)| Self::base_instr(ifam, sop, acc))
            .collect()
    }
}
