//! All six cases from upstream nested_artboard_constrained_test.cpp at
//! a6b6723ba291f6f00888c09a1f975c6c23936095. Placement is read from the mounted
//! instance, keeping the assertions independent of the approved layout solver.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    layout_component::LayoutComponent,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    nested_artboard_layout::NestedArtboardLayout,
    node::Node,
    shapes::shape::Shape,
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
use std::path::PathBuf;

fn load(asset: &str, name: &str) -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle) {
    let path = std::env::var_os("RIVE_RUNTIME_DIR").map_or_else(
        || {
            PathBuf::from(
                option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
            )
            .join("../../fixtures/sync")
            .join(asset)
        },
        |root| {
            PathBuf::from(root)
                .join("tests/unit_tests/assets/layout")
                .join(asset)
        },
    );
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file =
        File::import(&bytes, retained, Some(&mut result), None, None).expect("fixture imports");
    assert_eq!(result, ImportResult::Success);
    // Use an instance: source artboards do not mount their nested artboards.
    let artboard = file
        .with_file(|file| file.artboard_named(name))
        .expect("named artboard instance");
    artboard.advance_default(0.0);
    (file, artboard)
}

fn nested(artboard: &RuntimeArtboardInstanceHandle) -> (CoreHandle, RuntimeArtboardInstanceHandle) {
    let nested =
        artboard.with_artboard(|artboard| artboard.find_all_handles::<NestedArtboardLayout>());
    assert_eq!(nested.len(), 1);
    let host = nested[0].clone();
    let instance = host
        .with_downcast::<NestedArtboardLayout, _>(|host| host.base.base.artboard_instance_handle(0))
        .flatten()
        .expect("mounted nested artboard instance");
    (host, instance)
}

fn world(owner: &CoreHandle) -> Mat2D {
    owner
        .with(|owner| {
            *owner
                .as_transform_component()
                .expect("transform")
                .world_transform()
        })
        .expect("live owner")
}

struct Scene {
    parent: Mat2D,
    world: Mat2D,
    constrained_to: Vec2D,
    base: Vec2D,
}

fn constrained_scene() -> Scene {
    let (_file, artboard) = load("nested_artboard_constrained.riv", "NestedConstrainedHost");
    let row = artboard
        .with_artboard(|artboard| {
            artboard
                .find_all_handles::<LayoutComponent>()
                .into_iter()
                .filter(|layout| {
                    layout.with_downcast::<Artboard, _>(|_| ()).is_none()
                        && layout
                            .with(|owner| {
                                owner
                                    .as_layout_component()
                                    .is_some_and(|layout| layout.rotation() != 0.0)
                            })
                            .unwrap_or(false)
                })
                .last()
        })
        .expect("rotated layout row");
    let (host, instance) = nested(&artboard);
    let target = artboard
        .with_artboard(|artboard| artboard.find_handle::<Node>("Target"))
        .expect("Target node");
    let target = world(&target);
    let (slot, origin) = instance.with_artboard(|instance| {
        (
            Vec2D::new(instance.layout_x(), instance.layout_y()),
            instance.origin(),
        )
    });
    assert_ne!(slot.x, 0.0);
    assert_ne!(origin.x, 0.0);
    Scene {
        parent: world(&row),
        world: world(&host),
        constrained_to: Vec2D::new(target[4], target[5]),
        base: slot - origin,
    }
}

// Catch Approx(expected).margin(1e-4), including its default relative epsilon
// and double-precision comparisons; no new tolerance is introduced here.
fn approx(actual: f32, expected: f32) -> bool {
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    let within = |margin: f64| expected + margin >= actual && actual + margin >= expected;
    let scale = if expected.is_infinite() {
        0.0
    } else {
        expected.abs()
    };
    within(1e-4) || within(f64::from(f32::EPSILON * 100.0) * scale)
}

#[test]
fn a_constrained_nested_artboard_keeps_its_layout_placement() {
    let s = constrained_scene();
    let p = s.parent;
    let expected = s.constrained_to
        + Vec2D::new(
            p[0] * s.base.x + p[2] * s.base.y,
            p[1] * s.base.x + p[3] * s.base.y,
        );
    assert!(approx(s.world[4], expected.x));
    assert!(approx(s.world[5], expected.y));
}

#[test]
fn a_constraint_does_not_erase_the_layout_placement() {
    let s = constrained_scene();
    let off = (s.world[4] - s.constrained_to.x).abs() + (s.world[5] - s.constrained_to.y).abs();
    assert!(off > 1e-4);
}

#[test]
fn a_reapplied_placement_still_turns_with_the_parent() {
    let s = constrained_scene();
    let legacy = s.constrained_to + s.base;
    assert!(!approx(s.world[4], legacy.x));
}

fn follow_scene() -> (Mat2D, Vec2D, Vec2D) {
    let (_file, artboard) = load("nested_artboard_followpath.riv", "NestedFollowPathHost");
    let (host, instance) = nested(&artboard);
    let target = artboard
        .with_artboard(|artboard| artboard.find_handle::<Shape>("FollowTarget"))
        .expect("FollowTarget shape");
    let target = world(&target);
    let base = instance.with_artboard(|instance| {
        let origin = instance.origin();
        assert_ne!(origin.x, 0.0);
        assert_ne!(origin.y, 0.0);
        Vec2D::new(instance.layout_x(), instance.layout_y()) - origin
    });
    (world(&host), Vec2D::new(target[4], target[5]), base)
}

#[test]
fn a_follow_path_constraint_keeps_the_layout_placement() {
    let (world, landed, base) = follow_scene();
    assert!(approx(world[4], landed.x + base.x));
    assert!(approx(world[5], landed.y + base.y));
}

#[test]
fn a_follow_path_constraint_does_not_erase_the_placement() {
    let (world, landed, _) = follow_scene();
    let off = (world[4] - landed.x).abs() + (world[5] - landed.y).abs();
    assert!(off > 1e-4);
}

#[test]
fn the_placement_uses_an_overridden_nested_artboard_origin() {
    let (_file, artboard) = load("nested_artboard_origin_override.riv", "OriginOverrideHost");
    let (host, instance) = nested(&artboard);
    let target = artboard
        .with_artboard(|artboard| artboard.find_handle::<Node>("Target"))
        .expect("Target node");
    let target = world(&target);
    let landed = Vec2D::new(target[4], target[5]);
    let (slot, origin, width, height) = instance.with_artboard(|instance| {
        (
            Vec2D::new(instance.layout_x(), instance.layout_y()),
            instance.origin(),
            instance.layout_width(),
            instance.layout_height(),
        )
    });
    assert!(approx(origin.x, -0.25 * width));
    assert!(approx(origin.y, -0.75 * height));
    assert!(width > 0.0);
    assert!(height > 0.0);
    let base = slot - origin;
    let world = world(&host);
    assert!(approx(world[4], landed.x + base.x));
    assert!(approx(world[5], landed.y + base.y));
    let from_source = slot - Vec2D::new(-0.5 * width, -0.5 * height);
    assert!(!approx(world[5], landed.y + from_source.y));
}
