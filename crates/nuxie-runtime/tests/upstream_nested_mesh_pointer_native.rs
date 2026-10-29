//! Literal nested_mesh_pointer_test.cpp at upstream fab1d5692.
//! The Rust runtime's nested raster/mesh implementation is the RIVE_CANVAS lane.
#![cfg(feature = "testing")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        nested_state_machine::NestedStateMachine, state_machine::StateMachine,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
        state_machine_listener::StateMachineListener,
    },
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    generated::{
        animation::state_machine_listener_base::StateMachineListenerBase,
        core_registry::CoreRegistry,
    },
    hit_result::HitResult,
    listener_phase::ListenerPhase,
    listener_type::ListenerType,
    math::vec2d::Vec2D,
    nested_artboard::NestedArtboard,
    nested_artboard_mesh_host::NestedArtboardMeshHost,
    node::Node,
    pointer_button::PointerButton,
    scroll_event::ScrollEvent,
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
use std::{cell::RefCell, rc::Rc};

fn v(x: f32, y: f32) -> Vec2D {
    Vec2D::new(x, y)
}
// Literal Catch Approx defaults and comparison from tests/include/catch.hpp:
// epsilon=100*float epsilon, scale=0, margin=0. The relative margin is based
// only on the expected value, and comparisons avoid subtraction for infinity.
fn approx(actual: f32, expected: f32) -> bool {
    let (actual, expected) = (f64::from(actual), f64::from(expected));
    let margin_comparison = |margin| (expected + margin >= actual) && (actual + margin >= expected);
    margin_comparison(0.0)
        || margin_comparison(
            f64::from(f32::EPSILON)
                * 100.0
                * if expected.is_infinite() {
                    0.0
                } else {
                    expected.abs()
                },
        )
}
fn assert_approx(actual: f32, expected: f32) {
    assert!(approx(actual, expected), "{actual} != Approx({expected})");
}
fn capture(file: &RuntimeFileHandle, name: &str) {
    let source = file
        .with_file(|f| f.artboard_named_source("Inner"))
        .unwrap();
    let machine = source
        .with_downcast::<Artboard, _>(|a| a.state_machine_handle_at(0))
        .flatten()
        .unwrap();
    let listeners = machine
        .with_downcast::<StateMachine, _>(|m| {
            (0..m.listener_count())
                .map(|i| m.listener(i).unwrap())
                .collect::<Vec<_>>()
        })
        .unwrap();
    for listener in listeners {
        if listener
            .with_downcast::<StateMachineListener, _>(|l| l.name() == name)
            .unwrap()
        {
            assert!(CoreRegistry::set_uint_handle(
                &listener,
                StateMachineListenerBase::PHASE_VALUE_PROPERTY_KEY.into(),
                ListenerPhase::Capture as u32
            ));
            return;
        }
    }
    panic!("no listener named {name}");
}
fn inner(nested: &CoreHandle) -> RuntimeStateMachineInstanceHandle {
    let animation = nested
        .with_downcast::<NestedArtboard, _>(|n| n.nested_animations()[0].clone())
        .unwrap();
    animation
        .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
        .flatten()
        .unwrap()
}
fn artboard(machine: &RuntimeStateMachineInstanceHandle) -> RuntimeArtboardInstanceHandle {
    machine.with_instance(|m| m.artboard().upgrade()).unwrap()
}
fn input(machine: &RuntimeStateMachineInstanceHandle, name: &str) -> bool {
    machine.with_instance(|m| m.get_bool(name).unwrap().value())
}
fn follower(machine: &RuntimeStateMachineInstanceHandle) -> CoreHandle {
    artboard(machine)
        .with_artboard(|a| a.find_handle::<Node>("Follower"))
        .unwrap()
}
fn check_follower(node: &CoreHandle, x: f32, y: f32) {
    let (actual_x, actual_y) = node
        .with_downcast::<Node, _>(|n| (n.base.x(), n.base.y()))
        .unwrap();
    assert_approx(actual_x, x);
    assert_approx(actual_y, y);
}
fn host(nested: &CoreHandle) -> Rc<RefCell<NestedArtboardMeshHost>> {
    nested
        .with_downcast::<NestedArtboard, _>(NestedArtboard::mesh_host_if_any)
        .flatten()
        .unwrap()
}
fn map(nested: &CoreHandle, pointer: Vec2D) -> (Vec2D, bool) {
    let mut local = Vec2D::default();
    let mut off_mesh = true;
    assert!(
        nested
            .with_downcast::<NestedArtboard, _>(|n| n.world_to_local_mapped(
                pointer,
                &mut local,
                Some(&mut off_mesh),
                None
            ))
            .unwrap()
    );
    (local, off_mesh)
}
struct Scene {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
}
impl Scene {
    fn new(capture_listener: Option<&str>) -> Self {
        let root = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream"),
        );
        let bytes =
            std::fs::read(root.join("tests/unit_tests/assets/nested_mesh_pointers.riv")).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        if let Some(name) = capture_listener {
            capture(&file, name);
        }
        let artboard = file.with_file(|f| f.artboard_named("Host")).unwrap();
        let machine = artboard.state_machine_at(0).unwrap();
        // Upstream sorts hit components by draw order on the second advance.
        machine.advance_and_apply(0.0);
        machine.advance_and_apply(0.0);
        Self {
            _file: file,
            artboard,
            machine,
        }
    }
    fn nested(&self, name: &str) -> CoreHandle {
        self.artboard
            .with_artboard(|a| a.find_handle::<NestedArtboard>(name))
            .unwrap()
    }
    fn inner(&self, name: &str) -> RuntimeStateMachineInstanceHandle {
        inner(&self.nested(name))
    }
    fn input(&self, nested: &str, name: &str) -> bool {
        input(&self.inner(nested), name)
    }
    fn down(&self, p: Vec2D) {
        self.machine
            .with_instance_mut(|m| m.pointer_down(p, 0, PointerButton::Primary));
    }
    fn up(&self, p: Vec2D) {
        self.machine
            .with_instance_mut(|m| m.pointer_up(p, 0, PointerButton::Primary));
    }
    fn moved(&self, p: Vec2D) {
        self.machine
            .with_instance_mut(|m| m.pointer_move(p, 0.0, 0));
    }
    fn press(&self, p: Vec2D) {
        self.down(p);
        self.up(p);
        self.machine.advance_and_apply(0.0);
    }
    fn hit(&self, p: Vec2D) -> bool {
        self.machine.with_instance(|m| m.hit_test(p))
    }
}

#[test]
fn pointers_follow_a_nested_artboards_mesh() {
    let scene = Scene::new(None);
    assert!(scene.hit(v(130.0, 100.0)));
    scene.press(v(130.0, 100.0));
    assert!(scene.input("Card", "Left"));
    assert!(!scene.input("Card", "Right"));
    scene.press(v(200.0, 140.0));
    assert!(scene.input("Card", "Right"));
}
#[test]
fn a_pointer_past_a_nested_mesh_edge_hits_nothing() {
    let scene = Scene::new(None);
    assert!(!scene.hit(v(110.0, 140.0)));
    scene.press(v(110.0, 140.0));
    assert!(!scene.input("Card", "Left"));
    assert!(!scene.input("Card", "Right"));
}
#[test]
fn a_drag_keeps_following_the_pointer_past_a_nested_mesh_edge() {
    let scene = Scene::new(None);
    let follower = follower(&scene.inner("Card"));
    scene.down(v(130.0, 100.0));
    scene.moved(v(140.0, 100.0));
    check_follower(&follower, 40.0, 50.0);
    assert!(!scene.input("Card", "Exited"));
    scene.moved(v(110.0, 140.0));
    scene.machine.advance_and_apply(0.0);
    assert!(scene.input("Card", "Exited"));
    check_follower(&follower, -30.0, 90.0);
    scene.up(v(110.0, 140.0));
}
#[test]
fn a_capture_listener_off_a_nested_mesh_sees_the_pointer_leave() {
    let scene = Scene::new(Some("LeftDown"));
    assert!(scene.machine.with_instance(|m| m.has_capture_listeners()));
    scene.press(v(290.0, 100.0));
    assert!(!scene.input("Sliced", "Left"));
    scene.press(v(260.0, 60.0));
    assert!(scene.input("Sliced", "Left"));
}
#[test]
fn a_missed_press_does_not_reach_capture_listeners() {
    let scene = Scene::new(Some("LeftDown"));
    scene.machine.with_instance_mut(|m| {
        m.pointer_miss(
            v(130.0, 100.0),
            ListenerType::Down,
            0.0,
            0,
            PointerButton::Primary,
        )
    });
    scene.machine.with_instance_mut(|m| {
        m.pointer_miss(
            v(130.0, 100.0),
            ListenerType::Up,
            0.0,
            0,
            PointerButton::Primary,
        )
    });
    scene.machine.advance_and_apply(0.0);
    assert!(!scene.input("Card", "Left"));
    scene.press(v(130.0, 100.0));
    assert!(scene.input("Card", "Left"));
}
#[test]
fn a_hidden_n_slice_patch_is_a_hole_for_pointers() {
    let scene = Scene::new(None);
    assert!(!scene.hit(v(300.0, 100.0)));
    scene.press(v(300.0, 100.0));
    assert!(!scene.input("Sliced", "Left"));
    assert!(!scene.input("Sliced", "Right"));
    scene.press(v(260.0, 60.0));
    assert!(scene.input("Sliced", "Left"));
}
#[test]
fn nested_mesh_mapping_round_trips() {
    let scene = Scene::new(None);
    let card = scene.nested("Card");
    let (local, off_mesh) = map(&card, v(130.0, 100.0));
    assert!(!off_mesh);
    assert_approx(local.x, 30.0);
    assert_approx(local.y, 50.0);
    let mut world = Vec2D::default();
    assert!(host(&card).borrow().world_through_mesh(local, &mut world));
    assert_approx(world.x, 130.0);
    assert_approx(world.y, 100.0);
}
#[test]
fn pointers_skip_a_nested_mesh_that_did_not_draw() {
    let scene = Scene::new(None);
    host(&scene.nested("Card"))
        .borrow_mut()
        .drew_through_mesh(false);
    scene.press(v(110.0, 140.0));
    assert!(scene.input("Card", "Right"));
}
#[test]
fn a_drag_in_an_artboard_nested_in_a_meshed_one_survives_the_mesh_edge() {
    let scene = Scene::new(None);
    let card_inner = scene.inner("Card");
    let deep = artboard(&card_inner)
        .with_artboard(|a| a.find_handle::<NestedArtboard>("Deep"))
        .unwrap();
    let deep_machine = inner(&deep);
    let follower = follower(&deep_machine);
    scene.down(v(230.0, 140.0));
    scene.moved(v(231.0, 140.0));
    check_follower(&follower, 11.0, 10.0);
    assert!(!input(&deep_machine, "Exited"));
    scene.moved(v(245.0, 140.0));
    assert!(input(&deep_machine, "Exited"));
    check_follower(&follower, 25.0, 10.0);
    scene.up(v(245.0, 140.0));
}
#[test]
fn ending_a_drag_in_an_n_slice_hole_reaches_nothing_there() {
    let scene = Scene::new(None);
    scene.down(v(260.0, 100.0));
    scene.moved(v(310.0, 100.0));
    assert!(!scene.input("Sliced", "RightMoved"));
    scene.up(v(310.0, 100.0));
    scene.machine.advance_and_apply(0.0);
    assert!(!scene.input("Sliced", "RightMoved"));
    scene.moved(v(310.0, 60.0));
    assert!(scene.input("Sliced", "RightMoved"));
}
#[test]
fn a_hosts_drag_ending_over_an_n_slice_hole_reaches_nothing_there() {
    let scene = Scene::new(None);
    // Upstream retains the HostDragged input before dispatching pointers.
    assert!(
        scene
            .machine
            .with_instance(|m| m.get_bool("HostDragged").is_some())
    );
    scene.down(v(430.0, 300.0));
    scene.moved(v(440.0, 300.0));
    assert!(input(&scene.machine, "HostDragged"));
    scene.moved(v(510.0, 300.0));
    scene.up(v(510.0, 300.0));
    scene.machine.advance_and_apply(0.0);
    assert!(!scene.input("Holed", "RightMoved"));
    scene.moved(v(510.0, 260.0));
    assert!(scene.input("Holed", "RightMoved"));
}
#[test]
fn an_n_slice_with_every_patch_hidden_takes_no_pointers() {
    let scene = Scene::new(None);
    assert!(!scene.hit(v(450.0, 100.0)));
    scene.press(v(450.0, 100.0));
    scene.press(v(410.0, 60.0));
    assert!(!scene.input("Hollow", "Left"));
    assert!(!scene.input("Hollow", "Right"));
    let mut local = Vec2D::default();
    let mut off_mesh = false;
    assert!(
        scene
            .nested("Hollow")
            .with_downcast::<NestedArtboard, _>(|n| n.world_to_local_mapped(
                v(450.0, 100.0),
                &mut local,
                Some(&mut off_mesh),
                None
            ))
            .unwrap()
    );
    assert!(off_mesh);
}
#[test]
fn a_repeated_n_slice_tile_maps_back_to_the_pointers_own_tile() {
    let scene = Scene::new(None);
    let tiled = scene.nested("Tiled");
    let pointer = v(137.5, 300.0);
    let (local, off_mesh) = map(&tiled, pointer);
    assert!(!off_mesh);
    let mesh_host = host(&tiled);
    let mut world = Vec2D::default();
    assert!(mesh_host.borrow().world_through_mesh(local, &mut world));
    assert_approx(world.x, pointer.x);
    assert_approx(world.y, pointer.y);
    let mesh = mesh_host.borrow().hosted_mesh().unwrap();
    let mut last_tile = Vec2D::default();
    assert!(
        mesh.with(|mesh| mesh
            .mesh_drawable_position_at_uv(v(local.x / 100.0, local.y / 100.0), &mut last_tile))
            .unwrap()
    );
    assert!(!approx(last_tile.x, pointer.x));
}
#[test]
fn a_scroll_over_a_repeated_n_slice_tile_keeps_its_direction() {
    let scene = Scene::new(None);
    let tiled_strip = scene.nested("TiledStrip");
    let scroll = artboard(&scene.inner("TiledStrip"))
        .with_artboard(|a| a.find_handle::<ScrollConstraint>("Scroll"))
        .unwrap();
    let pointer = v(137.5, 450.0);
    let delta = v(-40.0, 0.0);
    let mut local = Vec2D::default();
    let mut end = Vec2D::default();
    assert!(
        tiled_strip
            .with_downcast::<NestedArtboard, _>(|n| n.world_to_local(pointer, &mut local))
            .unwrap()
    );
    assert!(
        tiled_strip
            .with_downcast::<NestedArtboard, _>(|n| n.world_to_local(pointer + delta, &mut end))
            .unwrap()
    );
    assert!(end.x > local.x);
    let event = ScrollEvent {
        delta,
        ..ScrollEvent::default()
    };
    assert_ne!(
        scene
            .machine
            .with_instance_mut(|m| m.pointer_scroll(pointer, &event, 0.0, 0)),
        HitResult::None
    );
    assert_approx(
        scroll
            .with_downcast::<ScrollConstraint, _>(ScrollConstraint::offset_x)
            .unwrap(),
        -40.0,
    );
    let mut world = Vec2D::default();
    assert!(
        host(&tiled_strip)
            .borrow()
            .world_through_mesh(local, &mut world)
    );
    assert_approx(world.x, pointer.x);
}

#[test]
fn inner_artboard_hit_reverse_maps_without_reborrowing_its_bounds() {
    // The ordinary pointer path caches content->world first. Direct inner
    // hits must also work when no cache exists or its content point differs.
    // Cover both explicit Mesh and N-slice fallback transforms.
    for (name, prior_pointer, content) in [
        ("Card", v(130.0, 100.0), v(60.0, 90.0)),
        ("Sliced", v(255.0, 55.0), v(10.0, 10.0)),
    ] {
        for cached_other_point in [false, true] {
            let scene = Scene::new(None);
            let nested = scene.nested(name);
            if cached_other_point {
                let (prior_content, off_mesh) = map(&nested, prior_pointer);
                assert!(!off_mesh);
                let difference = content - prior_content;
                assert!(difference.x * difference.x + difference.y * difference.y > 1e-6);
            }
            let inner = artboard(&scene.inner(name));
            assert!(
                inner.with_artboard_mut(|a| a.hit_test_point(&content, false, true)),
                "{name}, cached_other_point={cached_other_point}"
            );
        }
    }
}

#[test]
fn direct_nested_host_trait_hits_do_not_reborrow_the_owner() {
    // Artboard's released handle path is not the only public dispatch: hosts
    // also expose ArtboardHost under their mutable CoreObject occurrence.
    for (name, pointer, uncached_content) in [
        ("Card", v(130.0, 100.0), v(60.0, 90.0)),
        ("Sliced", v(255.0, 55.0), v(10.0, 10.0)),
    ] {
        for cached in [false, true] {
            let scene = Scene::new(None);
            let nested = scene.nested(name);
            let content = if cached {
                let (content, off_mesh) = map(&nested, pointer);
                assert!(!off_mesh);
                content
            } else {
                uncached_content
            };
            let instance = artboard(&scene.inner(name));
            assert!(
                nested
                    .with_mut(|owner| {
                        owner.as_artboard_host_mut().unwrap().hit_test_host(
                            &content,
                            false,
                            instance.downgrade(),
                        )
                    })
                    .unwrap(),
                "{name}, cached={cached}"
            );
        }
    }
}
