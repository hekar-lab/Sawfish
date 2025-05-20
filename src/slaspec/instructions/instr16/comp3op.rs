use crate::slaspec::instructions::common::BinOp;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "Comp3op",
        "Compute with 3 operands",
        "c3o",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x5), 4),
            ProtoField::new("opc", FieldType::Blank, 3),
            ProtoField::new("dst", FieldType::Blank, 3),
            ProtoField::new("src1", FieldType::Blank, 3),
            ProtoField::new("src0", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&TernaryOpFactory());

    ifam
}

struct TernaryOpFactory();

impl TernaryOpFactory {
    fn base_instr(
        ifam: &InstrFamilyBuilder,
        name: &str,
        opc: u16,
        reg: &RegisterSet,
        display: &str,
        op: BinOp,
    ) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(name)
            .display(display.to_string())
            .set_field_type("opc", FieldType::Mask(opc))
            .set_field_type("dst", FieldType::Variable(reg.clone()))
            .set_field_type("src1", FieldType::Variable(reg.clone()))
            .set_field_type("src0", FieldType::Variable(reg.clone()))
            .add_pcode(e_copy(b_field("dst"), op(b_field("src0"), b_field("src1"))))
    }
}

impl InstrFactory for TernaryOpFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let dreg = RegisterSet::DReg;
        let preg = RegisterSet::PReg;

        fn e_addshifted1(lhs: Expr, rhs: Expr) -> Expr {
            e_add(lhs, b_grp(e_lshft(rhs, b_num(1))))
        }

        fn e_addshifted2(lhs: Expr, rhs: Expr) -> Expr {
            e_add(lhs, b_grp(e_lshft(rhs, b_num(2))))
        }

        let params: [(&str, &RegisterSet, &str, BinOp); 8] = [
            ("AddSub32", &dreg, "{dst} = {src0} + {src1}", e_add),
            ("AddSub32", &dreg, "{dst} = {src0} - {src1}", e_sub),
            ("Logic32", &dreg, "{dst} = {src0} & {src1}", e_bit_and),
            ("Logic32", &dreg, "{dst} = {src0} | {src1}", e_bit_or),
            ("Logic32", &dreg, "{dst} = {src0} ^ {src1}", e_bit_xor),
            ("DagAdd32", &preg, "{dst} = {src0} + {src1}", e_add),
            (
                "PtrOp",
                &preg,
                "{dst} = {src0} + ({src1} << 1)",
                e_addshifted1,
            ),
            (
                "PtrOp",
                &preg,
                "{dst} = {src0} + ({src1} << 2)",
                e_addshifted2,
            ),
        ];

        params
            .into_iter()
            .enumerate()
            .map(|(opc, (name, reg, display, op))| {
                Self::base_instr(ifam, name, opc as u16, reg, display, op)
            })
            .collect()
    }
}
