#[allow(non_snake_case)]
pub(crate) fn isWhitespace(c: char) -> bool {
    matches!(c, ' ' | '\n' | '\r' | '\t')
}
