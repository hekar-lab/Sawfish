use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

pub struct ExtractFactory();

impl ExtractFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, sext: bool) -> InstrBuilder {
        let ext_len = "extract_length";
        let ext_off = "extract_offset";
        let lshift = "extract_lshift";
        let rshift_op = if sext { e_arshft } else { e_rshft };

        InstrBuilder::new(ifam)
            .name("Extract")
            .display(format!(
                "{{dst}} = EXTRACT ({{src0}}, {{src1}}) ({})",
                if sext { "X" } else { "Z" }
            ))
            .set_field_type("sopc", FieldType::Mask(0xa))
            .set_field_type("sop", FieldType::Mask(sext as u16))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DRegL))
            .add_pcode(cs_mline(vec![
                e_copy(
                    e_local(ext_len, 1),
                    e_bit_and(b_size(e_rfield("src1"), 1), b_num(0x1f)),
                ),
                e_copy(
                    e_local(ext_off, 1),
                    e_bit_and(b_trunc(e_rfield("src1"), 1), b_num(0x1f)),
                ),
                e_copy(e_local(lshift, 1), e_sub(b_num(32), b_var(ext_len))),
            ]))
            .add_pcode(e_copy(
                e_rfield("dst"),
                rshift_op(
                    b_grp(e_lshft(
                        b_grp(e_rshft(e_rfield("src0"), b_var(ext_off))),
                        b_var(lshift),
                    )),
                    b_var(lshift),
                ),
            ))
    }
}

impl InstrFactory for ExtractFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [false, true]
            .into_iter()
            .map(|sext| Self::base_instr(ifam, sext))
            .collect()
    }
}
