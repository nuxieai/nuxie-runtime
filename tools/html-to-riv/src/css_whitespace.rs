//! CSS whitespace is narrower than Rust's Unicode or ASCII whitespace classes.
//! Other characters remain part of the value and must reach token validation.

pub(crate) fn is_whitespace(value: char) -> bool {
    matches!(value, '\t' | '\n' | '\u{c}' | '\r' | ' ')
}

pub(crate) fn trim(text: &str) -> &str {
    text.trim_matches(is_whitespace)
}

pub(crate) fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(is_whitespace).filter(|word| !word.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_matches_css_token_boundaries_including_vertical_tab_control() {
        use cssparser::{Parser, ParserInput, Token};
        for separator in ['\t', '\n', '\u{c}', '\r', ' ', '\u{b}', '\u{85}',
            '\u{a0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}',
            '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}',
            '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}', '\u{205f}', '\u{3000}',
            '\u{200b}', '\u{feff}'] {
            let raw = separator.to_string();
            let mut input = ParserInput::new(&raw);
            let mut parser = Parser::new(&mut input);
            let token_is_space = matches!(parser.next_including_whitespace_and_comments(), Ok(Token::WhiteSpace(_)));
            assert_eq!(is_whitespace(separator), token_is_space, "{separator:?}");
            let text = format!("{separator}auto{separator}0{separator}");
            if token_is_space {
                assert_eq!(trim(&text), format!("auto{separator}0"));
                assert_eq!(words(&text).collect::<Vec<_>>(), ["auto", "0"]);
            } else {
                assert_eq!(trim(&text), text);
                assert_eq!(words(&text).collect::<Vec<_>>(), [text.as_str()]);
            }
        }
    }
}
