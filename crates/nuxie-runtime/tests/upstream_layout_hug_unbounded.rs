//! Complete layout_hug_unbounded_test.cpp translation at f0ec9921.
use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags, generated::core_registry::CoreField, layout::layout_node_provider,
    layout_component::LayoutComponent, math::aabb::Aabb, shapes::shape::Shape,
};
use nuxie_runtime::{Artboard, CoreHandle, File, RuntimeFactoryHandle, RuntimeFileHandle};
use std::path::PathBuf;

struct Fixture {
    file: RuntimeFileHandle,
    _factory: PersistentFactory<NullFactory>,
}

impl Fixture {
    fn new() -> Self {
        let path = std::env::var_os("RIVE_RUNTIME_DIR").map_or_else(
            || {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../fixtures/sync/layout/hug_unbounded.riv")
            },
            |root| PathBuf::from(root).join("tests/unit_tests/assets/layout/hug_unbounded.riv"),
        );
        let bytes = std::fs::read(&path).expect("pinned hug_unbounded fixture");
        let mut factory = PersistentFactory::new(NullFactory);
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("import hug_unbounded fixture");
        Self {
            file,
            _factory: factory,
        }
    }

    fn advanced(&self, name: &str) -> CoreHandle {
        let artboard = self
            .file
            .with_file(|file| file.artboard_named_source(name))
            .expect("named source artboard");
        Artboard::advance_handle(
            &artboard,
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
        artboard
    }
}

fn subject_bounds(artboard: &CoreHandle) -> Aabb {
    let subject = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<LayoutComponent>("Subject"))
        .flatten()
        .expect("Subject layout");
    subject
        .with(|subject| {
            subject
                .as_layout_component()
                .expect("layout component")
                .layout_bounds()
        })
        .expect("live Subject")
}

fn participant_bounds(artboard: &CoreHandle) -> Aabb {
    let shape = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<Shape>("Subject"))
        .flatten()
        .expect("Subject shape");
    let provider = layout_node_provider::from_component(&shape).expect("shape layout provider");
    provider
        .with_mut(|provider| {
            provider
                .as_layout_node_provider_mut()
                .expect("layout provider")
                .layout_bounds()
        })
        .expect("live provider")
}

#[test]
fn a_hug_leaf_is_bounded_by_the_space_available_to_it() {
    let fixture = Fixture::new();
    let bounded = subject_bounds(&fixture.advanced("LeafBounded"));
    assert_eq!(bounded.width(), 200.0);
    assert_eq!(bounded.height(), 120.0);
    let unbounded = subject_bounds(&fixture.advanced("LeafUnbounded"));
    assert_eq!(unbounded.width(), 260.0);
    assert_eq!(unbounded.height(), 180.0);
}

#[test]
fn a_hug_participant_is_bounded_the_same_way_as_a_hug_leaf() {
    let fixture = Fixture::new();
    let bounded = participant_bounds(&fixture.advanced("ParticipantBounded"));
    assert_eq!(bounded.width(), 200.0);
    assert_eq!(bounded.height(), 120.0);
    let unbounded = participant_bounds(&fixture.advanced("ParticipantUnbounded"));
    assert_eq!(unbounded.width(), 260.0);
    assert_eq!(unbounded.height(), 180.0);
}

#[test]
fn max_sizing_still_bounds_an_unbounded_hug() {
    let fixture = Fixture::new();
    let bounds = subject_bounds(&fixture.advanced("UnboundedMaxed"));
    assert_eq!(bounds.width(), 180.0);
    assert_eq!(bounds.height(), 150.0);
}

#[test]
fn a_fixed_axis_survives_hug_unbounded() {
    let fixture = Fixture::new();
    let bounds = subject_bounds(&fixture.advanced("UnboundedFixedHeight"));
    assert_eq!(bounds.width(), 260.0);
    assert_eq!(bounds.height(), 40.0);
}

// Source-review regression: CoreRegistry boolean writes must dispatch each
// derived owner's hugUnboundedChanged hook, not the generated default no-op.
#[test]
fn registry_hug_unbounded_changes_dirty_leaf_and_participant_layout() {
    for (name, participant) in [("LeafUnbounded", false), ("ParticipantBounded", true)] {
        let fixture = Fixture::new();
        let artboard = fixture.advanced(name);
        let target = if participant {
            let shape = artboard
                .with_downcast::<Artboard, _>(|a| a.find_handle::<Shape>("Subject"))
                .flatten()
                .unwrap();
            layout_node_provider::from_component(&shape).unwrap()
        } else {
            let subject = artboard
                .with_downcast::<Artboard, _>(|a| a.find_handle::<LayoutComponent>("Subject"))
                .flatten()
                .unwrap();
            subject
                .with(|o| o.as_layout_component().unwrap().style_handle())
                .flatten()
                .unwrap()
        };
        let initial = if participant {
            participant_bounds(&artboard)
        } else {
            subject_bounds(&artboard)
        };
        assert_eq!(initial.width(), if participant { 200.0 } else { 260.0 });
        assert_eq!(initial.height(), if participant { 120.0 } else { 180.0 });
        // Upstream ParametricPath::controlSize overwrites a leaf's content
        // dimensions after a bounded solve. Re-enabling unbounded measurement
        // cannot restore them. Participant shapes retain their intrinsic path.
        let changes = if participant {
            [(true, 260.0, 180.0), (false, 200.0, 120.0)]
        } else {
            [(false, 200.0, 120.0), (true, 200.0, 120.0)]
        };
        for (value, width, height) in changes {
            target
                .with_mut(|o| o.set_bool(CoreField::LayoutSizingStyleHugUnbounded, value))
                .unwrap();
            Artboard::advance_handle(
                &artboard,
                0.0,
                AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
            );
            let bounds = if participant {
                participant_bounds(&artboard)
            } else {
                subject_bounds(&artboard)
            };
            assert_eq!(bounds.width(), width, "{name}, hugUnbounded={value}");
            assert_eq!(bounds.height(), height, "{name}, hugUnbounded={value}");
        }
    }
}
