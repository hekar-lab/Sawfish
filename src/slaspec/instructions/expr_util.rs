use std::collections::VecDeque;

use crate::slaspec::globals::DEFAULT_MEM;

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
pub fn b_field(id: &str, is_reg: bool) -> Expr {
    Expr::Field {
        id: id.to_string(),
        is_reg,
    }
}

pub fn b_var(id: &str) -> Expr {
    Expr::Var { id: id.to_string() }
}

pub fn b_reg(id: &str) -> Expr {
    Expr::Reg { id: id.to_string() }
}

pub fn b_num(val: i128) -> Expr {
    Expr::Number { val }
}

pub fn b_size(var: Expr, size: usize) -> Expr {
    Expr::Size {
        var: Box::new(var),
        size,
    }
}

pub fn b_trunc(var: Expr, size: usize) -> Expr {
    Expr::Trunc {
        var: Box::new(var),
        size,
    }
}

pub fn b_ptr(space: &str, addr: Expr, size: usize) -> Expr {
    Expr::Ptr {
        space: space.to_string(),
        addr: Box::new(addr),
        size,
    }
}

pub fn b_ref(var: Expr) -> Expr {
    Expr::Ref { var: Box::new(var) }
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

pub fn e_field(id: &str) -> Expr {
    b_field(id, false)
}

pub fn e_rfield(id: &str) -> Expr {
    b_field(id, true)
}

pub fn e_local(var: &str, size: usize) -> Expr {
    b_local(b_var(var), size)
}

pub fn e_ptr(addr: Expr, size: usize) -> Expr {
    b_ptr(DEFAULT_MEM, addr, size)
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

pub fn e_zext(var: Expr) -> Expr {
    e_macp("zext", var)
}

pub fn e_sext(var: Expr) -> Expr {
    e_macp("sext", var)
}

pub fn e_carry(a: Expr, b: Expr) -> Expr {
    e_mac2p("carry", a, b)
}

pub fn e_scarry(a: Expr, b: Expr) -> Expr {
    e_mac2p("scarry", a, b)
}

pub fn e_sborrow(a: Expr, b: Expr) -> Expr {
    e_mac2p("sborrow", a, b)
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

pub fn e_rem(lhs: Expr, rhs: Expr) -> Expr {
    b_bin(lhs, Op::Rem, rhs)
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
    cs_mline(vec![
        e_copy(b_reg("SP"), e_sub(b_reg("SP"), b_num(4))),
        e_copy(
            e_ptr(b_reg("SP"), 4),
            if size < 4 { e_sext(val) } else { val },
        ),
    ])
}

pub fn cs_pop(val: Expr, size: usize) -> Expr {
    cs_mline(vec![
        e_copy(val, e_ptr(b_reg("SP"), size)),
        e_copy(b_reg("SP"), e_add(b_reg("SP"), b_num(4))),
    ])
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

pub fn cs_mline(exprs: Vec<Expr>) -> Expr {
    if exprs.is_empty() {
        panic!("Cannot make a multiline expression with an empty array!")
    }

    let mut exprs_queue: VecDeque<Expr> = exprs.into();

    b_line(exprs_queue.pop_front().unwrap(), rec_mline(exprs_queue))
}

pub fn cs_ifgoto_lab(cond: Expr, goto: Expr, label: &str) -> Expr {
    let inv = if let Expr::Unary { op: Op::Bang, expr } = cond {
        *expr
    } else {
        e_not(cond)
    };
    cs_mline(vec![
        b_ifgoto(inv, b_label(label)),
        b_goto(b_indirect(goto)),
        b_label(label),
    ])
}

pub fn cs_ifgoto(cond: Expr, goto: Expr) -> Expr {
    cs_ifgoto_lab(cond, goto, "ignore_jump")
}

pub fn cs_add_sat(dst: Expr, src0: Expr, src1: Expr, size: usize, id: &str) -> Expr {
    let sat_label = b_label(&format!("end_add_sat_{id}"));
    cs_mline(vec![
        e_copy(dst.clone(), b_num((1 << (8 * size)) - 1)),
        b_ifgoto(e_carry(src0.clone(), src1.clone()), sat_label.clone()),
        e_copy(dst, e_add(src0, src1)),
        sat_label,
    ])
}

pub fn cs_sub_sat(dst: Expr, src0: Expr, src1: Expr, id: &str) -> Expr {
    let sat_label = b_label(&format!("end_sub_sat_{id}"));
    cs_mline(vec![
        e_copy(dst.clone(), b_num(0)),
        b_ifgoto(b_grp(e_lt(src0.clone(), src1.clone())), sat_label.clone()),
        e_copy(dst, e_sub(src0, src1)),
        sat_label,
    ])
}

pub fn cs_sadd_sat(dst: Expr, src0: Expr, src1: Expr, size: usize, id: &str) -> Expr {
    let end_label = b_label(&format!("end_sadd_sat_{id}"));
    let src0_cpy = b_var(&format!("sadd_src0_cpy_{id}"));
    cs_mline(vec![
        e_copy(b_local(src0_cpy.clone(), size), src0.clone()),
        e_copy(dst.clone(), e_add(src0.clone(), src1.clone())),
        b_ifgoto(e_not(e_scarry(src0_cpy, src1.clone())), end_label.clone()),
        e_copy(dst.clone(), b_num(1 << (size * 8 - 1))),
        b_ifgoto(e_lts(src1, b_num(0)), end_label.clone()),
        e_copy(dst.clone(), b_num((1 << (size * 8 - 1)) - 1)),
        end_label,
    ])
}

pub fn cs_ssub_sat(dst: Expr, src0: Expr, src1: Expr, size: usize, id: &str) -> Expr {
    let end_label = b_label(&format!("end_ssub_sat_{id}"));
    let src0_cpy = b_var(&format!("sadd_src0_cpy_{id}"));
    cs_mline(vec![
        e_copy(b_local(src0_cpy.clone(), size), src0.clone()),
        e_copy(dst.clone(), e_sub(src0.clone(), src1.clone())),
        b_ifgoto(e_not(e_sborrow(src0_cpy, src1.clone())), end_label.clone()),
        e_copy(dst.clone(), b_num(1 << (size * 8 - 1))),
        b_ifgoto(e_gts(src1, b_num(0)), end_label.clone()),
        e_copy(dst.clone(), b_num((1 << (size * 8 - 1)) - 1)),
        end_label,
    ])
}

pub fn cs_strunc_sat(dst: Expr, src: Expr, size: usize, id: &str) -> Expr {
    let end_label = b_label(&format!("end_strunc_{id}"));
    cs_mline(vec![
        e_copy(dst.clone(), b_size(src.clone(), size)),
        b_ifgoto(e_eq(e_sext(dst.clone()), src.clone()), end_label.clone()),
        e_copy(dst.clone(), b_num(1 << (size * 8 - 1))),
        b_ifgoto(e_lts(src, b_num(0)), end_label.clone()),
        e_copy(dst.clone(), b_num((1 << (size * 8 - 1)) - 1)),
        end_label,
    ])
}

pub fn cs_trunc_sat(dst: Expr, src: Expr, size: usize, id: &str) -> Expr {
    let end_label = b_label(&format!("end_trunc_{id}"));
    cs_mline(vec![
        e_copy(dst.clone(), b_size(src.clone(), size)),
        b_ifgoto(e_eq(e_zext(dst.clone()), src.clone()), end_label.clone()),
        e_copy(dst.clone(), b_num((1 << (size * 8)) - 1)),
        end_label,
    ])
}

pub fn cs_round(dst: Expr, dst_size: usize, src: Expr, src_size: usize, id: &str) -> Expr {
    let rnd_mod_label = b_label(&format!("biased_rnd_{id}"));
    let add_label = b_label(&format!("rounding_{id}"));
    let end_label = b_label(&format!("end_rnd_{id}"));
    let rem_var = b_var(&format!("rem_var_{id}"));
    let rem_size = src_size - dst_size;
    let threshold = 1 << (rem_size * 8 - 1);
    cs_mline(vec![
        e_copy(
            b_local(rem_var.clone(), rem_size),
            b_size(src.clone(), rem_size),
        ),
        e_copy(dst.clone(), b_trunc(src, rem_size)),
        b_ifgoto(
            e_eq(dst.clone(), b_num((1 << (8 * dst_size - 1)) - 1)),
            end_label.clone(),
        ),
        b_ifgoto(b_reg("RND_MOD"), rnd_mod_label.clone()),
        b_ifgoto(
            e_or(
                e_gt(rem_var.clone(), b_num(threshold)),
                b_grp(e_and(
                    e_eq(rem_var.clone(), b_num(threshold)),
                    e_eq(b_grp(e_bit_and(dst.clone(), b_num(1))), b_num(1)),
                )),
            ),
            add_label.clone(),
        ),
        b_goto(end_label.clone()),
        rnd_mod_label,
        b_ifgoto(e_ge(rem_var, b_num(threshold)), add_label.clone()),
        b_goto(end_label.clone()),
        add_label,
        cs_assign_by(e_add, dst, b_num(1)),
        end_label,
    ])
}

pub fn cs_round_biased(dst: Expr, dst_size: usize, src: Expr, src_size: usize, id: &str) -> Expr {
    let end_label = b_label(&format!("end_rnd_{id}"));
    let rem_var = b_var(&format!("rem_var_{id}"));
    let rem_size = src_size - dst_size;
    let threshold = 1 << (rem_size * 8 - 1);
    cs_mline(vec![
        e_copy(
            b_local(rem_var.clone(), rem_size),
            b_size(src.clone(), rem_size),
        ),
        e_copy(dst.clone(), b_trunc(src, rem_size)),
        b_ifgoto(
            e_eq(dst.clone(), b_num((1 << (8 * dst_size - 1)) - 1)),
            end_label.clone(),
        ),
        b_ifgoto(e_lt(rem_var, b_num(threshold)), end_label.clone()),
        cs_assign_by(e_add, dst, b_num(1)),
        end_label,
    ])
}

pub fn cs_max(dst: Expr, src0: Expr, src1: Expr, id: &str) -> Expr {
    cs_mline(vec![
        e_copy(dst.clone(), src0),
        b_ifgoto(
            e_ges(dst.clone(), src1.clone()),
            b_label(&format!("max_end_{}", id)),
        ),
        e_copy(dst, src1),
        b_label(&format!("max_end_{}", id)),
    ])
}

pub fn cs_min(dst: Expr, src0: Expr, src1: Expr, id: &str) -> Expr {
    cs_mline(vec![
        e_copy(dst.clone(), src0),
        b_ifgoto(
            e_les(dst.clone(), src1.clone()),
            b_label(&format!("min_end_{}", id)),
        ),
        e_copy(dst, src1),
        b_label(&format!("min_end_{}", id)),
    ])
}

pub fn cs_abs_sat(dst: Expr, src: Expr, size: usize, id: &str) -> Expr {
    cs_mline(vec![
        e_copy(dst.clone(), src.clone()),
        b_ifgoto(
            e_ges(dst.clone(), b_num(0)),
            b_label(&format!("abs_end_{}", id)),
        ),
        e_copy(dst.clone(), e_neg(src.clone())),
        b_ifgoto(
            e_ne(src, b_num(1 << (size * 8 - 1))),
            b_label(&format!("abs_end_{}", id)),
        ),
        e_copy(dst.clone(), b_num((1 << (size * 8 - 1)) - 1)),
        b_label(&format!("abs_end_{}", id)),
    ])
}

pub fn cs_neg_sat(dst: Expr, src: Expr, sat: bool, size: usize, id: &str) -> Expr {
    let mut code = vec![e_copy(dst.clone(), e_neg(src.clone()))];
    if sat {
        code.push(b_ifgoto(
            e_ne(src, b_num(1 << (size * 8 - 1))),
            b_label(&format!("neg_end_{}", id)),
        ));
        code.push(e_copy(dst.clone(), b_num((1 << (size * 8 - 1)) - 1)));
        code.push(b_label(&format!("neg_end_{}", id)))
    }
    cs_mline(code)
}
//// Code snippet
