//! Exercises the opt-in Rust rasterizer with real runtime glyph exports, then
//! records bounded mask images for native replay. No browser layout is read.
#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use nuxie_render_api::{
        BlendMode, Factory, GlyphFontRef, ImageSampler, Mat2D, PositionedGlyph, RecordingFactory,
        RenderGlyphRun, Renderer,
    };
    use nuxie_renderer::glyph_rasterizer::rasterize;
    use serde::Deserialize;
    use std::{fs, sync::Arc};
    #[derive(Deserialize)]
    struct Glyph {
        id: u16,
        size: f32,
        x: f32,
        y: f32,
        color: u32,
    }
    #[derive(Deserialize)]
    struct Run {
        matrix: [f32; 6],
        glyphs: Vec<Glyph>,
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 7 {
        return Err(
            "native_glyph_composite font glyphs width height argb out.stream metrics.json".into(),
        );
    }
    let bytes: Arc<[u8]> = fs::read(&args[0])?.into();
    let runs: Vec<Run> = serde_json::from_slice(&fs::read(&args[1])?)?;
    let width: u32 = args[2].parse()?;
    let height: u32 = args[3].parse()?;
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return Err("Invalid viewport".into());
    }
    let mut factory = RecordingFactory::new();
    factory.frame_size(width, height);
    factory.clear_color(u32::from_str_radix(&args[4], 16)?);
    factory.add_sample(0.0);
    let mut renderer = factory.make_renderer();
    let mut metrics = Vec::new();
    for run in runs {
        let mut start = 0;
        while start < run.glyphs.len() {
            let first = &run.glyphs[start];
            let mut end = start + 1;
            while end < run.glyphs.len()
                && run.glyphs[end].size == first.size
                && run.glyphs[end].color == first.color
            {
                end += 1;
            }
            let glyphs: Vec<_> = run.glyphs[start..end]
                .iter()
                .map(|g| PositionedGlyph {
                    id: g.id,
                    x: g.x,
                    y: g.y,
                })
                .collect();
            let mask = rasterize(
                &RenderGlyphRun {
                    font: GlyphFontRef {
                        bytes: &bytes,
                        face_index: 0,
                        variations: &[],
                    },
                    glyphs: &glyphs,
                    font_size: first.size,
                    color: first.color,
                    blend_mode: BlendMode::SrcOver,
                },
                Mat2D(run.matrix),
            )
            .ok_or("Rasterizer declined runtime run")?;
            metrics.push(serde_json::json!({"x":mask.x,"y":mask.y,"width":mask.width,"height":mask.height,"bytes":mask.rgba.len(),"glyphs":glyphs.len()}));
            if mask.width > 0 && mask.height > 0 {
                let mut png = Vec::new();
                let mut encoder = png::Encoder::new(&mut png, mask.width, mask.height);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder.write_header()?.write_image_data(&mask.rgba)?;
                let image = factory.decode_image(&png)?;
                renderer.save();
                renderer.translate(mask.x as f32, mask.y as f32);
                renderer.draw_image(
                    Some(image.as_ref()),
                    ImageSampler::default(),
                    BlendMode::SrcOver,
                    1.0,
                );
                renderer.restore();
            }
            start = end;
        }
    }
    fs::write(&args[5], factory.stream())?;
    fs::write(&args[6], serde_json::to_vec_pretty(&metrics)?)?;
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn main() {
    panic!("This experimental control requires macOS");
}
