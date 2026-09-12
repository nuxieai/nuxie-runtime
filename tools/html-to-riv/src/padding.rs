//! Computed physical padding and ordinary style fields, staged before admission.
//! Call after variable substitution and computing the element's font size.
use super::{Diagnostic, MAX_SIZE, ROOT_FONT_SIZE, resolved_length, unsupported};
use crate::wire::{Record, Value};

/// Percentages retain their containing-block-width dependency on every side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Inset { Pixels(f32), Percent(f32) }

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Padding { sides: [Inset; 4] } // left, top, right, bottom
impl Default for Padding {
    fn default() -> Self { Self { sides: [Inset::Pixels(0.); 4] } }
}
impl Padding {
    pub fn sides(self) -> [Inset; 4] { self.sides }
    pub fn is_zero(self) -> bool {
        self.sides.iter().all(|v| matches!(v, Inset::Pixels(x) | Inset::Percent(x) if *x == 0.))
    }
    pub fn has_percentage(self) -> bool {
        self.sides.iter().any(|v| matches!(v, Inset::Percent(x) if *x != 0.))
    }
    pub fn apply(&mut self, name: &str, text: &str, parent: Self, font_size: f32, source: &str) -> Result<(), Diagnostic> {
        let side = match name {
            "padding-left" => Some(0), "padding-top" => Some(1),
            "padding-right" => Some(2), "padding-bottom" => Some(3),
            "padding" => None,
            _ => return Err(unsupported(source, "Unknown padding property")),
        };
        let tokens = tokens(text, font_size, source)?;
        let replacement = match tokens.as_slice() {
            [Token::Inherit] => parent,
            [Token::Reset] => Self::default(),
            _ => {
                if tokens.is_empty() || tokens.len() > if side.is_some() { 1 } else { 4 } {
                    return Err(unsupported(source, "Padding requires one to four lengths, or one length for a side"));
                }
                let values = tokens.iter().map(|v| match v {
                    Token::Length(v) => Ok(*v),
                    _ => Err(unsupported(source, "CSS-wide padding keywords must be the complete value")),
                }).collect::<Result<Vec<_>, _>>()?;
                let top = values[0];
                let right = *values.get(1).unwrap_or(&top);
                let bottom = *values.get(2).unwrap_or(&top);
                let left = *values.get(3).unwrap_or(&right);
                Self { sides: [left, top, right, bottom] }
            },
        };
        if let Some(index) = side { self.sides[index] = replacement.sides[index]; }
        else { *self = replacement; }
        Ok(())
    }
    /// No extra objects; zero sides omit both fields to retain existing bytes.
    /// This emission operation does not certify its containing layout context.
    pub fn emit(self, style: &mut Record) -> Result<(), Diagnostic> {
        if style.kind != "LayoutComponentStyle" {
            return Err(Diagnostic::new("schema-mismatch", "padding", "Padding requires a LayoutComponentStyle record"));
        }
        for (side, field, units_field) in [
            (1, "paddingTop", "paddingTopUnitsValue"),
            (2, "paddingRight", "paddingRightUnitsValue"),
            (3, "paddingBottom", "paddingBottomUnitsValue"),
            (0, "paddingLeft", "paddingLeftUnitsValue"),
        ] {
            let (value, units) = match self.sides[side] { Inset::Pixels(v) => (v, 1), Inset::Percent(v) => (v, 2) };
            if value == 0. { continue; }
            style.set(field, Value::Float(value))?;
            style.set(units_field, Value::Uint(units))?;
        }
        Ok(())
    }
}

enum Token { Length(Inset), Inherit, Reset }
fn tokens(text: &str, font_size: f32, source: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut input = cssparser::ParserInput::new(text);
    let mut parser = cssparser::Parser::new(&mut input);
    let mut out = Vec::new();
    while !parser.is_exhausted() {
        parser.skip_whitespace();
        let start = parser.position();
        let token = parser.next().map_err(|_| unsupported(source, "Invalid padding value"))?.clone();
        let length = |value: f32| {
            if value.is_finite() && (0. ..=MAX_SIZE).contains(&value) { Ok(value) }
            else { Err(unsupported(source, "Padding must be finite and between 0 and 1000000")) }
        };
        out.push(match &token {
            cssparser::Token::Ident(v) if v.eq_ignore_ascii_case("inherit") => Token::Inherit,
            cssparser::Token::Ident(v) if v.eq_ignore_ascii_case("initial") || v.eq_ignore_ascii_case("unset") => Token::Reset,
            cssparser::Token::Number { value, .. } if *value == 0. => Token::Length(Inset::Pixels(0.)),
            cssparser::Token::Percentage { .. } => {
                // Keep the authored percentage in the wire descriptor. Going
                // through cssparser's normalized fraction adds a second round.
                let number = parser.slice_from(start).strip_suffix('%').expect("percentage token");
                let value = number.parse::<f32>().map_err(|_| unsupported(source, "Invalid percentage padding"))?;
                Token::Length(Inset::Percent(length(value)?))
            },
            cssparser::Token::Dimension { value, unit, .. } => {
                let value = length(*value)?;
                let pixels = if unit.eq_ignore_ascii_case("px") { value }
                    else if unit.eq_ignore_ascii_case("em") { resolved_length(value, font_size, source)? }
                    else if unit.eq_ignore_ascii_case("rem") { resolved_length(value, ROOT_FONT_SIZE, source)? }
                    else { return Err(unsupported(source, "Padding admits px, em, rem and percentages")); };
                Token::Length(Inset::Pixels(pixels))
            },
            _ => return Err(unsupported(source, "Padding admits nonnegative lengths, percentages, zero and CSS-wide keywords")),
        });
        if out.len() > 4 { return Err(unsupported(source, "Padding has too many components")); }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parsed(text: &str) -> Padding {
        let mut p = Padding::default();p.apply("padding", text, Padding::default(), 20., "fixture").unwrap();p
    }
    #[test]
    fn shorthand_expands_physical_sides_and_comments() {
        for (text, expected) in [("1px", [1.,1.,1.,1.]), ("1px 2px", [2.,1.,2.,1.]),
            ("1px 2px 3px", [2.,1.,2.,3.]), ("1px/**/2px 3px 4px", [4.,1.,2.,3.])] {
            assert_eq!(parsed(text).sides(), expected.map(Inset::Pixels));
        }
    }
    #[test]
    fn computed_inheritance_preserves_percent_and_resolves_relative_lengths_once() {
        let parent = parsed("1em 2rem 25% 0");
        assert_eq!(parent.sides(), [Inset::Pixels(0.),Inset::Pixels(20.),Inset::Pixels(32.),Inset::Percent(25.)]);
        let mut child = parsed("9px");child.apply("padding", "inherit", parent, 40., "child").unwrap();assert_eq!(child,parent);
        child.apply("padding-top", "2em", parent, 40., "child").unwrap();assert_eq!(child.sides()[1],Inset::Pixels(80.));
        child.apply("padding-top", "inherit", parent, 40., "child").unwrap();assert_eq!(child,parent);
        child.apply("padding", "unset", parent, 40., "child").unwrap();assert!(child.is_zero());
        assert!(parent.has_percentage());
    }
    #[test]
    fn longhand_changes_only_its_side_and_resets_are_not_inherited() {
        let mut p = parsed("1px 2px 3px 4px");p.apply("padding-left", "INITIAL", parsed("9px"), 20., "fixture").unwrap();
        assert_eq!(p.sides(), [Inset::Pixels(0.),Inset::Pixels(1.),Inset::Pixels(2.),Inset::Pixels(3.)]);
        p.apply("padding-bottom", ".5e2%", Padding::default(),20.,"fixture").unwrap();assert_eq!(p.sides()[3],Inset::Percent(50.));
    }
    #[test]
    fn percentage_keeps_authored_binary32_value_without_fraction_roundtrip() {
        let p = parsed("/*leading*/0.027%/**/2.7e-2% 0% 1000000%");
        assert_eq!(p.sides(), [Inset::Percent(1_000_000.),Inset::Percent(0.027),Inset::Percent(0.027),Inset::Percent(0.)]);
    }
    #[test]
    fn invalid_values_do_not_partially_mutate_computed_padding() {
        for text in ["", "auto", "-1px", "1", "1px 2px 3px 4px 5px", "1px inherit", "inherit 0", "calc(1px)", "var(--p)", "1px,2px", "1e40px", "1000001px", "1000000em", "1vw"] {
            let mut p=parsed("7px");let old=p;assert!(p.apply("padding",text,Padding::default(),20.,"bad").is_err(),"{text}");assert_eq!(p,old);
        }
        let mut p=parsed("7px");assert!(p.apply("padding-top","1px 2px",Padding::default(),20.,"bad").is_err());assert_eq!(p,parsed("7px"));
        assert!(p.apply("unknown","0",Padding::default(),20.,"bad").is_err());
    }
    #[test]
    fn zero_spellings_and_default_preserve_wire_bytes() {
        let before=crate::wire::encode(&[Record::new("LayoutComponentStyle")]).unwrap();
        for p in [Padding::default(),parsed("0 -0.0 +0e0 0%"),parsed("0em 0rem 0px")] {
            let mut style=Record::new("LayoutComponentStyle");p.emit(&mut style).unwrap();assert_eq!(crate::wire::encode(&[style]).unwrap(),before);
        }
    }
    #[test]
    fn emission_matches_explicit_native_side_units() {
        let p=parsed("1px 2% 3em 4rem");let mut actual=Record::new("LayoutComponentStyle");p.emit(&mut actual).unwrap();
        let mut expected=Record::new("LayoutComponentStyle");
        for (name,units,value,kind) in [("paddingLeft","paddingLeftUnitsValue",64.,1),("paddingTop","paddingTopUnitsValue",1.,1),
            ("paddingRight","paddingRightUnitsValue",2.,2),("paddingBottom","paddingBottomUnitsValue",60.,1)] {
            expected.set(name,Value::Float(value)).unwrap();expected.set(units,Value::Uint(kind)).unwrap();
        }
        assert_eq!(crate::wire::encode(&[actual]).unwrap(),crate::wire::encode(&[expected]).unwrap());
        assert!(p.emit(&mut Record::new("Node")).is_err());
    }
}
