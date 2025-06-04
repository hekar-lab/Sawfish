use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

pub struct MvAccRegFactory();

impl MvAccRegFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, acc: u16) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("MvAxXToDregL")
            .display(format!("{{dst0}} = A{acc}.X"))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0xa))
            .set_field_type("aop", FieldType::Mask(acc))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DRegL))
            .add_pcode(e_copy(
                e_rfield("dst0"),
                e_sext(b_reg(&format!("A{acc}.X"))),
            ))
    }
}

impl InstrFactory for MvAccRegFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![Self::base_instr(ifam, 0), Self::base_instr(ifam, 1)]
    }
}
