use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::{cs_pop, cs_push};
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

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

#[derive(Debug, Clone)]
enum Info {
    Reg { id: String, size: usize, mask: u16 },
    Var(RegisterSet),
}

impl Info {
    fn reg(id: &str, size: usize, mask: u16) -> Self {
        Info::Reg {
            id: id.to_string(),
            size,
            mask,
        }
    }
}

impl PushPopFactory {
    fn push_display(val: &str) -> String {
        format!("[--SP] = {val}")
    }

    fn pop_display(val: &str) -> String {
        format!("{val} = [SP++]")
    }

    fn push_pop_instr(ifam: &InstrFamilyBuilder, push: bool, grp: u16, info: Info) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name(if push { "Push" } else { "Pop" })
            .set_field_type("w", FieldType::Mask(if push { 0x1 } else { 0x0 }))
            .set_field_type("grp", FieldType::Mask(grp));

        let op = if push { cs_push } else { cs_pop };
        let display = if push {
            Self::push_display
        } else {
            Self::pop_display
        };

        match info {
            Info::Reg { id, size, mask } => {
                instr = instr
                    .set_field_type("reg", FieldType::Mask(mask))
                    .display(display(&id))
                    .add_pcode(op(Expr::var(&id), size));
            }
            Info::Var(rset) => match rset {
                RegisterSet::IReg | RegisterSet::MReg | RegisterSet::BReg | RegisterSet::LReg => {
                    instr = instr
                        .split_field(
                            "reg",
                            ProtoPattern::new(vec![
                                ProtoField::new(
                                    "regH",
                                    FieldType::Mask(match rset {
                                        RegisterSet::IReg | RegisterSet::BReg => 0x0,
                                        _ => 0x1,
                                    }),
                                    1,
                                ),
                                ProtoField::new("regL", FieldType::Variable(rset), 2),
                            ]),
                        )
                        .display(display("{regL}"))
                        .add_pcode(op(Expr::field("regL"), 4));
                }
                _ => {
                    instr = instr
                        .set_field_type("reg", FieldType::Variable(rset))
                        .display(display("{reg}"))
                        .add_pcode(op(Expr::field("reg"), 4));
                }
            },
        }

        instr
    }
}

impl InstrFactory for PushPopFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut instrs = Vec::new();
        let infos = [
            (0x0, Info::Var(RegisterSet::DReg)),
            (0x1, Info::Var(RegisterSet::PReg)),
            (0x2, Info::Var(RegisterSet::IReg)),
            (0x2, Info::Var(RegisterSet::MReg)),
            (0x3, Info::Var(RegisterSet::BReg)),
            (0x3, Info::Var(RegisterSet::LReg)),
            (0x4, Info::reg("A0.X", 1, 0x0)),
            (0x4, Info::reg("A0.W", 4, 0x1)),
            (0x4, Info::reg("A1.X", 1, 0x2)),
            (0x4, Info::reg("A1.W", 4, 0x3)),
            (0x4, Info::reg("ASTAT", 4, 0x6)),
            (0x4, Info::reg("RETS", 4, 0x7)),
            (0x6, Info::Var(RegisterSet::SyRg2)),
            (0x7, Info::Var(RegisterSet::SyRg3)),
        ];

        for (grp, info) in infos {
            instrs.push(Self::push_pop_instr(ifam, false, grp, info.clone()));
            instrs.push(Self::push_pop_instr(ifam, true, grp, info));
        }

        instrs
    }
}
