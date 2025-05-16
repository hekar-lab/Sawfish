use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::{e_copy, e_eq, e_le, e_les, e_lt, e_lts};
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "CCFlag",
        "Set CC conditional bit",
        "cfg",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x01), 5),
            ProtoField::new("i", FieldType::Blank, 1),
            ProtoField::new("opc", FieldType::Blank, 3),
            ProtoField::new("g", FieldType::Blank, 1),
            ProtoField::new("x", FieldType::Blank, 3),
            ProtoField::new("y", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&CompRegFactory());

    ifam
}

#[derive(Debug, Clone, Copy)]
enum CompOp {
    Eq = 0x0,
    Lower = 0x1,
    LowerEq = 0x2,
    LowerU = 0x3,
    LowerEqU = 0x4,
}

impl CompOp {
    fn op_str(&self) -> String {
        match self {
            Self::Eq => "==",
            Self::Lower | Self::LowerU => "<",
            Self::LowerEq | Self::LowerEqU => "<=",
        }
        .to_string()
    }

    fn op_expr(&self) -> fn(Expr, Expr) -> Expr {
        match self {
            Self::Eq => e_eq,
            Self::Lower => e_lts,
            Self::LowerEq => e_les,
            Self::LowerU => e_lt,
            Self::LowerEqU => e_le,
        }
    }

    fn is_unsigned(&self) -> bool {
        match self {
            Self::LowerU | Self::LowerEqU => true,
            _ => false,
        }
    }
}

struct CompRegFactory();

impl CompRegFactory {
    fn reg_instr(ifam: &InstrFamilyBuilder, imm: bool, preg: bool, op: CompOp) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(if preg { "CCFlagP" } else { "CompRegisters" })
            .display(format!(
                "CC = {{x}} {} {{y}}{}",
                op.op_str(),
                if op.is_unsigned() { " (IU)" } else { "" }
            ))
            .set_field_type("i", FieldType::Mask(if imm { 0x1 } else { 0x0 }))
            .set_field_type("opc", FieldType::Mask(op as u16))
            .set_field_type(
                "x",
                FieldType::Variable(if preg {
                    RegisterSet::PReg
                } else {
                    RegisterSet::DReg
                }),
            )
            .set_field_type(
                "y",
                if imm {
                    if op.is_unsigned() {
                        FieldType::UImmVal
                    } else {
                        FieldType::SImmVal
                    }
                } else {
                    FieldType::Variable(if preg {
                        RegisterSet::PReg
                    } else {
                        RegisterSet::DReg
                    })
                },
            )
            .add_pcode(e_copy(
                Expr::reg("CC"),
                op.op_expr()(Expr::field("x"), Expr::field("y")),
            ))
    }

    fn acc_instr(ifam: &InstrFamilyBuilder, opc: u16, op: CompOp) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("CompAccumulators")
            .display(format!("CC = A0 {} A1", op.op_str()))
            .set_field_type("opc", FieldType::Mask(opc))
            .add_pcode(e_copy(
                Expr::reg("CC"),
                op.op_expr()(Expr::reg("A0"), Expr::reg("A1")),
            ))
    }
}

impl InstrFactory for CompRegFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let bool_param = [false, true];
        let op_param = [
            CompOp::Eq,
            CompOp::Lower,
            CompOp::LowerEq,
            CompOp::LowerU,
            CompOp::LowerEqU,
        ];
        let mut reg_instrs: Vec<InstrBuilder> = bool_param
            .iter()
            .cartesian_product(bool_param.iter())
            .cartesian_product(op_param.iter())
            .map(|((imm, preg), op)| Self::reg_instr(ifam, *imm, *preg, *op))
            .collect();

        let acc_params = [
            (0x5, CompOp::Eq),
            (0x6, CompOp::Lower),
            (0x7, CompOp::LowerEq),
        ];
        let mut acc_instrs = Vec::new();

        for (opc, op) in acc_params {
            acc_instrs.push(Self::acc_instr(ifam, opc, op));
        }

        reg_instrs.append(&mut acc_instrs);
        reg_instrs
    }
}
