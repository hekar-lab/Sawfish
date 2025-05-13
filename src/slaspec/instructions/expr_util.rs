use super::{
    core::InstrBuilder,
    expr::{Expr, Op},
};

// Binary operations
pub fn e_copy(dst: Expr, src: Expr) -> Expr {
    Expr::Binary {
        lhs: Box::new(dst),
        op: Op::Copy,
        rhs: Box::new(src),
    }
}

pub fn e_add(lhs: Expr, rhs: Expr) -> Expr {
    Expr::Binary {
        lhs: Box::new(lhs),
        op: Op::Plus,
        rhs: Box::new(rhs),
    }
}

pub fn e_sub(lhs: Expr, rhs: Expr) -> Expr {
    Expr::Binary {
        lhs: Box::new(lhs),
        op: Op::Minus,
        rhs: Box::new(rhs),
    }
}

pub fn e_bit_or(lhs: Expr, rhs: Expr) -> Expr {
    Expr::Binary {
        lhs: Box::new(lhs),
        op: Op::BitOr,
        rhs: Box::new(rhs),
    }
}

pub fn e_ne(lhs: Expr, rhs: Expr) -> Expr {
    Expr::Binary {
        lhs: Box::new(lhs),
        op: Op::NE,
        rhs: Box::new(rhs),
    }
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
