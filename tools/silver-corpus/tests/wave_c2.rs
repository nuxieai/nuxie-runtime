//! Exact pinned Silver action streams for Wave C2 layout cases.

#[path = "support/browser_layout.rs"]
mod browser_layout;

use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};

const PROVENANCE: &str = "tests/unit_tests/runtime/layout_test.cpp";

fn workspace_root() -> PathBuf {
    Path::new(option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn runtime_root() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

fn replay(id: &str) {
    replay_result(id).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

fn replay_result(id: &str) -> Result<(), silver_corpus::Difference> {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read Silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .unwrap_or_else(|| panic!("missing Silver case {id}"));
    assert_eq!(case.provenance_file, PROVENANCE);
    assert!(
        case.actions
            .executable()
            .is_some_and(|actions| !actions.is_empty()),
        "the shared manifest retains the complete pinned action stream"
    );
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual_bytes = actual.bytes().to_vec();
    let actual = parse_sriv(&actual_bytes).expect("parse Rust SRIV");
    browser_layout::reviewed_result(id, compare_sriv(&expected, &actual), &actual_bytes)
}

#[test]
fn wave_c2_layout_014_collapsing_and_soloing() {
    replay("collapsing_elements");
}

#[test]
fn wave_c2_layout_015_animating_layout_display() {
    replay("layout_display");
}

#[test]
fn wave_c2_layout_016_background_and_foreground_paints() {
    replay("layout_paint");
}

#[test]
fn wave_c2_layout_017_animation_time_databound() {
    replay("layout_anim_bound");
}

#[test]
fn wave_c2_layout_018_animation_nested_artboards() {
    replay("layout_anim_nested");
}

#[test]
fn wave_c2_layout_019_animation_component_list() {
    replay("layout_anim_component_list");
}

// Layout divergence (stock Taffy, CSS semantics): the aspect-ratio box solves
// to its parent's 142x133 inner box through main-axis Fill and cross-axis
// stretch; Yoga keeps 2:1 instead. The first difference stays pinned.
#[test]
fn wave_c2_layout_020_aspect_ratio() {
    // The upstream producer binds its default VMI to the artboard before
    // constructing the state machine, rather than rebinding an existing one.
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml")).unwrap();
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "layout_aspect_ratio")
        .unwrap();
    let actions = case.actions.executable().unwrap();
    assert!(matches!(actions[0], silver_corpus::Action::FrameSize));
    assert!(matches!(
        actions[1],
        silver_corpus::Action::BindDefaultViewModelToArtboard
    ));
    assert!(matches!(
        actions[2],
        silver_corpus::Action::SelectStateMachine
    ));
    assert_eq!(
        actions
            .iter()
            .filter(|action| matches!(action, silver_corpus::Action::Draw))
            .count(),
        33
    );
    let difference =
        replay_result("layout_aspect_ratio").expect_err("layout divergence should remain present");
    assert_eq!(
        difference.to_string(),
        "frame 0, op 42 (addRawPath), field point: expected (142, 71), got (142, 133)"
    );
}

#[test]
fn wave_c2_layout_021_fixed_fill_round_trip() {
    replay("layout_fixed_fill");
}

#[test]
fn wave_c2_layout_022_top_level_hug_artboard() {
    replay("layout_hug_artboard");
}

#[test]
fn aspect_ratio_fill_and_stretch_follow_both_animated_parent_axes() {
    use nuxie_render_api::{PersistentFactory, SerializingFactory};
    use nuxie_runtime::source::layout::layout_style_applier::{
        YGAlign, YGFlexDirection, YGUnit, rive_align_self, rive_dimension, rive_flex_direction,
    };
    use nuxie_runtime::source::math::random::RandomProvider;
    use nuxie_runtime::{File, RuntimeFactoryHandle};

    // Match the upstream TESTING FIFO, and restore the runtime mode even if
    // an assertion unwinds out of this manual producer.
    struct TestingRandom;
    impl Drop for TestingRandom {
        fn drop(&mut self) {
            RandomProvider::clear_testing_mode();
        }
    }
    let _testing_random = TestingRandom;
    RandomProvider::clear_randoms();

    // Literal setup from layout_test.cpp: default VMI bound to the artboard
    // before machine construction. The assertions below express our CSS
    // Fill/stretch contract rather than importing Yoga's ratio priority.
    let bytes = std::fs::read(
        runtime_root().join("tests/unit_tests/assets/layout/layout_aspect_ratio.riv"),
    )
    .unwrap();
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|file| file.artboard_default()).unwrap();
    let (width, height) = artboard.with_artboard(|a| (a.width(), a.height()));
    factory.borrow_mut().frame_size(width as u32, height as u32);
    let view_model = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    artboard.bind_view_model_instance(Some(view_model));
    let machine = artboard.state_machine_at(0).unwrap();
    machine.advance_and_apply(0.1);
    let mut renderer = factory.borrow().make_renderer();

    for frame in 0..33 {
        if frame != 0 {
            factory.borrow_mut().add_frame();
            machine.advance_and_apply(0.016);
        }
        artboard.draw(&mut renderer);
        for (parent_id, child_id, ratio, direction, initial_size) in [
            (3, 5, 0.5, YGFlexDirection::Column, (208.0, 202.0)),
            (9, 12, 2.0, YGFlexDirection::Row, (142.0, 133.0)),
        ] {
            let (parent, child) = artboard.with_artboard(|a| {
                (
                    a.objects()[parent_id].clone().unwrap(),
                    a.objects()[child_id].clone().unwrap(),
                )
            });
            let parent_layout = parent
                .with(|owner| {
                    let layout = owner.as_layout_component().unwrap();
                    let style = layout.layout_style();
                    assert_eq!(style.flex_direction, rive_flex_direction(direction));
                    for padding in [
                        style.padding.left,
                        style.padding.right,
                        style.padding.top,
                        style.padding.bottom,
                    ] {
                        assert_eq!(padding.into_raw().value(), 16.0);
                    }
                    layout.layout()
                })
                .unwrap();
            child
                .with(|owner| {
                    assert_eq!(
                        owner.as_component().unwrap().parent_id() as usize,
                        parent_id
                    );
                    let layout = owner.as_layout_component().unwrap();
                    let style = layout.layout_style();
                    assert_eq!(style.aspect_ratio, Some(ratio));
                    assert!(style.size.width.is_auto() && style.size.height.is_auto());
                    assert_eq!(style.flex_basis, rive_dimension(0.0, YGUnit::Percent));
                    assert_eq!((style.flex_grow, style.flex_shrink), (1.0, 1.0));
                    assert_eq!(style.align_self, rive_align_self(YGAlign::Stretch));
                    let actual = layout.layout();
                    assert_eq!((actual.left(), actual.top()), (16.0, 16.0));
                    // Both axes are determined by the animated parent's inner box:
                    // flex-grow fills its main axis, stretch fills its cross axis.
                    assert_eq!(
                        actual.width(),
                        parent_layout.width() - 32.0,
                        "frame {frame}, child {child_id} width"
                    );
                    assert_eq!(
                        actual.height(),
                        parent_layout.height() - 32.0,
                        "frame {frame}, child {child_id} height"
                    );
                    if frame == 0 {
                        assert_eq!((actual.width(), actual.height()), initial_size);
                    }
                })
                .unwrap();
        }
    }
}
