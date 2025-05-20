use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "UJump",
        "Unconditional Branch PC relative with 12bit offset",
        "ujp",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x2), 4),
            ProtoField::new("off", FieldType::SImmVal, 12),
        ]),
    );

    ifam.add_instrs(&JumpAbsFactory());

    ifam
}

struct JumpAbsFactory();

impl InstrFactory for JumpAbsFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let addr_var = "addr";
        vec![
            InstrBuilder::new(ifam)
                .name("JumpAbs")
                .display(format!("JUMP.S {{${addr_var}}}"))
                .add_action(e_copy(
                    b_var(addr_var),
                    e_add(b_var("inst_start"), e_mult(b_field("off"), b_num(2))),
                ))
                .add_pcode(b_goto(b_indirect(b_ptr(b_size(b_var(addr_var), 4), 4)))),
        ]
    }
}
