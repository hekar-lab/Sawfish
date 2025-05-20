use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
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
            .set_field_type("t", FieldType::Mask(cc as u16))
            .set_field_type("b", FieldType::Mask(branch_pred as u16))
            .add_action(e_copy(
                b_var(addr_var),
                e_add(b_var("inst_start"), e_mult(b_field("off"), b_num(2))),
            ))
            .add_pcode(e_copy(
                e_local(addr_ptr_var, 4),
                b_ptr(b_size(b_var(addr_var), 4), 4),
            ))
            .add_pcode(cs_ifgoto(
                if cc { b_reg("CC") } else { e_not(b_reg("CC")) },
                b_var(addr_ptr_var),
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
