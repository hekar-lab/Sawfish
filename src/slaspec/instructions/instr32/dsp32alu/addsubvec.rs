use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy)]
enum Aop {
    AA = 0x0,
    AS = 0x1,
    SA = 0x2,
    SS = 0x3,
}

impl Aop {
    fn op_str(&self) -> String {
        match self {
            Aop::AA => "+|+",
            Aop::AS => "+|-",
            Aop::SA => "-|+",
            Aop::SS => "-|-",
        }
        .to_string()
    }

    fn expr(&self, sat: bool, id: &str) -> Expr {
        let add_expr = if sat {
            |hl: &str, id: &str| {
                cs_sadd_sat(
                    b_var(&format!("res{hl}")),
                    b_var(&format!("src0{hl}")),
                    b_var(&format!("src1{hl}")),
                    4,
                    &format!("asv{hl}_{id}"),
                )
            }
        } else {
            |hl: &str, _: &str| {
                e_copy(
                    b_var(&format!("res{hl}")),
                    e_add(b_var(&format!("src0{hl}")), b_var(&format!("src1{hl}"))),
                )
            }
        };

        let sub_expr = if sat {
            |hl: &str, id: &str| {
                cs_ssub_sat(
                    b_var(&format!("res{hl}")),
                    b_var(&format!("src0{hl}")),
                    b_var(&format!("src1{hl}")),
                    4,
                    &format!("asv{hl}_{id}"),
                )
            }
        } else {
            |hl: &str, _: &str| {
                e_copy(
                    b_var(&format!("res{hl}")),
                    e_sub(b_var(&format!("src0{hl}")), b_var(&format!("src1{hl}"))),
                )
            }
        };

        match self {
            Aop::AA => cs_mline(vec![add_expr("L", id), add_expr("H", id)].into()),
            Aop::AS => cs_mline(vec![add_expr("L", id), sub_expr("H", id)].into()),
            Aop::SA => cs_mline(vec![sub_expr("L", id), add_expr("H", id)].into()),
            Aop::SS => cs_mline(vec![sub_expr("L", id), sub_expr("H", id)].into()),
        }
    }
}

pub struct AddSubVecFactory();

impl AddSubVecFactory {
    fn display(dst_id: &str, aop: Aop) -> String {
        format!("{{{dst_id}}} = {{src0}} {} {{src1}}", aop.op_str())
    }

    fn expr_init() -> Expr {
        cs_mline(
            vec![
                e_copy(e_local("src0L", 2), b_size(e_rfield("src0"), 2)),
                e_copy(e_local("src0H", 2), b_trunc(e_rfield("src0"), 2)),
                e_copy(e_local("src1L", 2), b_size(e_rfield("src1"), 2)),
                e_copy(e_local("src1H", 2), b_trunc(e_rfield("src1"), 2)),
                e_local("resL", 2),
                e_local("resH", 2),
            ]
            .into(),
        )
    }

    fn expr_cpy(dst_id: &str, cross: bool) -> Expr {
        cs_mline(
            vec![e_copy(
                e_rfield(dst_id),
                e_bit_or(
                    e_zext(b_grp(e_lshft(
                        b_var(if cross { "resL" } else { "resH" }),
                        b_num(16),
                    ))),
                    e_zext(b_var(if cross { "resH" } else { "resL" })),
                ),
            )]
            .into(),
        )
    }

    fn base_instr(ifam: &InstrFamilyBuilder, sat: bool, cross: bool, aop: Aop) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("AddSubVec16")
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(cross as u16))
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(Self::expr_init())
    }

    fn simple_instr(ifam: &InstrFamilyBuilder, sat: bool, cross: bool, aop: Aop) -> InstrBuilder {
        Self::base_instr(ifam, sat, cross, aop)
            .display(format!(
                "{}{}",
                Self::display("dst0", aop),
                match (sat, cross) {
                    (false, false) => "",
                    (false, true) => " (CO)",
                    (true, false) => " (S)",
                    (true, true) => " (SCO)",
                }
            ))
            .set_field_type("aopc", FieldType::Mask(0x0))
            .add_pcode(aop.expr(sat, ""))
            .add_pcode(Self::expr_cpy("dst0", cross))
    }

    fn dual_instr(
        ifam: &InstrFamilyBuilder,
        sat: bool,
        cross: bool,
        aop: Aop,
        hl: bool,
    ) -> InstrBuilder {
        let dst0_aop = if hl { Aop::AS } else { Aop::AA };
        let dst1_aop = if hl { Aop::SA } else { Aop::SS };
        let mut opt_vec = vec![];

        match (sat, cross) {
            (false, false) => {}
            (false, true) => opt_vec.push("CO".to_string()),
            (true, false) => opt_vec.push("S".to_string()),
            (true, true) => opt_vec.push("SCO".to_string()),
        }

        match aop {
            Aop::SA => opt_vec.push("ASR".to_string()),
            Aop::SS => opt_vec.push("ASL".to_string()),
            _ => {}
        }

        let opt_str = if opt_vec.is_empty() {
            String::new()
        } else {
            format!(" ({})", opt_vec.join(", "))
        };

        let shift_expr = match aop {
            Aop::SA => Some(cs_mline(
                vec![
                    cs_assign_by(e_arshft, b_var("resL"), b_num(1)),
                    cs_assign_by(e_arshft, b_var("resH"), b_num(1)),
                ]
                .into(),
            )),
            Aop::SS => Some(cs_mline(
                vec![
                    cs_assign_by(e_lshft, b_var("resL"), b_num(1)),
                    cs_assign_by(e_lshft, b_var("resH"), b_num(1)),
                ]
                .into(),
            )),
            _ => None,
        };

        Self::base_instr(ifam, sat, cross, aop)
            .display(format!(
                "{}, {}{}",
                Self::display("dst0", dst0_aop),
                Self::display("dst1", dst1_aop),
                opt_str
            ))
            .set_field_type("aopc", FieldType::Mask(0x1))
            .set_field_type("hl", FieldType::Mask(hl as u16))
            .set_field_type("dst1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(dst0_aop.expr(sat, "dst0"))
            .add_pcode_opt(shift_expr.clone())
            .add_pcode(Self::expr_cpy("dst0", cross))
            .add_pcode(dst1_aop.expr(sat, "dst1"))
            .add_pcode_opt(shift_expr.clone())
            .add_pcode(Self::expr_cpy("dst1", cross))
    }
}

impl InstrFactory for AddSubVecFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut simple_instrs: Vec<InstrBuilder> = [Aop::AA, Aop::AS, Aop::SA, Aop::AS]
            .into_iter()
            .cartesian_product([false, true])
            .cartesian_product([false, true])
            .map(|((aop, sat), cross)| Self::simple_instr(ifam, sat, cross, aop))
            .collect();

        let mut dual_instrs: Vec<InstrBuilder> = [false, true]
            .into_iter()
            .cartesian_product([Aop::AA, Aop::SA, Aop::AS])
            .cartesian_product([false, true])
            .cartesian_product([false, true])
            .map(|(((hl, aop), sat), cross)| Self::dual_instr(ifam, sat, cross, aop, hl))
            .collect();

        simple_instrs.append(&mut dual_instrs);

        simple_instrs
    }
}
