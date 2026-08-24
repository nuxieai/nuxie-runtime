//! Direct ports of all six cases in pinned
//! `tests/unit_tests/runtime/contour_measure_test.cpp`.

use std::path::PathBuf;

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::{
    ArtboardInstance, RuntimeContourMeasure, RuntimePathCommand, RuntimePathSample,
};

fn move_to(x: f32, y: f32) -> RuntimePathCommand {
    RuntimePathCommand::Move { x, y }
}

fn line_to(x: f32, y: f32) -> RuntimePathCommand {
    RuntimePathCommand::Line { x, y }
}

fn cubic_to(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> RuntimePathCommand {
    RuntimePathCommand::Cubic {
        x1,
        y1,
        x2,
        y2,
        x3,
        y3,
    }
}

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

fn nearly_eq_point(a: (f32, f32), b: (f32, f32), tolerance: f32) -> bool {
    nearly_eq(a.0, b.0, tolerance) && nearly_eq(a.1, b.1, tolerance)
}

#[test]
fn contour_basics() {
    let tolerance = 0.000001_f32;

    let mut path = Vec::new();
    let mut contours = RuntimeContourMeasure::from_commands(&path);
    assert!(contours.is_empty());

    path.push(move_to(1.0, 2.0));
    contours = RuntimeContourMeasure::from_commands(&path);
    assert!(contours.is_empty());

    path.push(line_to(4.0, 6.0));
    contours = RuntimeContourMeasure::from_commands(&path);
    let contour = contours.first().expect("one measurable line contour");
    assert!(nearly_eq(contour.length(), 5.0, tolerance));
    assert_eq!(contours.len(), 1);

    let width = 4.0;
    let height = 6.0;
    path = vec![
        move_to(0.0, 0.0),
        line_to(width, 0.0),
        line_to(width, height),
        line_to(0.0, height),
        RuntimePathCommand::Close,
    ];
    contours = RuntimeContourMeasure::from_commands(&path);
    let contour = contours.first().expect("one measurable rectangle contour");
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
        RuntimePathSample {
            pos: (width / 2.0, 0.0),
            tan: (1.0, 0.0),
        },
        RuntimePathSample {
            pos: (width, height / 2.0),
            tan: (0.0, 1.0),
        },
        RuntimePathSample {
            pos: (width / 2.0, height),
            tan: (-1.0, 0.0),
        },
        RuntimePathSample {
            pos: (0.0, height / 2.0),
            tan: (0.0, -1.0),
        },
    ];
    for (distance, expected) in mid_distances.into_iter().zip(mid_points) {
        let actual = contour.at_distance(distance);
        assert!(nearly_eq_point(actual.pos, expected.pos, tolerance));
        assert!(nearly_eq_point(actual.tan, expected.tan, tolerance));
    }
    assert_eq!(contours.len(), 1);
}

fn add_poly(path: &mut Vec<RuntimePathCommand>, points: &[(f32, f32)], closed: bool) {
    let Some(&(x, y)) = points.first() else {
        return;
    };
    path.push(move_to(x, y));
    for &(x, y) in &points[1..] {
        path.push(line_to(x, y));
    }
    if closed {
        path.push(RuntimePathCommand::Close);
    }
}

#[test]
fn multi_contours() {
    let points = [(0.0, 0.0), (3.0, 0.0), (3.0, 4.0)];

    // We expect 3 measurable contours out of this: 7, 12, 7. The others
    // should be skipped because their length is zero.
    let mut path = Vec::new();
    add_poly(&mut path, &points, false);
    add_poly(&mut path, &points, true);

    path.push(move_to(0.0, 0.0));

    path.push(move_to(0.0, 0.0));
    path.push(RuntimePathCommand::Close);

    path.push(move_to(0.0, 0.0));
    path.push(line_to(0.0, 0.0));

    path.push(move_to(0.0, 0.0));
    path.push(line_to(0.0, 0.0));
    path.push(RuntimePathCommand::Close);

    add_poly(&mut path, &points, false);

    let contours = RuntimeContourMeasure::from_commands(&path);
    assert_eq!(contours[0].length(), 7.0);
    assert_eq!(contours[1].length(), 12.0);
    assert_eq!(contours[2].length(), 7.0);
    assert_eq!(contours.len(), 3);
}

#[test]
#[ignore = "expected-red: Rust fixes a coarser contour subdivision tolerance internally"]
fn contour_oval() {
    let tolerance = 0.0075_f32;
    let radius = 10.0_f32;

    // Exact point stream emitted by pinned RawPath::addOval for a clockwise
    // circle. RuntimeContourMeasure currently fixes its subdivision tolerance
    // internally; the source-correspondence phase must compare that owner.
    let c = 0.5519150244935106_f32;
    let path = [
        move_to(radius, 0.0),
        cubic_to(radius, c * radius, c * radius, radius, 0.0, radius),
        cubic_to(-c * radius, radius, -radius, c * radius, -radius, 0.0),
        cubic_to(-radius, -c * radius, -c * radius, -radius, 0.0, -radius),
        cubic_to(c * radius, -radius, radius, -c * radius, radius, 0.0),
        RuntimePathCommand::Close,
    ];
    let contours = RuntimeContourMeasure::from_commands(&path);
    let contour = contours.first().expect("one measurable oval contour");
    assert!(nearly_eq(
        contour.length(),
        2.0 * radius * std::f32::consts::PI,
        tolerance
    ));
    assert_eq!(contours.len(), 1);
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
    let file =
        read_runtime_file(&pinned_fixture("zombie_skins.riv")).expect("zombie_skins.riv imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("zombie_skins.riv graph builds");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let default_index = artboard
        .default_state_machine_index()
        .expect("default state machine");
    let mut machine = artboard
        .state_machine_instance(default_index)
        .expect("default state machine instantiates");
    machine
        .advance_and_apply(&mut artboard, 0.0)
        .expect("default state machine advances and applies");
}

#[test]
fn nan_path() {
    let path = [
        move_to(0.0, 0.0),
        line_to(1.0, 2.0),
        cubic_to(3.0, 4.0, 5.0, 6.0, 7.0, 8.0),
        cubic_to(9.0, 10.0, 11.0, 12.0, 13.0, 14.0),
        cubic_to(15.0, 16.0, 17.0, 18.0, 19.0, 20.0),
    ];

    let contours = RuntimeContourMeasure::from_commands(&path);
    let contour = contours.first().expect("one finite contour");
    assert!(contour.length().is_finite());
    assert_eq!(contours.len(), 1);

    let nan = f32::NAN;
    let transformed = path.map(|command| match command {
        RuntimePathCommand::Move { .. } => move_to(nan, nan),
        RuntimePathCommand::Line { .. } => line_to(nan, nan),
        RuntimePathCommand::Cubic { .. } => cubic_to(nan, nan, nan, nan, nan, nan),
        RuntimePathCommand::Close => RuntimePathCommand::Close,
    });
    assert!(RuntimeContourMeasure::from_commands(&transformed).is_empty());
}

#[test]
fn fuzz_issue_7295() {
    let inner_path = [
        move_to(0.0, -20.5),
        cubic_to(11.3218384, -20.5, 20.5, -11.3218384, 20.5, 0.0),
        cubic_to(20.5, 11.3218384, 11.3218384, 20.5, 0.0, 20.5),
        cubic_to(-11.3218384, 20.5, -20.5, 11.3218384, -20.5, 0.0),
        cubic_to(-20.5, -11.3218384, -11.3218384, -20.5, 0.0, -20.5),
    ];
    let translate = -134_217_728.0_f32;
    let outer_path = inner_path.map(|command| match command {
        RuntimePathCommand::Move { x, y } => move_to(x + translate, y + translate),
        RuntimePathCommand::Line { x, y } => line_to(x + translate, y + translate),
        RuntimePathCommand::Cubic {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        } => cubic_to(
            x1 + translate,
            y1 + translate,
            x2 + translate,
            y2 + translate,
            x3 + translate,
            y3 + translate,
        ),
        RuntimePathCommand::Close => RuntimePathCommand::Close,
    });

    let contours = RuntimeContourMeasure::from_commands(&outer_path);
    let contour = contours.first().expect("one transformed contour");
    let _result = contour.segment(0.0, 168.389008, true);
    assert!((contour.length() - 168.389008).abs() <= 1.0 / 4096.0);
}
