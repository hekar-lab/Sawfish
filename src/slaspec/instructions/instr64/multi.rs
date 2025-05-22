use std::collections::HashSet;

use itertools::Itertools;

use crate::slaspec::instructions::{
    core::{InstrBuilder, InstrFactory, InstrFamilyBuilder, Prefixed},
    expr::Code,
    expr_util::{b_local, b_reg, b_var, e_copy, e_rfield},
    format::display_add_prefix,
    instr16::*,
    instr32::*,
    pattern::{FieldType, Pattern, ProtoField, ProtoPattern},
};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_64(
        "Multi",
        "64-bit Instruction Shell",
        "m",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0x19), 5),
                ProtoField::new("H32bit", FieldType::Blank, 11),
            ]),
            ProtoPattern::new(vec![ProtoField::new("L32bit", FieldType::Blank, 16)]),
            ProtoPattern::new(vec![ProtoField::new("A16bit", FieldType::Blank, 16)]),
            ProtoPattern::new(vec![ProtoField::new("B16bit", FieldType::Blank, 16)]),
        ],
    );

    ifam.add_instrs(&MultiFactory());

    ifam
}

struct MultiFactory();

impl MultiFactory {
    fn multi_name(
        instr32: &InstrBuilder,
        instr16a: &InstrBuilder,
        instr16b: &InstrBuilder,
    ) -> String {
        vec![instr32.get_name(), instr16a.get_name(), instr16b.get_name()].join(" || ")
    }

    fn multi_display(
        ifam32: &InstrFamilyBuilder,
        instr32: &InstrBuilder,
        ifam16a: &InstrFamilyBuilder,
        instr16a: &InstrBuilder,
        ifam16b: &InstrFamilyBuilder,
        instr16b: &InstrBuilder,
    ) -> String {
        let mut displays = Vec::new();

        if instr32.get_name() != "NOP32" {
            displays.push(display_add_prefix(&instr32.get_display(), &ifam32.prefix()));
        }

        if instr16a.get_name() != "NOP" {
            displays.push(display_add_prefix(
                &instr16a.get_display(),
                &format!("{}A", ifam16a.prefix()),
            ));
        }

        if instr16b.get_name() != "NOP" {
            displays.push(display_add_prefix(
                &instr16b.get_display(),
                &format!("{}B", ifam16b.prefix()),
            ));
        }

        displays.join(" || ")
    }

    fn multi_pattern(
        ifam32: &InstrFamilyBuilder,
        instr32: &InstrBuilder,
        ifam16a: &InstrFamilyBuilder,
        instr16a: &InstrBuilder,
        ifam16b: &InstrFamilyBuilder,
        instr16b: &InstrBuilder,
    ) -> Pattern {
        let fields32 = instr32.pattern().fields_prefix(&ifam32.prefix());
        let fields16a = instr16a
            .pattern()
            .fields_prefix(&format!("{}A", ifam16a.prefix()));
        let fields16b = instr16b
            .pattern()
            .fields_prefix(&format!("{}B", ifam16b.prefix()));

        let mut h32 = fields32[0].clone();
        let l32 = fields32[1].clone();
        let a16 = fields16a[0].clone();
        let b16 = fields16b[0].clone();

        if let Some(field) = h32.get(0) {
            if field.len() == 5 {
                h32[0] = ProtoField::new("sig", FieldType::Mask(0x19), 5).to_field_end(15);
            } else {
                panic!("Cannot make a multi instr with this 32 bit instr")
            }
        } else {
            panic!("This 32 bit instr is missing a pattern")
        }

        Pattern::new([h32, l32, a16, b16])
    }

    fn multi_action(
        ifam32: &InstrFamilyBuilder,
        instr32: &InstrBuilder,
        ifam16a: &InstrFamilyBuilder,
        instr16a: &InstrBuilder,
        ifam16b: &InstrFamilyBuilder,
        instr16b: &InstrBuilder,
    ) -> Code {
        let mut regs: HashSet<(bool, String)> = HashSet::new();
        let mut action32 = instr32
            .get_actions()
            .multify(&ifam32.prefix(), false, &mut regs);
        let action16a =
            instr16a
                .get_actions()
                .multify(&format!("{}A", ifam16a.prefix()), false, &mut regs);
        let action16b =
            instr16b
                .get_actions()
                .multify(&format!("{}B", ifam16b.prefix()), false, &mut regs);

        action32.append(action16a);
        action32.append(action16b);
        action32
    }

    fn multi_pcode(
        ifam32: &InstrFamilyBuilder,
        instr32: &InstrBuilder,
        ifam16a: &InstrFamilyBuilder,
        instr16a: &InstrBuilder,
        ifam16b: &InstrFamilyBuilder,
        instr16b: &InstrBuilder,
    ) -> Code {
        let mut regs: HashSet<(bool, String)> = HashSet::new();
        let mut code = Code::new();
        let action32 = instr32
            .get_pcodes()
            .multify(&ifam32.prefix(), false, &mut regs);
        let action16a =
            instr16a
                .get_pcodes()
                .multify(&format!("{}A", ifam16a.prefix()), false, &mut regs);
        let action16b =
            instr16b
                .get_pcodes()
                .multify(&format!("{}B", ifam16b.prefix()), false, &mut regs);

        for (field, reg_id) in regs.iter().sorted() {
            if *field {
                code.add_expr(e_copy(
                    b_local(b_var(&format!("old_{reg_id}Reg")), 4),
                    e_rfield(reg_id),
                ));
            } else {
                code.add_expr(e_copy(
                    b_local(b_var(&format!("old_{reg_id}")), 4),
                    b_reg(reg_id),
                ));
            }
        }

        code.append(action32);
        code.append(action16a);
        code.append(action16b);
        code
    }

    fn multi_instr(
        ifam: &InstrFamilyBuilder,
        ifam32: &InstrFamilyBuilder,
        instr32: &InstrBuilder,
        ifam16a: &InstrFamilyBuilder,
        instr16a: &InstrBuilder,
        ifam16b: &InstrFamilyBuilder,
        instr16b: &InstrBuilder,
    ) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(&Self::multi_name(instr32, instr16a, instr16b))
            .display(Self::multi_display(
                ifam32, instr32, ifam16a, instr16a, ifam16b, instr16b,
            ))
            .set_pattern(Self::multi_pattern(
                ifam32, instr32, ifam16a, instr16a, ifam16b, instr16b,
            ))
            .set_actions(Self::multi_action(
                ifam32, instr32, ifam16a, instr16a, ifam16b, instr16b,
            ))
            .set_pcodes(Self::multi_pcode(
                ifam32, instr32, ifam16a, instr16a, ifam16b, instr16b,
            ))
    }
}

impl InstrFactory for MultiFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let ifams32 = vec![nop32::instr_fam()];

        let ifams16 = vec![
            // MAIN_16A
            nop16::instr_fam(),
            ptr2op::instr_fam(),
            compi2op::instr_fam(),
            // MAIN_16B
            ldstpmod::instr_fam(),
            ldst::instr_fam(),
            dspldst::instr_fam(),
            ldstii::instr_fam(),
            ldstiifp::instr_fam(),
        ];

        let mut instrs = Vec::new();

        for ifam32 in ifams32.iter() {
            for instr32 in ifam32.instrs() {
                for ifam16a in ifams16.iter() {
                    for instr16a in ifam16a.instrs() {
                        for ifam16b in ifams16.iter() {
                            for instr16b in ifam16b.instrs() {
                                instrs.push(Self::multi_instr(
                                    ifam, ifam32, instr32, ifam16a, instr16a, ifam16b, instr16b,
                                ));
                            }
                        }
                    }
                }
            }
        }

        instrs
    }
}
