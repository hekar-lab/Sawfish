use super::{pattern::Pattern, util::capitalize};

#[derive(Debug, Clone)]
enum Token {
    Literal(String),
    Field(String),
    Variable(String),
}

struct Scanner {
    text: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
}

impl Scanner {
    fn new(text: &str) -> Self {
        Scanner {
            text: String::from(text),
            tokens: Vec::new(),
            start: 0,
            current: 0,
        }
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn at_end(&self) -> bool {
        self.current >= self.text.len()
    }

    fn peek(&self) -> char {
        if self.at_end() {
            return '\0';
        }

        self.text.chars().nth(self.current).unwrap()
    }

    fn advance(&mut self) -> char {
        let c = self.peek();
        self.current += 1;
        c
    }

    fn chr_match(&mut self, expected: char) -> bool {
        if self.at_end() {
            return false;
        }
        if self.peek() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn scan(&mut self) -> Vec<Token> {
        while !self.at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '{' => {
                if self.chr_match('{') {
                    self.add_token(Token::Literal(String::from("{")));
                } else {
                    self.variable();
                }
            }
            '}' => {
                if self.chr_match('}') {
                    self.add_token(Token::Literal(String::from("}")));
                } else {
                    panic!("single '}}' is not allowed");
                }
            }
            _ => {
                self.literal();
            }
        }
    }

    fn literal(&mut self) {
        while !self.at_end() && self.peek() != '{' {
            self.advance();
        }

        let value = String::from(&self.text[self.start..self.current]);
        self.add_token(Token::Literal(value));
    }

    fn variable(&mut self) {
        if self.chr_match('$') {
            let content = self.var_content(true);
            self.add_token(Token::Variable(content));
        } else {
            let content = self.var_content(false);
            self.add_token(Token::Field(content));
        }
    }

    fn var_content(&mut self, is_var: bool) -> String {
        while !self.at_end() && self.peek() != '}' {
            self.advance();
        }

        if self.at_end() {
            panic!("Missing closing bracket");
        }

        self.advance();

        if is_var {
            String::from(&self.text[self.start + 2..self.current - 1])
        } else {
            String::from(&self.text[self.start + 1..self.current - 1])
        }
    }
}

pub fn display_format(txt: &str, pattern: &Pattern, prefix: &str) -> (String, usize) {
    let mut scanner = Scanner::new(txt);
    let tokens = scanner.scan();
    let mut out = String::new();
    let mut var_count = 0;

    for tok in tokens {
        match &tok {
            Token::Literal(s) => out += &format!("\"{s}\""),
            Token::Variable(s) => {
                out += &s;
                var_count += 1;
            }
            Token::Field(s) => {
                if let Some(f) = pattern.get_field(s) {
                    out += &format!("{}{}", prefix, f.name())
                }
                var_count += 1
            }
        }
    }

    (out, var_count)
}

#[allow(dead_code)]
pub fn display_add_prefix(txt: &str, prefix: &str) -> String {
    let mut scanner = Scanner::new(txt);
    let tokens = scanner.scan();
    let mut out = String::new();

    for tok in tokens {
        match &tok {
            Token::Literal(s) => out += s,
            Token::Variable(s) => {
                out += &format!("{{${}{}}}", prefix, capitalize(s));
            }
            Token::Field(s) => {
                out += &format!("{{{}{}}}", prefix, capitalize(s));
            }
        }
    }

    out
}
