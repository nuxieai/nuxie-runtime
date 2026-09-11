//! Border authoring values. Parsing preserves specified lengths; resolution
//! uses the final font size and the pinned Chrome profile's CSS width snapping.
use crate::Diagnostic;
use cssparser::{Parser, ParserInput, Token, ToCss};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BorderStyle { None, Hidden, Solid }

impl BorderStyle {
    pub fn parse(value: &str, source: &str) -> Result<Self, Diagnostic> {
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        let invalid = || Diagnostic::new("unsupported-border-style", source,
            "Expected a uniform none, hidden or solid border style");
        let name = parser.expect_ident().map_err(|_| invalid())?;
        let style = if name.eq_ignore_ascii_case("none") { Self::None }
            else if name.eq_ignore_ascii_case("hidden") { Self::Hidden }
            else if name.eq_ignore_ascii_case("solid") { Self::Solid }
            else { return Err(invalid()); };
        parser.expect_exhausted().map_err(|_| invalid())?;
        Ok(style)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BorderColor { Current, Rgba(u32) }

impl BorderColor {
    pub fn parse(value: &str, source: &str) -> Result<Self, Diagnostic> {
        if value.trim().eq_ignore_ascii_case("currentcolor") { Ok(Self::Current) }
        else { crate::color(value, source).map(Self::Rgba) }
    }

    pub fn resolve(self, current: u32) -> u32 {
        match self { Self::Current => current, Self::Rgba(value) => value }
    }
}

/// Return the physical edge and component for supported side declarations.
pub(crate) fn side_property(name: &str) -> Option<(usize, &str)> {
    for (index, side) in ["top", "right", "bottom", "left"].into_iter().enumerate() {
        if let Some(component) = name.strip_prefix(&format!("border-{side}")) {
            return match component {
                "" => Some((index, "border")), "-width" => Some((index, "border-width")),
                "-style" => Some((index, "border-style")), "-color" => Some((index, "border-color")),
                _ => None,
            };
        }
    }
    None
}

/// Expand one through four physical values in CSS top/right/bottom/left order.
/// Keep function arguments together: whitespace inside rgb()/hsl() is not a side separator.
/// Admission remains gated by the style/runtime transport until P02 is integrated.
pub(crate) fn parse_side_values<T: Copy>(
    value: &str, source: &str, parse: impl Fn(&str, &str) -> Result<T, Diagnostic>,
) -> Result<[T; 4], Diagnostic> {
    let invalid = || Diagnostic::new("unsupported-border-values", source,
        "Expected one through four supported physical border values");
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let mut values = Vec::new();
    while !parser.is_exhausted() {
        if values.len() == 4 { return Err(invalid()); }
        parser.skip_whitespace();
        let start = parser.position();
        let token = parser.next().map_err(|_| invalid())?.clone();
        if matches!(token, Token::Function(_)) {
            parser.parse_nested_block(|nested| {
                while !nested.is_exhausted() {
                    let token = nested.next()?;
                    if token.is_parse_error() || matches!(token, Token::Function(_) |
                        Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock) {
                        return Err(nested.new_custom_error::<_, ()>(()));
                    }
                }
                Ok(())
            }).map_err(|_| invalid())?;
            if !parser.slice_from(start).ends_with(')') { return Err(invalid()); }
        }
        values.push(parse(parser.slice_from(start), source)?);
    }
    match values.as_slice() {
        [a] => Ok([*a; 4]),
        [a,b] => Ok([*a,*b,*a,*b]),
        [a,b,c] => Ok([*a,*b,*c,*b]),
        [a,b,c,d] => Ok([*a,*b,*c,*d]),
        _ => Err(invalid()),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Border {
    pub width: BorderWidth,
    pub style: BorderStyle,
    pub color: BorderColor,
}

impl Border {
    /// Initial shorthand defaults differ from the authoring reset `border:0`.
    pub fn parse(value: &str, source: &str) -> Result<Self, Diagnostic> {
        let invalid = || Diagnostic::new("unsupported-border", source,
            "Expected one optional border width, none/hidden/solid style and supported color");
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        let (mut width, mut style, mut color) = (None, None, None);
        while !parser.is_exhausted() {
            let token = parser.next().map_err(|_| invalid())?.clone();
            let component = match token {
                Token::Ident(name) => name.to_string(),
                Token::Function(name) if ["rgb", "rgba", "hsl", "hsla"].iter()
                    .any(|function| name.eq_ignore_ascii_case(function)) => {
                    let start = parser.position();
                    let body = parser.parse_nested_block(|nested| {
                        let mut body = String::new();
                        while !nested.is_exhausted() {
                            let token = nested.next_including_whitespace_and_comments()?.clone();
                            if token.is_parse_error() || matches!(token, Token::Function(_) |
                                Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock) {
                                return Err(nested.new_custom_error::<_, ()>(()));
                            }
                            if matches!(token, Token::Comment(_)) { body.push(' '); }
                            else { body.push_str(&token.to_css_string()); }
                        }
                        Ok(body)
                    }).map_err(|_| invalid())?;
                    if !parser.slice_from(start).ends_with(')') { return Err(invalid()); }
                    format!("{}({body})", name.to_ascii_lowercase())
                }
                token => token.to_css_string(),
            };
            if let Ok(parsed) = BorderWidth::parse(&component, source) {
                if width.replace(parsed).is_some() { return Err(invalid()); }
            } else if let Ok(parsed) = BorderStyle::parse(&component, source) {
                if style.replace(parsed).is_some() { return Err(invalid()); }
            } else {
                let parsed = if component.eq_ignore_ascii_case("currentcolor") { BorderColor::Current }
                    else { BorderColor::Rgba(crate::color(&component, source).map_err(|_| invalid())?) };
                if color.replace(parsed).is_some() { return Err(invalid()); }
            }
        }
        if width.is_none() && style.is_none() && color.is_none() { return Err(invalid()); }
        Ok(Self { width: width.unwrap_or(BorderWidth::Medium),
            style: style.unwrap_or(BorderStyle::None), color: color.unwrap_or(BorderColor::Current) })
    }

    pub fn used_width(self, font_size: f32, source: &str) -> Result<f32, Diagnostic> {
        if self.style == BorderStyle::Solid { self.width.resolve(font_size, source) }
        else { Ok(0.0) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BorderWidth {
    Px(f32),
    Em(f32),
    Rem(f32),
    Thin,
    Medium,
    Thick,
}

impl BorderWidth {
    pub fn resolve(self, font_size: f32, source: &str) -> Result<f32, Diagnostic> {
        let pixels = match self {
            Self::Px(value) => value,
            Self::Em(value) => value * font_size,
            Self::Rem(value) => value * 16.0,
            Self::Thin => 1.0,
            Self::Medium => 3.0,
            Self::Thick => 5.0,
        };
        if !pixels.is_finite() || pixels < 0.0 {
            return Err(Diagnostic::new("unsupported-border-width", source,
                "Resolved border width must be finite and nonnegative"));
        }
        // Chrome153 at page zoom1 computes this in CSS pixels, including when
        // deviceScaleFactor is2. Do not divide by the renderer's backing scale.
        Ok(if pixels == 0.0 { 0.0 } else { pixels.floor().max(1.0) })
    }

    pub fn parse(value: &str, source: &str) -> Result<Self, Diagnostic> {
        let invalid = || Diagnostic::new("unsupported-border-width", source,
            "Expected a finite nonnegative px/em/rem length, zero, thin, medium or thick");
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        let width = match parser.next().map_err(|_| invalid())?.clone() {
            Token::Number { value: 0.0, .. } => Self::Px(0.0),
            Token::Dimension { value, unit, .. } if value.is_finite() && value >= 0.0 => {
                if unit.eq_ignore_ascii_case("px") { Self::Px(value) }
                else if unit.eq_ignore_ascii_case("em") { Self::Em(value) }
                else if unit.eq_ignore_ascii_case("rem") { Self::Rem(value) }
                else { return Err(invalid()); }
            }
            Token::Ident(name) if name.eq_ignore_ascii_case("thin") => Self::Thin,
            Token::Ident(name) if name.eq_ignore_ascii_case("medium") => Self::Medium,
            Token::Ident(name) if name.eq_ignore_ascii_case("thick") => Self::Thick,
            _ => return Err(invalid()),
        };
        if !parser.is_exhausted() { return Err(invalid()); }
        Ok(width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_border_lists_expand_in_css_order() {
        for (value, expected) in [("1px", [1.,1.,1.,1.]), ("1px 2px", [1.,2.,1.,2.]),
            ("1px 2px 3px", [1.,2.,3.,2.]), ("1px 2px 3px 4px", [1.,2.,3.,4.])] {
            let actual = parse_side_values(value, "test", BorderWidth::parse).unwrap();
            assert_eq!(actual, expected.map(BorderWidth::Px));
        }
        let colors = parse_side_values("rgb(18 52 86)/**/currentColor transparent hsl(0 100% 50%)", "test", BorderColor::parse).unwrap();
        assert_eq!(colors, [BorderColor::Rgba(0xff123456), BorderColor::Current, BorderColor::Rgba(0), BorderColor::Rgba(0xffff0000)]);
        assert_eq!(parse_side_values("solid none hidden", "test", BorderStyle::parse).unwrap(),
            [BorderStyle::Solid, BorderStyle::None, BorderStyle::Hidden, BorderStyle::None]);
    }

    #[test]
    fn physical_border_lists_reject_malformed_or_unsupported_components() {
        for value in ["", "1px 2px 3px 4px 5px", "-1px", "2%", "1px, 2px", "calc(2px)", "1px / 2px"] {
            assert!(parse_side_values(value, "test", BorderWidth::parse).is_err(), "{value}");
        }
        for value in ["rgb(1 2 3", "red blue green black white", "red / blue", "var(--x)"] {
            assert!(parse_side_values(value, "test", BorderColor::parse).is_err(), "{value}");
        }
    }

    fn style(values: &[(&str, &str)], parent: &crate::style::Style) -> crate::style::Style {
        let declarations = values.iter().map(|(name, value)| crate::css::Declaration {
            name: (*name).into(), value: (*value).into(), important: false, source: "test".into(),
        }).collect::<Vec<_>>();
        crate::style::Style::compute(&declarations, parent).unwrap()
    }

    #[test]
    fn cascade_resolves_final_font_and_inherits_computed_width() {
        let default = crate::style::Style::default();
        let parent = style(&[("border", "0.075em solid currentcolor"), ("font-size", "20px"),
            ("color", "#123456")], &default);
        assert_eq!(parent.borders[0].width, BorderWidth::Px(1.0));
        let child = style(&[("border", "inherit"), ("font-size", "40px"), ("color", "#abcdef")], &parent);
        assert_eq!(child.borders[0].width, BorderWidth::Px(1.0));
        assert_eq!(child.borders[0].color.resolve(child.color), 0xffabcdef);
        let child = style(&[], &parent);
        assert_eq!(child.borders[0].width, BorderWidth::Px(0.0));
        assert_eq!(child.borders[0].style, BorderStyle::None);
    }

    #[test]
    fn cascade_reset_and_longhand_order_preserve_css_defaults() {
        let default = crate::style::Style::default();
        for (values, expected) in [
            (vec![("border-style", "solid")], 0.0),
            (vec![("border", "solid")], 3.0),
            (vec![("border", "8px"), ("border-style", "solid")], 8.0),
            (vec![("border", "8px solid"), ("border", "initial"), ("border-style", "solid")], 3.0),
            (vec![("border", "8px solid"), ("border-width", "unset")], 3.0),
        ] {
            assert_eq!(style(&values, &default).borders[0].width, BorderWidth::Px(expected), "{values:?}");
        }
        let parent = style(&[("border", "8px none")], &default);
        let child = style(&[("border-width", "inherit"), ("border-style", "solid")], &parent);
        assert_eq!(child.borders[0].width, BorderWidth::Px(0.0));
    }

    #[test]
    fn side_cascade_preserves_other_edges_and_shorthand_resets() {
        let base = crate::style::Style::default();
        let result = style(&[("border", "8px solid red"), ("border-left", "2px solid blue"),
            ("border-top-width", "4px"), ("border-bottom-style", "none")], &base);
        assert_eq!(result.borders.map(|b| b.width), [4.,8.,0.,2.].map(BorderWidth::Px));
        assert_eq!(result.borders[3].color, BorderColor::Rgba(0xff0000ff));
        let result = style(&[("border-left", "9px solid blue"), ("border", "solid"),
            ("border-width", "1px 2px 3px 4px")], &base);
        assert_eq!(result.borders.map(|b| b.width), [1.,2.,3.,4.].map(BorderWidth::Px));
        assert!(result.borders.iter().all(|b| b.color == BorderColor::Current));
    }

    #[test]
    fn side_global_values_and_substitution_apply_to_selected_edge() {
        let base = crate::style::Style::default();
        let parent = style(&[("border", "7px solid red"), ("border-left", "3px solid blue")], &base);
        let child = style(&[("border", "1px solid green"), ("border-left", "inherit"),
            ("border-top", "initial"), ("--edge", "-2px solid red"), ("border-right", "var(--edge)")], &parent);
        assert_eq!(child.borders.map(|b| b.width), [0.,0.,1.,3.].map(BorderWidth::Px));
        assert_eq!(child.borders[3], parent.borders[3]);
    }

    #[test]
    fn invalid_side_component_lists_reset_only_the_selected_component() {
        for (property, value) in [("border-left-width", "2px 4px"),
            ("border-left-width", "calc(2px) 4px"), ("border-left-style", "solid hidden"),
            ("border-left-color", "red blue"), ("border-left-color", "rgb(1 2 3) red")] {
            let computed = style(&[("border", "8px solid red"), ("--edge", value),
                (property, "var(--edge)")], &crate::style::Style::default());
            assert_eq!(computed.borders[0].width, BorderWidth::Px(8.));
            let left = computed.borders[3];
            match property {
                "border-left-width" => assert_eq!(left.width, BorderWidth::Px(3.)),
                "border-left-style" => { assert_eq!(left.style, BorderStyle::None); assert_eq!(left.width, BorderWidth::Px(0.)); },
                _ => assert_eq!(left.color, BorderColor::Current),
            }
        }
        for (property, value) in [("border-left-width", "calc(2px)"),
            ("border-left-style", "dashed"), ("border-left-color", "oklch(50% 0.2 40)")] {
            assert!(!crate::substitution_validity::definitely_invalid(property, value), "{property}: {value}");
        }
    }

    #[test]
    fn computed_border_components_inherit_all_physical_edges() {
        let mut parent = crate::style::Style::default();
        parent.borders = [1.,2.,3.,4.].map(|width| Border {
            width: BorderWidth::Px(width), style: BorderStyle::Solid, color: BorderColor::Current,
        });
        let inherited = style(&[("border", "inherit")], &parent);
        assert_eq!(inherited.borders, parent.borders);
        let inherited = style(&[("border-width", "inherit"), ("border-style", "solid")], &parent);
        assert_eq!(inherited.borders, parent.borders);
        let reset = style(&[("border", "inherit"), ("border", "initial")], &parent);
        assert!(reset.borders.iter().all(|b| b.width == BorderWidth::Px(0.) && b.style == BorderStyle::None));
    }

    #[test]
    fn border_declarations_accept_css_wide_values_and_variables() {
        for name in ["border", "border-width", "border-style", "border-color"] {
            for value in ["initial", "var(--border)"] {
                let d = crate::css::Declaration { name:name.into(), value:value.into(), important:false, source:"test".into() };
                crate::style::Style::validate_declaration(&d).unwrap();
            }
        }
    }

    #[test]
    fn variables_reset_invalid_values_but_retain_valid_profile_errors() {
        let default = crate::style::Style::default();
        for value in ["-8px solid", "8% solid", "solid solid", "8px red blue", "-1px rgb(1 2 3)"] {
            let computed = style(&[("--b", value), ("border", "8px solid"), ("border", "var(--b)")], &default);
            assert_eq!(computed.borders[0].width, BorderWidth::Px(0.0), "{value}");
            assert_eq!(computed.borders[0].style, BorderStyle::None);
        }
        let computed = style(&[("--b", "0.075em solid rgb(18 52 86)"), ("border", "var(--b)"), ("font-size", "20px")], &default);
        assert_eq!(computed.borders[0].width, BorderWidth::Px(1.0));
        assert_eq!(computed.borders[0].color, BorderColor::Rgba(0xff123456));
        for (property, value) in [("border", "8px dashed red"), ("border-width", "1px 2px"),
            ("border", "1vw solid"), ("border-color", "red blue"), ("border", "calc(2px) solid")] {
            assert!(!crate::substitution_validity::invalid_top_level_component(property, value));
            assert!(!crate::substitution_validity::definitely_invalid(property, value), "{property}: {value}");
        }
    }

    #[test]
    fn shorthand_permutations_and_function_colors() {
        let expected = Border { width: BorderWidth::Px(8.0), style: BorderStyle::Solid,
            color: BorderColor::Rgba(0xff123456) };
        for value in ["8px solid #123456", "8px #123456 solid", "solid 8px #123456",
            "solid #123456 8px", "#123456 solid 8px", "#123456 8px solid",
            "SOLID/**/8PX rgb(18 52 86)", "8px solid rgb(18/**/52/**/86)"] {
            assert_eq!(Border::parse(value, "test").unwrap(), expected, "{value}");
        }
        assert_eq!(Border::parse("solid transparent", "test").unwrap().color, BorderColor::Rgba(0));
    }

    #[test]
    fn omitted_components_reset_and_current_color_stays_symbolic() {
        let mut border = Border::parse("8px", "test").unwrap();
        assert_eq!(border.used_width(16.0, "test").unwrap(), 0.0);
        border.style = BorderStyle::Solid;
        assert_eq!(border.used_width(16.0, "test").unwrap(), 8.0);
        assert_eq!(Border::parse("solid", "test").unwrap().used_width(16.0, "test").unwrap(), 3.0);
        assert_eq!(Border::parse("0", "test").unwrap().width, BorderWidth::Px(0.0));
        let border = Border::parse("8px hidden currentColor", "test").unwrap();
        assert_eq!(border.used_width(16.0, "test").unwrap(), 0.0);
        assert_eq!(border.color.resolve(0xff123456), 0xff123456);
        assert_eq!(border.color.resolve(0xffabcdef), 0xffabcdef);
    }

    #[test]
    fn shorthand_rejects_duplicates_repairs_and_outside_profile_values() {
        for value in ["", "8px 9px solid", "solid solid", "red blue", "solid dashed",
            "-1px solid", "solid 1%", "inherit", "solid rgb(1 2 3", "solid rgb(calc(1) 2 3)",
            "solid url(x)", "8px / red", "none hidden"] {
            assert!(Border::parse(value, "test").is_err(), "{value}");
        }
    }

    #[test]
    fn preserves_font_relative_and_fractional_authored_widths() {
        for (input, expected) in [
            ("0", BorderWidth::Px(0.0)), ("-0px", BorderWidth::Px(0.0)),
            ("0.25px", BorderWidth::Px(0.25)), ("2EM", BorderWidth::Em(2.0)),
            ("1.5rem", BorderWidth::Rem(1.5)), ("/**/THIN/**/", BorderWidth::Thin),
            ("medium", BorderWidth::Medium), ("thick", BorderWidth::Thick),
        ] {
            assert_eq!(BorderWidth::parse(input, "test").unwrap(), expected, "{input}");
        }
    }

    #[test]
    fn rejects_invalid_lengths_and_nonuniform_lists() {
        for input in ["", "1", "-1px", "-0.1em", "10%", "auto", "1vw",
            "1e99px", "1px 2px", "thin medium", "calc(1px)", "inherit", "1px,", "none"] {
            let error = BorderWidth::parse(input, "css:3:4").unwrap_err();
            assert_eq!(error.code, "unsupported-border-width", "{input}");
        }
    }

    #[test]
    fn resolution_matches_independent_chrome_computed_widths() {
        let reference: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/assets/border-width-reference.json")).unwrap();
        assert_eq!(reference["browser"], "153.0.8010.12");
        let mut count = 0;
        for case in reference["cases"].as_array().unwrap() {
            let value = case["name"].as_str().unwrap();
            let Ok(width) = BorderWidth::parse(value, "reference") else { continue };
            let font = case["result"]["fontSize"].as_str().unwrap().strip_suffix("px").unwrap().parse().unwrap();
            let expected: f32 = case["result"]["width"].as_str().unwrap().strip_suffix("px").unwrap().parse().unwrap();
            assert_eq!(width.resolve(font, "reference").unwrap(), expected, "{case}");
            count += 1;
        }
        assert_eq!(count, 30);
        assert!(BorderWidth::Em(2.0).resolve(f32::MAX, "test").is_err());
        assert_eq!(BorderWidth::Em(2.0).resolve(0.0, "test").unwrap(), 0.0);
    }
}
