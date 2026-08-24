//! Expected-red action-stream ports from pinned
//! `tests/unit_tests/runtime/data_binding_artboards_test.cpp`.

use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
enum Action {
    Import(&'static str),
    SelectArtboard(&'static str),
    FrameSize,
    CreateViewModel,
    BindViewModel,
    Advance(f32),
    Draw,
    Frame,
    SetArtboard(&'static str, Option<(&'static str, &'static str)>),
    SetArtboardIndex(&'static str, u64),
    SetBoundViewModel(&'static str, &'static str, &'static str),
    ReplaceViewModel(&'static str, &'static str),
    SetString(&'static str, &'static str),
    SetBool(&'static str, bool),
    Fire(&'static str, usize),
    PointerDown(f32, f32),
    PointerUp(f32, f32),
    ExpectNested(&'static str, bool),
    Compare(&'static str),
}

fn pinned_fixture(name: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    root.join("tests/unit_tests/assets").join(name)
}

fn execute_until_missing_cross_file_registration(actions: &[Action]) {
    for action in actions {
        match *action {
            Action::Import(name) => {
                let fixture = pinned_fixture(name);
                let bytes = std::fs::read(&fixture).unwrap_or_else(|error| {
                    panic!("missing fixture {}: {error}", fixture.display())
                });
                nuxie::File::import(&bytes)
                    .unwrap_or_else(|error| panic!("fixture {name} imports: {error}"));
            }
            Action::SelectArtboard(name) => assert!(!name.is_empty()),
            Action::FrameSize
            | Action::CreateViewModel
            | Action::BindViewModel
            | Action::Draw
            | Action::Frame => {}
            Action::Advance(seconds) => assert!(seconds >= 0.0),
            Action::SetArtboard(property, source) => {
                assert!(!property.is_empty());
                if let Some((file, artboard)) = source {
                    assert!(!file.is_empty() && !artboard.is_empty());
                }
            }
            Action::SetArtboardIndex(property, value) => {
                assert!(!property.is_empty());
                assert!(value <= u64::MAX);
            }
            Action::SetBoundViewModel(property, model, instance) => {
                assert!(!property.is_empty() && !model.is_empty() && !instance.is_empty());
            }
            Action::ReplaceViewModel(property, source) => {
                assert!(!property.is_empty() && !source.is_empty());
            }
            Action::SetString(property, value) => {
                assert!(!property.is_empty() && !value.is_empty());
            }
            Action::SetBool(property, value) => {
                assert!(!property.is_empty());
                assert!(value || !value);
            }
            Action::Fire(property, count) => assert!(!property.is_empty() && count > 0),
            Action::PointerDown(x, y) | Action::PointerUp(x, y) => {
                assert!(x.is_finite() && y.is_finite());
            }
            Action::ExpectNested(name, expected) => {
                assert!(!name.is_empty());
                assert!(expected || !expected);
            }
            Action::Compare(name) => {
                assert!(!name.is_empty());
                panic!(
                    "expected-red: execute the preserved stream after external bindable-artboard registration exists"
                );
            }
        }
    }
}

#[test]
#[ignore = "expected-red: external bindable-artboard registration is absent"]
fn data_binding_artboards_from_same_and_different_sources() {
    execute_until_missing_cross_file_registration(&[
        Action::Import("data_binding_artboards_test.riv"),
        Action::Import("data_binding_artboards_source_test.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch1"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "source_1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Fire("ch/tr", 1),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Fire("ch/tr", 1),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "source_2")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "source_1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", None),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_binding_artboards_test"),
    ]);
}

#[test]
#[ignore = "expected-red: recursive artboard stream diverges at frame 1 makeRenderPaint"]
fn recursive_data_binding_artboards_are_skipped() {
    execute_until_missing_cross_file_registration(&[
        Action::Import("data_binding_artboards_test.riv"),
        Action::SelectArtboard("recursive-grand-parent"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-grand-child-1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-parent")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-grand-parent")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-grand-child-2")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_binding_artboards_test_recursive"),
    ]);
}

#[test]
#[ignore = "expected-red: external default bindable-artboard registration is absent"]
fn default_data_binding_artboard_from_different_source() {
    execute_until_missing_cross_file_registration(&[
        Action::Import("data_binding_artboards_test.riv"),
        Action::Import("data_binding_artboards_source_test.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch1"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "default")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_binding_artboards_default_test"),
    ]);
}

#[test]
#[ignore = "expected-red: scripted external Artboard input registration is absent"]
fn scripted_artboard_input_bound_to_internal_and_external_artboards() {
    execute_until_missing_cross_file_registration(&[
        Action::Import("data_bind_artboard_input.riv"),
        Action::Import("data_binding_artboards_source_test.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::Draw,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "artboardProperty",
            Some(("data_bind_artboard_input.riv", "child2")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "artboardProperty",
            Some(("data_bind_artboard_input.riv", "child1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "artboardProperty",
            Some(("data_binding_artboards_source_test.riv", "default")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboardIndex("artboardProperty", 1),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboardIndex("artboardProperty", 10),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_bind_artboard_input"),
    ]);
}

#[test]
#[ignore = "expected-red: external artboard plus view-model graph registration is absent"]
fn external_artboard_with_no_initial_source() {
    execute_until_missing_cross_file_registration(&[
        Action::Import("databind_external_artboard_main.riv"),
        Action::Import("databind_external_artboard_child.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::Advance(0.0),
        Action::Draw,
        Action::Frame,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::SetArtboard(
            "ab",
            Some(("databind_external_artboard_child.riv", "ExternalChild")),
        ),
        Action::SetBoundViewModel("ab", "Child", "Instance"),
        Action::ReplaceViewModel("child", "Child/Instance"),
        Action::SetString("child/label", "updated label"),
        Action::Advance(0.016),
        Action::Draw,
        Action::Compare("databind_external_artboard_main"),
    ]);
}

#[test]
#[ignore = "expected-red: external artboard with bound view-model injection is absent"]
fn bound_artboard_with_view_model_instance_resets_properties() {
    execute_until_missing_cross_file_registration(&[
        Action::Import("bindable_artboard_nesty.riv"),
        Action::Import("bindable_artboard_child.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::SetArtboard(
            "someArtboard",
            Some(("bindable_artboard_child.riv", "Artboard")),
        ),
        Action::SetBoundViewModel("someArtboard", "ViewModelRenamed", "new"),
        Action::BindViewModel,
        Action::Advance(0.016),
        Action::Draw,
        Action::Frame,
        Action::PointerDown(250.0, 250.0),
        Action::PointerUp(250.0, 250.0),
        Action::Advance(0.016),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::PointerDown(250.0, 250.0),
        Action::PointerUp(250.0, 250.0),
        Action::Advance(0.016),
        Action::Draw,
        Action::Compare("bindable_artboard_nesty"),
    ]);
}

#[test]
#[ignore = "expected-red: multiple external artboard targets are not registered"]
fn multiple_targets_bound_to_same_artboard_property_bidirectionally() {
    let mut actions = vec![
        Action::Import("bidirectional_binding_source.riv"),
        Action::Import("bidirectional_binding_target_1.riv"),
        Action::Import("bidirectional_binding_target_2.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::Advance(0.0),
        Action::SetBool("costume_db_bool", true),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.016),
        Action::Draw,
    ];
    for source in [
        "bidirectional_binding_target_1.riv",
        "bidirectional_binding_target_2.riv",
        "bidirectional_binding_target_1.riv",
        "bidirectional_binding_target_2.riv",
    ] {
        actions.extend([
            Action::Frame,
            Action::SetArtboard("costume_db_artboard", Some((source, "costume_artboard"))),
            Action::Advance(0.016),
            Action::Draw,
            Action::Frame,
            Action::Advance(0.016),
            Action::Draw,
        ]);
    }
    actions.push(Action::Compare("bidirectional_binding_source"));
    execute_until_missing_cross_file_registration(&actions);
}

#[test]
#[ignore = "expected-red: nested-artboard instance presence is not exposed through the facade"]
fn null_bound_artboard_swap_survives_pending_layout_sync() {
    execute_until_missing_cross_file_registration(&[
        Action::Import("databind_null_artboard_swap.riv"),
        Action::SelectArtboard("default"),
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::ExpectNested("swap host", true),
        Action::Advance(0.0),
        Action::Advance(0.0),
        Action::ExpectNested("swap host", false),
        Action::Compare("no-crash-and-null-instance"),
    ]);
}
