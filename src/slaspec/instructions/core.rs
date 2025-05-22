use std::collections::HashSet;

use crate::slaspec::instructions::{format::display_format, pattern::Pattern};

use super::{
    expr::{Code, Expr},
    pattern::{Field, FieldType, ProtoPattern},
    util::mask_hex,
};

#[derive(Debug, Clone)]
pub struct InstrBuilder {
    pattern: Pattern,
    prefix: String,
    name: String,
    display: String,
    actions: Code,
    pcodes: Code,
}

impl InstrBuilder {
    pub fn new(ifam: &InstrFamilyBuilder) -> Self {
        InstrBuilder {
            pattern: ifam.base_pattern.clone(),
            name: String::new(),
            prefix: ifam.prefix(),
            display: String::new(),
            actions: Code::new(),
            pcodes: Code::new(),
        }
    }

    pub fn set_pattern(mut self, pattern: Pattern) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn display(mut self, display: String) -> Self {
        self.display = display;
        self
    }

    pub fn get_display(&self) -> String {
        self.display.clone()
    }

    pub fn add_action(mut self, action: Expr) -> Self {
        self.actions.add_expr(action);
        self
    }

    pub fn get_actions(&self) -> Code {
        self.actions.clone()
    }

    pub fn set_actions(mut self, actions: Code) -> Self {
        self.actions = actions;
        self
    }

    pub fn add_pcode(mut self, pcode: Expr) -> Self {
        self.pcodes.add_expr(pcode);
        self
    }

    pub fn get_pcodes(&self) -> Code {
        self.pcodes.clone()
    }

    pub fn set_pcodes(mut self, pcodes: Code) -> Self {
        self.pcodes = pcodes;
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

    fn build_pattern(&self, alt: bool, alt_display: &str) -> String {
        let mut pattern_str = format!(
            "\n\tis {}",
            if alt {
                format!("{alt_display} & ")
            } else {
                "".to_string()
            }
        );

        let words = self.pattern.fields();
        for word in words {
            for field in word {
                if field.is_blank() {
                    continue;
                }

                pattern_str += &field.token_name(&self.prefix);
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
        let mut actions = String::new();

        if self.actions.is_empty() {
            return actions;
        }

        actions += &self.actions.build(&self.pattern, &self.prefix);

        format!("\n[{}\n]", actions)
    }

    fn build_pcode(&self) -> String {
        if self.pcodes.is_empty() {
            return String::from("{}");
        }

        let mut pcodes = String::new();
        let nl = if self.actions.is_empty() { "\n" } else { " " };

        pcodes += &self.pcodes.build(&self.pattern, &self.prefix);

        format!("{}{{{}\n}}", nl, pcodes)
    }

    pub fn build(&self, alt_display: String) -> (String, bool) {
        let (display, vars) = display_format(&self.display, &self.pattern, &self.prefix);
        let empty_display = display.is_empty();
        let no_vars = vars == 0;
        let alt = no_vars && !empty_display;
        (
            format!(
                "{} {}{}{}{}",
                self.build_name(),
                if alt { &alt_display } else { &display },
                self.build_pattern(alt, &alt_display),
                self.build_action(),
                self.build_pcode()
            ),
            alt,
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

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn instrs(&self) -> &Vec<InstrBuilder> {
        &self.instructions
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
                    if field.is_blank() {
                        continue;
                    }
                    if field.is_var() {
                        self.variables.insert(field.clone());
                    }
                    self.tokens[wi].insert(field.clone());
                }
            }
        }
    }

    fn build_desc(&self) -> String {
        let mut desc_str = String::new();

        desc_str += &format!("## {} ({})\n", self.desc, self.name);
        desc_str += "##\n";

        let pattern_sep = "## +---+---+---+---|---+---+---+---|---+---+---+---|---+---+---+---+\n";

        for word in self.base_pattern.fields() {
            if word.is_empty() {
                continue;
            }
            desc_str += pattern_sep;
            let mut word_str = "## ".to_string();

            for field in word {
                if field.id().starts_with("sig") || field.id().starts_with("x") {
                    if let FieldType::Mask(mask) = field.ftype() {
                        let bin_str = format!("{mask:0len$b}", len = field.len());
                        for bit in bin_str.chars() {
                            word_str += &format!("| {bit} ");
                        }
                    }
                } else {
                    word_str += &format!("|{:.^len$}", field.id(), len = (4 * field.len() - 1));
                }
            }

            word_str += "|\n";
            desc_str += &word_str;
        }

        desc_str += pattern_sep;
        desc_str
    }

    fn build_tokens(&self) -> String {
        let mut tokens_str = String::new();

        for i in 0..4 {
            if self.tokens[i].is_empty() {
                continue;
            }
            let mut tokens: Vec<Field> = self.tokens[i].clone().into_iter().collect();
            tokens.sort();
            tokens.reverse();

            tokens_str += &format!("define token {}Instr{} (16)\n", self.prefix, (i + 1) * 16);
            for tok in tokens {
                tokens_str += &format!(
                    "\t{:16} = ({:2},{:2}) {}\n",
                    tok.token_name(&self.prefix),
                    tok.start(),
                    tok.end(),
                    if tok.is_signed() { "signed" } else { "" }
                );
            }
            tokens_str += ";\n\n"
        }

        tokens_str
    }

    fn build_variables(&self) -> String {
        let mut var_str = String::new();
        let mut variables: Vec<Field> = self.variables.clone().into_iter().collect();
        variables.sort_by(|a, b| a.ftype().cmp(&b.ftype()));

        for var in variables {
            if let FieldType::Variable(regset) = var.ftype() {
                let regs = regset.regs();

                var_str += &format!(
                    "attach {} {} [{}];\n",
                    regset.attach_type(),
                    var.token_name(&self.prefix),
                    if regs.len() <= 8 {
                        regs.join(" ")
                    } else {
                        format!(
                            "\n{}\n",
                            regs.chunks(8)
                                .map(|chunk| format!("\t{}", chunk.join(" ")))
                                .collect::<Vec<String>>()
                                .join("\n")
                        )
                    }
                );
            }
        }

        var_str
    }

    fn build_pcodeops(&self) -> String {
        let mut pcodeops_str = String::new();

        for op in &self.pcodeops {
            pcodeops_str += &format!("define pcodeop {};\n", op);
        }

        pcodeops_str
    }

    fn build_instructions(&self) -> String {
        let mut instr_str = String::new();
        let mut instr_count = 0;

        for instr in &self.instructions {
            let literal_desc = format!("{}Desc{:02X}", self.name, instr_count);
            let (build, alt_disp) = instr.build(literal_desc.clone());
            if alt_disp {
                instr_str += &format!("{literal_desc}: \"{}\" is epsilon {{}}\n", instr.display);
            }
            instr_str += &format!("{}{}\n\n", self.name, build);
            instr_count += 1;
        }

        instr_str
    }

    fn build_final_instr(&self) -> String {
        format!(
            ":^{ifam} is {ifam} {{ build {ifam}; }}\n",
            ifam = self.name()
        )
    }

    pub fn build(&self) -> String {
        let mut build = String::new();
        build += &format!("{}\n", self.build_desc());
        build += &format!("### Tokens ###\n\n{}\n", self.build_tokens());
        if !self.variables.is_empty() {
            build += &format!("### Variables ###\n\n{}\n\n", self.build_variables());
        }
        if !self.pcodeops.is_empty() {
            build += &format!("### Operations ###\n\n{}\n\n", self.build_pcodeops());
        }
        build += &format!(
            "### Instructions ###\n\n{}\n\n{}",
            self.build_instructions(),
            self.build_final_instr()
        );

        build
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
