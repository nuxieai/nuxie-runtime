//! All six tests from tests/unit_tests/runtime/frame_probe_test.cpp at 955d6a05.
//! Kept in the unit-test target so upstream TESTING instrumentation stays absent
//! from shipping builds. Hidden report-only sweeps map to ignored Rust tests.

use crate::source::{
    generated::{
        core_registry::CoreRegistry, node_base::NodeBase,
        transform_component_base::TransformComponentBase,
    },
    shapes::{path::Path, shape::Shape},
};
use crate::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use std::path::PathBuf;

const FRAME_SECONDS: f32 = 1.0 / 60.0;
const SETTLE_FRAMES: usize = 120;
const PROBE_FRAMES: usize = 60;

#[derive(Default)]
struct ProbeResult {
    layout_passes: u64,
    keep_going_frames: usize,
}

fn report(kind: &str, path: &str, result: &ProbeResult) {
    println!(
        "[frame_probe] {kind:<8} {path:<52} layout passes {:3} / {PROBE_FRAMES} frames, keep-going {:2} / {PROBE_FRAMES}",
        result.layout_passes, result.keep_going_frames
    );
}

fn read_file(path: &str) -> RuntimeFileHandle {
    let upstream = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(PathBuf::from(upstream).join("tests/unit_tests").join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .unwrap_or_else(|| panic!("{path} imports"))
}

fn default_artboard(file: &RuntimeFileHandle) -> RuntimeArtboardInstanceHandle {
    file.with_file(File::artboard_default)
        .expect("default artboard")
}

fn bind_default_view_model(
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
) -> Option<CoreHandle> {
    let instance = file.with_file(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    });
    if let Some(instance) = &instance {
        artboard.bind_view_model_instance(Some(instance.clone()));
    }
    instance
}

fn probe_artboard(path: &str) -> ProbeResult {
    let file = read_file(path);
    let artboard = default_artboard(&file);
    let _view_model = bind_default_view_model(&file, &artboard);
    for _ in 0..SETTLE_FRAMES {
        artboard.advance_default(FRAME_SECONDS);
    }
    let mut result = ProbeResult::default();
    let before = Artboard::layout_pass_count();
    for _ in 0..PROBE_FRAMES {
        result.keep_going_frames += usize::from(artboard.advance_default(FRAME_SECONDS));
    }
    result.layout_passes = Artboard::layout_pass_count().wrapping_sub(before);
    result
}

fn probe_scene(path: &str) -> ProbeResult {
    let file = read_file(path);
    let artboard = default_artboard(&file);
    let view_model = bind_default_view_model(&file, &artboard);
    let mut machine = artboard.default_state_machine();
    if machine.is_none() && artboard.with_artboard(|artboard| artboard.state_machine_count()) > 0 {
        machine = artboard.state_machine_at(0);
    }
    let Some(machine) = machine else {
        return probe_artboard(path);
    };
    if let Some(view_model) = view_model {
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model));
    }
    for _ in 0..SETTLE_FRAMES {
        machine.advance_and_apply(FRAME_SECONDS);
    }
    let mut result = ProbeResult::default();
    let before = Artboard::layout_pass_count();
    for _ in 0..PROBE_FRAMES {
        result.keep_going_frames += usize::from(machine.advance_and_apply(FRAME_SECONDS));
    }
    result.layout_passes = Artboard::layout_pass_count().wrapping_sub(before);
    result
}

fn first_participant_shape(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    artboard
        .with_artboard(|artboard| artboard.find_all_handles::<Shape>())
        .into_iter()
        .find(|shape| {
            shape
                .with_downcast::<Shape, _>(|shape| {
                    shape.is_participating_in_layout() && !shape.paths().is_empty()
                })
                .unwrap_or(false)
        })
        .expect("participant Shape with a path")
}

const FILES: &[&str] = &[
    "assets/layout/stack_participant.riv",
    "assets/layout/hug_participant.riv",
    "assets/layout/scroll_participant.riv",
    "assets/layout/group_participant.riv",
    "assets/layout/solo_participant.riv",
    "assets/layout/animated_participant.riv",
    "assets/layout_grid_stack.riv",
    "assets/layout/layout_paint.riv",
    "assets/layout/layout_display.riv",
    "assets/layout_text_match.riv",
    "assets/data_converter_interpolator_reset.riv",
    "assets/interpolation_zero_duration.riv",
    "assets/time_based_interpolation.riv",
    "assets/interpolate_to_end.riv",
    "assets/db_health_tracker.riv",
    "assets/data_viz_demo.riv",
    "assets/car_widgets_v01.riv",
    "assets/superbowl.riv",
];

#[test]
#[ignore = "upstream hidden [.frame_probe] report-only sweep"]
fn settled_artboards_without_a_state_machine() {
    for path in FILES {
        report("artboard", path, &probe_artboard(path));
    }
}

#[test]
#[ignore = "upstream hidden [.frame_probe] report-only sweep"]
fn settled_scenes_with_their_state_machine() {
    for path in FILES {
        report("scene", path, &probe_scene(path));
    }
}

#[test]
fn settled_layout_participants_stop_resolving_the_layout() {
    for path in &FILES[..6] {
        let artboard = probe_artboard(path);
        assert_eq!(artboard.layout_passes, 0, "{path}: artboard layout");
        assert_eq!(artboard.keep_going_frames, 0, "{path}: artboard keep-going");
        let scene = probe_scene(path);
        assert_eq!(scene.layout_passes, 0, "{path}: scene layout");
        assert_eq!(scene.keep_going_frames, 0, "{path}: scene keep-going");
    }
}

#[test]
fn folded_participant_still_remeasures_when_its_path_moves() {
    let file = read_file("assets/layout/stack_participant.riv");
    let artboard = default_artboard(&file);
    let shape = first_participant_shape(&artboard);
    assert!(CoreRegistry::set_double_handle(
        &shape,
        i32::from(TransformComponentBase::SCALE_X_PROPERTY_KEY),
        0.0
    ));
    for _ in 0..SETTLE_FRAMES {
        artboard.advance_default(FRAME_SECONDS);
    }
    let settled = Artboard::layout_pass_count();
    artboard.advance_default(FRAME_SECONDS);
    assert_eq!(Artboard::layout_pass_count(), settled);
    let path = shape
        .with_downcast::<Shape, _>(|shape| shape.paths()[0].clone())
        .expect("Shape");
    let x_key = i32::from(NodeBase::X_PROPERTY_KEY);
    let x = CoreRegistry::get_double_handle(&path, x_key).expect("path x");
    assert!(CoreRegistry::set_double_handle(&path, x_key, x + 10.0));
    artboard.advance_default(FRAME_SECONDS);
    artboard.advance_default(FRAME_SECONDS);
    assert!(Artboard::layout_pass_count() > settled);
}

#[test]
fn participant_whose_path_rebuilds_in_place_does_not_resolve() {
    let file = read_file("assets/layout/stack_participant.riv");
    let artboard = default_artboard(&file);
    let shape = first_participant_shape(&artboard);
    for _ in 0..SETTLE_FRAMES {
        artboard.advance_default(FRAME_SECONDS);
    }
    let settled = Artboard::layout_pass_count();
    for _ in 0..PROBE_FRAMES {
        let path = shape
            .with_downcast::<Shape, _>(|shape| shape.paths()[0].clone())
            .expect("Shape");
        Path::mark_path_dirty_occurrence(&path, false);
        artboard.advance_default(FRAME_SECONDS);
    }
    assert_eq!(Artboard::layout_pass_count(), settled);
}

#[test]
fn a_layout_tween_runs_without_resolving_the_layout() {
    use crate::source::{
        core::CoreType,
        generated::{
            layout::layout_component_style_base::LayoutComponentStyleBase,
            layout_component_base::LayoutComponentBase,
        },
        layout::layout_enums::{LayoutAnimationStyle, LayoutStyleInterpolation},
        layout_component::LayoutComponent,
    };
    let file = read_file("assets/layout/layout_anim_bound.riv");
    let artboard = default_artboard(&file);
    let mut container = None;
    let mut layout = None;
    for candidate in artboard.with_artboard(|a| a.find_all_handles::<LayoutComponent>()) {
        if candidate.is_type_of(Artboard::TYPE_KEY) {
            continue;
        }
        let style = candidate
            .with(|object| {
                let candidate = object.as_layout_component().unwrap();
                candidate
                    .style_handle()
                    .map(|_| candidate.animation_style())
            })
            .flatten();
        if container.is_none() && style == Some(LayoutAnimationStyle::Custom) {
            container = Some(candidate.clone());
        }
        if layout.is_none() && style == Some(LayoutAnimationStyle::Inherit) {
            layout = Some(candidate);
        }
    }
    let container = container.expect("custom animation container");
    let layout = layout.expect("inheriting layout");
    let style = container
        .with(|object| object.as_layout_component().unwrap().style_handle())
        .flatten()
        .unwrap();
    assert!(CoreRegistry::set_uint_handle(
        &style,
        i32::from(LayoutComponentStyleBase::INTERPOLATION_TYPE_PROPERTY_KEY),
        LayoutStyleInterpolation::Linear as u32
    ));
    assert!(CoreRegistry::set_double_handle(
        &style,
        i32::from(LayoutComponentStyleBase::INTERPOLATION_TIME_PROPERTY_KEY),
        1.0
    ));
    artboard.advance_default(0.0);
    let width = || {
        layout
            .with(|object| object.as_layout_component().unwrap().layout_width())
            .unwrap()
    };
    assert_eq!(width(), 100.0);
    assert!(CoreRegistry::set_double_handle(
        &layout,
        i32::from(LayoutComponentBase::WIDTH_PROPERTY_KEY),
        50.0
    ));
    artboard.advance_default(FRAME_SECONDS);
    let retargeted = Artboard::layout_pass_count();
    let mut frames = 0;
    while width() > 50.0 && frames < 2 * PROBE_FRAMES {
        artboard.advance_default(FRAME_SECONDS);
        frames += 1;
    }
    assert_eq!(width(), 50.0);
    assert!(frames > 1);
    assert_eq!(Artboard::layout_pass_count(), retargeted);
}
