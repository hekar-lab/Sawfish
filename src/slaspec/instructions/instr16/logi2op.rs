use crate::slaspec::instructions::common::BinOp;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "Logi2Op",
        "Logic Binary Operations",
        "l2o",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x09), 5),
            ProtoField::new("opc", FieldType::Blank, 3),
            ProtoField::new("src", FieldType::UImmVal, 5),
            ProtoField::new("dst", FieldType::Variable(RegisterSet::DReg), 3),
        ]),
    );

    ifam.add_instrs(&BitTstFactory());
    ifam.add_instrs(&BitModFactory());
    ifam.add_instrs(&ShiftFactory());

    ifam
}

struct BitTstFactory();

impl BitTstFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, cc: bool) -> InstrBuilder {
        let comp = if cc { e_ne } else { e_eq };
        InstrBuilder::new(ifam)
            .name("ShiftBitTst")
            .set_field_type("opc", FieldType::Mask(cc as u16))
            .display(format!(
                "CC = {}BITTST({{dst}}, {{src}})",
                if cc { "" } else { "!" }
            ))
            .add_pcode(e_copy(
                b_reg("CC"),
                comp(
                    b_num(0),
                    b_grp(e_bit_and(
                        b_field("dst"),
                        b_grp(e_lshft(b_num(1), b_field("src"))),
                    )),
                ),
            ))
    }
}

impl InstrFactory for BitTstFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![Self::base_instr(ifam, false), Self::base_instr(ifam, true)]
    }
}

struct BitModFactory();

impl BitModFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, display: &str, opc: u16, expr: Expr) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("Shift_BitMod")
            .display(format!("{}({{dst}}, {{src}})", display))
            .set_field_type("opc", FieldType::Mask(opc))
            .add_pcode(e_copy(b_field("dst"), expr))
    }
}

impl InstrFactory for BitModFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        fn e_bitshift() -> Expr {
            b_grp(e_lshft(b_num(1), b_field("src")))
        }

        let params = [
            ("BITSET", 0x2, e_bit_or(b_field("dst"), e_bitshift())),
            ("BITTGL", 0x3, e_bit_xor(b_field("dst"), e_bitshift())),
            (
                "BITCLR",
                0x4,
                e_bit_and(b_field("dst"), e_bit_not(e_bitshift())),
            ),
        ];

        params
            .into_iter()
            .map(|(display, opc, op)| Self::base_instr(ifam, display, opc, op))
            .collect()
    }
}

struct ShiftFactory();

impl ShiftFactory {
    fn base_instr(
        ifam: &InstrFamilyBuilder,
        name: &str,
        op_str: &str,
        opc: u16,
        op: BinOp,
    ) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(name)
            .display(format!("{{dst}} {} {{src}}", op_str))
            .set_field_type("opc", FieldType::Mask(opc))
            .add_pcode(e_copy(b_field("dst"), op(b_field("dst"), b_field("src"))))
    }
}

impl InstrFactory for ShiftFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let params: [(&str, &str, u16, BinOp); 3] = [
            ("AShift32", ">>>=", 0x5, e_arshft),
            ("LShift", ">>=", 0x6, e_rshft),
            ("LShift", "<<=", 0x7, e_lshft),
        ];

        params
            .into_iter()
            .map(|(name, op_str, opc, op)| Self::base_instr(ifam, name, op_str, opc, op))
            .collect()
    }
}
