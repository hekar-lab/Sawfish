use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, RegisterSet};

enum Src {
    Reg(RegisterSet),
    Acc(String),
}

impl Src {
    fn is_reg(&self) -> bool {
        match self {
            Src::Reg(_) => true,
            _ => false,
        }
    }

    fn reg(&self) -> RegisterSet {
        match self {
            Self::Reg(reg) => *reg,
            _ => panic!("Not a register"),
        }
    }

    fn display(&self) -> String {
        match self {
            Src::Reg(_) => "{src0}".to_string(),
            Src::Acc(id) => id.clone(),
        }
    }

    fn offset(&self) -> i128 {
        match self {
            Src::Reg(_) => 1,
            Self::Acc(_) => 9,
        }
    }
}

pub struct SignBitsFactory();

impl SignBitsFactory {
    fn sign_instr(ifam: &InstrFamilyBuilder, sop: u16, src_reg: Src) -> InstrBuilder {
        let counted = match &src_reg {
            Src::Reg(_) => e_rfield("src0"),
            Src::Acc(id) => b_reg(&id),
        };
        let mut instr = InstrBuilder::new(ifam)
            .name("SignBits")
            .display(format!("{{dst}} = SIGNBITS {}", src_reg.display()))
            .set_field_type("sopc", FieldType::Mask(0x5 + (!src_reg.is_reg()) as u16))
            .set_field_type("sop", FieldType::Mask(sop))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DRegL))
            .add_pcode(cs_mline(vec![
                b_ifgoto(e_lts(counted.clone(), b_num(0)), b_label("sign_neg")),
                e_copy(
                    e_rfield("dst"),
                    e_sub(e_macp("lzcount", counted.clone()), b_num(src_reg.offset())),
                ),
                b_goto(b_label("sign_end")),
                b_label("sign_neg"),
                e_copy(
                    e_rfield("dst"),
                    e_sub(
                        e_macp("lzcount", e_bit_not(counted)),
                        b_num(src_reg.offset()),
                    ),
                ),
                b_label("sign_end"),
            ]));

        if src_reg.is_reg() {
            instr = instr.set_field_type("src0", FieldType::Variable(src_reg.reg()));
        }

        instr
    }

    fn ones_instr(ifam: &InstrFamilyBuilder) -> InstrBuilder {
        InstrBuilder::new(ifam)
            .name("Ones")
            .display("{dst} = ONES {src0}".to_string())
            .set_field_type("sopc", FieldType::Mask(0x6))
            .set_field_type("sop", FieldType::Mask(0x3))
            .set_field_type("hls", FieldType::Mask(0x0))
            .set_field_type("dst", FieldType::Variable(RegisterSet::DRegL))
            .set_field_type("src0", FieldType::Variable(RegisterSet::DReg))
            .add_pcode(e_copy(
                e_rfield("dst"),
                e_macp("popcount", e_rfield("src0")),
            ))
    }
}

impl InstrFactory for SignBitsFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        vec![
            Self::sign_instr(ifam, 0x0, Src::Reg(RegisterSet::DReg)),
            Self::sign_instr(ifam, 0x1, Src::Reg(RegisterSet::DRegL)),
            Self::sign_instr(ifam, 0x2, Src::Reg(RegisterSet::DRegH)),
            Self::sign_instr(ifam, 0x0, Src::Acc("A0".to_string())),
            Self::sign_instr(ifam, 0x1, Src::Acc("A1".to_string())),
            Self::ones_instr(ifam),
        ]
    }
}
