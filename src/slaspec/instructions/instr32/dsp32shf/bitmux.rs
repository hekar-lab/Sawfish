use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

pub struct BitMuxFactory();

impl BitMuxFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sop: bool) -> InstrBuilder {
        let shft_op = if sop { e_lshft } else { e_rshft };
        let src_ord = if sop {
            ("src0", "src1")
        } else {
            ("src1", "src0")
        };

        fn get_bit(src: &str, sop: bool) -> Expr {
            b_grp(if sop {
                e_rshft(e_rfield(src), b_num(31))
            } else {
                e_bit_and(e_rfield(src), b_num(1))
            })
        }

        InstrBuilder::new(ifam)
            .name("BitMux")
            .display(format!(
                "BITMUX ({{src1}}, {{src0}}, A0) ({})",
                if sop { "ASL" } else { "ASR" }
            ))
            .set_field_type("sopc", FieldType::Mask(0x8))
            .set_field_type("sop", FieldType::Mask(sop as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(cs_assign_by(shft_op, b_reg("A0"), b_num(1)))
            .add_pcode(cs_assign_by(e_bit_or, b_reg("A0"), get_bit(src_ord.0, sop)))
            .add_pcode(cs_assign_by(shft_op, e_rfield(src_ord.0), b_num(1)))
            .add_pcode(cs_assign_by(shft_op, b_reg("A0"), b_num(1)))
            .add_pcode(cs_assign_by(e_bit_or, b_reg("A0"), get_bit(src_ord.1, sop)))
            .add_pcode(cs_assign_by(shft_op, e_rfield(src_ord.1), b_num(1)))
    }
}

impl InstrFactory for BitMuxFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [false, true]
            .into_iter()
            .map(|sop| Self::base_instr(ifam, sop))
            .collect()
    }
}
