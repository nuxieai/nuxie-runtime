//! Clip-edge authoring value for the documented compiler profile.
//! CSS-wide keywords belong to the cascade, not this property grammar.
use crate::Diagnostic;
use cssparser::{Parser, ParserInput, Token};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ClipBox {
    Content,
    #[default]
    Padding,
    Border,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ClipMargin {
    pub origin: ClipBox,
    pub pixels: f32,
}

impl ClipMargin {
    pub fn parse(value: &str, font_size: f32, source: &str) -> Result<Self, Diagnostic> {
        let invalid = || Diagnostic::new("unsupported-overflow-clip-margin", source,
            "Expected an optional content-box, padding-box or border-box and a finite signed px/em/rem length");
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        let mut origin = None;
        let mut pixels = None;
        while !parser.is_exhausted() {
            match parser.next().map_err(|_| invalid())?.clone() {
                Token::Ident(name) if origin.is_none() => {
                    origin = Some(if name.eq_ignore_ascii_case("content-box") { ClipBox::Content }
                        else if name.eq_ignore_ascii_case("padding-box") { ClipBox::Padding }
                        else if name.eq_ignore_ascii_case("border-box") { ClipBox::Border }
                        else { return Err(invalid()); });
                }
                Token::Number { value: 0.0, .. } if pixels.is_none() => pixels = Some(0.0),
                Token::Dimension { value, unit, .. } if pixels.is_none() => {
                    let scale = if unit.eq_ignore_ascii_case("px") { 1.0 }
                        else if unit.eq_ignore_ascii_case("em") { font_size }
                        else if unit.eq_ignore_ascii_case("rem") { 16.0 }
                        else { return Err(invalid()); };
                    let resolved = value * scale;
                    if !value.is_finite() || !scale.is_finite()
                        || scale < 0.0 || !resolved.is_finite() { return Err(invalid()); }
                    pixels = Some(resolved);
                }
                _ => return Err(invalid()),
            }
        }
        if origin.is_none() && pixels.is_none() { return Err(invalid()); }
        Ok(Self { origin: origin.unwrap_or_default(), pixels: pixels.unwrap_or(0.0) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_components_accept_either_order_and_comments() {
        for (input, origin, pixels) in [
            ("-8px", ClipBox::Padding, -8.), ("-2em content-box", ClipBox::Content, -40.),
            ("0", ClipBox::Padding, 0.), ("8px", ClipBox::Padding, 8.),
            ("content-box", ClipBox::Content, 0.),
            ("border-box 8px", ClipBox::Border, 8.),
            ("8px BORDER-BOX", ClipBox::Border, 8.),
            ("padding-box/**/2em", ClipBox::Padding, 40.),
            ("1.5rem content-box", ClipBox::Content, 24.),
        ] {
            assert_eq!(ClipMargin::parse(input,20.,"test").unwrap(), ClipMargin { origin,pixels },"{input}");
        }
    }

    #[test]
    fn rejects_duplicate_components_invalid_units_and_nonfinite_lengths() {
        for input in ["", "8", "10%", "auto", "margin-box", "8px 9px",
            "padding-box content-box", "1vw", "1e99px", "calc(2px)", "inherit", "8px,content-box"] {
            assert_eq!(ClipMargin::parse(input,16.,"css:2:3").unwrap_err().code,
                "unsupported-overflow-clip-margin", "{input}");
        }
        assert!(ClipMargin::parse("2em",f32::MAX,"test").is_err());
    }
}
