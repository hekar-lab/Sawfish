use std::collections::HashSet;

use super::{expr_util::*, pattern::Pattern, util::capitalize};

#[derive(Debug, Clone)]
pub enum Op {
    Copy,
    Plus,
    Minus,
    Mult,
    BitNot,
    BitOr,
    BitAnd,
    BitXor,
    ALShft,
    ARShft,
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
            Op::Mult => "*",
            Op::BitNot => "~",
            Op::BitOr => "|",
            Op::BitAnd => "&",
            Op::BitXor => "^",
            Op::ALShft => "s<<",
            Op::ARShft => "s>>",
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
        is_reg: bool,
    },
    Var {
        id: String,
    },
    Reg {
        id: String,
    },
    Number {
        val: i128,
    },

    Macro {
        id: String,
        params: Vec<Box<Expr>>,
    },
    Indirect {
        val: Box<Expr>,
    },
    Label {
        id: String,
    },
    Size {
        var: Box<Expr>,
        size: usize,
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
        space: String,
        addr: Box<Expr>,
        size: usize,
    },
    Ref {
        var: Box<Expr>,
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
            Expr::Field { id, is_reg: _ } => {
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
            Expr::Indirect { val } => return format!("[{}]", val.build(pattern, prefix)),
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
            Expr::Size { var, size } => {
                return format!("{}:{size}", var.build(pattern, prefix));
            }
            Expr::Trunc { var, size } => {
                return format!("{}({size})", var.build(pattern, prefix));
            }
            Expr::Ptr { space, addr, size } => {
                return format!("*[{space}]:{size} {}", addr.build(pattern, prefix));
            }
            Expr::Ref { var } => {
                return format!("&{}", var.build(pattern, prefix));
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

    pub fn multify(self, prefix: &str, rhs_cpy: bool, regs: &mut HashSet<(bool, String)>) -> Expr {
        match self {
            Expr::Line { current, next } => b_line(
                current.multify(prefix, rhs_cpy, regs),
                if let Some(val) = next {
                    Some(val.multify(prefix, rhs_cpy, regs))
                } else {
                    None
                },
            ),
            Expr::Field { id, is_reg } => {
                if is_reg && rhs_cpy {
                    regs.insert((true, format!("{}{}", prefix, &capitalize(&id))));
                    b_var(&format!("old_{}{}Reg", prefix, &capitalize(&id)))
                } else {
                    b_field(&format!("{}{}", prefix, &capitalize(&id)), is_reg)
                }
            }
            Expr::Var { id } => b_var(&format!("{}{}", prefix, &capitalize(&id))),
            Expr::Reg { id } => {
                if rhs_cpy {
                    regs.insert((false, id.clone()));
                    b_var(&format!("old_{id}"))
                } else {
                    b_reg(&id)
                }
            }
            Expr::Number { val } => b_num(val),
            Expr::Macro { id, params } => b_mac(
                &id,
                params
                    .into_iter()
                    .map(|e| e.multify(prefix, rhs_cpy, regs))
                    .collect(),
            ),
            Expr::Label { id } => b_label(&capitalize(&id)),
            Expr::Indirect { val } => b_indirect(val.multify(prefix, rhs_cpy, regs)),
            Expr::Local { var, size } => b_local(var.multify(prefix, rhs_cpy, regs), size),
            Expr::Unary { op, expr } => b_un(op, expr.multify(prefix, rhs_cpy, regs)),
            Expr::Binary { lhs, op, rhs } => match op {
                Op::Copy => b_bin(
                    lhs.multify(prefix, false, regs),
                    op,
                    rhs.multify(prefix, true, regs),
                ),
                _ => b_bin(
                    lhs.multify(prefix, rhs_cpy, regs),
                    op,
                    rhs.multify(prefix, rhs_cpy, regs),
                ),
            },
            Expr::Size { var, size } => b_size(var.multify(prefix, rhs_cpy, regs), size),
            Expr::Trunc { var, size } => b_trunc(var.multify(prefix, rhs_cpy, regs), size),
            Expr::Ptr { space, addr, size } => {
                b_ptr(&space, addr.multify(prefix, rhs_cpy, regs), size)
            }
            Expr::Ref { var } => b_ref(var.multify(prefix, rhs_cpy, regs)),
            Expr::Return { addr } => b_ret(addr.multify(prefix, rhs_cpy, regs)),
            Expr::Goto { dest } => b_goto(dest.multify(prefix, rhs_cpy, regs)),
            Expr::Call { addr } => b_call(addr.multify(prefix, rhs_cpy, regs)),
            Expr::Group { expr } => b_grp(expr.multify(prefix, rhs_cpy, regs)),
            Expr::IfGoto { cond, goto } => b_ifgoto(
                cond.multify(prefix, rhs_cpy, regs),
                goto.multify(prefix, rhs_cpy, regs),
            ),
        }
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

    pub fn multify(self, prefix: &str, rhs: bool, regs: &mut HashSet<(bool, String)>) -> Code {
        Code {
            exprs: self
                .exprs
                .into_iter()
                .map(|e| e.multify(prefix, rhs, regs))
                .collect(),
        }
    }

    pub fn append(&mut self, mut code: Code) {
        self.exprs.append(&mut code.exprs);
    }
}
