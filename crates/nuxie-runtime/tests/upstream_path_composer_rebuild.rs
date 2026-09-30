//! Complete path_composer_rebuild_test.cpp at upstream f01e9e26.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::RuntimeArtboardInstanceHandle,
    generated::{core_registry::CoreRegistry, node_base::NodeBase},
    math::{mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
    shapes::{
        paint::shape_paint::ShapePaintPathKind, path::Path, path_flags::PathFlags, shape::Shape,
    },
};
use nuxie_runtime::{Artboard, CoreHandle, File, RuntimeFactoryHandle, RuntimeFileHandle};
use std::path::PathBuf;

const FILES: &[&str] = &[
    "car_widgets_v01.riv",
    "zombie_skins.riv",
    "echo_show_demo.riv",
    "jellyfish_test.riv",
    "trim_path.riv",
    "fill_trim_path.riv",
    "follow_path.riv",
    "follow_path_shapes.riv",
    "follow_path_solos.riv",
    "solo_test.riv",
    "clip_tests.riv",
    "clipping_and_draw_order.riv",
];

fn load(name: &str) -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(name),
    )
    .unwrap_or_else(|error| panic!("{name}: {error}"));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let source = file.with_file(File::artboard).unwrap();
    let artboard = Artboard::instance_from_handle(&source).unwrap();
    (file, artboard)
}

fn shapes(artboard: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
    artboard.with_artboard(|board| board.find_all_handles::<Shape>())
}

fn paths(shape: &CoreHandle) -> Vec<CoreHandle> {
    shape.with_downcast::<Shape, _>(Shape::paths).unwrap()
}

fn local_path(shape: &CoreHandle) -> RawPath {
    shape
        .with_downcast::<Shape, _>(|shape| {
            shape.with_path_mut(ShapePaintPathKind::Local, |path| path.raw_path().clone())
        })
        .unwrap()
}

fn force_rebuild(artboard: &RuntimeArtboardInstanceHandle) {
    for shape in shapes(artboard) {
        for path in paths(&shape) {
            Path::mark_path_dirty_occurrence(&path, true);
        }
    }
    artboard.advance_default(0.0);
}

fn cpp_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

fn allowed_delta(shape: &CoreHandle) -> f64 {
    let world = shape
        .with_downcast::<Shape, _>(|shape| *shape.world_transform())
        .unwrap();
    let mut scale = cpp_max(
        cpp_max(1.0, f64::from(world[4].abs())),
        f64::from(world[5].abs()),
    );
    for path in paths(shape) {
        let world = path
            .with(|object| {
                if let Some(points) = object.as_points_path() {
                    *points.path_transform()
                } else {
                    object.as_path().unwrap().path_transform()
                }
            })
            .unwrap();
        scale = cpp_max(
            cpp_max(scale, f64::from(world[4].abs())),
            f64::from(world[5].abs()),
        );
    }
    16.0 * f64::from(f32::EPSILON) * scale
}

fn set_world(artboard: &RuntimeArtboardInstanceHandle, world: Mat2D) {
    artboard.with_artboard_mut(|board| {
        *board.mutable_world_transform() = world;
        board.mark_world_transform_dirty();
    });
}

fn advance_frame(
    artboard: &RuntimeArtboardInstanceHandle,
    machine: Option<&RuntimeStateMachineInstanceHandle>,
    i: i32,
) {
    set_world(
        artboard,
        Mat2D::from_rotation(i as f32 * 0.013)
            * Mat2D::from_translate((i % 23) as f32 * 1.7, (i % 7) as f32),
    );
    if let Some(machine) = machine {
        machine.with_instance_mut(|machine| {
            machine.pointer_move(
                Vec2D::new((i * 13 % 500) as f32, (i * 29 % 500) as f32),
                0.0,
                0,
            );
        });
        machine.advance_and_apply(1.0 / 60.0);
    } else {
        artboard.advance_default(1.0 / 60.0);
    }
}

fn is_comparable(shape: &CoreHandle) -> bool {
    shape
        .with_downcast::<Shape, _>(|shape| {
            shape.is_flagged(PathFlags::LOCAL)
                && !shape.can_defer_path_update()
                && !shape.is_flagged(PathFlags::FOLLOW_PATH)
        })
        .unwrap()
}

fn delta(a: Vec2D, b: Vec2D) -> f64 {
    cpp_max(
        (f64::from(a.x) - f64::from(b.x)).abs(),
        (f64::from(a.y) - f64::from(b.y)).abs(),
    )
}

#[test]
fn a_retained_local_path_equals_a_fresh_rebuild() {
    for name in FILES {
        let (_file, artboard) = load(name);
        let machine = artboard.default_state_machine_handle();
        artboard.advance_default(0.0);
        let (mut verb_mismatches, mut violations) = (0, 0);
        let (mut worst_delta, mut worst_allowed) = (0.0, 0.0);
        for i in 0..120 {
            advance_frame(&artboard, machine.as_ref(), i);
            let held: Vec<_> = shapes(&artboard)
                .into_iter()
                .filter(is_comparable)
                .map(|shape| {
                    let path = local_path(&shape);
                    (shape, path)
                })
                .collect();
            force_rebuild(&artboard);
            for (shape, kept) in held {
                let fresh = local_path(&shape);
                if kept.verbs().len() != fresh.verbs().len()
                    || kept.points().len() != fresh.points().len()
                {
                    verb_mismatches += 1;
                    continue;
                }
                for (a, b) in kept.verbs().iter().zip(fresh.verbs()) {
                    if a != b {
                        verb_mismatches += 1;
                        break;
                    }
                }
                let allowed = allowed_delta(&shape);
                for (&a, &b) in kept.points().iter().zip(fresh.points()) {
                    let delta = delta(a, b);
                    if delta > allowed {
                        violations += 1;
                    }
                    if delta > worst_delta {
                        worst_delta = delta;
                        worst_allowed = allowed;
                    }
                }
            }
        }
        assert_eq!(
            verb_mismatches, 0,
            "file {name}, worst delta {worst_delta}, allowance {worst_allowed}"
        );
        assert_eq!(
            violations, 0,
            "file {name}, worst delta {worst_delta}, allowance {worst_allowed}"
        );
    }
}

#[test]
fn a_rigid_move_retains_the_local_path_it_already_built() {
    let (_file, artboard) = load("car_widgets_v01.riv");
    artboard.advance_default(0.0);
    let moved = shapes(&artboard)
        .into_iter()
        .find(|shape| is_comparable(shape) && !local_path(shape).empty())
        .expect("comparable shape");
    let before = local_path(&moved).points().to_vec();
    assert!(!before.is_empty());
    set_world(
        &artboard,
        Mat2D::from_rotation(std::f32::consts::PI / 3.0)
            * Mat2D::from_scale(2.0, 0.5)
            * Mat2D::from_translate(137.0, -42.0),
    );
    artboard.advance_default(0.0);
    let after = local_path(&moved);
    assert_eq!(after.points().len(), before.len());
    for (after, before) in after.points().iter().zip(before) {
        assert_eq!(after.x, before.x);
        assert_eq!(after.y, before.y);
    }
}

#[test]
fn a_path_moving_inside_its_shape_rebuilds_the_local_path() {
    let (_file, artboard) = load("car_widgets_v01.riv");
    artboard.advance_default(0.0);
    let shape = shapes(&artboard)
        .into_iter()
        .find(|shape| {
            is_comparable(shape) && !paths(shape).is_empty() && !local_path(shape).empty()
        })
        .expect("comparable shape with a path");
    let path = paths(&shape)[0].clone();
    let before = local_path(&shape).points().to_vec();
    let version = path
        .with(|object| object.as_path().unwrap().geometry_version())
        .unwrap();
    let x = i32::from(NodeBase::X_PROPERTY_KEY);
    let next_x = CoreRegistry::get_double_handle(&path, x).unwrap() + 10.0;
    assert!(CoreRegistry::set_double_handle(&path, x, next_x));
    path.with_mut(|object| {
        object
            .as_transform_component_mut()
            .unwrap()
            .mark_transform_dirty()
    })
    .unwrap();
    artboard.advance_default(0.0);
    assert_eq!(
        path.with(|object| object.as_path().unwrap().geometry_version())
            .unwrap(),
        version
    );
    let moved = local_path(&shape).points().to_vec();
    assert_eq!(moved.len(), before.len());
    assert!(moved.iter().zip(&before).any(|(a, b)| a.x != b.x));
    let held = moved;
    force_rebuild(&artboard);
    let fresh = local_path(&shape);
    assert_eq!(fresh.points().len(), held.len());
    let allowed = allowed_delta(&shape);
    let violations = held
        .iter()
        .zip(fresh.points())
        .filter(|(a, b)| delta(**a, **b) > allowed)
        .count();
    assert_eq!(violations, 0);
}
