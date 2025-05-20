use std::collections::VecDeque;

use super::{
    common::BinOp,
    expr::{Expr, Op},
};

//// Base operations ////
// Arch operations
pub fn b_line(current: Expr, next: Option<Expr>) -> Expr {
    Expr::Line {
        current: Box::new(current),
        next: if let Some(ex) = next {
            Some(Box::new(ex))
        } else {
            None
        },
    }
}

pub fn b_grp(expr: Expr) -> Expr {
    Expr::Group {
        expr: Box::new(expr),
    }
}

// Var operations
pub fn b_field(id: &str) -> Expr {
    Expr::Field { id: id.to_string() }
}

pub fn b_var(id: &str) -> Expr {
    Expr::Var { id: id.to_string() }
}

pub fn b_reg(id: &str) -> Expr {
    Expr::Reg { id: id.to_string() }
}

pub fn b_num(val: isize) -> Expr {
    Expr::Number { val }
}

pub fn b_size(var: Expr, size: usize) -> Expr {
    Expr::Size {
        var: Box::new(var),
        size,
    }
}

pub fn b_ptr(addr: Expr, size: usize) -> Expr {
    Expr::Ptr {
        addr: Box::new(addr),
        size,
    }
}

pub fn b_local(var: Expr, size: usize) -> Expr {
    Expr::Local {
        var: Box::new(var),
        size,
    }
}

// Macro operation
pub fn b_mac(id: &str, params: Vec<Expr>) -> Expr {
    Expr::Macro {
        id: id.to_string(),
        params: params.into_iter().map(|p| Box::new(p)).collect(),
    }
}

// Control flow operations
pub fn b_label(id: &str) -> Expr {
    Expr::Label { id: id.to_string() }
}

pub fn b_indirect(val: Expr) -> Expr {
    Expr::Indirect { val: Box::new(val) }
}

pub fn b_ret(addr: Expr) -> Expr {
    Expr::Return {
        addr: Box::new(addr),
    }
}

pub fn b_call(addr: Expr) -> Expr {
    Expr::Call {
        addr: Box::new(addr),
    }
}

pub fn b_goto(dest: Expr) -> Expr {
    Expr::Goto {
        dest: Box::new(dest),
    }
}

pub fn b_ifgoto(cond: Expr, goto: Expr) -> Expr {
    Expr::IfGoto {
        cond: Box::new(cond),
        goto: Box::new(goto),
    }
}

// Unary/Binary operations
pub fn b_bin(lhs: Expr, op: Op, rhs: Expr) -> Expr {
    Expr::Binary {
        lhs: Box::new(lhs),
        op: op,
        rhs: Box::new(rhs),
    }
}

pub fn b_un(op: Op, expr: Expr) -> Expr {
    Expr::Unary {
        op: op,
        expr: Box::new(expr),
    }
}
//// Base operations

//// Compound expressions
// Var expressions

pub fn e_local(var: &str, size: usize) -> Expr {
    b_local(b_var(var), size)
}

// Macro expressions
pub fn e_mac(id: &str) -> Expr {
    b_mac(id, Vec::new())
}

pub fn e_macp(id: &str, param: Expr) -> Expr {
    b_mac(id, vec![param])
}

pub fn e_mac2p(id: &str, param1: Expr, param2: Expr) -> Expr {
    b_mac(id, vec![param1, param2])
}

// Control flow expressions

pub fn e_ret(addr: Expr) -> Expr {
    b_ret(b_indirect(addr))
}

pub fn e_call(addr: Expr) -> Expr {
    b_call(b_indirect(addr))
}

// Unary expressions
pub fn e_not(expr: Expr) -> Expr {
    b_un(Op::Bang, expr)
}

pub fn e_neg(expr: Expr) -> Expr {
    b_un(Op::Minus, expr)
}

pub fn e_bit_not(expr: Expr) -> Expr {
    b_un(Op::BitNot, expr)
}

// Binary expressions
pub fn e_copy(dst: Expr, src: Expr) -> Expr {
    b_bin(dst, Op::Copy, src)
}

pub fn e_add(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::Plus, rhs)
}

pub fn e_sub(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::Minus, rhs)
}

pub fn e_mult(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::Mult, rhs)
}

pub fn e_bit_or(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::BitOr, rhs)
}

pub fn e_bit_and(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::BitAnd, rhs)
}

pub fn e_bit_xor(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::BitXor, rhs)
}

pub fn e_alshft(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::ALShft, rhs)
}

pub fn e_arshft(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::ARShft, rhs)
}

pub fn e_lshft(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::LShft, rhs)
}

pub fn e_rshft(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::RShft, rhs)
}

pub fn e_and(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::And, rhs)
}

pub fn e_or(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::Or, rhs)
}

pub fn e_xor(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::Xor, rhs)
}

pub fn e_eq(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::EQ, rhs)
}

pub fn e_ne(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::NE, rhs)
}

pub fn e_lt(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::LT, rhs)
}

pub fn e_lts(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::LTS, rhs)
}

pub fn e_le(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::LE, rhs)
}

pub fn e_les(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::LES, rhs)
}

pub fn e_gt(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::GT, rhs)
}

pub fn e_gts(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::GTS, rhs)
}

pub fn e_ge(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::GE, rhs)
}

pub fn e_ges(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::GES, rhs)
}
//// Compound expressions

//// Code snippet
pub fn cs_push(val: Expr, size: usize) -> Expr {
    b_line(
        e_copy(b_reg("SP"), e_sub(b_reg("SP"), b_num(4))),
        Some(b_line(
            e_copy(b_ptr(b_reg("SP"), size), b_size(val, size)),
            None,
        )),
    )
}

pub fn cs_pop(val: Expr, size: usize) -> Expr {
    b_line(
        e_copy(val, b_ptr(b_reg("SP"), size)),
        Some(b_line(
            e_copy(b_reg("SP"), e_add(b_reg("SP"), b_num(4))),
            None,
        )),
    )
}

pub fn cs_assign_by(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
    e_copy(lhs.clone(), op(lhs, rhs))
}

fn rec_mline(mut exprs: VecDeque<Expr>) -> Option<Expr> {
    match exprs.pop_front() {
        Some(e) => Some(b_line(e, rec_mline(exprs))),
        None => None,
    }
}

pub fn cs_mline(mut exprs: VecDeque<Expr>) -> Expr {
    if exprs.is_empty() {
        panic!("Cannot make a multiline expression with an empty array!")
    }

    b_line(exprs.pop_front().unwrap(), rec_mline(exprs))
}

pub fn cs_ifgoto(cond: Expr, goto: Expr) -> Expr {
    let label = "workaround";
    let inv = if let Expr::Unary { op: Op::Bang, expr } = cond {
        *expr
    } else {
        e_not(cond)
    };
    cs_mline(
        vec![
            b_ifgoto(inv, b_label(label)),
            b_goto(b_indirect(goto)),
            b_label(label),
        ]
        .into(),
    )
}
//// Code snippet
