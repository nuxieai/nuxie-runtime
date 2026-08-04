#[allow(non_snake_case)]
#[inline]
pub fn is_space(ch: char) -> bool {
    ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n' || ch == '\x0b' || ch == '\x0c'
}
