use super::{
    core::InstrBuilder,
    expr::Expr,
    pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet},
};

pub type UnOp = fn(Expr) -> Expr;
pub type BinOp = fn(Expr, Expr) -> Expr;

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

    pub fn set_field(&self, instr: InstrBuilder, field_id: &str) -> InstrBuilder {
        match self {
            RegParam::Fixed {
                group: _,
                id: _,
                size: _,
                mask,
            } => instr.set_field_type(field_id, FieldType::Mask(*mask)),
            RegParam::Var { group: _, regset } => match regset {
                RegisterSet::IReg | RegisterSet::MReg | RegisterSet::BReg | RegisterSet::LReg => {
                    instr.split_field(
                        field_id,
                        ProtoPattern::new(vec![
                            ProtoField::new(
                                &format!("{field_id}H"),
                                FieldType::Mask(match regset {
                                    RegisterSet::IReg | RegisterSet::BReg => 0x0,
                                    _ => 0x1,
                                }),
                                1,
                            ),
                            ProtoField::new(
                                &format!("{field_id}L"),
                                FieldType::Variable(*regset),
                                2,
                            ),
                        ]),
                    )
                }
                _ => instr.set_field_type(field_id, FieldType::Variable(*regset)),
            },
        }
    }

    pub fn get_field_id(&self, field_id: &str) -> String {
        match self {
            RegParam::Var { group: _, regset } => match regset {
                RegisterSet::IReg | RegisterSet::MReg | RegisterSet::BReg | RegisterSet::LReg => {
                    format!("{field_id}L")
                }
                _ => field_id.to_string(),
            },
            _ => field_id.to_string(),
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
