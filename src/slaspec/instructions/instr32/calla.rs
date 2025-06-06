use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "CallA",
        "Call function with pcrel address",
        "cla",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0x71), 7),
                ProtoField::new("s", FieldType::Blank, 1),
                ProtoField::new("swH", FieldType::SImmVal, 8),
            ]),
            ProtoPattern::new(vec![ProtoField::new("swL", FieldType::UImmVal, 16)]),
        ],
    );

    ifam.add_instrs(&CallAFactory());

    ifam
}

struct CallAFactory();

impl CallAFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, call: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(if call { "Call" } else { "JumpAbs" })
            .display(format!(
                "{} {{$addr}}",
                if call { "CALL" } else { "JUMP.L" }
            ))
            .set_field_type("s", FieldType::Mask(call as u16))
            .add_action(e_copy(
                b_var("addr"),
                e_add(
                    b_var("inst_start"),
                    e_mult(
                        b_grp(e_bit_or(
                            b_grp(e_lshft(e_rfield("swH"), b_num(16))),
                            e_rfield("swL"),
                        )),
                        b_num(2),
                    ),
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

impl InstrFactory for CallAFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![Self::base_instr(ifam, false), Self::base_instr(ifam, true)]
    }
}
