use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mmode {
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
    fn display(&self) -> Option<String> {
        match self {
            Self::Default => None,
            _ => Some(format!("{:?}", self)),
        }
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
enum AccOp {
    Copy = 0,
    Add = 1,
    Sub = 2,
    None = 3,
}

impl AccOp {
    fn display(&self) -> String {
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
enum Half {
    L = 0,
    H = 1,
}

impl Half {
    fn regset(&self) -> RegisterSet {
        match self {
            Self::L => RegisterSet::DRegL,
            Self::H => RegisterSet::DRegH,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Acc {
    A0 = 0,
    A1 = 1,
}

impl Acc {
    fn to_str(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, Copy)]
struct Mac {
    opa: AccOp,
    acc: Acc,
    assign: bool,
}

impl Mac {
    fn new(opa: AccOp, acc: Acc, assign: bool) -> Self {
        Mac { opa, acc, assign }
    }

    fn id(&self) -> String {
        self.acc.to_str()
    }

    fn reg_name(&self, full_reg: bool) -> String {
        format!(
            "Dreg{}",
            if full_reg {
                match self.acc {
                    Acc::A0 => "E",
                    Acc::A1 => "O",
                }
            } else {
                match self.acc {
                    Acc::A0 => "L",
                    Acc::A1 => "H",
                }
            }
        )
    }

    fn name(&self, mv_para: bool, full_reg: bool) -> String {
        if self.assign {
            if self.opa == AccOp::None {
                if mv_para {
                    "Mv".to_string()
                } else {
                    format!("Mv{}To{}", &self.acc.to_str(), self.reg_name(full_reg))
                }
            } else {
                "Mac16WithMv".to_string()
            }
        } else {
            "Mac16".to_string()
        }
    }

    fn mv_acc(&self) -> bool {
        self.opa == AccOp::None && self.assign
    }

    fn display(&self, mode: Mmode, mml: bool) -> String {
        let op_str = if self.opa == AccOp::None {
            self.acc.to_str()
        } else {
            format!(
                "{} {} {{src0}} * {{src1}}",
                self.acc.to_str(),
                self.opa.display()
            )
        };
        let mut opt_vec = vec![];

        if let Some(mode) = mode.display() {
            opt_vec.push(mode);
        }

        if mml {
            opt_vec.push("M".to_string());
        }

        let opt_str = if opt_vec.is_empty() {
            String::new()
        } else {
            format!(" ({})", opt_vec.join(", "))
        };

        if self.assign {
            if self.opa == AccOp::None {
                format!("{{dst{}}} = {op_str}{opt_str}", self.acc.to_str())
            } else {
                format!("{{dst{}}} = ({op_str}){opt_str}", self.acc.to_str())
            }
        } else {
            format!("{op_str}{opt_str}")
        }
    }

    fn set_fields(&self, instr: InstrBuilder, oper: Oper) -> InstrBuilder {
        instr
            .set_field_type(
                &format!("h0{}", self.acc as u32),
                FieldType::Mask(oper.lhs as u16),
            )
            .set_field_type(
                &format!("h1{}", self.acc as u32),
                FieldType::Mask(oper.rhs as u16),
            )
            .set_field_type(
                &format!("op{}", self.acc as u32),
                FieldType::Mask(self.opa as u16),
            )
            .set_field_type(
                &format!("w{}", self.acc as u32),
                FieldType::Mask(self.assign as u16),
            )
    }

    fn mult_expr(
        &self,
        dst_id: &str,
        src0_id: &str,
        src1_id: &str,
        mode: Mmode,
        mml: bool,
    ) -> Expr {
        let dst = b_var(dst_id);
        let src0 = e_rfield(src0_id);
        let src1 = e_rfield(src1_id);
        let mut code = vec![];

        let src0ext = if mml { e_zext } else { mode.ext() };
        let src1ext = if mml { e_sext } else { mode.ext() };

        code.push(e_copy(
            b_local(dst.clone(), 5),
            e_mult(src0ext(src0), src1ext(src1)),
        ));

        if !mml && mode.shft_correct() {
            code.push(cs_assign_by(e_lshft, dst, b_num(1)));
        }

        cs_mline(code.into())
    }

    fn acc_expr(&self, res_id: &str, mode: Mmode) -> Expr {
        let acc = b_reg(&self.acc.to_str());
        let res = b_var(res_id);
        let mut code = vec![];

        match self.opa {
            AccOp::Copy => code.push(e_copy(acc.clone(), res.clone())),
            AccOp::Add => code.push(if mode.signed() {
                cs_sadd_sat(acc.clone(), acc.clone(), res.clone(), 5, &self.id())
            } else {
                cs_add_sat(acc.clone(), acc.clone(), res.clone(), 5, &self.id())
            }),
            AccOp::Sub => code.push(if mode.signed() {
                cs_ssub_sat(acc.clone(), acc.clone(), res.clone(), 5, &self.id())
            } else {
                cs_sub_sat(acc.clone(), acc.clone(), res.clone(), &self.id())
            }),
            _ => {}
        }

        if mode.sat32() {
            let tmp_var = b_var(&format!("sat32_tmp_{}", &self.id()));
            code.push(b_local(tmp_var.clone(), 4));
            code.push(cs_strunc_sat(tmp_var.clone(), acc.clone(), 4, &self.id()));
            code.push(e_copy(acc, e_sext(tmp_var)));
        }

        cs_mline(code.into())
    }

    fn extract_expr(&self, dst_id: &str, mut src: Expr, full_reg: bool, mode: Mmode) -> Expr {
        let dst = e_rfield(dst_id);
        let src_2x = b_var(&format!("tmp_2x_src_{}", &self.id()));
        let rnd_dst = b_var(&format!("tmp_rnd_{}", &self.id()));
        let dst_size = if full_reg { 4 } else { 2 };
        let mut code = vec![];

        if mode.extract_2x() {
            code.push(b_local(src_2x.clone(), 5));
            code.push(e_copy(src_2x.clone(), e_mult(src.clone(), b_num(2))));
        }

        src = if mode.extract_2x() { src_2x } else { src };

        if !full_reg {
            if mode.fraction() {
                code.push(b_local(rnd_dst.clone(), 3));
                if mode.extract_trunc() {
                    code.push(e_copy(rnd_dst.clone(), b_trunc(src.clone(), 2)));
                } else {
                    code.push(cs_round(rnd_dst.clone(), 3, src.clone(), 5, &self.id()));
                }
            }

            src = if mode.fraction() { rnd_dst } else { src };
        }

        if mode.signed() {
            code.push(cs_strunc_sat(dst, src, dst_size, &self.id()));
        } else {
            code.push(cs_trunc_sat(dst, src, dst_size, &self.id()));
        }

        cs_mline(code.into())
    }

    fn expr(&self, full_reg: bool, mode: Mmode, mml: bool) -> Expr {
        let res_id = &format!("results_{}", self.acc.to_str());
        let src0_id = "src0";
        let src1_id = "src1";
        let dst_id = &format!("dst{}", self.acc.to_str());
        let mut code = vec![];

        if self.opa != AccOp::None {
            code.push(self.mult_expr(res_id, src0_id, src1_id, mode, mml));
            code.push(self.acc_expr(res_id, mode));
        }

        if self.assign {
            let src = if self.opa != AccOp::None {
                b_var(res_id)
            } else {
                b_reg(&self.acc.to_str())
            };
            code.push(self.extract_expr(dst_id, src, full_reg, mode));
        }

        cs_mline(code.into())
    }
}

#[derive(Debug, Clone, Copy)]
struct Oper {
    lhs: Half,
    rhs: Half,
}

impl Oper {
    fn all() -> Vec<Self> {
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

#[derive(Debug, Clone, Copy)]
enum MacEnum {
    Mac0(Mac),
    Mac1(Mac),
    Mac10(Mac, Mac),
}

#[derive(Debug, Clone)]
struct TradMacParam {
    mac0: Option<Mac>,
    mac1: Option<Mac>,
    mode: Mmode,
    mml: bool,
    operands: Oper,
    full_reg: bool,
}

impl TradMacParam {
    fn new(
        mac0: Option<Mac>,
        mac1: Option<Mac>,
        mode: Mmode,
        mml: bool,
        operands: Oper,
        full_reg: bool,
    ) -> Self {
        Self {
            mac0,
            mac1,
            mode,
            mml,
            operands,
            full_reg,
        }
    }

    fn mac_enum(&self) -> MacEnum {
        if self.mac0.is_some() && self.mac1.is_some() {
            MacEnum::Mac10(self.mac0.unwrap(), self.mac1.unwrap())
        } else if self.mac0.is_some() {
            MacEnum::Mac0(self.mac0.unwrap())
        } else if self.mac1.is_some() {
            MacEnum::Mac1(self.mac1.unwrap())
        } else {
            panic!("At least one Mac must be set to make a MacEnum")
        }
    }

    fn name(&self) -> String {
        match self.mac_enum() {
            MacEnum::Mac0(mac) | MacEnum::Mac1(mac) => mac.name(false, self.full_reg),
            MacEnum::Mac10(mac0, mac1) => {
                let mv_para = mac0.mv_acc() && mac1.mv_acc();
                format!(
                    "Para{}And{}",
                    mac0.name(mv_para, self.full_reg),
                    mac1.name(mv_para, self.full_reg)
                )
            }
        }
    }

    fn display(&self) -> String {
        match self.mac_enum() {
            MacEnum::Mac0(mac) => mac.display(self.mode, false),
            MacEnum::Mac1(mac) => mac.display(self.mode, self.mml),
            MacEnum::Mac10(mac0, mac1) => format!(
                "{}, {}",
                mac0.display(self.mode, false),
                mac1.display(Mmode::Default, self.mml)
            ),
        }
    }

    fn set_fields(&self, instr: InstrBuilder) -> InstrBuilder {
        let instr = match self.mac_enum() {
            MacEnum::Mac0(mac) => mac
                .set_fields(instr, self.operands)
                .set_field_type("op1", FieldType::Mask(0x3))
                .set_field_type("w1", FieldType::Mask(0x0)),
            MacEnum::Mac1(mac) => mac
                .set_fields(instr, self.operands)
                .set_field_type("op0", FieldType::Mask(0x3))
                .set_field_type("w0", FieldType::Mask(0x0)),
            MacEnum::Mac10(mac0, mac1) => {
                mac1.set_fields(mac0.set_fields(instr, self.operands), self.operands)
            }
        }
        .set_field_type("mmod", FieldType::Mask(self.mode as u16))
        .set_field_type("mm", FieldType::Mask(self.mml as u16))
        .set_field_type("p", FieldType::Mask(self.full_reg as u16))
        .set_field_type("src0", FieldType::Variable(self.operands.lhs.regset()))
        .set_field_type("src1", FieldType::Variable(self.operands.rhs.regset()));

        if self.full_reg {
            instr.divide_field(
                "dst",
                ProtoPattern::new(vec![
                    ProtoField::new("dstA0", FieldType::Variable(RegisterSet::DRegE), 3),
                    ProtoField::new("dstA1", FieldType::Variable(RegisterSet::DRegO), 3),
                ]),
            )
        } else {
            instr.divide_field(
                "dst",
                ProtoPattern::new(vec![
                    ProtoField::new("dstA0", FieldType::Variable(RegisterSet::DRegL), 3),
                    ProtoField::new("dstA1", FieldType::Variable(RegisterSet::DRegH), 3),
                ]),
            )
        }
    }

    fn pcode(&self) -> Expr {
        match self.mac_enum() {
            MacEnum::Mac0(mac) => mac.expr(self.full_reg, self.mode, false),
            MacEnum::Mac1(mac) => mac.expr(self.full_reg, self.mode, self.mml),
            MacEnum::Mac10(mac0, mac1) => cs_mline(
                vec![
                    mac0.expr(self.full_reg, self.mode, false),
                    mac1.expr(self.full_reg, Mmode::Default, self.mml),
                ]
                .into(),
            ),
        }
    }

    fn all_macs(assign: bool) -> Vec<(Option<Mac>, Option<Mac>)> {
        let opacc = [AccOp::Copy, AccOp::Add, AccOp::Sub, AccOp::None];

        let mut mac0: Vec<Option<Mac>> = opacc
            .iter()
            .map(|opa| Some(Mac::new(*opa, Acc::A0, assign)))
            .collect();
        mac0.insert(0, None);

        let mut mac1: Vec<Option<Mac>> = opacc
            .iter()
            .map(|opa| Some(Mac::new(*opa, Acc::A1, assign)))
            .collect();
        mac1.insert(0, None);

        let macs: Vec<(Option<Mac>, Option<Mac>)> = mac0
            .into_iter()
            .cartesian_product(mac1)
            .filter_map(|(mac0, mac1)| {
                if mac0.is_some() || mac1.is_some() {
                    Some((mac0, mac1))
                } else {
                    None
                }
            })
            .collect();

        macs
    }

    fn all_params() -> Vec<Self> {
        let mmod0 = vec![Mmode::Default, Mmode::W32, Mmode::FU, Mmode::IS];
        let mmod1 = vec![
            Mmode::Default,
            Mmode::S2RND,
            Mmode::T,
            Mmode::FU,
            Mmode::TFU,
            Mmode::IS,
            Mmode::ISS2,
            Mmode::IH,
            Mmode::IU,
        ];
        let mmode = vec![
            Mmode::Default,
            Mmode::S2RND,
            Mmode::FU,
            Mmode::IS,
            Mmode::ISS2,
            Mmode::IU,
        ];

        let mml = [false, true];
        let oper = Oper::all();

        let mut mmod0_params: Vec<TradMacParam> = Self::all_macs(false)
            .into_iter()
            .cartesian_product(oper.iter())
            .cartesian_product(mml.iter())
            .cartesian_product(mmod0)
            .filter_map(|((((mac0, mac1), oper), mml), mode)| {
                if (mac0.is_some() && mac0.unwrap().opa == AccOp::None)
                    || (mac1.is_some() && mac1.unwrap().opa == AccOp::None)
                {
                    None
                } else {
                    Some(Self::new(mac0, mac1, mode, *mml, *oper, false))
                }
            })
            .collect();

        let mut mmod1_params: Vec<TradMacParam> = Self::all_macs(true)
            .into_iter()
            .cartesian_product(oper.iter())
            .cartesian_product(mml.iter())
            .cartesian_product(mmod1)
            .map(|((((mac0, mac1), oper), mml), mode)| {
                Self::new(mac0, mac1, mode, *mml, *oper, false)
            })
            .collect();

        let mut mmode_params: Vec<TradMacParam> = Self::all_macs(true)
            .into_iter()
            .cartesian_product(oper.iter())
            .cartesian_product(mml.iter())
            .cartesian_product(mmode)
            .map(|((((mac0, mac1), oper), mml), mode)| {
                Self::new(mac0, mac1, mode, *mml, *oper, true)
            })
            .collect();

        mmod0_params.append(&mut mmod1_params);
        mmod0_params.append(&mut mmode_params);
        mmod0_params
    }
}

pub struct TradMacFactory();

impl TradMacFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, params: &TradMacParam) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name(&params.name())
            .display(params.display());

        instr = params.set_fields(instr).add_pcode(params.pcode());

        instr
    }
}

impl InstrFactory for TradMacFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        TradMacParam::all_params()
            .into_iter()
            .map(|params| Self::base_instr(ifam, &params))
            .collect()
    }
}
