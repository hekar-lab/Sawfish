use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_64(
        "Jump32",
        "Jump/Call to 32-bit Immediate",
        "jmp",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0x37), 6),
                ProtoField::new("c", FieldType::Blank, 1),
                ProtoField::new("mask8z", FieldType::Mask(0x0), 8),
                ProtoField::new("rel", FieldType::Blank, 1),
            ]),
            ProtoPattern::new(vec![ProtoField::new("immH", FieldType::Blank, 16)]),
            ProtoPattern::new(vec![ProtoField::new("immL", FieldType::Blank, 16)]),
            ProtoPattern::new(vec![ProtoField::new(
                "emotionalSupportBits",
                FieldType::Any,
                16,
            )]),
        ],
    );

    ifam.add_instrs(&JumpFactory());

    ifam
}

struct JumpFactory();

impl JumpFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, call: bool, rel: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(if call { "Call" } else { "JumpAbs" })
            .display(format!(
                "{}{} {{$addr}}",
                if call { "CALL" } else { "JUMP" },
                if rel { "" } else { ".A" }
            ))
            .set_field_type("c", FieldType::Mask(call as u16))
            .set_field_type("rel", FieldType::Mask(rel as u16))
            .set_field_type(
                "immH",
                if rel {
                    FieldType::SImmVal
                } else {
                    FieldType::UImmVal
                },
            )
            .set_field_type("immL", FieldType::UImmVal)
            .add_action(e_copy(
                b_var("addr"),
                e_bit_or(
                    b_grp(e_lshft(e_rfield("immH"), b_num(16))),
                    e_rfield("immL"),
                ),
            ))
            .add_pcode(if call {
                cs_mline(vec![
                    e_copy(b_reg("RETS"), b_var("inst_next")),
                    e_call(e_ptr(b_size(b_var("addr"), 4), 4)),
                ])
            } else {
                b_goto(b_indirect(e_ptr(b_size(b_var("addr"), 4), 4)))
            })
    }
}

impl InstrFactory for JumpFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [false, true]
            .into_iter()
            .cartesian_product([false, true])
            .map(|(call, rel)| Self::base_instr(ifam, call, rel))
            .collect()
    }
}
