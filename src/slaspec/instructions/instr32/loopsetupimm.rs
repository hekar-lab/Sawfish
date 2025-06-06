use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "LoopSetupImm",
        "Virtually Zero Overhead Loop Mechanism with Immediate Values",
        "lpi",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0x1c1), 9),
                ProtoField::new("rop", FieldType::Mask(0x2), 2),
                ProtoField::new("c", FieldType::Blank, 1),
                ProtoField::new("immH", FieldType::UImmVal, 4),
            ]),
            ProtoPattern::new(vec![
                ProtoField::new("immL", FieldType::UImmVal, 6),
                ProtoField::new("eoff", FieldType::UImmVal, 10),
            ]),
        ],
    );

    ifam.add_instrs(&LoopSetupImmFactory());

    ifam
}

struct LoopSetupImmFactory();

impl LoopSetupImmFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, loop_id: bool) -> InstrBuilder {
        let loop_act = if loop_id {
            "loop1active"
        } else {
            "loop0active"
        };
        let lt = if loop_id { "LT1" } else { "LT0" };
        let lb = if loop_id { "LB1" } else { "LB0" };
        let lc = if loop_id { "LC1" } else { "LC0" };

        InstrBuilder::new(ifam)
            .name("LoopSetup")
            .display(format!("LSETUP ({{$endImm}}) {{cReg}} = {{$lcImm}}"))
            .divide_field(
                "c",
                ProtoPattern::new(vec![
                    ProtoField::new("cReg", FieldType::Variable(RegisterSet::LC), 1),
                    ProtoField::new("cMsk", FieldType::Mask(loop_id as u16), 1),
                ]),
            )
            .add_action(e_copy(
                b_var("lcImm"),
                e_bit_or(b_grp(e_lshft(e_rfield("immH"), b_num(6))), e_rfield("immL")),
            ))
            .add_action(e_copy(
                b_var("endImm"),
                e_add(e_mult(e_rfield("eoff"), b_num(2)), b_var("inst_start")),
            ))
            .add_action(e_copy(b_var(loop_act), b_num(1)))
            .add_action(e_mac2p("globalset", b_var("endImm"), b_var(loop_act)))
            .add_pcode(e_copy(b_reg(lt), b_var("startImm")))
            .add_pcode(e_copy(b_reg(lb), b_var("endImm")))
            .add_pcode(e_copy(b_reg(lc), b_var("lcImm")))
    }
}

impl InstrFactory for LoopSetupImmFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![Self::base_instr(ifam, false), Self::base_instr(ifam, true)]
    }
}
