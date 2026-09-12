//! Source-derived point constraints for ordinary image layout owners.
//!
//! No parent dimensions or browser measurements enter this plan. Responsive
//! sizes, automatic minima, flex sizing, and stretching need different plans.
use super::{Diagnostic, Direction, Size, Style, unsupported,
    box_sizing::{BoxSizing, ContentOwner, Lowered},
    computed_provenance::NumericSize,
    scalar_provenance::{ScalarError, ScalarProvenance as Scalar}};
use crate::assets::ImageMetadata;

pub(super) struct Resolved {
    /// Selected content dimensions keep their authored/arithmetic provenance.
    pub content: [Scalar; 2],
    pub border: [Scalar; 2],
    padded: bool,
    packing: Direction,
}

pub(super) fn has_constraints(style: &Style) -> bool {
    let zero = |size: Size, numeric: &NumericSize| matches!((size, numeric),
        (Size::Pixels(0.), NumericSize::Pixels(Ok(scalar))) if scalar.is_exact_zero());
    !(zero(style.min_width, &style.numeric.min_width) && zero(style.min_height, &style.numeric.min_height)
        && matches!((&style.max_width, &style.numeric.max_width), (Size::Auto, NumericSize::Auto))
        && matches!((&style.max_height, &style.numeric.max_height), (Size::Auto, NumericSize::Auto)))
}

fn exact(value: f32) -> Result<Scalar, ScalarError> { Scalar::exact_constant(value) }
fn point(size: Size, numeric: &NumericSize) -> Result<Option<Scalar>, &'static str> {
    match (size, numeric) {
        (Size::Auto, NumericSize::Auto) => Ok(None),
        (Size::Pixels(value), NumericSize::Pixels(Ok(scalar)))
            if value == scalar.native() && scalar.is_nonnegative() => Ok(Some(scalar.clone())),
        _ => Err("Image point constraints require matching original point provenance; percentages and automatic minima remain unqualified"),
    }
}
fn clamp(value: &Scalar, minimum: &Scalar, maximum: Option<&Scalar>) -> Result<Scalar, ScalarError> {
    let capped = if let Some(maximum) = maximum { value.minimum(maximum)? } else { value.clone() };
    capped.maximum(minimum)
}

/// Return no plan for the existing unconstrained profile, leaving its bytes
/// entirely untouched. This function alone does not admit constrained images.
pub(super) fn resolve(style: &Style, parent: &Style, metadata: ImageMetadata, source: &str) -> Result<Option<Resolved>, Diagnostic> {
    if !has_constraints(style) {
        return Ok(None);
    }
    if !style.flex.legacy() || style.margins.any() || style.self_alignment.is_baseline() {
        return Err(unsupported(source, "Image point constraint evaluation requires ordinary nonflexible sizing without automatic margins or baseline alignment"));
    }
    let preferred = [point(style.width, &style.numeric.width), point(style.height, &style.numeric.height)];
    let [width, height] = preferred.map(|v|v.map_err(|e|unsupported(source,e)));
    let preferred = [width?, height?];
    let cross = usize::from(parent.direction.is_row());
    if preferred[cross].is_none() && style.self_alignment.stretches() {
        return Err(unsupported(source, "Automatic constrained image axes with stretch need a responsive sizing composition"));
    }
    let minimum = [point(style.min_width, &style.numeric.min_width), point(style.min_height, &style.numeric.min_height)];
    let maximum = [point(style.max_width, &style.numeric.max_width), point(style.max_height, &style.numeric.max_height)];
    let mut minima = Vec::new();
    let mut maxima = Vec::new();
    for (minimum, maximum) in minimum.into_iter().zip(maximum) {
        minima.push(minimum.map_err(|e|unsupported(source,e))?.ok_or_else(||unsupported(source, "Automatic image minima need replaced-element flex minimum qualification"))?);
        maxima.push(maximum.map_err(|e|unsupported(source,e))?);
    }
    let sides = style.padding.sides();
    let mut insets = Vec::new();
    for axis in 0..2 {
        let mut pair = Vec::new();
        for index in [axis, axis + 2] {
            let scalar = style.numeric.padding[index].as_ref().map_err(|_|unsupported(source,"Image point padding requires original provenance"))?;
            let value = match sides[index] {
                super::padding::Inset::Pixels(value) => value,
                super::padding::Inset::Percent(0.) if scalar.is_exact_zero() => 0.,
                _ => return Err(unsupported(source,"Constrained images with percentage padding require a responsive constraint composition")),
            };
            if value != scalar.native() || !scalar.is_nonnegative() {
                return Err(unsupported(source,"Image point padding provenance does not match emitted values"));
            }
            pair.push(scalar);
        }
        insets.push(pair[0].add_nonnegative(pair[1]).map_err(|_|unsupported(source,"Image point padding sum exceeds finite geometry"))?);
    }
    let packing = match preferred.each_ref().map(Option::is_some) {
        [false, true] => Direction::Row,
        [true, false] => Direction::Column,
        _ if parent.direction.is_row() => Direction::Row,
        _ => Direction::Column,
    };
    let solved = (|| -> Result<Resolved, ScalarError> {
        let mut preferred = preferred;
        // Min wins when min > max, before border-to-content zero floors.
        for axis in 0..2 {
            if let Some(maximum) = &maxima[axis] { maxima[axis] = Some(maximum.maximum(&minima[axis])?); }
            if style.box_sizing == BoxSizing::BorderBox {
                preferred[axis] = preferred[axis].as_ref().map(|p|p.subtract_floor_zero(&insets[axis])).transpose()?;
                minima[axis] = minima[axis].subtract_floor_zero(&insets[axis])?;
                maxima[axis] = maxima[axis].as_ref().map(|p|p.subtract_floor_zero(&insets[axis])).transpose()?;
            }
        }
        let intrinsic = [exact(metadata.width as f32)?, exact(metadata.height as f32)?];
        if metadata.width == 0 || metadata.height == 0 { return Err(ScalarError::InvalidDivisor); }
        let content = match preferred {
            [Some(width), Some(height)] => [clamp(&width, &minima[0], maxima[0].as_ref())?, clamp(&height, &minima[1], maxima[1].as_ref())?],
            [None, None] => {
                let low = minima[0].divide_positive_constant(metadata.width as f32)?
                    .maximum(&minima[1].divide_positive_constant(metadata.height as f32)?)?;
                // An absent maximum is infinity; cap at 1 before selecting low
                // so no nonfinite synthetic scalar enters the provenance model.
                let mut scale = exact(1.)?;
                for axis in 0..2 {
                    if let Some(maximum) = &maxima[axis] {
                        scale = scale.minimum(&maximum.divide_positive_constant(intrinsic[axis].native())?)?;
                    }
                }
                let scale = scale.maximum(&low)?;
                [clamp(&intrinsic[0].multiply(&scale)?, &minima[0], maxima[0].as_ref())?,
                 clamp(&intrinsic[1].multiply(&scale)?, &minima[1], maxima[1].as_ref())?]
            },
            values => {
                let fixed = usize::from(values[0].is_none());
                let automatic = 1 - fixed;
                let fixed_value = clamp(values[fixed].as_ref().expect("one fixed axis"), &minima[fixed], maxima[fixed].as_ref())?;
                let automatic_value = fixed_value.multiply(&intrinsic[automatic])?.divide_positive_constant(intrinsic[fixed].native())?;
                let automatic_value = clamp(&automatic_value, &minima[automatic], maxima[automatic].as_ref())?;
                if fixed == 0 { [fixed_value, automatic_value] } else { [automatic_value, fixed_value] }
            },
        };
        let border = [content[0].add_nonnegative(&insets[0])?, content[1].add_nonnegative(&insets[1])?];
        Ok(Resolved { content, border, padded: !style.padding.is_zero(), packing })
    })();
    solved.map(Some).map_err(|_|unsupported(source,"Image point constraint or padding arithmetic exceeds finite geometry; cannot preserve source-derived numerical provenance"))
}
impl Resolved {
    pub fn lower_outer(&self, style: &Style) -> Lowered {
        let mut numeric = style.numeric.clone();
        numeric.width = NumericSize::Pixels(Ok(self.border[0].clone()));
        numeric.height = NumericSize::Pixels(Ok(self.border[1].clone()));
        numeric.min_width = NumericSize::Pixels(exact(0.).map_err(Into::into));
        numeric.min_height = NumericSize::Pixels(exact(0.).map_err(Into::into));
        numeric.max_width = NumericSize::Auto;
        numeric.max_height = NumericSize::Auto;
        Lowered { sizes: self.border.each_ref().map(|v|Size::Pixels(v.native())),
            bounds: [Size::Pixels(0.), Size::Pixels(0.), Size::Auto, Size::Auto], numeric }
    }
    pub fn content_owner(&self) -> Option<ContentOwner> {
        self.padded.then(||ContentOwner { packing: self.packing, sizes: self.content.each_ref().map(|v|Size::Pixels(v.native())),
            bounds: [Size::Pixels(0.), Size::Pixels(0.), Size::Auto, Size::Auto] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn style(css: &str) -> Style {
        let document = scraper::Html::parse_document("<img id=image>");
        let selector = scraper::Selector::parse("#image").unwrap();
        super::super::computed(document.select(&selector).next().unwrap(),
            &crate::css::stylesheet(&format!("#image{{align-self:flex-start;{css}}}")).unwrap(), &Style::default(), false, true).unwrap()
    }
    fn resolved(css: &str) -> Resolved {
        resolve(&style(css), &Style::default(), ImageMetadata { width:96,height:64,asset_index:0 }, "image").unwrap().unwrap()
    }
    fn size(css: &str) -> [f32;2] { resolved(css).content.each_ref().map(Scalar::native) }
    #[test]
    fn intrinsic_constraints_preserve_ratio_until_opposing_bounds_require_distortion() {
        for (css, expected) in [
            ("max-width:48px",[48.,32.]),("min-width:192px",[192.,128.]),
            ("max-height:32px",[48.,32.]),("min-height:128px",[192.,128.]),
            ("max-width:72px;max-height:40px",[60.,40.]),
            ("min-width:120px;min-height:100px",[150.,100.]),
            ("min-width:192px;max-height:32px",[192.,32.]),
            ("max-width:48px;min-height:128px",[48.,128.]),
            ("min-width:120px;max-height:70px",[120.,70.]),
            ("min-width:120px;max-width:48px",[120.,80.]),
            ("min-height:96px;max-height:32px",[144.,96.]),
            ("max-width:0px",[0.,0.]),
        ] { assert_eq!(size(css),expected,"{css}"); }
    }
    #[test]
    fn authored_fixed_axis_is_clamped_before_ratio_and_is_not_changed_by_auto_axis_bounds() {
        for (css, expected) in [
            ("width:120px;max-width:60px",[60.,40.]),
            ("width:120px;max-height:40px",[120.,40.]),
            ("width:120px;min-height:120px",[120.,120.]),
            ("height:80px;max-height:40px",[60.,40.]),
            ("height:80px;max-width:60px",[60.,80.]),
            ("height:80px;min-width:180px",[180.,80.]),
            ("width:120px;min-width:150px;max-width:60px;max-height:40px",[150.,40.]),
            ("width:120px;height:80px;max-width:60px",[60.,80.]),
        ] { assert_eq!(size(css),expected,"{css}"); }
    }
    #[test]
    fn padding_constraints_use_the_authored_box_and_keep_content_precision() {
        assert_eq!(size("width:120px;max-width:60px;padding:8px 10px;box-sizing:content-box"),[60.,40.]);
        let border = resolved("width:120px;max-width:60px;padding:8px 10px;box-sizing:border-box");
        assert_eq!(border.content[0].native(),40.);
        assert!((border.content[1].native()-26.666666).abs()<0.00001);
        assert_eq!(size("width:160px;height:120px;padding:80px;max-width:20px;max-height:20px"),[0.,0.]);
        let tiny = resolved("box-sizing:content-box;width:.03125px;height:.03125px;padding-left:1000000px;max-width:.0625px");
        assert_eq!(tiny.content[0].native(),0.03125);
        assert!(matches!(tiny.content_owner().unwrap().sizes[0],Size::Pixels(v) if v==0.03125));
        assert_eq!(tiny.border[0].native(),1000000.);
    }
    #[test]
    fn folded_numeric_provenance_keeps_authored_decimal_and_ratio_error() {
        let result=resolved("width:47.99px;max-width:100px");
        let ideal = 47.99_f64 * 64. / 96.;
        assert!(result.content[1].ideal_bounds().lower() <= ideal);
        assert!(result.content[1].ideal_bounds().upper() >= ideal);
        assert!(result.content[1].absolute_error_upper()>0.);
        let lowered=result.lower_outer(&style("width:47.99px;max-width:100px"));
        let NumericSize::Pixels(Ok(scalar))=&lowered.numeric.height else {panic!("missing derived provenance")};
        assert!(scalar.proves_equal(&result.border[1]));
    }
    #[test]
    fn unsupported_dependencies_and_missing_provenance_remain_closed() {
        for css in ["width:50%;max-width:100px","max-width:50%","min-width:auto;max-width:100px", "max-width:100px;padding:5%", "align-self:stretch;max-width:100px"] {
            assert!(resolve(&style(css),&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").is_err(),"{css}");
        }
        let mut style=style("width:120px;max-width:100px");
        style.numeric.width=NumericSize::Auto;
        assert!(resolve(&style,&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").is_err());
    }
    #[test]
    fn tiny_nonzero_minimum_is_not_laundered_as_the_zero_default() {
        let authored=style("min-width:1e-50px");
        assert!(has_constraints(&authored));
        let result=resolve(&authored,&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").unwrap().unwrap();
        assert_eq!(result.content[0].native(),96.);
    }
    #[test]
    fn finite_final_constraint_does_not_hide_derived_ratio_overflow() {
        let mut authored=style("width:120px;max-height:40px");
        authored.width=Size::Pixels(f32::MAX);
        authored.numeric.width=NumericSize::Pixels(exact(f32::MAX).map_err(Into::into));
        assert!(resolve(&authored,&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").is_err());
    }
    #[test]
    fn unconstrained_images_do_not_request_a_new_plan() {
        assert!(resolve(&style("width:120px;height:80px;padding:8px"),&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").unwrap().is_none());
    }
}
