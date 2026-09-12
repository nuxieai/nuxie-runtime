//! Computed physical gaps and ordinary style fields, staged before admission.
//! Call after variable substitution and computing the element's font size.
use super::{Diagnostic, MAX_SIZE, ROOT_FONT_SIZE, resolved_length, unsupported};
use crate::wire::{Record, Value};

/// Physical row and column gaps; percentages retain their corresponding-axis dependency.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum GapValue { Normal, Pixels(f32), Percent(f32) }

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Gap { values: [GapValue; 2] } // row, column
impl Default for Gap {
    fn default() -> Self { Self { values: [GapValue::Normal; 2] } }
}
impl Gap {
    pub fn values(self) -> [GapValue; 2] { self.values }
    pub fn is_zero(self) -> bool {
        self.values.iter().all(|v| match v { GapValue::Normal => true, GapValue::Pixels(x) | GapValue::Percent(x) => *x == 0. })
    }
    pub fn has_percentage(self) -> bool {
        self.values.iter().any(|v| matches!(v, GapValue::Percent(x) if *x != 0.))
    }
    pub fn apply(&mut self, name: &str, text: &str, parent: Self, font_size: f32, source: &str) -> Result<(), Diagnostic> {
        let axis = match name {
            "row-gap" => Some(0), "column-gap" => Some(1), "gap" => None,
            _ => return Err(unsupported(source, "Unknown gap property")),
        };
        let tokens = tokens(text, font_size, source)?;
        let replacement = match tokens.as_slice() {
            [Token::Inherit] => parent,
            [Token::Reset] => Self::default(),
            _ => {
                if tokens.is_empty() || tokens.len() > if axis.is_some() { 1 } else { 2 } {
                    return Err(unsupported(source, "Gap requires one or two values, or one value for a longhand"));
                }
                let values = tokens.iter().map(|v| match v {
                    Token::Length(v) => Ok(*v),
                    _ => Err(unsupported(source, "CSS-wide gap keywords must be the complete value")),
                }).collect::<Result<Vec<_>, _>>()?;
                Self { values: [values[0], *values.get(1).unwrap_or(&values[0])] }
            },
        };
        if let Some(index) = axis { self.values[index] = replacement.values[index]; }
        else { *self = replacement; }
        Ok(())
    }
    /// Call once on a fresh style: normal and zero omit both ordinary fields.
    /// This emission does not certify percentage bases or synthetic-child contexts.
    pub fn emit(self, style: &mut Record) -> Result<(), Diagnostic> {
        if style.kind != "LayoutComponentStyle" {
            return Err(Diagnostic::new("schema-mismatch", "gap", "Gap requires a LayoutComponentStyle record"));
        }
        for (axis, field, units_field) in [
            (0, "gapVertical", "gapVerticalUnitsValue"),
            (1, "gapHorizontal", "gapHorizontalUnitsValue"),
        ] {
            let (value, units) = match self.values[axis] {
                GapValue::Normal => continue,
                GapValue::Pixels(v) => (v, 1), GapValue::Percent(v) => (v, 2),
            };
            if value == 0. { continue; }
            style.set(field, Value::Float(value))?;
            style.set(units_field, Value::Uint(units))?;
        }
        Ok(())
    }
}

enum Token { Length(GapValue), Inherit, Reset }
fn tokens(text: &str, font_size: f32, source: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut input = cssparser::ParserInput::new(text);
    let mut parser = cssparser::Parser::new(&mut input);
    let mut out = Vec::new();
    while !parser.is_exhausted() {
        parser.skip_whitespace();
        let start = parser.position();
        let token = parser.next().map_err(|_| unsupported(source, "Invalid gap value"))?.clone();
        let length = |value: f32| {
            if value.is_finite() && (0. ..=MAX_SIZE).contains(&value) { Ok(value) }
            else { Err(unsupported(source, "Gap must be finite and between 0 and 1000000")) }
        };
        out.push(match &token {
            cssparser::Token::Ident(v) if v.eq_ignore_ascii_case("normal") => Token::Length(GapValue::Normal),
            cssparser::Token::Ident(v) if v.eq_ignore_ascii_case("inherit") => Token::Inherit,
            cssparser::Token::Ident(v) if v.eq_ignore_ascii_case("initial") || v.eq_ignore_ascii_case("unset") => Token::Reset,
            cssparser::Token::Number { value, .. } if *value == 0. => Token::Length(GapValue::Pixels(0.)),
            cssparser::Token::Percentage { .. } => {
                // Keep the authored percentage in the wire descriptor. Going
                // through cssparser's normalized fraction adds a second round.
                let number = parser.slice_from(start).strip_suffix('%').expect("percentage token");
                let value = number.parse::<f32>().map_err(|_| unsupported(source, "Invalid percentage gap"))?;
                Token::Length(GapValue::Percent(length(value)?))
            },
            cssparser::Token::Dimension { value, unit, .. } => {
                let value = length(*value)?;
                let pixels = if unit.eq_ignore_ascii_case("px") { value }
                    else if unit.eq_ignore_ascii_case("em") { resolved_length(value, font_size, source)? }
                    else if unit.eq_ignore_ascii_case("rem") { resolved_length(value, ROOT_FONT_SIZE, source)? }
                    else { return Err(unsupported(source, "Gap admits px, em, rem and percentages")); };
                Token::Length(GapValue::Pixels(pixels))
            },
            _ => return Err(unsupported(source, "Gap admits nonnegative lengths, percentages, zero and CSS-wide keywords")),
        });
        if out.len() > 2 { return Err(unsupported(source, "Gap has too many components")); }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parsed(text: &str) -> Gap {
        let mut gap=Gap::default();gap.apply("gap",text,Gap::default(),20.,"fixture").unwrap();gap
    }
    #[test]
    fn shorthand_and_longhands_follow_declaration_order() {
        assert_eq!(parsed("1px").values(),[GapValue::Pixels(1.);2]);
        let mut gap=parsed("1px/**/2px");
        assert_eq!(gap.values(),[GapValue::Pixels(1.),GapValue::Pixels(2.)]);
        gap.apply("row-gap","3px",Gap::default(),20.,"fixture").unwrap();
        assert_eq!(gap.values(),[GapValue::Pixels(3.),GapValue::Pixels(2.)]);
        gap.apply("gap","4px",Gap::default(),20.,"fixture").unwrap();
        gap.apply("column-gap","5px",Gap::default(),20.,"fixture").unwrap();
        assert_eq!(gap.values(),[GapValue::Pixels(4.),GapValue::Pixels(5.)]);
    }
    #[test]
    fn inherited_computed_relative_lengths_are_not_resolved_again() {
        let parent=parsed("1em 2rem");
        assert_eq!(parent.values(),[GapValue::Pixels(20.),GapValue::Pixels(32.)]);
        let mut child=parsed("9px");child.apply("gap","inherit",parent,40.,"child").unwrap();assert_eq!(child,parent);
        child.apply("column-gap","2em",parent,40.,"child").unwrap();assert_eq!(child.values()[1],GapValue::Pixels(80.));
        child.apply("column-gap","inherit",parent,40.,"child").unwrap();assert_eq!(child,parent);
    }
    #[test]
    fn normal_and_css_wide_resets_preserve_computed_distinction() {
        let parent=parsed("10% normal");let mut child=parsed("3px 4px");
        child.apply("row-gap","inherit",parent,20.,"child").unwrap();assert_eq!(child.values(),[GapValue::Percent(10.),GapValue::Pixels(4.)]);
        child.apply("column-gap","INITIAL",parent,20.,"child").unwrap();assert_eq!(child,parent);
        child.apply("gap","unset",parent,20.,"child").unwrap();assert_eq!(child,Gap::default());
        assert_eq!(parsed("normal 2px").values(),[GapValue::Normal,GapValue::Pixels(2.)]);
        assert!(parent.has_percentage());assert!(!parsed("0% normal").has_percentage());
    }
    #[test]
    fn percentage_retains_authored_binary32_precision() {
        assert_eq!(parsed("/*leading*/0.027%/**/2.7e-2%").values(),[GapValue::Percent(0.027);2]);
    }
    #[test]
    fn invalid_values_are_atomic() {
        for text in ["", "auto", "-1px", "-1%", "1", "1px 2px 3px", "1px inherit", "inherit 0", "calc(1px)", "var(--g)", "1px,2px", "1e40px", "1000001%", "1000000em", "1vw", "revert", "revert-layer"] {
            let mut gap=parsed("7px");let old=gap;
            assert!(gap.apply("gap",text,Gap::default(),20.,"bad").is_err(),"{text}");assert_eq!(gap,old);
        }
        let mut gap=parsed("7px");
        for name in ["row-gap","column-gap","unknown"] {
            assert!(gap.apply(name,"1px 2px",Gap::default(),20.,"bad").is_err());assert_eq!(gap,parsed("7px"));
        }
    }
    #[test]
    fn normal_and_zero_preserve_default_wire_bytes() {
        let before=crate::wire::encode(&[Record::new("LayoutComponentStyle")]).unwrap();
        for gap in [Gap::default(),parsed("normal"),parsed("0 -0.0"),parsed("+0e0 0%"),parsed("0em 0rem")] {
            assert!(gap.is_zero());let mut style=Record::new("LayoutComponentStyle");gap.emit(&mut style).unwrap();
            assert_eq!(crate::wire::encode(&[style]).unwrap(),before);
        }
    }
    #[test]
    fn ordinary_fields_remain_physical_for_all_four_directions() {
        for direction in 0..4 {
            let mut actual=Record::new("LayoutComponentStyle");actual.set("flexDirectionValue",Value::Uint(direction)).unwrap();
            parsed("3em 5%").emit(&mut actual).unwrap();
            let mut expected=Record::new("LayoutComponentStyle");expected.set("flexDirectionValue",Value::Uint(direction)).unwrap();
            for (field,units,value,kind) in [("gapVertical","gapVerticalUnitsValue",60.,1),("gapHorizontal","gapHorizontalUnitsValue",5.,2)] {
                expected.set(field,Value::Float(value)).unwrap();expected.set(units,Value::Uint(kind)).unwrap();
            }
            assert_eq!(crate::wire::encode(&[actual]).unwrap(),crate::wire::encode(&[expected]).unwrap());
        }
        assert!(parsed("1px").emit(&mut Record::new("Node")).is_err());
    }
}
