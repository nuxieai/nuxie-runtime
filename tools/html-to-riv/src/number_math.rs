//! Typed CSS math with dimensionless results. Range/representability policy belongs to the caller.
use cssparser::{ParseError, Parser, Token};
type Result<'i> = std::result::Result<Quantity, ParseError<'i, ()>>;

#[cfg(test)]
pub(crate) fn function<'i>(p: &mut Parser<'i, '_>, name: &str) -> std::result::Result<f64, ParseError<'i, ()>> {
    function_with_font_size(p, name, 16.0)
}

pub(crate) fn function_with_font_size<'i>(p: &mut Parser<'i, '_>, name: &str, font_size: f64) -> std::result::Result<f64, ParseError<'i, ()>> {
    let result = Evaluator { remaining: 256, font_size }.function(p, name, 0)?;
    if result.dimensions != [0; 6] { return Err(p.new_custom_error(())); }
    Ok(result.value)
}

#[derive(Clone, Copy)]
struct Quantity { value: f64, dimensions: [i16; 6] }
impl Quantity {
    fn number(value: f64) -> Self { Self { value, dimensions: [0; 6] } }
    fn unit(value: f64, axis: usize) -> Self {
        let mut result = Self::number(value); result.dimensions[axis] = 1; result
    }
}

// Canonical CSS units: px, deg, seconds, Hz, dppx, percentage.
// Font-relative lengths use the separately supplied computed-style context.
pub(crate) fn absolute_unit(unit: &str) -> Option<(usize, f64)> {
    Some(match unit.to_ascii_lowercase().as_str() {
        "px" => (0, 1.0), "in" => (0, 96.0), "cm" => (0, 96.0 / 2.54),
        "mm" => (0, 96.0 / 25.4), "q" => (0, 96.0 / 101.6),
        "pt" => (0, 96.0 / 72.0), "pc" => (0, 16.0),
        "deg" => (1, 1.0), "grad" => (1, 0.9), "rad" => (1, 180.0 / std::f64::consts::PI), "turn" => (1, 360.0),
        "s" => (2, 1.0), "ms" => (2, 0.001),
        "hz" => (3, 1.0), "khz" => (3, 1000.0),
        "dppx" | "x" => (4, 1.0), "dpi" => (4, 1.0 / 96.0), "dpcm" => (4, 2.54 / 96.0),
        _ => return None,
    })
}

pub(crate) fn font_unit(unit: &str, font_size: f64) -> Option<(usize, f64)> {
    if unit.eq_ignore_ascii_case("em") { Some((0, font_size)) }
    // The authoring fragment cannot restyle the host html element.
    else if unit.eq_ignore_ascii_case("rem") { Some((0, 16.0)) }
    else { None }
}

// Read the numeric prefix of a validated dimension token, retaining double
// precision even for escaped units. An e without exponent digits starts a unit.
fn numeric_prefix(text: &str) -> Option<f64> {
    let bytes = text.as_bytes(); let mut end = 0;
    if matches!(bytes.first(), Some(b'+' | b'-')) { end += 1; }
    while bytes.get(end).is_some_and(u8::is_ascii_digit) { end += 1; }
    if bytes.get(end) == Some(&b'.') {
        end += 1; while bytes.get(end).is_some_and(u8::is_ascii_digit) { end += 1; }
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let mut exponent = end + 1;
        if matches!(bytes.get(exponent), Some(b'+' | b'-')) { exponent += 1; }
        let digits = exponent;
        while bytes.get(exponent).is_some_and(u8::is_ascii_digit) { exponent += 1; }
        if exponent > digits { end = exponent; }
    }
    text[..end].parse::<f64>().ok().map(|n| n.clamp(-(f32::MAX as f64), f32::MAX as f64))
}

struct Evaluator { remaining: usize, font_size: f64 }
impl Evaluator {
    fn function<'i>(&mut self, p: &mut Parser<'i, '_>, name: &str, depth: usize) -> Result<'i> {
        if depth >= 32 { return Err(p.new_custom_error(())); }
        let name = name.to_ascii_lowercase();
        if !matches!(name.as_str(), "calc" | "min" | "max" | "clamp") {
            return Err(p.new_custom_error(()));
        }
        p.parse_nested_block(|p| {
            if name == "calc" { return self.sum(p, depth + 1); }
            let values = p.parse_comma_separated(|p| self.sum(p, depth + 1))?;
            if values.is_empty() || (name == "clamp" && values.len() != 3) {
                return Err(p.new_custom_error(()));
            }
            let dimensions = values[0].dimensions;
            if values.iter().any(|v| v.dimensions != dimensions) { return Err(p.new_custom_error(())); }
            // Type checking precedes NaN propagation, including typed zero.
            let value = if values.iter().any(|v| v.value.is_nan()) { f64::NAN }
            else { match name.as_str() {
                "min" => values.iter().map(|v| v.value).fold(f64::INFINITY, f64::min),
                "max" => values.iter().map(|v| v.value).fold(f64::NEG_INFINITY, f64::max),
                _ => values[0].value.max(values[1].value.min(values[2].value)),
            }};
            Ok(Quantity { value, dimensions })
        })
    }

    fn sum<'i>(&mut self, p: &mut Parser<'i, '_>, depth: usize) -> Result<'i> {
        let mut value = self.product(p, depth)?;
        loop {
            let state = p.state();
            let mut whitespace = false;
            let op = loop {
                match p.next_including_whitespace() {
                    Ok(Token::WhiteSpace(_)) => whitespace = true,
                    Ok(Token::Delim(op @ ('+' | '-'))) => break Some(*op),
                    _ => break None,
                }
            };
            let Some(op) = op else { p.reset(&state); break; };
            if !whitespace || !matches!(p.next_including_whitespace(), Ok(Token::WhiteSpace(_))) {
                return Err(p.new_custom_error(()));
            }
            let right = self.product(p, depth)?;
            if value.dimensions != right.dimensions { return Err(p.new_custom_error(())); }
            value.value = if op == '+' { value.value + right.value } else { value.value - right.value };
        }
        Ok(value)
    }

    fn product<'i>(&mut self, p: &mut Parser<'i, '_>, depth: usize) -> Result<'i> {
        let mut value = self.value(p, depth)?;
        loop {
            let state = p.state();
            let op = match p.next() {
                Ok(Token::Delim(op @ ('*' | '/'))) => *op,
                _ => { p.reset(&state); break; }
            };
            let right = self.value(p, depth)?;
            let typed_divisor = right.dimensions != [0; 6];
            for axis in 0..6 { value.dimensions[axis] += if op == '*' { right.dimensions[axis] } else { -right.dimensions[axis] }; }
            value.value = if op == '*' { value.value * right.value }
            else if typed_divisor {
                // Chrome represents typed division as multiplication by an
                // inverse node. Preserve its observable underflow behavior.
                value.value * (1.0 / right.value)
            } else { value.value / right.value };
        }
        Ok(value)
    }

    fn value<'i>(&mut self, p: &mut Parser<'i, '_>, depth: usize) -> Result<'i> {
        if depth >= 32 || self.remaining == 0 { return Err(p.new_custom_error(())); }
        self.remaining -= 1;
        p.skip_whitespace();
        let start = p.position();
        match p.next()?.clone() {
            // cssparser's cached numeric value is f32; evaluating that loses
            // cancellation and turns finite double literals into zero/infinity.
            // Tokens retain double precision within the f32 range; the infinity keyword
            // retains IEEE infinity. Chrome distinguishes 1e400 from infinity.
            Token::Number { .. } => numeric_prefix(p.slice_from(start)).map(Quantity::number)
                .ok_or_else(|| p.new_custom_error(())),
            Token::Dimension { unit, .. } => {
                let (axis, factor) = absolute_unit(&unit).or_else(|| font_unit(&unit, self.font_size))
                    .ok_or_else(|| p.new_custom_error(()))?;
                let value = numeric_prefix(p.slice_from(start)).ok_or_else(|| p.new_custom_error(()))?;
                Ok(Quantity::unit(value * factor, axis))
            },
            Token::Percentage { .. } => numeric_prefix(p.slice_from(start)).map(|n| Quantity::unit(n, 5))
                .ok_or_else(|| p.new_custom_error(())),
            Token::Ident(name) if name.eq_ignore_ascii_case("infinity") => Ok(Quantity::number(f64::INFINITY)),
            Token::Ident(name) if name.eq_ignore_ascii_case("-infinity") => Ok(Quantity::number(f64::NEG_INFINITY)),
            Token::Ident(name) if name.eq_ignore_ascii_case("NaN") => Ok(Quantity::number(f64::NAN)),
            Token::Ident(name) if name.eq_ignore_ascii_case("pi") => Ok(Quantity::number(std::f64::consts::PI)),
            Token::Ident(name) if name.eq_ignore_ascii_case("e") => Ok(Quantity::number(std::f64::consts::E)),
            Token::Function(name) => self.function(p, &name, depth),
            Token::ParenthesisBlock => p.parse_nested_block(|p| self.sum(p, depth + 1)),
            _ => Err(p.new_custom_error(())),
        }
    }
}

#[cfg(test)]
mod tests {
    use cssparser::{Parser, ParserInput, Token};
    fn evaluate(value: &str) -> Option<f64> {
        let mut input = ParserInput::new(value);
        let mut p = Parser::new(&mut input);
        let Token::Function(name) = p.next().ok()?.clone() else { return None; };
        let value = super::function(&mut p, &name).ok()?;
        p.expect_exhausted().ok()?;
        Some(value)
    }
    #[test]
    fn precedence_functions_and_unclamped_intermediate_values() {
        for (s, expected) in [("calc(2 + 3 * 4)",14.),("calc((2 + 3) * 4)",20.),
            ("calc(8 / 2 / 2)",2.),("calc(1 - 3)",-2.),("min(3, calc(8 / 4))",2.),
            ("max(-3, -2)",-2.),("clamp(4, 2, 1)",4.),("CALC(2 + 1)",3.),
            ("calc(2/**/ + /**/3)",5.)] {
            assert_eq!(evaluate(s),Some(expected),"{s}");
        }
    }
    #[test]
    fn retains_double_precision_literals_before_arithmetic() {
        for expression in ["calc(16777217 - 16777216)", "calc(100000001 - 100000000)",
            "calc(1e40 / 1e40)", "calc(1e-50 / 1e-50)", "calc(1e400 / 1e400)"] {
            assert_eq!(evaluate(expression), Some(1.0), "{expression}");
        }
    }

    #[test]
    fn chrome_token_range_and_typed_reciprocal_order() {
        for (expression, expected) in [("calc(1e40 / 1e39)",1.0),
            ("calc(1e40 - 1e39)",0.0), ("calc(1e308in / 1e308in)",1.0),
            ("calc(1e308in / 1e308px)",96.0),
            ("calc(1e308px / 1e308in)",1.0 / 96.0),
            ("calc(1e-320ms / 1e-320ms)",f64::INFINITY),
            ("calc(1e-320 / 1e-320)",1.0)] {
            let actual=evaluate(expression).unwrap();
            assert!(actual == expected || (actual-expected).abs() < 1e-12, "{expression}: {actual} vs {expected}");
        }
    }

    #[test]
    fn cancels_absolute_units_and_preserves_typed_zero() {
        for (expression, expected) in [("calc(1px / 1px)", 1.0),
            ("calc(1in / 1px)", 96.0), ("calc(1s / 500ms)", 2.0),
            ("calc(1turn / 180deg)", 2.0), ("calc(1% / 1%)", 1.0),
            ("calc(1px * 1s / 1px / 1s)", 1.0),
            ("calc(min(1in, 100px) / 1px)", 96.0)] {
            assert_eq!(evaluate(expression), Some(expected), "{expression}");
        }
        for expression in ["calc(1px)", "calc(0 * 1px)", "calc(1px - 1px)",
            "calc(1px + 1)", "calc(1px / 1s)", "clamp(1, 2px, 3)"] {
            assert_eq!(evaluate(expression), None, "{expression}");
        }
    }

    #[test]
    fn rejects_invalid_types_arity_and_operator_whitespace() {
        for s in ["calc(1px)","calc(50%)","calc(1+2)","calc(1 +2)","calc(1+ 2)",
            "calc(1/**/+/**/2)","calc(1 + )","min()","clamp(1,2)","clamp(1,2,3,4)",
            "calc(2,3)","calc(2) junk","calc(sin(1))","calc(--1)"] {
            assert!(evaluate(s).is_none(),"{s}");
        }
    }
    #[test]
    fn preserves_nonfinite_results_for_property_range_policy() {
        assert_eq!(evaluate("calc(1 / 0)"),Some(f64::INFINITY));
        assert!(evaluate("calc(0 / 0)").unwrap().is_nan());
        assert!(evaluate("min(NaN, 2)").unwrap().is_nan());
        assert!(evaluate("clamp(1, NaN, 2)").unwrap().is_nan());
        assert_eq!(evaluate("calc(infinity * -1)"),Some(f64::NEG_INFINITY));
    }
    #[test]
    fn bounds_recursive_and_flat_expression_work() {
        let nested = format!("{}1{}", "calc(".repeat(40), ")".repeat(40));
        assert!(evaluate(&nested).is_none());
        let flat = format!("calc({})",vec!["1";257].join(" + "));
        assert!(evaluate(&flat).is_none());
        assert_eq!(evaluate(&format!("calc({})",vec!["1";64].join(" + "))),Some(64.));
    }
}
