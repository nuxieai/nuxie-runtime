#[allow(non_snake_case)]
pub fn is_identifier_start_char(c: char) -> bool {
    (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || c == '_'
}
