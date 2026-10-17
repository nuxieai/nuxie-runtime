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

// UNIV-4844: a shape is non-reflowing content. Its natural size from
// authored geometry is its CSS min-content, so a bounded Hug (fit-content)
// cannot go below 260 even though the row offers 200. Rive's leaf measure
// clamps the shape to the offer instead. Each reviewed width keeps its C++
// value; every other field is the exact C++ value.
const CPP_BOUNDED: (f32, f32) = (200.0, 120.0);
const NATURAL_MINIMUM: &str = "layout-diverges:the shape's natural 260 width is its CSS min-content, so fit-content keeps 260 where Rive's leaf measure clamps to the 200 offer";

fn assert_reviewed(observed: &[(&str, f32, f32)], reviewed: &[&str]) {
    let differences = observed
        .iter()
        .filter(|(_, css, cpp)| css != cpp)
        .map(|(field, css, cpp)| format!("{field}: {css} != {cpp}"))
        .collect::<Vec<_>>();
    assert_eq!(
        differences, reviewed,
        "{NATURAL_MINIMUM}; observed {observed:?}"
    );
}

#[test]
fn a_hug_leaf_is_bounded_by_the_space_available_to_it() {
    let fixture = Fixture::new();
    let bounded = subject_bounds(&fixture.advanced("LeafBounded"));
    let unbounded = subject_bounds(&fixture.advanced("LeafUnbounded"));
    assert_reviewed(
        &[
            (
                "LeafBounded.layoutBounds[2]",
                bounded.width(),
                CPP_BOUNDED.0,
            ),
            (
                "LeafBounded.layoutBounds[3]",
                bounded.height(),
                CPP_BOUNDED.1,
            ),
            ("LeafUnbounded.layoutBounds[2]", unbounded.width(), 260.0),
            ("LeafUnbounded.layoutBounds[3]", unbounded.height(), 180.0),
        ],
        &["LeafBounded.layoutBounds[2]: 260 != 200"],
    );
}

#[test]
fn a_hug_participant_is_bounded_the_same_way_as_a_hug_leaf() {
    let fixture = Fixture::new();
    let bounded = participant_bounds(&fixture.advanced("ParticipantBounded"));
    let unbounded = participant_bounds(&fixture.advanced("ParticipantUnbounded"));
    assert_reviewed(
        &[
            (
                "ParticipantBounded.layoutBounds[2]",
                bounded.width(),
                CPP_BOUNDED.0,
            ),
            (
                "ParticipantBounded.layoutBounds[3]",
                bounded.height(),
                CPP_BOUNDED.1,
            ),
            (
                "ParticipantUnbounded.layoutBounds[2]",
                unbounded.width(),
                260.0,
            ),
            (
                "ParticipantUnbounded.layoutBounds[3]",
                unbounded.height(),
                180.0,
            ),
        ],
        &["ParticipantBounded.layoutBounds[2]: 260 != 200"],
    );
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
        let bounds = |artboard: &CoreHandle| {
            if participant {
                participant_bounds(artboard)
            } else {
                subject_bounds(artboard)
            }
        };
        let initial = bounds(&artboard);
        let mut observed = vec![
            (
                format!("{name}.initial[2]"),
                initial.width(),
                if participant { 200.0 } else { 260.0 },
            ),
            (
                format!("{name}.initial[3]"),
                initial.height(),
                if participant { 120.0 } else { 180.0 },
            ),
        ];
        // C++ values: upstream ParametricPath::controlSize overwrites a leaf's
        // path after a bounded solve, so re-enabling unbounded measurement
        // cannot restore 260x180 there. The CSS minimum reads authored
        // geometry, never the written-back size, so Rust restores it.
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
            let solved = bounds(&artboard);
            observed.push((
                format!("{name}.hugUnbounded={value}[2]"),
                solved.width(),
                width,
            ));
            observed.push((
                format!("{name}.hugUnbounded={value}[3]"),
                solved.height(),
                height,
            ));
        }
        let observed = observed
            .iter()
            .map(|(field, css, cpp)| (field.as_str(), *css, *cpp))
            .collect::<Vec<_>>();
        let reviewed: &[&str] = if participant {
            &[
                "ParticipantBounded.initial[2]: 260 != 200",
                "ParticipantBounded.hugUnbounded=false[2]: 260 != 200",
            ]
        } else {
            &[
                "LeafUnbounded.hugUnbounded=false[2]: 260 != 200",
                "LeafUnbounded.hugUnbounded=true[2]: 260 != 200",
                "LeafUnbounded.hugUnbounded=true[3]: 180 != 120",
            ]
        };
        assert_reviewed(&observed, reviewed);
    }
}
