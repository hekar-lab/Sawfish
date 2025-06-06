use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "LdStExcl",
        "Long Load/Store with indexed addressing",
        "lse",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0x39), 6),
                ProtoField::new("w", FieldType::Blank, 1),
                ProtoField::new("z", FieldType::Blank, 1),
                ProtoField::new("sz", FieldType::Blank, 2),
                ProtoField::new("ptr", FieldType::Variable(RegisterSet::PReg), 3),
                ProtoField::new("reg", FieldType::Blank, 3),
            ]),
            ProtoPattern::new(vec![ProtoField::new(
                "emotionalSupportBits",
                FieldType::Any,
                16,
            )]),
        ],
    );

    ifam.add_pcodeop("syncexcl");
    ifam.add_instrs(&LdStExclFactory());

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
        format!("X{:02}bit", self.bytes() * 8)
    }

    fn display(&self) -> String {
        match self {
            Self::Byte => "B[{ptr}]",
            Self::Word => "W[{ptr}]",
            Self::Double => "[{ptr}]",
        }
        .to_string()
    }

    fn epxr(&self) -> Expr {
        e_ptr(e_rfield("ptr"), self.bytes())
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
    fn display(&self) -> &'static str {
        match self {
            Self::Zero => "Z, EXCL",
            Self::Signed => "X, EXCL",
        }
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
}

impl Reg {
    fn regset(&self) -> RegisterSet {
        match self {
            Self::Dreg => RegisterSet::DReg,
            Self::DregL => RegisterSet::DRegL,
            Self::DregH => RegisterSet::DRegH,
        }
    }

    fn full_reg(&self) -> bool {
        match self {
            Self::Dreg => true,
            _ => false,
        }
    }

    fn name(&self) -> String {
        match self {
            Self::Dreg => "Dreg",
            Self::DregL => "DregL",
            Self::DregH => "DregH",
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
            Op::Store => format!("St{}To{}", self.reg.name(), self.size.name()),
        }
    }

    fn display(&self) -> String {
        let reg_part = "{reg}";
        let addr_part = self.size.display();
        let opt_part = if self.op == Op::Load && self.size != Size::Double && self.reg.full_reg() {
            self.ext.display()
        } else {
            "EXCL"
        };

        match self.op {
            Op::Load => format!("{reg_part} = {addr_part} ({opt_part})"),
            Op::Store => format!("CC = ({addr_part} = {reg_part}) ({opt_part})"),
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
            LdStParams::new(Size::Word, Op::Load, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Word, Op::Load, Ext::Signed, Reg::Dreg),
            LdStParams::new(Size::Byte, Op::Load, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Byte, Op::Load, Ext::Signed, Reg::Dreg),
            LdStParams::new(Size::Word, Op::Load, Ext::Zero, Reg::DregL),
            LdStParams::new(Size::Word, Op::Load, Ext::Signed, Reg::DregH),
            LdStParams::new(Size::Double, Op::Store, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Word, Op::Store, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Byte, Op::Store, Ext::Zero, Reg::Dreg),
            LdStParams::new(Size::Word, Op::Store, Ext::Zero, Reg::DregH),
        ]
    }
}

struct LdStExclFactory();

impl LdStExclFactory {
    fn ldst_instr(ifam: &InstrFamilyBuilder, params: LdStParams) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(&params.name())
            .display(params.display())
            .set_field_type("w", FieldType::Mask(params.op as u16))
            .set_field_type("z", FieldType::Mask(params.ext as u16))
            .set_field_type("sz", FieldType::Mask(params.sz_mask() as u16))
            .set_field_type("reg", FieldType::Variable(params.reg.regset()))
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
                Op::Store => cs_mline(vec![
                    e_copy(
                        params.size.epxr(),
                        b_size(e_rfield("reg"), params.size.bytes()),
                    ),
                    e_copy(b_reg("CC"), b_num(1)),
                ]),
            })
    }

    fn sync_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("SyncExcl")
            .set_field_type("w", FieldType::Mask(0x1))
            .set_field_type("z", FieldType::Mask(0x1))
            .set_field_type("sz", FieldType::Mask(0x3))
            .add_pcode(e_mac("syncexcl"))
    }
}

impl InstrFactory for LdStExclFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut ldst_instrs: Vec<InstrBuilder> = LdStParams::all_params()
            .into_iter()
            .map(|params| Self::ldst_instr(ifam, params))
            .collect();

        ldst_instrs.push(Self::sync_instr(ifam));

        ldst_instrs
    }
}
