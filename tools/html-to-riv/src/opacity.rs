//! Opacity component parser. Public admission awaits isolated group rendering.
use crate::Diagnostic;
use cssparser::{Parser, ParserInput, Token};

/// CSS-wide keywords and substitution belong to the cascade, not this parser.
pub(crate) fn parse(value: &str, source: &str) -> Result<f32, Diagnostic> {
    let invalid = || Diagnostic::new("unsupported-opacity", source,
        "Expected one finite number or percentage; opacity math is not supported");
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let (spelling, percent) = loop {
        let start = parser.position();
        match parser.next_including_whitespace_and_comments().map_err(|_| invalid())? {
            Token::WhiteSpace(_) | Token::Comment(_) => continue,
            Token::Number { .. } => break (parser.slice_from(start), false),
            Token::Percentage { .. } => break (parser.slice_from(start), true),
            _ => return Err(invalid()),
        }
    };
    if !parser.is_exhausted() { return Err(invalid()); }
    // Parse before narrowing so large finite authored values can clamp safely.
    let number = if percent { &spelling[..spelling.len() - 1] } else { spelling };
    let number: f64 = number.parse().map_err(|_| invalid())?;
    if !number.is_finite() { return Err(invalid()); }
    let normalized = if percent { number / 100. } else { number };
    let result = normalized.clamp(0., 1.) as f32;
    Ok(if result == 0. { 0. } else { result })
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn numbers_percentages_and_comments() {
        for (input, expected) in [(".5", 0.5), ("50%", 0.5), ("+2.5e1%", 0.25),
            (" /* lead */ .125 /* tail */ ", 0.125), ("1e-2", 0.01)] {
            assert_eq!(parse(input, "opacity").unwrap(), expected, "{input}");
        }
    }

    #[test]
    fn clamps_before_narrowing_and_normalizes_zero() {
        for input in ["-1", "-200%", "-1e100", "-0", "-0%"] {
            assert_eq!(parse(input, "opacity").unwrap().to_bits(), 0f32.to_bits());
        }
        for input in ["2", "200%", "1e100", "1e100%"] {
            assert_eq!(parse(input, "opacity").unwrap(), 1.);
        }
    }

    #[test]
    fn rejects_extra_tokens_units_math_and_nonfinite_values() {
        for input in ["", "/* empty */", "0 1", "50 %", "1px", "NaN", "infinity",
            "1e999", "-1e999%", "calc(.5)", "var(--alpha)", "inherit", ".5,", "--1"] {
            let diagnostic = parse(input, "#card opacity").unwrap_err();
            assert_eq!(diagnostic.code, "unsupported-opacity", "{input}");
        }
    }
}
