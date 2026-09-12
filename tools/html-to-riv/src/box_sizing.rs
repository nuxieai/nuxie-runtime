//! CSS sizing interpretation is distinct from the ordinary border-box fields.
//! Keep this view out of inherited computed styles: it belongs to emitted boxes.
use super::{computed_provenance::{NumericSize, NumericStyle, Scalar, Unresolved},
    padding::Inset, Diagnostic, Direction, Size, Style, unsupported};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum BoxSizing { #[default] BorderBox, ContentBox }
pub(super) fn computed(text: &str, parent: BoxSizing, source: &str) -> Result<BoxSizing, Diagnostic> {
    match crate::css_whitespace::trim(text).to_ascii_lowercase().as_str() {
        "border-box" => Ok(BoxSizing::BorderBox),
        "content-box" | "initial" | "unset" => Ok(BoxSizing::ContentBox),
        "inherit" => Ok(parent),
        _ => Err(unsupported(source, "box-sizing requires border-box, content-box, inherit, initial or unset")),
    }
}

pub(super) struct Lowered {
    pub sizes: [Size; 2],
    pub bounds: [Size; 4],
    pub numeric: Box<NumericStyle>,
}

/// A separate ordinary content object preserves authored point precision when
/// the outer border-box addition and padding subtraction would cancel it.
/// This is an emission plan, never an inherited computed style.
#[derive(Clone, Copy)]
pub(super) struct ContentOwner {
    pub packing: Direction,
    pub sizes: [Size; 2],
    pub bounds: [Size; 4],
}
pub(super) fn content_owner(style: &Style, parent_direction: Direction) -> Option<ContentOwner> {
    let sizes = [style.width, style.height];
    let bounds = [style.min_width, style.min_height, style.max_width, style.max_height];
    if style.box_sizing != BoxSizing::ContentBox || style.padding.is_zero()
        || style.padding.has_percentage()
        || sizes.iter().chain(&bounds).any(|size| matches!(size, Size::Percent(_))) {
        return None;
    }
    // Keeping the participant's parent axis preserves automatic cross stretch;
    // the inner container still lays out its children in the authored direction.
    Some(ContentOwner { packing: if parent_direction.is_row() { Direction::Row } else { Direction::Column }, sizes, bounds })
}
fn add(a: &Scalar, b: &Scalar) -> Scalar {
    a.as_ref().map_err(Clone::clone)?.add_nonnegative(b.as_ref().map_err(Clone::clone)?).map_err(Into::into)
}
fn translated(value: Size, numeric: &mut NumericSize, padding: f32, provenance: &Scalar,
    name: &str, source: &str) -> Result<Size, Diagnostic> {
    match value {
        Size::Auto => Ok(value),
        Size::Percent(_) if padding == 0. => {
            // A tiny/missing ideal inset cannot become a percentage coefficient.
            *numeric = NumericSize::Percent(Err(Unresolved::UnitMismatch));
            Ok(value)
        }
        Size::Percent(_) => Err(unsupported(source, format!("content-box {name} with nonzero point padding requires a responsive percentage-plus-points file composition; a single ordinary dimension cannot encode that sum"))),
        Size::Pixels(v) => {
            // Match native padding accumulation before adding the content size.
            // Do not cap a derived size at the source coefficient limit.
            let outer = v + padding;
            if !outer.is_finite() { return Err(unsupported(source, "Content-box size plus padding exceeds finite binary32 geometry")); }
            *numeric = NumericSize::Pixels(match numeric {
                NumericSize::Pixels(value) => add(value, provenance)
                    .and_then(|p| p.with_native(outer).map_err(Into::into)),
                _ => Err(Unresolved::UnitMismatch),
            });
            Ok(Size::Pixels(outer))
        }
    }
}
pub(super) fn lower(style: &Style, source: &str) -> Result<Lowered, Diagnostic> {
    lower_impl(style, source, false)
}
/// Padded images retain exact point content on a separate ordinary owner.
pub(super) fn lower_image(style: &Style, source: &str) -> Result<Lowered, Diagnostic> {
    lower_impl(style, source, true)
}
fn lower_impl(style: &Style, source: &str, image: bool) -> Result<Lowered, Diagnostic> {
    let mut result = Lowered {sizes: [style.width, style.height],
        bounds: [style.min_width, style.min_height, style.max_width, style.max_height],
        numeric: style.numeric.clone()};
    if style.box_sizing == BoxSizing::BorderBox { return Ok(result); }
    let sides = style.padding.sides();
    for axis in 0..2 {
        if image && [axis, axis + 2].iter().any(|&i| matches!(sides[i], Inset::Percent(v) if v != 0.)) {
            if matches!(result.sizes[axis], Size::Percent(_)) {
                return Err(unsupported(source, "Image content-box percentage size with nonzero same-axis padding requires a separate original-containing-size composition"));
            }
            // Only image defaults (zero minima and absent maxima) reach this
            // path. Responsive padding supplies its own native border floor.
            result.sizes[axis] = Size::Auto;
            if axis == 0 { result.numeric.width = NumericSize::Auto; }
            else { result.numeric.height = NumericSize::Auto; }
            continue;
        }
        let points = |side| match side { Inset::Pixels(v) | Inset::Percent(v) if v == 0. => Some(v),
            Inset::Pixels(v) => Some(v), _ => None };
        let (Some(start), Some(end)) = (points(sides[axis]), points(sides[axis+2])) else {
            return Err(unsupported(source, "content-box with nonzero percentage padding requires a responsive mixed-size file composition; all padding percentages depend on containing width, including vertical sides"));
        };
        let padding = start + end;
        if !padding.is_finite() { return Err(unsupported(source, "Content-box padding sum exceeds finite binary32 geometry")); }
        let provenance = |index: usize| match sides[index] {
            // A zero *coefficient* with nonzero/missing ideal still depends on
            // containing width. It cannot be reinterpreted as point padding.
            Inset::Percent(_) if !style.numeric.padding[index].as_ref().is_ok_and(|p|p.is_exact_zero()) => Err(Unresolved::UnitMismatch),
            _ => style.numeric.padding[index].clone(),
        };
        let provenance = add(&provenance(axis), &provenance(axis+2));
        if padding == 0. && provenance.as_ref().is_ok_and(|p|p.is_exact_zero()) { continue; }
        let (dimension, minimum, maximum, name) = if axis == 0 {
            (&mut result.numeric.width, &mut result.numeric.min_width, &mut result.numeric.max_width, "width")
        } else { (&mut result.numeric.height, &mut result.numeric.min_height, &mut result.numeric.max_height, "height") };
        result.sizes[axis] = translated(result.sizes[axis], dimension, padding, &provenance, name, source)?;
        result.bounds[axis] = translated(result.bounds[axis], minimum, padding, &provenance, &format!("min-{name}"), source)?;
        result.bounds[axis+2] = translated(result.bounds[axis+2], maximum, padding, &provenance, &format!("max-{name}"), source)?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{computed, computed_provenance::NumericSize, scalar_provenance::ScalarProvenance};
    fn style(css: &str, parent: &Style) -> Style {
        let document = scraper::Html::parse_document("<div id=a></div>");
        let selector = scraper::Selector::parse("#a").unwrap();
        computed(document.select(&selector).next().unwrap(), &crate::css::stylesheet(css).unwrap(), parent, false, true).unwrap()
    }
    fn scalar(value: &NumericSize) -> &ScalarProvenance {
        match value { NumericSize::Pixels(Ok(p)) | NumericSize::Percent(Ok(p)) => p, _ => panic!("missing scalar: {value:?}") }
    }
    fn encloses(value: &NumericSize, expected: f64) {
        let p=scalar(value);assert!(p.ideal_bounds().lower()<=expected && p.ideal_bounds().upper()>=expected, "{p:?} excludes {expected}");
    }
    #[test]
    fn authored_inheritance_never_receives_translated_outer_dimensions() {
        let parent=style("#a{box-sizing:content-box;width:100px;height:40px;min-width:80px;max-width:200px;padding:10px}", &Style::default());
        let lowered=lower(&parent,"parent").unwrap();
        assert!(matches!(lowered.sizes[0],Size::Pixels(120.)));
        let child=style("#a{width:inherit;min-width:inherit;max-width:inherit;padding:inherit}",&parent);
        assert!(child.box_sizing==BoxSizing::BorderBox);
        assert!(matches!(child.width,Size::Pixels(100.)));
        assert!(matches!(lower(&child,"child").unwrap().sizes[0],Size::Pixels(100.)));
        let child=style("#a{box-sizing:inherit;width:inherit;min-width:inherit;max-width:inherit;padding:2px}",&parent);
        let lowered=lower(&child,"child").unwrap();
        assert!(matches!(lowered.sizes[0],Size::Pixels(104.)));
        assert!(matches!(lowered.bounds[0],Size::Pixels(84.)));
        assert!(matches!(lowered.bounds[2],Size::Pixels(204.)));
        assert!(matches!(parent.width,Size::Pixels(100.)));
    }
    #[test]
    fn scalar_addition_retains_rounding_error_and_original_source_range() {
        for (width,left,right,native,ideal) in [
            ("1000000",".03125",".03125",1000000.0625_f32,1000000.0625_f64),
            (".03125","1000000",".03125",1000000.,1000000.0625),
            ("999999.875","1000000","1000000",3000000.,2999999.875),
            ("1000000","1000000","1000000",3000000.,3000000.),
        ] {
            let s=style(&format!("#a{{box-sizing:content-box;width:{width}px;padding:0 {right}px 0 {left}px}}"),&Style::default());
            let lowered=lower(&s,"a").unwrap();let value=scalar(&lowered.numeric.width);
            assert_eq!(value.native().to_bits(),native.to_bits());encloses(&lowered.numeric.width,ideal);
            assert!(value.absolute_error_upper()>=(f64::from(native)-ideal).abs());
            assert!(matches!(lowered.sizes[0],Size::Pixels(v)if v.to_bits()==native.to_bits()));
        }
    }
    #[test]
    fn padding_shorthand_font_and_inherited_provenance_stay_on_correct_sides() {
        let parent=style(r"#a{font-size:10px;--p:1.00000001em 2rem 3p\78  4px;padding:var(--p)}",&Style::default());
        for (i,want) in [(0,4.),(1,10.0000001),(2,32.),(3,3.)] {
            let p=parent.numeric.padding[i].as_ref().unwrap();assert!(p.ideal_bounds().lower()<=want && p.ideal_bounds().upper()>=want);
        }
        let child=style("#a{box-sizing:content-box;font-size:30px;padding:inherit;padding-left:.5em;width:10px;height:20px}",&parent);
        let lowered=lower(&child,"a").unwrap();encloses(&lowered.numeric.width,57.);encloses(&lowered.numeric.height,33.0000001);
        assert!(child.numeric.padding[1].as_ref().unwrap().proves_equal(parent.numeric.padding[1].as_ref().unwrap()));
    }
    #[test]
    fn underflowed_padding_never_becomes_an_exact_zero_dimension_proof() {
        let s=style("#a{box-sizing:content-box;width:0px;padding:1e-50px}",&Style::default());
        let lowered=lower(&s,"a").unwrap();let p=scalar(&lowered.numeric.width);
        assert_eq!(p.native(),0.);assert!(!p.is_exact_zero());encloses(&lowered.numeric.width,2e-50);
        for css in ["width:0px;padding:1e-50%","width:50%;padding:1e-50px"] {
            let s=style(&format!("#a{{box-sizing:content-box;{css}}}"),&Style::default());
            let lowered=lower(&s,"a").unwrap();assert!(matches!(lowered.numeric.width,NumericSize::Pixels(Err(_))|NumericSize::Percent(Err(_))));
        }
        let s=style("#a{box-sizing:content-box;width:50%;padding:0%}",&Style::default());
        encloses(&lower(&s,"a").unwrap().numeric.width,50.);
    }
    #[test]
    fn transformed_descriptors_bind_native_values_but_do_not_certify_padded_layouts() {
        use super::super::{compile_profile_with_descriptors,FlexPolicy,flex_sizes,flex_numeric::ErrorEnvelope};
        let input=crate::CompileInput { assets: Default::default(),html:"<div id=p><div id=c></div></div>".into(),css:"#p{box-sizing:content-box;width:100px;height:40px;padding:10px}#c{width:50%;height:20px}".into(),width:240.,height:160.};
        let (output,groups)=compile_profile_with_descriptors(&input,FlexPolicy::Guarded).unwrap();
        let id=|name|output.source_map.iter().find(|n|n.id==name).unwrap().object_id;
        let p=groups.iter().flat_map(|g|g.items.iter()).find(|i|i.source_id=="p").unwrap();
        encloses(&p.computed.numeric.cross,120.);assert_eq!(p.native.cross.value,Some(120.));assert!(!p.computed.padding_zero);
        let inner=groups.iter().find(|g|g.items.iter().any(|i|i.source_id=="c")).unwrap();
        assert!(inner.content_owner);assert_ne!(inner.parent_id,id("p"));
        encloses(&inner.parent_numeric_cross,100.);
        assert!(inner.structural_issues.iter().any(|issue|issue.contains("content-owner")));
        let domains=flex_sizes::propagate(&groups,[ErrorEnvelope::new(0.,16384.,0.001).unwrap();2]);
        for name in ["p","c"] {assert!(domains[&id(name)].axes.iter().all(Result::is_err));}
    }
}
