pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn quote(s: &str) -> String {
    format!("\"{s}\"")
}

pub fn mask_hex(val: u16, len: usize) -> String {
    let hex_lex = ((len as f32) / 4.0).ceil() as usize;
    format!("{val:#0len$x}", len = hex_lex)
}
