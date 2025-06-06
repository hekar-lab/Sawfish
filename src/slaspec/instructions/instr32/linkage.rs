use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "Linkage",
        "Save/restore registers and link/unlink frame, multiple cycles",
        "cla",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0x7400), 15),
                ProtoField::new("r", FieldType::Blank, 1),
            ]),
            ProtoPattern::new(vec![ProtoField::new("frm", FieldType::Blank, 16)]),
        ],
    );

    ifam.add_instrs(&LinkageFactory());

    ifam
}

struct LinkageFactory();

impl LinkageFactory {
    fn link_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("Linkage")
            .display("LINK {$imm}".to_string())
            .set_field_type("r", FieldType::Mask(0x0))
            .set_field_type("frm", FieldType::UImmVal)
            .add_action(e_copy(b_var("imm"), e_mult(e_rfield("frm"), b_num(4))))
            .add_pcode(cs_push(b_reg("RETS"), 4))
            .add_pcode(cs_push(b_reg("FP"), 4))
            .add_pcode(e_copy(b_reg("FP"), b_reg("SP")))
            .add_pcode(e_copy(b_reg("SP"), e_sub(b_reg("sub"), b_var("imm"))))
    }

    fn unlink_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("Linkage")
            .display("UNLINK".to_string())
            .set_field_type("r", FieldType::Mask(0x1))
            .add_pcode(e_copy(b_reg("SP"), b_reg("FP")))
            .add_pcode(cs_push(b_reg("FP"), 4))
            .add_pcode(cs_push(b_reg("RETS"), 4))
    }
}

impl InstrFactory for LinkageFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![Self::link_instr(ifam), Self::unlink_instr(ifam)]
    }
}
