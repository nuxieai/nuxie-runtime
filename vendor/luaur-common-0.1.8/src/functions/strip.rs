use crate::functions::is_whitespace::isWhitespace;

#[allow(non_snake_case)]
pub fn strip(mut s: &str) -> &str {
    while !s.is_empty() && isWhitespace(s.chars().next().unwrap()) {
        s = &s[1..];
    }

    while !s.is_empty() && isWhitespace(s.chars().next_back().unwrap()) {
        s = &s[..s.len() - 1];
    }

    s
}
