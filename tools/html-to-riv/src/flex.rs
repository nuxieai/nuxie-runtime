//! Computed flex values, staged independently of immutable-target admission.
//! Parsing a value here does not establish an ordinary-file encoding for it.
use super::{Diagnostic, Size, computed_size, unsupported};
// Provisional compiler numeric admission limit, not a CSS grammar restriction
// or proof of aggregate arithmetic safety. Revisit with resource qualification.
const MAX_FACTOR: f32 = 1_000_000.;

#[derive(Clone, Copy)]
pub(super) struct Flex { pub grow: f32, pub shrink: f32, pub basis: Size }
impl Default for Flex {
    // The authoring reset differs intentionally from the CSS initial value.
    fn default() -> Self { Self { grow: 0., shrink: 0., basis: Size::Auto } }
}
impl Flex {
    const INITIAL: Self = Self { grow: 0., shrink: 1., basis: Size::Auto };
    pub fn apply(&mut self, name: &str, text: &str, parent: Self, font_size: f32, source: &str) -> Result<(), Diagnostic> {
        let words = words(text, source)?;
        let text = words.join(" ").to_ascii_lowercase();
        let inherited = text == "inherit";
        let reset = matches!(text.as_str(), "initial" | "unset");
        match name {
            "flex-grow" => self.grow = if inherited { parent.grow } else if reset { 0. } else { factor(&text, source)? },
            "flex-shrink" => self.shrink = if inherited { parent.shrink } else if reset { 1. } else { factor(&text, source)? },
            "flex-basis" => self.basis = computed_basis(&text, parent.basis, font_size, source)?,
            "flex" => *self = if inherited { parent } else if reset { Self::INITIAL } else {
                shorthand(&text, font_size, source)?
            },
            _ => return Err(unsupported(source, "Unknown flex property")),
        }
        Ok(())
    }
}
fn words(text: &str, source: &str) -> Result<Vec<String>, Diagnostic> {
    let mut input = cssparser::ParserInput::new(text);
    let mut parser = cssparser::Parser::new(&mut input);
    let mut words = Vec::new();
    while !parser.is_exhausted() {
        parser.skip_whitespace();
        let start = parser.position();
        let token = parser.next().map_err(|_| unsupported(source, "Invalid flex value"))?;
        match token {
            cssparser::Token::Ident(value) => words.push(value.to_string()),
            cssparser::Token::Number { .. } | cssparser::Token::Dimension { .. } | cssparser::Token::Percentage { .. } => words.push(parser.slice_from(start).trim().to_string()),
            _ => return Err(unsupported(source, "Flex values require numbers, supported lengths or keywords")),
        }
        if words.len() > 3 { return Err(unsupported(source, "Flex value has too many components")); }
    }
    Ok(words)
}
fn factor(text: &str, source: &str) -> Result<f32, Diagnostic> {
    let mut input = cssparser::ParserInput::new(text);
    let mut parser = cssparser::Parser::new(&mut input);
    let value = match parser.next() {
        Ok(cssparser::Token::Number { value, .. }) if value.is_finite() && (0. ..=MAX_FACTOR).contains(value) => *value,
        _ => return Err(unsupported(source, "Flex factor must be a finite nonnegative CSS number at most 1000000")),
    };
    parser.expect_exhausted().map_err(|_| unsupported(source, "Flex factor requires exactly one number"))?;
    Ok(value)
}
fn computed_basis(text: &str, parent: Size, font_size: f32, source: &str) -> Result<Size, Diagnostic> {
    // Every CSS numeric spelling of zero is a valid unitless zero length.
    if matches!(factor(text, source), Ok(value) if value == 0.) { return Ok(Size::Pixels(0.)); }
    computed_size(text, parent, font_size, source)
}
fn shorthand(text: &str, font_size: f32, source: &str) -> Result<Flex, Diagnostic> {
    match text {
        "none" => return Ok(Flex::default()),
        "auto" => return Ok(Flex { grow: 1., shrink: 1., basis: Size::Auto }),
        _ => (),
    }
    let parts: Vec<_> = text.split_ascii_whitespace().collect();
    if parts.iter().any(|v| matches!(*v, "inherit" | "initial" | "unset" | "none")) {
        return Err(unsupported(source, "CSS-wide flex keywords must be the complete value"));
    }
    let basis = |v| computed_basis(v, Size::Auto, font_size, source);
    // Blink's omitted shorthand basis is 0%, preserved as a percentage.
    let omitted = Size::Percent(0.);
    match parts.as_slice() {
        [a] => if let Ok(grow) = factor(a, source) { Ok(Flex { grow, shrink: 1., basis: omitted }) }
            else { Ok(Flex { grow: 1., shrink: 1., basis: basis(a)? }) },
        [a,b] => if let Ok(grow) = factor(a, source) {
            if let Ok(shrink) = factor(b, source) { Ok(Flex { grow, shrink, basis: omitted }) }
            else { Ok(Flex { grow, shrink: 1., basis: basis(b)? }) }
        } else { Ok(Flex { grow: factor(b, source)?, shrink: 1., basis: basis(a)? }) },
        [a,b,c] => if let (Ok(grow),Ok(shrink)) = (factor(a, source),factor(b, source)) {
            Ok(Flex { grow, shrink, basis: basis(c)? })
        } else { Ok(Flex { grow: factor(b, source)?, shrink: factor(c, source)?, basis: basis(a)? }) },
        _ => Err(unsupported(source, "flex requires one to three components")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(text: &str) -> Flex { let mut f=Flex::default(); f.apply("flex",text,Flex::default(),20.,"test").unwrap(); f }
    #[test]
    fn reset_initial_and_inheritance_are_distinct() {
        assert_eq!(Flex::default().shrink,0.);
        assert_eq!(parse("initial").shrink,1.);
        assert_eq!(parse("unset").shrink,1.);
        assert_eq!(parse("none").shrink,0.);
        let parent=parse("2 .25 3em"); let mut f=Flex::default();
        f.apply("flex","inherit",parent,80.,"test").unwrap();
        assert_eq!((f.grow,f.shrink),(2.,0.25)); assert!(matches!(f.basis,Size::Pixels(60.)));
        f.apply("flex-shrink","initial",parent,80.,"test").unwrap(); assert_eq!(f.shrink,1.);
    }
    #[test]
    fn shorthand_preserves_independent_factors_and_basis_units() {
        for value in ["2 .25 10px","10px 2 .25","2/**/.25/**/10px"] { let f=parse(value);assert_eq!((f.grow,f.shrink),(2.,0.25));assert!(matches!(f.basis,Size::Pixels(10.))); }
        for value in ["2","2 1"] { assert!(matches!(parse(value).basis,Size::Percent(0.))); }
        for value in ["2 1 0", "2 1 -0", "2 1 +0", "2 1 0.0", "2 1 0e0"] { assert!(matches!(parse(value).basis,Size::Pixels(0.))); }
        assert!(matches!(parse("2 1 0%").basis,Size::Percent(0.)));
        assert_eq!(parse("auto").grow,1.);
        assert!(matches!(parse("2 1 2rem").basis,Size::Pixels(32.)));
    }
    #[test]
    fn longhands_preserve_other_fields_and_computed_percentages() {
        let parent=parse("3 2 25%");let mut f=parse("1 .5 10px");
        f.apply("flex-grow","inherit",parent,16.,"test").unwrap();
        assert_eq!((f.grow,f.shrink),(3.,0.5));assert!(matches!(f.basis,Size::Pixels(10.)));
        f.apply("flex-basis","inherit",parent,64.,"test").unwrap();
        assert!(matches!(f.basis,Size::Percent(25.)));
        f.apply("flex-basis","2em",parent,20.,"test").unwrap();
        assert!(matches!(f.basis,Size::Pixels(40.)));
        for (name,value) in [("flex-grow","-1"),("flex-shrink","1px"),("flex-basis","-2px"),("flex-grow","1 2")] {
            assert!(f.apply(name,value,parent,16.,"test").is_err());
            assert_eq!((f.grow,f.shrink),(3.,0.5));assert!(matches!(f.basis,Size::Pixels(40.)));
        }
    }
    #[test]
    fn invalid_values_do_not_mutate_computed_state() {
        for value in ["", "-1", "NaN", "1e999", "1000001", "1 2 3", "1 auto 2", "none 2", "inherit 1", "1 2 0px 4", "1 / 2", "calc(1)"] {
            let mut f=Flex::default();assert!(f.apply("flex",value,Flex::default(),16.,"test").is_err(),"{value}");assert_eq!((f.grow,f.shrink),(0.,0.));assert!(matches!(f.basis,Size::Auto));
        }
    }
}
