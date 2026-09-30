//! All 13 offscreen_raster_test.cpp cases, upstream 8398db31.
use nuxie_runtime::source::{math::aabb::Aabb, offscreen_raster::*};
fn plan(scale: f32) -> RasterPlan {
    RasterPlan {
        raster_scale: scale,
        ..Default::default()
    }
}
fn approx(a: f32, b: f32) {
    assert!(
        (a - b).abs() <= 100.0 * f32::EPSILON * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}
fn whole(x: f32, s: f32) -> bool {
    (x * s - (x * s).round()).abs() < 1e-3
}
#[test]
fn exact_fit_keeps_historical_sizing() {
    let mut p = plan(2.0);
    assert!(fit_raster_to_box(
        &Aabb::new(0.0, 0.0, 100.0, 50.0),
        0,
        RasterFit::Exact,
        &mut p
    ));
    assert_eq!((p.width_px, p.height_px), (200, 100));
    assert_eq!(p.bounds, Aabb::new(0.0, 0.0, 100.0, 50.0));
    assert_eq!(p.raster_scale, 2.0);
    let mut p = plan(1.0);
    assert!(fit_raster_to_box(
        &Aabb::new(0.0, 0.0, 100.5, 50.0),
        0,
        RasterFit::Exact,
        &mut p
    ));
    assert_eq!(p.width_px, 101);
    assert_eq!(p.bounds.right(), 100.5);
}
#[test]
fn degenerate_box_refused() {
    let mut p = plan(1.0);
    for b in [
        Aabb::new(0.0, 0.0, 0.0, 50.0),
        Aabb::new(0.0, 0.0, 50.0, 0.0),
        Aabb::new(60.0, 0.0, 50.0, 50.0),
    ] {
        assert!(!fit_raster_to_box(&b, 0, RasterFit::Exact, &mut p));
    }
}
#[test]
fn max_dim_coarsens_scale_uniformly() {
    let mut p = plan(2.0);
    assert!(fit_raster_to_box(
        &Aabb::new(0.0, 0.0, 4000.0, 1000.0),
        0,
        RasterFit::Exact,
        &mut p
    ));
    approx(p.raster_scale, 0.512);
    assert_eq!((p.width_px, p.height_px), (2048, 512));
    assert!(p.raster_scale < 2.0);
}
#[test]
fn bucket_ladder_multiples_of_64() {
    for (n, b) in [
        (1, 64),
        (64, 64),
        (65, 128),
        (200, 256),
        (256, 256),
        (257, 320),
        (2047, 2048),
        (2048, K_MAX_DIM),
        (5000, K_MAX_DIM),
    ] {
        assert_eq!(bucketed_raster_dim(n), b);
    }
    for n in 1..=2048 {
        let b = bucketed_raster_dim(n);
        assert!(b >= n);
        assert!(b - n < 64);
    }
}
#[test]
fn stable_grid_maps_pixel_span_exactly() {
    for left in [0.0, 10.3, -7.9, 123.456] {
        for scale in [1.0, 2.0, 1.5, 3.0] {
            let mut p = plan(scale);
            let b = Aabb::new(left, left, left + 100.0, left + 50.0);
            assert!(fit_raster_to_box(&b, 2, RasterFit::StableGrid, &mut p));
            approx(p.bounds.width() * p.raster_scale, p.width_px as f32);
            approx(p.bounds.height() * p.raster_scale, p.height_px as f32);
            assert!(p.bounds.left() <= b.left());
            assert!(p.bounds.top() <= b.top());
            assert!(p.bounds.right() >= b.right());
            assert!(p.bounds.bottom() >= b.bottom());
        }
    }
}
#[test]
fn stable_grid_snaps_origin() {
    let mut p = plan(2.0);
    assert!(fit_raster_to_box(
        &Aabb::new(10.3, 4.6, 110.3, 54.6),
        0,
        RasterFit::StableGrid,
        &mut p
    ));
    approx(p.bounds.left(), 10.0);
    approx(p.bounds.top(), 4.5);
    assert!(whole(p.bounds.left(), p.raster_scale));
    assert!(whole(p.bounds.top(), p.raster_scale));
}
#[test]
fn translating_box_holds_allocation_and_phase() {
    let mut first = 0;
    let mut fractional = false;
    for step in 0..40 {
        let x = 5.0 + step as f32 * 0.1;
        let mut p = plan(2.0);
        assert!(fit_raster_to_box(
            &Aabb::new(x, 0.0, x + 100.0, 50.0),
            2,
            RasterFit::StableGrid,
            &mut p
        ));
        if first == 0 {
            first = p.width_px;
        }
        assert_eq!(p.width_px, first);
        assert_eq!(p.height_px, bucketed_raster_dim(104));
        assert!(whole(p.bounds.left(), 2.0));
        let phase = (x - p.bounds.left()) * 2.0;
        if (phase - phase.round()).abs() > 1e-3 {
            fractional = true;
        }
    }
    assert!(fractional);
}
#[test]
fn guard_band_adds_margin() {
    let b = Aabb::new(0.0, 0.0, 100.0, 100.0);
    let mut bare = plan(1.0);
    assert!(fit_raster_to_box(&b, 0, RasterFit::StableGrid, &mut bare));
    let mut guarded = plan(1.0);
    assert!(fit_raster_to_box(
        &b,
        2,
        RasterFit::StableGrid,
        &mut guarded
    ));
    approx(guarded.bounds.left(), bare.bounds.left() - 2.0);
    approx(guarded.bounds.top(), bare.bounds.top() - 2.0);
    assert!(guarded.bounds.left() < 0.0);
    assert!(guarded.bounds.right() > 100.0);
}
#[test]
fn guard_band_cannot_exceed_max() {
    let mut p = plan(8.0);
    assert!(fit_raster_to_box(
        &Aabb::new(0.0, 0.0, 4000.0, 4000.0),
        2,
        RasterFit::StableGrid,
        &mut p
    ));
    assert!(p.width_px <= K_MAX_DIM);
    assert!(p.height_px <= K_MAX_DIM);
    approx(p.bounds.width() * p.raster_scale, p.width_px as f32);
}
#[test]
fn hold_allocation_grows_right_bottom_only() {
    let mut p = plan(2.0);
    assert!(fit_raster_to_box(
        &Aabb::new(10.0, 10.0, 110.0, 60.0),
        0,
        RasterFit::StableGrid,
        &mut p
    ));
    let (l, t) = (p.bounds.left(), p.bounds.top());
    hold_raster_allocation(1024, 512, &mut p);
    assert_eq!((p.width_px, p.height_px), (1024, 512));
    assert_eq!((p.bounds.left(), p.bounds.top()), (l, t));
    approx(p.bounds.width() * p.raster_scale, 1024.0);
    approx(p.bounds.height() * p.raster_scale, 512.0);
    hold_raster_allocation(64, 64, &mut p);
    assert_eq!((p.width_px, p.height_px), (1024, 512));
}
#[test]
fn resize_grows_immediately() {
    let mut streak = 0;
    let first = decide_raster_resize(200, 100, 0, 0, &mut streak);
    assert!(first.reallocate);
    assert_eq!((first.width_px, first.height_px), (256, 128));
    let steady = decide_raster_resize(250, 120, 256, 128, &mut streak);
    assert!(!steady.reallocate);
    assert_eq!((steady.width_px, steady.height_px), (256, 128));
    let grow = decide_raster_resize(300, 120, 256, 128, &mut streak);
    assert!(grow.reallocate);
    assert_eq!((grow.width_px, grow.height_px), (320, 128));
    streak = 0;
    let one = decide_raster_resize(300, 10, 256, 512, &mut streak);
    assert!(one.reallocate);
    assert_eq!((one.width_px, one.height_px), (320, 512));
}
#[test]
fn shrink_waits_streak() {
    let mut streak = 0;
    for _ in 1..K_SHRINK_DRAWS {
        let r = decide_raster_resize(100, 100, 512, 512, &mut streak);
        assert!(!r.reallocate);
        assert_eq!(r.width_px, 512);
    }
    let r = decide_raster_resize(100, 100, 512, 512, &mut streak);
    assert!(r.reallocate);
    assert_eq!((r.width_px, r.height_px), (128, 128));
    assert_eq!(streak, 0);
}
#[test]
fn full_frame_resets_streak() {
    let mut streak = 0;
    for _ in 0..5 {
        for _ in 0..K_SHRINK_DRAWS - 1 {
            assert!(!decide_raster_resize(100, 100, 512, 512, &mut streak).reallocate);
        }
        assert!(!decide_raster_resize(500, 500, 512, 512, &mut streak).reallocate);
        assert_eq!(streak, 0);
    }
}
