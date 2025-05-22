use crate::slaspec::instructions::common::BinOp;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "Ptr2op",
        "Pointer Arithmetic Operations",
        "p2o",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x22), 7),
            ProtoField::new("opc", FieldType::Blank, 3),
            ProtoField::new("src", FieldType::Variable(RegisterSet::PReg), 3),
            ProtoField::new("dst", FieldType::Variable(RegisterSet::PReg), 3),
        ]),
    );

    ifam.add_instrs(&OpAssignFactory());

    ifam
}

struct OpAssignParam {
    mask: u16,
    name: String,
    op: BinOp,
    display: String,
}

impl OpAssignParam {
    fn new(mask: u16, name: &str, op: BinOp, display: &str) -> OpAssignParam {
        OpAssignParam {
            mask,
            name: name.to_string(),
            op,
            display: display.to_string(),
        }
    }
}

struct OpAssignFactory();

impl OpAssignFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, param: &OpAssignParam) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(&param.name)
            .display(param.display.clone())
            .set_field_type("opc", FieldType::Mask(param.mask))
            .add_pcode(e_copy(
                e_rfield("dst"),
                (param.op)(e_rfield("dst"), e_rfield("src")),
            ))
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

        fn e_lshft1(_lhs: Expr, rhs: Expr) -> Expr {
            e_lshft(rhs, b_num(1))
        }

        fn e_lshft2(_lhs: Expr, rhs: Expr) -> Expr {
            e_lshft(rhs, b_num(2))
        }

        fn e_rshft1(_lhs: Expr, rhs: Expr) -> Expr {
            e_rshft(rhs, b_num(1))
        }

        fn e_rshft2(_lhs: Expr, rhs: Expr) -> Expr {
            e_rshft(rhs, b_num(2))
        }

        let params = vec![
            OpAssignParam::new(0x0, "DagAdd32", e_sub, "{dst} -= {src}"),
            OpAssignParam::new(0x1, "LShiftPtr", e_lshft2, "{dst} = {src} << 2"),
            OpAssignParam::new(0x2, "LShiftPtr", e_lshft1, "{dst} = {src} << 1"),
            OpAssignParam::new(0x3, "LShiftPtr", e_rshft2, "{dst} = {src} >> 2"),
            OpAssignParam::new(0x4, "LShiftPtr", e_rshft1, "{dst} = {src} >> 1"),
            OpAssignParam::new(0x5, "DagAdd32", e_add, "{dst} += {src} (BREV)"),
            OpAssignParam::new(0x6, "DagAddSubShift", e_addshift1, "{dst} +<<1= {src}"),
            OpAssignParam::new(0x7, "DagAddSubShift", e_addshift2, "{dst} +<<2= {src}"),
        ];

        params
            .iter()
            .map(|param| Self::base_instr(ifam, param))
            .collect()
    }
}
