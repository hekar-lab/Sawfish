use crate::slaspec::instructions::{
    core::{InstrBuilder, InstrFactory, InstrFamilyBuilder},
    pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet},
    pcode::{p_macro, p_op, p_return},
    text::Text,
    util::quote,
};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "ProgCtrl",
        "Basic Program Sequencer Control Functions",
        "pgc",
        ProtoPattern {
            fields: vec![
                ProtoField::new("sig", FieldType::Mask(0x00), 8),
                ProtoField::new("opc", FieldType::Blank, 4),
                ProtoField::new("reg", FieldType::Blank, 4),
            ],
        },
    );

    ifam.add_pcodeop("idle");
    ifam.add_pcodeop("csync");
    ifam.add_pcodeop("ssync");
    ifam.add_pcodeop("emuexcpt");

    ifam.add_instrs(&ReturnFactory());
    ifam.add_instrs(&SyncModeFactory());

    ifam
}

struct ReturnFactory();

impl ReturnFactory {
    fn instr_rt(ifam: &InstrFamilyBuilder, retreg: char, regmask: u16) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .set_field_type("opc", FieldType::Mask(0x01))
            .set_field_type("reg", FieldType::Mask(regmask))
            .name("Return")
            .display(Text::from(format!("RT{retreg}")))
            .pcode(p_op(p_return(Text::from(format!("RET{retreg}")))))
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
            .display(Text::from(pcodeop.to_uppercase()))
            .pcode(p_op(p_macro(pcodeop)))
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
        ProtoPattern {
            fields: vec![
                ProtoField::new("regH", FieldType::Mask(0x0), 1),
                ProtoField::new("regL", FieldType::Blank, 3),
            ],
        },
    )
}

struct IMaskFactory();

impl IMaskFactory {
    fn base_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
        reg_instr(InstrBuilder::new(ifam))
            .name("IMaskMv")
            .set_field_type("regL", FieldType::Variable(RegisterSet::DReg))
    }
}

impl InstrFactory for IMaskFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        todo!()
    }
}
