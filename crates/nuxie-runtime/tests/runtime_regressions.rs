#[path = "support/riv_bytes.rs"]
mod riv_bytes;

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

fn check_fill_width_text(participant_layout: bool, solve_layout: bool) {
    let root = std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").unwrap());
    let bytes = if solve_layout {
        layout_fixture()
    } else {
        std::fs::read(root.join("tests/unit_tests/assets/new_text.riv")).unwrap()
    };
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
        if solve_layout {
            use nuxie_runtime::source::{
                generated::layout_component_base::LayoutComponentBase,
                layout_component::LayoutComponent,
            };
            assert!(CoreRegistry::set_double_handle(
                &board.core_handle(),
                i32::from(LayoutComponentBase::WIDTH_PROPERTY_KEY),
                width
            ));
            board.advance_default(0.0);
            let wrapper = text
                .with(|text| text.component_parent_handle())
                .flatten()
                .unwrap();
            let (actual_width, height) = wrapper
                .with_downcast::<LayoutComponent, _>(|layout| {
                    (layout.layout_width(), layout.layout_height())
                })
                .unwrap();
            assert!(
                (actual_width - width).abs() < 0.1,
                "fill width: {actual_width}"
            );
            assert!(
                (height - expected_height).abs() < 0.1,
                "solved {content:?} at {width}: {height}, expected {expected_height}"
            );
            continue;
        }
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
    check_fill_width_text(true, false);
}
#[test]
fn controlled_fill_width_text_measures_one_and_three_lines_at_354() {
    check_fill_width_text(false, false);
}

#[test]
fn fill_width_hug_wrapper_solves_text_height_without_a_participant() {
    check_fill_width_text(false, true);
}

fn layout_fixture() -> Vec<u8> {
    use nuxie_runtime::source::generated::{
        artboard_base::ArtboardBase,
        backboard_base::BackboardBase,
        component_base::ComponentBase,
        layout::{
            layout_component_style_base::LayoutComponentStyleBase,
            layout_sizing_style_base::LayoutSizingStyleBase,
        },
        layout_component_base::LayoutComponentBase,
    };
    let mut riv = riv_bytes::RivBytes::default();
    riv.object(BackboardBase::TYPE_KEY);
    riv.end();
    riv.object(ArtboardBase::TYPE_KEY); // 0: fixed-width container
    riv.prop_float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 354.0);
    riv.prop_float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 500.0);
    riv.prop_uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 1);
    riv.end();
    riv.object(LayoutComponentStyleBase::TYPE_KEY); // 1: container style
    riv.end();
    riv.object(LayoutComponentBase::TYPE_KEY); // 2: fill/hug wrapper
    riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0);
    riv.prop_uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 3);
    riv.end();
    riv.object(LayoutComponentStyleBase::TYPE_KEY); // 3: wrapper style
    riv.prop_bool(
        LayoutComponentStyleBase::INTRINSICALLY_SIZED_VALUE_PROPERTY_KEY,
        true,
    );
    riv.prop_uint(
        LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
        LayoutScaleType::Fill as u64,
    );
    riv.prop_uint(
        LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
        LayoutScaleType::Hug as u64,
    );
    riv.end();
    riv.object(TextBase::TYPE_KEY); // 4: no LayoutParticipant
    riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2);
    riv.prop_uint(TextBase::SIZING_VALUE_PROPERTY_KEY, 1);
    riv.prop_float(TextBase::WIDTH_PROPERTY_KEY, 1.0);
    riv.end();
    riv.object(TextStyleBase::TYPE_KEY); // 5
    riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 4);
    riv.prop_float(TextStyleBase::FONT_SIZE_PROPERTY_KEY, 40.0);
    riv.prop_float(TextStyleBase::LINE_HEIGHT_PROPERTY_KEY, 44.0);
    riv.end();
    riv.object(TextValueRunBase::TYPE_KEY); // 6
    riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 4);
    riv.prop_uint(TextValueRunBase::STYLE_ID_PROPERTY_KEY, 5);
    riv.prop_string(TextValueRunBase::TEXT_PROPERTY_KEY, "Choose");
    riv.end();
    riv.bytes()
}
