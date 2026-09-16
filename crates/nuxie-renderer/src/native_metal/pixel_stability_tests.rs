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
        assert!(pixels
            .chunks_exact(4)
            .any(|pixel| pixel != [17, 34, 51, 255]));
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

fn render_published_text(mode: ShaderCompilationMode) -> Vec<u8> {
    use nuxie_render_api::PersistentFactory;
    use nuxie_runtime::{File, FileAssetLoader, FileAssetLoaderRef, RuntimeFactoryHandle};
    struct FontLoader(Vec<u8>);
    impl FileAssetLoader for FontLoader {
        fn load_contents(
            &mut self,
            asset: nuxie_runtime::CoreHandle,
            _: &[u8],
            factory: &RuntimeFactoryHandle,
        ) -> bool {
            asset
                .with_downcast_mut::<nuxie_runtime::source::assets::font_asset::FontAsset, _>(
                    |font| font.decode(&self.0, factory),
                )
                .unwrap_or(false)
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/published-font-metrics");
    let bytes = std::fs::read(root.join("screen.riv")).unwrap();
    let font = std::fs::read(
        root.join("2898476918b21c3f9b5ba22e86853c6d63b544f92da277a92533011a28c93af5.otf"),
    )
    .unwrap();
    let native = NativeMetalFactory::new_with_context_options(
        390,
        844,
        NativeMetalContextOptions {
            shader_compilation_mode: mode,
            ..Default::default()
        },
    )
    .unwrap();
    let mut factory = PersistentFactory::new(native);
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(
        &bytes,
        retained,
        None,
        Some(FileAssetLoaderRef::new(Box::new(FontLoader(font)))),
        None,
    )
    .unwrap();
    let artboard = file
        .with_file(|file| file.artboard_named("Paywall"))
        .unwrap();
    let model =
        file.with_file(|file| file.create_view_model_instance_for_artboard(artboard.core_handle()));
    artboard.bind_view_model_instance(model.clone());
    let machine = artboard.default_state_machine();
    if let (Some(machine), Some(model)) = (&machine, model) {
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(model));
    }
    let mut static_scene =
        nuxie_runtime::source::static_scene::StaticScene::new(artboard.downgrade());
    let mut baseline: Option<Vec<u8>> = None;
    for index in 0..16 {
        if let Some(machine) = &machine {
            machine.advance_and_apply(0.0);
        } else {
            static_scene.advance_and_apply(0.0);
        }
        let mut frame = factory.borrow().begin_frame(0xff14_263e).unwrap();
        artboard.draw(&mut frame);
        let pixels = frame.finish().unwrap();
        assert!(pixels.chunks_exact(4).any(|pixel| pixel[0] > 100));
        if let Some(baseline) = &baseline {
            assert_same_pixels(baseline, &pixels, &format!("text {mode:?} frame {index}"));
        } else {
            baseline = Some(pixels);
        }
    }
    baseline.unwrap()
}

#[test]
fn published_text_is_stable_across_shader_modes() {
    let specialized = render_published_text(ShaderCompilationMode::AlwaysSynchronous);
    for mode in [
        ShaderCompilationMode::OnlyUbershaders,
        ShaderCompilationMode::AllowAsynchronous,
    ] {
        let actual = render_published_text(mode);
        assert_same_pixels(&specialized, &actual, &format!("published text {mode:?}"));
    }
}
