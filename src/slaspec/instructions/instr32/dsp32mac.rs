use itertools::Itertools;

use crate::slaspec::globals::REGISTER_SPACE;
use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "Dsp32Mac",
        "Multiply Accumulate",
        "dmc",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0x18), 5),
                ProtoField::new("x2", FieldType::Mask(0x0), 2),
                ProtoField::new("mmod", FieldType::Blank, 4),
                ProtoField::new("mm", FieldType::Blank, 1),
                ProtoField::new("p", FieldType::Blank, 1),
                ProtoField::new("w1", FieldType::Blank, 1),
                ProtoField::new("op1", FieldType::Blank, 2),
            ]),
            ProtoPattern::new(vec![
                ProtoField::new("h01", FieldType::Blank, 1),
                ProtoField::new("h11", FieldType::Blank, 1),
                ProtoField::new("w0", FieldType::Blank, 1),
                ProtoField::new("op0", FieldType::Blank, 2),
                ProtoField::new("h00", FieldType::Blank, 1),
                ProtoField::new("h10", FieldType::Blank, 1),
                ProtoField::new("dst", FieldType::Blank, 3),
                ProtoField::new("src0", FieldType::Blank, 3),
                ProtoField::new("src1", FieldType::Blank, 3),
            ]),
        ],
    );

    ifam.add_instrs(&CmplxMacFactory());

    ifam
}

#[derive(Debug, Clone, Copy)]
enum Dest {
    None,
    Reg,
    Pair,
}

impl Dest {
    fn name(&self) -> String {
        match self {
            Self::None => "",
            Self::Reg => "WithMvN",
            Self::Pair => "WithMv",
        }
        .to_string()
    }

    fn display(&self) -> String {
        match self {
            Self::None => "",
            Self::Reg | Self::Pair => "{dst} = ",
        }
        .to_string()
    }

    fn mask_w0(&self) -> u16 {
        match self {
            Self::None => 0,
            _ => 1,
        }
    }

    fn mask_p(&self) -> u16 {
        match self {
            Self::Pair => 1,
            _ => 0,
        }
    }

    fn set_fields(&self, mut instr: InstrBuilder) -> InstrBuilder {
        instr = instr
            .set_field_type("p", FieldType::Mask(self.mask_p()))
            .set_field_type("w0", FieldType::Mask(self.mask_w0()));

        match self {
            Self::None => instr,
            Self::Reg => instr.set_field_type("dst", FieldType::Variable(RegisterSet::DReg)),
            Self::Pair => instr.set_field_type("dst", FieldType::Variable(RegisterSet::DRegPair)),
        }
    }

    fn is_move(&self) -> bool {
        match self {
            Self::None => false,
            _ => true,
        }
    }

    fn size(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Reg => 2,
            Self::Pair => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cmode {
    Default = 0xd,
    T = 0xe,
    IS = 0xf,
}

impl Cmode {
    fn display(&self) -> String {
        match self {
            Cmode::Default => "",
            Cmode::T => " (T)",
            Cmode::IS => " (IS)",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpAcc {
    Copy = 0,
    Add = 1,
    Sub = 2,
    None = 3,
}

impl OpAcc {
    fn display(&self) -> String {
        match self {
            Self::Copy => "A1:0 = ",
            Self::Add => "A1:0 += ",
            Self::Sub => "A1:0 -= ",
            Self::None => "",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy)]
enum OpCmul {
    AxB = 0,
    AxBc = 1,
    AcxBc = 2,
}

impl OpCmul {
    fn display(&self) -> String {
        match self {
            Self::AxB => "CMUL({src0}, {src1})",
            Self::AxBc => "CMUL({src0}, {src1}*)",
            Self::AcxBc => "CMUL({src0}*, {src1}*)",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy)]
struct CmplxMacParam {
    dst: Dest,
    opa: OpAcc,
    opc: OpCmul,
    mode: Cmode,
}

impl CmplxMacParam {
    const RES_IM_ID: &str = "res_im";
    const RES_RE_ID: &str = "res_re";

    fn name(&self) -> String {
        format!("Mac32Cmplx{}", self.dst.name())
    }

    fn display(&self) -> String {
        let mut op_str = format!("{}{}", self.opa.display(), self.opc.display());
        op_str = if self.dst.is_move() && self.opa != OpAcc::None {
            format!("({})", op_str)
        } else {
            op_str
        };
        format!("{}{}{}", self.dst.display(), op_str, self.mode.display())
    }

    fn set_fields(&self, mut instr: InstrBuilder) -> InstrBuilder {
        instr = instr
            .set_field_type("mmod", FieldType::Mask(self.mode as u16))
            .set_field_type("op1", FieldType::Mask(self.opc as u16))
            .set_field_type("op0", FieldType::Mask(self.opa as u16));

        self.dst.set_fields(instr)
    }

    fn mult_expr(&self, dst: Expr, src_a: Expr, src_b: Expr, id: &str) -> Expr {
        let spe_case_label = b_label(&format!("spe_case_mult_{id}"));

        let mut mult_exprs = vec![];

        if self.mode == Cmode::IS {
            mult_exprs.push(e_copy(dst, e_mult(e_zext(src_a), e_zext(src_b))));
        } else {
            mult_exprs.append(&mut vec![
                e_copy(dst.clone(), b_num(0x7fffffff)),
                b_ifgoto(
                    e_and(
                        e_eq(src_a.clone(), b_num(0x8000)),
                        e_eq(src_b.clone(), b_num(0x8000)),
                    ),
                    spe_case_label.clone(),
                ),
                e_copy(dst.clone(), e_mult(e_zext(src_a), e_zext(src_b))),
                cs_assign_by(e_lshft, dst, b_num(1)),
                spe_case_label,
            ])
        }

        cs_mline(mult_exprs.into())
    }

    fn op_expr(&self) -> Expr {
        let a_im_var = b_var("a_im");
        let a_re_var = b_var("a_re");
        let b_im_var = b_var("b_im");
        let b_re_var = b_var("b_re");
        let tmp_arbi_var = b_var("tmp_arbi");
        let tmp_aibr_var = b_var("tmp_aibr");
        let tmp_arbr_var = b_var("tmp_arbr");
        let tmp_aibi_var = b_var("tmp_aibi");
        let res_im_var = b_var(Self::RES_IM_ID);
        let res_re_var = b_var(Self::RES_RE_ID);

        let mut exprs = vec![
            e_copy(b_local(a_im_var.clone(), 2), b_trunc(e_rfield("src0"), 2)),
            e_copy(b_local(a_re_var.clone(), 2), b_size(e_rfield("src0"), 2)),
            e_copy(b_local(b_im_var.clone(), 2), b_trunc(e_rfield("src1"), 2)),
            e_copy(b_local(b_re_var.clone(), 2), b_size(e_rfield("src1"), 2)),
            b_local(tmp_arbi_var.clone(), 4),
            b_local(tmp_aibr_var.clone(), 4),
            b_local(tmp_arbr_var.clone(), 4),
            b_local(tmp_aibi_var.clone(), 4),
            b_local(res_im_var.clone(), 5),
            b_local(res_re_var.clone(), 5),
            self.mult_expr(
                tmp_arbi_var.clone(),
                a_re_var.clone(),
                b_im_var.clone(),
                "arbi",
            ),
            self.mult_expr(
                tmp_aibr_var.clone(),
                a_im_var.clone(),
                b_re_var.clone(),
                "aibr",
            ),
            self.mult_expr(
                tmp_arbr_var.clone(),
                a_re_var.clone(),
                b_re_var.clone(),
                "arbr",
            ),
            self.mult_expr(
                tmp_aibi_var.clone(),
                a_im_var.clone(),
                b_im_var.clone(),
                "aibi",
            ),
        ];

        match self.opc {
            OpCmul::AxB => exprs.append(&mut vec![
                e_copy(
                    res_im_var.clone(),
                    e_add(e_sext(tmp_arbi_var), e_sext(tmp_aibr_var)),
                ),
                e_copy(
                    res_re_var.clone(),
                    e_sub(e_sext(tmp_arbr_var), e_sext(tmp_aibi_var)),
                ),
            ]),
            OpCmul::AxBc => exprs.append(&mut vec![
                e_copy(
                    res_im_var.clone(),
                    e_sub(e_sext(tmp_aibr_var), e_sext(tmp_arbi_var)),
                ),
                e_copy(
                    res_re_var.clone(),
                    e_add(e_sext(tmp_arbr_var), e_sext(tmp_aibi_var)),
                ),
            ]),
            OpCmul::AcxBc => exprs.append(&mut vec![
                e_copy(
                    res_im_var.clone(),
                    e_neg(b_grp(e_add(e_sext(tmp_arbi_var), e_sext(tmp_aibr_var)))),
                ),
                e_copy(
                    res_re_var.clone(),
                    e_sub(e_sext(tmp_arbr_var), e_sext(tmp_aibi_var)),
                ),
            ]),
        }

        match self.opa {
            OpAcc::Copy => exprs.append(&mut vec![
                e_copy(b_reg("A1"), res_im_var),
                e_copy(b_reg("A0"), res_re_var),
            ]),
            OpAcc::Add => exprs.append(&mut vec![
                cs_sadd_sat(b_reg("A1"), b_reg("A1"), res_im_var, 5, "A1"),
                cs_sadd_sat(b_reg("A0"), b_reg("A0"), res_re_var, 5, "A0"),
            ]),
            OpAcc::Sub => exprs.append(&mut vec![
                cs_ssub_sat(b_reg("A1"), b_reg("A1"), res_im_var, 5, "A1"),
                cs_ssub_sat(b_reg("A0"), b_reg("A0"), res_re_var, 5, "A0"),
            ]),
            _ => {}
        }

        cs_mline(exprs.into())
    }

    fn epxr(&self) -> Expr {
        let im_var = if self.opa == OpAcc::None {
            b_var(Self::RES_IM_ID)
        } else {
            b_reg("A1")
        };
        let re_var = if self.opa == OpAcc::None {
            b_var(Self::RES_RE_ID)
        } else {
            b_reg("A0")
        };

        let mut exprs = vec![self.op_expr()];

        fn cpy_to_var(dst: Expr, src: Expr, dst_info: &Dest, mode: &Cmode, id: &str) -> Expr {
            let tmp_trunc_var = b_var(&format!("tmp_trunc_{id}"));

            let mut exprs = vec![];

            if dst_info.size() == 2 {
                exprs.push(b_local(tmp_trunc_var.clone(), 3));

                match mode {
                    Cmode::Default => {
                        exprs.push(cs_round(tmp_trunc_var.clone(), 3, src.clone(), 5, id))
                    }
                    Cmode::T => exprs.push(e_copy(tmp_trunc_var.clone(), b_trunc(src.clone(), 2))),
                    _ => {}
                }
            }

            exprs.push(cs_strunc_sat(
                dst,
                if mode == &Cmode::IS {
                    src
                } else {
                    tmp_trunc_var
                },
                dst_info.size(),
                id,
            ));

            cs_mline(exprs.into())
        }

        if self.dst.is_move() {
            let half_tmp_var = b_var("half_tmp");
            let halfaddr_im_var = b_var("half_im_addr");
            let halfaddr_re_var = b_var("half_re_addr");

            exprs.push(b_local(half_tmp_var.clone(), self.dst.size()));
            exprs.push(e_copy(
                b_local(halfaddr_re_var.clone(), 2),
                b_ref(e_rfield("dst")),
            ));
            exprs.push(e_copy(
                b_local(halfaddr_im_var.clone(), 2),
                e_add(b_ref(e_rfield("dst")), b_num(self.dst.size() as isize)),
            ));

            exprs.push(cpy_to_var(
                half_tmp_var.clone(),
                im_var,
                &self.dst,
                &self.mode,
                "dst_im",
            ));
            exprs.push(e_copy(
                b_ptr(REGISTER_SPACE, halfaddr_im_var, self.dst.size()),
                half_tmp_var.clone(),
            ));

            exprs.push(cpy_to_var(
                half_tmp_var.clone(),
                re_var,
                &self.dst,
                &self.mode,
                "dst_re",
            ));
            exprs.push(e_copy(
                b_ptr(REGISTER_SPACE, halfaddr_re_var, self.dst.size()),
                half_tmp_var.clone(),
            ));
        }

        cs_mline(exprs.into())
    }

    fn new(dst: &Dest, opa: &OpAcc, opc: &OpCmul, mode: &Cmode) -> Self {
        Self {
            dst: *dst,
            opa: *opa,
            opc: *opc,
            mode: *mode,
        }
    }

    fn all_params() -> Vec<Self> {
        let cmode = [Cmode::Default, Cmode::IS];
        let ncmode = [Cmode::Default, Cmode::T, Cmode::IS];

        let opacc = [OpAcc::Copy, OpAcc::Add, OpAcc::Sub];
        let opaccmv = [OpAcc::Copy, OpAcc::Add, OpAcc::Sub, OpAcc::None];

        let opcmul = [OpCmul::AxB, OpCmul::AxBc, OpCmul::AcxBc];

        let mut no_mv: Vec<CmplxMacParam> = cmode
            .iter()
            .cartesian_product(opacc.iter())
            .cartesian_product(opcmul.iter())
            .map(|((mode, opa), opc)| Self::new(&Dest::None, opa, opc, mode))
            .collect();

        let mut reg: Vec<CmplxMacParam> = ncmode
            .iter()
            .cartesian_product(opaccmv.iter())
            .cartesian_product(opcmul.iter())
            .map(|((mode, opa), opc)| Self::new(&Dest::Reg, opa, opc, mode))
            .collect();

        let mut pair: Vec<CmplxMacParam> = cmode
            .iter()
            .cartesian_product(opaccmv.iter())
            .cartesian_product(opcmul.iter())
            .map(|((mode, opa), opc)| Self::new(&Dest::Pair, opa, opc, mode))
            .collect();

        no_mv.append(&mut reg);
        no_mv.append(&mut pair);

        no_mv
    }
}

struct CmplxMacFactory();

impl CmplxMacFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, param: &CmplxMacParam) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name(&param.name())
            .display(param.display())
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(param.epxr());

        instr = param.set_fields(instr);

        instr
    }
}

impl InstrFactory for CmplxMacFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        CmplxMacParam::all_params()
            .iter()
            .map(|param| Self::base_instr(ifam, param))
            .collect()
    }
}
