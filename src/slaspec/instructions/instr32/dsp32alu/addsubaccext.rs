use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

pub struct AddSubAccExtFactory();

impl AddSubAccExtFactory {
    fn aae_instr(ifam: &InstrFamilyBuilder, aop: bool, hl: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("AddAccExt")
            .display("{dst0} = (A0 += A1)".to_string())
            .set_field_type("hl", FieldType::Mask((aop && hl) as u16))
            .set_field_type("aopc", FieldType::Mask(0xb))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type(
                "dst0",
                FieldType::Variable(if aop {
                    if hl {
                        RegisterSet::DRegH
                    } else {
                        RegisterSet::DRegL
                    }
                } else {
                    RegisterSet::DReg
                }),
            )
            .add_pcode(cs_sadd_sat(b_reg("A0"), b_reg("A0"), b_reg("A1"), 5, "aae"))
            .add_pcode(if aop {
                cs_mline(vec![
                    e_local("A0_trunc", 4),
                    cs_strunc_sat(b_var("A0_trunc"), b_reg("A0"), 4, "aae"),
                    cs_round(e_rfield("dst0"), 2, b_var("A0_trunc"), 4, "aae"),
                ])
            } else {
                cs_strunc_sat(e_rfield("dst0"), b_reg("A0"), 4, "aae")
            })
    }

    fn asa_instr(ifam: &InstrFamilyBuilder, sub: bool, sat: bool) -> InstrBuilder {
        let arithm_op = if sub { cs_ssub_sat } else { cs_sadd_sat };
        InstrBuilder::new(ifam)
            .name("AddSubAcc")
            .display(format!(
                "(A0 {}= A1){}",
                if sub { "-" } else { "+" },
                if sat { " (W32)" } else { "" }
            ))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0xb))
            .set_field_type("aop", FieldType::Mask(0x2 + sub as u16))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(0x0))
            .add_pcode(if sat {
                cs_mline(vec![
                    arithm_op(b_reg("A0"), b_reg("A0"), b_reg("A1"), 5, "asa"),
                    e_local("A0_trunc", 4),
                    cs_strunc_sat(b_var("A0_trunc"), b_reg("A0"), 4, "asa"),
                    e_copy(b_reg("A0"), e_sext(b_var("A0_trunc"))),
                ])
            } else {
                arithm_op(b_reg("A0"), b_reg("A0"), b_reg("A1"), 5, "asa")
            })
    }

    fn asae_instr(ifam: &InstrFamilyBuilder, aop: bool, sat: bool) -> InstrBuilder {
        let lhs = if aop { "A0" } else { "A1" };
        let rhs = if aop { "A1" } else { "A0" };
        InstrBuilder::new(ifam)
            .name("AddSubAccExt")
            .display(format!(
                "{{dst0}} = {lhs} + {rhs}, {{dst1}} = {lhs} - {rhs} ({})",
                if sat { "S" } else { "NS" }
            ))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0x11))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("dst1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(if sat {
                cs_mline(vec![
                    e_local("res_dst0", 5),
                    cs_sadd_sat(b_var("res_dst0"), b_reg(lhs), b_reg(rhs), 5, "asae_dst0"),
                    cs_strunc_sat(e_rfield("dst0"), b_var("res_dst0"), 4, "asae_dst0"),
                ])
            } else {
                cs_mline(vec![
                    e_local("res_dst0", 5),
                    e_copy(b_var("res_dst0"), e_add(b_reg(lhs), b_reg(rhs))),
                    e_copy(e_rfield("dst0"), b_size(b_var("res_dst0"), 4)),
                ])
            })
            .add_pcode(if sat {
                cs_mline(vec![
                    e_local("res_dst1", 5),
                    cs_ssub_sat(b_var("res_dst1"), b_reg(lhs), b_reg(rhs), 5, "asae_dst1"),
                    cs_strunc_sat(e_rfield("dst1"), b_var("res_dst1"), 4, "asae_dst1"),
                ])
            } else {
                cs_mline(vec![
                    e_local("res_dst1", 5),
                    e_copy(b_var("res_dst1"), e_sub(b_reg(lhs), b_reg(rhs))),
                    e_copy(e_rfield("dst1"), b_size(b_var("res_dst1"), 4)),
                ])
            })
    }
}

impl InstrFactory for AddSubAccExtFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::aae_instr(ifam, false, false),
            Self::aae_instr(ifam, true, false),
            Self::aae_instr(ifam, true, true),
            Self::asa_instr(ifam, false, false),
            Self::asa_instr(ifam, false, true),
            Self::asa_instr(ifam, true, false),
            Self::asa_instr(ifam, true, true),
            Self::asae_instr(ifam, false, false),
            Self::asae_instr(ifam, false, true),
            Self::asae_instr(ifam, true, false),
            Self::asae_instr(ifam, true, true),
        ]
    }
}
