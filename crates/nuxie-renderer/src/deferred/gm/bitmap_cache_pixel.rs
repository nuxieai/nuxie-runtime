//! tests/unit_tests/renderer/bitmap_cache_pixel_test.cpp at a4dbc3ff.
//!
//! Pixel-level validation of cache-as-bitmap: the same artboard rendered twice
//! against the live Metal backend -- once cached, once not -- with the two
//! framebuffers compared. The uncached render is the golden, so there is no
//! stored PNG. Cache-as-bitmap only engages on the deferred/recording path, so
//! the frame is recorded into a DeferredSession and replayed through GmHost,
//! this port's TestingWindowFrameSink.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    command_stream::CommandReader,
    deferred_replayer::{snapshot_frame, DeferredReplayer},
    deferred_session::DeferredSession,
    render_commands::{payload_size_of, RenderCmd},
};
use nuxie_runtime::mechanical_port::source::generated::{
    bitmap_cache_base::BitmapCacheBase, core_registry::CoreRegistry,
};

// Difference between two RGBA8 buffers of the same size.
struct Diff {
    max_channel: u32,
    mean_channel: f64,
    // Fraction of pixels where some channel moved by more than
    // CHANNEL_EPSILON, which absorbs dithering on a handful of edge pixels.
    significant_fraction: f64,
}
const CHANNEL_EPSILON: u32 = 2;
impl Diff {
    fn between(a: &[u8], b: &[u8]) -> Self {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len() % 4, 0);
        let mut total = 0u64;
        let mut significant = 0usize;
        let mut max_channel = 0;
        let pixels = a.len() / 4;
        for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
            let mut worst = 0;
            for c in 0..4 {
                let delta = (i32::from(pa[c]) - i32::from(pb[c])).unsigned_abs();
                total += u64::from(delta);
                worst = worst.max(delta);
            }
            max_channel = max_channel.max(worst);
            if worst > CHANNEL_EPSILON {
                significant += 1;
            }
        }
        Self {
            max_channel,
            mean_channel: total as f64 / (pixels * 4) as f64,
            significant_fraction: significant as f64 / pixels as f64,
        }
    }
    fn report(&self, what: &str) {
        println!(
            "[bitmap-cache] {what}: max channel {}, mean {:.4}, {:.4}% of pixels off by more than {CHANNEL_EPSILON}",
            self.max_channel,
            self.mean_channel,
            self.significant_fraction * 100.0
        );
    }
}

// Gradient energy: the summed *squared* luma step between neighboring pixels.
// Squaring makes spreading the same climb over more pixels strictly cheaper,
// which is what blur does.
fn gradient_energy(px: &[u8], w: u32, h: u32) -> f64 {
    let luma = |x: u32, y: u32| {
        let i = (y as usize * w as usize + x as usize) * 4;
        0.299 * f64::from(px[i]) + 0.587 * f64::from(px[i + 1]) + 0.114 * f64::from(px[i + 2])
    };
    let mut energy = 0.0;
    for y in 0..h.saturating_sub(1) {
        for x in 0..w.saturating_sub(1) {
            let dx = luma(x + 1, y) - luma(x, y);
            let dy = luma(x, y + 1) - luma(x, y);
            energy += dx * dx + dy * dy;
        }
    }
    energy
}

// How much of the buffer is not the (opaque black) clear color. A cached and
// an uncached render of *nothing* match perfectly.
// (drawImage, drawPath) counts in a recorded frame.
fn draws(commands: &[u8], blobs: &[u8]) -> (usize, usize) {
    let (mut images, mut paths) = (0, 0);
    let mut reader = CommandReader::new(commands, blobs);
    while let Some(command) = reader.next_u8().and_then(RenderCmd::from_byte) {
        match command {
            RenderCmd::DrawImage => images += 1,
            RenderCmd::DrawPath => paths += 1,
            _ => {}
        }
        reader.skip(payload_size_of(command));
    }
    (images, paths)
}

fn non_background_pixels(px: &[u8]) -> usize {
    px.chunks_exact(4)
        .filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0)
        .count()
}

struct Rendered {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

// Renders one frame of the fixture and returns the RGBA readback. device_scale
// stands in for what the editor's stage applies: viewport zoom times window
// density.
fn render_once(
    cache_enabled: bool,
    resolution: f32,
    device_scale: f32,
    translate: (f32, f32),
) -> Rendered {
    let mut host = GmHost::with_screen(0xff00_0000, false);
    let mut session = PersistentFactory::new(DeferredSession::with_caps(Default::default()));
    session
        .borrow_mut()
        .bind_render_context(host.factory.persistent_context());
    let file = nuxie_runtime::File::import(
        &fixture("sync/cache_as_bitmap_test.riv"),
        nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut session).unwrap(),
        None,
        None,
        None,
    )
    .expect("fixture imports");
    let count = file.with_file(|f| f.artboard_count());
    let artboard = (0..count)
        .filter_map(|i| file.with_file(|f| f.artboard_at(i)))
        .find(|a| a.with_artboard(|a| a.bitmap_cache().is_some()))
        .expect("cached artboard");
    let cache = artboard.with_artboard(|a| a.bitmap_cache()).unwrap();
    // Driven straight on the core object.
    CoreRegistry::set_bool_handle(
        &cache,
        i32::from(BitmapCacheBase::CACHE_ENABLED_PROPERTY_KEY),
        cache_enabled,
    );
    CoreRegistry::set_double_handle(
        &cache,
        i32::from(BitmapCacheBase::RESOLUTION_PROPERTY_KEY),
        resolution,
    );

    let (aw, ah) = artboard.with_artboard(|a| (a.width(), a.height()));
    let width = (aw * device_scale).ceil() as u32 + 8;
    let height = (ah * device_scale).ceil() as u32 + 8;
    host.factory.borrow_mut().resize(width, height).unwrap();

    artboard.advance_default(0.0);
    let screen = session.borrow().screen_renderer(0);
    {
        let mut screen = screen.borrow_mut();
        screen.save();
        screen.translate(translate.0, translate.1);
        screen.transform(Mat2D([device_scale, 0.0, 0.0, device_scale, 0.0, 0.0]));
        // draw_internal, not draw: draw() is the standalone-root path and
        // deliberately bypasses the cache.
        artboard.draw_internal(screen.as_mut());
        screen.restore();
    }
    let frame = snapshot_frame(&mut session.borrow_mut());
    session.borrow_mut().reset_frame();
    DeferredReplayer::default().replay_frame(&frame, &mut host);
    if cache_enabled {
        // If the cache never opened an offscreen frame it silently fell back
        // to a vector draw, and every comparison would be tautological.
        assert_eq!(host.canvas_frames(), 1);
    }
    Rendered {
        pixels: host.finish(),
        width,
        height,
    }
}

// Not in upstream: a static frame composites the raster an earlier frame
// made, through a canvas that is no longer in that frame's content set. It
// has to resolve to the backing the first replay installed on the device.
#[test]
fn a_later_frame_composites_the_raster_an_earlier_frame_made() {
    const DEVICE_SCALE: f32 = 2.0;
    let uncached = render_once(false, 1.0, DEVICE_SCALE, (0.0, 0.0));
    assert!(non_background_pixels(&uncached.pixels) > uncached.pixels.len() / 4 / 100);

    let mut host = GmHost::with_screen(0xff00_0000, false);
    let mut session = PersistentFactory::new(DeferredSession::with_caps(Default::default()));
    session
        .borrow_mut()
        .bind_render_context(host.factory.persistent_context());
    let file = nuxie_runtime::File::import(
        &fixture("sync/cache_as_bitmap_test.riv"),
        nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut session).unwrap(),
        None,
        None,
        None,
    )
    .expect("fixture imports");
    let count = file.with_file(|f| f.artboard_count());
    let artboard = (0..count)
        .filter_map(|i| file.with_file(|f| f.artboard_at(i)))
        .find(|a| a.with_artboard(|a| a.bitmap_cache().is_some()))
        .expect("cached artboard");
    let cache = artboard.with_artboard(|a| a.bitmap_cache()).unwrap();
    CoreRegistry::set_double_handle(
        &cache,
        i32::from(BitmapCacheBase::RESOLUTION_PROPERTY_KEY),
        1.0,
    );
    host.factory
        .borrow_mut()
        .resize(uncached.width, uncached.height)
        .unwrap();

    let mut replayer = DeferredReplayer::default();
    let mut frames = Vec::new();
    for _ in 0..2 {
        artboard.advance_default(0.0);
        let screen = session.borrow().screen_renderer(0);
        {
            let mut screen = screen.borrow_mut();
            screen.save();
            screen.transform(Mat2D([DEVICE_SCALE, 0.0, 0.0, DEVICE_SCALE, 0.0, 0.0]));
            artboard.draw_internal(screen.as_mut());
            screen.restore();
        }
        let frame = snapshot_frame(&mut session.borrow_mut());
        session.borrow_mut().reset_frame();
        let recorded = (
            frame.content_canvases.len(),
            draws(&frame.commands, &frame.blobs),
        );
        replayer.replay_frame(&frame, &mut host);
        frames.push((recorded, host.finish_frame()));
    }
    assert_eq!(replayer.dropped_draws(), 0);
    // The first frame rasterizes; the second only composites: one image and
    // no vector content, so a silent vector fallback cannot pass.
    assert_eq!(frames[0].0 .0, 1);
    assert_eq!(frames[1].0, (0, (1, 0)));
    assert_eq!(host.canvas_frames(), 1);
    for (index, (_, pixels)) in frames.iter().enumerate() {
        let d = Diff::between(&uncached.pixels, pixels);
        d.report(&format!("frame {index} at 2x"));
        assert!(d.significant_fraction < 0.005, "frame {index}");
        assert!(d.max_channel < 32, "frame {index}");
    }
}

#[test]
fn a_cached_artboard_composites_the_pixels_the_vector_draw_would() {
    // At a device scale of 2 the old artboard-unit sizing rasterized at half
    // the resolution the screen was showing and magnified the result.
    for device_scale in [1.0, 2.0] {
        let uncached = render_once(false, 1.0, device_scale, (0.0, 0.0));
        let cached = render_once(true, 1.0, device_scale, (0.0, 0.0));
        assert!(non_background_pixels(&uncached.pixels) > uncached.pixels.len() / 4 / 100);
        let d = Diff::between(&uncached.pixels, &cached.pixels);
        d.report(&format!("resolution 1 at {device_scale:.1}x"));
        assert!(d.significant_fraction < 0.005, "scale {device_scale}");
        assert!(d.max_channel < 32, "scale {device_scale}");
    }
}

#[test]
fn a_coarser_cache_resolution_degrades_the_match() {
    // Proves the comparison above measures sharpness: as the raster gets
    // coarser the difference from the vector draw has to grow.
    const DEVICE_SCALE: f32 = 2.0;
    let uncached = render_once(false, 1.0, DEVICE_SCALE, (0.0, 0.0));
    assert!(non_background_pixels(&uncached.pixels) > uncached.pixels.len() / 4 / 100);
    let mut previous = -1.0;
    for resolution in [1.0, 0.5, 0.25] {
        let cached = render_once(true, resolution, DEVICE_SCALE, (0.0, 0.0));
        let d = Diff::between(&uncached.pixels, &cached.pixels);
        d.report(&format!("resolution {resolution:.2} at 2x"));
        assert!(d.significant_fraction > previous, "resolution {resolution}");
        previous = d.significant_fraction;
    }
}

#[test]
fn a_fractionally_placed_cache_stays_as_sharp_as_an_aligned_one() {
    // Sharpness is judged on the cached render alone: its gradient energy at
    // a fractional offset against the same render at an integer one.
    const DEVICE_SCALE: f32 = 2.0;
    let aligned = render_once(true, 1.0, DEVICE_SCALE, (2.0, 4.0));
    let fractional = render_once(true, 1.0, DEVICE_SCALE, (2.25, 4.4));
    assert_eq!(
        (aligned.width, aligned.height),
        (fractional.width, fractional.height)
    );
    assert!(non_background_pixels(&aligned.pixels) > aligned.pixels.len() / 4 / 100);
    let aligned_energy = gradient_energy(&aligned.pixels, aligned.width, aligned.height);
    let fractional_energy =
        gradient_energy(&fractional.pixels, fractional.width, fractional.height);
    let ratio = fractional_energy / aligned_energy;
    println!(
        "[bitmap-cache] gradient energy aligned {aligned_energy:.0}, fractional {fractional_energy:.0} (ratio {ratio:.4})"
    );
    assert!(ratio > 0.95);
}
