//! Typed metadata updated beside native computation. Failures affect metadata
//! only; they never change native CSS admission or substitute native as ideal.
use super::scalar_provenance::{ScalarError, ScalarProvenance};
use super::{ResolvedDeclaration, Size, SpecifiedSize, ROOT_FONT_SIZE};
use cssparser::{Parser, ParserInput, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Unresolved {
    MissingOriginal,
    OriginalSyntax,
    NativeSyntax,
    UnitMismatch,
    Scalar(ScalarError),
}
impl From<ScalarError> for Unresolved {
    fn from(value: ScalarError) -> Self {
        Self::Scalar(value)
    }
}
pub(crate) type Scalar = Result<ScalarProvenance, Unresolved>;
#[allow(dead_code)] // Computed carriers are consumed by the next numerical descriptor stage.
#[derive(Clone, Debug)]
pub(crate) enum NumericSize {
    Auto,
    Pixels(Scalar),
    Percent(Scalar),
}
#[derive(Clone, Debug)]
pub(crate) struct NumericStyle {
    pub font: Scalar,
    pub width: NumericSize,
    pub height: NumericSize,
    pub min_width: NumericSize,
    pub min_height: NumericSize,
    pub max_width: NumericSize,
    pub max_height: NumericSize,
}
fn exact(value: f32) -> Scalar {
    ScalarProvenance::exact_constant(value).map_err(Into::into)
}
impl Default for NumericStyle {
    fn default() -> Self {
        Self {
            font: exact(ROOT_FONT_SIZE),
            width: NumericSize::Auto,
            height: NumericSize::Auto,
            min_width: NumericSize::Pixels(exact(0.)),
            min_height: NumericSize::Pixels(exact(0.)),
            max_width: NumericSize::Auto,
            max_height: NumericSize::Auto,
        }
    }
}
impl NumericStyle {
    pub fn host() -> Self {
        Self {
            width: NumericSize::Percent(exact(100.)),
            height: NumericSize::Percent(exact(100.)),
            ..Self::default()
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Unit {
    Pixel,
    Percent,
    Em,
    Rem,
}
fn original_number(text: Option<&str>) -> Result<(String, Unit), Unresolved> {
    let text = text.ok_or(Unresolved::MissingOriginal)?;
    let mut input = ParserInput::new(text);
    let mut parser = Parser::new(&mut input);
    parser.skip_whitespace();
    let start = parser.position();
    let token = parser
        .next()
        .map_err(|_| Unresolved::OriginalSyntax)?
        .clone();
    let unit = match token {
        Token::Number { .. } => Unit::Pixel,
        Token::Percentage { .. } => Unit::Percent,
        Token::Dimension { unit, .. } => match unit.to_ascii_lowercase().as_str() {
            "px" => Unit::Pixel,
            "em" => Unit::Em,
            "rem" => Unit::Rem,
            _ => return Err(Unresolved::OriginalSyntax),
        },
        _ => return Err(Unresolved::OriginalSyntax),
    };
    let raw = parser.slice_from(start);
    // The CSS tokenizer has already validated this numeric token. Extract its
    // numeric prefix without depending on decoded/possibly escaped unit length.
    let bytes = raw.as_bytes();
    let mut end = 0;
    if bytes.first().is_some_and(|b| matches!(b, b'+' | b'-')) {
        end += 1;
    }
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
        if bytes
            .get(exponent)
            .is_some_and(|b| matches!(b, b'+' | b'-'))
        {
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
    let number = raw[..end].to_owned();
    parser
        .expect_exhausted()
        .map_err(|_| Unresolved::OriginalSyntax)?;
    Ok((number, unit))
}
fn coefficient(declaration: &ResolvedDeclaration) -> Result<(ScalarProvenance, Unit), Unresolved> {
    // Reuse the actual native parser on its normalized input for coefficient
    // bits. This input is NEVER used to reconstruct original decimal intent.
    let native = super::size(&declaration.value, &declaration.source)
        .map_err(|_| Unresolved::NativeSyntax)?;
    let (value, native_unit) = match native {
        SpecifiedSize::Pixels(v) => (v, Unit::Pixel),
        SpecifiedSize::Percent(v) => (v, Unit::Percent),
        SpecifiedSize::Em(v) => (v, Unit::Em),
        SpecifiedSize::Rem(v) => (v, Unit::Rem),
        SpecifiedSize::Auto => return Err(Unresolved::NativeSyntax),
    };
    let (original, unit) = original_number(declaration.original.as_deref().map(String::as_str))?;
    if unit != native_unit {
        return Err(Unresolved::UnitMismatch);
    }
    Ok((ScalarProvenance::from_decimal(&original, value)?, unit))
}
fn known(value: &Scalar) -> Result<&ScalarProvenance, Unresolved> {
    value.as_ref().map_err(Clone::clone)
}
pub(super) fn font(declaration: &ResolvedDeclaration, native: f32, parent: &Scalar) -> Scalar {
    let value = declaration.value.trim().to_ascii_lowercase();
    let result = (|| {
        if matches!(value.as_str(), "inherit" | "unset") {
            return Ok(known(parent)?.clone());
        }
        if value == "initial" {
            return exact(ROOT_FONT_SIZE);
        }
        let (coefficient, unit) = coefficient(declaration)?;
        Ok(match unit {
            Unit::Pixel => coefficient,
            Unit::Em => coefficient.multiply(known(parent)?)?,
            Unit::Rem => coefficient.multiply(known(&exact(ROOT_FONT_SIZE))?)?,
            Unit::Percent => coefficient.font_percent(known(parent)?)?,
        })
    })();
    result.and_then(|value| value.with_native(native).map_err(Into::into))
}
pub(super) fn dimension(
    declaration: &ResolvedDeclaration,
    native: Size,
    parent: &NumericSize,
    font: &Scalar,
) -> NumericSize {
    if declaration.value.trim().eq_ignore_ascii_case("inherit") {
        return parent.clone();
    }
    if matches!(native, Size::Auto) {
        return NumericSize::Auto;
    }
    let value = (|| {
        let (coefficient, unit) = coefficient(declaration)?;
        let calculated = match unit {
            Unit::Pixel | Unit::Percent => coefficient,
            Unit::Em => coefficient.multiply(known(font)?)?,
            Unit::Rem => coefficient.multiply(known(&exact(ROOT_FONT_SIZE))?)?,
        };
        let value = match native {
            Size::Pixels(v) | Size::Percent(v) => v,
            Size::Auto => unreachable!(),
        };
        calculated.with_native(value).map_err(Into::into)
    })();
    match native {
        Size::Pixels(_) => NumericSize::Pixels(value),
        Size::Percent(_) => NumericSize::Percent(value),
        Size::Auto => NumericSize::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::super::{computed, Style};
    use super::*;
    fn style(css: &str, parent: &Style) -> Style {
        let document = scraper::Html::parse_document("<div id=a></div>");
        let selector = scraper::Selector::parse("#a").unwrap();
        let element = document.select(&selector).next().unwrap();
        let rules = crate::css::stylesheet(css).unwrap();
        computed(element, &rules, parent, false, true).unwrap()
    }
    fn pixels(value: &NumericSize) -> &ScalarProvenance {
        match value {
            NumericSize::Pixels(Ok(value)) => value,
            _ => panic!("expected resolved pixels: {value:?}"),
        }
    }
    fn percent(value: &NumericSize) -> &ScalarProvenance {
        match value {
            NumericSize::Percent(Ok(value)) => value,
            _ => panic!("expected resolved percent: {value:?}"),
        }
    }
    fn encloses(value: &ScalarProvenance, ideal: f64) {
        assert!(
            value.ideal_bounds().lower() <= ideal && value.ideal_bounds().upper() >= ideal,
            "{value:?} must enclose {ideal}"
        );
    }
    #[test]
    fn original_decimal_survives_native_rounding_and_selected_variable_fallback() {
        let value = style(
            "#a{--v:100.71428680419922px;width:var(--v);height:var(--absent,2.34567890123%)}",
            &Style::default(),
        );
        let width = pixels(&value.numeric.width);
        encloses(width, 100.71428680419922);
        assert_eq!(width.native().to_bits(), 100.714f32.to_bits());
        assert!(width.absolute_error_upper() > 0.0002);
        encloses(percent(&value.numeric.height), 2.34567890123);
    }
    #[test]
    fn font_order_relative_units_and_computed_inheritance_keep_typed_meaning() {
        let parent = style(
            "#a{width:2.125em;height:33.333333333%;font-size:125%}",
            &Style::default(),
        );
        encloses(parent.numeric.font.as_ref().unwrap(), 20.);
        encloses(pixels(&parent.numeric.width), 42.5);
        let child = style(
            "#a{font-size:150%;width:inherit;height:inherit;min-width:2em;max-width:2rem}",
            &parent,
        );
        encloses(child.numeric.font.as_ref().unwrap(), 30.);
        assert!(pixels(&child.numeric.width).proves_equal(pixels(&parent.numeric.width)));
        assert!(percent(&child.numeric.height).proves_equal(percent(&parent.numeric.height)));
        encloses(pixels(&child.numeric.min_width), 60.);
        encloses(pixels(&child.numeric.max_width), 32.);
    }
    #[test]
    fn css_wide_defaults_and_unavailable_originals_do_not_invent_numbers() {
        let mut parent = style("#a{font-size:23px;width:17px}", &Style::default());
        parent.variables.insert(
            "--lost".into(),
            Some(crate::variables::ResolvedValue {
                native: "12px".into(),
                original: None,
            }),
        );
        let child=style("#a{font-size:unset;width:var(--lost,99px);height:initial;min-width:initial;max-width:none}",&parent);
        assert!(child
            .numeric
            .font
            .as_ref()
            .unwrap()
            .proves_equal(parent.numeric.font.as_ref().unwrap()));
        assert!(matches!(child.width, Size::Pixels(12.)));
        assert!(matches!(
            child.numeric.width,
            NumericSize::Pixels(Err(Unresolved::MissingOriginal))
        ));
        assert!(matches!(child.numeric.height, NumericSize::Auto));
        assert!(matches!(child.numeric.min_width, NumericSize::Auto));
        assert!(matches!(child.numeric.max_width, NumericSize::Auto));
        let initial = style("#a{font-size:initial}", &parent);
        encloses(initial.numeric.font.as_ref().unwrap(), 16.);
        assert!(pixels(&initial.numeric.min_width).is_exact_zero());
    }
    #[test]
    fn escaped_units_exponents_and_missing_font_propagate_without_native_changes() {
        let escaped = style(r"#a{width:1.25e+2p\78;height:1.5EM}", &Style::default());
        encloses(pixels(&escaped.numeric.width), 125.);
        encloses(pixels(&escaped.numeric.height), 24.);
        let mut parent = Style::default();
        parent.variables.insert(
            "--lost".into(),
            Some(crate::variables::ResolvedValue {
                native: "20px".into(),
                original: None,
            }),
        );
        let child = style("#a{font-size:var(--lost);width:2em;height:2rem}", &parent);
        assert!(matches!(child.width, Size::Pixels(40.)));
        assert!(matches!(
            child.numeric.width,
            NumericSize::Pixels(Err(Unresolved::MissingOriginal))
        ));
        encloses(pixels(&child.numeric.height), 32.);
    }
}
