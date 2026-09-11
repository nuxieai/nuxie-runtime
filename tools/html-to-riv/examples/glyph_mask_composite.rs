//! Diagnostic image composition through the actual recording/replay renderer.
//! The supplied image is produced after native text layout, never by a browser.
use nuxie_render_api::{BlendMode, Factory, ImageSampler, RecordingFactory, Renderer};
use std::{error::Error, fs};
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 5 {
        return Err(
            "glyph_mask_composite <mask.png> <width> <height> <argb-hex> <out.stream>".into(),
        );
    }
    let width: u32 = args[1].parse()?;
    let height: u32 = args[2].parse()?;
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return Err("Invalid dimensions".into());
    }
    let background = u32::from_str_radix(&args[3], 16)?;
    let mut factory = RecordingFactory::new();
    let image = factory.decode_image(&fs::read(&args[0])?)?;
    factory.frame_size(width, height);
    factory.clear_color(background);
    factory.add_sample(0.0);
    let mut renderer = factory.make_renderer();
    renderer.draw_image(
        Some(image.as_ref()),
        ImageSampler::default(),
        BlendMode::SrcOver,
        1.0,
    );
    fs::write(&args[4], factory.stream())?;
    Ok(())
}
