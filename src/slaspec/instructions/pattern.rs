use std::hash::Hash;

use super::util::capitalize;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum RegisterSet {
    DReg,
    DRegL,
    DRegH,
    DRegB,
    PReg,
    IReg,
    MReg,
    BReg,
    LReg,
    SyRg2,
    SyRg3,
}

impl RegisterSet {
    pub fn name(&self) -> String {
        match self {
            Self::DReg => String::from("DReg"),
            Self::DRegL => String::from("DRegL"),
            Self::DRegH => String::from("DRegH"),
            Self::DRegB => String::from("DRegB"),
            Self::PReg => String::from("PReg"),
            Self::IReg => String::from("IReg"),
            Self::MReg => String::from("MReg"),
            Self::BReg => String::from("BReg"),
            Self::LReg => String::from("LReg"),
            Self::SyRg2 => String::from("SyRg2"),
            Self::SyRg3 => String::from("SyRg3"),
        }
    }

    fn build_regs(id: &str, len: usize, suffix: Option<&str>) -> Vec<String> {
        let mut regs = Vec::new();

        for i in 0..len {
            regs.push(format!(
                "{}{}{}",
                id,
                i,
                if let Some(s) = suffix {
                    format!(".{}", s)
                } else {
                    "".to_string()
                }
            ));
        }

        regs
    }

    fn build_regs_from(regs: Vec<&str>) -> Vec<String> {
        regs.into_iter().map(|v| v.to_string()).collect()
    }

    pub fn regs(&self) -> Vec<String> {
        match self {
            Self::DReg => Self::build_regs("R", 8, None),
            Self::DRegL => Self::build_regs("R", 8, Some("L")),
            Self::DRegH => Self::build_regs("R", 8, Some("H")),
            Self::DRegB => Self::build_regs("R", 8, Some("B")),
            Self::PReg => {
                let mut regs = Self::build_regs("P", 6, None);
                regs.append(&mut Self::build_regs_from(vec!["SP", "FP"]));
                regs
            }
            Self::IReg => Self::build_regs("I", 4, None),
            Self::MReg => Self::build_regs("M", 4, None),
            Self::BReg => Self::build_regs("B", 4, None),
            Self::LReg => Self::build_regs("L", 4, None),
            Self::SyRg2 => Self::build_regs_from(vec![
                "LC0", "LT0", "LB0", "LC1", "LT1", "LB1", "CYCLES", "CYCLES2",
            ]),
            Self::SyRg3 => Self::build_regs_from(vec![
                "USP", "SEQSTAT", "SYSCFG", "RETI", "RETX", "RETN", "RETE", "EMUDAT",
            ]),
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialOrd, Ord)]
pub enum FieldType {
    #[default]
    Blank,
    Mask(u16),
    UImmVal,
    SImmVal,
    Variable(RegisterSet),
}

impl Hash for FieldType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Self::Variable(v) = self {
            v.hash(state);
        }
        core::mem::discriminant(self).hash(state);
    }
}

impl PartialEq for FieldType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Variable(l0), Self::Variable(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Ord)]
pub struct BitRange {
    start: usize,
    end: usize,
}

impl BitRange {
    fn new(start: usize, end: usize) -> Self {
        BitRange { start, end }
    }

    fn len(&self) -> usize {
        self.end.saturating_sub(self.start) + 1
    }
}

impl PartialOrd for BitRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.end.partial_cmp(&other.end) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.len().partial_cmp(&other.len())
    }
}

#[derive(Debug, Clone)]
pub struct ProtoField {
    id: String,
    ftype: FieldType,
    size: usize,
}

impl ProtoField {
    pub fn new(id: &str, ftype: FieldType, size: usize) -> Self {
        ProtoField {
            id: String::from(id),
            ftype,
            size,
        }
    }

    pub fn to_field(&self, start: usize) -> Field {
        Field {
            id: self.id.clone(),
            ftype: self.ftype.clone(),
            bit_range: BitRange::new(start, start + self.size - 1),
        }
    }

    pub fn to_field_end(&self, end: usize) -> Field {
        Field {
            id: self.id.clone(),
            ftype: self.ftype.clone(),
            bit_range: BitRange::new(end - (self.size - 1), end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProtoPattern {
    pub fields: Vec<ProtoField>,
}

impl ProtoPattern {
    pub fn new(fields: Vec<ProtoField>) -> Self {
        ProtoPattern { fields }
    }

    pub fn len(&self) -> usize {
        self.fields.iter().map(|f| f.size).sum()
    }

    pub fn pop(&mut self) -> Option<ProtoField> {
        self.fields.pop()
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Ord)]
pub struct Field {
    id: String,
    ftype: FieldType,
    bit_range: BitRange,
}

impl Field {
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn ftype(&self) -> FieldType {
        self.ftype.clone()
    }

    pub fn start(&self) -> usize {
        self.bit_range.start
    }

    pub fn end(&self) -> usize {
        self.bit_range.end
    }

    pub fn len(&self) -> usize {
        self.bit_range.len()
    }

    pub fn is_signed(&self) -> bool {
        self.ftype == FieldType::SImmVal
    }

    pub fn is_var(&self) -> bool {
        if let FieldType::Variable(_) = self.ftype {
            return true;
        }

        false
    }

    pub fn is_blank(&self) -> bool {
        self.ftype == FieldType::Blank
    }

    pub fn name(&self) -> String {
        let suffix = match &self.ftype {
            FieldType::Variable(regset) => regset.name(),
            FieldType::UImmVal => String::from("UImm"),
            FieldType::SImmVal => String::from("SImm"),
            _ => String::new(),
        };

        format!("{}{}", capitalize(&self.id), suffix)
    }

    pub fn token_name(&self, fam_prefix: &str) -> String {
        format!("{}{}", fam_prefix, self.name())
    }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.bit_range.partial_cmp(&other.bit_range) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.id.partial_cmp(&other.id) {
            Some(core::cmp::Ordering::Equal) => {}
            Some(o) => return Some(o.reverse()),
            None => return None,
        }
        match self.ftype.partial_cmp(&other.ftype) {
            Some(o) => Some(o.reverse()),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Pattern {
    fields: [Vec<Field>; 4],
}

impl Pattern {
    pub fn fields(&self) -> [Vec<Field>; 4] {
        self.fields.clone()
    }

    fn get_field_index(&self, field_id: &str) -> Option<(usize, usize)> {
        for (wi, word) in self.fields.iter().enumerate() {
            for (fi, field) in word.iter().enumerate() {
                if field.id == field_id {
                    return Some((wi, fi));
                }
            }
        }

        None
    }

    pub fn get_field(&self, field_id: &str) -> Option<Field> {
        if let Some((wi, fi)) = self.get_field_index(field_id) {
            return Some(self.fields[wi][fi].to_owned());
        }

        None
    }

    pub fn set_field_type(mut self, field_id: &str, ftype: FieldType) -> Self {
        if let Some((wi, fi)) = self.get_field_index(field_id) {
            self.fields[wi][fi].ftype = ftype;
        }

        self
    }

    pub fn split_field(mut self, field_id: &str, split: ProtoPattern) -> Self {
        if let Some((wi, mut fi)) = self.get_field_index(field_id) {
            let field = &self.fields[wi][fi];

            if field.len() != split.len() {
                println!("WARNING: Lengths are not matching for field splitting");
                return self;
            }

            let mut end: isize = self.fields[wi].remove(fi).bit_range.end as isize;

            for proto in split.fields.iter() {
                let f = proto.to_field_end(end as usize);
                end = f.bit_range.start as isize - 1;

                self.fields[wi].insert(fi, f);
                fi += 1;
            }
        }

        self
    }
}

impl From<ProtoPattern> for Pattern {
    fn from(value: ProtoPattern) -> Self {
        let mut fields = Vec::new();
        let mut start: usize = 0;

        for pfield in value.fields.iter().rev() {
            fields.push(pfield.to_field(start));
            start += pfield.size;
        }

        fields.reverse();

        Pattern {
            fields: [fields, Vec::new(), Vec::new(), Vec::new()],
        }
    }
}

impl From<[ProtoPattern; 2]> for Pattern {
    fn from(value: [ProtoPattern; 2]) -> Self {
        let mut fields = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut word_index: usize = 0;

        for word in value {
            let mut start: usize = 0;
            for pfield in word.fields.iter().rev() {
                fields[word_index].push(pfield.to_field(start));
                start += pfield.size;
            }
            fields[word_index].reverse();
            word_index += 1;
        }

        Pattern { fields }
    }
}

impl From<[ProtoPattern; 4]> for Pattern {
    fn from(value: [ProtoPattern; 4]) -> Self {
        let mut fields = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut word_index: usize = 0;

        for word in value {
            let mut start: usize = word.len() - 1;
            for pfield in word.fields.iter().rev() {
                fields[word_index].push(pfield.to_field(start));
                start += pfield.size;
            }
            fields[word_index].reverse();
            word_index += 1;
        }

        Pattern { fields }
    }
}
