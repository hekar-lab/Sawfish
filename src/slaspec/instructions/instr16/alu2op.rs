use itertools::Itertools;

use crate::slaspec::instructions::common::{BinOp, UnOp};
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "ALU2op",
        "ALU Binary Operations",
        "a2o",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x10), 6),
            ProtoField::new("opc", FieldType::Blank, 4),
            ProtoField::new("src", FieldType::Blank, 3),
            ProtoField::new("dst", FieldType::Blank, 3),
        ]),
    );

    ifam.add_pcodeop("divs");
    ifam.add_pcodeop("divq");

    ifam.add_instrs(&OpAssignFactory());
    ifam.add_instrs(&DivideFactory());
    ifam.add_instrs(&MvDregToDregFactory());
    ifam.add_instrs(&UnaryFactory());

    ifam
}

struct OpAssignParam {
    mask: u16,
    name: String,
    op: BinOp,
    display: String,
    div_field: bool,
}

impl OpAssignParam {
    fn new(mask: u16, name: &str, op: BinOp, display: &str, div_field: bool) -> OpAssignParam {
        OpAssignParam {
            mask,
            name: name.to_string(),
            op,
            display: display.to_string(),
            div_field,
        }
    }
}

struct OpAssignFactory();

impl OpAssignFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, param: &OpAssignParam) -> InstrBuilder {
        let instr = InstrBuilder::new(ifam)
            .name(&param.name)
            .display(param.display.clone())
            .set_field_type("opc", FieldType::Mask(param.mask))
            .set_field_type("src", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_copy(
                e_rfield("dst"),
                (param.op)(e_rfield("dst"), e_rfield("src")),
            ));

        if param.div_field {
            instr.divide_field(
                "dst",
                ProtoPattern::new(vec![
                    ProtoField::new("dst", FieldType::Variable(RegisterSet::DReg), 3),
                    ProtoField::new("dstCpy", FieldType::Variable(RegisterSet::DReg), 3),
                ]),
            )
        } else {
            instr.set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
        }
    }
}

impl InstrFactory for OpAssignFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        fn e_addshift1(lhs: Expr, rhs: Expr) -> Expr {
            e_lshft(b_grp(e_add(lhs, rhs)), b_num(1))
        }

        fn e_addshift2(lhs: Expr, rhs: Expr) -> Expr {
            e_lshft(b_grp(e_add(lhs, rhs)), b_num(2))
        }

        let params = vec![
            OpAssignParam::new(0x0, "AShift32", e_arshft, "{dst} >>>= {src}", false),
            OpAssignParam::new(0x1, "LShift", e_rshft, "{dst} >>= {src}", false),
            OpAssignParam::new(0x2, "LShift", e_lshft, "{dst} <<= {src}", false),
            OpAssignParam::new(0x3, "MultInt", e_mult, "{dst} *= {src}", false),
            OpAssignParam::new(
                0x4,
                "AddSubShift",
                e_addshift1,
                "{dst} += ({dstCpy} + {src}) << 1",
                true,
            ),
            OpAssignParam::new(
                0x5,
                "AddSubShift",
                e_addshift2,
                "{dst} += ({dstCpy} + {src}) << 2",
                true,
            ),
        ];

        params
            .iter()
            .map(|param| Self::base_instr(ifam, param))
            .collect()
    }
}

struct DivideFactory();

impl DivideFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, div_type: char, opc: u16) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("Divide")
            .set_field_type("opc", FieldType::Mask(opc))
            .set_field_type("src", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .display(format!(
                "DIV{} ({{dst}}, {{src}})",
                div_type.to_ascii_uppercase()
            ))
            .add_pcode(e_mac2p(
                &format!("div{}", div_type.to_ascii_lowercase()),
                e_rfield("dst"),
                e_rfield("src"),
            ))
    }
}

impl InstrFactory for DivideFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::base_instr(ifam, 'q', 0x8),
            Self::base_instr(ifam, 's', 0x9),
        ]
    }
}

struct MvDregToDregFactory();

impl MvDregToDregFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, zext: bool, byte_src: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .set_field_type(
                "opc",
                FieldType::Mask(0xa + zext as u16 + (byte_src as u16) * 2),
            )
            .set_field_type(
                "src",
                FieldType::Variable(if byte_src {
                    RegisterSet::DRegB
                } else {
                    RegisterSet::DRegL
                }),
            )
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .name(if byte_src {
                "MvDregBToDreg"
            } else {
                "MvDregLToDreg"
            })
            .display(format!(
                "{{dst}} = {{src}} ({})",
                if zext { "Z" } else { "X" }
            ))
            .add_pcode(e_copy(
                e_rfield("dst"),
                e_macp(if zext { "zext" } else { "sext" }, e_rfield("src")),
            ))
    }
}

impl InstrFactory for MvDregToDregFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let param = [false, true];
        param
            .iter()
            .cartesian_product(param.iter())
            .map(|(byte_src, zext)| Self::base_instr(ifam, *zext, *byte_src))
            .collect()
    }
}

struct UnaryFactory();

impl UnaryFactory {
    fn base_instr(
        ifam: &InstrFamilyBuilder,
        name: &str,
        op_chr: char,
        op: UnOp,
        opc: u16,
    ) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(name)
            .display(format!("{{dst}} = {}{{src}}", op_chr))
            .set_field_type("opc", FieldType::Mask(opc))
            .set_field_type("src", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_copy(e_rfield("dst"), op(e_rfield("src"))))
    }
}

impl InstrFactory for UnaryFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::base_instr(ifam, "Neg32", '-', e_neg, 0xe),
            Self::base_instr(ifam, "Not32", '~', e_bit_not, 0xf),
        ]
    }
}
