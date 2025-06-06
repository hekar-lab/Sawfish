use crate::slaspec::instructions::common::RegParam;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_64(
        "LdImm",
        "Load Immediate Word",
        "liw",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0x368), 10),
                ProtoField::new("grp", FieldType::Blank, 3),
                ProtoField::new("reg", FieldType::Blank, 3),
            ]),
            ProtoPattern::new(vec![ProtoField::new("immH", FieldType::SImmVal, 16)]),
            ProtoPattern::new(vec![ProtoField::new("immL", FieldType::UImmVal, 16)]),
            ProtoPattern::new(vec![ProtoField::new("dummy", FieldType::Any, 16)]),
        ],
    );

    ifam.add_instrs(&LdImmFactory());

    ifam
}

struct LdImmFactory();

impl LdImmFactory {
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
            } => e_rfield(field_id),
        }
    }

    fn base_instr(ifam: &InstrFamilyBuilder, params: RegParam) -> InstrBuilder {
        params
            .set_field(InstrBuilder::new(ifam), "reg")
            .set_field_type("grp", FieldType::Mask(params.grp()))
            .name(if params.acc() {
                "LdImmToAcc"
            } else {
                "LdImmToReg"
            })
            .display(format!(
                "{} = {{$imm32}}",
                Self::display_reg(&params, &params.get_field_id("reg"))
            ))
            .add_action(e_copy(
                b_var("imm32"),
                e_bit_or(
                    b_grp(e_lshft(e_rfield("immH"), b_num(16))),
                    e_rfield("immL"),
                ),
            ))
            .add_pcode(if params.size() == 4 {
                e_copy(
                    Self::expr_reg(&params, &params.get_field_id("reg")),
                    b_var("imm32"),
                )
            } else {
                e_copy(
                    Self::expr_reg(&params, &params.get_field_id("reg")),
                    b_size(b_var("imm32"), 1),
                )
            })
    }
}

impl InstrFactory for LdImmFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        RegParam::all_regs()
            .into_iter()
            .map(|params| Self::base_instr(ifam, params))
            .collect()
    }
}
