//! Exact non-Silver owner-flow ports from pinned `layout_scroll_test.cpp`.

use std::path::PathBuf;

use crate::{RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance, StateMachineInstance};
use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};

use super::*;

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

struct Fixture {
    file: RuntimeFile,
    graphs: GraphFile,
    artboard_index: usize,
    artboard: ArtboardInstance,
}

impl Fixture {
    fn graph(&self) -> &ArtboardGraph {
        &self.graphs.artboards[self.artboard_index]
    }

    fn named_local(&self, name: &str) -> usize {
        self.graph()
            .local_objects
            .iter()
            .find(|object| object.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("fixture has object named {name}"))
            .local_id
    }

    fn scroll_local(&self) -> usize {
        let scrolls = self
            .artboard
            .components()
            .iter()
            .filter(|component| component.type_name == "ScrollConstraint")
            .map(|component| component.local_id)
            .collect::<Vec<_>>();
        assert_eq!(scrolls.len(), 1);
        scrolls[0]
    }

    fn scroll(&self) -> &crate::components::RuntimeScrollConstraintState {
        self.artboard
            .component(self.scroll_local())
            .and_then(|component| component.concrete.scroll.as_ref())
            .expect("live ScrollConstraint owner")
    }

    fn metrics(&self, include_item_bounds: bool) -> RuntimeScrollLayoutMetrics {
        let local = self.scroll_local();
        let handle = self
            .artboard
            .component_handle(local)
            .expect("scroll handle");
        runtime_scroll_layout_metrics(&self.artboard, handle, self.scroll(), include_item_bounds)
            .expect("layout-resolved scroll metrics")
    }
}

fn fixture(name: &str, artboard_name: Option<&str>) -> Fixture {
    let file = read_runtime_file(&pinned_fixture(name))
        .unwrap_or_else(|error| panic!("{name} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("{name} graphs: {error:#}"));
    let artboard_index = artboard_name.map_or(0, |wanted| {
        graphs
            .artboards
            .iter()
            .position(|graph| graph.name.as_deref() == Some(wanted))
            .unwrap_or_else(|| panic!("{name} has artboard {wanted}"))
    });
    let graph = &graphs.artboards[artboard_index];
    let artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .unwrap_or_else(|error| panic!("{name} instantiates: {error:#}"));
    Fixture {
        file,
        graphs,
        artboard_index,
        artboard,
    }
}

fn key(owner: &str, property: &str) -> u16 {
    property_key_for_name(owner, property)
        .unwrap_or_else(|| panic!("schema property {owner}.{property}"))
}

fn approx(actual: f32, expected: f32) {
    let scale = 1.0_f32.max(actual.abs()).max(expected.abs());
    assert!(
        (actual - expected).abs() <= 1.0e-5 * scale,
        "{actual} != {expected}"
    );
}

#[test]
fn scroll_constraint_vertical_offset() {
    let mut fixture = fixture("layout/layout_scroll_vertical.riv", None);
    assert_eq!(
        fixture
            .artboard
            .component(fixture.named_local("Content"))
            .unwrap()
            .type_name,
        "LayoutComponent"
    );
    let scroll = fixture.scroll_local();
    assert_eq!(fixture.scroll().offset_y, 0.0);
    fixture.artboard.advance(0.0).expect("initial advance");

    assert!(fixture.artboard.set_double_property(
        scroll,
        key("ScrollConstraint", "scrollPercentY"),
        1.0,
    ));
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollPercentY")),
        Some(1.0)
    );
    assert_eq!(fixture.scroll().offset_y, -610.0);
    assert_eq!(
        clamped_scroll_constraint_offsets(
            &fixture.artboard,
            fixture.artboard.component_handle(scroll).unwrap(),
            &fixture.metrics(false),
        )
        .1,
        -610.0
    );
    approx(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex"))
            .unwrap(),
        5.54545,
    );

    assert!(fixture.artboard.set_double_property(
        scroll,
        key("ScrollConstraint", "scrollIndex"),
        2.0,
    ));
    assert_eq!(fixture.scroll().offset_y, -220.0);
    assert_eq!(
        clamped_scroll_constraint_offsets(
            &fixture.artboard,
            fixture.artboard.component_handle(scroll).unwrap(),
            &fixture.metrics(false),
        )
        .1,
        -220.0
    );
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex")),
        Some(2.0)
    );
    let metrics = fixture.metrics(false);
    assert_eq!(metrics.content_height, 1090.0);
    assert_eq!(metrics.viewport_height, 490.0);
    assert_eq!(metrics.max_offset(RuntimeScrollAxis::Y), -610.0);
    assert_eq!(
        clamped_scroll_constraint_offsets(
            &fixture.artboard,
            fixture.artboard.component_handle(scroll).unwrap(),
            &metrics,
        )
        .1,
        -220.0
    );
}

#[test]
fn scroll_constraint_vertical_offset_manual() {
    let mut fixture = fixture("layout/layout_scroll_vertical.riv", None);
    assert_eq!(
        fixture
            .artboard
            .component(fixture.named_local("Content"))
            .unwrap()
            .type_name,
        "LayoutComponent"
    );
    assert_eq!(fixture.graph().state_machines.len(), 1);
    let mut machine = fixture
        .artboard
        .state_machine_instance(0)
        .expect("State Machine 1");
    let scroll = fixture.scroll_local();
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollPercentY")),
        Some(0.0)
    );
    assert_eq!(fixture.scroll().offset_y, 0.0);
    approx(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex"))
            .unwrap(),
        0.0,
    );
    assert!(!fixture.scroll().physics.as_ref().unwrap().is_running);

    fixture
        .artboard
        .advance(0.0)
        .expect("initial artboard advance");
    machine.pointer_move(&mut fixture.artboard, 50.0, 250.0, 0.0, 0);
    machine.pointer_down(&mut fixture.artboard, 50.0, 250.0, 0);
    fixture
        .artboard
        .advance(0.1)
        .expect("drag-start artboard advance");
    machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("drag-start state-machine advance");
    machine.pointer_move(&mut fixture.artboard, 50.0, 50.0, 0.1, 0);
    fixture
        .artboard
        .advance(0.0)
        .expect("drag-move artboard advance");
    machine
        .advance_and_apply(&mut fixture.artboard, 0.0)
        .expect("drag-move state-machine advance");

    approx(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollPercentY"))
            .unwrap(),
        0.32787,
    );
    assert_eq!(fixture.scroll().offset_y, -200.0);
    approx(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex"))
            .unwrap(),
        1.818182,
    );
    machine.pointer_up(&mut fixture.artboard, 50.0, 50.0, 0);
    assert!(fixture.scroll().physics.as_ref().unwrap().is_running);
}

#[test]
fn scroll_constraint_horizontal_offset() {
    let mut fixture = fixture("layout/layout_scroll_horizontal.riv", None);
    assert_eq!(
        fixture
            .artboard
            .component(fixture.named_local("Content"))
            .unwrap()
            .type_name,
        "LayoutComponent"
    );
    let scroll = fixture.scroll_local();
    assert_eq!(fixture.scroll().offset_x, 0.0);
    fixture.artboard.advance(0.0).expect("initial advance");
    assert!(fixture.artboard.set_double_property(
        scroll,
        key("ScrollConstraint", "scrollPercentX"),
        1.0,
    ));
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollPercentX")),
        Some(1.0)
    );
    assert_eq!(fixture.scroll().offset_x, -610.0);
    assert_eq!(
        clamped_scroll_constraint_offsets(
            &fixture.artboard,
            fixture.artboard.component_handle(scroll).unwrap(),
            &fixture.metrics(false),
        )
        .0,
        -610.0
    );
    approx(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex"))
            .unwrap(),
        5.54545,
    );
    assert!(fixture.artboard.set_double_property(
        scroll,
        key("ScrollConstraint", "scrollIndex"),
        2.0
    ));
    assert_eq!(fixture.scroll().offset_x, -220.0);
    assert_eq!(
        clamped_scroll_constraint_offsets(
            &fixture.artboard,
            fixture.artboard.component_handle(scroll).unwrap(),
            &fixture.metrics(false),
        )
        .0,
        -220.0
    );
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex")),
        Some(2.0)
    );
    let metrics = fixture.metrics(false);
    assert_eq!(metrics.content_width, 1090.0);
    assert_eq!(metrics.viewport_width, 490.0);
    assert_eq!(metrics.max_offset(RuntimeScrollAxis::X), -610.0);
    assert_eq!(
        clamped_scroll_constraint_offsets(
            &fixture.artboard,
            fixture.artboard.component_handle(scroll).unwrap(),
            &metrics,
        )
        .0,
        -220.0
    );
}

#[test]
fn scroll_constraint_list() {
    let mut fixture = fixture("layout/layout_scroll_list.riv", Some("Main"));
    assert!(
        fixture
            .artboard
            .bind_default_view_model_artboard_list_context(&fixture.file)
    );
    let content = fixture.named_local("Content");
    let list = fixture.named_local("List");
    assert_eq!(
        fixture.artboard.component(content).unwrap().type_name,
        "LayoutComponent"
    );
    assert_eq!(
        fixture.artboard.component(list).unwrap().type_name,
        "ArtboardComponentList"
    );
    let scroll = fixture.scroll_local();
    assert_eq!(fixture.scroll().offset_y, 0.0);
    fixture.artboard.advance(0.0).expect("initial advance");
    let assigned = fixture
        .artboard
        .runtime_component_list_assigned_layout_bounds();
    let rows = assigned.get(&list).expect("List layout-node bounds");
    assert_eq!(rows.len(), 20);
    for (index, bounds) in rows.iter().enumerate() {
        assert_eq!(bounds.y, index as f32 * 48.0);
    }
    assert!(fixture.artboard.set_double_property(
        scroll,
        key("ScrollConstraint", "scrollIndex"),
        2.0
    ));
    let metrics = fixture.metrics(true);
    assert_eq!(metrics.item_bounds.len(), 20);
    assert_eq!(fixture.scroll().offset_y, -96.0);
    assert_eq!(
        clamped_scroll_constraint_offsets(
            &fixture.artboard,
            fixture.artboard.component_handle(scroll).unwrap(),
            &metrics,
        )
        .1,
        -96.0
    );
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex")),
        Some(2.0)
    );
    assert_eq!(metrics.content_height, 960.0);
    assert_eq!(metrics.viewport_height, 500.0);
    assert_eq!(metrics.max_offset(RuntimeScrollAxis::Y), -460.0);
}

#[test]
#[ignore = "expected-red: scroll_intent.riv frame 6 keeps offset zero after live VM scrollIndex=2"]
fn scroll_constraint_index_intent_across_hidden_layout_live_assertions() {
    let mut fixture = fixture("scroll_intent.riv", None);
    let mut machine = fixture
        .artboard
        .state_machine_instance(0)
        .expect("default state machine");
    let view_model_index = usize::try_from(
        fixture
            .file
            .artboard(fixture.artboard_index)
            .and_then(|artboard| artboard.uint_property("viewModelId"))
            .expect("artboard viewModelId"),
    )
    .expect("viewModelId fits usize");
    let instance_index = fixture
        .file
        .view_model_default_instance(view_model_index)
        .expect("authored default view-model instance")
        .instance_index;
    let view_model = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(
            &fixture.file,
            view_model_index,
            instance_index,
        )
        .expect("default view-model instance builds"),
    );
    machine.bind_owned_view_model_handle(&view_model);
    let scroll_local = fixture.scroll_local();
    let content = fixture.scroll().content.expect("ScrollConstraint content");
    let content_local = fixture
        .artboard
        .objects
        .component_local_id(content)
        .expect("content occurrence");
    machine
        .advance_and_apply(&mut fixture.artboard, 0.0)
        .expect("initial state-machine advance");
    let vertical = fixture.metrics(false).constrains_vertical();
    let offset = |fixture: &Fixture| {
        if vertical {
            fixture.scroll().offset_y
        } else {
            fixture.scroll().offset_x
        }
    };
    for frame in 0..35 {
        if frame == 5 {
            assert!(
                view_model
                    .borrow_mut()
                    .set_number_by_property_name("scrollIndex", 2.0)
            );
        }
        if frame == 6 {
            assert!(offset(&fixture) < 0.0);
        }
        if frame == 10 {
            assert!(
                view_model
                    .borrow_mut()
                    .set_enum_by_property_name("display", 1)
            );
        }
        if frame == 11 {
            assert!(
                fixture
                    .artboard
                    .component(content_local)
                    .unwrap()
                    .is_collapsed()
            );
        }
        if frame == 15 {
            assert!(
                view_model
                    .borrow_mut()
                    .set_number_by_property_name("scrollIndex", 4.0)
            );
        }
        if frame == 16 {
            assert_eq!(
                fixture
                    .artboard
                    .double_property(scroll_local, key("ScrollConstraint", "scrollIndex"),),
                Some(4.0),
            );
        }
        if frame == 20 {
            assert!(
                view_model
                    .borrow_mut()
                    .set_enum_by_property_name("display", 0)
            );
        }
        if frame == 21 {
            assert!(
                !fixture
                    .artboard
                    .component(content_local)
                    .unwrap()
                    .is_collapsed()
            );
            assert!(offset(&fixture) < 0.0);
        }
        if frame == 25 {
            assert!(
                view_model
                    .borrow_mut()
                    .set_number_by_property_name("scrollIndex", 100.0)
            );
        }
        if frame == 26 {
            let metrics = fixture.metrics(false);
            assert_eq!(
                offset(&fixture),
                if vertical {
                    metrics.max_offset(RuntimeScrollAxis::Y)
                } else {
                    metrics.max_offset(RuntimeScrollAxis::X)
                },
            );
        }
        if frame == 30 {
            assert!(
                view_model
                    .borrow_mut()
                    .set_number_by_property_name("scrollIndex", 0.0)
            );
        }
        machine
            .advance_and_apply(&mut fixture.artboard, 1.0 / 60.0)
            .expect("pinned intent frame advance");
    }
    assert_eq!(offset(&fixture), 0.0);
}

#[test]
#[ignore = "expected-red: live ScrollConstraint snapshot cannot address nearestSnapOffsetInDirection"]
fn scroll_constraint_nearest_snap_offset_in_direction() {
    let mut fixture = fixture("layout/layout_scroll_vertical.riv", None);
    fixture.artboard.advance(0.0).expect("initial advance");
    let scroll = fixture.scroll_local();
    assert!(!constraint_bool(
        &fixture.artboard,
        scroll,
        "ScrollConstraint",
        "snap",
        false,
    ));
    let disabled_snapshot = fixture
        .artboard
        .scroll_constraint_occurrences()
        .into_iter()
        .next()
        .expect("live ScrollConstraint snapshot");
    assert_eq!(disabled_snapshot.offset, (0.0, 0.0));
    assert!(
        fixture
            .artboard
            .set_bool_property(scroll, key("ScrollConstraint", "snap"), true)
    );
    assert!(constraint_bool(
        &fixture.artboard,
        scroll,
        "ScrollConstraint",
        "snap",
        false,
    ));
    let enabled_snapshot = fixture
        .artboard
        .scroll_constraint_occurrences()
        .into_iter()
        .next()
        .expect("live ScrollConstraint snapshot after enabling snap");
    let pinned_owner_calls = [
        ((0.0, 0.0), (42.0, -150.0), (42.0, -150.0)),
        ((0.0, 0.0), (0.0, -150.0), (0.0, -220.0)),
        ((0.0, -500.0), (0.0, -150.0), (0.0, -110.0)),
        ((0.0, -330.0), (0.0, -330.0), (0.0, -330.0)),
        ((0.0, 0.0), (0.0, -220.0), (0.0, -220.0)),
    ];
    assert_ne!(
        disabled_snapshot, enabled_snapshot,
        "the live owner has no addressable nearestSnapOffsetInDirection capability for {pinned_owner_calls:?}",
    );
}

#[test]
fn elastic_scroll_physics_helper_snap_respects_trailing_padding() {
    fn settle(helper: &mut crate::components::RuntimeElasticScrollPhysicsHelper) -> f32 {
        let mut last = 0.0;
        for _ in 0..2000 {
            if !helper.is_running {
                break;
            }
            last = helper.advance(0.016);
        }
        assert!(!helper.is_running);
        last
    }
    let snaps = [0.0, 100.0, 200.0];
    for (acceleration, range_min, expected) in [
        (-781_250.0, -210.0, -210.0),
        (-781_250.0, -200.0, -200.0),
        (-343_750.0, -210.0, -100.0),
    ] {
        let mut helper = crate::components::RuntimeElasticScrollPhysicsHelper::new(8.0, 1.0, 0.66);
        helper.run(acceleration, range_min, 0.0, 0.0, &snaps, 300.0, 100.0);
        assert!((settle(&mut helper) - expected).abs() <= 0.5);
    }
}

fn velocity_fixture() -> (Fixture, StateMachineInstance) {
    let mut fixture = fixture("layout/scroll_velocity.riv", None);
    let mut machine = fixture
        .artboard
        .state_machine_instance(0)
        .expect("default state machine");
    let _ = machine.bind_default_view_model_context_on_artboard(&mut fixture.artboard);
    machine
        .advance_and_apply(&mut fixture.artboard, 0.0)
        .expect("initial state-machine advance");
    assert!(fixture.scroll().physics.is_some());
    (fixture, machine)
}

#[test]
fn viewport_drag_updates_velocity() {
    let (mut fixture, mut machine) = velocity_fixture();
    assert_eq!(fixture.scroll().physics.as_ref().unwrap().speed.1, 0.0);
    machine.pointer_move(&mut fixture.artboard, 200.0, 250.0, 1.0, 0);
    machine.pointer_down(&mut fixture.artboard, 200.0, 250.0, 0);
    for i in 1..=3 {
        machine.pointer_move(
            &mut fixture.artboard,
            200.0,
            250.0 - 50.0 * i as f32,
            1.0 + i as f32,
            0,
        );
        machine
            .advance_and_apply(&mut fixture.artboard, 0.016)
            .expect("drag advance");
    }
    assert_ne!(fixture.scroll().physics.as_ref().unwrap().speed.1, 0.0);
    assert!(runtime_scroll_active(fixture.scroll()));
}

#[test]
fn viewport_release_triggers_fling() {
    let (mut fixture, mut machine) = velocity_fixture();
    machine.pointer_move(&mut fixture.artboard, 200.0, 250.0, 1.0, 0);
    machine.pointer_down(&mut fixture.artboard, 200.0, 250.0, 0);
    for i in 1..=4 {
        machine.pointer_move(
            &mut fixture.artboard,
            200.0,
            250.0 - 100.0 * i as f32,
            1.0 + i as f32,
            0,
        );
        machine
            .advance_and_apply(&mut fixture.artboard, 0.016)
            .expect("swipe advance");
    }
    machine.pointer_up(&mut fixture.artboard, 200.0, -150.0, 0);
    assert!(fixture.scroll().physics.as_ref().unwrap().is_running);
    assert_ne!(fixture.scroll().physics.as_ref().unwrap().speed.1, 0.0);
}

#[test]
fn viewport_drag_held_still_clears_velocity() {
    let (mut fixture, mut machine) = velocity_fixture();
    machine.pointer_move(&mut fixture.artboard, 200.0, 250.0, 1.0, 0);
    machine.pointer_down(&mut fixture.artboard, 200.0, 250.0, 0);
    machine.pointer_move(&mut fixture.artboard, 200.0, 200.0, 2.0, 0);
    machine
        .advance_and_apply(&mut fixture.artboard, 0.016)
        .expect("motion advance");
    assert_ne!(fixture.scroll().physics.as_ref().unwrap().speed.1, 0.0);
    machine
        .advance_and_apply(&mut fixture.artboard, 0.016)
        .expect("held advance");
    assert_eq!(fixture.scroll().physics.as_ref().unwrap().speed.1, 0.0);
}

#[test]
fn scrollbar_drag_updates_velocity() {
    let (mut fixture, mut machine) = velocity_fixture();
    machine.pointer_move(&mut fixture.artboard, 475.0, 50.0, 1.0, 0);
    machine.pointer_down(&mut fixture.artboard, 475.0, 50.0, 0);
    for i in 1..=3 {
        machine.pointer_move(
            &mut fixture.artboard,
            475.0,
            50.0 + 30.0 * i as f32,
            1.0 + i as f32,
            0,
        );
        machine
            .advance_and_apply(&mut fixture.artboard, 0.016)
            .expect("scrollbar advance");
    }
    assert!(fixture.scroll().is_scroll_bar_dragging);
    assert_ne!(fixture.scroll().physics.as_ref().unwrap().speed.1, 0.0);
}

#[test]
#[ignore = "expected-red: live ScrollBarConstraint release leaves is_scroll_bar_dragging set"]
fn scrollbar_release_zeros_velocity() {
    let (mut fixture, mut machine) = velocity_fixture();
    machine.pointer_move(&mut fixture.artboard, 475.0, 50.0, 1.0, 0);
    machine.pointer_down(&mut fixture.artboard, 475.0, 50.0, 0);
    for i in 1..=3 {
        machine.pointer_move(
            &mut fixture.artboard,
            475.0,
            50.0 + 30.0 * i as f32,
            1.0 + i as f32,
            0,
        );
        machine
            .advance_and_apply(&mut fixture.artboard, 0.016)
            .expect("scrollbar advance");
    }
    assert_ne!(fixture.scroll().physics.as_ref().unwrap().speed.1, 0.0);
    machine.pointer_up(&mut fixture.artboard, 475.0, 140.0, 0);
    machine
        .advance_and_apply(&mut fixture.artboard, 0.016)
        .expect("release advance");
    assert_eq!(fixture.scroll().physics.as_ref().unwrap().speed.1, 0.0);
    assert!(!fixture.scroll().is_scroll_bar_dragging);
    assert!(!fixture.scroll().physics.as_ref().unwrap().is_running);
}

#[test]
fn scroll_constraint_index_set_before_layout_resolves_on_advance() {
    let mut fixture = fixture("layout/layout_scroll_vertical.riv", None);
    let scroll = fixture.scroll_local();
    assert!(fixture.artboard.set_double_property(
        scroll,
        key("ScrollConstraint", "scrollIndex"),
        2.0
    ));
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex")),
        Some(2.0)
    );
    assert_eq!(fixture.scroll().offset_y, 0.0);
    fixture.artboard.advance(0.0).expect("first layout advance");
    assert_eq!(fixture.scroll().offset_y, -220.0);
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex")),
        Some(2.0)
    );
}

#[test]
fn scroll_constraint_percent_set_before_layout_resolves_on_advance() {
    let mut fixture = fixture("layout/layout_scroll_vertical.riv", None);
    let scroll = fixture.scroll_local();
    assert!(fixture.artboard.set_double_property(
        scroll,
        key("ScrollConstraint", "scrollPercentY"),
        0.5
    ));
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollPercentY")),
        Some(0.5)
    );
    assert_eq!(fixture.scroll().offset_y, 0.0);
    fixture.artboard.advance(0.0).expect("first layout advance");
    assert_eq!(fixture.scroll().offset_y, -305.0);
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollPercentY")),
        Some(0.5)
    );
}

#[test]
fn scroll_constraint_out_of_range_index_clamps_to_the_ends() {
    let mut fixture = fixture("layout/layout_scroll_vertical.riv", None);
    let scroll = fixture.scroll_local();
    fixture.artboard.advance(0.0).expect("initial advance");
    assert!(fixture.artboard.set_double_property(
        scroll,
        key("ScrollConstraint", "scrollIndex"),
        99.0
    ));
    assert_eq!(fixture.scroll().offset_y, -610.0);
    approx(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex"))
            .unwrap(),
        5.54545,
    );
    assert!(fixture.artboard.set_double_property(
        scroll,
        key("ScrollConstraint", "scrollIndex"),
        -5.0
    ));
    assert_eq!(fixture.scroll().offset_y, 0.0);
    assert_eq!(
        fixture
            .artboard
            .double_property(scroll, key("ScrollConstraint", "scrollIndex")),
        Some(0.0)
    );
}
