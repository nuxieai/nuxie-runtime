//! Seven additions to upstream layout_grid_test.cpp / layout_stack_test.cpp
//! at 8e8492f8312c67ac54558adce2f0798baabcdce3.

use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::source::{
    layout_component::LayoutComponent, math::aabb::Aabb,
    nested_artboard_layout::NestedArtboardLayout,
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
use std::path::PathBuf;

struct Fixture {
    artboard: RuntimeArtboardInstanceHandle,
    _file: RuntimeFileHandle,
    _factory: PersistentFactory<NullFactory>,
}

impl Fixture {
    fn new(asset: &str, name: &str) -> Self {
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
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
        let mut factory = PersistentFactory::new(NullFactory);
        let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
        let file = File::import(&bytes, retained, None, None, None).expect("import layout fixture");
        let artboard = file
            .with_file(|file| file.artboard_named(name))
            .expect("named artboard instance");
        artboard.advance_default(0.0);
        Self {
            artboard,
            _file: file,
            _factory: factory,
        }
    }

    fn layouts(&self) -> Vec<CoreHandle> {
        self.artboard
            .with_artboard(|artboard| artboard.find_all_handles::<LayoutComponent>())
    }

    fn named_layout(&self, name: &str) -> CoreHandle {
        self.layouts()
            .into_iter()
            .find(|owner| {
                owner.with_downcast::<Artboard, _>(|_| ()).is_none()
                    && owner
                        .with(|owner| {
                            owner
                                .as_component()
                                .is_some_and(|component| component.name() == name)
                        })
                        .unwrap_or(false)
            })
            .unwrap_or_else(|| panic!("missing non-artboard layout {name}"))
    }

    fn nested_bounds(&self) -> Aabb {
        let nested = self
            .artboard
            .with_artboard(|artboard| artboard.find_all_handles::<NestedArtboardLayout>());
        assert_eq!(nested.len(), 1);
        nested[0]
            .with_downcast::<NestedArtboardLayout, _>(NestedArtboardLayout::layout_bounds)
            .expect("nested artboard layout")
    }
}

fn width(owner: &CoreHandle) -> f32 {
    owner
        .with(|owner| owner.as_layout_component().expect("layout").layout_width())
        .expect("live layout")
}

#[test]
fn an_intrinsic_track_floors_at_min_content_not_max_content() {
    let fixture = Fixture::new("grid_min_content.riv", "ReflowGrid");
    let container = fixture.named_layout("Container");
    let child = fixture.named_layout("Child");
    assert_eq!(width(&container), 400.0);
    assert_eq!(width(&child), 400.0);
    let mut chips: Vec<f32> = fixture
        .layouts()
        .into_iter()
        .filter_map(|owner| {
            owner
                .with(|owner| {
                    if owner
                        .as_component()
                        .and_then(|component| component.parent_handle())
                        .as_ref()
                        == Some(&child)
                    {
                        Some(owner.as_layout_component().expect("chip layout").layout_y())
                    } else {
                        None
                    }
                })
                .flatten()
        })
        .collect();
    assert_eq!(chips.len(), 4);
    chips.sort_by(|a, b| a.partial_cmp(b).expect("finite chip layout"));
    assert_eq!(chips[0], 0.0);
    assert!(chips[3] > 0.0);
}

#[test]
fn content_that_cannot_shrink_still_grows_an_intrinsic_track() {
    let fixture = Fixture::new("grid_min_content.riv", "RigidGrid");
    assert_eq!(width(&fixture.named_layout("Container")), 400.0);
    assert_eq!(width(&fixture.named_layout("Child")), 600.0);
}

#[test]
fn a_wrapping_child_gives_an_intrinsic_track_nothing_to_floor_at() {
    let fixture = Fixture::new("grid_min_content.riv", "WrapRigidGrid");
    assert_eq!(width(&fixture.named_layout("Child")), 400.0);
}

#[test]
fn an_authored_zero_minimum_removes_the_content_floor() {
    let fixture = Fixture::new("grid_min_content.riv", "MinMaxGrid");
    assert_eq!(width(&fixture.named_layout("Child")), 400.0);
}

fn nested_bounds(name: &str) -> Aabb {
    Fixture::new("nested_artboard_fill.riv", name).nested_bounds()
}

#[test]
fn a_fill_nested_artboard_fills_a_stack_and_a_grid_alike() {
    let in_grid = nested_bounds("NestedInGrid");
    let in_stack = nested_bounds("NestedInStack");
    assert_eq!(in_grid.width(), 200.0);
    assert_eq!(in_grid.height(), 40.0);
    assert_eq!(in_stack.width(), in_grid.width());
    assert_eq!(in_stack.height(), in_grid.height());
    assert_eq!(in_stack.left(), 0.0);
    assert_eq!(in_stack.top(), 0.0);
}

#[test]
fn a_stack_cell_floors_at_min_content_too() {
    let fixture = Fixture::new("grid_min_content.riv", "ReflowStack");
    assert_eq!(width(&fixture.named_layout("Container")), 400.0);
    assert_eq!(width(&fixture.named_layout("Child")), 400.0);
}

#[test]
fn a_hug_nested_artboard_hugs_in_a_stack_and_a_grid_alike() {
    let in_stack = nested_bounds("HugInStack");
    let in_grid = nested_bounds("HugInGrid");
    assert_eq!(in_stack.width(), 120.0);
    assert_eq!(in_stack.height(), 40.0);
    assert_eq!(in_grid.width(), in_stack.width());
    assert_eq!(in_grid.height(), in_stack.height());
}
