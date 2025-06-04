use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

pub struct VidOpMiscFactory();

impl VidOpMiscFactory {
    fn init_expr() -> Expr {
        cs_mline(vec![
            e_local("tmp_shft", 8),
            e_local("res_byte0", 1),
            e_local("res_byte1", 1),
            e_local("res_abs_diff", 2),
        ])
    }

    fn get_byte(byte_n: i128, srcid: usize) -> Expr {
        cs_mline(vec![
            e_copy(
                b_var("tmp_shft"),
                e_rshft(
                    e_rfield(&format!("src{srcid}")),
                    b_grp(e_mult(
                        b_num(8),
                        b_grp(e_rem(
                            b_grp(e_add(
                                b_grp(e_bit_and(b_reg(&format!("I{srcid}")), b_num(0x3))),
                                b_num(byte_n),
                            )),
                            b_num(8),
                        )),
                    )),
                ),
            ),
            e_copy(
                b_var(&format!("res_byte{srcid}")),
                b_size(b_var("tmp_shft"), 1),
            ),
        ])
    }

    fn set_acc(byte_n: i128, rev: bool) -> Expr {
        let acc_id = format!(
            "A{}.{}",
            (byte_n >= 2) as usize,
            if byte_n % 2 == 0 { "L" } else { "H" }
        );
        let end_lab = b_label(&format!("end_sad_b{byte_n}"));
        let absdiff_var = b_var("res_abs_diff");
        let byte0_var = b_var("res_byte0");
        let byte1_var = b_var("res_byte1");
        cs_mline(vec![
            Self::get_byte(byte_n + (rev as i128) * 4, 0),
            Self::get_byte(byte_n + (rev as i128) * 4, 1),
            e_copy(
                absdiff_var.clone(),
                e_zext(e_sub(byte1_var.clone(), byte0_var.clone())),
            ),
            b_ifgoto(e_ge(byte0_var.clone(), byte1_var.clone()), end_lab.clone()),
            e_copy(
                absdiff_var.clone(),
                e_zext(e_sub(byte1_var.clone(), byte0_var.clone())),
            ),
            end_lab,
            cs_add_sat(
                b_reg(&acc_id),
                b_reg(&acc_id),
                absdiff_var,
                2,
                &format!("sad_b{byte_n}"),
            ),
        ])
    }

    fn sadv_instr(ifam: &InstrFamilyBuilder, sat: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("SAD8Vec")
            .display(format!(
                "SAA ({{src0}}, {{src1}}){}",
                if sat { " (R)" } else { "" }
            ))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0x12))
            .set_field_type("aop", FieldType::Mask(0x0))
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(0x0))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DRegPair))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DRegPair))
            .add_pcode(Self::init_expr())
            .add_pcode(Self::set_acc(0, sat))
            .add_pcode(Self::set_acc(1, sat))
            .add_pcode(Self::set_acc(2, sat))
            .add_pcode(Self::set_acc(3, sat))
    }

    fn disalign_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("DisAlignExcept")
            .display("DISALIGNEXCPT".to_string())
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0x12))
            .set_field_type("aop", FieldType::Mask(0x3))
            .set_field_type("s", FieldType::Mask(0x0))
            .set_field_type("x", FieldType::Mask(0x0))
            .add_pcode(e_mac("disalignexcpt"))
    }
}

impl InstrFactory for VidOpMiscFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::sadv_instr(ifam, false),
            Self::sadv_instr(ifam, true),
            Self::disalign_instr(ifam),
        ]
    }
}
