use std::hash::Hash;

use super::util::capitalize;

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum RegisterSet {
    DReg,
    DRegL,
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
            Self::PReg => String::from("PReg"),
            Self::IReg => String::from("IReg"),
            Self::MReg => String::from("MReg"),
            Self::BReg => String::from("BReg"),
            Self::LReg => String::from("LReg"),
            Self::SyRg2 => String::from("SyRg2"),
            Self::SyRg3 => String::from("SyRg3"),
        }
    }

    pub fn regs(&self) -> Vec<String> {
        match self {
            _ => Vec::new(),
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

    // fn set_field_id(mut self, old_id: &str, new_id: &str) -> Self {
    //     if let Some(i) = self.get_field_index(old_id) {
    //         self.fields[i].id = String::from(new_id);
    //     }

    //     self
    // }

    pub fn set_field_type(mut self, field_id: &str, ftype: FieldType) -> Self {
        if let Some((wi, fi)) = self.get_field_index(field_id) {
            self.fields[wi][fi].ftype = ftype;
        }

        self
    }

    pub fn split_field(mut self, field_id: &str, mut split: ProtoPattern) -> Self {
        if let Some((wi, mut fi)) = self.get_field_index(field_id) {
            let field = &self.fields[wi][fi];

            if field.len() != split.len() {
                println!("WARNING: Length are not matching for field splitting");
                return self;
            }

            let mut start = self.fields[wi].remove(fi).bit_range.start;

            while let Some(proto) = split.pop() {
                let f = proto.to_field(start);
                start = f.bit_range.end + 1;

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
            word_index += 1;
        }

        Pattern { fields }
    }
}

// #[derive(Debug, Clone)]
// pub enum Pattern {
//     Word(WordPattern),
//     Double([WordPattern; 2]),
//     Quad([WordPattern; 4]),
// }

// impl Pattern {
//     fn get_field_index(&self, field_id: &str) -> Option<(usize, usize)> {
//         for (i, field) in self.fields.iter().enumerate() {
//             if field.id == field_id {
//                 return Some(i);
//             }
//         }

//         None
//     }

//     pub fn set_field_type(mut self, field_id: &str, ftype: FieldType) -> Self {
//         if let Some(i) = self.get_field_index(field_id) {
//             self.fields[i].ftype = ftype;
//         }

//         self
//     }

//     pub fn split_field(mut self, field_id: &str, mut split: ProtoPattern) -> Self {

//     }
// }

// impl From<ProtoPattern> for Pattern {
//     fn from(value: ProtoPattern) -> Self {
//         Pattern::Word(WordPattern::from(value))
//     }
// }

// impl From<[ProtoPattern; 2]> for Pattern {
//     fn from(value: [ProtoPattern; 2]) -> Self {
//         Pattern::Double([
//             WordPattern::from(value[0].clone()),
//             WordPattern::from(value[1].clone()),
//         ])
//     }
// }

// impl From<[ProtoPattern; 4]> for Pattern {
//     fn from(value: [ProtoPattern; 4]) -> Self {
//         Pattern::Quad([
//             WordPattern::from(value[0].clone()),
//             WordPattern::from(value[1].clone()),
//             WordPattern::from(value[2].clone()),
//             WordPattern::from(value[3].clone()),
//         ])
//     }
// }

// impl Into<Vec<WordPattern>> for Pattern {
//     fn into(self) -> Vec<WordPattern> {
//         match self {
//             Pattern::Word(word) => vec![word],
//             Pattern::Double(words) => words.into(),
//             Pattern::Quad(words) => words.into(),
//         }
//     }
// }
