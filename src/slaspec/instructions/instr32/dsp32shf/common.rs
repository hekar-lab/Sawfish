use crate::slaspec::instructions::expr::Expr;
use crate::slaspec::instructions::expr_util::*;

fn get_shift(out_var: &str, nbits: usize) -> Expr {
    cs_mline(vec![
        e_copy(
            b_var(out_var),
            e_bit_and(b_num((1 << nbits) - 1), e_rfield("src1")),
        ),
        cs_assign_by(e_lshft, b_var(out_var), b_num(16 - nbits as i128)),
        cs_assign_by(e_arshft, b_var(out_var), b_num(16 - nbits as i128)),
    ])
}

pub fn shift(dst: Expr, src: Expr, size: usize, arithm: bool, sat: bool, id: &str) -> Expr {
    let shift_var = &format!("shift_{}", id);
    let res_var = &format!("shft_res_{}", id);
    let lshft_lab = &format!("lshift_{}", id);
    let rshft_lab = &format!("rshift_{}", id);
    let end_lab = &format!("end_shift_{}", id);
    let nbits = ((size * 8) as f64).log2().floor() as usize + 1;

    let mut code = vec![
        e_local(res_var, size),
        e_local(shift_var, 2),
        get_shift(shift_var, nbits),
        b_ifgoto(e_gts(b_var(shift_var), b_num(0)), b_label(rshft_lab)),
        b_ifgoto(e_lts(b_var(shift_var), b_num(0)), b_label(lshft_lab)),
        e_copy(dst.clone(), src.clone()),
        b_goto(b_label(end_lab)),
    ];

    code.push(b_label(rshft_lab));

    if arithm {
        code.push(e_copy(
            b_var(res_var),
            e_arshft(src.clone(), b_var(shift_var)),
        ));
    } else {
        code.push(e_copy(
            b_var(res_var),
            e_rshft(src.clone(), b_var(shift_var)),
        ));
    }

    code.push(e_copy(dst.clone(), b_var(res_var)));
    code.push(b_goto(b_label(end_lab)));
    code.push(b_label(lshft_lab));
    code.push(e_copy(b_var(shift_var), e_neg(b_var(shift_var))));

    if arithm & sat {
        let buf_var = &format!("shft_buf_{}", id);
        code.push(e_local(buf_var, size * 2));
        code.push(e_copy(
            b_var(buf_var),
            e_lshft(e_sext(src.clone()), b_var(shift_var)),
        ));
        code.push(cs_strunc_sat(
            b_var(res_var),
            b_var(buf_var),
            size,
            &format!("shift_{id}"),
        ));
    } else {
        code.push(e_copy(
            b_var(res_var),
            e_lshft(src.clone(), b_var(shift_var)),
        ));
    }

    code.push(e_copy(dst.clone(), b_var(res_var)));
    code.push(b_label(end_lab));

    cs_mline(code)
}

pub fn rot(dst: Expr, src: Expr, size: usize, id: &str) -> Expr {
    let shftrot_var = &format!("rot_{}", id);
    let res_var = &format!("rot_res_{}", id);
    let bit_var = &format!("cc_bit_{}", id);
    let lrot_lab = &format!("lrot_{}", id);
    let rrot_lab = &format!("rrot_{}", id);
    let end_lab = &format!("end_rot_{}", id);
    let nbits = size * 8;
    let nbits_shft = (nbits as f64).log2().floor() as usize + 1;

    let mut code = vec![
        e_local(res_var, size),
        e_local(bit_var, size),
        get_shift(shftrot_var, nbits_shft),
        b_ifgoto(e_gts(b_var(shftrot_var), b_num(0)), b_label(rrot_lab)),
        b_ifgoto(e_lts(b_var(shftrot_var), b_num(0)), b_label(lrot_lab)),
        e_copy(dst.clone(), src.clone()),
        b_goto(b_label(end_lab)),
    ];

    code.push(b_label(rrot_lab));

    code.push(e_copy(
        b_var(bit_var),
        e_bit_and(
            b_num(1),
            b_grp(e_rshft(
                src.clone(),
                b_grp(e_sub(b_num(nbits as i128), b_var(shftrot_var))),
            )),
        ),
    ));
    code.push(e_copy(
        b_var(res_var),
        e_bit_or(
            b_grp(e_lshft(src.clone(), b_var(shftrot_var))),
            e_bit_or(
                b_grp(e_lshft(
                    e_zext(b_reg("CC")),
                    b_grp(e_sub(b_var(shftrot_var), b_num(1))),
                )),
                b_grp(e_rshft(
                    src.clone(),
                    b_grp(e_sub(b_num(nbits as i128 + 1), b_var(shftrot_var))),
                )),
            ),
        ),
    ));
    code.push(e_copy(b_reg("CC"), b_size(b_var(bit_var), 1)));

    code.push(e_copy(dst.clone(), b_var(res_var)));
    code.push(b_goto(b_label(end_lab)));
    code.push(b_label(lrot_lab));
    code.push(e_copy(b_var(shftrot_var), e_neg(b_var(shftrot_var))));

    code.push(e_copy(
        b_var(bit_var),
        e_bit_and(
            b_num(1),
            b_grp(e_rshft(
                src.clone(),
                b_grp(e_sub(b_var(shftrot_var), b_num(1))),
            )),
        ),
    ));
    code.push(e_copy(
        b_var(res_var),
        e_bit_or(
            b_grp(e_lshft(
                src.clone(),
                b_grp(e_sub(b_num(nbits as i128 + 1), b_var(shftrot_var))),
            )),
            e_bit_or(
                b_grp(e_lshft(
                    e_zext(b_reg("CC")),
                    b_grp(e_sub(b_num(nbits as i128), b_var(shftrot_var))),
                )),
                b_grp(e_rshft(src.clone(), b_var(shftrot_var))),
            ),
        ),
    ));
    code.push(e_copy(b_reg("CC"), b_size(b_var(bit_var), 1)));

    code.push(e_copy(dst.clone(), b_var(res_var)));
    code.push(b_label(end_lab));

    cs_mline(code)
}
