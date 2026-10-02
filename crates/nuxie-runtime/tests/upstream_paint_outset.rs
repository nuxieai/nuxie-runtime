//! All eight paint_outset_test.cpp cases, upstream 8398db31.
use nuxie_render_api::{StrokeCap, StrokeJoin, StrokeParams, paint_outset::*};
fn approx(a: f32, b: f32) {
    assert!((a - b).abs() <= 100.0 * f32::EPSILON * a.abs().max(b.abs()).max(1.0));
}
fn stroke(join: StrokeJoin, cap: StrokeCap) -> StrokeParams {
    StrokeParams {
        thickness: 10.0,
        join,
        cap,
        position: nuxie_render_api::StrokePosition::Center,
    }
}
#[test]
fn no_stroke_or_feather() {
    assert_eq!(paint_bounds_outset(None, 0.0), 0.0);
}
#[test]
fn stroke_half_thickness() {
    let mut s = stroke(StrokeJoin::Round, StrokeCap::Butt);
    approx(paint_bounds_outset(Some(&s), 0.0), 5.0);
    s.thickness = 0.0;
    assert_eq!(paint_bounds_outset(Some(&s), 0.0), 0.0);
}
#[test]
fn miter_limit() {
    let s = stroke(StrokeJoin::Miter, StrokeCap::Butt);
    approx(paint_bounds_outset(Some(&s), 0.0), 5.0 * K_MITER_LIMIT);
    approx(paint_bounds_outset(Some(&s), 0.0), 20.0);
}
#[test]
fn square_cap_diagonal() {
    let mut s = stroke(StrokeJoin::Round, StrokeCap::Square);
    approx(
        paint_bounds_outset(Some(&s), 0.0),
        5.0 * std::f32::consts::SQRT_2,
    );
    s.join = StrokeJoin::Bevel;
    approx(
        paint_bounds_outset(Some(&s), 0.0),
        5.0 * std::f32::consts::SQRT_2,
    );
}
#[test]
fn miter_outranks_square() {
    let s = stroke(StrokeJoin::Miter, StrokeCap::Square);
    approx(paint_bounds_outset(Some(&s), 0.0), 5.0 * K_MITER_LIMIT);
}
#[test]
fn feather_reaches_one_point_five() {
    approx(feather_radius_from_feather(10.0), 15.0);
    approx(paint_bounds_outset(None, 10.0), 15.0);
    assert_eq!(paint_bounds_outset(None, 0.0), 0.0);
}
#[test]
fn feather_adds_to_stroke() {
    let mut s = stroke(StrokeJoin::Round, StrokeCap::Butt);
    approx(paint_bounds_outset(Some(&s), 10.0), 5.0 + 15.0);
    s.join = StrokeJoin::Miter;
    approx(paint_bounds_outset(Some(&s), 10.0), 20.0 + 15.0);
}
#[test]
fn outset_never_shrinks() {
    for thickness in [0.0, 0.5, 1.0, 40.0] {
        for feather in [0.0, 0.25, 30.0] {
            for join in [StrokeJoin::Miter, StrokeJoin::Round, StrokeJoin::Bevel] {
                for cap in [StrokeCap::Butt, StrokeCap::Round, StrokeCap::Square] {
                    let s = StrokeParams {
                        thickness,
                        join,
                        cap,
                        position: nuxie_render_api::StrokePosition::Center,
                    };
                    let outset = paint_bounds_outset(Some(&s), feather);
                    assert!(outset >= 0.0);
                    assert!(
                        outset + 1e-4 >= thickness * 0.5 + feather_radius_from_feather(feather)
                    );
                }
            }
        }
    }
}

#[test]
fn outside_stroke_reaches_full_thickness() {
    let mut s = stroke(StrokeJoin::Round, StrokeCap::Butt);
    s.position = nuxie_render_api::StrokePosition::Outside;
    approx(paint_bounds_outset(Some(&s), 0.0), 10.0);
    s.join = StrokeJoin::Miter;
    approx(paint_bounds_outset(Some(&s), 0.0), 10.0 * K_MITER_LIMIT);
    approx(paint_bounds_outset(Some(&s), 0.0), 40.0);
    s.join = StrokeJoin::Bevel;
    s.cap = StrokeCap::Square;
    approx(
        paint_bounds_outset(Some(&s), 0.0),
        10.0 * std::f32::consts::SQRT_2,
    );
    s.position = nuxie_render_api::StrokePosition::Inside;
    s.join = StrokeJoin::Round;
    s.cap = StrokeCap::Butt;
    approx(paint_bounds_outset(Some(&s), 0.0), 5.0);
}

#[test]
fn shape_paint_outset_reads_stroke_position() {
    use nuxie_runtime::source::shapes::paint::{
        paint_outset::shape_paint_outset, shape_paint::ShapePaintBehavior, stroke::Stroke,
    };
    use nuxie_runtime::{File, RuntimeFactoryHandle};
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream");
    let bytes = std::fs::read(
        std::path::PathBuf::from(root).join("tests/unit_tests/assets/stroke_name_test.riv"),
    )
    .unwrap();
    let mut factory = nuxie_render_api::PersistentFactory::new(nuxie_render_api::NullFactory);
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    artboard.advance(
        0.0,
        nuxie_runtime::AdvanceFlags::ADVANCE_NESTED
            | nuxie_runtime::AdvanceFlags::ANIMATE
            | nuxie_runtime::AdvanceFlags::NEW_FRAME,
    );
    let stroke = artboard
        .with_artboard(|a| a.find_handle::<Stroke>("white_stroke"))
        .unwrap();
    stroke
        .with_downcast_mut::<Stroke, _>(|stroke| {
            assert!(stroke.should_draw());
            let mut params = StrokeParams {
                thickness: stroke.thickness(),
                join: nuxie_runtime::source::shapes::paint::stroke_join::StrokeJoin::from(
                    u32::from(stroke.join()),
                )
                .into(),
                cap: nuxie_runtime::source::shapes::paint::stroke_cap::StrokeCap::from(u32::from(
                    stroke.cap(),
                ))
                .into(),
                ..StrokeParams::default()
            };
            let centered_stroke = paint_bounds_outset(Some(&params), 0.0);
            params.position = nuxie_render_api::StrokePosition::Outside;
            let outside_stroke = paint_bounds_outset(Some(&params), 0.0);
            assert!(outside_stroke > centered_stroke);
            let centered = shape_paint_outset(Some(stroke)).outset;
            stroke.set_position(2);
            approx(
                shape_paint_outset(Some(stroke)).outset - centered,
                outside_stroke - centered_stroke,
            );
        })
        .unwrap();
}
