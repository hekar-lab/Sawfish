use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

pub struct AddSubAc0Factory();

impl AddSubAc0Factory {
    fn base_instr(ifam: &InstrFamilyBuilder, sub: bool, sat: bool) -> InstrBuilder {
        let op = if sub { e_sub } else { e_add };
        InstrBuilder::new(ifam)
            .name("AddSubAC0")
            .display(format!(
                "{{dst0}} = {{src0}} {} {{src1}} + AC0{} {}",
                if sub { "-" } else { "+" },
                if sub { " - 1" } else { "" },
                if sat { "(S)" } else { "(NS)" }
            ))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0x19))
            .set_field_type("aop", FieldType::Mask(sub as u16))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_copy(
                e_local("result", 5),
                op(e_sext(e_rfield("src0")), e_sext(e_rfield("src1"))),
            ))
            .add_pcode(if sub {
                cs_assign_by(e_sub, b_var("result"), e_zext(e_not(b_reg("AC0"))))
            } else {
                cs_assign_by(e_add, b_var("result"), e_zext(b_reg("AC0")))
            })
            .add_pcode(if sat {
                cs_strunc_sat(e_rfield("dst0"), b_var("result"), 4, "asac0")
            } else {
                e_copy(e_rfield("dst0"), b_size(b_var("result"), 4))
            })
    }
}

impl InstrFactory for AddSubAc0Factory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::base_instr(ifam, false, false),
            Self::base_instr(ifam, false, true),
            Self::base_instr(ifam, true, false),
            Self::base_instr(ifam, true, true),
        ]
    }
}
