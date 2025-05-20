use std::collections::VecDeque;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "PushPopMult",
        "Push or Pop Multiple contiguous registers",
        "ppm",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x02), 7),
            ProtoField::new("d", FieldType::Blank, 1),
            ProtoField::new("p", FieldType::Blank, 1),
            ProtoField::new("w", FieldType::Blank, 1),
            ProtoField::new("dr", FieldType::Blank, 3),
            ProtoField::new("pr", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&PushPopFactory());

    ifam
}

struct PushPopFactory();

impl PushPopFactory {
    fn range_display(dreg: Option<u16>, preg: Option<u16>) -> String {
        let dstr = if let Some(reg) = dreg {
            format!("R7:{}", reg)
        } else {
            String::new()
        };
        let pstr = if let Some(reg) = preg {
            format!("P5:{}", reg)
        } else {
            String::new()
        };

        if dstr.is_empty() || pstr.is_empty() {
            format!("({}{})", dstr, pstr)
        } else {
            format!("({}, {})", dstr, pstr)
        }
    }

    fn push_display(dreg: Option<u16>, preg: Option<u16>) -> String {
        format!("[--SP] = {}", Self::range_display(dreg, preg))
    }

    fn pop_display(dreg: Option<u16>, preg: Option<u16>) -> String {
        format!("{} = [SP++]", Self::range_display(dreg, preg))
    }

    fn dreg_init(instr: InstrBuilder, dreg: Option<u16>, push: bool) -> InstrBuilder {
        if dreg.is_none() {
            return instr;
        }
        let reg = dreg.unwrap();

        let op = if push { cs_push } else { cs_pop };
        let range: Vec<u16> = if push {
            (reg..8).collect()
        } else {
            (reg..8).rev().collect()
        };

        instr
            .set_field_type("dr", FieldType::Mask(reg))
            .add_pcode(cs_mline(
                range
                    .iter()
                    .map(|i| op(b_reg(&format!("R{i}")), 4))
                    .collect::<VecDeque<Expr>>(),
            ))
    }

    fn preg_init(instr: InstrBuilder, preg: Option<u16>, push: bool) -> InstrBuilder {
        if preg.is_none() {
            return instr;
        }
        let reg = preg.unwrap();

        let op = if push { cs_push } else { cs_pop };
        let range: Vec<u16> = if push {
            (reg..6).collect()
        } else {
            (reg..6).rev().collect()
        };

        instr
            .set_field_type("pr", FieldType::Mask(reg))
            .add_pcode(cs_mline(
                range
                    .iter()
                    .map(|i| op(b_reg(&format!("P{i}")), 4))
                    .collect::<VecDeque<Expr>>(),
            ))
    }

    fn base_instr(
        ifam: &InstrFamilyBuilder,
        dreg: Option<u16>,
        preg: Option<u16>,
        push: bool,
    ) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name("PushPopMul16")
            .set_field_type("d", FieldType::Mask(!dreg.is_none() as u16))
            .set_field_type("p", FieldType::Mask(!preg.is_none() as u16))
            .set_field_type("w", FieldType::Mask(push as u16))
            .display(if push {
                Self::push_display(dreg, preg)
            } else {
                Self::pop_display(dreg, preg)
            });

        if push {
            instr = Self::dreg_init(instr, dreg, push);
            instr = Self::preg_init(instr, preg, push);
        } else {
            instr = Self::preg_init(instr, preg, push);
            instr = Self::dreg_init(instr, dreg, push);
        }

        instr
    }
}

impl InstrFactory for PushPopFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut instrs = Vec::new();

        for i in 0..8 {
            instrs.push(Self::base_instr(ifam, Some(i), None, true));
            instrs.push(Self::base_instr(ifam, Some(i), None, false));
        }

        for i in 0..6 {
            instrs.push(Self::base_instr(ifam, None, Some(i), true));
            instrs.push(Self::base_instr(ifam, None, Some(i), false));
        }

        for i in 0..8 {
            for j in 0..6 {
                instrs.push(Self::base_instr(ifam, Some(i), Some(j), true));
                instrs.push(Self::base_instr(ifam, Some(i), Some(j), false));
            }
        }

        instrs
    }
}
