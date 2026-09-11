#![cfg(all(target_os = "macos", feature = "native-glyph-controls"))]
use nuxie_render_api::{
    BlendMode, GlyphFontRef, PersistentFactory, PositionedGlyph, RecordingFactory, RenderGlyphRun,
    Renderer,
};
use nuxie_renderer::glyph_adapter::GlyphCache;
use std::sync::Arc;
fn font() -> Arc<[u8]> {
    Arc::from(include_bytes!("assets/Inter-Regular.ttf").as_slice())
}
fn run<'a>(bytes: &'a Arc<[u8]>, glyphs: &'a [PositionedGlyph]) -> RenderGlyphRun<'a> {
    RenderGlyphRun {
        font: GlyphFontRef {
            bytes,
            face_index: 0,
            variations: &[],
        },
        glyphs,
        font_size: 20.0,
        color: 0xff336699,
        blend_mode: BlendMode::SrcOver,
    }
}
fn h(bytes: &[u8]) -> [PositionedGlyph; 1] {
    [PositionedGlyph {
        id: ttf_parser::Face::parse(bytes, 0)
            .unwrap()
            .glyph_index('H')
            .unwrap()
            .0,
        x: 0.25,
        y: 22.25,
    }]
}
#[test]
fn cache_survives_frames_and_reuses_only_matching_subpixel_paint() {
    let factory = PersistentFactory::new(RecordingFactory::new());
    let mut cache = GlyphCache::new(factory.clone());
    let bytes = font();
    let glyphs = h(&bytes);
    let mut request = run(&bytes, &glyphs);
    for (x, y) in [(0.0, 0.0), (100.0, -50.0)] {
        let mut inner = factory.borrow().make_renderer();
        let mut renderer = cache.wrap(&mut inner);
        renderer.translate(x, y);
        assert!(renderer.draw_glyph_run(&request));
    }
    assert_eq!((cache.stats().hits, cache.stats().misses), (1, 1));
    let mut inner = factory.borrow().make_renderer();
    {
        let mut renderer = cache.wrap(&mut inner);
        renderer.translate(0.5, 0.0);
        assert!(renderer.draw_glyph_run(&request));
    }
    request.color = 0x80336699;
    {
        let mut renderer = cache.wrap(&mut inner);
        assert!(renderer.draw_glyph_run(&request));
    }
    assert_eq!((cache.stats().hits, cache.stats().misses), (1, 3));
    cache.clear();
    assert_eq!(
        (cache.stats().entries, cache.stats().retained_bytes),
        (0, 0)
    );
    assert_eq!(Arc::strong_count(&bytes), 1);
}
#[test]
fn declined_run_does_not_change_recorded_destination_state() {
    let factory = PersistentFactory::new(RecordingFactory::new());
    let mut cache = GlyphCache::new(factory.clone());
    let bytes = font();
    let glyphs = h(&bytes);
    let mut request = run(&bytes, &glyphs);
    request.font_size = f32::NAN;
    let mut inner = factory.borrow().make_renderer();
    let mut renderer = cache.wrap(&mut inner);
    renderer.translate(10.0, 20.0);
    let before = factory.borrow().stream();
    assert!(!renderer.draw_glyph_run(&request));
    assert_eq!(factory.borrow().stream(), before);
}
#[test]
fn cache_eviction_bounds_retention_and_releases_font_references() {
    let factory = PersistentFactory::new(RecordingFactory::new());
    let mut cache = GlyphCache::new(factory.clone());
    let bytes = font();
    let glyphs = h(&bytes);
    let mut request = run(&bytes, &glyphs);
    let mut inner = factory.borrow().make_renderer();
    for color in 0..270 {
        request.color = 0xff000000 | color;
        assert!(cache.wrap(&mut inner).draw_glyph_run(&request));
        assert!(cache.stats().retained_bytes <= 32 * 1024 * 1024);
        assert!(cache.stats().entries <= 256);
    }
    assert!(cache.stats().entries < 270);
    assert_eq!(Arc::strong_count(&bytes), cache.stats().entries + 1);
    drop(cache);
    assert_eq!(Arc::strong_count(&bytes), 1);
}
#[test]
fn nested_restore_restores_cache_transform_identity() {
    let factory = PersistentFactory::new(RecordingFactory::new());
    let mut cache = GlyphCache::new(factory.clone());
    let bytes = font();
    let glyphs = h(&bytes);
    let request = run(&bytes, &glyphs);
    let mut inner = factory.borrow().make_renderer();
    {
        let mut renderer = cache.wrap(&mut inner);
        assert!(renderer.draw_glyph_run(&request));
        renderer.save();
        renderer.translate(0.25, 0.5);
        assert!(renderer.draw_glyph_run(&request));
        renderer.restore();
        assert!(renderer.draw_glyph_run(&request));
    }
    assert_eq!((cache.stats().hits, cache.stats().misses), (1, 2));
}

#[test]
fn live_runtime_uses_solid_glyphs_but_keeps_multiple_paints_as_vectors() {
    use nuxie_html_to_riv::{Asset, CompileInput, compile};
    use nuxie_runtime::mechanical_port::source::generated::core_registry::{
        CoreField, CoreRegistryObject,
    };
    use nuxie_runtime::mechanical_port::source::text::text_style_paint::TextStylePaint;
    use nuxie_runtime::{File, RuntimeFactoryHandle};
    let mut input = CompileInput {
        html: "<p id=text>Hg sample</p>".into(),
        css: "p { font:20px/30px Inter; color:#369 }".into(),
        width: 390.0,
        height: 320.0,
        ..Default::default()
    };
    input.assets.insert(
        "inter".into(),
        Asset::Font {
            family: "Inter".into(),
            weight: 400,
            bytes: font().to_vec(),
        },
    );
    let bytes = compile(&input).unwrap().riv;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    artboard.update_pass(true);
    let mut cache = GlyphCache::new(factory.clone());
    let mut inner = factory.borrow().make_renderer();
    artboard.draw(&mut cache.wrap(&mut inner));
    assert_eq!(cache.stats().entries, 1);
    let style = artboard
        .with_artboard(|a| a.find_all_handles::<TextStylePaint>())
        .into_iter()
        .next()
        .unwrap();
    let fill = style
        .with_downcast::<TextStylePaint, _>(|style| style.paints.shape_paints()[0].clone())
        .unwrap();
    fill.with_downcast_mut::<nuxie_runtime::mechanical_port::source::shapes::paint::fill::Fill, _>(
        |fill| {
            fill.set_uint(
                CoreField::FillFillRule,
                nuxie_render_api::FillRule::EvenOdd as u32,
            )
        },
    );
    cache.clear();
    artboard.draw(&mut cache.wrap(&mut inner));
    assert_eq!(
        cache.stats().entries,
        0,
        "even-odd paint must retain its vector fill rule"
    );
    fill.with_downcast_mut::<nuxie_runtime::mechanical_port::source::shapes::paint::fill::Fill, _>(
        |fill| {
            fill.set_uint(
                CoreField::FillFillRule,
                nuxie_render_api::FillRule::NonZero as u32,
            )
        },
    );
    style.with_downcast_mut::<TextStylePaint, _>(|style| {
        let first = style.paints.shape_paints()[0].clone();
        style.paints.add_paint(first);
    });
    cache.clear();
    let before = factory.borrow().stream().len();
    artboard.draw(&mut cache.wrap(&mut inner));
    let stream = factory.borrow().stream();
    let added = &stream[before..];
    assert_eq!(cache.stats().entries, 0);
    assert_eq!(added.matches("drawPath ").count(), 2);
    assert!(!added.contains("drawImage "));
}

#[test]
fn glyph_opacity_is_explicit_once_and_restored_across_nested_scopes() {
    let factory = PersistentFactory::new(RecordingFactory::new());
    let mut cache = GlyphCache::new(factory.clone());
    let bytes = font();
    let glyphs = h(&bytes);
    let request = run(&bytes, &glyphs);
    let mut inner = factory.borrow().make_renderer();
    {
        let mut renderer = cache.wrap(&mut inner);
        renderer.save();
        renderer.modulate_opacity(0.0);
        assert!(renderer.draw_glyph_run(&request));
        renderer.restore();
        renderer.save();
        renderer.modulate_opacity(0.5);
        renderer.save();
        renderer.modulate_opacity(0.5);
        assert!(renderer.draw_glyph_run(&request));
        renderer.restore();
        assert!(renderer.draw_glyph_run(&request));
        renderer.restore();
        assert!(renderer.draw_glyph_run(&request));
    }
    let stream = factory.borrow().stream();
    let opacities: Vec<f32> = stream
        .lines()
        .filter(|line| line.starts_with("drawImage "))
        .map(|line| line.split("opacity=").nth(1).unwrap().parse().unwrap())
        .collect();
    assert_eq!(opacities, [0.25, 0.5, 1.0]);
    assert_eq!((cache.stats().hits, cache.stats().misses), (2, 1));
}
