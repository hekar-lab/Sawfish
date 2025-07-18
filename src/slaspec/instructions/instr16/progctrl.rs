use crate::slaspec::{
    globals::IMASK,
    instructions::{
        core::{InstrBuilder, InstrFactory, InstrFamilyBuilder},
        pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet},
    },
};

use crate::slaspec::instructions::expr_util::*;

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "ProgCtrl",
        "Basic Program Sequencer Control Functions",
        "pgc",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x00), 8),
            ProtoField::new("opc", FieldType::Blank, 4),
            ProtoField::new("reg", FieldType::Blank, 4),
        ]),
    );

    ifam.add_pcodeop("idle");
    ifam.add_pcodeop("csync");
    ifam.add_pcodeop("ssync");
    ifam.add_pcodeop("emuexcpt");
    ifam.add_pcodeop("raise");
    ifam.add_pcodeop("excpt");

    ifam.add_instrs(&ReturnFactory());
    ifam.add_instrs(&SyncModeFactory());
    ifam.add_instrs(&IMaskFactory());
    ifam.add_instrs(&JumpFactory());
    ifam.add_instrs(&CallFactory());
    ifam.add_instrs(&RaiseFactory());
    ifam.add_instrs(&TestSetFactory());
    ifam.add_instrs(&SyncFactory());

    ifam
}

struct ReturnFactory();

impl ReturnFactory {
    fn instr_rt(ifam: &InstrFamilyBuilder, retreg: char, regmask: u16) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .set_field_type("opc", FieldType::Mask(0x01))
            .set_field_type("reg", FieldType::Mask(regmask))
            .name("Return")
            .display(format!("RT{retreg}"))
            .add_pcode(e_ret(b_reg(&format!("RET{retreg}"))))
    }
}

impl InstrFactory for ReturnFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut instrs = Vec::new();
        let retregs = "SIXNE";
        let mut regmask = 0x0;
        for c in retregs.chars() {
            instrs.push(Self::instr_rt(&ifam, c, regmask));
            regmask += 1;
        }

        instrs
    }
}

struct SyncModeFactory();

impl SyncModeFactory {
    fn template_instr(
        ifam: &InstrFamilyBuilder,
        reg: u16,
        name: &str,
        pcodeop: &str,
    ) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .set_field_type("opc", FieldType::Mask(0x02))
            .set_field_type("reg", FieldType::Mask(reg))
            .name(name)
            .display(pcodeop.to_uppercase())
            .add_pcode(e_mac(pcodeop))
    }
}

impl InstrFactory for SyncModeFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let instr_vals = [
            (0x0, "Sync", "idle"),
            (0x3, "Sync", "csync"),
            (0x4, "Sync", "ssync"),
            (0x5, "Mode", "emuexcpt"),
        ];

        instr_vals
            .into_iter()
            .map(|(reg, name, pcodeop)| Self::template_instr(ifam, reg, name, pcodeop))
            .collect()
    }
}

fn reg_instr(instr: InstrBuilder) -> InstrBuilder {
    instr.split_field(
        "reg",
        ProtoPattern::new(vec![
            ProtoField::new("regH", FieldType::Mask(0x0), 1),
            ProtoField::new("regL", FieldType::Blank, 3),
        ]),
    )
}

struct IMaskFactory();

impl IMaskFactory {
    const IMASK_VAR: &'static str = "imaskAddr";

    fn base_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
        reg_instr(InstrBuilder::new(ifam))
            .name("IMaskMv")
            .set_field_type("regL", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_copy(e_local(IMaskFactory::IMASK_VAR, 4), b_var(IMASK)))
    }
}

impl InstrFactory for IMaskFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            IMaskFactory::base_instr(ifam)
                .set_field_type("opc", FieldType::Mask(0x3))
                .display("CLI {regL}".to_string())
                .add_pcode(e_copy(
                    e_rfield("regL"),
                    e_ptr(b_var(IMaskFactory::IMASK_VAR), 4),
                ))
                .add_pcode(e_copy(e_ptr(b_var(IMaskFactory::IMASK_VAR), 4), b_num(0))),
            IMaskFactory::base_instr(ifam)
                .set_field_type("opc", FieldType::Mask(0x4))
                .display("STI {regL}".to_string())
                .add_pcode(e_copy(
                    e_ptr(b_var(IMaskFactory::IMASK_VAR), 4),
                    e_rfield("regL"),
                )),
        ]
    }
}

fn goto_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
    reg_instr(InstrBuilder::new(ifam))
        .set_field_type("regL", FieldType::Variable(RegisterSet::PReg))
}

struct JumpFactory();

impl JumpFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, pc: bool) -> InstrBuilder {
        goto_instr(ifam)
            .name("Jump")
            .display(format!("JUMP ({}{{regL}})", if pc { "PC + " } else { "" }))
            .add_pcode(b_goto(b_indirect(if pc {
                e_add(e_rfield("regL"), b_reg("PC"))
            } else {
                e_rfield("regL")
            })))
    }
}

impl InstrFactory for JumpFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::base_instr(ifam, false).set_field_type("opc", FieldType::Mask(0x5)),
            Self::base_instr(ifam, true).set_field_type("opc", FieldType::Mask(0x8)),
        ]
    }
}

struct CallFactory();

impl CallFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, pc: bool) -> InstrBuilder {
        goto_instr(ifam)
            .name("Call")
            .display(format!("CALL ({}{{regL}})", if pc { "PC + " } else { "" }))
            .add_pcode(e_copy(b_reg("RETS"), b_var("inst_next")))
            .add_pcode(e_call(if pc {
                e_add(e_rfield("regL"), b_reg("PC"))
            } else {
                e_rfield("regL")
            }))
    }
}

impl InstrFactory for CallFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::base_instr(ifam, false).set_field_type("opc", FieldType::Mask(0x6)),
            Self::base_instr(ifam, true).set_field_type("opc", FieldType::Mask(0x7)),
        ]
    }
}

struct RaiseFactory();

impl RaiseFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, opc_mask: u16, op: &str) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .set_field_type("reg", FieldType::UImmVal)
            .set_field_type("opc", FieldType::Mask(opc_mask))
            .name("Raise")
            .display(format!("{} {{reg}}", op.to_uppercase()))
            .add_pcode(e_macp(op, b_size(e_field("reg"), 1)))
    }
}

impl InstrFactory for RaiseFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::base_instr(ifam, 0x9, "raise"),
            Self::base_instr(ifam, 0xa, "excpt"),
        ]
    }
}

struct TestSetFactory();

impl InstrFactory for TestSetFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            reg_instr(InstrBuilder::new(ifam))
                .set_field_type("regL", FieldType::Variable(RegisterSet::PReg))
                .set_field_type("opc", FieldType::Mask(0xb))
                .name("TestSet")
                .display("TESTSET ({regL})".to_string())
                .add_pcode(e_copy(e_local("testVal", 1), e_ptr(e_rfield("regL"), 1)))
                .add_pcode(e_copy(b_reg("CC"), b_num(0x0)))
                .add_pcode(b_ifgoto(
                    e_ne(b_var("testVal"), b_num(0x0)),
                    b_label("is_set"),
                ))
                .add_pcode(e_copy(b_reg("CC"), b_num(0x1)))
                .add_pcode(b_label("is_set"))
                .add_pcode(e_copy(
                    e_ptr(e_rfield("regL"), 1),
                    e_bit_or(b_var("testVal"), b_num(0x80)),
                )),
        ]
    }
}

struct SyncFactory();

impl InstrFactory for SyncFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            reg_instr(InstrBuilder::new(ifam))
                .set_field_type("regL", FieldType::Variable(RegisterSet::DReg))
                .set_field_type("opc", FieldType::Mask(0xc))
                .name("Sync")
                .display("STI IDLE {regL}".to_string())
                .add_pcode(e_copy(e_local(IMaskFactory::IMASK_VAR, 4), b_var(IMASK)))
                .add_pcode(e_copy(
                    e_ptr(b_var(IMaskFactory::IMASK_VAR), 4),
                    e_rfield("regL"),
                ))
                .add_pcode(e_mac("idle")),
        ]
    }
}
