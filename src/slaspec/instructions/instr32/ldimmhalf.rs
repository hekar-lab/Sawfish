use itertools::Itertools;

use crate::slaspec::instructions::core::{InstrBuilder, InstrFactory, InstrFamilyBuilder};
use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;
use crate::slaspec::instructions::pattern::{FieldType, ProtoField, ProtoPattern, RegisterSet};

pub fn instr_fam() -> InstrFamilyBuilder {
    let mut ifam = InstrFamilyBuilder::new_32(
        "LdImmHalf",
        "Load Immediate Half Word",
        "lih",
        [
            ProtoPattern::new(vec![
                ProtoField::new("sig", FieldType::Mask(0xe1), 8),
                ProtoField::new("z", FieldType::Blank, 1),
                ProtoField::new("h", FieldType::Blank, 1),
                ProtoField::new("s", FieldType::Blank, 1),
                ProtoField::new("grp", FieldType::Blank, 2),
                ProtoField::new("reg", FieldType::Blank, 3),
            ]),
            ProtoPattern::new(vec![ProtoField::new("hword", FieldType::SImmVal, 16)]),
        ],
    );

    ifam.add_instrs(&LdImmHalfFactory());

    ifam
}

#[derive(Debug, Clone, Copy)]
enum Dst {
    Dreg,
    Preg,
    Ireg,
    Mreg,
    Breg,
    Lreg,
}

impl Dst {
    fn full_rset(&self) -> bool {
        match self {
            Dst::Dreg | Dst::Preg => true,
            _ => false,
        }
    }

    fn display(&self, hzs: HZS) -> String {
        if hzs.full_reg() {
            format!(
                "{} = {{hword}} ({})",
                if self.full_rset() { "{reg}" } else { "{regL}" },
                hzs.display_ext()
            )
        } else {
            format!(
                "{} = {{hword}}",
                if self.full_rset() { "{reg}" } else { "{regL}" }
            )
        }
    }

    fn grp_mask(&self) -> u16 {
        match self {
            Dst::Dreg => 0x0,
            Dst::Preg => 0x1,
            Dst::Ireg | Dst::Mreg => 0x2,
            Dst::Breg | Dst::Lreg => 0x3,
        }
    }

    fn regset(&self, hzs: HZS) -> RegisterSet {
        match hzs {
            HZS::S | HZS::Z => match self {
                Dst::Dreg => RegisterSet::DReg,
                Dst::Preg => RegisterSet::PReg,
                Dst::Ireg => RegisterSet::IReg,
                Dst::Mreg => RegisterSet::MReg,
                Dst::Breg => RegisterSet::BReg,
                Dst::Lreg => RegisterSet::LReg,
            },
            HZS::L => match self {
                Dst::Dreg => RegisterSet::DRegL,
                Dst::Preg => RegisterSet::PRegL,
                Dst::Ireg => RegisterSet::IRegL,
                Dst::Mreg => RegisterSet::MRegL,
                Dst::Breg => RegisterSet::BRegL,
                Dst::Lreg => RegisterSet::LRegL,
            },
            HZS::H => match self {
                Dst::Dreg => RegisterSet::DRegH,
                Dst::Preg => RegisterSet::PRegH,
                Dst::Ireg => RegisterSet::IRegH,
                Dst::Mreg => RegisterSet::MRegH,
                Dst::Breg => RegisterSet::BRegH,
                Dst::Lreg => RegisterSet::LRegH,
            },
        }
    }

    fn regh_msk(&self) -> u16 {
        match self {
            Dst::Ireg | Dst::Breg => 0x0,
            Dst::Mreg | Dst::Lreg => 0x1,
            Dst::Dreg | Dst::Preg => panic!("Full register sets do not have regH"),
        }
    }

    fn set_fields(&self, instr: InstrBuilder, hzs: HZS) -> InstrBuilder {
        if self.full_rset() {
            instr
                .set_field_type("grp", FieldType::Mask(self.grp_mask()))
                .set_field_type("reg", FieldType::Variable(self.regset(hzs)))
        } else {
            instr
                .set_field_type("grp", FieldType::Mask(self.grp_mask()))
                .split_field(
                    "reg",
                    ProtoPattern::new(vec![
                        ProtoField::new("regH", FieldType::Mask(self.regh_msk()), 1),
                        ProtoField::new("regL", FieldType::Variable(self.regset(hzs)), 2),
                    ]),
                )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HZS {
    L,
    S,
    Z,
    H,
}

impl HZS {
    fn full_reg(&self) -> bool {
        match self {
            HZS::S | HZS::Z => true,
            HZS::L | HZS::H => false,
        }
    }

    fn display_ext(&self) -> &'static str {
        match self {
            Self::S => "X",
            Self::Z => "Z",
            _ => "",
        }
    }

    fn set_fields(&self, instr: InstrBuilder) -> InstrBuilder {
        instr
            .set_field_type("z", FieldType::Mask((*self == HZS::Z) as u16))
            .set_field_type("h", FieldType::Mask((*self == HZS::H) as u16))
            .set_field_type("s", FieldType::Mask((*self == HZS::S) as u16))
    }

    fn expr(&self, full_rset: bool) -> Expr {
        let src = match self {
            Self::S => e_sext(b_size(e_rfield("hword"), 2)),
            Self::Z => e_zext(b_size(e_rfield("hword"), 2)),
            _ => e_rfield("hword"),
        };
        e_copy(e_rfield(if full_rset { "reg" } else { "regL" }), src)
    }
}

struct LdImmHalfFactory();

impl LdImmHalfFactory {
    fn base_instr(ifam: &InstrFamilyBuilder, hzs: HZS, dst: Dst) -> InstrBuilder {
        dst.set_fields(hzs.set_fields(InstrBuilder::new(ifam)), hzs)
            .name(if hzs.full_reg() {
                "LdImmToDreg"
            } else {
                "LdImmToDregHL"
            })
            .display(dst.display(hzs))
            .add_pcode(hzs.expr(dst.full_rset()))
    }
}

impl InstrFactory for LdImmHalfFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder> {
        [HZS::L, HZS::S, HZS::Z, HZS::H]
            .into_iter()
            .cartesian_product([
                Dst::Dreg,
                Dst::Preg,
                Dst::Ireg,
                Dst::Mreg,
                Dst::Breg,
                Dst::Lreg,
            ])
            .map(|(hzs, dst)| Self::base_instr(ifam, hzs, dst))
            .collect()
    }
}
