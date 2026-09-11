//! Ordinary-file capability experiment, not public HTML/CSS admission.
//! Emits only baseline schema objects. No sidecar controls rendering.
use nuxie_html_to_riv::Diagnostic;
#[allow(dead_code)]
#[path = "../src/wire.rs"]
mod wire;
#[path = "../src/color.rs"]
mod color;
use wire::{Record, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: ordinary-layout OUTPUT.riv COLOR solid|fractional|nested|corners|corners-limit|corners-wrapper|alpha".into());
    }
    let color = color::parse(&args[2], "experiment color")?;
    let mode = &args[3];
    if !["solid", "fractional", "nested", "corners", "corners-limit", "corners-wrapper", "alpha"].contains(&mode.as_str()) {
        return Err("unknown experiment".into());
    }
    let mut records = vec![Record::new("Backboard")];
    let mut artboard = Record::new("Artboard");
    artboard.set("name", Value::String("Ordinary file capability experiment".into()))?;
    artboard.set("width", Value::Float(390.))?;
    artboard.set("height", Value::Float(160.))?;
    artboard.set("styleId", Value::Uint(1))?;
    records.push(artboard);
    let mut root_style = Record::new("LayoutComponentStyle");
    root_style.set("flexDirectionValue", Value::Uint(0))?;
    records.push(root_style);

    // IDs are local to the artboard: its style is 1, first layout is 2.
    let outer = add_box(&mut records, 0, "outer", 100., 2, 100., 2, if mode == "corners-wrapper" { 0 } else { color }, if mode == "corners-limit" { 2 } else if mode == "corners" { 1 } else { 0 })?;
    if mode == "corners-wrapper" {
        // Ordinary file composition: clip an oversized rounded paint child.
        // This experiment requires CSS min-height:100px on the outer box.
        records[outer as usize + 1].set("clip", Value::Bool(true))?;
        records[outer as usize + 2].set("minHeight", Value::Float(100.))?;
        records[outer as usize + 2].set("minHeightUnitsValue", Value::Uint(1))?;
        let paint = add_box(&mut records, outer, "corner-paint", 100., 2, 100., 2, color, 2)?;
        records[paint as usize + 2].set("minHeight", Value::Float(200.))?;
        records[paint as usize + 2].set("minHeightUnitsValue", Value::Uint(1))?;
    }
    if mode == "nested" || mode == "fractional" {
        let (width, units) = if mode == "fractional" { (80.5, 1) } else { (50., 2) };
        add_box(&mut records, outer, "inner", width, units, 50., 2, 0xff663399, 0)?;
    }
    std::fs::write(&args[1], wire::encode(&records)?)?;
    Ok(())
}

fn add_box(records: &mut Vec<Record>, parent: u32, name: &str,
    width: f32, width_units: u32, height: f32, height_units: u32,
    color: u32, corners: u8,
) -> Result<u32, Diagnostic> {
    let id = records.len() as u32 - 1;
    let mut layout = Record::new("LayoutComponent");
    layout.set("name", Value::String(name.into()))?;
    layout.set("parentId", Value::Uint(parent))?;
    layout.set("styleId", Value::Uint(id + 1))?;
    let mut style = Record::new("LayoutComponentStyle");
    for (axis, value, units) in [("width", width, width_units), ("height", height, height_units)] {
        layout.set(axis, Value::Float(value))?;
        style.set(&format!("{axis}UnitsValue"), Value::Uint(units))?;
        style.set(if axis == "width" { "layoutWidthScaleType" } else { "layoutHeightScaleType" }, Value::Uint(0))?;
    }
    style.set("flexDirectionValue", Value::Uint(0))?;
    if corners > 0 {
        style.set("linkCornerRadius", Value::Bool(false))?;
        let radii = if corners == 2 { [100., 100., 0., 0.] } else { [8., 20., 32., 4.] };
        for (name, value) in ["cornerRadiusTL", "cornerRadiusTR", "cornerRadiusBR", "cornerRadiusBL"].into_iter().zip(radii) {
            style.set(name, Value::Float(value))?;
        }
    }
    records.push(layout);records.push(style);
    let fill_id = records.len() as u32 - 1;
    let mut fill = Record::new("Fill");fill.set("parentId", Value::Uint(id))?;
    records.push(fill);
    let mut paint = Record::new("SolidColor");paint.set("parentId", Value::Uint(fill_id))?;
    paint.set("colorValue", Value::Color(color))?;records.push(paint);
    Ok(id)
}
