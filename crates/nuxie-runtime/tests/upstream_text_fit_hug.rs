//! The three numeric text_test.cpp regressions added at upstream 45d4d01d.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    generated::{
        core_registry::CoreRegistry,
        text::{text_base::TextBase, text_value_run_base::TextValueRunBase},
    },
    layout_component::LayoutComponent,
    text::text::{Text, TextValueRunHandle},
    text_engine::{TextOverflow, TextTrimBottom, TextTrimTop, pack_text_vertical_trim},
};
use nuxie_runtime::{CoreHandle, File, RuntimeFactoryHandle, RuntimeFileHandle};

fn import_text_with_minor_version(minor: u8) -> RuntimeFileHandle {
    let path = std::env::var_os("RIVE_RUNTIME_DIR").map_or_else(
        || {
            std::path::PathBuf::from(
                option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
            )
            .join("../../fixtures/sync/fit_font_size_hug_test.riv")
        },
        |root| {
            std::path::PathBuf::from(root)
                .join("tests/unit_tests/assets/fit_font_size_hug_test.riv")
        },
    );
    let mut bytes = std::fs::read(path).expect("fit_font_size_hug_test fixture");
    assert!(bytes.len() > 5);
    assert_eq!(bytes[4], 7);
    bytes[5] = minor;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("fixture imports")
}

fn parent(owner: &CoreHandle) -> CoreHandle {
    owner
        .with(|owner| owner.component_parent_handle())
        .flatten()
        .expect("parent")
}

fn solve_title_layout(file: &RuntimeFileHandle, resizes_box: bool) -> (f32, f32) {
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let texts = artboard.with_artboard(|artboard| artboard.find_all_handles::<Text>());
    let title = texts
        .into_iter()
        .find(|text| {
            text.with_downcast::<Text, _>(|text| {
                text.runs().first().is_some_and(|run| match run {
                    TextValueRunHandle::Core(run) => run
                        .with(|run| {
                            !run.as_text_value_run()
                                .expect("text run")
                                .base
                                .text()
                                .is_empty()
                        })
                        .unwrap(),
                    TextValueRunHandle::Runtime(run) => !run.borrow().base.text().is_empty(),
                })
            })
            .unwrap()
        })
        .expect("nonempty title");
    assert_eq!(
        title.with_downcast::<Text, _>(Text::overflow).unwrap(),
        TextOverflow::FitFontSize
    );
    assert!(CoreRegistry::set_bool_handle(
        &title,
        i32::from(TextBase::FIT_FONT_SIZE_RESIZES_BOX_PROPERTY_KEY),
        resizes_box
    ));
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
    let vmi = file.with_file_mut(|file| {
        if id == u32::MAX {
            file.create_view_model_instance_for_artboard(artboard.core_handle())
        } else {
            file.create_view_model_instance_at(id as usize, 0)
        }
    });
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi));
    machine.advance_and_apply(0.0);
    let hug = parent(&title);
    let hug_height = hug
        .with_downcast::<LayoutComponent, _>(LayoutComponent::layout_height)
        .expect("title parent is LayoutComponent");
    let container = parent(&hug);
    assert!(
        container
            .with_downcast::<LayoutComponent, _>(|_| ())
            .is_some()
    );
    let children = container
        .with(|owner| owner.as_container_component().unwrap().children().to_vec())
        .unwrap();
    let bar = children
        .into_iter()
        .find(|child| child != &hug && child.with_downcast::<LayoutComponent, _>(|_| ()).is_some())
        .expect("bar sibling");
    (
        hug_height,
        bar.with_downcast::<LayoutComponent, _>(LayoutComponent::layout_y)
            .unwrap(),
    )
}

fn check(minor: u8, resizes_box: bool, height: f32, bar_y: f32) {
    let file = import_text_with_minor_version(minor);
    let actual = solve_title_layout(&file, resizes_box);
    // Exact upstream Approx(...).margin(0.5f) expectations.
    assert!(
        (actual.0 - height).abs() <= 0.5,
        "hug height {} != {height}",
        actual.0
    );
    assert!(
        (actual.1 - bar_y).abs() <= 0.5,
        "bar y {} != {bar_y}",
        actual.1
    );
}

#[test]
fn fit_font_size_hug_slot_keeps_authored_size_before_7_4() {
    check(3, true, 423.49, 439.49);
}
#[test]
fn fit_font_size_hug_slot_tracks_fitted_text_at_7_4() {
    check(4, true, 292.81, 308.81);
}
#[test]
fn fit_font_size_hug_slot_honors_resizes_box_at_7_4() {
    check(4, false, 423.49, 439.49);
}

#[test]
fn fit_font_size_measure_returns_for_a_huge_font_size() {
    use nuxie_runtime::source::{
        generated::text::text_style_base::TextStyleBase,
        layout::layout_measure_mode::LayoutMeasureMode, text::text_style::TextStyle,
    };
    let file = import_text_with_minor_version(4);
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let title = artboard
        .with_artboard(|board| board.find_all_handles::<Text>())
        .into_iter()
        .find(|text| {
            text.with_downcast::<Text, _>(|text| {
                text.runs().first().is_some_and(|run| match run {
                    TextValueRunHandle::Core(run) => run
                        .with(|run| !run.as_text_value_run().unwrap().base.text().is_empty())
                        .unwrap(),
                    TextValueRunHandle::Runtime(run) => !run.borrow().base.text().is_empty(),
                })
            })
            .unwrap()
        })
        .expect("nonempty title");
    assert!(CoreRegistry::set_bool_handle(
        &title,
        i32::from(TextBase::FIT_FONT_SIZE_RESIZES_BOX_PROPERTY_KEY),
        true
    ));
    for size in [f32::INFINITY, f32::NAN, 3e9] {
        for style in artboard.with_artboard(|board| board.find_all_handles::<TextStyle>()) {
            assert!(CoreRegistry::set_double_handle(
                &style,
                i32::from(TextStyleBase::FONT_SIZE_PROPERTY_KEY),
                size
            ));
        }
        title
            .with_downcast_mut::<Text, _>(|title| {
                title.measure_layout(
                    1e38,
                    LayoutMeasureMode::AtMost,
                    1e38,
                    LayoutMeasureMode::AtMost,
                );
            })
            .unwrap();
        artboard.advance_default(0.0);
    }
}

fn check_trimmed_hug_fit(top: TextTrimTop, bottom: TextTrimBottom) -> (f32, f32) {
    let file = import_text_with_minor_version(4);
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let title = artboard
        .with_artboard(|board| board.find_all_handles::<Text>())
        .into_iter()
        .find(|text| {
            text.with_downcast::<Text, _>(|text| {
                text.runs().first().is_some_and(|run| match run {
                    TextValueRunHandle::Core(run) => run
                        .with(|run| !run.as_text_value_run().unwrap().base.text().is_empty())
                        .unwrap(),
                    TextValueRunHandle::Runtime(run) => !run.borrow().base.text().is_empty(),
                })
            })
            .unwrap()
        })
        .expect("nonempty title");
    assert_eq!(
        title.with_downcast::<Text, _>(Text::overflow).unwrap(),
        TextOverflow::FitFontSize
    );
    assert!(CoreRegistry::set_bool_handle(
        &title,
        i32::from(TextBase::FIT_FONT_SIZE_RESIZES_BOX_PROPERTY_KEY),
        true
    ));
    let run = title
        .with_downcast::<Text, _>(|text| text.runs()[0].clone())
        .unwrap();
    match run {
        TextValueRunHandle::Core(run) => assert!(CoreRegistry::set_string_handle(
            &run,
            i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
            "Ag".into()
        )),
        TextValueRunHandle::Runtime(run) => run.borrow_mut().set_bound_text("Ag".into()),
    }
    assert!(CoreRegistry::set_uint_handle(
        &title,
        i32::from(TextBase::VERTICAL_TRIM_VALUE_PROPERTY_KEY),
        pack_text_vertical_trim(top, bottom)
    ));
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let id = artboard.with_artboard(|board| board.base.view_model_id());
    let vmi = file.with_file_mut(|file| {
        if id == u32::MAX {
            file.create_view_model_instance_for_artboard(artboard.core_handle())
        } else {
            file.create_view_model_instance_at(id as usize, 0)
        }
    });
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi));
    machine.advance_and_apply(0.0);
    let height = parent(&title)
        .with_downcast::<LayoutComponent, _>(LayoutComponent::layout_height)
        .expect("title parent is LayoutComponent");
    let size = title
        .with_downcast::<Text, _>(|text| {
            assert_eq!(text.shape().len(), 1);
            assert!(!text.shape()[0].runs.is_empty());
            text.shape()[0].runs[0].size
        })
        .unwrap();
    (height, size)
}

/// Complete f3e99df8 regression: inspect the chosen shaped-run size, not its box.
#[test]
fn vertical_trim_does_not_shrink_a_fit_font_size_hug_text() {
    let (untrimmed_height, untrimmed_size) =
        check_trimmed_hug_fit(TextTrimTop::None, TextTrimBottom::None);
    let (trimmed_height, trimmed_size) =
        check_trimmed_hug_fit(TextTrimTop::Cap, TextTrimBottom::Alphabetic);
    let approx = |actual: f32, expected: f32, margin: f64| {
        let delta = (f64::from(actual) - f64::from(expected)).abs();
        delta <= margin || delta <= 100.0 * f64::from(f32::EPSILON) * f64::from(expected).abs()
    };
    assert!(approx(untrimmed_size, 175.0, f64::from(0.01f32)));
    assert!(approx(trimmed_size, untrimmed_size, f64::from(0.01f32)));
    assert!(trimmed_height < untrimmed_height);
    assert!(approx(untrimmed_height, 211.74, f64::from(0.5f32)));
    assert!(approx(trimmed_height, 127.32, f64::from(0.5f32)));
}
