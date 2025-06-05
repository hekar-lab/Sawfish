use std::usize;

use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

type VOF = VideoOpFactory;

pub struct VideoOpFactory();

impl VideoOpFactory {
    const SRC0: &'static str = "bytes_src0";
    const SRC1: &'static str = "bytes_src1";
    const TMP: &'static str = "tmp_vidop";
    const RES: &'static str = "res_vidop";

    fn init_vars() -> Expr {
        cs_mline(vec![
            e_local(Self::SRC0, 8),
            e_local(Self::SRC1, 8),
            e_local(Self::RES, 8),
            e_local(Self::TMP, 8),
        ])
    }

    fn get_byte(b_idx: usize, dstid: &str, srcid: usize, iid: usize) -> Expr {
        e_copy(
            b_var(dstid),
            e_bit_and(
                b_num(0xff),
                e_rshft(
                    e_rfield(&format!("src{srcid}")),
                    b_grp(e_mult(
                        b_num(8),
                        b_grp(e_rem(
                            b_grp(e_add(
                                b_grp(e_bit_and(b_reg(&format!("I{iid}")), b_num(0x3))),
                                b_num(b_idx as i128),
                            )),
                            b_num(8),
                        )),
                    )),
                ),
            ),
        )
    }

    fn get_sword(b_idx: usize, dstid: &str, srcid: usize, iid: usize) -> Expr {
        cs_mline(vec![
            Self::get_byte(b_idx, dstid, srcid, iid),
            Self::get_byte(b_idx + 1, VOF::TMP, srcid, iid),
            cs_assign_by(e_bit_or, b_var(dstid), e_lshft(b_var(VOF::TMP), b_num(8))),
            cs_assign_by(e_lshft, b_var(dstid), b_num(6)),
            cs_assign_by(e_arshft, b_var(dstid), b_num(6)),
        ])
    }

    fn get_bytepair(b_idx: usize, rev: bool) -> Expr {
        cs_mline(vec![
            Self::get_byte(b_idx + 4 * rev as usize, Self::SRC0, 0, 0),
            Self::get_byte(b_idx + 4 * rev as usize, Self::SRC1, 1, 1),
        ])
    }

    fn get_bytepair_i0(b_idx: usize, rev: bool) -> Expr {
        cs_mline(vec![
            Self::get_byte(b_idx + 4 * rev as usize, Self::SRC0, 0, 0),
            Self::get_byte(b_idx + 4 * rev as usize, Self::SRC1, 1, 0),
        ])
    }

    fn get_bytewordpair(b_idx: usize, rev: bool) -> Expr {
        cs_mline(vec![
            Self::get_sword(b_idx - (b_idx % 2) + 4 * rev as usize, Self::SRC0, 0, 0),
            Self::get_byte(b_idx + 4 * rev as usize, Self::SRC1, 1, 1),
        ])
    }

    fn base_instr(ifam: &InstrFamilyBuilder, sat: bool) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .set_field_type("s", FieldType::Mask(sat as u16))
            .set_field_type("x", FieldType::Mask(0x0))
    }

    fn byteop_inst(ifam: &InstrFamilyBuilder, sat: bool) -> InstrBuilder {
        Self::base_instr(ifam, sat)
            .set_field_type("dst0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DRegPair))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DRegPair))
    }

    fn byteop1p_instr(ifam: &InstrFamilyBuilder, aop: bool, sat: bool) -> InstrBuilder {
        let mut opt_vec = vec![];
        if aop {
            opt_vec.push("T");
        }
        if sat {
            opt_vec.push("R");
        }
        let opt_str = if opt_vec.is_empty() {
            "".to_string()
        } else {
            format!(" ({})", opt_vec.join(", "))
        };

        fn avgb(b_idx: usize, trunc: bool, rev: bool) -> Expr {
            let mut code = vec![
                VideoOpFactory::get_bytepair(b_idx, rev),
                e_copy(b_var(VOF::RES), e_add(b_var(VOF::SRC0), b_var(VOF::SRC1))),
            ];

            if trunc {
                code.push(cs_assign_by(e_rshft, b_var(VOF::RES), b_num(1)));
            } else {
                code.push(e_copy(
                    b_var(VOF::TMP),
                    e_bit_and(b_var(VOF::RES), b_num(1)),
                ));
                code.push(cs_assign_by(e_rshft, b_var(VOF::RES), b_num(1)));
                code.push(cs_assign_by(e_add, b_var(VOF::RES), b_var(VOF::TMP)));
            }

            code.push(cs_assign_by(e_lshft, e_rfield("dst0"), b_num(8)));
            code.push(cs_assign_by(
                e_bit_or,
                e_rfield("dst0"),
                b_size(b_var(VOF::RES), 4),
            ));
            cs_mline(code)
        }

        Self::byteop_inst(ifam, sat)
            .name("Avg8Vec")
            .display(format!("{{dst0}} = BYTEOP1P ({{src0}}, {{src1}}){opt_str}",))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0x14))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .add_pcode(Self::init_vars())
            .add_pcode(avgb(3, aop, sat))
            .add_pcode(avgb(2, aop, sat))
            .add_pcode(avgb(1, aop, sat))
            .add_pcode(avgb(0, aop, sat))
    }

    fn byteop16_instr(ifam: &InstrFamilyBuilder, aop: bool, sat: bool) -> InstrBuilder {
        let op_str = if aop { "M" } else { "P" };
        let opt_str = if sat { " (R)" } else { "" };

        fn addsub(b_idx: usize, sub: bool, rev: bool, dst_id: usize) -> Expr {
            let op = if sub { e_sub } else { e_add };
            let dst = &format!("dst{dst_id}");

            cs_mline(vec![
                VideoOpFactory::get_bytepair(b_idx, rev),
                e_copy(
                    b_var(VOF::RES),
                    e_bit_and(b_num(0xffff), op(b_var(VOF::SRC0), b_var(VOF::SRC1))),
                ),
                cs_assign_by(e_lshft, e_rfield(dst), b_num(16)),
                cs_assign_by(e_bit_or, e_rfield(dst), b_size(b_var(VOF::RES), 4)),
            ])
        }

        Self::byteop_inst(ifam, sat)
            .name("AddSub4x8")
            .display(format!(
                "({{dst1}}, {{dst0}}) = BYTEOP16{op_str} ({{src0}}, {{src1}}){opt_str}",
            ))
            .set_field_type("hl", FieldType::Mask(0x0))
            .set_field_type("aopc", FieldType::Mask(0x15))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .set_field_type("dst1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(Self::init_vars())
            .add_pcode(addsub(3, aop, sat, 1))
            .add_pcode(addsub(2, aop, sat, 1))
            .add_pcode(addsub(1, aop, sat, 0))
            .add_pcode(addsub(0, aop, sat, 0))
    }

    fn byteop2p_instr(ifam: &InstrFamilyBuilder, aop: bool, hl: bool, sat: bool) -> InstrBuilder {
        let opt_str = format!(
            "{}{}{}",
            if aop { "T" } else { "RND" },
            if hl { "H" } else { "L" },
            if sat { ", R" } else { "" }
        );

        fn avgb4(b_idx: usize, trunc: bool, hl: bool, rev: bool) -> Expr {
            let mut code = vec![
                VideoOpFactory::get_bytepair_i0(b_idx, rev),
                e_copy(b_var(VOF::RES), e_add(b_var(VOF::SRC0), b_var(VOF::SRC1))),
                VideoOpFactory::get_bytepair_i0(b_idx + 1, rev),
                e_copy(
                    b_var(VOF::RES),
                    e_add(b_var(VOF::RES), e_add(b_var(VOF::SRC0), b_var(VOF::SRC1))),
                ),
            ];

            if trunc {
                code.push(cs_assign_by(e_rshft, b_var(VOF::RES), b_num(2)));
            } else {
                code.push(e_copy(
                    b_var(VOF::TMP),
                    e_rshft(e_bit_and(b_var(VOF::RES), b_num(2)), b_num(1)),
                ));
                code.push(cs_assign_by(e_rshft, b_var(VOF::RES), b_num(2)));
                code.push(cs_assign_by(e_add, b_var(VOF::RES), b_var(VOF::TMP)));
            }

            code.push(cs_assign_by(
                e_lshft,
                e_rfield("dst0"),
                b_num(if hl { 8 } else { 16 }),
            ));
            code.push(cs_assign_by(
                e_bit_or,
                e_rfield("dst0"),
                b_size(b_var(VOF::RES), 4),
            ));
            if hl {
                code.push(cs_assign_by(e_lshft, e_rfield("dst0"), b_num(8)));
            }

            cs_mline(code)
        }

        Self::byteop_inst(ifam, sat)
            .name("Avg4x8Vec")
            .display(format!(
                "{{dst0}} = BYTEOP2P ({{src0}}, {{src1}}) ({opt_str})",
            ))
            .set_field_type("hl", FieldType::Mask(hl as u16))
            .set_field_type("aopc", FieldType::Mask(0x16))
            .set_field_type("aop", FieldType::Mask(aop as u16))
            .add_pcode(Self::init_vars())
            .add_pcode(avgb4(2, aop, hl, sat))
            .add_pcode(avgb4(0, aop, hl, sat))
    }

    fn byteop3p_instr(ifam: &InstrFamilyBuilder, hl: bool, sat: bool) -> InstrBuilder {
        let opt_str = format!(
            "{}{}",
            if hl { "HI" } else { "LO" },
            if sat { ", R" } else { "" }
        );

        fn addclip(w_idx: usize, hl: bool, rev: bool) -> Expr {
            let b_idx = 2 * w_idx + (!hl) as usize;

            let mut code = vec![
                VideoOpFactory::get_bytewordpair(b_idx, rev),
                e_copy(
                    b_var(VOF::RES),
                    e_bit_and(e_add(b_var(VOF::SRC0), b_var(VOF::SRC1)), b_num(0xff)),
                ),
                cs_assign_by(e_lshft, e_rfield("dst0"), b_num(if hl { 8 } else { 16 })),
                cs_assign_by(e_bit_or, e_rfield("dst0"), b_size(b_var(VOF::RES), 4)),
            ];

            if hl {
                code.push(cs_assign_by(e_lshft, e_rfield("dst0"), b_num(8)));
            }

            cs_mline(code)
        }

        Self::byteop_inst(ifam, sat)
            .name("AddClip")
            .display(format!(
                "{{dst0}} = BYTEOP2P ({{src0}}, {{src1}}) ({opt_str})",
            ))
            .set_field_type("hl", FieldType::Mask(hl as u16))
            .set_field_type("aopc", FieldType::Mask(0x17))
            .set_field_type("aop", FieldType::Mask(0x0))
            .add_pcode(Self::init_vars())
            .add_pcode(if hl {
                addclip(2, hl, sat)
            } else {
                addclip(3, hl, sat)
            })
            .add_pcode(if hl {
                addclip(0, hl, sat)
            } else {
                addclip(1, hl, sat)
            })
    }
}

impl InstrFactory for VideoOpFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let bo1p_instrs: Vec<InstrBuilder> = [false, true]
            .into_iter()
            .cartesian_product([false, true])
            .map(|(aop, sat)| Self::byteop1p_instr(ifam, aop, sat))
            .collect();

        let bo16_instrs: Vec<InstrBuilder> = [false, true]
            .into_iter()
            .cartesian_product([false, true])
            .map(|(aop, sat)| Self::byteop16_instr(ifam, aop, sat))
            .collect();

        let bo2p_instrs: Vec<InstrBuilder> = [false, true]
            .into_iter()
            .cartesian_product([false, true])
            .cartesian_product([false, true])
            .map(|((aop, hl), sat)| Self::byteop2p_instr(ifam, aop, hl, sat))
            .collect();

        let bo3p_instrs: Vec<InstrBuilder> = [false, true]
            .into_iter()
            .cartesian_product([false, true])
            .map(|(hl, sat)| Self::byteop3p_instr(ifam, hl, sat))
            .collect();

        vec![bo1p_instrs, bo16_instrs, bo2p_instrs, bo3p_instrs].concat()
    }
}
