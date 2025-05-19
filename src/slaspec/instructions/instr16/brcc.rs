use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::{cs_ifgoto, e_add, e_copy, e_mult, e_not};
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "BrCC",
        "Conditional Branch PC relative on CC",
        "brc",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x1), 4),
            ProtoField::new("t", FieldType::Blank, 1),
            ProtoField::new("b", FieldType::Blank, 1),
            ProtoField::new("off", FieldType::SImmVal, 10),
        ]),
    );

    ifam.add_instrs(&BranchCCFactory());

    ifam
}

struct BranchCCFactory();

impl BranchCCFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, cc: bool, branch_pred: bool) -> InstrBuilder {
        let addr_var = "addr";
        let addr_ptr_var = "addrPtr";
        InstrBuilder::new(ifam)
            .name("BrCC")
            .display(format!(
                "if {}CC JUMP {{${addr_var}}}{}",
                if cc { "" } else { "!" },
                if branch_pred { "(BP)" } else { "" }
            ))
            .set_field_type("t", FieldType::Mask(if cc { 0x1 } else { 0x0 }))
            .set_field_type("b", FieldType::Mask(if branch_pred { 0x1 } else { 0x0 }))
            .add_action(e_copy(
                Expr::var(addr_var),
                e_add(
                    Expr::var("inst_start"),
                    e_mult(Expr::field("off"), Expr::num(2)),
                ),
            ))
            .add_pcode(e_copy(
                Expr::local(addr_ptr_var, 4),
                Expr::ptr(Expr::size(Expr::var(addr_var), 4), 4),
            ))
            .add_pcode(cs_ifgoto(
                if cc {
                    Expr::reg("CC")
                } else {
                    e_not(Expr::reg("CC"))
                },
                Expr::var(addr_ptr_var),
            ))
    }
}

impl InstrFactory for BranchCCFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let param = [false, true];
        param
            .iter()
            .cartesian_product(param.iter())
            .map(|(cc, branch_pred)| Self::base_instr(ifam, *cc, *branch_pred))
            .collect()
    }
}
