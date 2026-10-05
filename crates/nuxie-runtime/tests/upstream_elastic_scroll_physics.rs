//! The three sections of "ElasticScrollPhysicsHelper only flings where there
//! is range" in upstream layout_scroll_test.cpp at 09e85e1f.

use nuxie_runtime::source::constraints::scrolling::elastic_scroll_physics::ElasticScrollPhysicsHelper;

// Furthest the content travels from the origin, and where it comes to rest.
fn release(range_min: f32, value: f32, settled: &mut f32) -> f32 {
    const FRICTION: f32 = 2.5;
    const SPEED_MUL: f32 = 1.2;
    const ELASTIC: f32 = 0.66;
    const ACCELERATION: f32 = -1_000_000.0;

    let mut helper = ElasticScrollPhysicsHelper::new(FRICTION, SPEED_MUL, ELASTIC);
    helper.run(ACCELERATION, range_min, 0.0, value, vec![], 0.0, 0.0);
    let mut peak = value.abs();
    *settled = value;
    for _ in 0..2000 {
        if !helper.is_running() {
            break;
        }
        *settled = helper.advance(1.0 / 60.0);
        peak = peak.max(settled.abs());
    }
    peak
}

#[test]
fn content_that_fits_never_moves_past_the_drag_itself() {
    let mut settled = 0.0;
    assert!(release(0.0, -3.0, &mut settled) <= 3.0 + 0.001);
    // Still bounces home: only the velocity phase is skipped.
    assert!((settled - 0.0).abs() <= 0.001);
}

#[test]
fn a_longer_bounce_still_returns_from_where_the_pointer_left_it() {
    let mut settled = 0.0;
    assert!(release(0.0, -12.0, &mut settled) <= 12.0 + 0.001);
    assert!((settled - 0.0).abs() <= 0.001);
}

#[test]
fn the_same_release_on_a_scrollable_axis_still_flings() {
    let mut settled = 0.0;
    assert!(release(-500.0, -3.0, &mut settled) > 100.0);
}

// layout_scroll_test.cpp additions at dc75beed.
#[test]
fn elastic_scroll_physics_snaps_each_looping_axis_by_its_own_cycle() {
    use nuxie_runtime::source::{
        constraints::{
            draggable_constraint::DraggableConstraintDirection,
            scrolling::elastic_scroll_physics::ElasticScrollPhysics,
        },
        math::vec2d::Vec2D,
    };
    let mut snaps = Vec::new();
    for x in [0.0, 110.0, 220.0, 330.0] {
        for y in [0.0, 70.0, 140.0, 210.0, 280.0] {
            snaps.push(Vec2D::new(x, y));
        }
    }
    let mut physics = ElasticScrollPhysics::default();
    physics.prepare(DraggableConstraintDirection::All);
    physics.run(
        Vec2D::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
        Vec2D::new(f32::INFINITY, f32::INFINITY),
        Vec2D::new(-1425.0, -768.0),
        snaps,
        Vec2D::new(440.0, 350.0),
        Vec2D::new(250.0, 200.0),
    );
    let mut last = Vec2D::default();
    for _ in 0..2000 {
        if !physics.is_running() {
            break;
        }
        last = physics.advance(0.016);
    }
    assert!(!physics.is_running());
    assert!((last.x - (-1430.0)).abs() <= 0.5);
    assert!((last.y - (-770.0)).abs() <= 0.5);
}
#[test]
fn elastic_scroll_physics_helper_shift_moves_far_end_with_it() {
    let mut helper = ElasticScrollPhysicsHelper::new(8.0, 1.0, 0.66);
    helper.run(-234375.0, -500.0, 0.0, -400.0, vec![], 0.0, 0.0);
    helper.shift(-50.0);
    let mut last = 0.0;
    for _ in 0..2000 {
        if !helper.is_running() {
            break;
        }
        last = helper.advance(0.016);
    }
    assert!(!helper.is_running());
    assert!(last < -510.0);
    assert!(last > -550.0);
}
