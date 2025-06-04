use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub struct MiscFactory();

impl MiscFactory {
    fn aos_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("AddOnSign")
            .display(
                "{dst0L} = {dst0H} = SIGN({src0L}) * {src1L} + SIGN({src0H}) * {src1H}".to_string(),
            )
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0xc))
            .set_field_type("aop", FieldType::Mask(0x0))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(0x0))
            .divide_field(
                "dst0",
                ProtoPattern::new(vec![
                    ProtoField::new("dst0L", FieldType::Variable(RegisterSet::DRegL), 3),
                    ProtoField::new("dst0H", FieldType::Variable(RegisterSet::DRegH), 3),
                ]),
            )
            .divide_field(
                "src0",
                ProtoPattern::new(vec![
                    ProtoField::new("src0L", FieldType::Variable(RegisterSet::DRegL), 3),
                    ProtoField::new("src0H", FieldType::Variable(RegisterSet::DRegH), 3),
                ]),
            )
            .divide_field(
                "src1",
                ProtoPattern::new(vec![
                    ProtoField::new("src1L", FieldType::Variable(RegisterSet::DRegL), 3),
                    ProtoField::new("src1H", FieldType::Variable(RegisterSet::DRegH), 3),
                ]),
            )
            .add_pcode(e_copy(
                e_local("signL", 2),
                e_bit_or(e_arshft(e_rfield("src0L"), b_num(14)), b_num(1)),
            ))
            .add_pcode(e_copy(e_local("magL", 2), e_rfield("src1L")))
            .add_pcode(e_copy(
                e_local("signH", 2),
                e_bit_or(e_arshft(e_rfield("src0H"), b_num(14)), b_num(1)),
            ))
            .add_pcode(e_copy(e_local("magH", 2), e_rfield("src1H")))
            .add_pcode(e_copy(
                e_local("res_aos", 2),
                e_add(
                    e_mult(b_var("signL"), b_var("magL")),
                    e_mult(b_var("signH"), b_var("magH")),
                ),
            ))
            .add_pcode(e_copy(e_rfield("dst0L"), b_var("res_aos")))
            .add_pcode(e_copy(e_rfield("dst0H"), b_var("res_aos")))
    }

    fn aah_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("AddAccHalf")
            .display("{dst0} = A1.L + A1.H, {dst1} = A0.L + A0.H".to_string())
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0xc))
            .set_field_type("aop", FieldType::Mask(0x1))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("dst1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_copy(
                e_rfield("dst0"),
                e_add(e_sext(b_reg("A1.L")), e_sext(b_reg("A1.H"))),
            ))
            .add_pcode(e_copy(
                e_rfield("dst1"),
                e_add(e_sext(b_reg("A0.L")), e_sext(b_reg("A0.H"))),
            ))
    }

    fn prnd_instr(ifam: &InstrFamilyBuilder, hl: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("Pass32Rnd16")
            .display("{dst0} = {src0} (RND)".to_string())
            .set_field_type("hl", FieldType::Mask(hl as u16))
            .set_field_type("aopc", FieldType::Mask(0xc))
            .set_field_type("aop", FieldType::Mask(0x3))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type(
                "dst0",
                FieldType::Variable(if hl {
                    RegisterSet::DRegH
                } else {
                    RegisterSet::DRegL
                }),
            )
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(cs_round_biased(
                e_rfield("dst0"),
                2,
                e_rfield("src0"),
                4,
                "prnd",
            ))
    }
}

impl InstrFactory for MiscFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::aos_instr(ifam),
            Self::aah_instr(ifam),
            Self::prnd_instr(ifam, false),
            Self::prnd_instr(ifam, true),
        ]
    }
}
