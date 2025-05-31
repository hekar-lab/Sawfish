use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::instr32::common32::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

#[derive(Debug, Clone, Copy)]
struct Mult32Params {
    mode: Mmode,
    pair: bool,
    mixed: bool,
    no_sat: bool,
}

impl Mult32Params {
    fn new(mode: Mmode, pair: bool, mixed: bool, no_sat: bool) -> Self {
        Mult32Params {
            mode,
            pair,
            mixed,
            no_sat,
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

pub struct Mult32Factory();

impl Mult32Factory {
    fn display(params: Mult32Params) -> String {
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

        format!("{{dst}} = {{src0}} * {{src1}}{opt_str}")
    }

    fn base_instr(ifam: &InstrFamilyBuilder, params: Mult32Params) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("Mult32")
            .display(Self::display(params))
            .set_field_type("mmod", FieldType::Mask(params.mode_mask() as u16))
            .set_field_type("mm", FieldType::Mask(0x1))
            .set_field_type("p", FieldType::Mask(params.pair as u16))
            .set_field_type("w1", FieldType::Mask(0x0))
            .set_field_type("op1", FieldType::Mask(0x1))
            .set_field_type("w0", FieldType::Mask(0x1))
            .set_field_type("op0", FieldType::Mask(0x0))
            .set_field_type(
                "dst",
                FieldType::Variable(if params.pair {
                    RegisterSet::DRegPair
                } else {
                    RegisterSet::DReg
                }),
            )
            .set_field_type("src1", FieldType::Variable(RegisterSet::DReg))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(mult_expr(
                "result_mult32",
                "src0",
                "src1",
                params.mode,
                params.mixed,
                9,
            ))
            .add_pcode(extract_expr(
                "dst",
                b_var("result_mult32"),
                params.pair,
                params.mode,
                params.no_sat,
                9,
                "Mult32",
            ))
    }
}

impl InstrFactory for Mult32Factory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        let mut mult_params: Vec<Mult32Params> = Mult32Params::m32mmod2()
            .into_iter()
            .map(|(mode, mixed, no_sat)| Mult32Params::new(mode, false, mixed, no_sat))
            .collect();

        let mut multp_params: Vec<Mult32Params> = Mult32Params::m32mmod()
            .into_iter()
            .map(|(mode, mixed, no_sat)| Mult32Params::new(mode, true, mixed, no_sat))
            .collect();

        mult_params.append(&mut multp_params);

        mult_params
            .into_iter()
            .map(|params| Self::base_instr(ifam, params))
            .collect()
    }
}
