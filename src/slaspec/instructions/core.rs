use std::collections::HashSet;

use crate::slaspec::instructions::pattern::Pattern;

use super::{
    pattern::{FieldType, ProtoPattern},
    util::mask_hex,
};

#[derive(Debug, Clone)]
pub struct InstrBuilder {
    pattern: Pattern,
    prefix: String,
    name: String,
    pub display: String,
    action: String,
    pcode: String,
}

impl InstrBuilder {
    pub fn new(name: &str, ifam: &InstrFamilyBuilder) -> Self {
        InstrBuilder {
            pattern: ifam.base_pattern.clone(),
            name: String::from(name),
            prefix: ifam.prefix(),
            display: String::new(),
            action: String::new(),
            pcode: String::new(),
        }
    }

    pub fn prefix(&self) -> String {
        self.prefix.clone()
    }

    pub fn set_field_type(&mut self, field_id: &str, ftype: FieldType) {
        self.pattern = self.pattern.clone().set_field_type(field_id, ftype);
    }

    pub fn split_field(&mut self, field_id: &str, split: ProtoPattern) {
        self.pattern = self.pattern.clone().split_field(field_id, split);
    }

    fn build_name(&self) -> String {
        format!(":^\"{}\"", self.name)
    }

    fn build_pattern(&self) -> String {
        let mut pattern_str = String::from("\n\tis ");

        let words = self.pattern.fields();
        for word in words {
            for field in word {
                pattern_str += &field.token_name(&self);
                if let FieldType::Mask(val) = field.ftype() {
                    pattern_str += "=";
                    pattern_str += &mask_hex(val, field.len());
                }
                pattern_str += " & "
            }
            pattern_str.truncate(pattern_str.len() - 2);
            pattern_str += "; ";
        }
        pattern_str.truncate(pattern_str.len() - 2);

        pattern_str
    }

    fn build_action(&self) -> String {
        if self.action.is_empty() {
            return String::new();
        }
        format!("\n[{}\n]", self.action)
    }

    fn build_pcode(&self) -> String {
        format!("\n{{{}\n}}", self.pcode)
    }

    pub fn build(&self) -> String {
        format!(
            "{} {}{}{}{}",
            self.build_name(),
            self.display,
            self.build_pattern(),
            self.build_action(),
            self.build_pcode()
        )
    }
}

pub struct InstrFamilyBuilder {
    name: String,
    desc: String,
    prefix: String,
    base_pattern: Pattern,
    instructions: Vec<InstrBuilder>,
    tokens: HashSet<InstrBuilder>,
    variables: HashSet<InstrBuilder>,
}

impl InstrFamilyBuilder {
    pub fn new_16(name: &str, desc: &str, prefix: &str, base_pattern: ProtoPattern) -> Self {
        InstrFamilyBuilder {
            name: String::from(name),
            desc: String::from(desc),
            prefix: String::from(prefix),
            base_pattern: Pattern::from(base_pattern),
            instructions: Vec::new(),
            tokens: HashSet::new(),
            variables: HashSet::new(),
        }
    }

    pub fn new_32(name: &str, desc: &str, prefix: &str, base_pattern: [ProtoPattern; 2]) -> Self {
        InstrFamilyBuilder {
            name: String::from(name),
            desc: String::from(desc),
            prefix: String::from(prefix),
            base_pattern: Pattern::from(base_pattern),
            instructions: Vec::new(),
            tokens: HashSet::new(),
            variables: HashSet::new(),
        }
    }

    pub fn new_64(name: &str, desc: &str, prefix: &str, base_pattern: [ProtoPattern; 4]) -> Self {
        InstrFamilyBuilder {
            name: String::from(name),
            desc: String::from(desc),
            prefix: String::from(prefix),
            base_pattern: Pattern::from(base_pattern),
            instructions: Vec::new(),
            tokens: HashSet::new(),
            variables: HashSet::new(),
        }
    }

    pub fn prefix(&self) -> String {
        self.prefix.clone()
    }

    pub fn add_instr(&mut self, instr: InstrBuilder) {
        self.instructions.push(instr);
    }

    fn build_instructions(&self) -> String {
        let mut ifam_str = String::new();

        for instr in &self.instructions {
            ifam_str += &self.name;
            ifam_str += ":";
            ifam_str += &instr.build();
            ifam_str += "\n";
        }

        ifam_str
    }

    pub fn build(&self) -> String {
        self.build_instructions()
    }
}
