use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_64(
        "LdStAbs",
        "Load/Store 32-bit Absolute Address",
        "lsa",
        [
            ProtoPattern::new(vec![ProtoField::new("sig", FieldType::Mask(0xd800), 16)]),
            ProtoPattern::new(vec![ProtoField::new("immH", FieldType::UImmVal, 16)]),
            ProtoPattern::new(vec![ProtoField::new("immL", FieldType::UImmVal, 16)]),
            ProtoPattern::new(vec![
                ProtoField::new("mask4z", FieldType::Mask(0x0), 4),
                ProtoField::new("sz", FieldType::Blank, 2),
                ProtoField::new("w", FieldType::Blank, 1),
                ProtoField::new("mask2z", FieldType::Mask(0x0), 2),
                ProtoField::new("z", FieldType::Blank, 1),
                ProtoField::new("mask3z", FieldType::Mask(0x0), 3),
                ProtoField::new("reg", FieldType::Blank, 3),
            ]),
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
            Self::Byte => "B[{$imm32}]",
            Self::Word => "W[{$imm32}]",
            Self::Double => "[{$imm32}]",
        }
        .to_string()
    }

    fn epxr(&self) -> Expr {
        e_ptr(b_var("imm32"), self.bytes())
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
    DregL,
    DregH,
    Preg,
}

impl Reg {
    fn regset(&self) -> RegisterSet {
        match self {
            Self::Dreg => RegisterSet::DReg,
            Self::DregL => RegisterSet::DRegL,
            Self::DregH => RegisterSet::DRegH,
            Self::Preg => RegisterSet::PReg,
        }
    }

    fn full_reg(&self) -> bool {
        match self {
            Self::Dreg | Self::Preg => true,
            _ => false,
        }
    }

    fn name(&self) -> String {
        match self {
            Self::Dreg => "Dreg",
            Self::DregL => "DregL",
            Self::DregH => "DregH",
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
                if self.size == Size::Word { "L" } else { "" },
                self.size.name()
            ),
        }
    }

    fn display(&self) -> String {
        let reg_part = "{reg}";
        let addr_part = format!(
            "{}{}",
            self.size.display(),
            if self.op == Op::Load && self.size != Size::Double && self.reg.full_reg() {
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

    fn sz_mask(&self) -> u16 {
        match self.reg {
            Reg::DregH | Reg::DregL => 0x3,
            _ => self.size as u16,
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
            LdStParams::new(Size::Word, Op::Load, Ext::Zero, Reg::DregL),
            LdStParams::new(Size::Word, Op::Load, Ext::Signed, Reg::DregH),
            LdStParams::new(Size::Double, Op::Store, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Double, Op::Store, Ext::Signed, Reg::Preg),
            LdStParams::new(Size::Word, Op::Store, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Byte, Op::Store, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Word, Op::Store, Ext::Signed, Reg::DregH),
        ]
    }
}

struct LdStIdxFactory();

impl LdStIdxFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, params: LdStParams) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(&params.name())
            .display(params.display())
            .set_field_type("w", FieldType::Mask(params.op as u16))
            .set_field_type("z", FieldType::Mask(params.ext as u16))
            .set_field_type("sz", FieldType::Mask(params.sz_mask()))
            .set_field_type("reg", FieldType::Variable(params.reg.regset()))
            .add_action(e_copy(
                b_var("imm32"),
                e_bit_or(
                    b_grp(e_lshft(e_rfield("immH"), b_num(16))),
                    e_rfield("immL"),
                ),
            ))
            .add_pcode(match params.op {
                Op::Load => e_copy(
                    e_rfield("reg"),
                    if (params.reg.full_reg() && params.size == Size::Double)
                        || (!params.reg.full_reg() && params.size == Size::Word)
                    {
                        params.size.epxr()
                    } else {
                        params.ext.expr(params.size.epxr())
                    },
                ),
                Op::Store => e_copy(
                    params.size.epxr(),
                    b_size(e_rfield("reg"), params.size.bytes()),
                ),
            })
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
