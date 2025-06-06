use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::RegisterSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mmode {
    Default = 0x0,
    S2RND = 0x1,
    T = 0x2,
    W32 = 0x3,
    FU = 0x4,
    TFU = 0x6,
    IS = 0x8,
    ISS2 = 0x9,
    IH = 0xb,
    IU = 0xc,
}

impl Mmode {
    pub fn to_str(&self) -> Option<String> {
        match self {
            Self::Default => None,
            _ => Some(format!("{:?}", self)),
        }
    }

    pub fn mmod0() -> Vec<Mmode> {
        vec![Mmode::Default, Mmode::W32, Mmode::FU, Mmode::IS]
    }

    pub fn mmod1() -> Vec<Mmode> {
        vec![
            Mmode::Default,
            Mmode::S2RND,
            Mmode::T,
            Mmode::FU,
            Mmode::TFU,
            Mmode::IS,
            Mmode::ISS2,
            Mmode::IH,
            Mmode::IU,
        ]
    }

    pub fn mmode() -> Vec<Mmode> {
        vec![
            Mmode::Default,
            Mmode::S2RND,
            Mmode::FU,
            Mmode::IS,
            Mmode::ISS2,
            Mmode::IU,
        ]
    }

    fn ext(&self) -> fn(Expr) -> Expr {
        match self {
            Self::Default
            | Self::S2RND
            | Self::T
            | Self::W32
            | Self::IS
            | Self::ISS2
            | Self::IH => e_sext,
            Self::FU | Self::TFU | Self::IU => e_zext,
        }
    }

    fn shft_correct(&self) -> bool {
        match self {
            Self::Default | Self::S2RND | Self::T | Self::W32 => true,
            Self::FU | Self::TFU | Self::IS | Self::ISS2 | Self::IH | Self::IU => false,
        }
    }

    fn sat32(&self) -> bool {
        match self {
            Self::W32 | Self::IH => true,
            _ => false,
        }
    }

    fn signed(&self) -> bool {
        match self {
            Self::FU | Self::TFU | Self::IU => false,
            _ => true,
        }
    }

    fn fraction(&self) -> bool {
        match self {
            Self::IS | Self::ISS2 | Self::IH | Self::IU => false,
            _ => true,
        }
    }

    fn extract_2x(&self) -> bool {
        match self {
            Self::S2RND | Self::ISS2 => true,
            _ => false,
        }
    }

    fn extract_trunc(&self) -> bool {
        match self {
            Self::T | Self::TFU => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccOp {
    Copy = 0,
    Add = 1,
    Sub = 2,
    None = 3,
}

impl AccOp {
    pub fn all() -> Vec<Self> {
        vec![Self::Copy, Self::Add, Self::Sub, Self::None]
    }

    pub fn op_str(&self) -> String {
        match self {
            Self::Copy => "=",
            Self::Add => "+=",
            Self::Sub => "-=",
            _ => "",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acc {
    A0 = 0,
    A1 = 1,
}

impl Acc {
    pub fn to_str(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Half {
    L = 0,
    H = 1,
}

impl Half {
    pub fn regset(&self) -> RegisterSet {
        match self {
            Self::L => RegisterSet::DRegL,
            Self::H => RegisterSet::DRegH,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Oper {
    pub lhs: Half,
    pub rhs: Half,
}

impl Oper {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                lhs: Half::L,
                rhs: Half::L,
            },
            Self {
                lhs: Half::L,
                rhs: Half::H,
            },
            Self {
                lhs: Half::H,
                rhs: Half::L,
            },
            Self {
                lhs: Half::H,
                rhs: Half::H,
            },
        ]
    }
}

pub fn mult_expr(
    dst_id: &str,
    src0_id: &str,
    src1_id: &str,
    mode: Mmode,
    mml: bool,
    res_size: usize,
) -> Expr {
    let dst = b_var(dst_id);
    let src0 = e_rfield(src0_id);
    let src1 = e_rfield(src1_id);
    let mut code = vec![];

    let src0ext = if mml { e_zext } else { mode.ext() };
    let src1ext = if mml { e_sext } else { mode.ext() };

    code.push(e_copy(
        b_local(dst.clone(), res_size),
        e_mult(src0ext(src0), src1ext(src1)),
    ));

    if !mml && mode.shft_correct() {
        code.push(cs_assign_by(e_lshft, dst, b_num(1)));
    }

    cs_mline(code)
}

pub fn acc_expr(
    acc: Expr,
    accop: AccOp,
    acc_size: usize,
    res_id: &str,
    mode: Mmode,
    ns: bool,
    id: &str,
) -> Expr {
    let res = b_var(res_id);
    let mut code = vec![];

    match accop {
        AccOp::Copy => code.push(e_copy(acc.clone(), res.clone())),
        AccOp::Add => code.push(if ns {
            cs_assign_by(e_add, acc.clone(), res.clone())
        } else {
            if mode.signed() {
                cs_sadd_sat(acc.clone(), acc.clone(), res.clone(), acc_size, id)
            } else {
                cs_add_sat(acc.clone(), acc.clone(), res.clone(), acc_size, id)
            }
        }),
        AccOp::Sub => code.push(if ns {
            cs_assign_by(e_sub, acc.clone(), res.clone())
        } else {
            if mode.signed() {
                cs_ssub_sat(acc.clone(), acc.clone(), res.clone(), acc_size, id)
            } else {
                cs_sub_sat(acc.clone(), acc.clone(), res.clone(), id)
            }
        }),
        _ => {}
    }

    if mode.sat32() {
        let tmp_var = b_var(&format!("sat32_tmp_{}", id));
        code.push(b_local(tmp_var.clone(), 4));
        // 32 bit saturation is not compatible with the no saturation directive
        // So no need to account for it here.
        code.push(cs_strunc_sat(tmp_var.clone(), acc.clone(), 4, id));
        code.push(e_copy(acc, e_sext(tmp_var)));
    }

    cs_mline(code)
}

pub fn extract_expr(
    dst_id: &str,
    mut src: Expr,
    full_reg: bool,
    mode: Mmode,
    ns: bool,
    res_size: usize,
    id: &str,
) -> Expr {
    let dst = e_rfield(dst_id);
    let src_2x = b_var(&format!("tmp_2x_src_{}", id));
    let rnd_dst = b_var(&format!("tmp_rnd_{}", id));
    let reg_size = if full_reg {
        res_size - 1
    } else {
        (res_size - 1) / 2
    };
    let mut code = vec![];

    if mode.extract_2x() {
        code.push(b_local(src_2x.clone(), res_size));
        code.push(e_copy(src_2x.clone(), e_mult(src.clone(), b_num(2))));
    }

    src = if mode.extract_2x() { src_2x } else { src };

    if !full_reg {
        if mode.fraction() {
            let rnd_size = res_size - reg_size;
            code.push(b_local(rnd_dst.clone(), rnd_size));
            if mode.extract_trunc() {
                code.push(e_copy(rnd_dst.clone(), b_trunc(src.clone(), reg_size)));
            } else {
                code.push(cs_round(
                    rnd_dst.clone(),
                    rnd_size,
                    src.clone(),
                    res_size,
                    id,
                ));
            }
        }

        src = if mode.fraction() { rnd_dst } else { src };
    }

    if ns {
        // The no saturation directive only works with integer so we can just truncate
        code.push(e_copy(dst, b_size(src, reg_size)));
    } else {
        if mode.signed() {
            code.push(cs_strunc_sat(dst, src, reg_size, id));
        } else {
            code.push(cs_trunc_sat(dst, src, reg_size, id));
        }
    }

    cs_mline(code)
}
