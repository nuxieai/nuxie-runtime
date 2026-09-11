//! Specified corner axes. Percentage values remain unresolved for live layout.
use crate::Diagnostic;
use cssparser::{Parser, ParserInput, Token};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RadiusValue {
    Pixels(f32),
    Em(f32),
    Rem(f32),
    Percent(f32),
}

fn invalid(source: &str) -> Diagnostic {
    Diagnostic::new("unsupported-corner-radius", source,
        "Expected nonnegative px/em/rem lengths, percentages or unitless zero; math expressions are not supported")
}

fn axis(token: &Token<'_>, spelling: &str, source: &str) -> Result<RadiusValue, Diagnostic> {
    let (value, result) = match token {
        Token::Number { value, .. } if *value == 0. => (*value, RadiusValue::Pixels(*value)),
        Token::Percentage { .. } => {
            // Read percentage points directly: multiplying cssparser's rounded
            // fraction by 100 introduces an avoidable second rounding step.
            let value: f32 = spelling.strip_suffix('%').ok_or_else(|| invalid(source))?
                .parse().map_err(|_| invalid(source))?;
            (value, RadiusValue::Percent(value))
        },
        Token::Dimension { value, unit, .. } => {
            let result = if unit.eq_ignore_ascii_case("px") { RadiusValue::Pixels(*value) }
                else if unit.eq_ignore_ascii_case("em") { RadiusValue::Em(*value) }
                else if unit.eq_ignore_ascii_case("rem") { RadiusValue::Rem(*value) }
                else { return Err(invalid(source)); };
            (*value, result)
        }
        _ => return Err(invalid(source)),
    };
    if value.is_finite() && value >= 0. { Ok(result) } else { Err(invalid(source)) }
}

fn next<'i>(parser: &mut Parser<'i, '_>, source: &str) -> Result<(Token<'i>, &'i str), Diagnostic> {
    loop {
        let start = parser.position();
        let token = parser.next_including_whitespace_and_comments().map_err(|_| invalid(source))?.clone();
        if matches!(token, Token::WhiteSpace(_) | Token::Comment(_)) { continue; }
        return Ok((token, parser.slice_from(start)));
    }
}

fn expand(values: &[RadiusValue], source: &str) -> Result<[RadiusValue;4], Diagnostic> {
    Ok(match values {
        [a] => [*a;4], [a,b] => [*a,*b,*a,*b],
        [a,b,c] => [*a,*b,*c,*b], [a,b,c,d] => [*a,*b,*c,*d],
        _ => return Err(invalid(source)),
    })
}

/// TL/TR/BR/BL, with horizontal then vertical axes. CSS-wide keywords are
/// handled by the cascade, not this component-value parser.
pub(crate) fn shorthand(value: &str, source: &str) -> Result<[[RadiusValue;2];4], Diagnostic> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let mut lists = [Vec::new(), Vec::new()];
    let mut slash = false;
    while !parser.is_exhausted() {
        let (token, spelling) = next(&mut parser, source)?;
        if matches!(token, Token::Delim('/')) {
            if slash || lists[0].is_empty() { return Err(invalid(source)); }
            slash = true;
            continue;
        }
        let list = &mut lists[usize::from(slash)];
        if list.len() == 4 { return Err(invalid(source)); }
        list.push(axis(&token, spelling, source)?);
    }
    let horizontal = expand(&lists[0], source)?;
    let vertical = if slash { expand(&lists[1], source)? } else { horizontal };
    Ok(std::array::from_fn(|i| [horizontal[i], vertical[i]]))
}

pub(crate) fn longhand(value: &str, source: &str) -> Result<[RadiusValue;2], Diagnostic> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let (token, spelling) = next(&mut parser, source)?;
    let first = axis(&token, spelling, source)?;
    let second = if parser.is_exhausted() { first }
        else { let (token, spelling) = next(&mut parser, source)?; axis(&token, spelling, source)? };
    parser.expect_exhausted().map_err(|_| invalid(source))?;
    Ok([first, second])
}

#[cfg(test)]
mod tests {
    use super::{shorthand, longhand, RadiusValue::*};

    #[test]
    fn independent_slash_lists_expand_in_physical_corner_order() {
        assert_eq!(shorthand("2em 1rem 4px / 50% 3px", "test").unwrap(),
            [[Em(2.),Percent(50.)],[Rem(1.),Pixels(3.)],[Pixels(4.),Percent(50.)],[Rem(1.),Pixels(3.)]]);
        assert_eq!(shorthand("1px 2px 3px 4px", "test").unwrap(),
            [[Pixels(1.);2],[Pixels(2.);2],[Pixels(3.);2],[Pixels(4.);2]]);
        assert_eq!(shorthand("50%", "test").unwrap(), [[Percent(50.);2];4]);
    }

    #[test]
    fn longhands_preserve_axis_units_and_zero() {
        assert_eq!(longhand("0 25%", "test").unwrap(), [Pixels(0.),Percent(25.)]);
        assert_eq!(longhand(".5EM/**/2REM", "test").unwrap(), [Em(0.5),Rem(2.)]);
        assert_eq!(longhand("8px", "test").unwrap(), [Pixels(8.);2]);
        for spelling in ["30%", "30.0%", "3e1%", "+30%", "/**/30%/**/"] {
            assert_eq!(longhand(spelling, "test").unwrap(), [Percent(30.);2]);
        }
        assert_eq!(longhand(" 30.125% /**/ 60%", "test").unwrap(), [Percent(30.125),Percent(60.)]);
    }

    #[test]
    fn malformed_unsupported_and_nonfinite_values_reject() {
        for value in ["", "/ 2px", "2px /", "2px / 3px / 4px", "1px 2px 3px 4px 5px",
            "1px / 1px 2px 3px 4px 5px", "-1px", "-2%", "2", "2vw", "1e999px",
            "calc(2px + 1px)", "1px,2px", "none", "[2px]"] {
            assert!(shorthand(value,"test").is_err(), "{value}");
        }
        for value in ["", "1px 2px 3px", "1px / 2px", "-1em", "calc(10%)"] {
            assert!(longhand(value,"test").is_err(), "{value}");
        }
    }
}
