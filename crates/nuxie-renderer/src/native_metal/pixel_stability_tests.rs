use super::{NativeMetalContextOptions, NativeMetalFactory, ShaderCompilationMode};
use nuxie_render_api::{Factory, FillRule, RawPath, RenderPaintStyle, Renderer};

fn render_unchanged_frames(mode: ShaderCompilationMode, color: u32, inset: f32) -> Vec<u8> {
    let mut factory = NativeMetalFactory::new_with_context_options(
        390,
        844,
        NativeMetalContextOptions {
            shader_compilation_mode: mode,
            ..Default::default()
        },
    )
    .expect("live Metal factory");
    let mut raw = RawPath::new();
    raw.move_to(inset, inset);
    raw.line_to(390.0 - inset, inset);
    raw.line_to(390.0 - inset, 844.0 - inset);
    raw.line_to(inset, 844.0 - inset);
    raw.close();
    let path = factory.make_render_path(raw, FillRule::NonZero);
    let mut paint = factory.make_render_paint();
    paint.style(RenderPaintStyle::Fill);
    paint.color(color);
    let mut baseline: Option<Vec<u8>> = None;
    for index in 0..16 {
        let mut frame = factory.begin_frame(0xff11_2233).expect("begin frame");
        frame.draw_path(path.as_ref(), paint.as_ref());
        let pixels = frame.finish().expect("render and read owned texture");
        // Equal clear frames must not satisfy shader parity after a lost draw.
        assert!(pixels.chunks_exact(4).any(|pixel| pixel != [17, 34, 51, 255]));
        if let Some(baseline) = &baseline {
            assert_same_pixels(
                baseline,
                &pixels,
                &format!("{mode:?} frame {index}, {color:#x}, inset {inset}"),
            );
        } else {
            baseline = Some(pixels);
        }
    }
    baseline.unwrap()
}

fn assert_same_pixels(expected: &[u8], actual: &[u8], context: &str) {
    assert_eq!(expected.len(), actual.len(), "{context}");
    let changed = expected
        .chunks_exact(4)
        .zip(actual.chunks_exact(4))
        .filter(|(first, current)| first != current)
        .count();
    assert_eq!(changed, 0, "{context}: changed pixels");
}

#[test]
fn unchanged_solid_fill_is_stable_across_shader_modes() {
    for inset in [0.0, 0.25] {
        for color in [0xff14_263e, 0x8014_263e, 0x40fa_b123] {
            let specialized =
                render_unchanged_frames(ShaderCompilationMode::AlwaysSynchronous, color, inset);
            for mode in [
                ShaderCompilationMode::OnlyUbershaders,
                ShaderCompilationMode::AllowAsynchronous,
            ] {
                let actual = render_unchanged_frames(mode, color, inset);
                // This comparison forces both pipeline variants, independent of
                // whether asynchronous compilation finishes during the frame loop.
                assert_same_pixels(
                    &specialized,
                    &actual,
                    &format!("{mode:?}, {color:#x}, inset {inset}"),
                );
            }
        }
    }
}
