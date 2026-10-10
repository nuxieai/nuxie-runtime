//! Complete stroke_position and stroke_position_clipped additions to
//! tests/gm/strokes.cpp at ee60b701. Earlier scenes remain in existing harnesses.
//! C++ references use the existing Metal FMA/coverage adaptations; pristine
//! captures and exact comparison/provenance records are retained alongside them.
use super::{
    clipstrokes::{circle, rect, rounded_rect},
    ore_gm_helper::*,
};
use crate::deferred::cmd::{
    deferred_render_factory::DeferredFactory,
    render_replay::{replay_render_commands, ReplayHooks, ResourceTable},
};

const DIM: u32 = 768;

pub(super) fn check_position_scene(name: &str, draw: impl Fn(&mut dyn Factory, &mut dyn Renderer)) {
    let render = |deferred| {
        let mut host = GmHost::with_size(0, true, DIM, DIM);
        let screen = host.screen();
        if deferred {
            let mut factory = DeferredFactory::new();
            let mut renderer = factory.make_renderer(None);
            draw(&mut factory, &mut renderer);
            let buffer = factory.buffer.lock().unwrap();
            replay_render_commands(
                &mut host.factory,
                Some(screen.borrow_mut().as_mut()),
                buffer.command_bytes(),
                buffer.blob_bytes(),
                &mut ResourceTable::default(),
                &mut ReplayHooks::default(),
            );
        } else {
            draw(&mut host.factory, screen.borrow_mut().as_mut());
        }
        host.finish()
    };
    let immediate = render(false);
    let deferred = render(true);
    assert_eq!(immediate, deferred, "{name}: immediate/deferred pixels");
    let actual = pixel_compare::RgbaImage::new(DIM, DIM, immediate).expect("GM dimensions");
    let reference = std::path::Path::new(
        option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
    )
    .join(format!(
        "../../fixtures/renderer/reference/metal/gm/{name}.png"
    ));
    let expected = pixel_compare::RgbaImage::read_png(reference).expect("pinned C++ GM capture");
    let result = pixel_compare::compare(&expected, &actual, pixel_compare::Tolerance::EXACT)
        .expect("same GM dimensions");
    if !result.within_tolerance {
        let diagnostic = std::env::temp_dir().join(format!("nuxie-{name}-actual.png"));
        actual.write_png(&diagnostic).expect("GM diagnostic PNG");
        eprintln!("actual GM pixels: {}", diagnostic.display());
    }
    assert!(
        result.within_tolerance,
        "{name}: {} differing pixels, max channel delta {}",
        result.different_pixels, result.max_channel_delta
    );
}

fn draw_position(f: &mut dyn Factory, r: &mut dyn Renderer, clipped: bool) {
    {
        let mut paint = f.make_render_paint();
        paint.color(0xff000000);
        let background = f.make_render_path(rect(0., 0., 768., 768.), FillRule::NonZero);
        r.draw_path(background.as_ref(), paint.as_ref());
    }
    let clip = clipped
        .then(|| f.make_render_path(rounded_rect(0., 64., 256., 192., 20.), FillRule::NonZero));
    let path = f.make_render_path(circle(128., 128., 100.), FillRule::Clockwise);
    for (row, feather) in [0., 20., 150.].into_iter().enumerate() {
        r.save();
        r.translate(0., row as f32 * 256.);
        for position in [
            StrokePosition::Inside,
            StrokePosition::Center,
            StrokePosition::Outside,
        ] {
            if let Some(clip) = &clip {
                r.save();
                r.clip_path(clip.as_ref());
            }
            let mut paint = f.make_render_paint();
            paint.color(0xffefefef);
            paint.stroke(&StrokeParams {
                thickness: 20.,
                position,
                ..StrokeParams::default()
            });
            paint.feather(feather);
            r.draw_path(path.as_ref(), paint.as_ref());
            paint.color(0x5f881177);
            paint.style(RenderPaintStyle::Fill);
            paint.feather(0.);
            r.draw_path(path.as_ref(), paint.as_ref());
            if clipped {
                r.restore();
            }
            r.translate(256., 0.);
        }
        r.restore();
    }
}

#[test]
fn stroke_position() {
    check_position_scene("stroke_position", |f, r| draw_position(f, r, false));
}

#[test]
fn stroke_position_clipped() {
    check_position_scene("stroke_position_clipped", |f, r| draw_position(f, r, true));
}
