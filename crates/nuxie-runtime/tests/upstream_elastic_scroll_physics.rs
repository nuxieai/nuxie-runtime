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
