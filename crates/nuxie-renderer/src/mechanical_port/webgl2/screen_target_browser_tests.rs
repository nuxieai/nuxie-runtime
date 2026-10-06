//! Browser contract for a script drawing into the public WebGL2 screen target.

use nuxie_ore_metal::{context::FrameDescriptor, types::*};
use nuxie_render_api::Factory;
use web_sys::HtmlCanvasElement;

use crate::{RenderMode, WebGl2Factory};

fn solid(pixels: &[u8], width: u32, height: u32, color: [u8; 4]) -> Result<(), String> {
    if pixels.len() != width as usize * height as usize * 4 {
        return Err(format!(
            "readback size {} for {width}x{height}",
            pixels.len()
        ));
    }
    if let Some((index, pixel)) = pixels
        .chunks_exact(4)
        .enumerate()
        .find(|(_, p)| *p != color)
    {
        return Err(format!("pixel {index}: {pixel:?}, expected {color:?}"));
    }
    Ok(())
}

pub fn run_screen_target_browser_tests(canvas: HtmlCanvasElement) -> Result<String, String> {
    let mut factory = WebGl2Factory::new(canvas, 37, 23).map_err(|e| e.to_string())?;
    let mut failures = Vec::new();
    let desc = factory.ore_target_desc();
    if (desc.width, desc.height, desc.format) != (37, 23, TextureFormat::rgba8unorm) {
        failures.push(format!("description: {desc:?}, expected 37x23 RGBA8"));
    }
    for mode in [
        RenderMode::RasterOrdering,
        RenderMode::Msaa,
        RenderMode::ClockwiseMsaa1,
        RenderMode::ClockwiseAtomic,
    ] {
        factory.set_target_preserved(false);
        let frame = factory
            .begin_frame(0xff000000, mode)
            .map_err(|e| e.to_string())?;
        let target = factory.ore_render_target();
        if let Some(target) = target {
            let ore = factory.ore().ok_or("no Ore context")?;
            let mut ore = ore.borrow_mut();
            ore.beginFrame(&FrameDescriptor::new(0, 1));
            let view =
                unsafe { ore.wrapRenderTarget(target) }.ok_or("Ore refused the screen target")?;
            let mut pass = RenderPassDesc::default();
            pass.colorAttachments[0].view = Some(&view);
            pass.colorAttachments[0].clearColor = ClearColor {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            };
            ore.beginRenderPass(&pass, None)
                .ok_or("screen clear pass refused")?
                .finish();
            ore.endFrame();
        } else {
            failures.push(format!("{mode:?}: no Ore screen target"));
        }
        factory.after_deferred_ore_frame();
        // The script requests preservation after the host frame has opened.
        factory.set_target_preserved(true);
        let pixels = frame.finish().map_err(|e| e.to_string())?;
        if let Err(error) = solid(&pixels, 37, 23, [0, 255, 0, 255]) {
            failures.push(format!("{mode:?}: script clear: {error}"));
        }
        let pixels = factory
            .begin_frame(0xffff0000, mode)
            .map_err(|e| e.to_string())?
            .finish()
            .map_err(|e| e.to_string())?;
        if let Err(error) = solid(&pixels, 37, 23, [0, 255, 0, 255]) {
            failures.push(format!("{mode:?}: preserved frame: {error}"));
        }
        let frame = factory
            .begin_frame(0xff0000ff, mode)
            .map_err(|e| e.to_string())?;
        factory.set_target_preserved(false);
        solid(
            &frame.finish().map_err(|e| e.to_string())?,
            37,
            23,
            [0, 0, 255, 255],
        )?;
    }
    factory.set_target_preserved(true);
    factory.resize(19, 11).map_err(|e| e.to_string())?;
    let desc = factory.ore_target_desc();
    if (desc.width, desc.height, desc.format) != (19, 11, TextureFormat::rgba8unorm) {
        failures.push(format!(
            "resized description: {desc:?}, expected 19x11 RGBA8"
        ));
    }
    let pixels = factory
        .begin_frame(0xffff0000, RenderMode::Msaa)
        .map_err(|e| e.to_string())?
        .finish()
        .map_err(|e| e.to_string())?;
    solid(&pixels, 19, 11, [255, 0, 0, 255])?;
    if failures.is_empty() {
        Ok("screen description, script pixels, preservation, clear and resize passed in every render mode".into())
    } else {
        Err(failures.join("\n"))
    }
}

pub fn run_screen_target_benchmark(canvas: HtmlCanvasElement) -> Result<String, String> {
    let factory = WebGl2Factory::new(canvas, 900, 900).map_err(|e| e.to_string())?;
    let mut batches = Vec::new();
    for batch in 0..6 {
        let start = js_sys::Date::now();
        for _ in 0..100 {
            factory
                .begin_frame(0xff102030, RenderMode::Msaa)
                .map_err(|e| e.to_string())?
                .finish_present()
                .map_err(|e| e.to_string())?;
        }
        if batch != 0 {
            batches.push((js_sys::Date::now() - start) / 100.0);
        }
    }
    batches.sort_by(f64::total_cmp);
    Ok(format!(
        "900x900 empty MSAA frame, median ms/frame over five batches of 100 after 100 warmups: {}; batches: {batches:?}",
        batches[2]
    ))
}
