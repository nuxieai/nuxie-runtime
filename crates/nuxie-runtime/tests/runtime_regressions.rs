use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    generated::{
        core_registry::CoreRegistry,
        text::{
            text_base::TextBase, text_style_base::TextStyleBase,
            text_value_run_base::TextValueRunBase,
        },
    },
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
    },
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

fn check_fill_width_text(participant_layout: bool) {
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
    let text = board.with_artboard(|board| board.find_all_handles::<Text>())[0].clone();
    for (key, value) in [
        (TextBase::SIZING_VALUE_PROPERTY_KEY, 1),
        (TextBase::OVERFLOW_VALUE_PROPERTY_KEY, 0),
        (TextBase::WRAP_VALUE_PROPERTY_KEY, 0),
    ] {
        assert!(CoreRegistry::set_uint_handle(&text, i32::from(key), value));
    }
    let styles = board.with_artboard(|board| {
        board.find_all_handles::<nuxie_runtime::source::text::text_style::TextStyle>()
    });
    for style in styles {
        use nuxie_runtime::source::{
            assets::font_asset::FontAsset,
            text::{font_hb::HbFont, text_style::TextStyle},
        };
        let asset = style.insert_sibling(FontAsset::default()).unwrap();
        let bytes =
            std::fs::read(root.join("tests/unit_tests/assets/fonts/Inter_18pt-Regular.ttf"))
                .unwrap();
        FontAsset::set_font_occurrence(&asset, Some(HbFont::decode(&bytes).unwrap()));
        TextStyle::set_asset_occurrence(&style, Some(asset));

        assert!(CoreRegistry::set_double_handle(
            &style,
            i32::from(TextStyleBase::FONT_SIZE_PROPERTY_KEY),
            40.0
        ));
        assert!(CoreRegistry::set_double_handle(
            &style,
            i32::from(TextStyleBase::LINE_HEIGHT_PROPERTY_KEY),
            44.0
        ));
    }
    if participant_layout {
        let participant = text
            .insert_sibling(
                nuxie_runtime::source::layout::layout_participant::LayoutParticipant::default(),
            )
            .unwrap();
        use nuxie_runtime::source::generated::layout::layout_sizing_style_base::LayoutSizingStyleBase;
        assert!(CoreRegistry::set_uint_handle(
            &participant,
            i32::from(LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY),
            LayoutScaleType::Fill as u32
        ));
        assert!(CoreRegistry::set_uint_handle(
            &participant,
            i32::from(LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY),
            LayoutScaleType::Hug as u32
        ));
        text.with_downcast_mut::<Text, _>(|text| text.base.add_child(participant))
            .unwrap();
    }
    assert!(CoreRegistry::set_double_handle(
        &text,
        i32::from(TextBase::WIDTH_PROPERTY_KEY),
        1.0
    ));
    // Inter 40px: two words fit in 354px, while only one fits in 200px.
    // First-line height is 47.521587px; each later line advances 44px.
    for (content, width, expected_lines, expected_height) in [
        ("Choose", 354.0, 1, 47.521587),
        ("Choose\nChoose\nChoose", 354.0, 3, 135.52159),
        ("Choose Choose Choose", 354.0, 2, 91.52159),
        ("Choose Choose Choose", 200.0, 3, 135.52159),
    ] {
        set_text(&text, content);
        let first_measure = text
            .with_downcast_mut::<Text, _>(|text| {
                text.measure_layout(
                    width,
                    LayoutMeasureMode::Exactly,
                    f32::NAN,
                    LayoutMeasureMode::Undefined,
                )
            })
            .unwrap();
        assert!(
            (first_measure.y - expected_height).abs() < 0.1,
            "first measurement for {content:?}: {}",
            first_measure.y
        );

        text.with_downcast_mut::<Text, _>(|text| {
            text.control_size(
                Vec2D::new(width, 0.0),
                LayoutScaleType::Fill,
                LayoutScaleType::Hug,
                LayoutDirection::Ltr,
            )
        })
        .unwrap();
        board.advance_default(0.0);
        let (measured, lines) = text
            .with_downcast_mut::<Text, _>(|text| {
                let measured = text.measure_layout(
                    width,
                    LayoutMeasureMode::Exactly,
                    f32::NAN,
                    LayoutMeasureMode::Undefined,
                );
                (measured, text.ordered_lines().len())
            })
            .unwrap();
        assert_eq!(lines, expected_lines, "rendered lines for {content:?}");
        assert!(
            (measured.y - expected_height).abs() < 0.1,
            "measured height for {content:?}: {}",
            measured.y
        );
    }
}

#[test]
fn participant_fill_width_text_measures_one_and_three_lines_at_354() {
    check_fill_width_text(true);
}
#[test]
fn controlled_fill_width_text_measures_one_and_three_lines_at_354() {
    check_fill_width_text(false);
}
