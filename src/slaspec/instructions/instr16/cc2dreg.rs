use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "CC2Dreg",
        "Move CC conditional bit, to and from Dreg",
        "c2d",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x010), 11),
            ProtoField::new("opc", FieldType::Blank, 2),
            ProtoField::new("reg", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&CCFactory());

    ifam
}

struct CCFactory();

impl InstrFactory for CCFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            InstrBuilder::new(ifam)
                .set_field_type("opc", FieldType::Mask(0x0))
                .set_field_type("reg", FieldType::Variable(RegisterSet::DReg))
                .name("CCToDreg")
                .display("{reg} = CC".to_string())
                .add_pcode(e_copy(e_rfield("reg"), e_macp("zext", b_reg("CC")))),
            InstrBuilder::new(ifam)
                .set_field_type("opc", FieldType::Mask(0x1))
                .set_field_type("reg", FieldType::Variable(RegisterSet::DReg))
                .name("MvToCC")
                .display("CC = {reg}".to_string())
                .add_pcode(e_copy(b_reg("CC"), e_ne(e_rfield("reg"), b_num(0)))),
            InstrBuilder::new(ifam)
                .set_field_type("opc", FieldType::Mask(0x2))
                .set_field_type("reg", FieldType::Variable(RegisterSet::DReg))
                .name("CCToDreg")
                .display("{reg} = !CC".to_string())
                .add_pcode(e_copy(e_rfield("reg"), e_macp("zext", e_not(b_reg("CC"))))),
            InstrBuilder::new(ifam)
                .set_field_type("opc", FieldType::Mask(0x3))
                .name("MvToCC")
                .display("CC = !CC".to_string())
                .add_pcode(e_copy(b_reg("CC"), e_not(b_reg("CC")))),
        ]
    }
}
