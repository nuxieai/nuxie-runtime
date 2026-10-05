//! Direct ports of all seven cases in pinned
//! `tests/unit_tests/runtime/contour_measure_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::math::{
    aabb::Aabb,
    contour_measure::{ContourMeasureIter, PosTan},
    mat2d::Mat2D,
    path_types::PathDirection,
    raw_path::RawPath,
    vec2d::Vec2D,
};
use nuxie_runtime::{File, RuntimeFactoryHandle};

fn nearly_eq(a: f32, b: f32, tolerance: f32) -> bool {
    assert!(tolerance >= 0.0);
    let diff = (a - b).abs();
    let max = a.abs().max(b.abs());
    let allowed = tolerance * max;
    if diff > allowed {
        eprintln!("{a} {b} delta {diff} allowed {allowed}");
        return false;
    }
    true
}

fn nearly_eq_point(a: Vec2D, b: Vec2D, tolerance: f32) -> bool {
    nearly_eq(a.x, b.x, tolerance) && nearly_eq(a.y, b.y, tolerance)
}

#[test]
fn contour_basics() {
    let tolerance = 0.000001_f32;
    let empty_path = RawPath::default();
    let mut contours = ContourMeasureIter::new(&empty_path, ContourMeasureIter::DEFAULT_TOLERANCE);
    assert!(contours.next().is_none());

    let mut move_path = RawPath::default();
    move_path.move_to(1.0, 2.0);
    contours.rewind(&move_path, ContourMeasureIter::DEFAULT_TOLERANCE);
    assert!(contours.next().is_none());

    let mut line_path = RawPath::default();
    line_path.move_to(1.0, 2.0);
    line_path.line_to(4.0, 6.0);
    contours.rewind(&line_path, ContourMeasureIter::DEFAULT_TOLERANCE);
    let contour = contours.next().expect("one measurable line contour");
    assert!(nearly_eq(contour.length(), 5.0, tolerance));
    assert!(contours.next().is_none());

    let width = 4.0;
    let height = 6.0;
    let mut rectangle_path = RawPath::default();
    rectangle_path.add_rect(Aabb::new(0.0, 0.0, width, height), PathDirection::Clockwise);
    contours.rewind(&rectangle_path, ContourMeasureIter::DEFAULT_TOLERANCE);
    let contour = contours.next().expect("one measurable rectangle contour");
    assert!(nearly_eq(
        contour.length(),
        2.0 * (width + height),
        tolerance
    ));

    let mid_distances = [
        width / 2.0,
        width + height / 2.0,
        width + height + width / 2.0,
        width + height + width + height / 2.0,
    ];
    let mid_points = [
        PosTan {
            pos: Vec2D::new(width / 2.0, 0.0),
            tan: Vec2D::new(1.0, 0.0),
        },
        PosTan {
            pos: Vec2D::new(width, height / 2.0),
            tan: Vec2D::new(0.0, 1.0),
        },
        PosTan {
            pos: Vec2D::new(width / 2.0, height),
            tan: Vec2D::new(-1.0, 0.0),
        },
        PosTan {
            pos: Vec2D::new(0.0, height / 2.0),
            tan: Vec2D::new(0.0, -1.0),
        },
    ];
    for (distance, expected) in mid_distances.into_iter().zip(mid_points) {
        let actual = contour.get_pos_tan(distance);
        assert!(nearly_eq_point(actual.pos, expected.pos, tolerance));
        assert!(nearly_eq_point(actual.tan, expected.tan, tolerance));
    }
    assert!(contours.next().is_none());
}

#[test]
fn multi_contours() {
    let points = [
        Vec2D::new(0.0, 0.0),
        Vec2D::new(3.0, 0.0),
        Vec2D::new(3.0, 4.0),
    ];
    // Three measurable contours: 7, 12, 7. All intervening contours have zero length.
    let mut path = RawPath::default();
    path.add_poly(&points, false);
    path.add_poly(&points, true);
    path.move_to(0.0, 0.0);
    path.move_to(0.0, 0.0);
    path.close();
    path.move_to(0.0, 0.0);
    path.line_to(0.0, 0.0);
    path.move_to(0.0, 0.0);
    path.line_to(0.0, 0.0);
    path.close();
    path.add_poly(&points, false);

    let mut contours = ContourMeasureIter::new(&path, ContourMeasureIter::DEFAULT_TOLERANCE);
    assert_eq!(contours.next().expect("first contour").length(), 7.0);
    assert_eq!(contours.next().expect("second contour").length(), 12.0);
    assert_eq!(contours.next().expect("third contour").length(), 7.0);
    assert!(contours.next().is_none());
}

#[test]
fn contour_oval() {
    let tolerance = 0.0075_f32;
    let radius = 10.0_f32;
    let mut path = RawPath::default();
    path.add_oval(
        Aabb::new(-radius, -radius, radius, radius),
        PathDirection::Clockwise,
    );
    let mut contours = ContourMeasureIter::new(&path, tolerance);
    let contour = contours.next().expect("one measurable oval contour");
    assert!(nearly_eq(
        contour.length(),
        2.0 * radius * std::f32::consts::PI,
        tolerance,
    ));
    assert!(contours.next().is_none());
}

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

#[test]
fn bad_contour() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("explicit retained factory");
    let file = File::import(
        &pinned_fixture("zombie_skins.riv"),
        factory,
        None,
        None,
        None,
    )
    .expect("zombie_skins.riv imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard instantiates");
    let machine = artboard
        .default_state_machine_handle()
        .expect("default state machine instantiates");
    machine.advance_and_apply(0.0);
}

#[test]
fn nan_path() {
    let mut path = RawPath::default();
    path.move_to(0.0, 0.0);
    path.line_to(1.0, 2.0);
    path.cubic_to(3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    path.cubic_to(9.0, 10.0, 11.0, 12.0, 13.0, 14.0);
    path.cubic_to(15.0, 16.0, 17.0, 18.0, 19.0, 20.0);
    let mut contours = ContourMeasureIter::new(&path, ContourMeasureIter::DEFAULT_TOLERANCE);
    let contour = contours.next().expect("one finite contour");
    assert!(contour.length().is_finite());
    assert!(contours.next().is_none());

    let nan = f32::NAN;
    let transformed = path.transform(Mat2D::new(nan, nan, nan, nan, nan, nan));
    let mut contours = ContourMeasureIter::new(&transformed, ContourMeasureIter::DEFAULT_TOLERANCE);
    assert!(contours.next().is_none());
}

#[test]
fn fuzz_issue_7295() {
    let mut inner_path = RawPath::default();
    inner_path.move_to(0.0, -20.5);
    inner_path.cubic_to(11.3218384, -20.5, 20.5, -11.3218384, 20.5, 0.0);
    inner_path.cubic_to(20.5, 11.3218384, 11.3218384, 20.5, 0.0, 20.5);
    inner_path.cubic_to(-11.3218384, 20.5, -20.5, 11.3218384, -20.5, 0.0);
    inner_path.cubic_to(-20.5, -11.3218384, -11.3218384, -20.5, 0.0, -20.5);
    let translate = -134_217_728.0_f32;
    let outer_path = inner_path.transform(Mat2D::from_translate(translate, translate));
    let contour = ContourMeasureIter::new(&outer_path, ContourMeasureIter::DEFAULT_TOLERANCE)
        .next()
        .expect("one transformed contour");
    let mut result = RawPath::default();
    contour.get_segment(0.0, 168.389008, &mut result, true);
    assert!((contour.length() - 168.389008).abs() <= 1.0 / 4096.0);
}

// A trim chop must preserve coincident controls exactly so the stroker does not
// interpret floating-point residue as a tangent.
#[test]
fn trim_keeps_coincident_control_points() {
    let p0 = Vec2D::new(244.19449, 186.5583);
    let p2 = Vec2D::new(160.0, 90.0);
    let p3 = Vec2D::new(330.0, 280.0);
    for i in 1..1000 {
        let mut path = RawPath::default();
        path.move_to_point(p0);
        path.cubic_to_points(p0, p2, p3);
        let mut iter = ContourMeasureIter::new(&path, ContourMeasureIter::DEFAULT_TOLERANCE);
        let cm = iter.next().expect("one measurable cubic contour");

        let d = cm.length() * i as f32 / 1000.0;
        let mut head = RawPath::default();
        cm.get_segment(0.0, d, &mut head, true);
        let hp = head.points();
        assert_eq!(hp.len(), 4);
        assert_eq!(hp[1], hp[0]);

        // Mirror case: coincident end control points on the tail.
        let mut rpath = RawPath::default();
        rpath.move_to_point(p3);
        rpath.cubic_to_points(p2, p0, p0);
        let mut riter = ContourMeasureIter::new(&rpath, ContourMeasureIter::DEFAULT_TOLERANCE);
        let rcm = riter.next().expect("one measurable reversed cubic contour");
        let mut tail = RawPath::default();
        rcm.get_segment(rcm.length() - d, rcm.length(), &mut tail, true);
        let tp = tail.points();
        assert_eq!(tp.len(), 4);
        assert_eq!(tp[2], tp[3]);
    }
}
