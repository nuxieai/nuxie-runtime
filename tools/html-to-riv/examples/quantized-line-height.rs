//! Source/font-metric-driven ordinary line-box experiment, never public admission.
//! Usage: quantized-line-height REQUEST.json FONT.ttf OUTPUT.riv
//! The source plan retains an inherited unitless multiplier separately from font
//! size; it never reads browser output, text positions or a precomputed line count.
use nuxie_html_to_riv::Diagnostic;
use serde::Deserialize;
#[allow(dead_code)]
#[path = "../src/wire.rs"]
mod wire;
use wire::{Record, Value};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    parent_font_size: f32,
    child_font_size: f32,
    line_height: SourceHeight,
    y: f32,
    text: String,
    wrap: bool,
    mode: String,
}
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "lowercase",
    deny_unknown_fields
)]
enum SourceHeight {
    Number(f64),
    Px(f64),
}
// Pinned Chromium153 source operations; bounded finite values avoid saturation.
// Platform font metrics are separately restricted to this unchanged Roboto fixture.
fn used_height(source: &SourceHeight, font_size: f32) -> i32 {
    let scaled = match *source {
        SourceHeight::Number(number) => {
            let percent = (number * 100.0_f64) as f32;
            let font_fixed = (font_size * 64.0_f32).round() / 64.0_f32;
            let product = font_fixed * percent;
            ((product / 100.0_f32) * 64.0_f32).trunc()
        }
        SourceHeight::Px(px) => ((px as f32) * 64.0_f32).round(),
    };
    scaled as i32
}
fn source_baseline(ascent: f32, descent: f32, height_raw: i32) -> f32 {
    let ascent_integer = ascent.round() as i32;
    let descent_integer = descent.round() as i32;
    let leading_raw = height_raw - (ascent_integer + descent_integer) * 64;
    // LayoutUnit divides the signed raw integer by2 before Floor. For negative odd
    // raw values this can differ from flooring a real-valued half-leading.
    ascent_integer as f32 + (leading_raw / 2).div_euclid(64) as f32
}
fn set(r: &mut Record, name: &str, v: f32) -> Result<(), Diagnostic> {
    r.set(name, Value::Float(v))
}
fn uint(r: &mut Record, name: &str, v: u32) -> Result<(), Diagnostic> {
    r.set(name, Value::Uint(v))
}
fn table<'a>(font: &'a [u8], tag: &[u8; 4]) -> Result<&'a [u8], Box<dyn std::error::Error>> {
    let count = u16::from_be_bytes(font.get(4..6).ok_or("short sfnt")?.try_into()?) as usize;
    for i in 0..count {
        let r = font
            .get(12 + i * 16..28 + i * 16)
            .ok_or("short table directory")?;
        if &r[..4] == tag {
            let off = u32::from_be_bytes(r[8..12].try_into()?) as usize;
            let len = u32::from_be_bytes(r[12..16].try_into()?) as usize;
            return Ok(font.get(off..off + len).ok_or("short table")?);
        }
    }
    Err("missing table".into())
}
fn paint(records: &mut Vec<Record>, parent: u32, id: u32, color: u32) -> Result<(), Diagnostic> {
    let mut f = Record::new("Fill");
    uint(&mut f, "parentId", parent)?;
    records.push(f);
    let mut c = Record::new("SolidColor");
    uint(&mut c, "parentId", id)?;
    c.set("colorValue", Value::Color(color))?;
    records.push(c);
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("REQUEST.json FONT.ttf OUTPUT.riv".into());
    }
    let req: Request = serde_json::from_slice(&std::fs::read(&args[0])?)?;
    let font = std::fs::read(&args[1])?;
    if font.len() > 16 * 1024 * 1024 || req.text.is_empty() || req.text.len() > 8192 {
        return Err("bounded fixture required".into());
    }
    for n in [req.parent_font_size, req.child_font_size, req.y] {
        if !n.is_finite() || n < 0. || n > 128. {
            return Err("bounded fixture coefficients required".into());
        }
    }
    if req.child_font_size == 0. || req.parent_font_size == 0. {
        return Err("positive font size required".into());
    }
    let upem = u16::from_be_bytes(table(&font, b"head")?[18..20].try_into()?) as f32;
    let hhea = table(&font, b"hhea")?;
    let ascent = i16::from_be_bytes(hhea[4..6].try_into()?) as f32 / upem * req.child_font_size;
    let descent = -(i16::from_be_bytes(hhea[6..8].try_into()?) as f32) / upem * req.child_font_size;
    let source_value = match req.line_height {
        SourceHeight::Number(v) | SourceHeight::Px(v) => v,
    };
    if !source_value.is_finite() || source_value < 0. || source_value > 128. {
        return Err("bounded source height required".into());
    }
    let height_raw = used_height(&req.line_height, req.child_font_size);
    let line_height = height_raw as f32 / 64.;
    // Hypotheses differ only in the source-font ascent/descent quantization. The
    // rounded variant is a finite Chrome153/Roboto candidate, not a universal rule.
    let baseline = match req.mode.as_str() {
        "chromium-leading" => source_baseline(ascent, descent, height_raw),
        "half-leading" => ascent + (line_height - ascent - descent) / 2.,
        "rounded-leading" => (line_height + ascent.round() - descent.round()) / 2.,
        "direct" => ascent,
        _ => return Err("unknown mode".into()),
    };
    let (top, bottom) = if req.mode == "direct" {
        (0., 0.)
    } else {
        (
            baseline - ascent,
            line_height * ascent / (ascent + descent) - baseline,
        )
    };
    // Synthetic ordinary padding may be signed; this is not authored CSS padding.
    if !top.is_finite() || !bottom.is_finite() {
        return Err("nonfinite wrapper".into());
    }
    let mut records = vec![Record::new("Backboard")];
    let mut asset = Record::new("FontAsset");
    asset.set("name", Value::String("Unchanged licensed Roboto".into()))?;
    uint(&mut asset, "assetId", 1)?;
    records.push(asset);
    let mut contents = Record::new("FileAssetContents");
    contents.set("bytes", Value::Bytes(font))?;
    records.push(contents);
    // Local artboard object ids restart after global asset declarations.
    let mut art = Record::new("Artboard");
    set(&mut art, "width", 390.)?;
    set(&mut art, "height", 160.)?;
    uint(&mut art, "styleId", 1)?;
    records.push(art);
    let mut root = Record::new("LayoutComponentStyle");
    uint(&mut root, "flexDirectionValue", 0)?;
    for name in ["paddingTop", "paddingLeft", "paddingRight"] {
        set(
            &mut root,
            name,
            if name == "paddingTop" { req.y } else { 20. },
        )?;
        uint(&mut root, &format!("{name}UnitsValue"), 1)?;
    }
    records.push(root);
    paint(&mut records, 0, 2, 0xffffffff)?;
    let mut owner = Record::new("LayoutComponent");
    owner.set("name", Value::String("line-box-owner".into()))?;
    uint(&mut owner, "parentId", 0)?;
    uint(&mut owner, "styleId", 5)?;
    records.push(owner);
    let mut os = Record::new("LayoutComponentStyle");
    uint(&mut os, "flexDirectionValue", 0)?;
    uint(&mut os, "layoutWidthScaleType", 1)?;
    uint(&mut os, "layoutHeightScaleType", 2)?;
    uint(&mut os, "widthUnitsValue", 3)?;
    uint(&mut os, "heightUnitsValue", 3)?;
    for (name, v) in [("paddingTop", top), ("paddingBottom", bottom)] {
        set(&mut os, name, v)?;
        uint(&mut os, &format!("{name}UnitsValue"), 1)?;
    }
    records.push(os);
    let mut text = Record::new("Text");
    uint(&mut text, "parentId", 4)?;
    text.set("name", Value::String("text".into()))?;
    uint(&mut text, "sizingValue", 1)?;
    set(&mut text, "width", 200.)?;
    uint(&mut text, "originValue", 0)?;
    set(&mut text, "originX", 0.)?;
    set(&mut text, "originY", 0.)?;
    uint(&mut text, "wrapValue", if req.wrap { 0 } else { 1 })?;
    records.push(text);
    let mut participant = Record::new("LayoutParticipant");
    uint(&mut participant, "parentId", 6)?;
    uint(&mut participant, "layoutWidthScaleType", 1)?;
    uint(&mut participant, "layoutHeightScaleType", 2)?;
    uint(&mut participant, "widthUnitsValue", 3)?;
    uint(&mut participant, "heightUnitsValue", 3)?;
    records.push(participant);
    let mut style = Record::new("TextStylePaint");
    uint(&mut style, "parentId", 6)?;
    uint(&mut style, "fontAssetId", 0)?;
    set(&mut style, "fontSize", req.child_font_size)?;
    set(&mut style, "lineHeight", line_height)?;
    records.push(style);
    paint(&mut records, 8, 9, 0xff000000)?;
    let mut run = Record::new("TextValueRun");
    uint(&mut run, "parentId", 6)?;
    uint(&mut run, "styleId", 8)?;
    run.set("text", Value::String(req.text))?;
    records.push(run);
    std::fs::write(&args[2], wire::encode(&records)?)?;
    println!(
        "{}",
        serde_json::json!({"fontAscent":ascent,"fontDescent":descent,"usedLineHeight":line_height,"usedLineHeightRaw":height_raw,"candidateBaseline":baseline,"paddingTop":top,"paddingBottom":bottom,"textObjectId":6,"ownerObjectId":4,"browserMeasurementsConsumed":false,"lineCountConsumed":false})
    );
    Ok(())
}
