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

pub(super) struct Plan { pub metadata: ImageMetadata, pub aspect_axis: Option<usize> }
pub(super) fn plan(element: ElementRef<'_>, style: &mut Style, parent: &Style, assets: &AssetTable, source: &str) -> Result<Option<Plan>, Diagnostic> {
    if element.value().name() != "img" { return Ok(None); }
    let src = element.attr("src").filter(|s| !s.is_empty())
        .ok_or_else(|| Diagnostic::new("missing-image-source", source, "img requires a nonempty src naming an explicitly supplied asset"))?;
    let metadata = assets.get(src)?;
    if !style.padding.is_zero() || style.margins.any() || style.self_alignment.is_baseline()
        || style.self_alignment.wrapper_alignment(parent.direction).is_some()
        || !matches!(style.min_width, Size::Pixels(0.)) || !matches!(style.min_height, Size::Pixels(0.))
        || !matches!(style.max_width, Size::Auto) || !matches!(style.max_height, Size::Auto) {
        return Err(unsupported(source, "Image padding, min/max constraints, automatic margins and alignment wrappers require separate replaced-element qualification"));
    }
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
        return Ok(Some(Plan { metadata, aspect_axis: None }));
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
    Ok(Some(Plan { metadata, aspect_axis }))
}
impl Plan {
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
