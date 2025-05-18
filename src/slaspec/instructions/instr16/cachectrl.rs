use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::{cs_assign_by, e_add};
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "CacheCtrl",
        "Cache Control",
        "cct",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x009), 10),
            ProtoField::new("a", FieldType::Blank, 1),
            ProtoField::new("opc", FieldType::Blank, 2),
            ProtoField::new("reg", FieldType::Blank, 3),
        ]),
    );

    ifam.add_pcodeop("prefetch");
    ifam.add_pcodeop("flushinv");
    ifam.add_pcodeop("flush");
    ifam.add_pcodeop("iflush");

    ifam.add_instrs(&CacheCtrlFactory());

    ifam
}

struct CacheCtrlFactory();

impl CacheCtrlFactory {
    fn base_instr(
        ifam: &InstrFamilyBuilder,
        post_inc: bool,
        opc: u16,
        pcodeop: &str,
    ) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .set_field_type("a", FieldType::Mask(if post_inc { 0x1 } else { 0x0 }))
            .set_field_type("opc", FieldType::Mask(opc))
            .set_field_type("reg", FieldType::Variable(RegisterSet::PReg))
            .name("CacheCtrl")
            .display(format!(
                "{} [{{reg}}{}]",
                pcodeop.to_uppercase(),
                if post_inc { "++" } else { "" }
            ))
            .add_pcode(Expr::line(
                Expr::macp(pcodeop, Expr::field("reg")),
                if post_inc {
                    Some(Expr::line(
                        cs_assign_by(e_add, Expr::field("reg"), Expr::num(0x20)),
                        None,
                    ))
                } else {
                    None
                },
            ))
    }
}

impl InstrFactory for CacheCtrlFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut instrs = Vec::new();
        let instr_vals = [
            (0x0, "prefetch"),
            (0x1, "flushinv"),
            (0x2, "flush"),
            (0x3, "iflush"),
        ];

        for (opc, pcodeop) in instr_vals {
            instrs.push(Self::base_instr(ifam, false, opc, pcodeop));
            instrs.push(Self::base_instr(ifam, true, opc, pcodeop));
        }

        instrs
    }
}
