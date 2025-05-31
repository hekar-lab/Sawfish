use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "LdStII",
        "Load/Store indexed with small immediate offset",
        "lsi",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x5), 3),
            ProtoField::new("w", FieldType::Blank, 1),
            ProtoField::new("op", FieldType::Blank, 2),
            ProtoField::new("off", FieldType::UImmVal, 4),
            ProtoField::new("ptr", FieldType::Variable(RegisterSet::PReg), 3),
            ProtoField::new("reg", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&LdStImmFactory());

    ifam
}

#[derive(Debug, Clone, Copy)]
enum Op {
    Data = 0,
    WordZ = 1,
    WordX = 2,
    Ptr = 3,
}

impl Op {
    fn size(&self) -> usize {
        match self {
            Op::WordZ | Op::WordX => 2,
            _ => 4,
        }
    }

    fn name_addr(&self) -> String {
        match self {
            Op::Data | Op::Ptr => "M32bit",
            Op::WordZ | Op::WordX => "M16bit",
        }
        .to_string()
    }

    fn name_reg(&self) -> String {
        match self {
            Op::Ptr => "Preg",
            _ => "Dreg",
        }
        .to_string()
    }

    fn display(&self) -> String {
        match self {
            Op::WordZ => " (Z)",
            Op::WordX => " (X)",
            _ => "",
        }
        .to_string()
    }

    fn regset(&self) -> RegisterSet {
        match self {
            Op::Ptr => RegisterSet::PReg,
            _ => RegisterSet::DReg,
        }
    }

    fn expr_addr(&self, expr: Expr) -> Expr {
        match self {
            Op::WordZ => e_zext(expr),
            Op::WordX => e_sext(expr),
            _ => expr,
        }
    }

    fn expr_reg(&self, expr: Expr) -> Expr {
        match self {
            Op::WordZ | Op::WordX => b_size(expr, 2),
            _ => expr,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum LdSt {
    Load = 0,
    Store = 1,
}

impl LdSt {
    fn name(&self, op: Op) -> String {
        let addr = op.name_addr();
        let reg = op.name_reg();
        let size = op.size();
        match self {
            LdSt::Load => format!("Ld{}To{}", addr, reg),
            LdSt::Store => format!("St{}{}To{}", reg, if size == 2 { "L" } else { "" }, addr),
        }
    }

    fn display(&self, op: Op) -> String {
        let size = op.size();
        match self {
            LdSt::Load => format!(
                "{{reg}} = {}[{{ptr}} + {{$imm}}]{}",
                if size == 2 { "W" } else { "" },
                op.display()
            ),
            LdSt::Store => format!(
                "{}[{{ptr}} + {{$imm}}] = {{reg}}",
                if size == 2 { "W" } else { "" }
            ),
        }
    }

    fn expr(&self, op: Op) -> Expr {
        let reg = e_rfield("reg");
        let addr = e_ptr(b_grp(e_add(e_rfield("ptr"), b_var("imm"))), op.size());
        match self {
            LdSt::Load => e_copy(reg.clone(), op.expr_addr(addr.clone())),
            LdSt::Store => e_copy(addr, op.expr_reg(reg)),
        }
    }
}

struct LdStImmFactory();

impl LdStImmFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, w: LdSt, op: Op) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(&w.name(op))
            .display(w.display(op))
            .set_field_type("w", FieldType::Mask(w as u16))
            .set_field_type("op", FieldType::Mask(op as u16))
            .set_field_type("reg", FieldType::Variable(op.regset()))
            .add_action(e_copy(
                b_var("imm"),
                e_mult(e_field("off"), b_num(op.size() as i128)),
            ))
            .add_pcode(w.expr(op))
    }
}

impl InstrFactory for LdStImmFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::base_instr(ifam, LdSt::Load, Op::Data),
            Self::base_instr(ifam, LdSt::Load, Op::WordZ),
            Self::base_instr(ifam, LdSt::Load, Op::WordX),
            Self::base_instr(ifam, LdSt::Load, Op::Ptr),
            Self::base_instr(ifam, LdSt::Store, Op::Data),
            Self::base_instr(ifam, LdSt::Store, Op::WordZ),
            Self::base_instr(ifam, LdSt::Store, Op::Ptr),
        ]
    }
}
