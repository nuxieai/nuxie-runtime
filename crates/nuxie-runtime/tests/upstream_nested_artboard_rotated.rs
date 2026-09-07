//! All three cases from `tests/unit_tests/runtime/nested_artboard_rotated_test.cpp`
//! at upstream `e289232b776cf315863ad995ca987982e50445ab`.
//! Assertions use the actual solved slot and parent transform, not Yoga-specific
//! arithmetic: the retained Taffy adaptation must preserve the composition.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    layout_component::LayoutComponent,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    nested_artboard_layout::NestedArtboardLayout,
};
use nuxie_runtime::{Artboard, File, ImportResult, RuntimeFactoryHandle};

struct Transforms {
    parent: Mat2D,
    local: Mat2D,
    world: Mat2D,
    slot: Vec2D,
    origin: Vec2D,
}

fn fixture_transforms() -> Transforms {
    let path = std::env::var_os("RIVE_RUNTIME_DIR").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/sync/nested_artboard_rotated.riv")
        },
        |root| {
            PathBuf::from(root).join("tests/unit_tests/assets/layout/nested_artboard_rotated.riv")
        },
    );
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("nested_artboard_rotated.riv imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    // artboard_named, not artboard_named_source: nested children mount only
    // on the ArtboardInstance returned by upstream file->artboardNamed.
    let artboard = file
        .with_file(|file| file.artboard_named("NestedRotatedHost"))
        .expect("NestedRotatedHost instance");
    artboard.advance_default(0.0);
    let row = artboard
        .with_artboard(|artboard| {
            artboard
                .find_all_handles::<LayoutComponent>()
                .into_iter()
                .find(|layout| {
                    layout.with_downcast::<Artboard, _>(|_| ()).is_none()
                        && layout
                            .with(|owner| {
                                owner
                                    .as_layout_component()
                                    .is_some_and(|layout| layout.rotation() != 0.0)
                            })
                            .unwrap_or(false)
                })
        })
        .expect("rotated layout row");
    let nested =
        artboard.with_artboard(|artboard| artboard.find_all_handles::<NestedArtboardLayout>());
    assert_eq!(nested.len(), 1);
    let host = &nested[0];
    let instance = host
        .with_downcast::<NestedArtboardLayout, _>(|host| host.base.base.artboard_instance_handle(0))
        .flatten()
        .expect("mounted nested artboard instance");
    let (slot, origin) = instance.with_artboard(|instance| {
        (
            Vec2D::new(instance.layout_x(), instance.layout_y()),
            instance.origin(),
        )
    });
    let parent = row
        .with(|owner| {
            *owner
                .as_transform_component()
                .expect("row transform")
                .world_transform()
        })
        .expect("live row");
    let (local, world) = host
        .with(|owner| {
            let transform = owner
                .as_transform_component()
                .expect("nested host transform");
            (*transform.transform(), *transform.world_transform())
        })
        .expect("live nested host");
    Transforms {
        parent,
        local,
        world,
        slot,
        origin,
    }
}

// Catch Approx(expected).margin(1e-4) keeps its default relative epsilon in
// addition to the authored absolute margin, and calculates in double precision.
fn approx(actual: f32, expected: f32) -> bool {
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    let within = |margin: f64| expected + margin >= actual && actual + margin >= expected;
    let scale = if expected.is_infinite() { 0.0 } else { expected.abs() };
    within(1e-4) || within(f64::from(f32::EPSILON * 100.0) * scale)
}

#[test]
fn nested_artboard_slot_composes_inside_rotated_parent() {
    let t = fixture_transforms();
    assert_ne!(
        t.slot.x, 0.0,
        "sibling must push the actual solved slot off the origin"
    );
    let expected =
        t.parent * Mat2D::from_translation(t.slot) * t.local * Mat2D::from_translation(-t.origin);
    assert!(
        approx(t.world[4], expected[4]),
        "world x: {} != {}",
        t.world[4],
        expected[4]
    );
    assert!(
        approx(t.world[5], expected[5]),
        "world y: {} != {}",
        t.world[5],
        expected[5]
    );
}

#[test]
fn rotated_parent_does_not_leave_slot_axis_aligned() {
    let t = fixture_transforms();
    let legacy = Mat2D::from_translation(t.slot - t.origin) * t.parent * t.local;
    assert!(
        !approx(t.world[4], legacy[4]),
        "world x must not use the legacy composition"
    );
}

#[test]
fn rotated_parent_basis_reaches_nested_artboard() {
    let t = fixture_transforms();
    for i in 0..4 {
        assert!(
            approx(t.world[i], t.parent[i]),
            "basis {i}: {} != {}",
            t.world[i],
            t.parent[i]
        );
    }
}
