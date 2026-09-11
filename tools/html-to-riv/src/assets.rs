use crate::{
    Asset, Diagnostic,
    wire::{Record, Value},
};
use std::collections::BTreeMap;

pub(crate) fn write(
    assets: &BTreeMap<String, Asset>,
    records: &mut Vec<Record>,
) -> Result<(), Diagnostic> {
    if assets.len() > 128 {
        return Err(Diagnostic::new(
            "asset-limit",
            "assets",
            "At most 128 assets are supported",
        ));
    }
    let mut total = 0usize;
    let mut faces = std::collections::BTreeSet::new();
    for (index, (name, asset)) in assets.iter().enumerate() {
        let asset_bytes = match asset {
            Asset::Font { bytes, .. } | Asset::Image { bytes } => bytes,
        };
        total += asset_bytes.len();
        if asset_bytes.len() > 16 * 1024 * 1024 || total > 32 * 1024 * 1024 {
            return Err(Diagnostic::new(
                "asset-limit",
                name,
                "Asset exceeds 16 MiB or document exceeds 32 MiB of assets",
            ));
        }
        let (bytes, kind) = match asset {
            Asset::Font {
                family,
                weight,
                bytes,
            } => {
                if family.trim().is_empty() || !faces.insert((family.to_ascii_lowercase(), *weight))
                {
                    return Err(Diagnostic::new(
                        "duplicate-font",
                        name,
                        "Empty family or duplicate family/weight font face",
                    ));
                }
                let face = ttf_parser::Face::parse(bytes, 0).map_err(|_| {
                    Diagnostic::new(
                        "invalid-font",
                        name,
                        "Expected a valid OpenType/TrueType font",
                    )
                })?;
                if face.is_variable() || face.is_italic() || face.weight().to_number() != *weight {
                    return Err(Diagnostic::new(
                        "unsupported-font",
                        name,
                        "Use a static upright font matching the declared weight",
                    ));
                }
                (bytes, "FontAsset")
            }
            Asset::Image { bytes } => {
                let decoder = png::Decoder::new_with_limits(
                    std::io::Cursor::new(bytes),
                    png::Limits {
                        bytes: 64 * 1024 * 1024,
                    },
                );
                let mut reader = decoder.read_info().map_err(|_| {
                    Diagnostic::new("invalid-image", name, "Expected a valid PNG image")
                })?;
                let info = reader.info();
                if info.width > 4096
                    || info.height > 4096
                    || info.animation_control.is_some()
                    || info.icc_profile.is_some()
                    || info.gama_chunk.is_some()
                    || info.chrm_chunk.is_some()
                    || info.coding_independent_code_points.is_some()
                    || info.mastering_display_color_volume.is_some()
                    || info.content_light_level.is_some()
                    || info.exif_metadata.is_some()
                    || info.bit_depth != png::BitDepth::Eight
                    || !matches!(info.color_type, png::ColorType::Rgb | png::ColorType::Rgba)
                {
                    return Err(Diagnostic::new(
                        "unsupported-image",
                        name,
                        "Use a static RGB/RGBA 8-bit PNG, at most 4096px per side, without color-management or EXIF metadata (an sRGB marker is allowed)",
                    ));
                }
                let size = reader
                    .output_buffer_size()
                    .filter(|n| *n <= 64 * 1024 * 1024)
                    .ok_or_else(|| {
                        Diagnostic::new("asset-limit", name, "Decoded PNG exceeds 64 MiB")
                    })?;
                reader.next_frame(&mut vec![0; size]).map_err(|_| {
                    Diagnostic::new("invalid-image", name, "PNG pixel decode failed")
                })?;
                (bytes, "ImageAsset")
            }
        };
        let mut record = Record::new(kind);
        record.set("name", Value::String(name.clone()))?;
        record.set("assetId", Value::Uint(index as u32))?;
        records.push(record);
        let mut contents = Record::new("FileAssetContents");
        contents.set("bytes", Value::Bytes(bytes.clone()))?;
        records.push(contents);
    }
    Ok(())
}

pub(crate) struct FontSelection {
    pub id: u32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub underline_thickness: Option<f32>,
}

pub(crate) fn font_for(
    assets: &BTreeMap<String, Asset>,
    family: &str,
    weight: u16,
    text: &str,
    source: &str,
) -> Result<FontSelection, Diagnostic> {
    for (index, (_, asset)) in assets.iter().enumerate() {
        if let Asset::Font {
            family: f,
            weight: w,
            bytes,
        } = asset
            && f.eq_ignore_ascii_case(family)
            && *w == weight
        {
            let face = ttf_parser::Face::parse(bytes, 0)
                .map_err(|_| Diagnostic::new("invalid-font", source, "Invalid font"))?;
            if let Some(c) = text.chars().find(|c| {
                (!c.is_whitespace() || *c == '\u{1680}') && face.glyph_index(*c).is_none()
            }) {
                return Err(Diagnostic::new(
                    "missing-glyph",
                    source,
                    format!("Font {family} has no glyph for {c:?}; font fallback is unsupported"),
                ));
            }
            if text.contains('\t') {
                let space = face.glyph_index(' ');
                if !space.is_some_and(|glyph| {
                    face.glyph_hor_advance(glyph)
                        .is_some_and(|advance| advance > 0)
                        && face.glyph_bounding_box(glyph).is_none()
                }) {
                    return Err(Diagnostic::new(
                        "unsupported-tab-font",
                        source,
                        "Preserved tabs require a positive-width, inkless space glyph in the embedded font",
                    ));
                }
            }
            let units = f32::from(face.units_per_em());
            return Ok(FontSelection {
                id: index as u32,
                underline_thickness: face
                    .underline_metrics()
                    .map(|m| f32::from(m.thickness) / units),
                line_gap: f32::from(face.line_gap()) / units,
                ascent: (f32::from(face.ascender()) * 2048.0 / units).round() / 2048.0,
                descent: (-f32::from(face.descender()) * 2048.0 / units).round() / 2048.0,
            });
        }
    }
    Err(Diagnostic::new(
        "missing-font",
        source,
        format!("Supply an embedded font for family {family:?}, weight {weight}"),
    ))
}


/// Read metadata outside the recursive element compiler so PNG decoder state
/// does not enlarge every element's stack frame. Full validation runs in write.
pub(crate) fn image_dimensions(bytes: &[u8], source: &str) -> Result<[u32; 2], Diagnostic> {
    let reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info()
        .map_err(|_| Diagnostic::new("invalid-image", source, "Expected a valid PNG image"))?;
    Ok([reader.info().width, reader.info().height])
}
