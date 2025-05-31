use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::instr32::common32::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy)]
struct Mac32Params {
    mode: Mmode,
    pair: bool,
    assign: bool,
    mixed: bool,
    no_sat: bool,
    accop: AccOp,
}

impl Mac32Params {
    fn new(mode: Mmode, pair: bool, assign: bool, mixed: bool, no_sat: bool, accop: AccOp) -> Self {
        Mac32Params {
            mode,
            pair,
            assign,
            mixed,
            no_sat,
            accop,
        }
    }

    fn mode_32(&self) -> u16 {
        match self.mode {
            Mmode::Default => 0x0,
            Mmode::T => 0x1,
            Mmode::IS => 0x2,
            Mmode::FU => 0x4,
            Mmode::TFU => 0x5,
            Mmode::IU => 0x6,
            _ => 0x0,
        }
    }

    fn mode_mask(&self) -> u16 {
        self.mode_32() + self.mixed as u16 * 8 + self.no_sat as u16
    }

    fn m32mmod() -> Vec<(Mmode, bool, bool)> {
        vec![
            (Mmode::Default, false, false),
            (Mmode::IS, false, false),
            (Mmode::IS, false, true),
            (Mmode::FU, false, false),
            (Mmode::IU, false, false),
            (Mmode::IU, false, true),
            (Mmode::Default, true, false),
            (Mmode::IS, true, false),
            (Mmode::IS, true, true),
        ]
    }

    fn m32mmod1() -> Vec<(Mmode, bool, bool)> {
        vec![
            (Mmode::T, false, false),
            (Mmode::IS, false, false),
            (Mmode::IS, false, true),
            (Mmode::TFU, false, false),
            (Mmode::IU, false, false),
            (Mmode::IU, false, true),
            (Mmode::T, true, false),
            (Mmode::IS, true, false),
            (Mmode::IS, true, true),
        ]
    }

    fn m32mmod2() -> Vec<(Mmode, bool, bool)> {
        vec![
            (Mmode::Default, false, false),
            (Mmode::T, false, false),
            (Mmode::IS, false, false),
            (Mmode::IS, false, true),
            (Mmode::FU, false, false),
            (Mmode::TFU, false, false),
            (Mmode::IU, false, false),
            (Mmode::IU, false, true),
            (Mmode::Default, true, false),
            (Mmode::T, true, false),
            (Mmode::IS, true, false),
            (Mmode::IS, true, true),
        ]
    }
}

pub struct Mac32Factory();

impl Mac32Factory {
    fn set_fields(mut instr: InstrBuilder, params: Mac32Params) -> InstrBuilder {
        instr = instr
            .set_field_type("mmod", FieldType::Mask(params.mode_mask()))
            .set_field_type("mm", FieldType::Mask(0x0))
            .set_field_type("p", FieldType::Mask(params.pair as u16))
            .set_field_type("w1", FieldType::Mask(0x0))
            .set_field_type("op1", FieldType::Mask(0x1))
            .set_field_type("w0", FieldType::Mask(params.assign as u16))
            .set_field_type("op0", FieldType::Mask(params.accop as u16));

        instr = if params.pair {
            instr.set_field_type("dst", FieldType::Variable(RegisterSet::DRegPair))
        } else {
            instr.set_field_type("dst", FieldType::Variable(RegisterSet::DReg))
        };

        if params.accop != AccOp::None {
            instr = instr
                .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
                .set_field_type("src0", FieldType::Variable(RegisterSet::DReg));
        }

        instr
    }

    fn display(params: Mac32Params) -> String {
        let mut opt_vec = vec![];

        if params.mixed {
            opt_vec.push("M".to_string());
        }

        if let Some(mode) = params.mode.to_str() {
            opt_vec.push(mode);
        }

        if params.no_sat {
            opt_vec.push("NS".to_string());
        }

        let opt_str = if !opt_vec.is_empty() {
            format!(" ({})", opt_vec.join(", "))
        } else {
            String::new()
        };

        let op_str = if params.accop != AccOp::None {
            format!("A1:0 {} {{src0}} * {{src1}}", params.accop.op_str())
        } else {
            "A1:0".to_string()
        };

        if params.assign && params.accop != AccOp::None {
            format!("{{dst}} = ({op_str}){opt_str}")
        } else {
            format!("{{dst}} = {op_str}{opt_str}")
        }
    }

    fn get_accs(var_id: &str) -> Expr {
        e_copy(
            e_local(var_id, 9),
            e_bit_or(
                b_grp(e_lshft(e_zext(b_reg("A1")), b_num(4))),
                e_zext(b_reg("A0.W")),
            ),
        )
    }

    fn set_accs(var_id: &str) -> Expr {
        cs_mline(
            vec![
                e_copy(b_reg("A1"), b_trunc(b_var(var_id), 4)),
                e_copy(b_reg("A0.W"), b_size(b_var(var_id), 4)),
            ]
            .into(),
        )
    }

    fn expr(params: Mac32Params) -> Expr {
        let acc_var_id = "A10_var";
        let res_var_id = "result_var";
        let mut code = vec![];

        code.push(Self::get_accs(acc_var_id));

        if params.accop != AccOp::None {
            code.push(mult_expr(
                res_var_id,
                "src0",
                "src1",
                params.mode,
                params.mixed,
                9,
            ));
            code.push(acc_expr(
                b_var(acc_var_id),
                params.accop,
                res_var_id,
                params.mode,
                params.no_sat,
                "Mac32",
            ));
            code.push(Self::set_accs(acc_var_id));
        }

        if params.assign {
            code.push(extract_expr(
                "dst",
                b_var(acc_var_id),
                params.pair,
                params.mode,
                params.no_sat,
                9,
                "Mac32",
            ));
        }

        cs_mline(code.into())
    }

    fn base_instr(ifam: &InstrFamilyBuilder, params: Mac32Params) -> InstrBuilder {
        let mut instr = InstrBuilder::new(ifam)
            .name(if params.assign && params.accop != AccOp::None {
                "MvAxToDreg"
            } else if params.assign {
                "Mac32WithMv"
            } else {
                "Mac32"
            })
            .display(Self::display(params))
            .add_pcode(Self::expr(params));

        instr = Self::set_fields(instr, params);

        instr
    }
}

impl InstrFactory for Mac32Factory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut mac_params: Vec<Mac32Params> = AccOp::all()
            .into_iter()
            .cartesian_product(Mac32Params::m32mmod())
            .filter_map(|(accop, (mode, mixed, no_sat))| {
                if accop == AccOp::None {
                    None
                } else {
                    Some(Mac32Params::new(mode, false, false, mixed, no_sat, accop))
                }
            })
            .collect();

        let mut macmv_params: Vec<Mac32Params> = AccOp::all()
            .into_iter()
            .cartesian_product(Mac32Params::m32mmod1())
            .filter_map(|(accop, (mode, mixed, no_sat))| {
                if accop == AccOp::None {
                    None
                } else {
                    Some(Mac32Params::new(mode, false, true, mixed, no_sat, accop))
                }
            })
            .collect();

        let mut mv_params: Vec<Mac32Params> = AccOp::all()
            .into_iter()
            .cartesian_product(Mac32Params::m32mmod2())
            .filter_map(|(accop, (mode, mixed, no_sat))| {
                if accop == AccOp::None {
                    Some(Mac32Params::new(mode, false, true, mixed, no_sat, accop))
                } else {
                    None
                }
            })
            .collect();

        let mut macmvp_params: Vec<Mac32Params> = AccOp::all()
            .into_iter()
            .cartesian_product(Mac32Params::m32mmod())
            .map(|(accop, (mode, mixed, no_sat))| {
                Mac32Params::new(mode, true, true, mixed, no_sat, accop)
            })
            .collect();

        mac_params.append(&mut macmv_params);
        mac_params.append(&mut mv_params);
        mac_params.append(&mut macmvp_params);

        mac_params
            .into_iter()
            .map(|params| Self::base_instr(ifam, params))
            .collect()
    }
}
