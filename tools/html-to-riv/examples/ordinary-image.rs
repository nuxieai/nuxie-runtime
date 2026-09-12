//! Source-driven ordinary image/layout experiment. No public asset admission.
//! Usage: ordinary-image REQUEST.json ENCODED_IMAGE OUTPUT.riv
use nuxie_html_to_riv::Diagnostic;
use serde::Deserialize;
#[allow(dead_code)]
#[path = "../src/wire.rs"]
mod wire;
use wire::{Record, Value};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    asset_width: f32,
    asset_height: f32,
    width: Size,
    height: Size,
    aspect_ratio: bool,
    fit: u32,
    alignment_x: f32,
    alignment_y: f32,
    clip: bool,
    nearest: bool,
}
#[derive(Deserialize)]
#[serde(
    tag = "unit",
    content = "value",
    rename_all = "lowercase",
    deny_unknown_fields
)]
enum Size {
    Px(f32),
    Percent(f32),
    Auto,
}
fn f(r: &mut Record, key: &str, value: f32) -> Result<(), Diagnostic> {
    r.set(key, Value::Float(value))
}
fn u(r: &mut Record, key: &str, value: u32) -> Result<(), Diagnostic> {
    r.set(key, Value::Uint(value))
}
fn sizing(
    owner: &mut Record,
    style: &mut Record,
    axis: &str,
    size: &Size,
) -> Result<(), Diagnostic> {
    let (value, unit, scale) = match size {
        Size::Px(v) => (*v, 1, 0),
        Size::Percent(v) => (*v, 2, 0),
        Size::Auto => (0., 3, 2),
    };
    if !value.is_finite() || !(0.0..=4096.0).contains(&value) {
        return Err(Diagnostic::new(
            "invalid-fixture",
            axis,
            "finite bounded image size required",
        ));
    }
    f(owner, axis, value)?;
    u(style, &format!("{axis}UnitsValue"), unit)?;
    u(
        style,
        if axis == "width" {
            "layoutWidthScaleType"
        } else {
            "layoutHeightScaleType"
        },
        scale,
    )
}
fn paint(
    records: &mut Vec<Record>,
    parent: u32,
    fill_id: u32,
    color: u32,
) -> Result<(), Diagnostic> {
    let mut fill = Record::new("Fill");
    u(&mut fill, "parentId", parent)?;
    records.push(fill);
    let mut color_record = Record::new("SolidColor");
    u(&mut color_record, "parentId", fill_id)?;
    color_record.set("colorValue", Value::Color(color))?;
    records.push(color_record);
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("REQUEST.json ENCODED_IMAGE OUTPUT.riv".into());
    }
    let req: Request = serde_json::from_slice(&std::fs::read(&args[0])?)?;
    let bytes = std::fs::read(&args[1])?;
    if bytes.is_empty() || bytes.len() > 1024 * 1024 {
        return Err("bounded encoded fixture required".into());
    }
    for v in [req.asset_width, req.asset_height] {
        if !v.is_finite() || !(1.0..=4096.0).contains(&v) {
            return Err("bounded fixture dimensions required".into());
        }
    }
    if req.fit > 7
        || ![req.alignment_x, req.alignment_y]
            .iter()
            .all(|v| v.is_finite() && (-3.0..=3.0).contains(v))
    {
        return Err("bounded fit/alignment required".into());
    }
    let mut records = vec![Record::new("Backboard")];
    let mut asset = Record::new("ImageAsset");
    asset.set(
        "name",
        Value::String("Authored encoded image fixture".into()),
    )?;
    u(&mut asset, "assetId", 1)?;
    f(&mut asset, "width", req.asset_width)?;
    f(&mut asset, "height", req.asset_height)?;
    records.push(asset);
    let mut contents = Record::new("FileAssetContents");
    contents.set("bytes", Value::Bytes(bytes))?;
    records.push(contents);
    let mut art = Record::new("Artboard");
    f(&mut art, "width", 390.)?;
    f(&mut art, "height", 240.)?;
    u(&mut art, "styleId", 1)?;
    records.push(art);
    let mut root = Record::new("LayoutComponentStyle");
    u(&mut root, "flexDirectionValue", 0)?;
    for name in ["paddingTop", "paddingLeft", "paddingRight", "paddingBottom"] {
        f(&mut root, name, 20.)?;
        u(&mut root, &format!("{name}UnitsValue"), 1)?;
    }
    records.push(root);
    paint(&mut records, 0, 2, 0xffffffff)?;
    let mut owner = Record::new("LayoutComponent");
    owner.set("name", Value::String("image-box".into()))?;
    u(&mut owner, "parentId", 0)?;
    u(&mut owner, "styleId", 5)?;
    owner.set("clip", Value::Bool(req.clip))?;
    let mut style = Record::new("LayoutComponentStyle");
    u(&mut style, "flexDirectionValue", 0)?;
    sizing(&mut owner, &mut style, "width", &req.width)?;
    sizing(&mut owner, &mut style, "height", &req.height)?;
    if req.aspect_ratio {
        f(
            &mut style,
            "aspectRatio",
            req.asset_width / req.asset_height,
        )?;
    }
    records.push(owner);
    records.push(style);
    paint(&mut records, 4, 6, 0xffe9f0f6)?;
    let mut image = Record::new("Image");
    image.set("name", Value::String("authored-image".into()))?;
    u(&mut image, "parentId", 4)?;
    u(&mut image, "assetId", 0)?;
    f(&mut image, "originX", 0.)?;
    f(&mut image, "originY", 0.)?;
    u(&mut image, "fit", req.fit)?;
    f(&mut image, "alignmentX", req.alignment_x)?;
    f(&mut image, "alignmentY", req.alignment_y)?;
    if req.nearest {
        image.set_image_sampler_filter(true)?;
    }
    records.push(image);
    let mut participant = Record::new("LayoutParticipant");
    u(&mut participant, "parentId", 8)?;
    for (axis, size) in [("width", &req.width), ("height", &req.height)] {
        u(&mut participant, &format!("{axis}UnitsValue"), 3)?;
        let hug = matches!(size, Size::Auto) && !req.aspect_ratio;
        u(
            &mut participant,
            if axis == "width" {
                "layoutWidthScaleType"
            } else {
                "layoutHeightScaleType"
            },
            if hug { 2 } else { 1 },
        )?;
    }
    records.push(participant);
    std::fs::write(&args[2], wire::encode(&records)?)?;
    println!(
        "{}",
        serde_json::json!({"ownerObjectId":4,"imageObjectId":8,"participantObjectId":9,"globalAssetIndex":0,"sourceAssetWidth":req.asset_width,"sourceAssetHeight":req.asset_height,"aspectRatioFromSourceAsset":req.aspect_ratio.then_some(req.asset_width/req.asset_height),"browserGeometryConsumed":false})
    );
    Ok(())
}
