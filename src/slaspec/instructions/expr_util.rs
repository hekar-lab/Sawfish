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

pub fn e_ne(lhs: Expr, rhs: Expr) -> Expr {
    Expr::bin(lhs, Op::NE, rhs)
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
