use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::instr32::common32::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

enum OperEnum {
    L,
    H,
    LH,
}

#[derive(Debug, Clone, Copy)]
struct Mult16Params {
    mode: Mmode,
    mm: bool,
    p: bool,
    w0: Option<Oper>,
    w1: Option<Oper>,
}

impl Mult16Params {
    fn new(mode: Mmode, mm: bool, p: bool, w0: Option<Oper>, w1: Option<Oper>) -> Self {
        Self {
            mode,
            mm,
            p,
            w0,
            w1,
        }
    }

    fn oper_enum(&self) -> OperEnum {
        if self.w0.is_some() && self.w1.is_some() {
            OperEnum::LH
        } else if self.w0.is_some() {
            OperEnum::L
        } else if self.w1.is_some() {
            OperEnum::H
        } else {
            panic!("At least one operand should be set.")
        }
    }
}

pub struct Mult16Factory();

impl Mult16Factory {
    fn display(params: Mult16Params) -> String {
        let mode_opt = if let Some(s) = params.mode.to_str() {
            format!(" ({s})")
        } else {
            "".to_string()
        };

        match params.oper_enum() {
            OperEnum::L => format!("{{dstL}} = {{src0L}} * {{src1L}}{}", &mode_opt),
            OperEnum::H => format!(
                "{{dstH}} = {{src0H}} * {{src1H}}{}",
                if let Some(s) = params.mode.to_str() {
                    let mut opt_vec = vec![];
                    if params.mm {
                        opt_vec.push("M".to_string());
                    }
                    opt_vec.push(s);
                    format!(" ({})", opt_vec.join(", "))
                } else {
                    "".to_string()
                }
            ),
            OperEnum::LH => format!(
                "{{dstL}} = {{src0L}} * {{src1L}}{}, {{dstH}} = {{src0H}} * {{src1H}}{}",
                mode_opt,
                if params.mm { " (M)" } else { "" }
            ),
        }
    }

    fn set_fields(mut instr: InstrBuilder, params: Mult16Params) -> InstrBuilder {
        instr = instr
            .set_field_type("mmod", FieldType::Mask(params.mode as u16))
            .set_field_type("mm", FieldType::Mask(params.mm as u16))
            .set_field_type("p", FieldType::Mask(params.p as u16))
            .set_field_type("w0", FieldType::Mask(params.w0.is_some() as u16))
            .set_field_type("w1", FieldType::Mask(params.w1.is_some() as u16))
            .set_field_type("op0", FieldType::Mask(0x0))
            .set_field_type("op1", FieldType::Mask(0x0))
            .divide_field(
                "src0",
                ProtoPattern::new(vec![
                    ProtoField::new("src0L", FieldType::Blank, 3),
                    ProtoField::new("src0H", FieldType::Blank, 3),
                ]),
            )
            .divide_field(
                "src1",
                ProtoPattern::new(vec![
                    ProtoField::new("src1L", FieldType::Blank, 3),
                    ProtoField::new("src1H", FieldType::Blank, 3),
                ]),
            );

        if let Some(oper) = params.w0 {
            instr = instr
                .set_field_type("h00", FieldType::Mask(oper.lhs as u16))
                .set_field_type("h10", FieldType::Mask(oper.rhs as u16))
                .set_field_type("src0L", FieldType::Variable(oper.lhs.regset()))
                .set_field_type("src1L", FieldType::Variable(oper.rhs.regset()));
        }

        if let Some(oper) = params.w1 {
            instr = instr
                .set_field_type("h01", FieldType::Mask(oper.lhs as u16))
                .set_field_type("h11", FieldType::Mask(oper.rhs as u16))
                .set_field_type("src0H", FieldType::Variable(oper.lhs.regset()))
                .set_field_type("src1H", FieldType::Variable(oper.rhs.regset()));
        }

        if params.p {
            instr = instr.divide_field(
                "dst",
                ProtoPattern::new(vec![
                    ProtoField::new("dstL", FieldType::Variable(RegisterSet::DRegE), 3),
                    ProtoField::new("dstH", FieldType::Variable(RegisterSet::DRegO), 3),
                ]),
            )
        } else {
            instr = instr.divide_field(
                "dst",
                ProtoPattern::new(vec![
                    ProtoField::new("dstL", FieldType::Variable(RegisterSet::DRegL), 3),
                    ProtoField::new("dstH", FieldType::Variable(RegisterSet::DRegH), 3),
                ]),
            )
        }

        instr
    }

    fn expr(params: Mult16Params) -> Expr {
        let expr_l = cs_mline(
            vec![
                mult_expr("resL", "src0L", "src1L", params.mode, false, 5),
                extract_expr(
                    "dstL",
                    b_var("resL"),
                    params.p,
                    params.mode,
                    false,
                    5,
                    "Low",
                ),
            ]
            .into(),
        );

        let expr_h = cs_mline(
            vec![
                mult_expr(
                    "resH",
                    "src0H",
                    "src1H",
                    if params.w0.is_some() {
                        Mmode::Default
                    } else {
                        params.mode
                    },
                    params.mm,
                    5,
                ),
                extract_expr(
                    "dstH",
                    b_var("resH"),
                    params.p,
                    params.mode,
                    false,
                    5,
                    "High",
                ),
            ]
            .into(),
        );

        match params.oper_enum() {
            OperEnum::L => expr_l,
            OperEnum::H => expr_h,
            OperEnum::LH => cs_mline(vec![expr_l, expr_h].into()),
        }
    }

    fn base_instr(ifam: &InstrFamilyBuilder, params: Mult16Params) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name(if params.w0.is_some() && params.w1.is_some() {
                "ParaMult16AndMult16"
            } else {
                "Mult16"
            })
            .display(Self::display(params))
            .add_pcode(Self::expr(params));

        instr = Self::set_fields(instr, params);

        instr
    }
}

fn all_oper() -> Vec<(Option<Oper>, Option<Oper>)> {
    let mut oper_l: Vec<Option<Oper>> = Oper::all().into_iter().map(|o| Some(o)).collect();
    let mut oper_h: Vec<Option<Oper>> = Oper::all().into_iter().map(|o| Some(o)).collect();
    oper_l.insert(0, None);
    oper_h.insert(0, None);

    oper_l
        .into_iter()
        .cartesian_product(oper_h)
        .filter_map(|(ol, oh)| {
            if ol.is_none() && oh.is_none() {
                None
            } else {
                Some((ol, oh))
            }
        })
        .collect()
}

impl InstrFactory for Mult16Factory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut params_half: Vec<Mult16Params> = all_oper()
            .into_iter()
            .cartesian_product(Mmode::mmod1())
            .cartesian_product([false, true])
            .filter_map(|(((w0, w1), mode), mm)| {
                if w1.is_none() && mm {
                    None
                } else {
                    Some(Mult16Params::new(mode, mm, false, w0, w1))
                }
            })
            .collect();

        let mut params_full: Vec<Mult16Params> = all_oper()
            .into_iter()
            .cartesian_product(Mmode::mmode())
            .cartesian_product([false, true])
            .filter_map(|(((w0, w1), mode), mm)| {
                if w1.is_none() && mm {
                    None
                } else {
                    Some(Mult16Params::new(mode, mm, true, w0, w1))
                }
            })
            .collect();

        params_half.append(&mut params_full);

        params_half
            .into_iter()
            .map(|params| Self::base_instr(ifam, params))
            .collect()
    }
}
