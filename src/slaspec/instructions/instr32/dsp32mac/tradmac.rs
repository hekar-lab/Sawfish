use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::instr32::common32::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

#[derive(Debug, Clone, Copy)]
pub struct Mac {
    accop: AccOp,
    acc: Acc,
    operands: Oper,
    assign: bool,
}

impl Mac {
    pub fn new(accop: AccOp, acc: Acc, operands: Oper, assign: bool) -> Self {
        Mac {
            accop,
            acc,
            operands,
            assign,
        }
    }

    pub fn no_accop(&self) -> bool {
        self.accop == AccOp::None
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

    pub fn name(&self, mv_para: bool, full_reg: bool) -> String {
        if self.assign {
            if self.no_accop() {
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

    pub fn mv_acc(&self) -> bool {
        self.no_accop() && self.assign
    }

    pub fn display(&self, mode: Mmode, mml: bool) -> String {
        let op_str = if self.no_accop() {
            self.acc.to_str()
        } else {
            format!(
                "{acc} {} {{src0{acc}}} * {{src1{acc}}}",
                self.accop.op_str(),
                acc = self.acc.to_str(),
            )
        };
        let mut opt_vec = vec![];

        if mml {
            opt_vec.push("M".to_string());
        }

        if let Some(mode) = mode.to_str() {
            opt_vec.push(mode);
        }

        let opt_str = if opt_vec.is_empty() {
            String::new()
        } else {
            format!(" ({})", opt_vec.join(", "))
        };

        if self.assign {
            if self.no_accop() {
                format!("{{dst{}}} = {op_str}{opt_str}", self.acc.to_str())
            } else {
                format!("{{dst{}}} = ({op_str}){opt_str}", self.acc.to_str())
            }
        } else {
            format!("{op_str}{opt_str}")
        }
    }

    pub fn set_fields(&self, mut instr: InstrBuilder) -> InstrBuilder {
        if !self.no_accop() {
            instr = instr
                .set_field_type(
                    &format!("h0{}", self.acc as u32),
                    FieldType::Mask(self.operands.lhs as u16),
                )
                .set_field_type(
                    &format!("h1{}", self.acc as u32),
                    FieldType::Mask(self.operands.rhs as u16),
                )
                .set_field_type(
                    &format!("src0{}", self.acc.to_str()),
                    FieldType::Variable(self.operands.lhs.regset()),
                )
                .set_field_type(
                    &format!("src1{}", self.acc.to_str()),
                    FieldType::Variable(self.operands.rhs.regset()),
                );
        }

        instr
            .set_field_type(
                &format!("op{}", self.acc as u32),
                FieldType::Mask(self.accop as u16),
            )
            .set_field_type(
                &format!("w{}", self.acc as u32),
                FieldType::Mask(self.assign as u16),
            )
    }

    pub fn expr(&self, full_reg: bool, mode: Mmode, mml: bool) -> Expr {
        let res_id = &format!("results_{}", self.acc.to_str());
        let src0_id = &format!("src0{}", self.acc.to_str());
        let src1_id = &format!("src1{}", self.acc.to_str());
        let dst_id = &format!("dst{}", self.acc.to_str());
        let mut code = vec![];

        if !self.no_accop() {
            code.push(mult_expr(res_id, src0_id, src1_id, mode, mml, 5));
            code.push(acc_expr(
                b_reg(&self.acc.to_str()),
                self.accop,
                res_id,
                mode,
                false,
                &format!("{}accOp", self.acc.to_str()),
            ));
        }

        if self.assign {
            let src = if !self.no_accop() {
                b_var(res_id)
            } else {
                b_reg(&self.acc.to_str())
            };
            code.push(extract_expr(
                dst_id,
                src,
                full_reg,
                mode,
                false,
                5,
                &format!("{}extrOp", self.acc.to_str()),
            ));
        }

        cs_mline(code)
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
    full_reg: bool,
}

impl TradMacParam {
    fn new(mac0: Option<Mac>, mac1: Option<Mac>, mode: Mmode, mml: bool, full_reg: bool) -> Self {
        Self {
            mac0,
            mac1,
            mode,
            mml,
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

    fn set_fields(&self, mut instr: InstrBuilder) -> InstrBuilder {
        instr = instr
            .divide_field(
                "src0",
                ProtoPattern::new(vec![
                    ProtoField::new("src0A0", FieldType::Blank, 3),
                    ProtoField::new("src0A1", FieldType::Blank, 3),
                ]),
            )
            .divide_field(
                "src1",
                ProtoPattern::new(vec![
                    ProtoField::new("src1A0", FieldType::Blank, 3),
                    ProtoField::new("src1A1", FieldType::Blank, 3),
                ]),
            );
        instr = match self.mac_enum() {
            MacEnum::Mac0(mac) => mac
                .set_fields(instr)
                .set_field_type("op1", FieldType::Mask(0x3))
                .set_field_type("w1", FieldType::Mask(0x0)),
            MacEnum::Mac1(mac) => mac
                .set_fields(instr)
                .set_field_type("mm", FieldType::Mask(self.mml as u16))
                .set_field_type("op0", FieldType::Mask(0x3))
                .set_field_type("w0", FieldType::Mask(0x0)),
            MacEnum::Mac10(mac0, mac1) => mac1
                .set_fields(mac0.set_fields(instr))
                .set_field_type("mm", FieldType::Mask(self.mml as u16)),
        }
        .set_field_type("mmod", FieldType::Mask(self.mode as u16))
        .set_field_type("p", FieldType::Mask(self.full_reg as u16));

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
            MacEnum::Mac10(mac0, mac1) => cs_mline(vec![
                mac0.expr(self.full_reg, self.mode, false),
                mac1.expr(self.full_reg, Mmode::Default, self.mml),
            ]),
        }
    }

    fn all_macs(assign: bool) -> Vec<(Option<Mac>, Option<Mac>)> {
        let opacc = AccOp::all();
        let oper = Oper::all();

        let mut mac0: Vec<Option<Mac>> = opacc
            .iter()
            .cartesian_product(oper.iter())
            .filter_map(|(accop, operands)| {
                if *accop == AccOp::None && (operands.lhs != Half::L || operands.rhs != Half::L) {
                    None
                } else {
                    Some(Some(Mac::new(*accop, Acc::A0, *operands, assign)))
                }
            })
            .collect();
        mac0.insert(0, None);

        let mut mac1: Vec<Option<Mac>> = opacc
            .iter()
            .cartesian_product(oper.iter())
            .filter_map(|(accop, operands)| {
                if *accop == AccOp::None && (operands.lhs != Half::L || operands.rhs != Half::L) {
                    None
                } else {
                    Some(Some(Mac::new(*accop, Acc::A1, *operands, assign)))
                }
            })
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
        let mmod0 = Mmode::mmod0();
        let mmod1 = Mmode::mmod1();
        let mmode = Mmode::mmode();

        let mml = [false, true];

        let mut mmod0_params: Vec<TradMacParam> = Self::all_macs(false)
            .into_iter()
            .cartesian_product(mml.iter())
            .cartesian_product(mmod0)
            .filter_map(|(((mac0, mac1), mml), mode)| {
                if (mac0.is_some() && mac0.unwrap().no_accop())
                    || (mac1.is_some() && mac1.unwrap().no_accop())
                    || (mac1.is_none()) && *mml
                {
                    None
                } else {
                    Some(Self::new(mac0, mac1, mode, *mml, false))
                }
            })
            .collect();

        let mut mmod1_params: Vec<TradMacParam> = Self::all_macs(true)
            .into_iter()
            .cartesian_product(mml.iter())
            .cartesian_product(mmod1)
            .filter_map(|(((mac0, mac1), mml), mode)| {
                if (mac1.is_none()) && *mml {
                    None
                } else {
                    Some(Self::new(mac0, mac1, mode, *mml, false))
                }
            })
            .collect();

        let mut mmode_params: Vec<TradMacParam> = Self::all_macs(true)
            .into_iter()
            .cartesian_product(mml.iter())
            .cartesian_product(mmode)
            .filter_map(|(((mac0, mac1), mml), mode)| {
                if (mac1.is_none()) && *mml {
                    None
                } else {
                    Some(Self::new(mac0, mac1, mode, *mml, true))
                }
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
