//! Bytes-only ordinary embedded-font experiment, not public text admission.
//! Usage: ordinary-text OUTPUT.riv FONT.ttf [FONT_SIZE [Y [LINE_HEIGHT [STROKE_WIDTH [EXTRA_FILL_ALPHA [TEXT [X]]]]]]]
//! Defaults: font32, x20/y20, top origin, AutoWidth; lineHeight omitted.
//! LINE_HEIGHT may be "normal" to omit it; stroke defaults0, bounded0..fontSize.
//! Stroke is an unqualified ordinary-file glyph-coverage experiment.
//! TEXT defaults to "aaaa" (the subset fixture only has U+0061).
//! Optional TEXT is nonempty UTF-8 up to8192bytes; X defaults20, finite0..16384.
//! No post-import font setters, runtime policies or metadata are emitted.
use nuxie_html_to_riv::Diagnostic;
#[allow(dead_code)]
#[path = "../src/wire.rs"]
mod wire;
use wire::{Record, Value};

fn number(value: Option<&std::ffi::OsString>, default: f32, name: &str, minimum: f32) -> Result<f32, Box<dyn std::error::Error>> {
    let parsed = match value {
        Some(value) => value.to_str().ok_or("numeric argument is not UTF-8")?.parse::<f32>()?,
        None => default,
    };
    if !parsed.is_finite() || parsed < minimum || parsed > 16384. {
        return Err(format!("{name} must be finite and in [{minimum},16384]").into());
    }
    Ok(parsed)
}

fn paint(parent: u32, color: u32, fill_id: u32) -> Result<[Record; 2], Diagnostic> {
    let mut fill = Record::new("Fill");
    fill.set("parentId", Value::Uint(parent))?;
    let mut solid = Record::new("SolidColor");
    solid.set("parentId", Value::Uint(fill_id))?;
    solid.set("colorValue", Value::Color(color))?;
    Ok([fill, solid])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(2..=9).contains(&args.len()) {
        return Err("usage: ordinary-text OUTPUT.riv FONT.ttf [FONT_SIZE [Y [LINE_HEIGHT [STROKE_WIDTH [EXTRA_FILL_ALPHA [TEXT [X]]]]]]]".into());
    }
    let size = number(args.get(2), 32., "FONT_SIZE", f32::MIN_POSITIVE)?;
    let y = number(args.get(3), 20., "Y", 0.)?;
    let line_height = match args.get(4) {
        None => None,
        Some(value) if value == "normal" => None,
        Some(value) => Some(number(Some(value), 0., "LINE_HEIGHT", f32::MIN_POSITIVE)?),
    };
    let stroke_width = number(args.get(5), 0., "STROKE_WIDTH", 0.)?;
    if stroke_width > size {
        return Err("STROKE_WIDTH must be at most FONT_SIZE".into());
    }
    let extra_fill_alpha = number(args.get(6), 0., "EXTRA_FILL_ALPHA", 0.)?;
    if extra_fill_alpha > 1. { return Err("EXTRA_FILL_ALPHA must be in [0,1]".into()); }
    let content = match args.get(7) {
        Some(value) => value.to_str().ok_or("TEXT must be valid UTF-8")?,
        None => "aaaa",
    };
    if content.is_empty() || content.len() > 8192 {
        return Err("TEXT must contain 1 through 8192 UTF-8 bytes".into());
    }
    let x = number(args.get(8), 20., "X", 0.)?;
    let font_path = std::path::Path::new(&args[1]);
    let length = std::fs::metadata(font_path)?.len();
    if length == 0 || length > 16 * 1024 * 1024 {
        return Err("FONT.ttf must contain 1 byte through 16 MiB".into());
    }
    let font = std::fs::read(font_path)?;
    if font.is_empty() || font.len() > 16 * 1024 * 1024 {
        return Err("font size changed beyond the supported limit during read".into());
    }
    let mut records = vec![Record::new("Backboard")];
    let mut asset = Record::new("FontAsset");
    asset.set("name", Value::String("Embedded ordinary text font".into()))?;
    asset.set("assetId", Value::Uint(1))?;
    records.push(asset);
    let mut contents = Record::new("FileAssetContents");
    contents.set("bytes", Value::Bytes(font))?;
    records.push(contents);

    // New artboard namespace: preceding global asset records consume no local
    // IDs. Artboard0, whiteFill1/color2, Text3, style4, fill5/color6, run7.
    let mut artboard = Record::new("Artboard");
    artboard.set("name", Value::String("Ordinary embedded text".into()))?;
    artboard.set("width", Value::Float(240.))?;
    artboard.set("height", Value::Float(100.))?;
    records.push(artboard);
    records.extend(paint(0, 0xffffffff, 1)?);
    let mut text = Record::new("Text");
    text.set("parentId", Value::Uint(0))?;
    text.set("name", Value::String("text".into()))?;
    text.set("x", Value::Float(x))?;
    text.set("y", Value::Float(y))?;
    text.set("sizingValue", Value::Uint(0))?;
    text.set("originValue", Value::Uint(0))?;
    text.set("originX", Value::Float(0.))?;
    text.set("originY", Value::Float(0.))?;
    records.push(text);
    let mut style = Record::new("TextStylePaint");
    style.set("parentId", Value::Uint(3))?;
    // Font asset index0 in the global assets vector, independent of assetId1.
    style.set("fontAssetId", Value::Uint(0))?;
    style.set("fontSize", Value::Float(size))?;
    style.set("letterSpacing", Value::Float(0.))?;
    if let Some(value) = line_height { style.set("lineHeight", Value::Float(value))?; }
    records.push(style);
    records.extend(paint(4, 0xff000000, 5)?);
    let mut run = Record::new("TextValueRun");
    run.set("parentId", Value::Uint(3))?;
    run.set("styleId", Value::Uint(4))?;
    run.set("text", Value::String(content.into()))?;
    records.push(run);
    if stroke_width > 0. {
        // Append after run7, preserving all existing local IDs. Ordinary paint
        // composition only: no change to glyph rasterization or runtime policy.
        let mut stroke = Record::new("Stroke");
        stroke.set("parentId", Value::Uint(4))?;
        stroke.set("thickness", Value::Float(stroke_width))?;
        records.push(stroke); // local8
        let mut color = Record::new("SolidColor");
        color.set("parentId", Value::Uint(8))?;
        color.set("colorValue", Value::Color(0xff000000))?;
        records.push(color); // local9
    }
    if extra_fill_alpha > 0. {
        // Candidate edge-coverage composition, not a qualified text mapping.
        let fill_id = if stroke_width > 0. { 10 } else { 8 };
        let alpha = (extra_fill_alpha * 255.).round() as u32;
        records.extend(paint(4, alpha << 24, fill_id)?);
    }
    std::fs::write(&args[0], wire::encode(&records)?)?;
    Ok(())
}
