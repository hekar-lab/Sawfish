use std::fmt;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Aop {
    Max = 0x0,
    Min = 0x1,
    Abs = 0x2,
    Neg = 0x3,
}

impl Aop {
    fn display(&self, vec: bool, sat: bool) -> String {
        format!(
            "{}{}{}",
            match self {
                Aop::Max => "{dst0} = MAX({src0}, {src1})",
                Aop::Min => "{dst0} = MIN({src0}, {src1})",
                Aop::Abs => "{dst0} = ABS {src0}",
                Aop::Neg => "{dst0} = -{src0}",
            },
            if *self == Aop::Neg {
                if sat { " (S)" } else { " (NS)" }
            } else {
                ""
            },
            if vec { " (V)" } else { "" }
        )
    }

    fn maxmin(&self) -> bool {
        match self {
            Aop::Max | Aop::Min => true,
            _ => false,
        }
    }

    fn expr(&self, dst: Expr, src0: Expr, src1: Expr, sat: bool, size: usize, id: &str) -> Expr {
        match self {
            Aop::Max => cs_max(dst, src0, src1, id),
            Aop::Min => cs_min(dst, src0, src1, id),
            Aop::Abs => cs_abs_sat(dst, src0, size, id),
            Aop::Neg => cs_neg_sat(dst, src0, sat, size, id),
        }
    }
}

impl fmt::Display for Aop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct MMANFactory();

impl MMANFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, op32: bool, aop: Aop, sat: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(&format!("{}{}", aop, if op32 { "32" } else { "16Vec" }))
            .display(aop.display(!op32, sat))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type(
                "aopc",
                FieldType::Mask(if !op32 && aop == Aop::Neg {
                    0xf
                } else {
                    0x6 + op32 as u16
                }),
            )
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type_opt(aop.maxmin(), "src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode_opt(if !op32 {
                Some(cs_mline(vec![
                    e_copy(e_local("src0L", 2), b_size(e_rfield("src0"), 2)),
                    e_copy(e_local("src0H", 2), b_trunc(e_rfield("src0"), 2)),
                    e_local("resL", 2),
                    e_local("resH", 2),
                ]))
            } else {
                None
            })
            .add_pcode_opt(if !op32 && aop.maxmin() {
                Some(cs_mline(vec![
                    e_copy(e_local("src1L", 2), b_size(e_rfield("src1"), 2)),
                    e_copy(e_local("src1H", 2), b_trunc(e_rfield("src1"), 2)),
                ]))
            } else {
                None
            })
            .add_pcode(if op32 {
                aop.expr(
                    e_rfield("dst0"),
                    e_rfield("src0"),
                    e_rfield("src1"),
                    sat,
                    4,
                    "32",
                )
            } else {
                cs_mline(vec![
                    aop.expr(
                        b_var("resL"),
                        b_var("src0L"),
                        b_var("src1L"),
                        sat,
                        2,
                        "vecL",
                    ),
                    aop.expr(
                        b_var("resH"),
                        b_var("src0H"),
                        b_var("src1H"),
                        sat,
                        2,
                        "vecH",
                    ),
                ])
            })
            .add_pcode_opt(if !op32 {
                Some(e_copy(
                    e_rfield("dst0"),
                    e_bit_or(
                        b_grp(e_lshft(e_zext(b_var("resH")), b_num(16))),
                        e_zext(b_var("resL")),
                    ),
                ))
            } else {
                None
            })
    }
}

impl InstrFactory for MMANFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let op16vecparams: Vec<(bool, Aop, bool)> = [Aop::Max, Aop::Min, Aop::Abs, Aop::Neg]
            .into_iter()
            .map(|aop| (false, aop, false))
            .collect();

        let mut op32params: Vec<(bool, Aop, bool)> = [Aop::Max, Aop::Min, Aop::Abs, Aop::Neg]
            .into_iter()
            .map(|aop| (true, aop, false))
            .collect();

        op32params.push((true, Aop::Neg, true));

        vec![op16vecparams, op32params]
            .concat()
            .into_iter()
            .map(|(op32, aop, sat)| Self::base_instr(ifam, op32, aop, sat))
            .collect()
    }
}
