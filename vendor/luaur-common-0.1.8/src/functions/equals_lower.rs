#[allow(non_snake_case)]
pub fn equalsLower(lhs: &[u8], rhs: &[u8]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }

    for i in 0..lhs.len() {
        if (lhs[i] as char).to_ascii_lowercase() != (rhs[i] as char).to_ascii_lowercase() {
            return false;
        }
    }

    true
}
