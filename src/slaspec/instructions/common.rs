use super::{
    expr::Expr,
    pattern::{FieldType, RegisterSet},
};

pub type BinOp = fn(Expr, Expr) -> Expr;
pub type UnOp = fn(Expr) -> Expr;

#[derive(Debug, Clone)]
pub enum RegParam {
    Fixed {
        group: u16,
        id: String,
        size: usize,
        mask: u16,
    },
    Var {
        group: u16,
        regset: RegisterSet,
    },
}

impl RegParam {
    pub fn reg(group: u16, id: &str, size: usize, mask: u16) -> Self {
        RegParam::Fixed {
            group,
            id: id.to_string(),
            size,
            mask,
        }
    }

    pub fn var(group: u16, regset: RegisterSet) -> Self {
        RegParam::Var { group, regset }
    }

    pub fn grp(&self) -> u16 {
        match self {
            RegParam::Fixed {
                group,
                id: _,
                size: _,
                mask: _,
            } => *group,
            RegParam::Var { group, regset: _ } => *group,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Fixed {
                group: _,
                id: _,
                size,
                mask: _,
            } => *size,
            Self::Var {
                group: _,
                regset: _,
            } => 4,
        }
    }

    pub fn ftype(&self) -> FieldType {
        match self {
            RegParam::Fixed {
                group: _,
                id: _,
                size: _,
                mask,
            } => FieldType::Mask(*mask),
            RegParam::Var { group: _, regset } => FieldType::Variable(regset.clone()),
        }
    }

    pub fn all_regs() -> [RegParam; 14] {
        [
            RegParam::var(0x0, RegisterSet::DReg),
            RegParam::var(0x1, RegisterSet::PReg),
            RegParam::var(0x2, RegisterSet::IReg),
            RegParam::var(0x2, RegisterSet::MReg),
            RegParam::var(0x3, RegisterSet::BReg),
            RegParam::var(0x3, RegisterSet::LReg),
            RegParam::reg(0x4, "A0.X", 1, 0x0),
            RegParam::reg(0x4, "A0.W", 4, 0x1),
            RegParam::reg(0x4, "A1.X", 1, 0x2),
            RegParam::reg(0x4, "A1.W", 4, 0x3),
            RegParam::reg(0x4, "ASTAT", 4, 0x6),
            RegParam::reg(0x4, "RETS", 4, 0x7),
            RegParam::var(0x6, RegisterSet::SyRg2),
            RegParam::var(0x7, RegisterSet::SyRg3),
        ]
    }
}
