//! Conservative authored wrapping admission into the immutable, checked graph.
//! Fixed dimensions retain their cascade provenance through LayoutUnit conversion.
//! Responsive lengths require a separate live conversion proof; source numbers
//! that are not provably equal to their computed float are not silently rounded.
use super::{computed, unsupported, Style, Direction, AlignmentPosition, OverflowAlignment,
    computed_provenance::{NumericSize, NumericStyle}, fixed_layout::LayoutStyle,
    scalar_provenance::ScalarProvenance, wrapping_style::Wrap,
    wrapping_domains, wrapping_composition, wrapping_sizes::MachineInterval};
use crate::{css, css_whitespace, wire::{self, Record, Value}, CompileInput, CompileOutput,
    Diagnostic, SourceNode};
use scraper::ElementRef;
use std::collections::BTreeSet;

fn identity(element: ElementRef<'_>, path: &str, ids: &mut BTreeSet<String>) -> Result<String, Diagnostic> {
    if !["div", "section", "article", "main", "header", "footer", "aside", "nav"].contains(&element.value().name()) {
        return Err(unsupported(path, "Wrapping admits ordinary boxes only; images and other elements require separate qualification"));
    }
    for (name, _) in element.value().attrs() {
        let data = name.strip_prefix("data-").is_some_and(|suffix| !suffix.is_empty());
        if !["id", "class", "style"].contains(&name) && !data {
            return Err(unsupported(path, format!("Attribute {name} is not admitted")));
        }
    }
    let id = element.attr("id").map(str::to_owned).unwrap_or_else(|| format!("node{path}"));
    if id.is_empty() || !ids.insert(id.clone()) {
        return Err(Diagnostic::new("duplicate-id", path, "Empty or duplicate element identity"));
    }
    Ok(id)
}
fn no_text(element: ElementRef<'_>, path: &str) -> Result<(), Diagnostic> {
    if element.children().any(|n| n.value().as_text().is_some_and(|t| !css_whitespace::trim(t).is_empty())) {
        return Err(unsupported(path, "Text rendering is not yet requalified on the immutable runtime"));
    }
    Ok(())
}
fn ordinary(style: &Style, path: &str) -> Result<(), Diagnostic> {
    if style.margins.any() || !style.padding.is_zero() || !style.gap.is_zero()
        || style.spacing.distributes() || !style.flex.legacy() {
        return Err(unsupported(path, "Wrapping currently requires zero margins, padding and gaps, ordinary flex:0 0 auto, and no distributed spacing"));
    }
    Ok(())
}
fn layout(numeric: &NumericStyle, path: &str) -> Result<LayoutStyle, Diagnostic> {
    for (field, value, auto) in [("width", &numeric.width, false), ("height", &numeric.height, false),
        ("min-width", &numeric.min_width, false), ("min-height", &numeric.min_height, false),
        ("max-width", &numeric.max_width, true), ("max-height", &numeric.max_height, true)] {
        match value {
            NumericSize::Auto if auto => {},
            NumericSize::Auto => return Err(unsupported(path, format!("Wrapping requires definite fixed {field}; automatic preferred sizes and minima require separate qualification"))),
            NumericSize::Percent(_) => return Err(unsupported(path, format!("Wrapping percentage {field} awaits live LayoutUnit quantization qualification"))),
            NumericSize::Pixels(value) => {
                let source = value.as_ref().map_err(|e| unsupported(path, format!("Wrapping {field} has unresolved original numeric provenance: {e:?}")))?;
                let exact = ScalarProvenance::exact_constant(source.native())
                    .map_err(|e| unsupported(path, format!("Wrapping {field} has invalid computed value: {e:?}")))?;
                if !source.proves_equal(&exact) {
                    return Err(unsupported(path, format!("Wrapping {field} currently requires an authored value provably equal to its computed float; general decimal and expression conversion awaits qualification")));
                }
            }
        }
    }
    LayoutStyle::from_numeric(numeric).map_err(|e| unsupported(path, format!("Wrapping fixed LayoutUnit conversion is unresolved: {e:?}")))
}
fn alignment(style: &Style, reverse: bool, path: &str) -> Result<f32, Diagnostic> {
    if !matches!(style.self_alignment.overflow, OverflowAlignment::Default) {
        return Err(unsupported(path, "Wrapping safe/unsafe self alignment requires separate qualification"));
    }
    use AlignmentPosition::*;
    Ok(match style.self_alignment.position {
        Auto | Normal | Stretch | FlexStart => 0.,
        Start | SelfStart => if reverse { 1. } else { 0. },
        Center => 0.5,
        FlexEnd => 1.,
        End | SelfEnd => if reverse { 0. } else { 1. },
        Baseline | FirstBaseline | LastBaseline => return Err(unsupported(path, "Wrapping baseline alignment requires separate qualification")),
    })
}
fn node(records: &mut Vec<Record>, parent: u32, layout: &LayoutStyle) -> Result<u32, Diagnostic> {
    let id = records.len() as u32 - 1;
    let mut n = Record::new("LayoutComponent");
    let mut s = Record::new("LayoutComponentStyle");
    n.set("parentId", Value::Uint(parent))?;
    n.set("styleId", Value::Uint(id + 1))?;
    for (axis, preferred, minimum, maximum) in [("Width", &layout.width, &layout.min_width, &layout.max_width),
        ("Height", &layout.height, &layout.min_height, &layout.max_height)] {
        let lower = axis.to_ascii_lowercase();
        n.set(&lower, Value::Float(preferred.value().expect("admitted fixed preferred size")))?;
        s.set(&format!("{lower}UnitsValue"), Value::Uint(preferred.units().expect("admitted fixed units")))?;
        for (prefix, length) in [("min", minimum), ("max", maximum)] {
            length.validate().map_err(|e| unsupported("document", format!("Wrapping layout binding: {e:?}")))?;
            if let (Some(value), Some(units)) = (length.value(), length.units()) {
                s.set(&format!("{prefix}{axis}"), Value::Float(value))?;
                s.set(&format!("{prefix}{axis}UnitsValue"), Value::Uint(units))?;
            }
        }
    }
    records.extend([n, s]);
    Ok(id)
}
fn paint(records: &mut Vec<Record>, owner: u32, color: u32) -> Result<(), Diagnostic> {
    let fill = records.len() as u32 - 1;
    let mut f = Record::new("Fill"); f.set("parentId", Value::Uint(owner))?;
    let mut c = Record::new("SolidColor"); c.set("parentId", Value::Uint(fill))?;
    c.set("colorValue", Value::Color(color))?;
    records.extend([f, c]); Ok(())
}

pub(super) fn compile(body: ElementRef<'_>, rules: &[css::Rule], input: &CompileInput,
    host: &Style, candidate: bool, padding_candidate: bool) -> Result<Option<CompileOutput>, Diagnostic> {
    // Detection does not validate attributes or alter normal unwrapped emission.
    let root_count = body.child_elements().count();
    let mut wrapped = None;
    for element in body.child_elements() {
        // A computed Style owns its inherited variable environment. Discard
        // each ordinary sibling immediately instead of retaining all of them.
        let style = computed(element, rules, host, candidate, padding_candidate)?;
        if style.wrap.is_wrapped() {
            if root_count != 1 { return Err(unsupported("html", "Wrapping currently requires exactly one authored top-level root")); }
            wrapped = Some((element, style));
            break;
        }
    }
    let Some((element, style)) = wrapped.as_ref() else { return Ok(None); };
    if !input.assets.is_empty() { return Err(unsupported("html", "Wrapping with assets requires separate qualification")); }
    no_text(body, "html")?;
    let mut ids = BTreeSet::new();
    let parent_id = identity(*element, "/0", &mut ids)?;
    no_text(*element, "/0")?;
    ordinary(style, "/0")?;
    if style.background.used(style.foreground) >> 24 != 0 {
        return Err(unsupported("/0", "Wrapping parent backgrounds require separate paint qualification; use a transparent root"));
    }
    if alignment(style, false, "/0")? != 0. {
        return Err(unsupported("/0", "Wrapping root self alignment currently requires auto, normal, stretch or start"));
    }
    let reverse_cross = matches!(style.wrap, Wrap::Reverse);
    let line = style.line_alignment.fraction(reverse_cross);
    let pnum = layout(&style.numeric, "/0")?;
    let children = element.child_elements().collect::<Vec<_>>();
    if children.len() + 1 > 8192 { return Err(Diagnostic::new("object-limit", "/0", "Document exceeds 8192 authored elements")); }
    if children.is_empty() { return Err(unsupported("/0", "Wrapping admission currently requires at least one direct leaf box")); }
    let mut ordered = Vec::new();
    for (index, child) in children.into_iter().enumerate() {
        let path = format!("/0/{index}");
        let id = identity(child, &path, &mut ids)?;
        no_text(child, &path)?;
        if child.child_elements().next().is_some() { return Err(unsupported(&path, "Wrapping currently admits direct empty leaf boxes only")); }
        let s = computed(child, rules, style, candidate, padding_candidate)?;
        ordinary(&s, &path)?;
        if s.wrap.is_wrapped() { return Err(unsupported(&path, "Nested wrapping requires separate qualification")); }
        let fraction = alignment(&s, reverse_cross, &path)?;
        let dims = layout(&s.numeric, &path)?;
        ordered.push((s.order, index, id, path, dims, fraction, s.background.used(s.foreground)));
    }
    ordered.sort_by_key(|(order, index, ..)| (*order, *index));
    let row = style.direction.is_row();
    let source_children = ordered.iter().map(|item| item.4.clone()).collect::<Vec<_>>();
    let stretch = if line.is_none() {
        Some(super::wrapping_stretch::Plan::new(&pnum, &source_children, row)
            .map_err(|e| unsupported("/0", format!("Wrapping stretch-line source proof is unresolved: {e:?}")))?)
    } else { None };
    if let Some(plan) = &stretch {
        plan.validate().map_err(|e| unsupported("/0", format!("Wrapping stretch-line binding is unresolved: {e:?}")))?;
        if !plan.matches_sources(&pnum, &source_children, row) {
            return Err(unsupported("/0", "Wrapping stretch-line authored sources do not match"));
        }
    }
    let line = line.unwrap_or(0.);
    let mut records = vec![Record::new("Backboard"), Record::new("Artboard"), Record::new("LayoutComponentStyle")];
    records[1].set("name", Value::String("HTML".into()))?;
    records[1].set("styleId", Value::Uint(1))?;
    records[1].set("width", Value::Float(input.width))?;
    records[1].set("height", Value::Float(input.height))?;
    paint(&mut records, 0, 0xffffffff)?;
    let parent = node(&mut records, 0, &pnum)?;
    let reverse_main = matches!(style.direction, Direction::RowReverse | Direction::ColumnReverse);
    records[parent as usize + 2].set("flexWrapValue", Value::Uint(style.wrap.wire()))?;
    records[parent as usize + 2].set("flexDirectionValue", Value::Uint((if row { 2 } else { 0 }) + u32::from(reverse_main)))?;
    records[parent as usize + 2].set("layoutAlignmentType", Value::Uint((line * 2.) as u32 * if row { 3 } else { 1 }))?;
    let mut map = vec![SourceNode { id: parent_id, path: "/0".into(), object_id: parent }];
    let mut roles = Vec::new(); let mut authored = Vec::new(); let mut alignments = Vec::new();
    for (index, (_, _, id, path, dims, fraction, color)) in ordered.into_iter().enumerate() {
        let slot_dims = stretch.as_ref().map_or(&dims, |plan| &plan.slots()[index]);
        let slot = node(&mut records, parent, slot_dims)?;
        let visible = node(&mut records, slot, &dims)?;
        paint(&mut records, visible, color)?;
        roles.push((slot, visible)); authored.push((slot, slot_dims.clone())); authored.push((visible, dims));
        alignments.push(fraction); map.push(SourceNode { id, path, object_id: visible });
    }
    let references = authored.iter().map(|(id, n)| (*id, n)).collect::<Vec<_>>();
    let full = MachineInterval::new(0., 16384.).expect("fixed finite viewport domain");
    let domains = wrapping_domains::resolve_layout(&records, parent, &roles, &pnum, &references, [full; 2])
        .map_err(|e| unsupported("/0", format!("Wrapping dimension or structural proof is unresolved: {e:?}")))?;
    let derived = wrapping_composition::compose_original_with_bounds(domains, &alignments, 100_000)
        .map_err(|e| unsupported("/0", format!("Wrapping immutable-runtime graph proof is unresolved: {e:?}")))?;
    map.sort_by_cached_key(|n| n.path.split('/').skip(1).map(|p| p.parse::<usize>().expect("numeric DOM path")).collect::<Vec<_>>());
    Ok(Some(CompileOutput { riv: wire::encode(derived.candidate().records())?, source_map: map }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn numeric(text: &str, native: f32) -> NumericStyle {
        let fixed = NumericSize::Pixels(Ok(ScalarProvenance::from_decimal(text, native).unwrap()));
        NumericStyle { width: fixed.clone(), height: fixed, ..NumericStyle::default() }
    }
    #[test]
    fn exact_source_is_retained_while_layout_value_is_quantized() {
        let n = numeric("0.0703125", 0.0703125);
        let normalized = layout(&n, "/0").unwrap();
        let super::super::fixed_layout::LayoutLength::Fixed(f) = normalized.width else { panic!() };
        assert_eq!(f.emitted(), 0.0625);
        assert_eq!(f.authored().native(), 0.0703125);
        assert_eq!(f.computed_bits(), 0.0703125_f32.to_bits());
    }
    #[test]
    fn rounded_decimal_and_live_percentage_are_explicitly_unresolved() {
        assert!(layout(&numeric("0.1", 0.1), "/0").is_err());
        let mut n = numeric("20", 20.);
        n.min_width = NumericSize::Auto;
        assert!(layout(&n, "/0").is_err());
        n.min_width = NumericStyle::default().min_width;
        n.width = NumericSize::Percent(Ok(ScalarProvenance::exact_constant(100.).unwrap()));
        assert!(layout(&n, "/0").is_err());
    }
    #[test]
    fn physical_and_flex_alignment_differ_under_reverse_wrap() {
        let mut s = Style::default();
        s.self_alignment.position = AlignmentPosition::Start;
        assert_eq!(alignment(&s, true, "/0").unwrap(), 1.);
        s.self_alignment.position = AlignmentPosition::FlexStart;
        assert_eq!(alignment(&s, true, "/0").unwrap(), 0.);
        s.self_alignment.overflow = OverflowAlignment::Safe;
        assert!(alignment(&s, false, "/0").is_err());
    }
    fn request(extra: &str) -> CompileInput {
        CompileInput {
            html: "<div id=p><div id=a></div><div id=b></div><div id=c></div></div>".into(),
            css: format!("#p{{width:60px;height:80px;flex-direction:row;flex-wrap:wrap;align-content:flex-start}}#a,#b,#c{{width:20px;height:20px;background:red}}{extra}"),
            width: 320., height: 200., assets: Default::default(),
        }
    }
    fn reject(input: &CompileInput, message: &str) {
        let errors = crate::compile(input).expect_err("unqualified wrapping must diagnose");
        assert!(errors.iter().any(|e| e.code == "unsupported-target-semantics" && e.message.contains(message)), "{errors:?}");
    }
    #[test]
    fn public_fixed_boxes_are_deterministic_and_identity_stays_in_dom_order() {
        let input = request("#a,#c{order:2}#b{order:-1;background:currentColor;color:blue}#c{background:transparent}");
        let first = crate::compile(&input).unwrap();
        assert_eq!(first, crate::compile(&input).unwrap());
        assert_eq!(first.source_map.iter().map(|n| (n.id.as_str(), n.path.as_str())).collect::<Vec<_>>(),
            vec![("p", "/0"), ("a", "/0/0"), ("b", "/0/1"), ("c", "/0/2")]);
        let m = &first.source_map;
        // File owners follow stable CSS order; public paths follow the DOM.
        assert!(m[2].object_id < m[1].object_id && m[1].object_id < m[3].object_id);
        assert!(!first.riv.is_empty());
    }
    #[test]
    fn public_normal_and_reverse_axes_enter_the_checked_graph() {
        for direction in ["row", "row-reverse", "column", "column-reverse"] {
            for wrap in ["wrap", "wrap-reverse"] {
                let input = request(&format!("#p{{flex-direction:{direction};flex-wrap:{wrap};align-content:center}}#a{{align-self:center}}"));
                assert!(crate::compile(&input).is_ok(), "{direction} {wrap}");
            }
        }
    }
    #[test]
    fn public_shape_boundaries_diagnose() {
        let mut multiple = request(""); multiple.html.push_str("<div></div>");
        reject(&multiple, "exactly one");
        let mut descendants = request("");
        descendants.html = "<div id=p><div id=a><div></div></div></div>".into();
        reject(&descendants, "direct empty leaf");
        reject(&request("#a{flex-wrap:wrap}"), "Nested wrapping");
        let mut text = request(""); text.html = "<div id=p>Hello<div id=a></div></div>".into();
        reject(&text, "Text rendering");
        reject(&request("#p{background:red}"), "parent backgrounds");
    }
    #[test]
    fn public_numeric_and_stretch_boundaries_diagnose() {
        for (extra, message) in [
            ("#a{width:50%}", "live LayoutUnit"),
            ("#p{height:50%}", "live LayoutUnit"),
            ("#a{width:auto}", "definite fixed width"),
            ("#a{min-width:auto}", "definite fixed min-width"),
            ("#a{width:0.1px}", "provably equal"),
        ] { reject(&request(extra), message); }
    }
    #[test]
    fn explicit_nowrap_keeps_existing_public_bytes_and_maps() {
        let mut implicit = request("");
        implicit.css = "#p{width:60px;height:80px}#a,#b,#c{width:20px;height:20px;background:red}".into();
        let mut explicit = implicit.clone();
        explicit.css.push_str("#p,#a,#b,#c{flex-wrap:nowrap;align-content:normal}");
        assert_eq!(crate::compile(&implicit).unwrap(), crate::compile(&explicit).unwrap());
    }

}
