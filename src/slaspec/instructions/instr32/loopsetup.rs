use std::fmt;

use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "LoopSetup",
        "Virtually Zero Overhead Loop Mechanism",
        "lps",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0x1c1), 9),
                ProtoField::new("rop", FieldType::Blank, 2),
                ProtoField::new("c", FieldType::Blank, 1),
                ProtoField::new("soff", FieldType::UImmVal, 4),
            ]),
            ProtoPattern::new(vec![
                ProtoField::new("imm", FieldType::Blank, 1),
                ProtoField::new("reg", FieldType::Blank, 3),
                ProtoField::new("lop", FieldType::Blank, 2),
                ProtoField::new("eoff", FieldType::UImmVal, 10),
            ]),
        ],
    );

    ifam.add_instrs(&LoopSetupFactory());

    ifam
}

#[derive(Debug, Clone, Copy)]
enum Lop {
    LSETUP = 0x0,
    LSETUPZ = 0x1,
    LSETUPLEZ = 0x2,
}

impl Lop {
    fn default(&self) -> bool {
        match self {
            Lop::LSETUP => true,
            _ => false,
        }
    }
}

impl fmt::Display for Lop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy)]
enum Rop {
    NoLC = 0x0,
    RegLC = 0x1,
    ShftRegLC = 0x3,
}

impl Rop {
    fn display(&self) -> String {
        match self {
            Rop::NoLC => "",
            Rop::RegLC => " = {reg}",
            Rop::ShftRegLC => " = {reg} >> 1",
        }
        .to_string()
    }

    fn reg(&self) -> bool {
        match self {
            Rop::RegLC | Rop::ShftRegLC => true,
            _ => false,
        }
    }
}

struct LoopSetupFactory();

impl LoopSetupFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, lop: Lop, rop: Rop, loop_id: bool) -> InstrBuilder {
        let loop_act = if loop_id {
            "loop1active"
        } else {
            "loop0active"
        };
        let lt = if loop_id { "LT1" } else { "LT0" };
        let lb = if loop_id { "LB1" } else { "LB0" };
        let lc = if loop_id { "LC1" } else { "LC0" };

        let mut instr = InstrBuilder::new(ifam)
            .name("LoopSetup")
            .display(format!(
                "{lop} ({}{{$endImm}}) {{cReg}}{}",
                if lop.default() { "{$startImm}, " } else { "" },
                rop.display()
            ))
            .set_field_type("rop", FieldType::Mask(rop as u16))
            .set_field_type_opt(rop.reg(), "reg", FieldType::Variable(RegisterSet::PReg))
            .set_field_type("lop", FieldType::Mask(lop as u16))
            .divide_field(
                "c",
                ProtoPattern::new(vec![
                    ProtoField::new("cReg", FieldType::Variable(RegisterSet::LC), 1),
                    ProtoField::new("cMsk", FieldType::Mask(loop_id as u16), 1),
                ]),
            )
            .add_action(e_copy(
                b_var("endImm"),
                e_add(e_mult(e_rfield("eoff"), b_num(2)), b_var("inst_start")),
            ))
            .add_action(e_copy(b_var(loop_act), b_num(1)))
            .add_action(e_mac2p("globalset", b_var("endImm"), b_var(loop_act)))
            .add_action(if lop.default() {
                e_copy(
                    b_var("startImm"),
                    e_add(e_mult(e_rfield("soff"), b_num(2)), b_var("inst_start")),
                )
            } else {
                cs_mline(vec![
                    e_copy(b_var("zloop"), b_num(1)),
                    e_mac2p("globalset", b_var("endImm"), b_var("zloop")),
                ])
            })
            .add_pcode(if lop.default() {
                e_copy(b_reg(lt), b_var("startImm"))
            } else {
                e_copy(b_reg(lt), b_var("inst_next"))
            })
            .add_pcode(e_copy(b_reg(lb), b_var("endImm")));

        instr = match rop {
            Rop::RegLC => instr.add_pcode(e_copy(b_reg(lc), e_rfield("reg"))),
            Rop::ShftRegLC => {
                instr.add_pcode(e_copy(b_reg(lc), e_rshft(e_rfield("reg"), b_num(1))))
            }
            _ => instr,
        };

        instr = match lop {
            Lop::LSETUPZ => instr
                .add_pcode(b_ifgoto(e_gt(b_reg(lc), b_num(0)), b_label("end_setup")))
                .add_pcode(b_goto(b_indirect(b_size(b_var("endImm"), 4))))
                .add_pcode(b_label("end_setup")),
            Lop::LSETUPLEZ => instr
                .add_pcode(b_ifgoto(e_gts(b_reg(lc), b_num(0)), b_label("end_setup")))
                .add_pcode(e_copy(b_reg(lc), b_num(0)))
                .add_pcode(b_goto(b_indirect(b_size(b_var("endImm"), 4))))
                .add_pcode(b_label("end_setup")),
            _ => instr,
        };

        instr
    }
}

impl InstrFactory for LoopSetupFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let loop_instr: Vec<InstrBuilder> = [Rop::NoLC, Rop::RegLC, Rop::ShftRegLC]
            .into_iter()
            .cartesian_product([false, true])
            .map(|(rop, loop_id)| Self::base_instr(ifam, Lop::LSETUP, rop, loop_id))
            .collect();

        let loopz_instr: Vec<InstrBuilder> = [Lop::LSETUPZ, Lop::LSETUPLEZ]
            .into_iter()
            .cartesian_product([Rop::RegLC, Rop::ShftRegLC])
            .cartesian_product([false, true])
            .map(|((lop, rop), loop_id)| Self::base_instr(ifam, lop, rop, loop_id))
            .collect();

        vec![loop_instr, loopz_instr].concat()
    }
}
