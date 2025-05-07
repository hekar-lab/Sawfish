use crate::slaspec::globals::DEFAULT_MEM;

use super::text::Text;

pub fn p_op(code: Text) -> Text {
    "\n\t" + code + ";"
}

pub fn p_macro(mac: &str) -> Text {
    Text::from(format!("{mac}()"))
}

pub fn p_local(var: Text, size: usize) -> Text {
    "local " + var + format!(":{size}")
}

pub fn p_copy(dst: Text, src: Text) -> Text {
    dst + " = " + src
}

pub fn p_ptr(size: usize, addr: Text) -> Text {
    p_ptr_mem(DEFAULT_MEM, size, addr)
}

pub fn p_ptr_mem(mem: &str, size: usize, addr: Text) -> Text {
    format!("*[{mem}]:{size} ") + addr
}

pub fn p_return(addr: Text) -> Text {
    "return [" + addr + "]"
}
