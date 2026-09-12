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
    pub flex: NumericFlex,
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
            flex: NumericFlex::default(),
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

/// Authored independent factors; these must not be replaced by native linked
/// grow/shrink values when a participant is lowered to Fill.
#[derive(Clone, Debug)]
pub(crate) struct NumericFlex {
    pub grow: Scalar,
    pub shrink: Scalar,
    pub basis: NumericSize,
}
impl Default for NumericFlex {
    fn default() -> Self {
        Self {
            grow: exact(0.),
            shrink: exact(0.),
            basis: NumericSize::Auto,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum FlexTokenKind {
    Number,
    Dimension,
    Percent,
    Ident(String),
}
struct FlexToken {
    text: String,
    kind: FlexTokenKind,
}
fn flex_tokens(text: &str) -> Result<Vec<FlexToken>, Unresolved> {
    let mut input = ParserInput::new(text);
    let mut parser = Parser::new(&mut input);
    let mut out = Vec::new();
    while !parser.is_exhausted() {
        parser.skip_whitespace();
        let start = parser.position();
        let kind = match parser.next().map_err(|_| Unresolved::OriginalSyntax)? {
            Token::Number { .. } => FlexTokenKind::Number,
            Token::Dimension { .. } => FlexTokenKind::Dimension,
            Token::Percentage { .. } => FlexTokenKind::Percent,
            Token::Ident(v) => FlexTokenKind::Ident(v.to_ascii_lowercase()),
            _ => return Err(Unresolved::OriginalSyntax),
        };
        out.push(FlexToken {
            text: parser.slice_from(start).to_owned(),
            kind,
        });
        if out.len() > 3 {
            return Err(Unresolved::OriginalSyntax);
        }
    }
    Ok(out)
}
impl NumericFlex {
    pub(super) fn apply(
        &mut self,
        d: &ResolvedDeclaration,
        native: super::flex::Flex,
        parent: &Self,
        font: &Scalar,
    ) {
        // Native admission has already parsed the declaration. A metadata parse
        // failure only makes the fields touched by this declaration unresolved.
        let tokens = flex_tokens(&d.value);
        let originals = d
            .original
            .as_ref()
            .ok_or(Unresolved::MissingOriginal)
            .and_then(|s| flex_tokens(s));
        let result = (|| {
            let tokens = tokens.as_ref().map_err(Clone::clone)?;
            let keyword = if tokens.len() == 1 {
                match &tokens[0].kind {
                    FlexTokenKind::Ident(k) => Some(k.as_str()),
                    _ => None,
                }
            } else {
                None
            };
            if keyword == Some("inherit") {
                return Ok(parent.clone());
            }
            if matches!(keyword, Some("initial" | "unset")) {
                return Ok(Self {
                    grow: exact(0.),
                    shrink: exact(1.),
                    basis: NumericSize::Auto,
                });
            }
            if keyword == Some("none") {
                return Ok(Self::default());
            }
            if d.name == "flex" && keyword == Some("auto") {
                return Ok(Self {
                    grow: exact(1.),
                    shrink: exact(1.),
                    basis: NumericSize::Auto,
                });
            }
            let original = |index: usize| -> Result<&str, Unresolved> {
                let originals = originals.as_ref().map_err(Clone::clone)?;
                if originals.len() != tokens.len() || originals[index].kind != tokens[index].kind {
                    return Err(Unresolved::UnitMismatch);
                }
                Ok(originals[index].text.as_str())
            };
            let factor = |index: usize, value: f32| -> Scalar {
                let (number, _) = original_number(Some(original(index)?))?;
                ScalarProvenance::from_decimal(&number, value).map_err(Into::into)
            };
            let basis = |index: usize| -> NumericSize {
                if matches!(native.basis, Size::Auto) {
                    return NumericSize::Auto;
                }
                if tokens[index].kind == FlexTokenKind::Number {
                    return NumericSize::Pixels(factor(
                        index,
                        match native.basis {
                            Size::Pixels(v) => v,
                            _ => return NumericSize::Pixels(Err(Unresolved::UnitMismatch)),
                        },
                    ));
                }
                let mut declaration = d.native.clone();
                declaration.value = tokens[index].text.clone();
                let token = ResolvedDeclaration {
                    native: declaration,
                    original: original(index)
                        .ok()
                        .map(|s| std::sync::Arc::new(s.to_owned())),
                };
                dimension(&token, native.basis, &parent.basis, font)
            };
            if d.name == "flex-grow" {
                return Ok(Self {
                    grow: factor(0, native.grow),
                    ..self.clone()
                });
            }
            if d.name == "flex-shrink" {
                return Ok(Self {
                    shrink: factor(0, native.shrink),
                    ..self.clone()
                });
            }
            if d.name == "flex-basis" {
                return Ok(Self {
                    basis: basis(0),
                    ..self.clone()
                });
            }
            let number = |i: usize| tokens[i].kind == FlexTokenKind::Number;
            let (grow, shrink, basis_index) = match tokens.len() {
                1 if number(0) => (Some(0), None, None),
                1 => (None, None, Some(0)),
                2 if number(0) && number(1) => (Some(0), Some(1), None),
                2 if number(0) => (Some(0), None, Some(1)),
                2 => (Some(1), None, Some(0)),
                3 if number(0) && number(1) => (Some(0), Some(1), Some(2)),
                3 => (Some(1), Some(2), Some(0)),
                _ => return Err(Unresolved::NativeSyntax),
            };
            Ok(Self {
                grow: grow.map_or_else(|| exact(1.), |i| factor(i, native.grow)),
                shrink: shrink.map_or_else(|| exact(1.), |i| factor(i, native.shrink)),
                basis: basis_index.map_or_else(|| NumericSize::Percent(exact(0.)), basis),
            })
        })();
        let resolved = result.unwrap_or_else(|error: Unresolved| Self {
            grow: Err(error.clone()),
            shrink: Err(error.clone()),
            basis: match native.basis {
                Size::Auto => NumericSize::Auto,
                Size::Pixels(_) => NumericSize::Pixels(Err(error)),
                Size::Percent(_) => NumericSize::Percent(Err(error)),
            },
        });
        match d.name.as_str() {
            "flex-grow" => self.grow = resolved.grow,
            "flex-shrink" => self.shrink = resolved.shrink,
            "flex-basis" => self.basis = resolved.basis,
            _ => *self = resolved,
        }
    }
}

#[cfg(test)]
mod flex_tests {
    use super::super::{computed, Style};
    use super::*;
    fn style(css: &str, parent: &Style) -> Style {
        let document = scraper::Html::parse_document("<div id=a></div>");
        let selector = scraper::Selector::parse("#a").unwrap();
        computed(
            document.select(&selector).next().unwrap(),
            &crate::css::stylesheet(css).unwrap(),
            parent,
            true,
            false,
        )
        .unwrap()
    }
    fn basis(flex: &NumericFlex) -> &ScalarProvenance {
        match &flex.basis {
            NumericSize::Pixels(Ok(v)) | NumericSize::Percent(Ok(v)) => v,
            _ => panic!("missing basis"),
        }
    }
    fn ideal(value: &ScalarProvenance, want: f64) {
        assert!(
            value.ideal_bounds().lower() <= want && value.ideal_bounds().upper() >= want,
            "{value:?} does not enclose {want}"
        );
    }
    #[test]
    fn shorthand_slots_variables_and_longhand_overrides_preserve_independent_ideals() {
        for text in [
            "1.00000001 1.00000002 2.125em",
            "2.125em 1.00000001 1.00000002",
            "1.00000001/**/1.00000002/**/2.125em",
        ] {
            let value = style(
                &format!("#a{{--f:{text};flex:var(--f);font-size:20px}}"),
                &Style::default(),
            );
            let flex = &value.numeric.flex;
            let g = flex.grow.as_ref().unwrap();
            let s = flex.shrink.as_ref().unwrap();
            assert_eq!(g.native().to_bits(), s.native().to_bits());
            assert!(!g.proves_equal(s));
            ideal(g, 1.00000001);
            ideal(s, 1.00000002);
            ideal(basis(flex), 42.5);
        }
        let value = style(
            "#a{flex:2 .25 3em;flex-grow:.123456789123;flex-basis:2rem;font-size:20px}",
            &Style::default(),
        );
        ideal(value.numeric.flex.grow.as_ref().unwrap(), 0.123456789123);
        ideal(value.numeric.flex.shrink.as_ref().unwrap(), 0.25);
        ideal(basis(&value.numeric.flex), 32.);
    }
    #[test]
    fn resets_inheritance_and_omitted_basis_have_distinct_typed_values() {
        let parent = style("#a{flex:2 .25 3em;font-size:20px}", &Style::default());
        let child = style("#a{flex:inherit;font-size:80px}", &parent);
        assert!(basis(&child.numeric.flex).proves_equal(basis(&parent.numeric.flex)));
        ideal(basis(&child.numeric.flex), 60.);
        let child = style(
            "#a{flex:none;flex-grow:inherit;flex-shrink:initial;flex-basis:inherit}",
            &parent,
        );
        ideal(child.numeric.flex.grow.as_ref().unwrap(), 2.);
        ideal(child.numeric.flex.shrink.as_ref().unwrap(), 1.);
        ideal(basis(&child.numeric.flex), 60.);
        for text in ["2", "2 1"] {
            let v = style(&format!("#a{{flex:{text}}}"), &parent);
            assert!(matches!(v.numeric.flex.basis, NumericSize::Percent(_)));
            assert!(basis(&v.numeric.flex).is_exact_zero());
        }
        for text in ["none", "initial", "unset", "auto"] {
            let v = style(&format!("#a{{flex:{text}}}"), &parent);
            assert!(matches!(v.numeric.flex.basis, NumericSize::Auto));
            assert_eq!(v.numeric.flex.grow.as_ref().unwrap().native(), v.flex.grow);
            assert_eq!(
                v.numeric.flex.shrink.as_ref().unwrap().native(),
                v.flex.shrink
            );
        }
    }
    #[test]
    fn underflowed_authored_basis_is_not_proved_exact_zero_and_missing_metadata_stays_missing() {
        let value = style("#a{flex:2 7 1e-50}", &Style::default());
        assert_eq!(basis(&value.numeric.flex).native(), 0.);
        assert!(!basis(&value.numeric.flex).is_exact_zero());
        for token in ["-0", "+0", "0e0"] {
            let v = style(&format!("#a{{flex:2 7 {token}}}"), &Style::default());
            assert!(basis(&v.numeric.flex).is_exact_zero());
        }
        let mut parent = Style::default();
        parent.variables.insert(
            "--lost".into(),
            Some(crate::variables::ResolvedValue {
                native: "2 .25 12px".into(),
                original: None,
            }),
        );
        let v = style("#a{flex:var(--lost,1 1 99px)}", &parent);
        assert_eq!(v.flex.grow, 2.);
        assert!(matches!(
            v.numeric.flex.grow,
            Err(Unresolved::MissingOriginal)
        ));
        assert!(matches!(
            v.numeric.flex.basis,
            NumericSize::Pixels(Err(Unresolved::MissingOriginal))
        ));
    }
}
