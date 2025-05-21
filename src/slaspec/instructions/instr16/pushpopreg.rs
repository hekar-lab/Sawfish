use crate::slaspec::instructions::common::RegParam;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "PushPopReg",
        "Push or Pop register, to and from the stack pointed to by SP",
        "ppr",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x002), 9),
            ProtoField::new("w", FieldType::Blank, 1),
            ProtoField::new("grp", FieldType::Blank, 3),
            ProtoField::new("reg", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&PushPopFactory());

    ifam
}

struct PushPopFactory();

impl PushPopFactory {
    fn push_display(val: &str) -> String {
        format!("[--SP] = {val}")
    }

    fn pop_display(val: &str) -> String {
        format!("{val} = [SP++]")
    }

    fn push_pop_instr(ifam: &InstrFamilyBuilder, push: bool, param: RegParam) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name(if push { "Push" } else { "Pop" })
            .set_field_type("w", FieldType::Mask(push as u16))
            .set_field_type("grp", FieldType::Mask(param.grp()));

        let op = if push { cs_push } else { cs_pop };
        let display = if push {
            Self::push_display
        } else {
            Self::pop_display
        };

        instr = param.set_field(instr, "reg");

        instr = match param {
            RegParam::Fixed {
                group: _,
                id,
                size,
                mask: _,
            } => instr.display(display(&id)).add_pcode(op(b_var(&id), size)),
            RegParam::Var {
                group: _,
                regset: _,
            } => instr
                .display(display(&format!("{{{}}}", param.get_field_id("reg"))))
                .add_pcode(op(b_field(&param.get_field_id("reg")), 4)),
        };

        instr
    }
}

impl InstrFactory for PushPopFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut instrs = Vec::new();
        let params = RegParam::all_regs();

        for param in params {
            instrs.push(Self::push_pop_instr(ifam, false, param.clone()));
            instrs.push(Self::push_pop_instr(ifam, true, param));
        }

        instrs
    }
}
