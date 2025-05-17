use std::collections::VecDeque;

use super::expr::{Expr, Op};

// Unary operations
pub fn e_not(expr: Expr) -> Expr {
    Expr::un(Op::Bang, expr)
}

pub fn e_bit_not(expr: Expr) -> Expr {
    Expr::un(Op::BitNot, expr)
}

// Binary operations
pub fn e_copy(dst: Expr, src: Expr) -> Expr {
    Expr::bin(dst, Op::Copy, src)
}

pub fn e_add(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::Plus, rhs)
}

pub fn e_sub(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::Minus, rhs)
}

pub fn e_mult(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::Mult, rhs)
}

pub fn e_bit_or(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::BitOr, rhs)
}

pub fn e_bit_and(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::BitAnd, rhs)
}

pub fn e_bit_xor(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::BitXor, rhs)
}

pub fn e_lshft(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::LShft, rhs)
}

pub fn e_rshft(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::RShft, rhs)
}

pub fn e_and(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::And, rhs)
}

pub fn e_or(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::Or, rhs)
}

pub fn e_xor(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::Xor, rhs)
}

pub fn e_eq(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::EQ, rhs)
}

pub fn e_ne(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::NE, rhs)
}

pub fn e_lt(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::LT, rhs)
}

pub fn e_lts(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::LTS, rhs)
}

pub fn e_le(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::LE, rhs)
}

pub fn e_les(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::LES, rhs)
}

pub fn e_gt(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::GT, rhs)
}

pub fn e_gts(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::GTS, rhs)
}

pub fn e_ge(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::GE, rhs)
}

pub fn e_ges(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::GES, rhs)
}

// Code snippet
pub fn cs_push(val: Expr, size: usize) -> Expr {
    Expr::line(
        e_copy(Expr::reg("SP"), e_sub(Expr::reg("SP"), Expr::num(4))),
        Some(Expr::line(
            e_copy(Expr::ptr(Expr::reg("SP"), size), Expr::trunc(val, size)),
            None,
        )),
    )
}

pub fn cs_pop(val: Expr, size: usize) -> Expr {
    Expr::line(
        e_copy(Expr::trunc(val, size), Expr::ptr(Expr::reg("SP"), size)),
        Some(Expr::line(
            e_copy(Expr::reg("SP"), e_add(Expr::reg("SP"), Expr::num(4))),
            None,
        )),
    )
}

pub fn cs_assign_by(op: fn(Expr, Expr) -> Expr, lhs: Expr, rhs: Expr) -> Expr {
    e_copy(lhs.clone(), op(lhs, rhs))
}

fn rec_mline(mut exprs: VecDeque<Expr>) -> Option<Expr> {
    match exprs.pop_front() {
        Some(e) => Some(Expr::line(e, rec_mline(exprs))),
        None => None,
    }
}

pub fn cs_mline(mut exprs: VecDeque<Expr>) -> Expr {
    if exprs.is_empty() {
        panic!("Cannot make a multiline expression with an empty array!")
    }

    Expr::line(exprs.pop_front().unwrap(), rec_mline(exprs))
}
