//! Preserve authored numeric coefficients while normalizing decoded CSS units.
use cssparser::{Token, ToCss, serialize_identifier, serialize_name};

/// The caller supplies a tokenizer-validated numeric token's exact source slice.
/// Decoded unit length cannot identify this boundary because units may be escaped.
pub(crate) fn number_prefix(raw: &str) -> &str {
    let bytes = raw.as_bytes();
    let mut end = usize::from(bytes.first().is_some_and(|b| matches!(b, b'+' | b'-')));
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
    }
    if bytes.get(end).is_some_and(|b| matches!(b, b'e' | b'E')) {
        let mut exponent = end + 1;
        if bytes.get(exponent).is_some_and(|b| matches!(b, b'+' | b'-')) {
            exponent += 1;
        }
        let digits = exponent;
        while bytes.get(exponent).is_some_and(u8::is_ascii_digit) {
            exponent += 1;
        }
        if exponent > digits {
            end = exponent;
        }
    }
    &raw[..end]
}

/// Serialize one token without rounding numeric text through cssparser's float
/// formatter. Callers still validate finite values and preserve token separators.
pub(crate) fn serialize(token: &Token<'_>, raw: &str) -> String {
    let mut text = match token {
        Token::Number { .. } | Token::Percentage { .. } | Token::Dimension { .. } => {
            number_prefix(raw).to_owned()
        }
        _ => return token.to_css_string(),
    };
    match token {
        Token::Percentage { .. } => text.push('%'),
        Token::Dimension { unit, .. } => {
            let bytes = unit.as_bytes();
            // An escaped e/E followed by digits can become scientific notation
            // when decoded: 1\65 2 must remain a dimension with unit "e2".
            // Also protect a bare e/E or signed suffix at a token boundary.
            let exponent_like = matches!(bytes.first(), Some(b'e' | b'E'))
                && (bytes.len() == 1
                    || bytes[1].is_ascii_digit()
                    || matches!(bytes[1], b'+' | b'-'));
            if exponent_like {
                text.push_str(if bytes[0] == b'e' { "\\65 " } else { "\\45 " });
                serialize_name(&unit[1..], &mut text).expect("writing to String");
            } else {
                serialize_identifier(unit, &mut text).expect("writing to String");
            }
        }
        _ => {}
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use cssparser::{Parser, ParserInput};

    #[test]
    fn numeric_tokens_roundtrip_without_changing_type_value_or_integer_spelling() {
        let numbers = ["0", "-0", "+0", "1.0", "1e2", "1E-2", "+000001.2500000",
            "-.03125", "999999.875", "999999.9375", "62499.9921875",
            "100.71428680419922", "2.34567890123", "2147483648", "1e-50"];
        let units = ["", "%", "px", "em", "rem", r"p\78", r"\65 m", r"\45 M",
            r"\65 2", r"\45 2px", r"\65 -2", r"\65 \+2", r"\65 ",
            r"\45 -", r"\32 px", r"\2d 2", r"\3bb", r"\1 px"];
        for number in numbers {
            for unit in units {
                let raw = format!("{number}{unit}");
                let mut input = ParserInput::new(&raw);
                let mut parser = Parser::new(&mut input);
                let token = parser.next().unwrap().clone();
                assert!(parser.is_exhausted(), "{raw}");
                assert_eq!(number_prefix(&raw), number, "{raw}");
                let normalized = serialize(&token, &raw);
                let mut input2 = ParserInput::new(&normalized);
                let mut parser2 = Parser::new(&mut input2);
                let again = parser2.next().unwrap().clone();
                assert!(parser2.is_exhausted(), "{raw} -> {normalized}");
                assert_eq!(token, again, "{raw} -> {normalized}");
                let bits = |token: &Token<'_>| match token {
                    Token::Number { value, .. } | Token::Dimension { value, .. } => value.to_bits(),
                    Token::Percentage { unit_value, .. } => unit_value.to_bits(),
                    _ => panic!("expected numeric token"),
                };
                assert_eq!(bits(&token), bits(&again), "{raw}: signed zero/value bits");
                assert_eq!(serialize(&again, &normalized), normalized, "{raw}");
                if unit.is_empty() { assert_eq!(normalized, number); }
            }
        }
    }
}
