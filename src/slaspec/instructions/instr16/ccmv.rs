use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::{e_copy, e_not};
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "CCMV",
        "Conditional Move",
        "cmv",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x03), 7),
            ProtoField::new("t", FieldType::Blank, 1),
            ProtoField::new("d", FieldType::Blank, 1),
            ProtoField::new("s", FieldType::Blank, 1),
            ProtoField::new("dst", FieldType::Blank, 3),
            ProtoField::new("src", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&CCMvFactory());

    ifam
}

struct CCMvFactory();

impl CCMvFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, cc: bool, psrc: bool, pdst: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("MvRegToRegCond")
            .set_field_type("t", FieldType::Mask(cc as u16))
            .set_field_type("d", FieldType::Mask(psrc as u16))
            .set_field_type("s", FieldType::Mask(pdst as u16))
            .set_field_type(
                "dst",
                FieldType::Variable(if pdst {
                    RegisterSet::PReg
                } else {
                    RegisterSet::DReg
                }),
            )
            .set_field_type(
                "src",
                FieldType::Variable(if psrc {
                    RegisterSet::PReg
                } else {
                    RegisterSet::DReg
                }),
            )
            .display(format!(
                "if {}CC {{dst}} = {{src}}",
                if cc { "" } else { "!" },
            ))
            .add_pcode(Expr::ifgoto(
                if cc {
                    e_not(Expr::reg("CC"))
                } else {
                    Expr::reg("CC")
                },
                Expr::label("do_nothing"),
            ))
            .add_pcode(e_copy(Expr::field("dst"), Expr::field("src")))
            .add_pcode(Expr::label("do_nothing"))
    }
}

impl InstrFactory for CCMvFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let param = [false, true];
        param
            .iter()
            .cartesian_product(param.iter())
            .cartesian_product(param.iter())
            .map(|((cc, psrc), pdst)| Self::base_instr(ifam, *cc, *psrc, *pdst))
            .collect()
    }
}
