////// THIS SHOULD ONLY BE DISPLAY TEXT YOU JACKASS >:((((

use std::{collections::VecDeque, ops};

use super::{core::Prefixed, pattern::Field};

#[derive(Debug, Clone)]
pub enum Excerpt {
    Litteral(String),
    Local(String, usize),
    Variable(Field),
}

impl Excerpt {
    fn to_str(&self) -> String {
        match self {
            Self::Litteral(s) => s.clone(),
            _ => String::new(),
        }
    }
}

impl Into<Excerpt> for &str {
    fn into(self) -> Excerpt {
        Excerpt::Litteral(String::from(self))
    }
}

impl Into<Excerpt> for String {
    fn into(self) -> Excerpt {
        Excerpt::Litteral(self)
    }
}

impl Into<Excerpt> for &Field {
    fn into(self) -> Excerpt {
        Excerpt::Variable(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    content: VecDeque<Excerpt>,
}

impl Text {
    pub fn new() -> Self {
        Text {
            content: VecDeque::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn append(mut self, mut suffix: Text) -> Self {
        self.content.append(&mut suffix.content);
        self
    }

    pub fn prepend(mut self, mut prefix: Text) -> Self {
        prefix.content.append(&mut self.content);
        prefix
    }

    // pub fn append(mut self, suffix: Excerpt) -> Self {
    //     self.content.push_front(suffix);
    //     self
    // }

    // pub fn prepend(mut self, prefix: Excerpt) -> Self {
    //     self.content.push_back(prefix);
    //     self
    // }

    pub fn simplify(self) -> Self {
        let mut simp_txt = Text::new();
        if self.content.is_empty() {
            return simp_txt;
        }

        let mut content = self.content.clone();

        while let Some(ex) = content.pop_back() {
            if simp_txt.content.is_empty() {
                simp_txt.content.push_front(ex);
                continue;
            }

            if matches!(ex, Excerpt::Litteral(_))
                && matches!(simp_txt.content.front(), Some(Excerpt::Litteral(_)))
            {
                let prev = simp_txt.content.pop_front().unwrap().to_str();
                let current = ex.to_str();
                let concat = Excerpt::Litteral(format!("{prev}{current}"));
                simp_txt.content.push_front(concat);
            } else {
                simp_txt.content.push_front(ex);
            }
        }

        simp_txt
    }

    pub fn generate<I: Prefixed>(&self, instr: &I) -> String {
        let mut txt = String::new();
        let mut locals = String::new();

        for ex in &self.content {
            match ex {
                Excerpt::Litteral(s) => txt += &format!("\"{s}\""),
                Excerpt::Local(id, sz) => {
                    locals += &format!("local {id}:{sz};\n");
                    txt += id
                }
                Excerpt::Variable(f) => txt += &f.token_name(instr),
            }
        }

        format!("{locals}\n{txt}")
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        let content = VecDeque::from([value.into()]);
        Text { content }
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        let content = VecDeque::from([value.into()]);
        Text { content }
    }
}

impl From<&Field> for Text {
    fn from(value: &Field) -> Self {
        let content = VecDeque::from([value.into()]);
        Text { content }
    }
}

impl ops::Add for Text {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.append(rhs)
    }
}

impl ops::Add<&str> for Text {
    type Output = Self;
    fn add(self, rhs: &str) -> Self::Output {
        self.append(Text::from(rhs))
    }
}

impl ops::Add<String> for Text {
    type Output = Self;
    fn add(self, rhs: String) -> Self::Output {
        self.append(Text::from(rhs))
    }
}

impl ops::Add<Text> for &str {
    type Output = Text;
    fn add(self, rhs: Text) -> Self::Output {
        Text::from(self).append(rhs)
    }
}

impl ops::Add<Text> for String {
    type Output = Text;
    fn add(self, rhs: Text) -> Self::Output {
        Text::from(self).append(rhs)
    }
}
