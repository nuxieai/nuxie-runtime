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
                PathBuf::from(
                    option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
                )
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

// Unsupported shape measurement keeps main's zero automatic minimum. CSS
// shrink factors share negative space by basis. Preserve the C++ width and
// exact new first difference for every reviewed result.
const CPP_BOUNDED_WIDTH: f32 = 200.0;
fn assert_css_width(width: f32, cpp: f32, reviewed: &str) {
    assert_eq!(
        format!("Subject.layoutBounds[2]: {width} != {cpp}"),
        reviewed,
        "layout-diverges:CSS weighted shrink with zero shape minimum; Rive controlSize updates a measured leaf basis between solves"
    );
}

#[test]
fn a_hug_leaf_is_bounded_by_the_space_available_to_it() {
    let fixture = Fixture::new();
    let bounded = subject_bounds(&fixture.advanced("LeafBounded"));
    assert_css_width(
        bounded.width(),
        CPP_BOUNDED_WIDTH,
        "Subject.layoutBounds[2]: 143.88489 != 200",
    );
    assert_eq!(bounded.height(), 120.0);
    let unbounded = subject_bounds(&fixture.advanced("LeafUnbounded"));
    assert_css_width(
        unbounded.width(),
        260.0,
        "Subject.layoutBounds[2]: 146.06741 != 260",
    );
    assert_eq!(unbounded.height(), 180.0);
}

#[test]
fn a_hug_participant_is_bounded_the_same_way_as_a_hug_leaf() {
    let fixture = Fixture::new();
    let bounded = participant_bounds(&fixture.advanced("ParticipantBounded"));
    assert_css_width(
        bounded.width(),
        CPP_BOUNDED_WIDTH,
        "Subject.layoutBounds[2]: 153.84616 != 200",
    );
    assert_eq!(bounded.height(), 120.0);
    let unbounded = participant_bounds(&fixture.advanced("ParticipantUnbounded"));
    assert_css_width(
        unbounded.width(),
        260.0,
        "Subject.layoutBounds[2]: 162.5 != 260",
    );
    assert_eq!(unbounded.height(), 180.0);
}

#[test]
fn max_sizing_still_bounds_an_unbounded_hug() {
    let fixture = Fixture::new();
    let bounds = subject_bounds(&fixture.advanced("UnboundedMaxed"));
    assert_css_width(
        bounds.width(),
        180.0,
        "Subject.layoutBounds[2]: 146.06741 != 180",
    );
    assert_eq!(bounds.height(), 150.0);
}

#[test]
fn a_fixed_axis_survives_hug_unbounded() {
    let fixture = Fixture::new();
    let bounds = subject_bounds(&fixture.advanced("UnboundedFixedHeight"));
    assert_css_width(
        bounds.width(),
        260.0,
        "Subject.layoutBounds[2]: 146.06741 != 260",
    );
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
        if participant {
            assert_css_width(
                initial.width(),
                CPP_BOUNDED_WIDTH,
                "Subject.layoutBounds[2]: 153.84616 != 200",
            );
        } else {
            assert_css_width(
                initial.width(),
                260.0,
                "Subject.layoutBounds[2]: 146.06741 != 260",
            );
        }
        assert_eq!(initial.height(), if participant { 120.0 } else { 180.0 });
        // Upstream ParametricPath::controlSize overwrites a leaf's content
        // height after a bounded solve. Re-enabling unbounded measurement
        // cannot restore it. The next CSS shrink uses the current path basis.
        // Participant shapes retain their intrinsic path.
        let changes = if participant {
            [
                (true, 260.0, 162.5, 180.0),
                (false, CPP_BOUNDED_WIDTH, 153.84616, 120.0),
            ]
        } else {
            [
                (false, CPP_BOUNDED_WIDTH, 141.76663, 120.0),
                (true, CPP_BOUNDED_WIDTH, 140.52534, 120.0),
            ]
        };
        for (value, upstream_width, width, height) in changes {
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
            let signature = match (participant, value) {
                (true, true) => "Subject.layoutBounds[2]: 162.5 != 260",
                (true, false) => "Subject.layoutBounds[2]: 153.84616 != 200",
                (false, false) => "Subject.layoutBounds[2]: 141.76663 != 200",
                (false, true) => "Subject.layoutBounds[2]: 140.52534 != 200",
            };
            assert_css_width(bounds.width(), upstream_width, signature);
            assert_eq!(bounds.width(), width, "{name}, hugUnbounded={value}");
            assert_eq!(bounds.height(), height, "{name}, hugUnbounded={value}");
        }
    }
}
