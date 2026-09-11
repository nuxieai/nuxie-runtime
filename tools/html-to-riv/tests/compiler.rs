#[path = "support/declining_renderer.rs"]
mod declining_renderer;
use nuxie_html_to_riv::{CompileInput, compile};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn weighted_flex_grows_and_shrinks_after_import_without_recompiling() {
    let output = compile(&CompileInput {
        html:"<div id=row><div id=a></div><div id=b></div><div id=c></div></div>".into(),
        css:"#row { width:100%; flex-direction:row } #a,#b { height:20px; flex-basis:100px } #a {flex-grow:1;flex-shrink:1} #b {flex-grow:2;flex-shrink:2} #c {width:40px;height:20px}".into(),
        width:400.0,height:200.0,..Default::default()
    }).expect("representable weighted flex compiles");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    for (viewport, a, b) in [(400.0, 153.33333, 206.66667), (200.0, 86.66667, 73.33333)] {
        artboard.set_size(viewport, 200.0);
        artboard.update_pass(true);
        for (id, expected) in [("a", a), ("b", b), ("c", 40.0)] {
            let node = output.source_map.iter().find(|n| n.id == id).unwrap();
            let width = artboard
                .with_artboard(|a| a.objects()[node.object_id as usize].clone())
                .unwrap()
                .with(|o| o.as_layout_component().unwrap().layout_bounds().width())
                .unwrap();
            assert!(
                (width - expected).abs() < 0.1,
                "{id} at {viewport}: {width} vs {expected}"
            );
        }
    }
}

#[test]
fn aligned_flex_items_and_rounded_paints_import() {
    let output = compile(&CompileInput {
        html:"<div id=row><div id=a></div><div id=b></div></div>".into(),
        css:"#row { flex-direction:row; width:240px; height:80px; justify-content:space-between; align-items:center; padding:10px; background-color:#ddd; border-radius:12px } #a, #b { width:30px; height:20px; background-color:#36c }".into(),
        width:320.0, height:240.0, ..Default::default()
    }).expect("alignment compiles");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    artboard.update_pass(true);
    for (id, x) in [("a", 10.0), ("b", 200.0)] {
        let source = output.source_map.iter().find(|n| n.id == id).unwrap();
        let object = artboard
            .with_artboard(|a| a.objects()[source.object_id as usize].clone())
            .unwrap();
        let pos = object
            .with(|o| {
                let t = o.as_world_transform_component().unwrap().world_transform();
                (t[4], t[5])
            })
            .unwrap();
        assert_eq!(pos, (x, 30.0));
    }
}

#[test]
fn png_asset_is_embedded_and_drawn_at_the_authored_size() {
    let output = compile(&CompileInput {
        html: "<img id=picture src=asset:photo>".into(),
        css: "#picture { width:64px; height:48px; }".into(),
        width: 320.0,
        height: 240.0,
        assets: [(
            "photo".into(),
            nuxie_html_to_riv::Asset::Image {
                bytes: include_bytes!("assets/quadrants.png").to_vec(),
            },
        )]
        .into(),
    })
    .expect("image compiles");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    artboard.update_pass(true);
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    assert!(
        factory.borrow().stream().contains("drawImage"),
        "embedded PNG must draw: {}",
        factory.borrow().stream()
    );
}

#[test]
fn embedded_font_and_unicode_text_create_real_text_draw_calls() {
    let input = CompileInput {
        html: "<p id=label>Hello café</p>".into(),
        css: "#label { font-family: Inter; font-size:20px; line-height:28px; color:#123456; }"
            .into(),
        width: 320.0,
        height: 240.0,
        assets: [(
            "inter".into(),
            nuxie_html_to_riv::Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        )]
        .into(),
    };
    let output = compile(&input).expect("text compiles");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    artboard.update_pass(true);
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    assert!(
        factory.borrow().stream().contains("drawPath"),
        "text must shape into glyph paths and draw: {}",
        factory.borrow().stream()
    );
    let node = output.source_map.iter().find(|n| n.id == "label").unwrap();
    let height = artboard
        .with_artboard(|a| a.objects()[node.object_id as usize].clone())
        .unwrap()
        .with(|o| o.as_layout_component().unwrap().layout_bounds().height())
        .unwrap();
    assert!(
        (height - 28.0).abs() < 0.1,
        "line-height should be 28px; got {height}"
    );
    let run = artboard
        .with_artboard(|a| a.objects()[node.text_run_id.unwrap() as usize].clone())
        .unwrap();
    run.with_mut(|o| o.as_text_value_run_mut().unwrap().set_bound_text("This longer text must reflow across multiple lines when the same artboard becomes narrow.".into())).unwrap();
    artboard.set_size(120.0, 240.0);
    artboard.update_pass(true);
    let resized_height = artboard
        .with_artboard(|a| a.objects()[node.object_id as usize].clone())
        .unwrap()
        .with(|o| o.as_layout_component().unwrap().layout_bounds().height())
        .unwrap();
    assert!(
        resized_height > 3.0 * height,
        "host text updates must reflow without recompiling: {resized_height}"
    );
}

#[test]
fn flex_padding_gap_and_percentage_size_are_preserved_as_runtime_layout_rules() {
    let output = compile(&CompileInput {
        html: "<div id=row><div id=a></div><div id=b></div></div>".into(),
        css: "#row { display:flex; flex-direction:row; width:100%; padding:10px; gap:12px; } #a { width:40px; height:20px } #b { width:80px; height:30px }".into(),
        width: 320.0, height: 240.0, ..Default::default()
    }).expect("flex scene compiles");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    artboard.update_pass(true);
    for (id, expected) in [
        ("row", (0.0, 0.0, 320.0, 50.0)),
        ("a", (10.0, 10.0, 40.0, 20.0)),
        ("b", (62.0, 10.0, 80.0, 30.0)),
    ] {
        let source = output.source_map.iter().find(|n| n.id == id).unwrap();
        let object = artboard
            .with_artboard(|a| a.objects()[source.object_id as usize].clone())
            .unwrap();
        let actual = object
            .with(|o| {
                let b = o.as_layout_component().unwrap().layout_bounds();
                let t = o.as_world_transform_component().unwrap().world_transform();
                (t[4], t[5], b.width(), b.height())
            })
            .unwrap();
        assert_eq!(actual, expected, "{id}");
    }
}

#[test]
fn colored_box_compiles_to_a_scene_the_runtime_can_import_and_layout() {
    let output = compile(&CompileInput {
        html: "<div id=box></div>".into(),
        css: "#box { width: 120px; height: 40px; background-color: #336699; }".into(),
        width: 320.0,
        height: 240.0,
        ..Default::default()
    })
    .expect("supported box compiles");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("generated Rive imports");
    let artboard = file.with_file(File::artboard_default).unwrap();
    artboard.update_pass(true);
    let source = output
        .source_map
        .iter()
        .find(|node| node.id == "box")
        .unwrap();
    let object = artboard
        .with_artboard(|a| a.objects()[source.object_id as usize].clone())
        .unwrap();
    let size = object
        .with(|o| {
            o.as_layout_component()
                .map(|l| (l.layout_bounds().width(), l.layout_bounds().height()))
        })
        .flatten()
        .unwrap();
    assert_eq!(size, (120.0, 40.0));
}

#[test]
fn percentage_limits_reflow_the_same_imported_scene() {
    let output = compile(&CompileInput {
        html: "<div id=a></div><div id=b></div>".into(),
        css: "#a{width:120px;height:20px;min-width:25%;max-width:75%;min-height:25%;max-height:75%}#b{width:500px;height:500px;max-width:50%;max-height:50%}".into(),
        width: 390.0, height: 320.0, ..Default::default()
    }).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    for (width, height, aw, ah, bw, bh) in [
        (240.0, 200.0, 120.0, 50.0, 120.0, 100.0),
        (768.0, 400.0, 192.0, 100.0, 384.0, 200.0),
        (100.0, 100.0, 75.0, 25.0, 50.0, 50.0),
    ] {
        artboard.set_size(width, height);
        artboard.update_pass(true);
        for (id, expected_w, expected_h) in [("a", aw, ah), ("b", bw, bh)] {
            let node = output.source_map.iter().find(|n| n.id == id).unwrap();
            let (w, h) = artboard
                .with_artboard(|a| a.objects()[node.object_id as usize].clone())
                .unwrap()
                .with(|o| {
                    let bounds = o.as_layout_component().unwrap().layout_bounds();
                    (bounds.width(), bounds.height())
                })
                .unwrap();
            assert!(
                (w - expected_w).abs() < 0.1 && (h - expected_h).abs() < 0.1,
                "{id} at {width}x{height}: {w}x{h}, expected {expected_w}x{expected_h}"
            );
        }
    }
}

#[test]
fn capped_text_measures_at_its_clamped_width_after_resize() {
    use nuxie_html_to_riv::Asset;
    for (property, width, limit) in [
        ("max-width", 500, "75%"),
        ("max-width", 500, "168px"),
        ("min-width", 40, "75%"),
        ("min-width", 40, "168px"),
    ] {
        let output=compile(&CompileInput {
            html:"<div id=root><p id=a>Percentage limits preserve responsive text wrapping across the viewport.</p></div>".into(),
            css:format!("#root{{width:100%;padding:8px}}#a{{width:{width}px;{property}:{limit};font:16px/24px Inter}}"),
            width:390.0,height:320.0,
            assets:[("inter".into(),Asset::Font{family:"Inter".into(),weight:400,bytes:include_bytes!("assets/Inter-Regular.ttf").to_vec()})].into(),
        }).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &output.riv,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let artboard = file.with_file(File::artboard_default).unwrap();
        for (viewport, expected) in [
            (240.0, 96.0),
            (390.0, if limit == "75%" { 72.0 } else { 96.0 }),
            (
                768.0,
                if limit == "75%" {
                    if property == "min-width" { 24.0 } else { 48.0 }
                } else {
                    96.0
                },
            ),
        ] {
            artboard.set_size(viewport, 320.0);
            artboard.update_pass(true);
            for (id, height) in [("a", expected), ("root", expected + 16.0)] {
                let node = output.source_map.iter().find(|n| n.id == id).unwrap();
                let actual = artboard
                    .with_artboard(|a| a.objects()[node.object_id as usize].clone())
                    .unwrap()
                    .with(|o| o.as_layout_component().unwrap().layout_bounds().height())
                    .unwrap();
                assert!(
                    (actual - height).abs() < 0.1,
                    "{property}:{limit}, {id} at {viewport}: {actual} expected {height}"
                );
            }
        }
    }
}

#[test]
fn experimental_cluster_spacing_preserves_marks_and_default_font_behavior() {
    use nuxie_runtime::source::{
        text::font_hb::{HbFont, LetterSpacingMode},
        text_engine::{FontRef, TextRun},
    };
    let legacy = HbFont::decode(include_bytes!("assets/Inter-Regular.ttf")).unwrap();
    let clustered = legacy
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_letter_spacing_mode(LetterSpacingMode::ClusterExperimental);
    let shape = |font: &FontRef, spacing: f32| {
        let text: Vec<u32> = "a\u{301}\u{327} e\u{308}\u{301} BC"
            .chars()
            .map(u32::from)
            .collect();
        let mut paragraphs = font.shape_text(
            &text,
            &[TextRun {
                font: Some(font.clone()),
                size: 24.0,
                line_height: 36.0,
                letter_spacing: spacing,
                unichar_count: text.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        );
        paragraphs.remove(0).runs.remove(0)
    };
    let base = shape(&legacy, 0.0);
    for spacing in [3.0, -0.5] {
        let old = shape(&legacy, spacing);
        let new = shape(&clustered, spacing);
        assert_eq!(base.glyphs, new.glyphs);
        assert_eq!(base.offsets, new.offsets);
        let mut has_mark = false;
        for i in 0..base.glyphs.len() {
            let ends =
                i + 1 == base.glyphs.len() || base.text_indices[i + 1] != base.text_indices[i];
            has_mark |= !ends;
            assert!((old.advances[i] - base.advances[i] - spacing).abs() < 0.0001);
            assert!(
                (new.advances[i] - base.advances[i] - if ends { spacing } else { 0.0 }).abs()
                    < 0.0001
            );
        }
        assert!(has_mark, "fixture must shape a multi-glyph cluster");
        assert_eq!(
            new.advances,
            shape(&clustered.with_options(&[], &[]), spacing).advances
        );
        let restored = clustered
            .as_any()
            .downcast_ref::<HbFont>()
            .unwrap()
            .with_letter_spacing_mode(LetterSpacingMode::RiveGlyph);
        assert_eq!(old.advances, shape(&restored, spacing).advances);
    }
}

#[test]
fn cluster_policy_survives_font_asset_replacement_and_stays_file_local() {
    use nuxie_html_to_riv::Asset;
    use nuxie_runtime::source::{
        assets::font_asset::FontAsset,
        text::font_hb::{HbFont, LetterSpacingMode},
        text_engine::{FontRef, TextRun},
    };
    let bytes = include_bytes!("assets/Inter-Regular.ttf");
    let output = compile(&CompileInput {
        html: "<p>Marks</p>".into(),
        css: "p{letter-spacing:3px}".into(),
        width: 240.0,
        height: 320.0,
        assets: [(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: bytes.to_vec(),
            },
        )]
        .into(),
    })
    .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory_handle = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let first = File::import(&output.riv, factory_handle.clone(), None, None, None).unwrap();
    let second = File::import(&output.riv, factory_handle.clone(), None, None, None).unwrap();
    let asset = |file: &nuxie_runtime::RuntimeFileHandle| {
        file.with_file(|f| {
            f.assets()
                .iter()
                .find(|a| a.with(|a| a.as_any().is::<FontAsset>()).unwrap_or(false))
                .unwrap()
                .clone()
        })
    };
    let a = asset(&first);
    let b = asset(&second);
    let advances = |font: FontRef| {
        let text: Vec<u32> = "a\u{301}\u{327}BC".chars().map(u32::from).collect();
        font.shape_text(
            &text,
            &[TextRun {
                font: Some(font.clone()),
                size: 24.0,
                line_height: 36.0,
                letter_spacing: 3.0,
                unichar_count: text.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        )[0]
        .runs[0]
            .advances
            .clone()
    };
    let get = |asset: &nuxie_runtime::CoreHandle| {
        asset
            .with_downcast::<FontAsset, _>(FontAsset::font)
            .flatten()
            .unwrap()
    };
    assert!(!get(&b).preserves_line_break_space(0x2003));
    assert!(FontAsset::set_experimental_space_breaks_occurrence(
        &a, true
    ));
    let legacy = advances(get(&a));
    assert!(FontAsset::set_letter_spacing_mode_occurrence(
        &a,
        LetterSpacingMode::ClusterExperimental
    ));
    let expected = advances(get(&a));
    assert_ne!(legacy, expected);
    assert_eq!(advances(get(&b)), legacy);
    FontAsset::set_font_occurrence(&a, HbFont::decode(bytes));
    assert_eq!(advances(get(&a)), expected);
    assert!(get(&a).preserves_line_break_space(0x2003));
    a.with_downcast_mut::<FontAsset, _>(|a| a.set_font(HbFont::decode(bytes)))
        .unwrap();
    assert_eq!(advances(get(&a)), expected);
    assert!(get(&a).preserves_line_break_space(0x2003));
    assert!(
        a.with_downcast_mut::<FontAsset, _>(|a| a.decode(bytes, &factory_handle))
            .unwrap()
    );
    assert_eq!(advances(get(&a)), expected);
    assert!(get(&a).preserves_line_break_space(0x2003));
    FontAsset::set_font_occurrence(&a, None);
    FontAsset::set_font_occurrence(&a, HbFont::decode(bytes));
    assert_eq!(advances(get(&a)), expected);
    assert!(get(&a).preserves_line_break_space(0x2003));
    assert!(FontAsset::set_letter_spacing_mode_occurrence(
        &a,
        LetterSpacingMode::RiveGlyph
    ));
    FontAsset::set_font_occurrence(&a, HbFont::decode(bytes));
    assert_eq!(advances(get(&a)), legacy);
    assert_eq!(advances(get(&b)), legacy);
    assert!(!get(&b).preserves_line_break_space(0x2003));
    assert!(FontAsset::set_experimental_space_breaks_occurrence(
        &a, false
    ));
    FontAsset::set_font_occurrence(&a, HbFont::decode(bytes));
    assert!(!get(&a).preserves_line_break_space(0x2003));
}

#[test]
fn cluster_spacing_disables_optional_ligatures_only_for_nonzero_spacing() {
    use nuxie_runtime::source::{
        text::font_hb::{HbFont, LetterSpacingMode},
        text_engine::{FontFeature, FontRef, TextRun},
    };
    let font = HbFont::decode(include_bytes!("assets/OpenSans-Regular.ttf")).unwrap();
    let spaced = font
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_letter_spacing_mode(LetterSpacingMode::CssLetterSpacingExperimental);
    let no_liga = font.with_options(
        &[],
        &[
            FontFeature {
                tag: u32::from_be_bytes(*b"liga"),
                value: 0,
            },
            FontFeature {
                tag: u32::from_be_bytes(*b"clig"),
                value: 0,
            },
        ],
    );
    let shape = |font: &FontRef, spacing: f32| {
        let text: Vec<u32> = "office ffi fi fl".chars().map(u32::from).collect();
        font.shape_text(
            &text,
            &[TextRun {
                font: Some(font.clone()),
                size: 24.0,
                line_height: 36.0,
                letter_spacing: spacing,
                unichar_count: text.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        )[0]
        .runs[0]
            .glyphs
            .clone()
    };
    let normal = shape(&font, 0.0);
    let separate = shape(&no_liga, 0.0);
    assert!(
        normal.len() < separate.len(),
        "the fixture must actually form optional ligatures"
    );
    let cluster_only = font
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_letter_spacing_mode(LetterSpacingMode::ClusterExperimental);
    assert_eq!(
        shape(&cluster_only, 2.0),
        normal,
        "the existing cluster-only capability retains its original semantics"
    );
    assert_eq!(
        shape(&spaced.with_options(&[], &[]), 2.0),
        separate,
        "font option replacement preserves the CSS spacing policy"
    );
    assert_eq!(shape(&spaced, 0.0), normal);
    assert_eq!(
        shape(&font, 2.0),
        normal,
        "ordinary Rive fonts retain existing ligatures"
    );
    for spacing in [2.0, -0.5] {
        assert_eq!(shape(&spaced, spacing), separate);
    }
}

#[test]
fn word_spacing_source_map_preserves_all_text_in_logical_order() {
    use nuxie_html_to_riv::Asset;
    let output = compile(&CompileInput {
        html: "<p id=label> one  two&nbsp;three four </p>".into(),
        css: "p{word-spacing:4px}".into(),
        width: 390.0,
        height: 320.0,
        assets: [(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        )]
        .into(),
    })
    .unwrap();
    let node = output.source_map.iter().find(|n| n.id == "label").unwrap();
    assert!(
        node.text_run_id.is_none(),
        "no single run represents the whole source element"
    );
    assert_eq!(node.text_run_ids.len(), 7);
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    let text = node
        .text_run_ids
        .iter()
        .map(|id| {
            artboard
                .with_artboard(|a| a.objects()[*id as usize].clone())
                .unwrap()
                .with(|o| o.as_text_value_run().unwrap().base.text().to_owned())
                .unwrap()
        })
        .collect::<String>();
    assert_eq!(text, "one two\u{a0}three four");
}

#[test]
fn experimental_fixed_space_breaks_preserve_nonbreaking_and_legacy_fonts() {
    use nuxie_runtime::source::{
        text::font_hb::{HbFont, LetterSpacingMode},
        text_engine::{FontRef, TextRun},
    };
    let font = HbFont::decode(include_bytes!("assets/NotoSansOgham-Regular.ttf")).unwrap();
    let css = font
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_experimental_space_breaks(true);
    let shape_breaks = |font: &FontRef, text: &str| {
        let text: Vec<u32> = text.chars().map(u32::from).collect();
        let paragraphs = font.shape_text(
            &text,
            &[TextRun {
                font: Some(font.clone()),
                size: 24.0,
                line_height: 36.0,
                letter_spacing: 0.0,
                unichar_count: text.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        );
        let mut offset = 0;
        let mut breaks = Vec::new();
        for run in &paragraphs[0].runs {
            breaks.extend(run.breaks.iter().map(|index| index + offset));
            offset += run.glyphs.len() as u32;
        }
        breaks
    };
    let breaks = |font: &FontRef, ch: char| shape_breaks(font, &format!("one{ch}two"));
    assert_eq!(
        shape_breaks(&css, "one\u{2003}\u{2060}two"),
        shape_breaks(&font, "one\u{2003}\u{2060}two")
    );
    for ch in [
        '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}',
        '\u{2006}', '\u{2008}', '\u{2009}', '\u{200a}', '\u{205f}', '\u{3000}',
    ] {
        assert_eq!(breaks(&font, ch), vec![0, 7]);
        assert_eq!(breaks(&css, ch), vec![0, 4, 4, 7]);
        assert_eq!(breaks(&css.with_options(&[], &[]), ch), vec![0, 4, 4, 7]);
        let combined = css
            .as_any()
            .downcast_ref::<HbFont>()
            .unwrap()
            .with_letter_spacing_mode(LetterSpacingMode::CssLetterSpacingExperimental);
        assert_eq!(breaks(&combined, ch), vec![0, 4, 4, 7]);
    }
    for ch in ['\u{a0}', '\u{2007}', '\u{202f}'] {
        assert_eq!(breaks(&css, ch), breaks(&font, ch));
        assert_eq!(breaks(&css, ch), vec![0, 7]);
    }
    let restored = css
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_experimental_space_breaks(false);
    assert_eq!(breaks(&restored, '\u{2003}'), vec![0, 7]);
}

#[test]
fn explicit_breaks_preserve_native_text_and_source_identity() {
    use nuxie_html_to_riv::Asset;
    let output = compile(&CompileInput {
        html: "<p id=label> é <br id=cut> one two <br></p>".into(),
        css: "p{display:block;word-spacing:4px}".into(),
        width: 390.0,
        height: 320.0,
        assets: [(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        )]
        .into(),
    })
    .unwrap();
    let node = output.source_map.iter().find(|n| n.id == "label").unwrap();
    assert_eq!(node.text_breaks[0].id, "cut");
    assert_eq!(node.text_breaks[0].text_offset, 1);
    assert_eq!(node.text_breaks[1].text_offset, 9);
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    let text = node
        .text_run_ids
        .iter()
        .map(|id| {
            artboard
                .with_artboard(|a| a.objects()[*id as usize].clone())
                .unwrap()
                .with(|o| o.as_text_value_run().unwrap().base.text().to_owned())
                .unwrap()
        })
        .collect::<String>();
    assert_eq!(text, "é\none two\n");
}

#[test]
fn hidden_text_and_descendants_keep_source_but_emit_no_draw_commands() {
    use nuxie_html_to_riv::Asset;
    let output=compile(&CompileInput {
        html:"<div style='display:none'><p id=label style='display:block'>one<br id=cut>two</p></div>".into(),css:"".into(),width:390.0,height:320.0,
        assets:[("inter".into(),Asset::Font{family:"Inter".into(),weight:400,bytes:include_bytes!("assets/Inter-Regular.ttf").to_vec()})].into(),
    }).unwrap();
    let node = output.source_map.iter().find(|n| n.id == "label").unwrap();
    assert!(!node.text_run_ids.is_empty());
    assert_eq!(node.text_breaks[0].id, "cut");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    artboard.update_pass(true);
    let mut renderer = factory.borrow_mut().make_renderer();
    artboard.draw(&mut renderer);
    assert!(!factory.borrow().stream().contains("drawPath"));
}

#[test]
fn pre_preserves_native_text_with_spacing_runs() {
    use nuxie_html_to_riv::Asset;
    let output = compile(&CompileInput {
        html: "<p id=label> é <br id=cut> one two <br></p>".into(),
        css: "p{display:block;word-spacing:4px;white-space:pre}".into(),
        width: 390.0,
        height: 320.0,
        assets: [(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        )]
        .into(),
    })
    .unwrap();
    let node = output.source_map.iter().find(|n| n.id == "label").unwrap();
    assert_eq!(node.text_breaks[0].id, "cut");
    assert_eq!(node.text_breaks[0].text_offset, 3);
    assert_eq!(node.text_breaks[1].text_offset, 13);
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    let text = node
        .text_run_ids
        .iter()
        .map(|id| {
            artboard
                .with_artboard(|a| a.objects()[*id as usize].clone())
                .unwrap()
                .with(|o| o.as_text_value_run().unwrap().base.text().to_owned())
                .unwrap()
        })
        .collect::<String>();
    assert_eq!(text, " é \n one two \n");
}

#[test]
fn css_default_tabs_follow_font_stops() {
    use nuxie_runtime::source::{
        assets::font_asset::FontAsset,
        core::CoreArena,
        text::font_hb::{HbFont, LetterSpacingMode},
        text_engine::{FontRef, TextRun},
    };
    let bytes = include_bytes!("assets/Inter-Regular.ttf");
    let legacy = HbFont::decode(bytes).unwrap();
    let font = legacy
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_experimental_css_tabs(true);
    let origins = |font: &FontRef, value: &str, spacing| {
        let chars: Vec<u32> = value.chars().map(u32::from).collect();
        let paragraphs = font.shape_text(
            &chars,
            &[TextRun {
                font: Some(font.clone()),
                size: 20.0,
                line_height: 30.0,
                letter_spacing: spacing,
                unichar_count: chars.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        );
        let mut result = Vec::new();
        for run in paragraphs.iter().flat_map(|p| &p.runs) {
            for (i, source) in run.text_indices.iter().enumerate() {
                if chars[*source as usize] == u32::from('X') {
                    result.push(run.xpos[i]);
                }
                if chars[*source as usize] == 9 && !std::rc::Rc::ptr_eq(font, &legacy) {
                    assert!(
                        font.get_path(run.glyphs[i]).empty(),
                        "tab must carry no ink"
                    );
                }
            }
        }
        result
    };
    for (value, spacing, expected) in [
        ("i\tX", 0.0, 43.90625),
        ("MMMM\tX", 0.0, 87.8125),
        ("       \tX", 0.0, 43.90625),
        ("        \tX", 0.0, 87.8125),
        ("i\t\tX", 0.0, 87.8125),
        ("i\tX", 1.0, 51.90625),
        ("MMMM\tX", 1.0, 103.8125),
    ] {
        let actual = origins(&font, value, spacing);
        assert_eq!(actual.len(), 1);
        assert!(
            (actual[0] - expected).abs() < 0.02,
            "{value:?}: {actual:?} vs {expected}"
        );
    }
    assert_eq!(
        origins(&font, "i\tX\nMMMM\tX", 0.0),
        vec![43.90625, 87.8125]
    );
    let before = origins(&legacy, "i\tX", 0.0);
    assert_eq!(before, vec![17.8125]);
    assert_eq!(
        origins(&font.with_options(&[], &[]), "i\tX", 0.0),
        vec![43.90625]
    );
    let arena = CoreArena::default();
    let asset = arena.insert(FontAsset::default());
    FontAsset::set_font_occurrence(&asset, Some(legacy.clone()));
    assert!(FontAsset::set_experimental_css_tabs_occurrence(
        &asset, true
    ));
    assert!(FontAsset::set_experimental_space_breaks_occurrence(
        &asset, true
    ));
    assert!(FontAsset::set_letter_spacing_mode_occurrence(
        &asset,
        LetterSpacingMode::CssLetterSpacingExperimental
    ));
    for replacement in [Some(legacy.clone()), None, Some(legacy.clone())] {
        FontAsset::set_font_occurrence(&asset, replacement);
    }
    let retained = asset
        .with_downcast::<FontAsset, _>(|a| a.font().unwrap())
        .unwrap();
    assert_eq!(origins(&retained, "i\tX", 0.0), vec![43.90625]);
    assert!(retained.preserves_line_break_space(0x2003));
    assert_eq!(
        origins(&legacy, "i\tX", 0.0),
        before,
        "other font occurrence is unchanged"
    );
}

#[test]
fn resolved_underlines_draw_after_import_and_follow_resizing() {
    use nuxie_runtime::source::text::{
        css_decoration::{ResolvedStrikethrough, ResolvedUnderline, SkipInk},
        text::Text,
    };
    let output = compile(&CompileInput {
        html: "<p id=a>words to wrap over several lines as the viewport becomes narrow</p>".into(),
        css: "p{width:100%;font-family:Inter;font-size:20px;line-height:30px}".into(),
        width: 390.0,
        height: 320.0,
        assets: [(
            "font".into(),
            nuxie_html_to_riv::Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        )]
        .into(),
    })
    .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    let text = artboard
        .with_artboard(|a| {
            a.objects()
                .iter()
                .flatten()
                .find(|o| o.is_type_of(Text::TYPE_KEY))
                .cloned()
        })
        .unwrap();
    artboard.update_pass(true);
    let capture = || {
        factory.borrow_mut().clear();
        let mut renderer = factory.borrow().make_renderer();
        artboard.draw(&mut renderer);
        factory.borrow().canonical_recording().stream().to_owned()
    };
    let original = capture();
    assert!(!original.contains("color=0xffff0000"));
    text.with_downcast_mut::<Text, _>(|t| {
        t.set_css_underlines(vec![
            ResolvedUnderline::solid(0xffff0000, 2.0, 1.0, SkipInk::None).unwrap(),
        ]);
        t.set_css_strikethroughs(vec![
            ResolvedStrikethrough::solid(0xff0000ff, 2.0, -7.0, 22.0).unwrap(),
        ]);
    });
    let mut previous_lines = 0;
    for width in [768.0, 390.0, 240.0] {
        artboard.set_size(width, 320.0);
        artboard.update_pass(true);
        let recording = capture();
        let underline = recording
            .lines()
            .filter(|line| line.starts_with("drawPath") && line.contains("color=0xffff0000"))
            .collect::<Vec<_>>()
            .join("\n");
        let lines = text
            .with_downcast::<Text, _>(|t| t.ordered_lines().len())
            .unwrap();
        assert_eq!(
            underline.matches("move").count(),
            lines,
            "one rectangle per laid-out line"
        );
        let strike_draws: Vec<_> = recording
            .lines()
            .filter(|line| line.starts_with("drawPath") && line.contains("color=0xff0000ff"))
            .collect();
        assert_eq!(strike_draws.len(), lines, "one strike per resized line");
        let draws: Vec<_> = recording
            .lines()
            .filter(|line| line.starts_with("drawPath"))
            .collect();
        assert!(
            draws[..lines]
                .iter()
                .all(|line| line.contains("color=0xffff0000"))
        );
        assert!(
            draws[draws.len() - lines..]
                .iter()
                .all(|line| line.contains("color=0xff0000ff")),
            "strikethrough paints after every vector glyph draw"
        );
        assert!(lines > previous_lines);
        previous_lines = lines;
    }
    text.with_downcast_mut::<Text, _>(|t| {
        t.set_css_underlines(vec![
            ResolvedUnderline::solid(0xffff0000, 2.0, 1.0, SkipInk::Auto).unwrap(),
        ])
    });
    artboard.update_pass(true);
    for accepted_before_decline in [0, 1] {
        factory.borrow_mut().clear();
        let mut inner = factory.borrow().make_renderer();
        let mut declining =
            declining_renderer::DecliningRenderer::new(&mut inner, accepted_before_decline);
        artboard.draw(&mut declining);
        assert!(declining.declined, "fixture must exercise backend refusal");
        let fallback = factory.borrow().canonical_recording().stream().to_owned();
        let segments: usize = fallback
            .lines()
            .filter(|line| line.starts_with("drawPath") && line.contains("color=0xffff0000"))
            .map(|line| line.matches("move").count())
            .sum();
        assert!(
            segments > previous_lines,
            "fallback must retain descender gaps"
        );
    }
    let skipped = capture();
    assert!(
        skipped.contains("clipOutRect rect="),
        "descender gaps must survive recording as hard clips"
    );
    let underline_count = skipped
        .lines()
        .filter(|line| line.starts_with("drawPath") && line.contains("color=0xffff0000"))
        .count();
    assert_eq!(
        underline_count, previous_lines,
        "one uncut stripe per laid-out line"
    );
    #[cfg(all(target_os = "macos", feature = "native-glyph-controls"))]
    {
        factory.borrow_mut().clear();
        let mut renderer = factory.borrow().make_renderer();
        let mut cache = nuxie_renderer::glyph_adapter::GlyphCache::new(factory.clone());
        artboard.draw(&mut cache.wrap(&mut renderer));
        assert!(
            cache.stats().misses > 0,
            "exercise actual glyph rasterization"
        );
        let recording = factory.borrow().canonical_recording().stream().to_owned();
        let underline_index = recording
            .lines()
            .position(|line| line.starts_with("drawPath") && line.contains("color=0xffff0000"))
            .expect("glyph rendering must retain underline paths");
        let glyph_index = recording
            .lines()
            .position(|line| line.starts_with("drawImage"))
            .expect("glyph adapter must draw rasterized glyphs");
        let strike_index = recording
            .lines()
            .position(|line| line.starts_with("drawPath") && line.contains("color=0xff0000ff"))
            .expect("glyph rendering retains strikethrough");
        let last_glyph = recording
            .lines()
            .enumerate()
            .filter_map(|(index, line)| line.starts_with("drawImage").then_some(index))
            .last()
            .unwrap();
        assert!(
            strike_index > last_glyph,
            "strikethrough paints after raster glyphs"
        );
        assert!(
            underline_index < glyph_index,
            "underline paints before glyph ink"
        );
    }
    text.with_downcast_mut::<Text, _>(|t| {
        t.set_css_underlines(Vec::new());
        t.set_css_strikethroughs(Vec::new());
    });
    artboard.set_size(390.0, 320.0);
    artboard.update_pass(true);
    assert_eq!(
        capture(),
        original,
        "removing decoration restores the original drawing"
    );
}

#[test]
fn runtime_skip_ink_matches_pinned_chromium_boundaries() {
    use nuxie_runtime::source::text::css_decoration::SkipInk;
    #[derive(serde::Deserialize)]
    struct Reference {
        #[serde(rename = "boundary_cases")]
        checks: Vec<Check>,
    }
    #[derive(serde::Deserialize)]
    struct Check {
        codepoint: String,
        eligible: bool,
    }
    let reference: Reference = serde_json::from_str(include_str!(
        "../validation/underline-skip-eligibility-reference.json"
    ))
    .unwrap();
    assert_eq!(reference.checks.len(), 481);
    for check in reference.checks {
        let scalar = u32::from_str_radix(&check.codepoint, 16).unwrap();
        let character = char::from_u32(scalar).unwrap();
        assert_eq!(
            SkipInk::Auto.skips(character),
            check.eligible,
            "U+{}",
            check.codepoint
        );
        assert!(!SkipInk::None.skips(character));
        assert!(SkipInk::All.skips(character));
    }
}

#[test]
fn css_shaping_precision_matches_chrome_and_survives_font_replacement() {
    use nuxie_runtime::source::{
        assets::font_asset::FontAsset,
        core::CoreArena,
        text::font_hb::{HbFont, LetterSpacingMode, ShapingPrecision},
        text_engine::{FontRef, TextRun},
    };
    let legacy = HbFont::decode(include_bytes!("assets/NuxieJapaneseFixture-Regular.otf")).unwrap();
    let origin = |font: &FontRef| {
        let chars: Vec<u32> = "／＿ agypqj".chars().map(u32::from).collect();
        let shaped = font.shape_text(
            &chars,
            &[TextRun {
                font: Some(font.clone()),
                size: 24.0,
                line_height: 40.0,
                letter_spacing: 0.0,
                unichar_count: chars.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        );
        shaped[0]
            .runs
            .iter()
            .flat_map(|run| run.advances.iter())
            .take(7)
            .sum::<f32>()
    };
    let precise = legacy
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_shaping_precision(ShapingPrecision::CssExperimental);
    // Independent pinned Chrome Canvas prefix and suffix measurements agree.
    let expected = 108.21597290039062_f64;
    assert!((f64::from(origin(&precise)) - expected).abs() < 0.0001);
    assert_eq!(f64::from(origin(&legacy)), 108.22265625);
    let precise = precise
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_letter_spacing_mode(LetterSpacingMode::CssLetterSpacingExperimental);
    let precise = precise
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_experimental_css_tabs(true);
    let precise = precise
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_experimental_space_breaks(true)
        .with_options(&[], &[]);
    assert!((f64::from(origin(&precise)) - expected).abs() < 0.0001);
    let arena = CoreArena::default();
    let asset = arena.insert(FontAsset::default());
    FontAsset::set_font_occurrence(&asset, Some(legacy.clone()));
    assert!(FontAsset::set_shaping_precision_occurrence(
        &asset,
        ShapingPrecision::CssExperimental
    ));
    for replacement in [Some(legacy.clone()), None, Some(legacy.clone())] {
        FontAsset::set_font_occurrence(&asset, replacement);
    }
    let retained = asset
        .with_downcast::<FontAsset, _>(|a| a.font().unwrap())
        .unwrap();
    assert!((f64::from(origin(&retained)) - expected).abs() < 0.0001);
    assert!(FontAsset::set_shaping_precision_occurrence(
        &asset,
        ShapingPrecision::Rive
    ));
    assert_eq!(
        origin(
            &asset
                .with_downcast::<FontAsset, _>(|a| a.font().unwrap())
                .unwrap()
        ),
        origin(&legacy)
    );
}

#[test]
fn css_advance_callback_matches_chrome_scalar_and_pair_widths() {
    use nuxie_runtime::source::{
        text::font_hb::{HbFont, ShapingPrecision},
        text_engine::TextRun,
    };
    let decoded =
        HbFont::decode(include_bytes!("assets/NuxieJapaneseFixture-Regular.otf")).unwrap();
    let font = decoded
        .as_any()
        .downcast_ref::<HbFont>()
        .unwrap()
        .with_shaping_precision(ShapingPrecision::CssExperimental);
    // Pinned Chrome 153 canvas widths, including positive and negative kerning.
    for (text, size, expected) in [
        ("a", 24.0, 13.511993408203125_f64),
        ("p", 24.0, 14.879989624023438),
        ("q", 24.0, 14.879989624023438),
        ("j", 24.0, 6.5999908447265625),
        (" ", 24.0, 5.3759918212890625),
        ("gy", 24.0, 26.447998046875),
        ("ga", 24.0, 26.615982055664062),
        ("agypqj gap agypqj gap agypqj ga", 24.0, 365.44775390625),
        ("g", 36.0, 20.304000854492188),
        // Native robustness control: a subnormal size must not overflow its
        // reciprocal into a large advance. This is not a Chrome reference.
        ("g", 1.0e-40, 0.0),
    ] {
        let chars: Vec<u32> = text.chars().map(u32::from).collect();
        let shaped = font.shape_text(
            &chars,
            &[TextRun {
                font: Some(font.clone()),
                size,
                line_height: 40.0,
                letter_spacing: 0.0,
                unichar_count: chars.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        );
        let total: f64 = shaped[0]
            .runs
            .iter()
            .flat_map(|r| &r.advances)
            .map(|v| f64::from(*v))
            .sum();
        assert_eq!(f64::from(total as f32), expected, "{text:?}");
    }
}
