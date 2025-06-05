use itertools::Itertools;

use super::common::*;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

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

    fn display(&self, acc: bool, neg_shft: bool) -> String {
        let acc_str = if acc { "A1" } else { "A0" };

        if *self == Sop::RotShft {
            format!("{acc_str} = ROT {acc_str} BY {{imm}}")
        } else {
            let imm_str = if neg_shft { "{$negImm5}" } else { "{imm5}" };
            let op_str = if neg_shft {
                if self.arithm() { ">>>" } else { ">>" }
            } else {
                if self.arithm() { "<<<" } else { "<<" }
            };

            format!("{acc_str} = {acc_str} {op_str} {imm_str}")
        }
    }
}

pub struct ShiftAccFactory();

impl ShiftAccFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop, acc: bool, neg_shft: bool) -> InstrBuilder {
        let acc_id = if acc { "A1" } else { "A0" };
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
            .display(sop.display(acc, neg_shft))
            .set_field_type("sopc", FieldType::Mask(0x3))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(acc as u16))
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
                    b_reg(acc_id),
                    b_reg(acc_id),
                    shft_var.clone(),
                    5,
                    &sop.name(),
                )
            } else {
                shift(
                    b_reg(acc_id),
                    b_reg(acc_id),
                    shft_var.clone(),
                    5,
                    sop.arithm(),
                    false,
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

impl InstrFactory for ShiftAccFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let shft_instr: Vec<InstrBuilder> = [Sop::AShft, Sop::LShft]
            .into_iter()
            .cartesian_product([false, true])
            .cartesian_product([false, true])
            .map(|((sop, acc), neg_shft)| Self::base_instr(ifam, sop, acc, neg_shft))
            .collect();

        let rot_instr: Vec<InstrBuilder> = [false, true]
            .into_iter()
            .map(|acc| Self::base_instr(ifam, Sop::RotShft, acc, false))
            .collect();

        vec![shft_instr, rot_instr].concat()
    }
}
