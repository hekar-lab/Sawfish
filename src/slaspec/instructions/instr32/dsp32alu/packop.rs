use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

type POF = PackOpFactory;
pub struct PackOpFactory();

impl PackOpFactory {
    const TMP4: &'static str = "tmp4_vidop";
    const TMP8: &'static str = "tmp8_vidop";

    fn init_vars() -> Expr {
        cs_mline(vec![e_local(Self::TMP4, 4), e_local(Self::TMP8, 8)])
    }

    fn get_byte(b_idx: usize, dstid: &str, srcid: usize) -> Expr {
        e_copy(
            b_var(dstid),
            e_bit_and(
                b_num(0xff),
                e_rshft(e_rfield(&format!("src{srcid}")), b_num(8 * b_idx as i128)),
            ),
        )
    }

    fn get_byte_i0(b_idx: usize, dstid: &str) -> Expr {
        e_copy(
            b_var(dstid),
            e_bit_and(
                b_num(0xff),
                e_rshft(
                    e_rfield(&format!("src0")),
                    b_grp(e_mult(
                        b_num(8),
                        b_grp(e_rem(
                            b_grp(e_add(
                                b_grp(e_bit_and(b_reg(&format!("I0")), b_num(0x3))),
                                b_num(b_idx as i128),
                            )),
                            b_num(8),
                        )),
                    )),
                ),
            ),
        )
    }

    fn pack_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
        fn pack_bytes(b_idx: usize, srcid: usize) -> Expr {
            cs_mline(vec![
                POF::get_byte(b_idx, POF::TMP4, srcid),
                cs_assign_by(e_lshft, e_rfield("dst0"), b_num(1)),
                cs_assign_by(e_bit_or, e_rfield("dst0"), b_var(POF::TMP4)),
            ])
        }

        InstrBuilder::new(ifam)
            .name("BytePack")
            .display("{dst0} = BYTEPACK ({src0}, {src1})".to_string())
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0x18))
            .set_field_type("aop", FieldType::Mask(0x0))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(Self::init_vars())
            .add_pcode(pack_bytes(2, 1))
            .add_pcode(pack_bytes(0, 1))
            .add_pcode(pack_bytes(2, 0))
            .add_pcode(pack_bytes(0, 0))
    }

    fn unpack_instr(ifam: &InstrFamilyBuilder, sat: bool) -> InstrBuilder {
        fn unpack_bytes(b_idx: usize, rev: bool, dst_id: usize) -> Expr {
            let dst = &format!("dst{dst_id}");
            cs_mline(vec![
                POF::get_byte_i0(b_idx + 4 * rev as usize, POF::TMP8),
                cs_assign_by(e_lshft, e_rfield(dst), b_num(2)),
                cs_assign_by(e_bit_or, e_rfield(dst), b_var(POF::TMP8)),
            ])
        }

        InstrBuilder::new(ifam)
            .name("UnBytePack")
            .display(format!(
                "({{dst1}}, {{dst0}}) = UNBYTEPACK {{src0}}{}",
                if sat { " (R)" } else { "" }
            ))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0x18))
            .set_field_type("aop", FieldType::Mask(0x1))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("dst1", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DRegPair))
            .add_pcode(Self::init_vars())
            .add_pcode(unpack_bytes(3, sat, 1))
            .add_pcode(unpack_bytes(2, sat, 1))
            .add_pcode(unpack_bytes(1, sat, 0))
            .add_pcode(unpack_bytes(0, sat, 0))
    }
}

impl InstrFactory for PackOpFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::pack_instr(ifam),
            Self::unpack_instr(ifam, false),
            Self::unpack_instr(ifam, true),
        ]
    }
}
