use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "LdSt",
        "Load/Store",
        "ldst",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x9), 4),
            ProtoField::new("sz", FieldType::Blank, 2),
            ProtoField::new("w", FieldType::Blank, 1),
            ProtoField::new("aop", FieldType::Blank, 2),
            ProtoField::new("z", FieldType::Blank, 1),
            ProtoField::new("ptr", FieldType::Variable(RegisterSet::PReg), 3),
            ProtoField::new("reg", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&LdStFactory());

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
            Self::Byte => "B",
            Self::Word => "W",
            Self::Double => "",
        }
        .to_string()
    }

    fn epxr(&self) -> Expr {
        b_ptr(e_rfield("ptr"), self.bytes())
    }
}

#[derive(Debug, Clone, Copy)]
enum AddrOp {
    Inc = 0,
    Dec = 1,
    None = 2,
}

impl AddrOp {
    fn display(&self) -> String {
        format!(
            "[{{ptr}}{}]",
            match self {
                Self::Inc => "++",
                Self::Dec => "--",
                Self::None => "",
            }
        )
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
            Self::Zero => e_macp("zext", e),
            Self::Signed => e_macp("sext", e),
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
    aop: AddrOp,
    ext: Ext,
    reg: Reg,
}

impl LdStParams {
    fn new(size: Size, op: Op, aop: AddrOp, ext: Ext, reg: Reg) -> Self {
        Self {
            size,
            op,
            aop,
            ext,
            reg,
        }
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
            "{}{}{}",
            self.size.display(),
            self.aop.display(),
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
        fn aop_fill(size: Size, op: Op, ext: Ext, reg: Reg) -> Vec<LdStParams> {
            [AddrOp::Inc, AddrOp::Dec, AddrOp::None]
                .into_iter()
                .map(|aop| LdStParams::new(size, op, aop, ext, reg))
                .collect()
        }

        vec![
            aop_fill(Size::Double, Op::Load, Ext::Zero, Reg::Dreg),
            aop_fill(Size::Double, Op::Load, Ext::Signed, Reg::Preg),
            aop_fill(Size::Word, Op::Load, Ext::Zero, Reg::Dreg),
            aop_fill(Size::Word, Op::Load, Ext::Signed, Reg::Dreg),
            aop_fill(Size::Byte, Op::Load, Ext::Zero, Reg::Dreg),
            aop_fill(Size::Byte, Op::Load, Ext::Signed, Reg::Dreg),
            aop_fill(Size::Double, Op::Store, Ext::Zero, Reg::Dreg),
            aop_fill(Size::Double, Op::Store, Ext::Signed, Reg::Preg),
            aop_fill(Size::Word, Op::Store, Ext::Zero, Reg::Dreg),
            aop_fill(Size::Byte, Op::Store, Ext::Zero, Reg::Dreg),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

struct LdStFactory();

impl LdStFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, params: LdStParams) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name(&params.name())
            .display(params.display())
            .set_field_type("sz", FieldType::Mask(params.size as u16))
            .set_field_type("w", FieldType::Mask(params.op as u16))
            .set_field_type("aop", FieldType::Mask(params.aop as u16))
            .set_field_type("z", FieldType::Mask(params.ext as u16))
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

        let incr_expr = b_num(params.size.bytes() as isize);
        match params.aop {
            AddrOp::Inc => instr = instr.add_pcode(cs_assign_by(e_add, e_rfield("ptr"), incr_expr)),
            AddrOp::Dec => instr = instr.add_pcode(cs_assign_by(e_sub, e_rfield("ptr"), incr_expr)),
            _ => {}
        }

        instr
    }
}

impl InstrFactory for LdStFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        LdStParams::all_params()
            .into_iter()
            .map(|params| Self::base_instr(ifam, params))
            .collect()
    }
}
