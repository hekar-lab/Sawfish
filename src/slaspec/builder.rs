use super::instructions::core::InstrFamilyBuilder;

use super::instructions::instr16::{nop16, progctrl};

pub struct SLASpecBuilder {
    ifams: Vec<InstrFamilyBuilder>,
}

impl SLASpecBuilder {
    pub fn new() -> Self {
        let mut ifams = vec![nop16::instr_fam(), progctrl::instr_fam()];

        for ifam in ifams.iter_mut() {
            ifam.init_tokens_and_vars();
        }

        SLASpecBuilder { ifams }
    }

    pub fn build(&self) -> String {
        let mut slaspec = String::new();

        for ifam in &self.ifams {
            slaspec += &ifam.build();
        }

        slaspec
    }
}
