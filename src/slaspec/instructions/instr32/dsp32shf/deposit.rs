use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

pub struct DepositFactory();

impl DepositFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sext: bool) -> InstrBuilder {
        let ext_len = "extract_length";
        let ext_off = "extract_offset";
        let ext_fg = "extract_foreground";
        let ext_res = "extract_result";
        let ext_mask = "extract_mask";
        let sgn_shft = "extract_sign_shift";

        InstrBuilder::new(ifam)
            .name("Deposit")
            .display(format!(
                "{{dst}} = DEPOSIT ({{src0}}, {{src1}}){}",
                if sext { " (X)" } else { "" }
            ))
            .set_field_type("sopc", FieldType::Mask(0xb))
            .set_field_type("sop", FieldType::Mask(sext as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(cs_mline(vec![
                e_copy(
                    e_local(ext_len, 4),
                    e_bit_and(e_rfield("src1"), b_num(0x1f)),
                ),
                b_ifgoto(e_le(b_var(ext_len), b_num(16)), b_label("length_clamped")),
                e_copy(b_var(ext_len), b_num(16)),
                b_label("length_clamped"),
                e_copy(
                    e_local(ext_off, 4),
                    e_bit_and(b_grp(e_rshft(e_rfield("src1"), b_num(8))), b_num(0x1f)),
                ),
                e_copy(e_local(ext_fg, 4), e_rshft(e_rfield("src1"), b_num(16))),
                e_copy(e_local(ext_res, 4), e_rfield("src0")),
                e_copy(
                    e_local(sgn_shft, 4),
                    e_sub(b_num(32), b_grp(e_add(b_var(ext_len), b_var(ext_off)))),
                ),
                b_ifgoto(e_ge(b_var(sgn_shft), b_num(0)), b_label("pos_sgnshft")),
                e_copy(b_var(sgn_shft), b_num(0)),
                b_label("pos_sgnshft"),
                e_copy(
                    e_local(ext_mask, 4),
                    e_bit_not(b_grp(e_lshft(
                        b_grp(e_sub(b_grp(e_lshft(b_num(1), b_var(ext_len))), b_num(1))),
                        b_var(ext_off),
                    ))),
                ),
                e_copy(
                    b_var(ext_res),
                    e_bit_or(
                        b_grp(e_bit_and(b_var(ext_res), b_var(ext_mask))),
                        b_grp(e_bit_and(
                            b_grp(e_lshft(b_var(ext_fg), b_var(ext_off))),
                            e_bit_not(b_var(ext_mask)),
                        )),
                    ),
                ),
            ]))
            .add_pcode_opt(if sext {
                Some(e_copy(
                    b_var(ext_res),
                    e_arshft(
                        b_grp(e_lshft(b_var(ext_res), b_var(sgn_shft))),
                        b_var(sgn_shft),
                    ),
                ))
            } else {
                None
            })
            .add_pcode(e_copy(e_rfield("dst"), b_var(ext_res)))
    }
}

impl InstrFactory for DepositFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [false, true]
            .into_iter()
            .map(|sext| Self::base_instr(ifam, sext))
            .collect()
    }
}
