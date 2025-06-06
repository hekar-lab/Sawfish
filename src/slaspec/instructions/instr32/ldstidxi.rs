use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "LdStIdxI",
        "Long Load/Store with indexed addressing",
        "lid",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0x39), 6),
                ProtoField::new("w", FieldType::Blank, 1),
                ProtoField::new("z", FieldType::Blank, 1),
                ProtoField::new("sz", FieldType::Blank, 2),
                ProtoField::new("ptr", FieldType::Variable(RegisterSet::PReg), 3),
                ProtoField::new("reg", FieldType::Blank, 3),
            ]),
            ProtoPattern::new(vec![ProtoField::new("off", FieldType::SImmVal, 16)]),
        ],
    );

    ifam.add_instrs(&LdStIdxFactory());

    ifam
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Size {
    Double = 0,
    Word = 1,
    Byte = 2,
}

impl Size {
    fn bytes(&self) -> usize {
        match self {
            Self::Double => 4,
            Self::Word => 2,
            Self::Byte => 1,
        }
    }
    fn name(&self) -> String {
        format!("M{:02}bit", self.bytes() * 8)
    }

    fn display(&self) -> String {
        match self {
            Self::Byte => "B[{ptr} + {off}]",
            Self::Word => "W[{ptr} + {$imm2}]",
            Self::Double => "[{ptr} + {$imm4}]",
        }
        .to_string()
    }

    fn epxr(&self) -> Expr {
        let imm = match self {
            Self::Byte => e_rfield("off"),
            Self::Word => b_var("imm2"),
            Self::Double => b_var("imm4"),
        };
        e_ptr(b_grp(e_add(e_rfield("ptr"), imm)), self.bytes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Load = 0,
    Store = 1,
}

#[derive(Debug, Clone, Copy)]
enum Ext {
    Zero = 0,
    Signed = 1,
}

impl Ext {
    fn display(&self) -> String {
        match self {
            Self::Zero => " (Z)",
            Self::Signed => " (X)",
        }
        .to_string()
    }

    fn expr(&self, e: Expr) -> Expr {
        match self {
            Self::Zero => e_zext(e),
            Self::Signed => e_sext(e),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Reg {
    Dreg,
    Preg,
}

impl Reg {
    fn regset(&self) -> RegisterSet {
        match self {
            Self::Dreg => RegisterSet::DReg,
            Self::Preg => RegisterSet::PReg,
        }
    }

    fn name(&self) -> String {
        match self {
            Self::Dreg => "Dreg",
            Self::Preg => "Preg",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
struct LdStParams {
    size: Size,
    op: Op,
    ext: Ext,
    reg: Reg,
}

impl LdStParams {
    fn new(size: Size, op: Op, ext: Ext, reg: Reg) -> Self {
        Self { size, op, ext, reg }
    }
    fn name(&self) -> String {
        match self.op {
            Op::Load => format!("Ld{}To{}", self.size.name(), self.reg.name()),
            Op::Store => format!(
                "St{}{}To{}",
                self.reg.name(),
                if self.size == Size::Double { "" } else { "L" },
                self.size.name()
            ),
        }
    }

    fn display(&self) -> String {
        let reg_part = "{reg}";
        let addr_part = format!(
            "{}{}",
            self.size.display(),
            if self.op == Op::Load && self.size != Size::Double {
                self.ext.display()
            } else {
                "".to_string()
            }
        );
        match self.op {
            Op::Load => format!("{reg_part} = {addr_part}"),
            Op::Store => format!("{addr_part} = {reg_part}"),
        }
    }

    fn all_params() -> Vec<Self> {
        vec![
            LdStParams::new(Size::Double, Op::Load, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Double, Op::Load, Ext::Signed, Reg::Preg),
            LdStParams::new(Size::Word, Op::Load, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Word, Op::Load, Ext::Signed, Reg::Dreg),
            LdStParams::new(Size::Byte, Op::Load, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Byte, Op::Load, Ext::Signed, Reg::Dreg),
            LdStParams::new(Size::Double, Op::Store, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Double, Op::Store, Ext::Signed, Reg::Preg),
            LdStParams::new(Size::Word, Op::Store, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Byte, Op::Store, Ext::Zero, Reg::Dreg),
        ]
    }
}

struct LdStIdxFactory();

impl LdStIdxFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, params: LdStParams) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name(&params.name())
            .display(params.display())
            .set_field_type("w", FieldType::Mask(params.op as u16))
            .set_field_type("z", FieldType::Mask(params.ext as u16))
            .set_field_type("sz", FieldType::Mask(params.size as u16))
            .set_field_type("reg", FieldType::Variable(params.reg.regset()))
            .add_pcode(match params.op {
                Op::Load => e_copy(
                    e_rfield("reg"),
                    if params.size == Size::Double {
                        params.size.epxr()
                    } else {
                        params.ext.expr(params.size.epxr())
                    },
                ),
                Op::Store => e_copy(
                    params.size.epxr(),
                    b_size(e_rfield("reg"), params.size.bytes()),
                ),
            });

        instr = match params.size {
            Size::Word => {
                instr.add_action(e_copy(b_var("imm2"), e_mult(e_rfield("off"), b_num(2))))
            }
            Size::Double => {
                instr.add_action(e_copy(b_var("imm4"), e_mult(e_rfield("off"), b_num(4))))
            }
            _ => instr,
        };

        instr
    }
}

impl InstrFactory for LdStIdxFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        LdStParams::all_params()
            .into_iter()
            .map(|params| Self::base_instr(ifam, params))
            .collect()
    }
}
