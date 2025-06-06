use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy)]
struct BXORParams {
    feedback: bool,
    shift: bool,
}

impl BXORParams {
    fn new(feedback: bool, shift: bool) -> Self {
        Self { feedback, shift }
    }

    fn all() -> [Self; 4] {
        [
            Self::new(false, true),
            Self::new(false, false),
            Self::new(true, true),
            Self::new(true, false),
        ]
    }

    fn name(&self) -> String {
        format!(
            "BXOR{}{}",
            if self.shift { "Shift" } else { "" },
            if !self.feedback { "_NF" } else { "" }
        )
    }

    fn display(&self) -> String {
        if self.feedback && self.shift {
            "A0 = BXORSHIFT (A0, A1, CC)".to_string()
        } else {
            format!(
                "{{dst}} = CC = BXOR{} (A0, {})",
                if self.shift { "SHIFT" } else { "" },
                if self.feedback { "A1, CC" } else { "{src0}" }
            )
        }
    }

    fn sopc(&self) -> u16 {
        0xb + self.feedback as u16
    }

    fn sop(&self) -> u16 {
        (!self.shift) as u16
    }

    fn has_dst(&self) -> bool {
        return !(self.feedback & self.shift);
    }

    fn expr(&self) -> Expr {
        fn xor_reduc(dst: &str, src: &str) -> Expr {
            e_copy(
                b_var(dst),
                e_bit_and(e_macp("popcount", b_var(src)), b_num(1)),
            )
        }

        let reduc = "bxor_reduc";
        let data32 = "bxor_data32";
        let data40 = "bxor_data40";
        let mut code = vec![e_local(reduc, 2)];

        if self.feedback {
            code.push(e_local(data40, 5));
            code.push(e_copy(b_var(data40), e_bit_and(b_reg("A0"), b_reg("A1"))));
            code.push(xor_reduc(reduc, data40));
            code.push(cs_assign_by(e_bit_xor, b_var(reduc), e_zext(b_reg("CC"))));
            if self.shift {
                code.push(e_copy(
                    b_reg("A0"),
                    e_bit_or(b_grp(e_lshft(b_reg("A0"), b_num(1))), e_zext(b_var(reduc))),
                ));
            } else {
                code.push(e_copy(b_reg("CC"), b_size(b_var(reduc), 1)));
                code.push(e_copy(
                    e_rfield("dst"),
                    e_bit_or(
                        b_grp(e_bit_and(e_rfield("dst"), b_num(0xfffe))),
                        b_var(reduc),
                    ),
                ));
            }
        } else {
            code.push(e_local(data32, 4));
            if self.shift {
                code.push(cs_assign_by(e_lshft, b_reg("A0"), b_num(1)));
            }
            code.push(e_copy(
                b_var(data32),
                e_bit_and(b_reg("A0.W"), e_rfield("src0")),
            ));
            code.push(xor_reduc(reduc, data32));
            code.push(e_copy(b_reg("CC"), b_size(b_var(reduc), 1)));
            code.push(e_copy(
                e_rfield("dst"),
                e_bit_or(
                    b_grp(e_bit_and(e_rfield("dst"), b_num(0xfffe))),
                    b_var(reduc),
                ),
            ));
        }

        cs_mline(code)
    }
}

pub struct BXORFactory();

impl BXORFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, params: BXORParams) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name(&params.name())
            .display(params.display())
            .set_field_type("sopc", FieldType::Mask(params.sopc()))
            .set_field_type("sop", FieldType::Mask(params.sop()))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type_opt(
                params.has_dst(),
                "dst",
                FieldType::Variable(RegisterSet::DRegL),
            )
            .set_field_type_opt(
                !params.feedback,
                "src0",
                FieldType::Variable(RegisterSet::DReg),
            )
            .add_pcode(params.expr())
    }
}

impl InstrFactory for BXORFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        BXORParams::all()
            .into_iter()
            .map(|params| Self::base_instr(ifam, params))
            .collect()
    }
}
