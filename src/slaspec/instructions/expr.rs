use crate::slaspec::globals::DEFAULT_MEM;

use super::pattern::Pattern;

#[derive(Debug, Clone)]
pub enum Op {
    Copy,
    Plus,
    Minus,
    BitNot,
    BitOr,
    BitAnd,
    BitXor,
    LShft,
    RShft,
    Bang,
    And,
    Or,
    Xor,
    EQ,
    NE,
    LT,
    LTS,
    LE,
    LES,
    GT,
    GTS,
    GE,
    GES,
}

impl Op {
    pub fn to_string(&self) -> String {
        match self {
            Op::Copy => "=",
            Op::Plus => "+",
            Op::Minus => "-",
            Op::BitNot => "~",
            Op::BitOr => "|",
            Op::BitAnd => "&",
            Op::BitXor => "^",
            Op::LShft => "<<",
            Op::RShft => ">>",
            Op::Bang => "!",
            Op::And => "&&",
            Op::Or => "||",
            Op::Xor => "^^",
            Op::EQ => "==",
            Op::NE => "!=",
            Op::LT => "<",
            Op::LTS => "s<",
            Op::LE => "<=",
            Op::LES => "s<=",
            Op::GT => ">",
            Op::GTS => "s>",
            Op::GE => ">=",
            Op::GES => "s>=",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Line {
        current: Box<Expr>,
        next: Option<Box<Expr>>,
    },

    Field {
        id: String,
    },
    Var {
        id: String,
    },
    Reg {
        id: String,
    },
    Number {
        val: isize,
    },

    Macro {
        id: String,
        params: Vec<Box<Expr>>,
    },
    Addr {
        val: Box<Expr>,
    },
    Label {
        id: String,
    },
    Trunc {
        var: Box<Expr>,
        size: usize,
    },
    Local {
        var: Box<Expr>,
        size: usize,
    },
    Unary {
        op: Op,
        expr: Box<Expr>,
    },
    Binary {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
    Ptr {
        addr: Box<Expr>,
        size: usize,
    },
    Return {
        addr: Box<Expr>,
    },
    Goto {
        dest: Box<Expr>,
    },
    Call {
        addr: Box<Expr>,
    },

    IfGoto {
        cond: Box<Expr>,
        goto: Box<Expr>,
    },
    Group {
        expr: Box<Expr>,
    },
}

impl Expr {
    pub fn line(current: Expr, next: Option<Expr>) -> Expr {
        Expr::Line {
            current: Box::new(current),
            next: if let Some(ex) = next {
                Some(Box::new(ex))
            } else {
                None
            },
        }
    }

    pub fn field(id: &str) -> Expr {
        Expr::Field { id: id.to_string() }
    }

    pub fn var(id: &str) -> Expr {
        Expr::Var { id: id.to_string() }
    }

    pub fn reg(id: &str) -> Expr {
        Expr::Reg { id: id.to_string() }
    }

    pub fn trunc(var: Expr, size: usize) -> Expr {
        Expr::Trunc {
            var: Box::new(var),
            size,
        }
    }

    pub fn num(val: isize) -> Expr {
        Expr::Number { val }
    }

    pub fn mac(id: &str) -> Expr {
        Expr::Macro {
            id: id.to_string(),
            params: Vec::new(),
        }
    }

    pub fn macp(id: &str, param: Expr) -> Expr {
        Expr::Macro {
            id: id.to_string(),
            params: vec![Box::new(param)],
        }
    }

    pub fn mac2p(id: &str, param1: Expr, param2: Expr) -> Expr {
        Expr::Macro {
            id: id.to_string(),
            params: vec![Box::new(param1), Box::new(param2)],
        }
    }

    pub fn label(id: &str) -> Expr {
        Expr::Label { id: id.to_string() }
    }

    pub fn addr(val: Expr) -> Expr {
        Expr::Addr { val: Box::new(val) }
    }

    pub fn local(var: &str, size: usize) -> Expr {
        Expr::Local {
            var: Box::new(Expr::var(var)),
            size,
        }
    }

    pub fn bin(lhs: Expr, op: Op, rhs: Expr) -> Expr {
        Expr::Binary {
            lhs: Box::new(lhs),
            op: op,
            rhs: Box::new(rhs),
        }
    }

    pub fn un(op: Op, expr: Expr) -> Expr {
        Expr::Unary {
            op: op,
            expr: Box::new(expr),
        }
    }

    pub fn ptr(addr: Expr, size: usize) -> Expr {
        Expr::Ptr {
            addr: Box::new(addr),
            size,
        }
    }

    pub fn ret(addr: Expr) -> Expr {
        Expr::Return {
            addr: Box::new(Self::addr(addr)),
        }
    }

    pub fn call(addr: Expr) -> Expr {
        Expr::Call {
            addr: Box::new(Self::addr(addr)),
        }
    }

    pub fn goto(dest: Expr) -> Expr {
        Expr::Goto {
            dest: Box::new(dest),
        }
    }

    pub fn grp(expr: Expr) -> Expr {
        Expr::Group {
            expr: Box::new(expr),
        }
    }

    pub fn ifgoto(cond: Expr, goto: Expr) -> Expr {
        Expr::IfGoto {
            cond: Box::new(cond),
            goto: Box::new(goto),
        }
    }

    pub fn build(&self, pattern: &Pattern, prefix: &str) -> String {
        match self {
            Expr::Line { current, next } => {
                return format!(
                    "{}{}",
                    match &**current {
                        Expr::Label { id: _ } => format!("\n{}", current.build(pattern, prefix)),
                        Expr::Line {
                            current: _,
                            next: _,
                        } => current.build(pattern, prefix),
                        _ => format!("\n\t{};", current.build(pattern, prefix)),
                    },
                    if let Some(line) = next {
                        line.build(pattern, prefix)
                    } else {
                        "".to_string()
                    }
                );
            }
            Expr::Field { id } => {
                if let Some(f) = pattern.get_field(id) {
                    return format!("{}{}", prefix, f.name());
                }
            }
            Expr::Var { id } => return id.clone(),
            Expr::Reg { id } => return id.clone(),
            Expr::Number { val } => return format!("{val:#0x}"),
            Expr::Macro { id, params } => {
                let mut params_str = Vec::new();

                for p in params {
                    params_str.push(p.build(pattern, prefix));
                }

                return format!("{id}({})", params_str.join(", "));
            }
            Expr::Label { id } => return format!("<{id}>"),
            Expr::Addr { val } => return format!("[{}]", val.build(pattern, prefix)),
            Expr::Local { var, size } => {
                return format!("local {}:{size}", var.build(pattern, prefix));
            }
            Expr::Unary { op, expr } => {
                return format!("{}{}", op.to_string(), expr.build(pattern, prefix));
            }
            Expr::Binary { lhs, op, rhs } => {
                return format!(
                    "{} {} {}",
                    lhs.build(pattern, prefix),
                    op.to_string(),
                    rhs.build(pattern, prefix)
                );
            }
            Expr::Trunc { var, size } => {
                return format!("{}:{size}", var.build(pattern, prefix));
            }
            Expr::Ptr { addr, size } => {
                return format!("*[{}]:{size} {}", DEFAULT_MEM, addr.build(pattern, prefix));
            }
            Expr::Return { addr: expr } => {
                return format!("return {}", expr.build(pattern, prefix));
            }
            Expr::Goto { dest: expr } => {
                return format!("goto {}", expr.build(pattern, prefix));
            }
            Expr::Call { addr: expr } => {
                return format!("call {}", expr.build(pattern, prefix));
            }
            Expr::Group { expr } => {
                return format!("({})", expr.build(pattern, prefix));
            }
            Expr::IfGoto { cond, goto } => {
                return format!(
                    "if ({}) goto {}",
                    cond.build(pattern, prefix),
                    goto.build(pattern, prefix)
                );
            }
        }

        String::new()
    }
}

#[derive(Debug, Clone)]
pub struct Code {
    exprs: Vec<Expr>,
}

impl Code {
    pub fn new() -> Self {
        Code { exprs: Vec::new() }
    }

    pub fn add_expr(&mut self, e: Expr) {
        self.exprs.push(e);
    }

    pub fn is_empty(&self) -> bool {
        self.exprs.is_empty()
    }

    pub fn build(&self, pattern: &Pattern, prefix: &str) -> String {
        let mut out = String::new();

        for ex in &self.exprs {
            match ex {
                Expr::Label { id: _ } => out += &format!("\n{}", ex.build(pattern, prefix)),
                Expr::Line {
                    current: _,
                    next: _,
                } => out += &ex.build(pattern, prefix),
                _ => out += &format!("\n\t{};", ex.build(pattern, prefix)),
            }
        }

        out
    }
}
