pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn mask_hex(val: u16, len: usize) -> String {
    let mut hex_len = ((len as f32) / 4.0).ceil() as usize;
    hex_len += 2; // Add 2 to account for the "0x"
    format!("{val:#0len$x}", len = hex_len)
}
