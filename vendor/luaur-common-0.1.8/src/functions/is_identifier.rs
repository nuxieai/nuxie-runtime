#[allow(non_snake_case)]
pub fn isIdentifier(s: &str) -> bool {
    s.chars()
        .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
}
