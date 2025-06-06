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

    fn display(&self, neg_shft: bool) -> String {
        let sat_str = if self.sat() { "V, S" } else { "V" };
        let op_str = if neg_shft {
            if self.arithm() { ">>>" } else { ">>" }
        } else {
            if self.arithm() { "<<<" } else { "<<" }
        };
        let imm_str = if neg_shft { "{$negImm4}" } else { "{imm4}" };

        format!("{{dst}} = {{src}} {op_str} {imm_str}{sat_str}")
    }
}

pub struct Shift16VecFactory();

impl Shift16VecFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop, neg_shft: bool) -> InstrBuilder {
        let shft_var = if neg_shft {
            b_var("negImm4")
        } else {
            e_rfield("imm4")
        };

        InstrBuilder::new(ifam)
            .name(sop.name())
            .display(sop.display(neg_shft))
            .set_field_type("sopc", FieldType::Mask(0x1))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src", FieldType::Variable(RegisterSet::DReg))
            .split_field(
                "imm",
                ProtoPattern::new(vec![
                    ProtoField::new("immSig2", FieldType::Mask(0x3 * neg_shft as u16), 2),
                    ProtoField::new("imm4", FieldType::UImmVal, 4),
                ]),
            )
            .add_action_opt(if neg_shft {
                Some(e_copy(
                    b_var("negImm4"),
                    e_sub(b_num(0x10), e_rfield("imm4")),
                ))
            } else {
                None
            })
            .add_pcode(e_copy(e_local("src_vecL", 2), b_size(e_rfield("src"), 2)))
            .add_pcode(e_copy(e_local("src_vecH", 2), b_trunc(e_rfield("src"), 2)))
            .add_pcode(e_local("res_vecL", 2))
            .add_pcode(e_local("res_vecH", 2))
            .add_pcode(shift(
                b_var("res_vecL"),
                b_var("src_vecL"),
                shft_var.clone(),
                2,
                sop.arithm(),
                sop.sat(),
                &format!("{}L", sop.name()),
            ))
            .add_pcode(shift(
                b_var("res_vecH"),
                b_var("src_vecH"),
                shft_var,
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
            .cartesian_product([false, true])
            .map(|(sop, neg_shft)| Self::base_instr(ifam, sop, neg_shft))
            .collect()
    }
}
