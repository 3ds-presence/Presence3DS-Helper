use codepage_437::CP437_WINGDINGS;

pub fn encode_cp437(input: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    for c in input.chars() {
        if c.is_ascii() {
            out.push(c as u8);
        } else {
            out.push(CP437_WINGDINGS.encode(c).unwrap_or(b'?'));
        }
    }
    out
}
