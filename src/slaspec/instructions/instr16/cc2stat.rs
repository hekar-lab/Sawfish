use crate::slaspec::instructions::common::BinOp;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "CC2Stat",
        "Copy CC conditional bit, from status",
        "ccs",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x03), 8),
            ProtoField::new("d", FieldType::Blank, 1),
            ProtoField::new("op", FieldType::Blank, 2),
            ProtoField::new("cbit", FieldType::Variable(RegisterSet::CBIT), 5),
        ]),
    );

    ifam.add_instrs(&MvStatToCCFactory());
    ifam.add_instrs(&MvCCToStatFactory());

    ifam
}

#[derive(Debug, Clone, Copy)]
enum CCOp {
    Set = 0,
    Or = 1,
    And = 2,
    Xor = 3,
}

impl CCOp {
    fn to_str(&self) -> String {
        match self {
            Self::Set => "=",
            Self::Or => "|=",
            Self::And => "&=",
            Self::Xor => "^=",
        }
        .to_string()
    }

    fn to_s2c_op(&self) -> Option<BinOp> {
        match self {
            Self::Or => Some(e_or),
            Self::And => Some(e_and),
            Self::Xor => Some(e_xor),
            _ => None,
        }
    }
}

struct MvStatToCCFactory();

impl MvStatToCCFactory {
    fn astat_flag() -> Expr {
        e_bit_and(
            b_grp(e_rshft(
                b_size(b_reg("ASTAT"), 1),
                b_size(e_rfield("cbit"), 1),
            )),
            b_num(0x1),
        )
    }

    fn base_instr(ifam: &InstrFamilyBuilder, ccop: CCOp) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("MvToCC_STAT")
            .set_field_type("op", FieldType::Mask(ccop as u16))
            .set_field_type("d", FieldType::Mask(0x0))
            .display(format!("CC {} {{cbit}}", ccop.to_str()))
            .add_pcode(e_copy(
                b_reg("CC"),
                match ccop.to_s2c_op() {
                    Some(op) => op(b_reg("CC"), b_grp(Self::astat_flag())),
                    None => Self::astat_flag(),
                },
            ))
    }
}

impl InstrFactory for MvStatToCCFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut instrs = Vec::new();

        for op in [CCOp::Set, CCOp::Or, CCOp::And, CCOp::Xor] {
            instrs.push(Self::base_instr(ifam, op));
        }

        instrs
    }
}

struct MvCCToStatFactory();

impl MvCCToStatFactory {
    fn cc_flag() -> Expr {
        b_grp(e_lshft(e_zext(b_reg("CC")), e_rfield("cbit")))
    }

    fn cc_mask() -> Expr {
        b_grp(e_lshft(b_num(0x1), e_rfield("cbit")))
    }

    fn base_instr(ifam: &InstrFamilyBuilder, ccop: CCOp) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("CCToStat16")
            .set_field_type("op", FieldType::Mask(ccop as u16))
            .set_field_type("d", FieldType::Mask(0x1))
            .display(format!("{{cbit}} {} CC", ccop.to_str()))
            .add_pcode(match ccop {
                CCOp::Set => e_copy(
                    b_reg("ASTAT"),
                    e_bit_or(
                        b_grp(e_bit_and(b_reg("ASTAT"), e_bit_not(Self::cc_mask()))),
                        Self::cc_flag(),
                    ),
                ),
                CCOp::Or => cs_assign_by(e_bit_or, b_reg("ASTAT"), Self::cc_flag()),
                CCOp::And => e_copy(
                    b_reg("ASTAT"),
                    e_bit_and(b_reg("ASTAT"), e_bit_not(Self::cc_flag())),
                ),
                CCOp::Xor => cs_assign_by(e_bit_xor, b_reg("ASTAT"), Self::cc_flag()),
            })
    }
}

impl InstrFactory for MvCCToStatFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut instrs = Vec::new();

        for op in [CCOp::Set, CCOp::Or, CCOp::And, CCOp::Xor] {
            instrs.push(Self::base_instr(ifam, op));
        }

        instrs
    }
}
