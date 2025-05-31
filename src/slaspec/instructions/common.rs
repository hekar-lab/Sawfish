use super::expr::Expr;

pub type UnOp = fn(Expr) -> Expr;
pub type BinOp = fn(Expr, Expr) -> Expr;
