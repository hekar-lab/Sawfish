use crate::slaspec::globals::DEFAULT_MEM;

pub fn p_field(field: String) -> String {
    format!("{{{field}}}")
}

pub fn p_macro(mac: String) -> String {
    format!("{mac}()")
}

pub fn p_local(var: String, size: usize) -> String {
    format!("local {var}:{size}")
}

pub fn p_copy(dst: String, src: String) -> String {
    format!("{dst} = {src}")
}

pub fn p_ptr(size: usize, addr: String) -> String {
    p_ptr_mem(DEFAULT_MEM, size, addr)
}

pub fn p_ptr_mem(mem: &str, size: usize, addr: String) -> String {
    format!("*[{mem}]:{size} {addr}")
}

pub fn p_return(addr: String) -> String {
    format!("return [{}]", addr)
}
