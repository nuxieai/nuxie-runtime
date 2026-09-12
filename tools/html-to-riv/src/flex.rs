//! Computed flex values, staged independently of immutable-target admission.
//! Parsing a value here does not establish an ordinary-file encoding for it.
use super::{Diagnostic, Size, computed_size, unsupported};
use cssparser::ToCss;
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
            // Keep a single escaped identifier a single token. Decoding it to
            // text could turn embedded spaces into shorthand components.
            cssparser::Token::Ident(_) => words.push(token.to_css_string()),
            cssparser::Token::Number { .. } | cssparser::Token::Dimension { .. } | cssparser::Token::Percentage { .. } => words.push(parser.slice_from(start).to_owned()),
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
    let parts: Vec<_> = crate::css_whitespace::words(text).collect();
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
    fn escaped_identifier_spaces_cannot_manufacture_shorthand_components() {
        for (raw, expected) in [(r"\30 \20 \30 \20 auto", "0 0 auto"),
            (r"auto\a0", "auto\u{a0}"), (r"\30 \9 \30 \9 auto", "0\t0\tauto")] {
            let serialized = words(raw, "test").unwrap();
            assert_eq!(serialized.len(), 1);
            let mut input = cssparser::ParserInput::new(&serialized[0]);
            let mut parser = cssparser::Parser::new(&mut input);
            assert!(matches!(parser.next(), Ok(cssparser::Token::Ident(value)) if *value == expected));
            parser.expect_exhausted().unwrap();
            assert!(Flex::default().apply("flex", raw, Flex::default(), 20., "test").is_err(), "{raw}");
        }
        assert!(parse(r"n\6f ne").legacy());
        assert!(parse("0\t0\u{c}auto").legacy());
        assert!(Flex::default().apply("flex", "0\u{b}0 auto", Flex::default(), 20., "test").is_err());
    }
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

/// Native descriptors remain separate from the authored computed triple.
#[derive(Clone, Copy)]
pub(super) struct Lowering { fraction: f32, basis: Size }
impl Lowering {
    pub fn emit(self, records: &mut [crate::wire::Record], object: u32, direction: super::Direction) -> Result<(), Diagnostic> {
        use crate::wire::Value;
        let row = direction.is_row();
        records[object as usize + 1].set(if row { "fractionalWidth" } else { "fractionalHeight" }, Value::Float(self.fraction))?;
        let style = &mut records[object as usize + 2];
        style.set(if row { "layoutWidthScaleType" } else { "layoutHeightScaleType" }, Value::Uint(1))?;
        let (basis, units) = match self.basis {
            Size::Pixels(value) => (value, 1), Size::Auto => (0., 3),
            Size::Percent(_) => unreachable!("percentage basis is excluded by lowering"),
        };
        style.set("flexBasis", Value::Float(basis))?;
        // LayoutComponent's default basis unit is Auto, not Point.
        style.set("flexBasisUnitsValue", Value::Uint(units))?;
        Ok(())
    }
}
impl Flex {
    pub(super) fn legacy(self) -> bool { self.grow == 0. && self.shrink == 0. && matches!(self.basis, Size::Auto) }
}

/// Deliberately conservative: a definite descendant does not erase an unknown
/// intrinsic measurement request higher in this axis's ancestor chain.
pub(super) fn child_chain(parent: [bool; 2], style: &super::Style) -> [bool; 2] {
    let sizes = [style.width, style.height];
    let minimum = [style.min_width, style.min_height];
    let definite_size = |value| matches!(value, Size::Pixels(_))
        || matches!(value, Size::Percent(percent) if percent <= 100.);
    let maximum = [style.max_width, style.max_height];
    std::array::from_fn(|axis| parent[axis] && definite_size(sizes[axis])
        && definite_size(minimum[axis])
        && (matches!(maximum[axis], Size::Auto) || definite_size(maximum[axis])))
}

pub(super) fn lowering(style: &super::Style, parent: &super::Style, definite: [bool; 2], source: &str) -> Result<Option<Lowering>, Diagnostic> {
    if style.flex.legacy() { return Ok(None); }
    let main = if parent.direction.is_row() { 0 } else { 1 };
    if !definite[main] {
        return Err(unsupported(source, "Flex factors require an explicit definite main-size chain with percentages at most 100%; intrinsic, amplified or flex-sized ancestors need separate immutable-target qualification"));
    }
    let preferred = [style.width, style.height][main];
    let minimum = [style.min_width, style.min_height][main];
    let maximum = [style.max_width, style.max_height][main];
    if !matches!(preferred, Size::Auto) || !matches!(minimum, Size::Pixels(_)) || !matches!(maximum, Size::Auto | Size::Pixels(_)) {
        return Err(unsupported(source, "Flex factors currently require automatic preferred main size and point min/max bounds"));
    }
    let basis = style.flex.basis;
    if matches!(basis, Size::Percent(_)) {
        return Err(unsupported(source, "Percentage flex basis needs separate qualification"));
    }
    if !matches!(basis, Size::Pixels(0.)) && style.flex.grow != style.flex.shrink {
        return Err(unsupported(source, "Positive or automatic flex basis requires equal grow and shrink; independent factors need an ordinary-file composition"));
    }
    Ok(Some(Lowering { fraction: style.flex.grow, basis }))
}

#[derive(Default)]
pub(super) struct Group { active: bool, unequal_zero: bool, shrinking_basis: bool, helper: bool, baseline: bool }
impl Group {
    pub fn inspect(&mut self, style: &super::Style, parent: &super::Style, definite: [bool; 2], source: &str) -> Result<(), Diagnostic> {
        if let Some(plan) = lowering(style, parent, definite, source)? {
            self.active = true;
            self.unequal_zero |= matches!(plan.basis, Size::Pixels(0.)) && style.flex.grow != style.flex.shrink;
            self.shrinking_basis |= !matches!(plan.basis, Size::Pixels(0.)) && style.flex.shrink > 0.;
        }
        self.helper |= style.margins.main(parent.direction);
        self.baseline |= style.self_alignment.is_baseline() && !style.margins.cross(parent.direction);
        Ok(())
    }
    pub fn validate(self, parent: &super::Style, source: &str, candidate: bool) -> Result<(), Diagnostic> {
        if !self.active { return Ok(()); }
        if !candidate {
            return Err(unsupported(source, "Nonlegacy flex values are parsed but public lowering awaits aggregate numeric and visual qualification; only flex:0 0 auto is currently admitted"));
        }
        if self.unequal_zero && self.shrinking_basis {
            return Err(unsupported(source, "Unequal zero-basis factors cannot share a group with positive-basis shrinking items: unscaled shrink sums affect distribution"));
        }
        if self.helper || parent.spacing.distributes() {
            return Err(unsupported(source, "Flex factors cannot share a parent with main automatic margins or space-around/evenly helpers; leftover distribution needs a separate phase"));
        }
        if self.baseline {
            return Err(unsupported(source, "Flex factors combined with baseline groups require separate ordinary-file qualification"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod candidate_tests {
// Staged candidate invariants; this does not exercise public admission.
use crate::{CompileInput, CompileOutput};
fn compile(input: &CompileInput) -> Result<CompileOutput, Vec<crate::Diagnostic>> {
    super::super::compile_profile(input, super::super::FlexPolicy::Candidate).map_err(|error| vec![error])
}
use nuxie_schema::{definition_by_type_key, FieldKind};
use serde_json::{Value,json};
use std::collections::BTreeMap;
fn input(css: &str)->CompileInput { CompileInput { html:"<div id=p><div id=a><div id=leaf></div></div><div id=b></div></div>".into(),css:format!("#p{{width:160px;height:120px;flex-direction:row}}#a,#b{{height:20px}}#leaf{{width:10px;height:10px}}{css}"),width:240.,height:160. } }
fn scene(css:&str)->CompileOutput {compile(&input(css)).unwrap()}
fn uint(bytes:&[u8],i:&mut usize)->u32 {let mut result=0;for shift in (0..35).step_by(7){let b=bytes[*i];*i+=1;result|=((b&127)as u32)<<shift;if b<128{return result}}panic!("bad varuint")}
fn decoded(output:&CompileOutput)->Vec<(String,BTreeMap<String,Value>)> {
    let bytes=&output.riv;assert_eq!(&bytes[..7],b"RIVE\x07\x03\x00");let mut i=7;let mut fields=0;
    while uint(bytes,&mut i)!=0{fields+=1}i+=((fields+3)/4)*4;
    let mut result=Vec::new();
    while i<bytes.len(){let definition=definition_by_type_key(uint(bytes,&mut i)as u16).unwrap();let mut values=BTreeMap::new();loop{let key=uint(bytes,&mut i)as u16;if key==0{break}let property=definition.property_by_key_in_hierarchy(key).unwrap();let value=match property.runtime_type{
        FieldKind::Uint=>json!(uint(bytes,&mut i)),FieldKind::Double=>{let value=f32::from_le_bytes(bytes[i..i+4].try_into().unwrap());i+=4;assert!(value.is_finite());json!(value)},FieldKind::Color=>{let value=u32::from_le_bytes(bytes[i..i+4].try_into().unwrap());i+=4;json!(value)},FieldKind::String|FieldKind::Bytes=>{let len=uint(bytes,&mut i)as usize;let value=&bytes[i..i+len];i+=len;json!(String::from_utf8_lossy(value))},FieldKind::Bool=>{let value=bytes[i]!=0;i+=1;json!(value)},other=>panic!("unexpected {other:?}")};values.insert(property.name.to_owned(),value);}result.push((definition.name.to_owned(),values));}
    result
}

#[test]
fn legacy_explicit_values_leave_bytes_and_maps_unchanged() {
    for value in ["none","0 0 auto"] {assert_eq!(scene(""),scene(&format!("div{{flex:{value}}}")));}
    assert_eq!(scene(""),scene("div{flex-grow:0;flex-shrink:0;flex-basis:auto}"));
    assert_eq!(scene(""),scene("#a{flex:2 7 0px;flex:none}"));
}
#[test]
fn native_point_basis_fraction_and_source_ids_follow_original_axis() {
    for direction in ["row","row-reverse","column","column-reverse"] {
        let row=direction.starts_with("row");
        for align in ["stretch","flex-start","center","safe flex-end"] {
            let css=format!("#p{{flex-direction:{direction}}}#a{{height:auto;width:auto;flex:2 2 30px;align-self:{align}}}");
            let output=scene(&css); let r=decoded(&output);
            let a=output.source_map.iter().find(|n|n.id=="a").unwrap().object_id;
            let p=output.source_map[0].object_id;
            let outer=r[a as usize+1].1["parentId"].as_u64().unwrap() as u32;
            let slot=if outer==p {a}else{outer};
            assert_eq!(r[slot as usize+1].1[if row {"fractionalWidth"}else{"fractionalHeight"}],json!(2.));
            assert_eq!(r[slot as usize+2].1["flexBasis"],json!(30.));
            assert_eq!(r[slot as usize+2].1["flexBasisUnitsValue"],json!(1));
            assert_eq!(r[slot as usize+2].1[if row {"layoutWidthScaleType"}else{"layoutHeightScaleType"}],json!(1));
            assert_eq!(output.source_map.iter().map(|n|(n.id.as_str(),n.path.as_str())).collect::<Vec<_>>(),[("p","/0"),("a","/0/0"),("leaf","/0/0/0"),("b","/0/1")]);
        }
    }
}
#[test]
fn automatic_equal_basis_stays_distinct_and_requires_group_compatibility() {
    let output=scene("#a{flex:.25 .25 auto;align-self:center}");
    let r=decoded(&output);
    let a=output.source_map.iter().find(|n|n.id=="a").unwrap().object_id;
    let slot=r[a as usize+1].1["parentId"].as_u64().unwrap() as u32;
    assert_eq!(r[slot as usize+2].1["flexBasisUnitsValue"],json!(3));
    assert_eq!(r[slot as usize+1].1["fractionalWidth"],json!(0.25));
    assert!(compile(&input("#a{flex:1 7 0px}#b{flex:.25 .25 auto}")).is_err());
    assert!(compile(&input("#a{flex:1 1 0px}#p{width:101%}")).is_err());
    assert!(compile(&input("#a{flex:1 1 0px}#p{width:50%}")).is_ok());
}

#[test]
fn zero_basis_preserves_authored_cascade_without_normalizing_growth() {
    assert_eq!(scene("#a{flex:.25 7 0px}"),scene("#a{flex-grow:.25;flex-shrink:7;flex-basis:0px}"));
    assert_eq!(scene("#p{--f:.25 7 0px}#p>div:first-child{flex:var(--f);order:2}"),scene("#a{flex:.25 7 0px;order:2}"));
    assert_eq!(scene("#a{flex-grow:.25!important;flex:2 7 0px}"),scene("#a{flex:.25 7 0px}"));
    let r=decoded(&scene("#a{flex:.25 7 0px}"));
    assert_eq!(r.iter().filter_map(|(_,r)|r.get("fractionalWidth")).collect::<Vec<_>>(),vec![&json!(0.25)]);
    // Authored shrink still reaches a receiver through inheritance even though
    // zero-basis native lowering itself stores grow in both native factors.
    assert!(compile(&input("#a{flex:.25 7 0px}#leaf{width:auto;height:auto;flex-grow:.25;flex-shrink:inherit;flex-basis:30px}")).is_err());
}
#[test]
fn parent_wide_guards_cover_other_siblings_and_intrinsic_ancestors() {
    for css in [
        "#a{flex:1 7 0px}#b{width:auto;flex:.1 .1 100px}",
        "#a{flex:1 1 0px}#b{margin-left:auto}",
        "#a{flex:1 1 0px}#p{justify-content:space-around}",
        "#a{flex:1 1 0px}#b{align-self:baseline}",
        "#a{flex:1 1 0px}#p{width:auto}",
        "#a{flex:1 1 0px;width:20px}",
        "#a{flex:1 1 0px;min-width:auto}",
        "#a{flex:1 1 10%;width:auto}",
        "#a{flex:1 0 30px}",
        "#a{flex:initial}",
    ] {assert_eq!(compile(&input(css)).unwrap_err()[0].code,"unsupported-target-semantics","{css}");}
    let mut nested=input("#a{flex:1 1 0px}");nested.html=format!("<div id=outer>{}</div>",nested.html);
    assert!(compile(&nested).is_err()); // inner p fixed width cannot erase outer intrinsic chain
    nested.css.push_str("#outer{width:220px;height:150px}");assert!(compile(&nested).is_ok());
    assert!(compile(&input("#a{flex:1 7 0px}#b{width:auto;flex:0 0 30px}")).is_ok());
}
#[test]
fn vertical_flex_does_not_leak_false_ancestor_baseline_metrics() {
    let mut x=input("#a{width:60px;height:60px;align-self:baseline}#leaf{width:10px;height:auto;flex:1 1 0px}");
    assert!(compile(&x).is_err());
    x.css.push_str("#a{align-self:normal}");assert!(compile(&x).is_ok());
}
#[test]
fn invalid_flex_values_stay_strict_in_unmatched_and_overridden_rules() {
    for value in ["-1 1 0px","1 1 -2px","1 / 2","1 2 3px 4","1000001 1 0px"] {
        for css in [format!("#never{{flex:{value}}}"),format!("#a{{flex:{value};flex:none}}"),format!("#a{{--f:{value};flex:var(--f)}}")] {assert!(compile(&input(&css)).is_err(),"{css}");}
    }
}

}
