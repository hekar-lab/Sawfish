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
            Self::AShft | Self::AShftS => "AShift16",
            Self::LShft => "LShift16",
        }
    }

    fn display(&self, neg_shft: bool) -> String {
        let sat_str = if self.sat() { " (S)" } else { "" };
        let op_str = if neg_shft {
            if self.arithm() { ">>>" } else { ">>" }
        } else {
            if self.arithm() { "<<<" } else { "<<" }
        };
        let imm_str = if neg_shft { "{$negImm4}" } else { "{imm4}" };

        format!("{{dst}} = {{src}} {op_str} {imm_str}{sat_str}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hls {
    dst: bool,
    src: bool,
}

impl Hls {
    fn new(dst: bool, src: bool) -> Hls {
        Hls { dst, src }
    }

    fn mask(&self) -> u16 {
        2 * self.dst as u16 + self.src as u16
    }

    fn reg(hl: bool) -> RegisterSet {
        if hl {
            RegisterSet::DRegH
        } else {
            RegisterSet::DRegL
        }
    }

    fn dst_reg(&self) -> RegisterSet {
        Self::reg(self.dst)
    }

    fn src_reg(&self) -> RegisterSet {
        Self::reg(self.src)
    }

    fn all() -> [Hls; 4] {
        [
            Hls::new(false, false),
            Hls::new(false, true),
            Hls::new(true, false),
            Hls::new(true, true),
        ]
    }
}

pub struct Shift16Factory();

impl Shift16Factory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: Sop, hls: Hls, neg_shft: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(sop.name())
            .display(sop.display(neg_shft))
            .set_field_type("sopc", FieldType::Mask(0x0))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(hls.mask()))
            .set_field_type("dst", FieldType::Variable(hls.dst_reg()))
            .set_field_type("src", FieldType::Variable(hls.src_reg()))
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
            .add_pcode(shift(
                e_rfield("dst"),
                e_rfield("src"),
                if neg_shft {
                    b_var("negImm4")
                } else {
                    e_rfield("imm4")
                },
                2,
                sop.arithm(),
                sop.sat(),
                &sop.name(),
            ))
    }
}

impl InstrFactory for Shift16Factory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [Sop::AShft, Sop::AShftS, Sop::LShft]
            .into_iter()
            .cartesian_product(Hls::all())
            .cartesian_product([false, true])
            .map(|((sop, hls), neg_shft)| Self::base_instr(ifam, sop, hls, neg_shft))
            .collect()
    }
}
