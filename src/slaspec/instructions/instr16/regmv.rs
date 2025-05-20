use itertools::Itertools;

use crate::slaspec::instructions::common::RegParam;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_16(
        "RegMv",
        "Register to register transfer operation",
        "rmv",
        ProtoPattern::new(vec![
            ProtoField::new("sig", FieldType::Mask(0x3), 4),
            ProtoField::new("gd", FieldType::Blank, 3),
            ProtoField::new("gs", FieldType::Blank, 3),
            ProtoField::new("dst", FieldType::Blank, 3),
            ProtoField::new("src", FieldType::Blank, 3),
        ]),
    );

    ifam.add_instrs(&MvRegToRegFactory());

    ifam
}

struct MvRegToRegFactory();

impl MvRegToRegFactory {
    fn display_reg(reg: &RegParam, field_id: &str) -> String {
        match reg {
            RegParam::Fixed {
                group: _,
                id,
                size: _,
                mask: _,
            } => id.clone(),
            RegParam::Var {
                group: _,
                regset: _,
            } => format!("{{{field_id}}}"),
        }
    }

    fn expr_reg(reg: &RegParam, field_id: &str) -> Expr {
        match reg {
            RegParam::Fixed {
                group: _,
                id,
                size: _,
                mask: _,
            } => b_reg(id),
            RegParam::Var {
                group: _,
                regset: _,
            } => b_field(field_id),
        }
    }

    fn adjust_size(dst: &RegParam, src: &RegParam) -> Expr {
        let var = Self::expr_reg(src, "src");

        if dst.size() > src.size() {
            e_macp("sext", var)
        } else if dst.size() < src.size() {
            b_size(var, dst.size())
        } else {
            var
        }
    }

    fn base_instr(ifam: &InstrFamilyBuilder, dst: &RegParam, src: &RegParam) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("MvRegToReg")
            .set_field_type("gd", FieldType::Mask(dst.grp()))
            .set_field_type("gs", FieldType::Mask(src.grp()))
            .set_field_type("dst", dst.ftype())
            .set_field_type("src", src.ftype())
            .display(format!(
                "{} = {}",
                Self::display_reg(dst, "dst"),
                Self::display_reg(src, "src")
            ))
            .add_pcode(e_copy(
                Self::expr_reg(dst, "dst"),
                Self::adjust_size(dst, src),
            ))
    }
}

impl InstrFactory for MvRegToRegFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let all_regs = RegParam::all_regs();

        all_regs
            .iter()
            .cartesian_product(all_regs.iter())
            .map(|(dst, src)| Self::base_instr(ifam, dst, src))
            .collect()
    }
}
