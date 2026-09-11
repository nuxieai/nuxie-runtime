//! Diagnostic only: normalized runtime outlines and CSS stripe intercepts.
use nuxie_runtime::source::{
    text::css_decoration::glyph_stripe_intercept,
    text::font_hb::{HbFont, ShapingPrecision},
    text_engine::TextRun,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let bytes = std::fs::read(args.get(1).ok_or("font path required")?)?;
    let mut font = HbFont::decode(&bytes).ok_or("invalid font")?;
    let size: f32 = args.get(3).map(|v| v.parse()).transpose()?.unwrap_or(24.0);
    if !size.is_finite() || size <= 0.0 {
        return Err("positive finite size required".into());
    }
    if args.get(4).is_some_and(|v| v == "css") {
        font = font
            .as_any()
            .downcast_ref::<HbFont>()
            .ok_or("font backend")?
            .with_shaping_precision(ShapingPrecision::CssExperimental);
    }
    let chars: Vec<u32> = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("qj")
        .chars()
        .map(u32::from)
        .collect();
    let shaped = font.shape_text(
        &chars,
        &[TextRun {
            font: Some(font.clone()),
            size,
            line_height: size * (40.0 / 24.0),
            letter_spacing: 0.0,
            unichar_count: chars.len() as u32,
            script: u32::from_be_bytes(*b"Latn"),
            style_id: 0,
            level: 0,
        }],
        0,
    );
    let mut out = Vec::new();
    let mut origin = 0.0_f32;
    for run in &shaped[0].runs {
        for (i, &glyph) in run.glyphs.iter().enumerate() {
            let path = font.get_path(glyph);
            out.push(serde_json::json!({"glyph":glyph,"scalar":chars[run.text_indices[i] as usize],"advance":run.advances[i],"origin":origin,"verbs":path.verbs().iter().map(|v|format!("{v:?}")).collect::<Vec<_>>(),"points":path.points().iter().map(|p|[p.x,p.y]).collect::<Vec<_>>(),"intercept":glyph_stripe_intercept(&path,-5.5/24.0,-4.5/24.0)}));
            origin += run.advances[i];
        }
    }
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
