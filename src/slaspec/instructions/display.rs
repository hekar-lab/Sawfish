////// THIS SHOULD ONLY BE DISPLAY TEXT YOU JACKASS >:((((

use std::{collections::VecDeque, ops};

use super::{core::Prefixed, pattern::Field};

#[derive(Debug, Clone)]
pub enum Token {
    Literal(String),
    Variable(String),
    Token(Field),
}

impl Token {
    fn to_str(&self) -> String {
        match self {
            Self::Literal(s) | Self::Variable(s) => s.clone(),
            _ => String::new(),
        }
    }
}

impl Into<Token> for &str {
    fn into(self) -> Token {
        Token::Literal(String::from(self))
    }
}

impl Into<Token> for String {
    fn into(self) -> Token {
        Token::Literal(self)
    }
}

impl Into<Token> for &Field {
    fn into(self) -> Token {
        Token::Token(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct Display {
    content: VecDeque<Token>,
}

impl Display {
    pub fn new() -> Self {
        Display {
            content: VecDeque::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn append(mut self, mut suffix: Display) -> Self {
        self.content.append(&mut suffix.content);
        self
    }

    pub fn prepend(mut self, mut prefix: Display) -> Self {
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
        let mut simp_txt = Display::new();
        if self.content.is_empty() {
            return simp_txt;
        }

        let mut content = self.content.clone();

        while let Some(ex) = content.pop_back() {
            if simp_txt.content.is_empty() {
                simp_txt.content.push_front(ex);
                continue;
            }

            if matches!(ex, Token::Literal(_))
                && matches!(simp_txt.content.front(), Some(Token::Literal(_)))
            {
                let prev = simp_txt.content.pop_front().unwrap().to_str();
                let current = ex.to_str();
                let concat = Token::Literal(format!("{prev}{current}"));
                simp_txt.content.push_front(concat);
            } else {
                simp_txt.content.push_front(ex);
            }
        }

        simp_txt
    }

    pub fn generate<I: Prefixed>(&self, instr: &I) -> String {
        let mut txt = String::new();

        for ex in &self.content {
            match ex {
                Token::Literal(s) => txt += &format!("\"{s}\""),
                Token::Variable(id) => txt += &format!("{id}"),
                Token::Token(f) => txt += &f.token_name(instr),
            }
        }

        txt
    }
}

impl From<&str> for Display {
    fn from(value: &str) -> Self {
        let content = VecDeque::from([value.into()]);
        Display { content }
    }
}

impl From<String> for Display {
    fn from(value: String) -> Self {
        let content = VecDeque::from([value.into()]);
        Display { content }
    }
}

impl From<&Field> for Display {
    fn from(value: &Field) -> Self {
        let content = VecDeque::from([value.into()]);
        Display { content }
    }
}

impl ops::Add for Display {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.append(rhs)
    }
}

impl ops::Add<&str> for Display {
    type Output = Self;
    fn add(self, rhs: &str) -> Self::Output {
        self.append(Display::from(rhs))
    }
}

impl ops::Add<String> for Display {
    type Output = Self;
    fn add(self, rhs: String) -> Self::Output {
        self.append(Display::from(rhs))
    }
}

impl ops::Add<Display> for &str {
    type Output = Display;
    fn add(self, rhs: Display) -> Self::Output {
        Display::from(self).append(rhs)
    }
}

impl ops::Add<Display> for String {
    type Output = Display;
    fn add(self, rhs: Display) -> Self::Output {
        Display::from(self).append(rhs)
    }
}
