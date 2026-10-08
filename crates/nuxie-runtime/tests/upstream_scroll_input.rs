//! Native `runtime/scroll_input_test.cpp`, updated through upstream de3e8609.
use nuxie_render_api::{NullFactory, PersistentFactory, SerializingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::{RuntimeStateMachineInstanceHandle, StateMachineInstance},
    constraints::scrolling::{
        clamped_scroll_physics::ClampedScrollPhysics, scroll_constraint::ScrollConstraint,
        scroll_physics,
    },
    generated::{
        constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
        core_registry::CoreRegistry,
    },
    hit_result::HitResult,
    layout_component::LayoutComponent,
    math::vec2d::Vec2D,
    scroll_event::{ScrollEvent, ScrollPhase},
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};

fn wheel(dx: f32, dy: f32) -> ScrollEvent {
    ScrollEvent {
        delta: Vec2D::new(dx, dy),
        phase: ScrollPhase::Update,
        precise: false,
    }
}
fn trackpad(phase: ScrollPhase, dx: f32, dy: f32) -> ScrollEvent {
    ScrollEvent {
        delta: Vec2D::new(dx, dy),
        phase,
        precise: true,
    }
}
fn read_file(asset: &str, silver: bool) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(asset),
    )
    .expect("pinned scroll fixture");
    let factory = if silver {
        RuntimeFactoryHandle::from_factory(&mut PersistentFactory::new(SerializingFactory::new()))
            .unwrap()
    } else {
        RuntimeFactoryHandle::from_factory(&mut PersistentFactory::new(NullFactory::default()))
            .unwrap()
    };
    File::import(&bytes, factory, None, None, None).expect("scroll fixture imports")
}
struct Scene {
    // Machine drops before its artboard and defining file.
    smi: RuntimeStateMachineInstanceHandle,
    artboard: RuntimeArtboardInstanceHandle,
    scroll: CoreHandle,
    _file: RuntimeFileHandle,
}
impl Scene {
    fn open(asset: &str) -> Self {
        let file = read_file(asset, false);
        let source = file.with_file(File::artboard).expect("source artboard");
        let artboard = Artboard::instance_from_handle(&source).expect("instance");
        let definition = source
            .with_downcast::<Artboard, _>(|a| a.state_machine_named("State Machine 1"))
            .flatten()
            .expect("State Machine 1");
        let smi = StateMachineInstance::new(definition, artboard.downgrade());
        let scroll =
            artboard.with_artboard(|a| a.find_all_handles::<ScrollConstraint>()[0].clone());
        Self {
            smi,
            artboard,
            scroll,
            _file: file,
        }
    }
    fn vertical() -> Self {
        let scene = Self::open("layout/layout_scroll_vertical.riv");
        scene.advance(0.0);
        scene
    }
    fn advance(&self, seconds: f32) {
        self.smi.advance_and_apply(seconds);
    }
    fn idle(&self) {
        self.advance(0.2);
    }
    fn send_at(&self, over: Vec2D, event: ScrollEvent) -> HitResult {
        self.smi
            .with_instance_mut(|s| s.pointer_scroll(over, &event, 0.0, 0))
    }
    fn send(&self, event: ScrollEvent) -> HitResult {
        self.send_at(Vec2D::new(50.0, 250.0), event)
    }
    fn get<T>(&self, f: impl FnOnce(&ScrollConstraint) -> T) -> T {
        get(&self.scroll, f)
    }
    fn running(&self) -> bool {
        physics(&self.scroll, |p| p.is_running())
    }
    fn enabled(&self) -> bool {
        physics(&self.scroll, |p| p.enabled())
    }
    fn latch(&self) -> bool {
        self.smi.with_instance_mut(|s| s.has_scroll_latch())
    }
    fn center(&self) -> Vec2D {
        self.artboard
            .with_artboard(|a| Vec2D::new(a.width() / 2.0, a.height() / 2.0))
    }
}
fn get<T>(h: &CoreHandle, f: impl FnOnce(&ScrollConstraint) -> T) -> T {
    h.with_downcast::<ScrollConstraint, _>(f)
        .expect("live scroll")
}
fn physics<T>(h: &CoreHandle, f: impl FnOnce(&dyn scroll_physics::ScrollPhysicsRuntime) -> T) -> T {
    let p = get(h, ScrollConstraint::physics).expect("scroll physics");
    p.with(|o| f(scroll_physics::from_core(o).expect("physics runtime")))
        .unwrap()
}
fn approx(actual: f32, expected: f32) {
    let (actual, expected) = (f64::from(actual), f64::from(expected));
    let magnitude = if expected.is_infinite() {
        0.0
    } else {
        expected.abs()
    };
    let margin = f64::from(f32::EPSILON * 100.0) * magnitude;
    assert!(
        actual == expected || (actual + margin >= expected && expected + margin >= actual),
        "{actual} != Approx({expected})"
    );
}
struct NestedScene {
    scene: Scene,
    outer: CoreHandle,
    strip: CoreHandle,
    over: Vec2D,
}
fn nested() -> NestedScene {
    let scene = Scene::open("layout/scroll_nested.riv");
    let outer = scene
        .artboard
        .with_artboard(|a| a.find_handle::<ScrollConstraint>("Scroll V"))
        .expect("outer");
    let strip = scene
        .artboard
        .with_artboard(|a| a.find_handle::<ScrollConstraint>("Scroll H"))
        .expect("strip");
    scene.advance(0.0);
    let viewport = scene
        .artboard
        .with_artboard(|a| a.find_handle::<LayoutComponent>("Strip Viewport"))
        .expect("strip viewport");
    let over = viewport
        .with_downcast::<LayoutComponent, _>(|l| l.world_bounds().center())
        .unwrap();
    NestedScene {
        scene,
        outer,
        strip,
        over,
    }
}

#[test]
fn wheel_scrolls_without_flinging() {
    let s = Scene::vertical();
    assert_eq!(s.get(ScrollConstraint::offset_y), 0.0);
    assert_ne!(s.send(wheel(0.0, -50.0)), HitResult::None);
    assert_eq!(s.get(ScrollConstraint::offset_y), -50.0);
    s.send(wheel(0.0, -70.0));
    assert_eq!(s.get(ScrollConstraint::offset_y), -120.0);
    assert_eq!(s.get(ScrollConstraint::velocity_y), 0.0);
    assert!(!s.running());
}
#[test]
fn wheel_reports_active_until_idle() {
    let s = Scene::vertical();
    assert!(!s.get(ScrollConstraint::scroll_active));
    s.send(wheel(0.0, -50.0));
    assert!(s.get(ScrollConstraint::scroll_active));
    s.advance(0.05);
    assert!(s.get(ScrollConstraint::scroll_active));
    s.idle();
    assert!(!s.get(ScrollConstraint::scroll_active));
}
#[test]
fn immovable_view_declines_scroll() {
    let s = Scene::vertical();
    assert!(!s.get(|c| c.can_consume(Vec2D::new(0.0, 50.0))));
    assert_eq!(s.send(wheel(0.0, 50.0)), HitResult::None);
    assert_eq!(s.get(ScrollConstraint::offset_y), 0.0);
    assert!(!s.get(ScrollConstraint::scroll_active));
    assert!(s.get(|c| c.can_consume(Vec2D::new(0.0, -50.0))));
    assert_ne!(s.send(wheel(0.0, -50.0)), HitResult::None);
}
#[test]
fn outside_viewport_is_not_consumed() {
    let s = Scene::vertical();
    assert_eq!(
        s.send_at(Vec2D::new(-500.0, -500.0), wheel(0.0, -50.0)),
        HitResult::None
    );
    assert_eq!(s.get(ScrollConstraint::offset_y), 0.0);
}
#[test]
fn gesture_stays_latched_past_edge() {
    let s = Scene::vertical();
    assert_ne!(s.send(wheel(0.0, -600.0)), HitResult::None);
    assert_ne!(s.send(wheel(0.0, -100.0)), HitResult::None);
    assert_eq!(s.get(ScrollConstraint::offset_y), -610.0);
    assert!(!s.get(|c| c.can_consume(Vec2D::new(0.0, -100.0))));
    assert!(s.latch());
    assert_ne!(s.send(wheel(0.0, -100.0)), HitResult::None);
    s.idle();
    assert!(!s.latch());
    assert_eq!(s.send(wheel(0.0, -100.0)), HitResult::None);
}
#[test]
fn trackpad_flings_on_release() {
    let s = Scene::vertical();
    s.send(trackpad(ScrollPhase::Begin, 0.0, 0.0));
    assert!(!s.get(ScrollConstraint::scroll_active));
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    assert!(s.get(ScrollConstraint::scroll_active));
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    assert_eq!(s.get(ScrollConstraint::offset_y), -80.0);
    assert_ne!(s.get(ScrollConstraint::velocity_y), 0.0);
    s.send(trackpad(ScrollPhase::End, 0.0, 0.0));
    assert!(s.running());
}
#[test]
fn platform_momentum_never_flings() {
    let s = Scene::vertical();
    s.send(trackpad(ScrollPhase::Momentum, 0.0, -30.0));
    s.send(trackpad(ScrollPhase::Momentum, 0.0, -20.0));
    assert_eq!(s.get(ScrollConstraint::offset_y), -50.0);
    assert!(!s.running());
    assert_eq!(s.get(ScrollConstraint::velocity_y), 0.0);
}
#[test]
fn inertia_cancel_halts_fling() {
    let s = Scene::vertical();
    s.send(trackpad(ScrollPhase::Begin, 0.0, 0.0));
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    s.send(trackpad(ScrollPhase::End, 0.0, 0.0));
    assert!(s.running());
    s.send(trackpad(ScrollPhase::InertiaCancel, 0.0, 0.0));
    assert!(!s.running());
    assert_eq!(s.get(ScrollConstraint::velocity_y), 0.0);
}
#[test]
fn only_constrained_axes_move() {
    let s = Scene::open("layout/layout_scroll_horizontal.riv");
    s.advance(0.0);
    let over = s.center();
    assert!(!s.get(|c| c.can_consume(Vec2D::new(0.0, -50.0))));
    assert_eq!(s.send_at(over, wheel(0.0, -50.0)), HitResult::None);
    assert_ne!(s.send_at(over, wheel(-50.0, 0.0)), HitResult::None);
    assert_eq!(s.get(ScrollConstraint::offset_x), -50.0);
    assert_eq!(s.get(ScrollConstraint::offset_y), 0.0);
}
#[test]
fn snapping_view_settles_on_wheel_idle() {
    let file = read_file("layout/layout_scroll_snap.riv", true);
    let artboard = file
        .with_file(|f| f.artboard_named("main"))
        .expect("main instance");
    if let Some(vm) = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(artboard.core_handle()))
    {
        artboard.bind_view_model_instance(Some(vm));
    }
    let smi = artboard.default_state_machine().expect("default machine");
    smi.advance_and_apply(0.0);
    let scroll = artboard.with_artboard(|a| a.find_all_handles::<ScrollConstraint>()[0].clone());
    let s = Scene {
        smi,
        artboard,
        scroll,
        _file: file,
    };
    assert!(s.get(|c| c.snap()));
    assert_ne!(s.send_at(s.center(), wheel(-30.0, 0.0)), HitResult::None);
    assert!(!s.running());
    let mut settling = false;
    for _ in 0..20 {
        if settling {
            break;
        }
        s.advance(0.016);
        settling = s.running();
    }
    assert!(settling);
}
#[test]
fn nonsnapping_view_stops_on_wheel_idle() {
    let s = Scene::vertical();
    assert!(!s.get(|c| c.snap()));
    s.send(wheel(0.0, -50.0));
    s.idle();
    assert_eq!(s.get(ScrollConstraint::offset_y), -50.0);
    assert!(!s.running());
    assert!(!s.get(ScrollConstraint::scroll_active));
}
#[test]
fn inner_unused_axis_chains_outward() {
    let n = nested();
    assert_ne!(n.scene.send_at(n.over, wheel(0.0, -50.0)), HitResult::None);
    assert_eq!(get(&n.outer, ScrollConstraint::offset_y), -50.0);
    assert_eq!(get(&n.strip, ScrollConstraint::offset_x), 0.0);
}
#[test]
fn inner_view_takes_its_axis() {
    let n = nested();
    assert_ne!(n.scene.send_at(n.over, wheel(-50.0, 0.0)), HitResult::None);
    assert_eq!(get(&n.strip, ScrollConstraint::offset_x), -50.0);
    assert_eq!(get(&n.outer, ScrollConstraint::offset_y), 0.0);
}
#[test]
fn latched_inner_does_not_hand_tail_outward() {
    let n = nested();
    n.scene.send_at(n.over, wheel(-700.0, 0.0));
    assert_eq!(
        get(&n.strip, ScrollConstraint::offset_x),
        get(&n.strip, ScrollConstraint::max_offset_x)
    );
    assert!(!get(&n.strip, |c| c.can_consume(Vec2D::new(-100.0, 0.0))));
    n.scene.send_at(n.over, wheel(-100.0, -100.0));
    assert_eq!(get(&n.outer, ScrollConstraint::offset_y), 0.0);
    assert_eq!(
        get(&n.strip, ScrollConstraint::offset_x),
        get(&n.strip, ScrollConstraint::max_offset_x)
    );
    n.scene.idle();
    assert!(!n.scene.latch());
    n.scene.send_at(n.over, wheel(0.0, -50.0));
    assert_eq!(get(&n.outer, ScrollConstraint::offset_y), -50.0);
}
#[test]
fn no_end_gesture_closes_on_idle() {
    let s = Scene::vertical();
    s.send(trackpad(ScrollPhase::Begin, 0.0, 0.0));
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    assert!(s.get(ScrollConstraint::scroll_active));
    s.idle();
    assert!(!s.get(ScrollConstraint::scroll_active));
    assert!(!s.running());
}
#[test]
fn overscrolled_no_end_gesture_springs_back_on_idle() {
    let s = Scene::vertical();
    assert!(!s.get(|c| c.snap()));
    s.send(trackpad(ScrollPhase::Begin, 0.0, 0.0));
    s.send(trackpad(ScrollPhase::Update, 0.0, -900.0));
    assert!(s.get(ScrollConstraint::offset_y) < s.get(ScrollConstraint::max_offset_y));
    assert!(s.get(ScrollConstraint::is_overscrolled));
    let mut settling = false;
    for _ in 0..20 {
        if settling {
            break;
        }
        s.advance(0.016);
        settling = s.running();
    }
    assert!(settling);
    for _ in 0..300 {
        if !s.running() {
            break;
        }
        s.advance(0.016);
    }
    approx(
        s.get(ScrollConstraint::offset_y),
        s.get(ScrollConstraint::max_offset_y),
    );
    assert!(!s.get(ScrollConstraint::is_overscrolled));
}
#[test]
fn precise_gesture_stretches_elastic_edge() {
    let s = Scene::vertical();
    assert_eq!(s.get(ScrollConstraint::offset_y), 0.0);
    assert!(!s.get(|c| c.can_consume(Vec2D::new(0.0, 40.0))));
    assert_eq!(s.send(wheel(0.0, 40.0)), HitResult::None);
    assert_eq!(s.get(ScrollConstraint::offset_y), 0.0);
    assert!(s.get(|c| c.can_stretch(Vec2D::new(0.0, 40.0))));
    assert_ne!(
        s.send(trackpad(ScrollPhase::Update, 0.0, 40.0)),
        HitResult::None
    );
    assert!(s.get(ScrollConstraint::offset_y) > 0.0);
    assert!(s.get(ScrollConstraint::is_overscrolled));
    assert!(s.get(ScrollConstraint::clamped_offset_y) < s.get(ScrollConstraint::offset_y));
}
#[test]
fn settle_mid_gesture_does_not_pin_rendering() {
    let s = Scene::vertical();
    s.send(trackpad(ScrollPhase::Begin, 0.0, 0.0));
    s.send(trackpad(ScrollPhase::Update, 0.0, -900.0));
    s.send(trackpad(ScrollPhase::Momentum, 0.0, -20.0));
    assert!(s.running());
    for _ in 0..300 {
        if !s.running() {
            break;
        }
        s.advance(0.016);
        s.send(trackpad(ScrollPhase::Momentum, 0.0, -1.0));
    }
    assert!(s.get(ScrollConstraint::is_scrolling));
    assert!(!s.enabled());
    s.send(trackpad(ScrollPhase::Update, 0.0, -60.0));
    assert!(s.enabled());
    assert!(s.get(ScrollConstraint::clamped_offset_y) < s.get(ScrollConstraint::max_offset_y));
}
#[test]
fn interest_follows_drag_multiplier() {
    let s = Scene::vertical();
    assert!(CoreRegistry::set_double_handle(
        &s.scroll,
        ScrollConstraintBase::DRAG_MULTIPLIER_PROPERTY_KEY.into(),
        0.0
    ));
    assert!(!s.get(|c| c.can_consume(Vec2D::new(0.0, -50.0))));
    assert!(!s.get(|c| c.can_stretch(Vec2D::new(0.0, -50.0))));
    assert_eq!(s.send(wheel(0.0, -50.0)), HitResult::None);
    assert!(CoreRegistry::set_double_handle(
        &s.scroll,
        ScrollConstraintBase::DRAG_MULTIPLIER_PROPERTY_KEY.into(),
        -1.0
    ));
    assert_eq!(s.get(ScrollConstraint::offset_y), 0.0);
    assert!(s.get(|c| c.can_consume(Vec2D::new(0.0, 50.0))));
    assert!(!s.get(|c| c.can_consume(Vec2D::new(0.0, -50.0))));
    s.send(wheel(0.0, 50.0));
    assert_eq!(s.get(ScrollConstraint::offset_y), -50.0);
}
#[test]
fn no_begin_first_delta_does_not_derive_velocity() {
    let s = Scene::vertical();
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    assert_eq!(s.get(ScrollConstraint::offset_y), -40.0);
    assert_eq!(s.get(ScrollConstraint::velocity_y), 0.0);
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    assert_eq!(s.get(ScrollConstraint::offset_y), -80.0);
    assert_ne!(s.get(ScrollConstraint::velocity_y), 0.0);
}
#[test]
fn no_begin_drag_takes_control_from_fling() {
    let s = Scene::vertical();
    s.send(trackpad(ScrollPhase::Begin, 0.0, 0.0));
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    s.send(trackpad(ScrollPhase::End, 0.0, 0.0));
    assert!(s.running());
    s.advance(0.016);
    let before = s.get(ScrollConstraint::offset_y);
    s.send(trackpad(ScrollPhase::Update, 0.0, -25.0));
    assert!(!s.running());
    approx(s.get(ScrollConstraint::offset_y), before - 25.0);
    let dragged = s.get(ScrollConstraint::offset_y);
    s.advance(0.016);
    approx(s.get(ScrollConstraint::offset_y), dragged);
}
#[test]
fn zero_delta_begin_does_not_pick_target() {
    let n = nested();
    assert_eq!(
        n.scene
            .send_at(n.over, trackpad(ScrollPhase::Begin, 0.0, 0.0)),
        HitResult::None
    );
    assert!(!n.scene.latch());
    n.scene
        .send_at(n.over, trackpad(ScrollPhase::Update, 0.0, -50.0));
    assert_eq!(get(&n.outer, ScrollConstraint::offset_y), -50.0);
    assert_eq!(get(&n.strip, ScrollConstraint::offset_x), 0.0);
}
#[test]
fn clamped_view_keeps_accumulating_trackpad_velocity() {
    let s = Scene::open("layout/layout_scroll_vertical.riv");
    if let Some(previous) = s.get(ScrollConstraint::physics) {
        previous.remove_occurrence();
    }
    let physics = s
        .scroll
        .insert_sibling(ClampedScrollPhysics::default())
        .expect("scene retains the physics arena");
    s.scroll
        .with_downcast_mut::<ScrollConstraint, _>(|c| c.set_physics(physics))
        .unwrap();
    s.advance(0.0);
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    assert_eq!(s.get(ScrollConstraint::offset_y), -120.0);
    assert_ne!(s.get(ScrollConstraint::velocity_y), 0.0);
}
#[test]
fn wheel_idle_does_not_end_pointer_drag() {
    let s = Scene::vertical();
    let over = Vec2D::new(50.0, 250.0);
    s.send(wheel(0.0, -20.0));
    assert!(s.get(ScrollConstraint::is_scrolling));
    s.smi.with_instance_mut(|m| {
        m.pointer_move(over, 0.0, 0);
        m.pointer_down(
            over,
            0,
            nuxie_runtime::source::pointer_button::PointerButton::Primary,
        );
    });
    assert!(s.get(ScrollConstraint::is_dragging));
    s.idle();
    assert!(!s.get(ScrollConstraint::is_scrolling));
    assert!(s.get(ScrollConstraint::is_dragging));
    s.smi.with_instance_mut(|m| {
        m.pointer_up(
            over,
            0,
            nuxie_runtime::source::pointer_button::PointerButton::Primary,
        )
    });
    assert!(!s.get(ScrollConstraint::is_dragging));
}
#[test]
fn wheel_can_be_disabled_while_drag_works() {
    let s = Scene::vertical();
    let over = Vec2D::new(50.0, 250.0);
    assert!(s.get(|c| c.wheel_interactive()));
    assert!(CoreRegistry::set_bool_handle(
        &s.scroll,
        ScrollConstraintBase::WHEEL_INTERACTIVE_PROPERTY_KEY.into(),
        false
    ));
    assert!(s.get(ScrollConstraint::interactive));
    assert!(!s.smi.with_instance_mut(|m| m.has_scroll_target_at(over)));
    assert_eq!(s.send(wheel(0.0, -50.0)), HitResult::None);
    assert_eq!(s.get(ScrollConstraint::offset_y), 0.0);
    s.smi.with_instance_mut(|m| {
        m.pointer_move(over, 0.0, 0);
        m.pointer_down(
            over,
            0,
            nuxie_runtime::source::pointer_button::PointerButton::Primary,
        );
        m.pointer_move(Vec2D::new(50.0, 150.0), 0.0, 0);
    });
    assert!(s.get(ScrollConstraint::offset_y) < 0.0);
    s.smi.with_instance_mut(|m| {
        m.pointer_up(
            Vec2D::new(50.0, 150.0),
            0,
            nuxie_runtime::source::pointer_button::PointerButton::Primary,
        )
    });
}

#[test]
fn wheel_disabled_view_ignores_inertia_cancel() {
    let s = Scene::vertical();
    let over = Vec2D::new(50.0, 250.0);
    s.send(trackpad(ScrollPhase::Begin, 0.0, 0.0));
    s.send(trackpad(ScrollPhase::Update, 0.0, -40.0));
    s.send(trackpad(ScrollPhase::End, 0.0, 0.0));
    assert!(s.running());
    assert!(CoreRegistry::set_bool_handle(
        &s.scroll,
        ScrollConstraintBase::WHEEL_INTERACTIVE_PROPERTY_KEY.into(),
        false,
    ));
    let cancel = trackpad(ScrollPhase::InertiaCancel, 0.0, 0.0);
    assert!(!s.smi.with_instance_mut(|m| m.wants_scroll(over, &cancel)));
    assert_eq!(s.send_at(over, cancel), HitResult::None);
    assert!(s.running());
}
