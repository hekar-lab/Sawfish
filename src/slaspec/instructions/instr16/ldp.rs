use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "Ldp",
        "Load/Store",
        "ldp",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x48), 7),
            ProtoField::new("aop", FieldType::Blank, 2),
            ProtoField::new("x1", FieldType::Mask(0x1), 1),
            ProtoField::new("ptr", FieldType::Variable(RegisterSet::PReg), 3),
            ProtoField::new("reg", FieldType::Variable(RegisterSet::PReg), 3),
        ]),
    );

    ifam.add_instrs(&LdpFactory());

    ifam
}

#[derive(Debug, Clone, Copy)]
enum AddrOp {
    Inc = 0,
    Dec = 1,
    None = 2,
}

impl AddrOp {
    fn display(&self) -> String {
        format!(
            "[{{ptr}}{}]",
            match self {
                Self::Inc => "++",
                Self::Dec => "--",
                Self::None => "",
            }
        )
    }
}

struct LdpFactory();

impl LdpFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, aop: AddrOp) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name("ldpPtrOp")
            .display(format!("{{reg}} = {}", aop.display()))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .add_pcode(e_copy(b_field("reg"), b_ptr(b_field("ptr"), 4)));

        match aop {
            AddrOp::Inc => instr = instr.add_pcode(cs_assign_by(e_add, b_field("ptr"), b_num(4))),
            AddrOp::Dec => instr = instr.add_pcode(cs_assign_by(e_sub, b_field("ptr"), b_num(4))),
            _ => {}
        }

        instr
    }
}

impl InstrFactory for LdpFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [AddrOp::Inc, AddrOp::Dec, AddrOp::None]
            .into_iter()
            .map(|aop| Self::base_instr(ifam, aop))
            .collect()
    }
}
