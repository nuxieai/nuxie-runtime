//! Existing Image objects and their ordinary layout owners. No host asset loader.
use super::{Diagnostic, ElementRef, Record, Size, Style, Value, unsupported};
use crate::assets::{AssetTable, ImageMetadata};

#[derive(Clone, Copy)]
pub(super) struct Paint { fit: u32, position: [f32; 2], nearest: bool }
impl Default for Paint {
    fn default() -> Self { Self { fit: 7, position: [0.5; 2], nearest: false } }
}
impl Paint {
    pub fn inherited(parent: Self) -> Self { Self { nearest: parent.nearest, ..Self::default() } }
    pub fn apply(&mut self, property: &str, value: &str, parent: Self, source: &str) -> Result<(), Diagnostic> {
        let value = crate::css_whitespace::trim(value).to_ascii_lowercase();
        match property {
            "object-fit" => self.fit = match value.as_str() {
                "fill" | "initial" | "unset" => 7, "contain" => 1, "cover" => 2,
                "none" => 5, "scale-down" => 6, "inherit" => parent.fit,
                _ => return Err(unsupported(source, "object-fit requires fill, contain, cover, none or scale-down")),
            },
            "object-position" => self.position = match value.as_str() {
                "inherit" => parent.position, "initial" | "unset" => [0.5; 2],
                _ => position(&value, source)?,
            },
            "image-rendering" => self.nearest = match value.as_str() {
                "auto" | "initial" => false, "pixelated" => true, "inherit" | "unset" => parent.nearest,
                _ => return Err(unsupported(source, "image-rendering admits auto or pixelated; other sampling modes need qualification")),
            },
            _ => unreachable!(),
        }
        Ok(())
    }
}
fn position(text: &str, source: &str) -> Result<[f32; 2], Diagnostic> {
    let mut input = cssparser::ParserInput::new(text);
    let mut parser = cssparser::Parser::new(&mut input);
    let mut parts = Vec::new();
    while !parser.is_exhausted() {
        parser.skip_whitespace();
        let start = parser.position();
        let token = parser.next().cloned().map_err(|_| unsupported(source, "Invalid object-position"))?;
        let part = match token {
            cssparser::Token::Ident(v) => match v.as_ref() {
                "left" => (0., 0), "right" => (1., 0), "top" => (0., 1), "bottom" => (1., 1), "center" => (0.5, 2),
                _ => return Err(unsupported(source, "object-position requires position keywords or percentages")),
            },
            cssparser::Token::Percentage { unit_value, .. } if unit_value.is_finite() => {
                let raw = crate::numeric_tokens::number_prefix(parser.slice_from(start));
                let scalar = super::scalar_provenance::ScalarProvenance::from_decimal(raw, unit_value * 100.)
                    .map_err(|_| unsupported(source, "Cannot establish the authored object-position percentage bound"))?;
                let maximum = super::scalar_provenance::ScalarProvenance::exact_constant(100.).expect("exact finite constant");
                if !scalar.is_nonnegative() || (scalar.ideal_bounds().upper() > 100. && !scalar.proves_equal(&maximum)) {
                    return Err(unsupported(source, "Authored object-position percentage must be provably within 0% through 100% before float rounding"));
                }
                (unit_value, 3)
            },
            _ => return Err(unsupported(source, "object-position currently admits keywords and 0% through 100%; lengths, offsets and outside positions need qualification")),
        };
        parts.push(part);
        if parts.len() > 2 { return Err(unsupported(source, "Three/four-value object-position needs separate offset lowering")); }
    }
    match parts.as_slice() {
        [(v, 1)] => Ok([0.5, *v]),
        [(v, _)] => Ok([*v, 0.5]),
        [(x, a), (y, b)] if *a != 1 && *b != 0 => Ok([*x, *y]),
        [(y, a), (x, b)] if (*a == 1 || *a == 2) && (*b == 0 || *b == 2) => Ok([*x, *y]),
        _ => Err(unsupported(source, "Invalid two-axis object-position")),
    }
}

pub(super) struct Plan {
    pub metadata: ImageMetadata,
    pub aspect_axis: Option<usize>,
    padded: Option<PaddedLayout>,
    constraints: Option<super::image_constraints::Resolved>,
}
#[derive(Clone, Copy)]
struct PaddedLayout {
    content: super::box_sizing::ContentOwner,
    outer_stretch_axis: Option<usize>,
}
pub(super) fn plan(element: ElementRef<'_>, style: &mut Style, parent: &Style, assets: &AssetTable, source: &str) -> Result<Option<Plan>, Diagnostic> {
    if element.value().name() != "img" { return Ok(None); }
    let src = element.attr("src").filter(|s| !s.is_empty())
        .ok_or_else(|| Diagnostic::new("missing-image-source", source, "img requires a nonempty src naming an explicitly supplied asset"))?;
    let metadata = assets.get(src)?;
    if style.margins.any() || style.self_alignment.is_baseline()
        || style.self_alignment.wrapper_alignment(parent.direction).is_some() {
        return Err(unsupported(source, "Image automatic margins and alignment wrappers require separate replaced-element qualification"));
    }
    if let Some(constraints) = super::image_constraints::resolve(style, parent, metadata, source)? {
        return Ok(Some(Plan { metadata, aspect_axis: None, padded: None, constraints: Some(constraints) }));
    }
    if !style.padding.is_zero() { return padded_plan(metadata, style, parent, source).map(Some); }
    let auto = [matches!(style.width, Size::Auto), matches!(style.height, Size::Auto)];
    let main = if parent.direction.is_row() { 0 } else { 1 };
    let stretches = style.self_alignment.stretches();
    // Two intrinsic axes need a source-derived size when cross-axis stretching
    // does not supply one. This is encoded asset metadata, never a DOM rectangle.
    if auto == [true, true] && !stretches {
        style.width = Size::Pixels(metadata.width as f32);
        style.height = Size::Pixels(metadata.height as f32);
        style.numeric.width = super::computed_provenance::NumericSize::Pixels(
            super::scalar_provenance::ScalarProvenance::exact_constant(metadata.width as f32).map_err(Into::into));
        style.numeric.height = super::computed_provenance::NumericSize::Pixels(
            super::scalar_provenance::ScalarProvenance::exact_constant(metadata.height as f32).map_err(Into::into));
        return Ok(Some(Plan { metadata, aspect_axis: None, padded: None, constraints: None }));
    }
    if auto == [true, true] && stretches {
        // Resolve the ordinary owner's cross axis before measuring its Image.
        // Native auto+stretch otherwise measures the intrinsic main dimension
        // first and does not revisit it after cross-axis stretching. In this
        // zero-margin/padding, unclamped profile, stretch is the full containing
        // content dimension; a percentage remains responsive inside the file.
        let full = super::computed_provenance::NumericSize::Percent(
            super::scalar_provenance::ScalarProvenance::exact_constant(100.).map_err(Into::into));
        if main == 1 { style.width = Size::Percent(100.); style.numeric.width = full; }
        else { style.height = Size::Percent(100.); style.numeric.height = full; }
    }
    // CSS flex stretch overrides a sole automatic cross axis when the main size
    // is fixed. Automatic main size instead follows the intrinsic aspect ratio.
    let aspect_axis = if auto[main] { Some(main) }
        else if auto[1-main] && !stretches { Some(1-main) } else { None };
    Ok(Some(Plan { metadata, aspect_axis, padded: None, constraints: None }))
}

fn padded_plan(metadata: ImageMetadata, style: &Style, parent: &Style, source: &str) -> Result<Plan, Diagnostic> {
    use super::{Direction, box_sizing::{BoxSizing, ContentOwner}};
    let authored = [style.width, style.height];
    let auto = authored.map(|size| matches!(size, Size::Auto));
    let main = usize::from(!parent.direction.is_row());
    let stretches = style.self_alignment.stretches();
    let intrinsic = auto == [true, true] && !stretches;
    let aspect_axis = if intrinsic { None }
        else if auto[main] { Some(main) }
        else if auto[1-main] && !stretches { Some(1-main) }
        else { None };
    let horizontal_percent = [0, 2].iter().any(|&i|
        matches!(style.padding.sides()[i], super::padding::Inset::Percent(v) if v != 0.));
    if style.padding.has_percentage() && main == 0 && (aspect_axis == Some(0)
        || (horizontal_percent && (auto[0] || style.box_sizing == BoxSizing::ContentBox))) {
        return Err(unsupported(source, "Image row automatic width with percentage padding has unqualified intrinsic-main measurement: the immutable runtime measures padding without the original containing width; use a definite border-box width or remove percentage padding"));
    }
    let packing = match aspect_axis {
        Some(0) => Direction::Row,
        Some(1) => Direction::Column,
        _ if parent.direction.is_row() => Direction::Row,
        _ => Direction::Column,
    };
    let sizes = if intrinsic {
        [Size::Pixels(metadata.width as f32), Size::Pixels(metadata.height as f32)]
    } else {
        std::array::from_fn(|axis| {
            if aspect_axis == Some(axis) { Size::Auto }
            // Keep exact authored point content sizes across outer padding
            // arithmetic. A percentage was already resolved by the outer box;
            // repeating it on the inner would apply the coefficient twice.
            else if style.box_sizing == BoxSizing::ContentBox && matches!(authored[axis], Size::Pixels(_)) { authored[axis] }
            else { Size::Percent(100.) }
        })
    };
    Ok(Plan { metadata, aspect_axis, padded: Some(PaddedLayout {
        content: ContentOwner { packing, sizes, bounds: [Size::Pixels(0.), Size::Pixels(0.), Size::Auto, Size::Auto] },
        outer_stretch_axis: (auto == [true, true] && stretches).then_some(1-main),
    }), constraints: None })
}
impl Plan {
    pub fn lower_outer(&self, style: &Style, source: &str) -> Result<super::box_sizing::Lowered, Diagnostic> {
        if let Some(constraints) = &self.constraints { return Ok(constraints.lower_outer(style)); }
        let mut lowered = if self.padded.is_some() {
            super::box_sizing::lower_image(style, source)?
        } else { super::box_sizing::lower(style, source)? };
        self.adjust_outer(&mut lowered);
        Ok(lowered)
    }
    pub fn outer_stretch(&self, style: &Style, parent: &Style, lowered: &super::box_sizing::Lowered) -> bool {
        // Constraint folding produces two definite axes, including authored
        // automatic main dimensions; none should become native Fill.
        if self.constraints.is_some() { return false; }
        let cross = usize::from(parent.direction.is_row());
        let authored = [style.width, style.height];
        style.self_alignment.stretches() && !(matches!(authored[cross], Size::Pixels(_))
            && matches!(lowered.sizes[cross], Size::Auto))
    }
    /// Synthetic stretch resolves the outer border box before the inner ratio
    /// is measured. It must not become an authored content-box percentage.
    pub fn adjust_outer(&self, lowered: &mut super::box_sizing::Lowered) {
        if let Some(axis) = self.padded.and_then(|p| p.outer_stretch_axis) {
            lowered.sizes[axis] = Size::Percent(100.);
            let numeric = super::computed_provenance::NumericSize::Percent(
                super::scalar_provenance::ScalarProvenance::exact_constant(100.).map_err(Into::into));
            if axis == 0 { lowered.numeric.width = numeric; } else { lowered.numeric.height = numeric; }
        }
    }
    pub fn content_owner(&self) -> Option<super::box_sizing::ContentOwner> {
        self.constraints.as_ref().and_then(|c|c.content_owner())
            .or_else(||self.padded.map(|p| p.content))
    }
    pub fn emit(&self, records: &mut Vec<Record>, owner: u32, style: Paint) -> Result<(), Diagnostic> {
        records[owner as usize + 1].set("clip", Value::Bool(true))?;
        if self.aspect_axis.is_some() {
            records[owner as usize + 2].set("aspectRatio", Value::Float(self.metadata.width as f32 / self.metadata.height as f32))?;
        }
        let id = records.len() as u32 - 1;
        let mut image = Record::new("Image");
        image.set("parentId", Value::Uint(owner))?;
        image.set("assetId", Value::Uint(self.metadata.asset_index))?;
        image.set("originX", Value::Float(0.))?;
        image.set("originY", Value::Float(0.))?;
        image.set("fit", Value::Uint(style.fit))?;
        image.set("alignmentX", Value::Float(style.position[0] * 2. - 1.))?;
        image.set("alignmentY", Value::Float(style.position[1] * 2. - 1.))?;
        if style.nearest { image.set_image_sampler_filter(true)?; }
        records.push(image);
        let mut participant = Record::new("LayoutParticipant");
        participant.set("parentId", Value::Uint(id))?;
        for axis in ["width", "height"] {
            participant.set(&format!("{axis}UnitsValue"), Value::Uint(3))?;
            participant.set(if axis == "width" { "layoutWidthScaleType" } else { "layoutHeightScaleType" }, Value::Uint(1))?;
        }
        records.push(participant);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{box_sizing::BoxSizing, computed_provenance::NumericSize};
    fn computed(css: &str, parent: &Style) -> Style {
        let document = scraper::Html::parse_document("<img id=image>");
        let selector = scraper::Selector::parse("#image").unwrap();
        super::super::computed(document.select(&selector).next().unwrap(),
            &crate::css::stylesheet(css).unwrap(), parent, false, true).unwrap()
    }
    fn layout(css: &str, row: bool) -> (Style, Style, Plan, super::super::box_sizing::Lowered) {
        let mut parent = Style::default();
        if row { parent.direction = super::super::Direction::Row; }
        let style = computed(&format!("#image{{box-sizing:content-box;{css}}}"), &parent);
        let plan = padded_plan(ImageMetadata { asset_index: 0, width: 96, height: 64 }, &style, &parent, "image").unwrap();
        let lowered = plan.lower_outer(&style, "image").unwrap();
        (style, parent, plan, lowered)
    }
    #[test]
    fn synthetic_auto_definite_cross_is_hug_but_authored_auto_cross_still_stretches() {
        let (style,parent,plan,lowered) = layout("width:120px;height:80px;padding:5%", false);
        assert!(matches!(lowered.sizes, [Size::Auto, Size::Auto]));
        assert!(!plan.outer_stretch(&style, &parent, &lowered));
        assert!(matches!(plan.content_owner().unwrap().sizes, [Size::Pixels(120.), Size::Pixels(80.)]));
        assert!(matches!(style.numeric.width, NumericSize::Pixels(Ok(_))));
        assert!(matches!(lowered.numeric.width, NumericSize::Auto));
        let (style,parent,plan,lowered) = layout("width:auto;height:80px;padding:5%", false);
        assert!(plan.outer_stretch(&style, &parent, &lowered));
        assert!(plan.aspect_axis.is_none());
        let (style,parent,plan,lowered) = layout("width:120px;height:auto;padding:5% 0", true);
        assert!(plan.outer_stretch(&style, &parent, &lowered));
        assert!(matches!(lowered.sizes[0], Size::Pixels(120.)));
        assert!(plan.aspect_axis.is_none());
    }
    #[test]
    fn percentage_on_opposite_axis_is_resolved_once_without_laundering_padding_provenance() {
        for (css,axis) in [("width:50%;height:80px;padding:5% 0",0), ("width:120px;height:50%;padding:0 5%",1)] {
            let (style,_,plan,lowered) = layout(css,false);
            assert!(matches!(lowered.sizes[axis], Size::Percent(50.)));
            assert!(matches!(plan.content_owner().unwrap().sizes[axis], Size::Percent(100.)));
            for (before,after) in style.numeric.padding.iter().zip(&lowered.numeric.padding) {
                assert!(before.as_ref().unwrap().proves_equal(after.as_ref().unwrap()));
            }
            assert!(style.box_sizing == BoxSizing::ContentBox);
        }
        let (style,_,_,lowered) = layout("width:50%;height:80px;padding:5% 1e-50%",false);
        assert!(matches!(lowered.numeric.width, NumericSize::Percent(Err(_))));
        assert!(!style.numeric.padding[0].as_ref().unwrap().is_exact_zero());
    }
}
