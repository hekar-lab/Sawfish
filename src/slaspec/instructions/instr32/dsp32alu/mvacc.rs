use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::FieldType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Aop {
    A0 = 0x0,
    A1 = 0x1,
    Dual = 0x2,
    Mv = 0x3,
}

impl Aop {
    fn name(&self, sat: bool) -> &'static str {
        if sat {
            match self {
                Aop::A0 => "SatAccA0",
                Aop::A1 => "SatAccA1",
                Aop::Dual => "SatAccDual",
                Aop::Mv => "MvAxToAx",
            }
        } else {
            match self {
                Aop::A0 | Aop::A1 => "LdImmToAx",
                Aop::Dual => "LdImmToAxDul",
                Aop::Mv => "MvAxToAx",
            }
        }
    }

    fn display(&self, sat: bool) -> String {
        if sat {
            match self {
                Aop::A0 => "A0 = A0 (S)",
                Aop::A1 => "A1 = A1 (S)",
                Aop::Dual => "A0 = A0 (S), A1 = A1 (S)",
                Aop::Mv => "A1 = A0",
            }
        } else {
            match self {
                Aop::A0 => "A0 = 0",
                Aop::A1 => "A1 = 0",
                Aop::Dual => "A0 = A1 = 0",
                Aop::Mv => "A0 = A1",
            }
        }
        .to_string()
    }

    fn expr(&self, sat: bool) -> Expr {
        fn sat_acc(acc_id: &str) -> Expr {
            let neg_lab = b_label(&format!("sat_neg_{acc_id}"));
            let pos_lab = b_label(&format!("sat_pos_{acc_id}"));
            let end_lab = b_label(&format!("sat_end_{acc_id}"));
            cs_mline(vec![
                b_ifgoto(e_lt(b_reg(acc_id), b_num(0xff80000000)), neg_lab.clone()),
                b_ifgoto(e_gt(b_reg(acc_id), b_num(0x007fffffff)), pos_lab.clone()),
                e_copy(b_reg(&format!("{acc_id}.W")), b_size(b_reg(acc_id), 4)),
                b_goto(end_lab.clone()),
                neg_lab,
                e_copy(b_reg(&format!("{acc_id}.W")), b_num(0x80000000)),
                b_goto(end_lab.clone()),
                pos_lab,
                e_copy(b_reg(&format!("{acc_id}.W")), b_num(0x7fffffff)),
                end_lab,
            ])
        }
        if sat {
            match self {
                Aop::A0 => sat_acc("A0"),
                Aop::A1 => sat_acc("A1"),
                Aop::Dual => cs_mline(vec![sat_acc("A0"), sat_acc("A1")]),
                Aop::Mv => e_copy(b_reg("A1"), b_reg("A0")),
            }
        } else {
            match self {
                Aop::A0 => e_copy(b_reg("A0"), b_num(0)),
                Aop::A1 => e_copy(b_reg("A1"), b_num(0)),
                Aop::Dual => cs_mline(vec![
                    e_copy(b_reg("A0"), b_num(0)),
                    e_copy(b_reg("A1"), b_num(0)),
                ]),
                Aop::Mv => e_copy(b_reg("A0"), b_reg("A1")),
            }
        }
    }
}

pub struct MvAccFactory();

impl MvAccFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, aop: Aop, sat: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(aop.name(sat))
            .display(aop.display(sat))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0x8))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(0x0))
            .add_pcode(aop.expr(sat))
    }
}

impl InstrFactory for MvAccFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [Aop::A0, Aop::A1, Aop::Dual, Aop::Mv]
            .into_iter()
            .cartesian_product([false, true])
            .map(|(aop, sat)| Self::base_instr(ifam, aop, sat))
            .collect()
    }
}
