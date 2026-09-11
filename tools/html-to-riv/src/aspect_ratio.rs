//! Parsed preferred ratios retain `auto`: combined syntax uses a different
//! ratio box and may select a replaced element's natural ratio.
use crate::Diagnostic;
use cssparser::{Parser, ParserInput, Token};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AspectRatio {
    pub auto: bool,
    pub ratio: Option<f32>,
    pub pair: Option<[u32; 2]>,
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self { auto: true, ratio: None, pair: None }
    }
}

impl AspectRatio {
    pub fn parse(value: &str, source: &str) -> Result<Self, Diagnostic> {
        Self::parse_with_font_size(value, source, 16.0)
    }

    pub fn parse_with_font_size(value: &str, source: &str, font_size: f32) -> Result<Self, Diagnostic> {
        let invalid = || Diagnostic::new("unsupported-aspect-ratio", source,
            "Expected auto and/or a nonnegative numeric ratio representable by the runtime");
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        let mut auto = parser.try_parse(|p| p.expect_ident_matching("auto")).is_ok();
        if auto && parser.is_exhausted() {
            return Ok(Self::default());
        }
        let number = |parser: &mut Parser<'_, '_>| {
            parser.skip_whitespace();
            let start = parser.position();
            if let Ok(name) = parser.try_parse(|p| {
                match p.next()?.clone() {
                    Token::Function(name) => Ok(name),
                    _ => Err(p.new_custom_error::<_, ()>(())),
                }
            }) {
                let computed = crate::number_math::function_with_font_size(parser, &name, font_size as f64).map_err(|_| invalid())?;
                if !parser.slice_from(start).ends_with(')') { return Err(invalid()); }
                // Range checking applies per top-level ratio component, after
                // intermediate arithmetic. CSS NaN resolves to zero here.
                let computed = if computed.is_nan() { 0.0 } else { computed.max(0.0) };
                let value = computed.min(f32::MAX as f64) as f32;
                return Ok(value);
            }
            parser.expect_number().map_err(|_| invalid())?;
            // Match Chrome's double-to-float conversion rather than the
            // tokenizer's cached decimal-to-float rounding at halfway values.
            let literal = parser.slice_from(start).parse::<f64>().map_err(|_| invalid())?;
            let value = literal.min(f32::MAX as f64) as f32;
            let significant = parser.slice_from(start).split(['e', 'E']).next().unwrap();
            if value == 0.0 && significant.starts_with('-') && significant.bytes().any(|b| matches!(b, b'1'..=b'9')) {
                return Err(invalid());
            }
            Ok(value)
        };
        let numerator = number(&mut parser)?;
        let denominator = if parser.try_parse(|p| p.expect_delim('/')).is_ok() {
            number(&mut parser)?
        } else { 1.0 };
        if !auto {
            auto = parser.try_parse(|p| p.expect_ident_matching("auto")).is_ok();
        }
        parser.expect_exhausted().map_err(|_| invalid())?;
        if !numerator.is_finite() || !denominator.is_finite() || numerator < 0.0 || denominator < 0.0 {
            return Err(invalid());
        }
        // Chromium converts each computed component through gfx::SizeF before
        // dividing. Match its f32 rounding and small-size clamp for literals
        // and math alike. See validation/aspect-ratio-math-research.md.
        let clamp_component = |value: f32| if value > 8.0 * f32::EPSILON { value } else { 0.0 };
        let numerator = clamp_component(numerator);
        let denominator = clamp_component(denominator);
        // CSS degenerate ratios behave as auto; they are not a zero-sized box.
        let pair = if numerator == 0.0 || denominator == 0.0 {
            None
        } else {
            layout_ratio(numerator, denominator)
        };
        let ratio = pair.map(|p| p[0] as f32 / p[1] as f32);
        Ok(Self { auto, ratio, pair })
    }
}

// Chromium retains a 26.6 fixed-point pair for layout. Preserve its lossless
// path and continued-fraction stopping rules. Retain the integer pair for CSS
// runtime layout; the wire scalar alone cannot preserve large derived sizes. Source and browser boundary evidence: aspect-ratio-math-research.md.
fn layout_ratio(numerator: f32, denominator: f32) -> Option<[u32; 2]> {
    let raw = [(numerator * 64.0) as i32, (denominator * 64.0) as i32];
    let reduced = |pair: [i32; 2]| {
        if pair[0] <= 0 || pair[1] <= 0 { None }
        else {
            let (mut a, mut b) = (pair[0] as u32, pair[1] as u32);
            while b != 0 { (a, b) = (b, a % b); }
            Some([pair[0] as u32 / a, pair[1] as u32 / a])
        }
    };
    if raw[0] as f32 / 64.0 == numerator && raw[1] as f32 / 64.0 == denominator {
        return reduced(raw);
    }
    if numerator == denominator { return Some([1, 1]); }
    let target = numerator / denominator;
    let mut remainder = target;
    let mut previous = [0_i32, 1_i32];
    let mut current = [1_i32, 0_i32];
    for _ in 0..16 {
        if !remainder.is_finite() ||
            (target - current[0] as f32 / current[1] as f32).abs() < 0.000001 {
            break;
        }
        let whole = remainder.floor() as i32;
        let next = [0, 1].map(|axis| current[axis].saturating_mul(whole).saturating_add(previous[axis]));
        if next.contains(&i32::MAX) { break; }
        previous = current;
        current = next;
        remainder = 1.0 / (remainder - whole as f32);
    }
    reduced(current).or_else(|| reduced(raw))
}

#[cfg(test)]
mod tests {
    use super::AspectRatio;

    #[test]
    fn retains_auto_and_accepts_numeric_ratio_grammar() {
        for value in ["2", "2 / 1", "4/2", "2e0", "+2", "2/**/ / /**/1"] {
            assert_eq!(AspectRatio::parse(value, "test").unwrap(), AspectRatio { auto: false, ratio: Some(2.0), pair: Some([2, 1]) }, "{value}");
        }
        for value in ["auto 2", "2 auto", "AUTO 4 / 2", "4/2/**/auto"] {
            assert_eq!(AspectRatio::parse(value, "test").unwrap(), AspectRatio { auto: true, ratio: Some(2.0), pair: Some([2, 1]) }, "{value}");
        }
        assert_eq!(AspectRatio::parse("auto", "test").unwrap(), AspectRatio::default());
        assert_eq!(AspectRatio::parse(".5 / .25", "test").unwrap().ratio, Some(2.0));
    }

    #[test]
    fn degenerate_ratios_have_no_preferred_ratio() {
        for value in ["0", "0/1", "1/0", "0/0", "-0/2", "auto 0/0", "0e50"] {
            assert_eq!(AspectRatio::parse(value, "test").unwrap().ratio, None, "{value}");
        }
    }

    #[test]
    fn rejects_invalid_syntax_and_unrepresentable_ratios() {
        for value in ["", "none", "-1", "1/-2", "1px", "20%", "1/", "/2", "1 2", "auto auto", "auto 2 auto", "1/2/3", "1 / auto", "2,1", "NaN", "infinity", "-1e-100"] {
            assert!(AspectRatio::parse(value, "test").is_err(), "{value}");
        }
    }
}
