// Independent oracle: pinned C++ de3e86090892b68072e7d4505e8d979386fc9a30,
// rive_cpp_probe --text-measure-width-samples new_text.riv Inter_18pt-Regular.ttf.
// The probe uses Inter at 40 px, 44 px line advances, authored width 1,
// and an exact offered width 354. cpp-text-width.json records the full output:
// [{"participant":0,"width":1,"height":267.521576},
//  {"participant":1,"width":1,"height":47.5215874},
//  {"participant":2,"width":1,"height":47.5215874}].
// Known gap: Rust main returns x=354 at text.rs:2210-2212, where C++ returns 1.
// These assertions pin heights only, not that existing width-return difference:
// https://universe.basis.dev/issue/UNIV-3932.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    generated::{
        core_registry::CoreRegistry,
        text::{
            text_base::TextBase, text_style_base::TextStyleBase,
            text_value_run_base::TextValueRunBase,
        },
    },
    layout::{layout_enums::LayoutScaleType, layout_measure_mode::LayoutMeasureMode},
    math::vec2d::Vec2D,
    text::text::{Text, TextValueRunHandle},
};
use nuxie_runtime::{CoreHandle, File, RuntimeFactoryHandle};

fn set_text(owner: &CoreHandle, content: &str) {
    let runs = owner
        .with_downcast::<Text, _>(|text| text.runs().to_vec())
        .unwrap();
    for (index, run) in runs.iter().enumerate() {
        let TextValueRunHandle::Core(run) = run else {
            panic!("imported run")
        };
        assert!(CoreRegistry::set_string_handle(
            run,
            i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
            if index == 0 {
                content.into()
            } else {
                String::new()
            }
        ));
    }
}

fn measure(participant_width: Option<LayoutScaleType>, content: &str, offered_width: f32) -> Vec2D {
    let root = std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").unwrap());
    let bytes = std::fs::read(root.join("tests/unit_tests/assets/new_text.riv")).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let board = file.with_file(File::artboard_default).unwrap();
    let text = board.with_artboard(|b| b.find_all_handles::<Text>())[0].clone();
    // new_text.riv's first Text has one run. Both this test and the probe
    // populate and measure that Text; the probe also clears other Texts' runs.
    set_text(&text, content);
    for (key, value) in [
        (TextBase::SIZING_VALUE_PROPERTY_KEY, 1),
        (TextBase::OVERFLOW_VALUE_PROPERTY_KEY, 0),
        (TextBase::WRAP_VALUE_PROPERTY_KEY, 0),
    ] {
        assert!(CoreRegistry::set_uint_handle(&text, i32::from(key), value));
    }
    use nuxie_runtime::source::{
        assets::font_asset::FontAsset,
        text::{font_hb::HbFont, text_style::TextStyle},
    };
    for style in board.with_artboard(|b| b.find_all_handles::<TextStyle>()) {
        let asset = style.insert_sibling(FontAsset::default()).unwrap();
        let font = std::fs::read(root.join("tests/unit_tests/assets/fonts/Inter_18pt-Regular.ttf"))
            .unwrap();
        FontAsset::set_font_occurrence(&asset, Some(HbFont::decode(&font).unwrap()));
        TextStyle::set_asset_occurrence(&style, Some(asset));
        for (key, value) in [
            (TextStyleBase::FONT_SIZE_PROPERTY_KEY, 40.0),
            (TextStyleBase::LINE_HEIGHT_PROPERTY_KEY, 44.0),
        ] {
            assert!(CoreRegistry::set_double_handle(
                &style,
                i32::from(key),
                value
            ));
        }
    }
    if let Some(width) = participant_width {
        use nuxie_runtime::source::{
            generated::layout::layout_sizing_style_base::LayoutSizingStyleBase,
            layout::layout_participant::LayoutParticipant,
        };
        let participant = text.insert_sibling(LayoutParticipant::default()).unwrap();
        for (key, value) in [
            (
                LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
                width,
            ),
            (
                LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
                LayoutScaleType::Hug,
            ),
        ] {
            assert!(CoreRegistry::set_uint_handle(
                &participant,
                i32::from(key),
                value as u32
            ));
        }
        text.with_downcast_mut::<Text, _>(|t| t.base.add_child(participant))
            .unwrap();
    }
    assert!(CoreRegistry::set_double_handle(
        &text,
        i32::from(TextBase::WIDTH_PROPERTY_KEY),
        1.0
    ));
    text.with_downcast_mut::<Text, _>(|t| {
        t.measure_layout(
            offered_width,
            LayoutMeasureMode::Exactly,
            f32::NAN,
            LayoutMeasureMode::Undefined,
        )
    })
    .unwrap()
}

#[test]
fn upstream_fill_and_fixed_participants_measure_at_the_offered_width() {
    for width in [LayoutScaleType::Fill, LayoutScaleType::Fixed] {
        let measured = measure(Some(width), "Choose", 354.0);
        // This height excludes shaping at authored width 1. The headline
        // case below also distinguishes the offered limit from unbounded.
        assert!((measured.y - 47.5215874).abs() < 0.001, "{measured:?}");
    }
}

#[test]
fn upstream_text_without_a_width_owner_shapes_at_authored_width_in_an_exact_slot() {
    // Upstream deliberately caps measurement at authored width without a
    // fill/fixed LayoutParticipant. An exact layout offer alone does not own
    // this shaping axis. The returned x difference is documented above.
    let measured = measure(None, "Choose", 354.0);
    assert!((measured.y - 267.521576).abs() < 0.001, "{measured:?}");
}

#[test]
fn upstream_fill_and_fixed_participants_wrap_the_headline_at_the_offered_width() {
    // de3e8609 --text-measure-headline-samples: cpp-text-headline.json reports
    // (width=1, height=135.521591) for both participants. The matching
    // --text-measure-headline-unbounded-samples output is (1, 47.5215874).
    // These observations distinguish the finite offer from unbounded shaping.
    for width in [LayoutScaleType::Fill, LayoutScaleType::Fixed] {
        let content = "Choose what deserves your attention.";
        let bounded = measure(Some(width), content, 354.0);
        let unbounded = measure(Some(width), content, f32::MAX);
        assert!((bounded.y - 135.521591).abs() < 0.001, "{bounded:?}");
        assert!((unbounded.y - 47.5215874).abs() < 0.001, "{unbounded:?}");
    }
}
