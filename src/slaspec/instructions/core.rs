use std::collections::HashSet;

use crate::slaspec::instructions::pattern::Pattern;

use super::{
    pattern::{Field, FieldType, ProtoPattern},
    text::Text,
    util::mask_hex,
};

#[derive(Debug, Clone)]
pub struct InstrBuilder {
    pattern: Pattern,
    prefix: String,
    name: String,
    display: Text,
    action: Text,
    pcode: Text,
}

impl InstrBuilder {
    pub fn new(ifam: &InstrFamilyBuilder) -> Self {
        InstrBuilder {
            pattern: ifam.base_pattern.clone(),
            name: String::new(),
            prefix: ifam.prefix(),
            display: Text::new(),
            action: Text::new(),
            pcode: Text::new(),
        }
    }

    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }

    pub fn display(mut self, display: Text) -> Self {
        self.display = display;
        self
    }

    pub fn action(mut self, action: Text) -> Self {
        self.action = action;
        self
    }

    pub fn pcode(mut self, pcode: Text) -> Self {
        self.pcode = pcode;
        self
    }

    pub fn set_field_type(mut self, field_id: &str, ftype: FieldType) -> Self {
        self.pattern = self.pattern.clone().set_field_type(field_id, ftype);
        self
    }

    pub fn split_field(mut self, field_id: &str, split: ProtoPattern) -> Self {
        self.pattern = self.pattern.clone().split_field(field_id, split);
        self
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
        format!("\n[{}\n]", self.action.generate(&self))
    }

    fn build_pcode(&self) -> String {
        if self.pcode.is_empty() {
            return String::from("{}");
        }
        let nl = if self.action.is_empty() { "\n" } else { " " };
        format!("{}{{{}\n}}", nl, self.pcode.generate(&self))
    }

    pub fn build(&self) -> String {
        format!(
            "{} {}{}{}{}",
            self.build_name(),
            self.display.generate(&self),
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
    tokens: [HashSet<Field>; 4],
    variables: HashSet<Field>,
    pcodeops: Vec<String>,
}

impl InstrFamilyBuilder {
    pub fn new_16(name: &str, desc: &str, prefix: &str, base_pattern: ProtoPattern) -> Self {
        InstrFamilyBuilder {
            name: String::from(name),
            desc: String::from(desc),
            prefix: String::from(prefix),
            base_pattern: Pattern::from(base_pattern),
            instructions: Vec::new(),
            tokens: [
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ],
            variables: HashSet::new(),
            pcodeops: Vec::new(),
        }
    }

    pub fn new_32(name: &str, desc: &str, prefix: &str, base_pattern: [ProtoPattern; 2]) -> Self {
        InstrFamilyBuilder {
            name: String::from(name),
            desc: String::from(desc),
            prefix: String::from(prefix),
            base_pattern: Pattern::from(base_pattern),
            instructions: Vec::new(),
            tokens: [
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ],
            variables: HashSet::new(),
            pcodeops: Vec::new(),
        }
    }

    pub fn new_64(name: &str, desc: &str, prefix: &str, base_pattern: [ProtoPattern; 4]) -> Self {
        InstrFamilyBuilder {
            name: String::from(name),
            desc: String::from(desc),
            prefix: String::from(prefix),
            base_pattern: Pattern::from(base_pattern),
            instructions: Vec::new(),
            tokens: [
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ],
            variables: HashSet::new(),
            pcodeops: Vec::new(),
        }
    }

    pub fn add_pcodeop(&mut self, pcodeop: &str) {
        self.pcodeops.push(String::from(pcodeop));
    }

    pub fn add_instrs<Factory: InstrFactory>(&mut self, factory: &Factory) {
        self.instructions.append(&mut factory.build_instrs(&self));
    }

    pub fn init_tokens_and_vars(&mut self) {
        for instr in &self.instructions {
            for (wi, word) in instr.pattern().fields().iter().enumerate() {
                for field in word {
                    if field.is_var() {
                        self.variables.insert(field.clone());
                    }
                    self.tokens[wi].insert(field.clone());
                }
            }
        }
    }

    fn build_tokens(&self) -> String {
        let mut tokens_str = String::new();

        for i in 0..4 {
            if self.tokens[i].is_empty() {
                continue;
            }
            let mut tokens: Vec<Field> = self.tokens[i].clone().into_iter().collect();
            tokens.sort();

            tokens_str += &format!("define token {}Instr{} (16)\n", self.prefix, (i + 1) * 16);
            for tok in tokens {
                tokens_str += &format!(
                    "\t{} = ({}, {})\n",
                    tok.token_name(&self),
                    tok.start(),
                    tok.end()
                );

                if tok.is_signed() {
                    tokens_str += " signed";
                }
            }
            tokens_str += ";\n"
        }

        tokens_str
    }

    fn build_instructions(&self) -> String {
        let mut instr_str = String::new();

        for instr in &self.instructions {
            instr_str += &format!("{}{}\n", self.name, instr.build());
        }

        instr_str
    }

    pub fn build(&self) -> String {
        format!("{}{}", self.build_tokens(), self.build_instructions())
    }
}

pub trait Prefixed {
    fn prefix(&self) -> String;
}

impl Prefixed for &InstrBuilder {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
}

impl Prefixed for &InstrFamilyBuilder {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
}

pub trait InstrFactory {
    fn build_instrs(&self, ifam: &InstrFamilyBuilder) -> Vec<InstrBuilder>;
}
