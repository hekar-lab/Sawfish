use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::{
    cs_assign_by, e_and, e_bit_and, e_bit_not, e_bit_or, e_bit_xor, e_copy, e_lshft, e_or, e_rshft,
    e_xor,
};
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "CC2Stat",
        "Copy CC conditional bit, from status",
        "ccs",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x03), 8),
            ProtoField::new("d", FieldType::Blank, 1),
            ProtoField::new("op", FieldType::Blank, 2),
            ProtoField::new("cbit", FieldType::UImmVal, 5),
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

    fn to_s2c_op(&self) -> Option<fn(Expr, Expr) -> Expr> {
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
            Expr::trunc(
                Expr::grp(e_rshft(Expr::reg("ASTAT"), Expr::field("cbit"))),
                1,
            ),
            Expr::num(0x1),
        )
    }

    fn base_instr(ifam: &InstrFamilyBuilder, ccop: CCOp) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("MvToCC_STAT")
            .set_field_type("op", FieldType::Mask(ccop as u16))
            .set_field_type("d", FieldType::Mask(0x0))
            .display(format!("CC {} {{cbit}}", ccop.to_str()))
            .add_pcode(e_copy(
                Expr::reg("CC"),
                match ccop.to_s2c_op() {
                    Some(op) => op(Expr::reg("CC"), Expr::grp(Self::astat_flag())),
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
        Expr::grp(e_lshft(
            Expr::macp("zext", Expr::reg("CC")),
            Expr::field("cbit"),
        ))
    }

    fn cc_mask() -> Expr {
        Expr::grp(e_lshft(Expr::num(0x1), Expr::field("cbit")))
    }

    fn base_instr(ifam: &InstrFamilyBuilder, ccop: CCOp) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("CCToStat16")
            .set_field_type("op", FieldType::Mask(ccop as u16))
            .set_field_type("d", FieldType::Mask(0x1))
            .display(format!("{{cbit}} {} CC", ccop.to_str()))
            .add_pcode(match ccop {
                CCOp::Set => e_copy(
                    Expr::reg("ASTAT"),
                    e_bit_or(
                        Expr::grp(e_bit_and(Expr::reg("ASTAT"), e_bit_not(Self::cc_mask()))),
                        Self::cc_flag(),
                    ),
                ),
                CCOp::Or => cs_assign_by(e_bit_or, Expr::reg("ASTAT"), Self::cc_flag()),
                CCOp::And => e_copy(
                    Expr::reg("ASTAT"),
                    e_bit_and(Expr::reg("ASTAT"), e_bit_not(Self::cc_flag())),
                ),
                CCOp::Xor => cs_assign_by(e_bit_xor, Expr::reg("ASTAT"), Self::cc_flag()),
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
