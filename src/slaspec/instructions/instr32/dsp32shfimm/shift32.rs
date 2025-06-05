use itertools::Itertools;

use super::common::*;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

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

    fn display(&self, neg_shft: bool) -> String {
        if *self == Sop::RotShft {
            format!("{{dst}} = ROT {{src}} BY {{imm}}")
        } else {
            let imm_str = if neg_shft { "{$negImm5}" } else { "{imm5}" };
            let sat_str = if self.sat() { " (S)" } else { "" };
            let op_str = if neg_shft {
                if self.arithm() { ">>>" } else { ">>" }
            } else {
                if self.arithm() { "<<<" } else { "<<" }
            };

            format!("{{dst}} = {{src}} {op_str} {imm_str}{sat_str}")
        }
    }
}

pub struct Shift32Factory();

impl Shift32Factory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop, neg_shft: bool) -> InstrBuilder {
        let shft_var = if sop == Sop::RotShft {
            e_rfield("imm")
        } else {
            if neg_shft {
                b_var("negImm5")
            } else {
                e_rfield("imm5")
            }
        };

        let mut instr = InstrBuilder::new(ifam)
            .name(sop.name())
            .display(sop.display(neg_shft))
            .set_field_type("sopc", FieldType::Mask(0x2))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src", FieldType::Variable(RegisterSet::DReg))
            .add_action_opt(if neg_shft & (sop != Sop::RotShft) {
                Some(e_copy(
                    b_var("negImm5"),
                    e_sub(b_num(0x20), e_rfield("imm5")),
                ))
            } else {
                None
            })
            .add_pcode(if sop == Sop::RotShft {
                rot(
                    e_rfield("dst"),
                    e_rfield("src"),
                    shft_var.clone(),
                    4,
                    &sop.name(),
                )
            } else {
                shift(
                    e_rfield("dst"),
                    e_rfield("src"),
                    shft_var.clone(),
                    4,
                    sop.arithm(),
                    sop.sat(),
                    &sop.name(),
                )
            });

        if sop != Sop::RotShft {
            instr = instr.split_field(
                "imm",
                ProtoPattern::new(vec![
                    ProtoField::new("immSig1", FieldType::Mask(neg_shft as u16), 1),
                    ProtoField::new("imm5", FieldType::UImmVal, 5),
                ]),
            )
        } else {
            instr = instr.set_field_type("imm", FieldType::SImmVal)
        }

        instr
    }
}

impl InstrFactory for Shift32Factory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let shft_instr: Vec<InstrBuilder> = [Sop::AShft, Sop::AShftS, Sop::LShft]
            .into_iter()
            .cartesian_product([false, true])
            .map(|(sop, neg_shft)| Self::base_instr(ifam, sop, neg_shft))
            .collect();

        let rot_instr: Vec<InstrBuilder> = vec![Self::base_instr(ifam, Sop::RotShft, false)];

        vec![shft_instr, rot_instr].concat()
    }
}
