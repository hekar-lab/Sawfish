use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "DspLdSt",
        "Load/Store",
        "dls",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x27), 6),
            ProtoField::new("w", FieldType::Blank, 1),
            ProtoField::new("aop", FieldType::Blank, 2),
            ProtoField::new("i", FieldType::Variable(RegisterSet::IReg), 2),
            ProtoField::new("m", FieldType::Blank, 2),
            ProtoField::new("reg", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&DspLdStFactory());

    ifam
}

#[derive(Debug, Clone, Copy)]
enum AddrOp {
    Inc = 0,
    Dec = 1,
    None = 2,
    IncM = 3,
}

impl AddrOp {
    fn display(&self) -> String {
        format!(
            "[{{i}}{}]",
            match self {
                Self::Inc => "++",
                Self::Dec => "--",
                Self::None => "",
                Self::IncM => "++{m}",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Load = 0,
    Store = 1,
}

impl Op {
    fn name(&self, addr_str: &str, reg_str: &str) -> String {
        match self {
            Op::Load => format!("Ld{}To{}", addr_str, reg_str),
            Op::Store => format!("St{}To{}", reg_str, addr_str),
        }
    }

    fn display(&self, addr_str: &str) -> String {
        match self {
            Op::Load => format!("{{reg}} = {}", addr_str),
            Op::Store => format!("{} = {{reg}}", addr_str),
        }
    }

    fn expr(&self, addr_expr: Expr, reg_expr: Expr) -> Expr {
        match self {
            Op::Load => e_copy(reg_expr, addr_expr),
            Op::Store => e_copy(addr_expr, reg_expr),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MOp {
    Dreg = 0,
    DregL = 1,
    DregH = 2,
    Mreg = 3,
}

impl MOp {
    fn addr_str(&self) -> String {
        match self {
            Self::Dreg | Self::Mreg => "M32bit",
            Self::DregL | Self::DregH => "M16bit",
        }
        .to_string()
    }

    fn reg_str(&self) -> String {
        match self {
            Self::Dreg | Self::Mreg => "Dreg",
            Self::DregL => "DregL",
            Self::DregH => "DregH",
        }
        .to_string()
    }

    fn display(&self) -> String {
        match self {
            Self::Dreg | Self::Mreg => "",
            Self::DregL | Self::DregH => "W",
        }
        .to_string()
    }

    fn reg(&self) -> RegisterSet {
        match self {
            Self::Dreg | Self::Mreg => RegisterSet::DReg,
            Self::DregL => RegisterSet::DRegL,
            Self::DregH => RegisterSet::DRegH,
        }
    }

    fn expr(&self) -> Expr {
        match self {
            Self::Dreg | Self::Mreg => b_ptr(b_field("i"), 4),
            Self::DregL | Self::DregH => b_ptr(b_field("i"), 2),
        }
    }

    fn size(&self) -> isize {
        match self {
            Self::Dreg | Self::Mreg => 4,
            Self::DregL | Self::DregH => 2,
        }
    }
}

struct DspLdStFactory();

impl DspLdStFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, op: Op, aop: AddrOp, mop: MOp) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name(&op.name(&mop.addr_str(), &mop.reg_str()))
            .display(op.display(&format!("{}{}", mop.display(), aop.display())))
            .set_field_type("w", FieldType::Mask(op as u16))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("reg", FieldType::Variable(mop.reg()))
            .add_pcode(op.expr(mop.expr(), b_field("reg")));

        instr = match mop {
            MOp::Dreg => instr.set_field_type("m", FieldType::Mask(0x0)),
            MOp::DregL => instr.set_field_type("m", FieldType::Mask(0x1)),
            MOp::DregH => instr.set_field_type("m", FieldType::Mask(0x2)),
            MOp::Mreg => instr.set_field_type("m", FieldType::Variable(RegisterSet::MReg)),
        };

        instr = match aop {
            AddrOp::Inc => instr.add_pcode(cs_assign_by(e_add, b_field("i"), b_num(mop.size()))),
            AddrOp::Dec => instr.add_pcode(cs_assign_by(e_sub, b_field("i"), b_num(mop.size()))),
            AddrOp::None => instr,
            AddrOp::IncM => instr.add_pcode(cs_assign_by(e_add, b_field("i"), b_field("m"))),
        };

        instr
    }
}

impl InstrFactory for DspLdStFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut instrs = [Op::Load, Op::Store]
            .into_iter()
            .cartesian_product([AddrOp::Inc, AddrOp::Dec, AddrOp::None])
            .cartesian_product([MOp::Dreg, MOp::DregL, MOp::DregH])
            .map(|((op, aop), mop)| Self::base_instr(ifam, op, aop, mop))
            .collect::<Vec<InstrBuilder>>();

        instrs.append(
            &mut [Op::Load, Op::Store]
                .into_iter()
                .map(|op| Self::base_instr(ifam, op, AddrOp::IncM, MOp::Mreg))
                .collect(),
        );

        instrs
    }
}
