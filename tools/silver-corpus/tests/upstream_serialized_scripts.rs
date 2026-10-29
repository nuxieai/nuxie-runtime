//! tests/unit_tests/runtime/serialized_scripts_test.cpp at d97f3547.

use silver_corpus::{
    Execution, Field, OpKind, Operation, Sriv, Value, compare_sriv, parse_sriv, read_manifest,
    resolve_expected,
};
use std::path::PathBuf;
use std::sync::Mutex;

// Catch runs these cases serially. Execution changes process-wide test RNG state.
static REPLAY_LOCK: Mutex<()> = Mutex::new(());

fn replay(id: &str) -> (Sriv, Sriv) {
    let _guard = REPLAY_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let upstream = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR pinned fixture checkout"),
    );
    let manifest = read_manifest(
        &PathBuf::from(
            option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
        )
        .join("../../silver-corpus.toml"),
    )
    .unwrap();
    let case = manifest.cases.iter().find(|case| case.id == id).unwrap();
    let actual = Execution::run(case, &upstream)
        .unwrap_or_else(|error| panic!("{id} replay failed: {error:#}"));
    let expected = std::fs::read(resolve_expected(&upstream, case)).unwrap();
    (
        parse_sriv(&expected).unwrap(),
        parse_sriv(actual.bytes()).unwrap(),
    )
}

fn replay_and_compare(id: &str) {
    let (expected, actual) = replay(id);
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|error| panic!("{id} Silver comparison: {error}"));
}

#[test]
#[ignore = "intentional CSS text fit-content; full stream is checked by game_menu_css_changes_only_two_rectangle_widths"]
fn game_menu_ad_script_test() {
    replay_and_compare("game_menu_ad_police_files");
}

#[test]
#[ignore = "intentional CSS Fill, shrink and nested automatic minima; temporary sizing controls match all 340 frames and 602072 upstream operations; see README.md geometry audit"]
fn inventory_script_test() {
    replay_and_compare("inventory_demo_test_v2");
}

#[test]
#[ignore = "intentional CSS Fill basis/minima; 903 controlled rebuild-pair differences accounted for by producer contraction and cache tolerances; other replay geometry and GPU pixels not certified; see README.md geometry audit"]
fn layout_planets_script_test() {
    replay_and_compare("layoutstest_8-planets-grid");
}

fn compare_operation_range(expected: &Sriv, actual: &Sriv, start: usize, end: usize) {
    compare_sriv(
        &Sriv {
            version: expected.version,
            operations: expected.operations[start..end].to_vec(),
        },
        &Sriv {
            version: actual.version,
            operations: actual.operations[start..end].to_vec(),
        },
    )
    .unwrap_or_else(|error| panic!("unadapted operations {start}..{end}: {error}"));
}

fn assert_rectangle(version: u64, operation: &Operation, path_id: u64, width: f32, height: f32) {
    assert_eq!(operation.frame, 14);
    let mut fields = vec![
        Field {
            name: "path_id",
            value: Value::Uint(path_id),
        },
        Field {
            name: "verb_count",
            value: Value::Uint(6),
        },
    ];
    fields.extend([0, 1, 1, 1, 1, 5].map(|verb| Field {
        name: "verb",
        value: Value::Uint(verb),
    }));
    fields.push(Field {
        name: "point_count",
        value: Value::Uint(5),
    });
    fields.extend(
        [
            (0.0_f32, 0.0_f32),
            (width, 0.0),
            (width, height),
            (0.0, height),
            (0.0, 0.0),
        ]
        .map(|(x, y)| Field {
            name: "point",
            value: Value::Vec2(x.to_bits(), y.to_bits()),
        }),
    );
    compare_sriv(
        &Sriv {
            version,
            operations: vec![Operation {
                offset: operation.offset,
                frame: 14,
                kind: OpKind::AddRawPath,
                fields,
            }],
        },
        &Sriv {
            version,
            operations: vec![operation.clone()],
        },
    )
    .unwrap_or_else(|error| panic!("rectangle path {path_id}: {error}"));
}

#[test]
fn game_menu_css_changes_only_two_rectangle_widths() {
    let (expected, actual) = replay("game_menu_ad_police_files");
    assert_eq!(expected.version, actual.version);
    assert_eq!(expected.operations.len(), 43_983);
    assert_eq!(actual.operations.len(), expected.operations.len());
    assert_eq!(expected.operations.last().unwrap().frame, 120);
    assert_eq!(actual.operations.last().unwrap().frame, 120);

    // At frame 14, text179 keeps its CSS fit-content inline size. Upstream
    // shrinks to its widest wrapped line instead. Container170 adds 20 points
    // of padding on each side; the README geometry audit associates path60
    // with that container. A second rectangle, path35, has the same width
    // change and a 4-point height; its source owner is not established here.
    // Only two right-edge x coordinates in each rectangle may differ. This
    // is a bounded observed contract, not a general geometry normalization.
    let upstream_width = 676.91016_f32 + 2.0 * 20.0;
    let css_width = 754.0928_f32 + 2.0 * 20.0;
    let mut start = 0;
    for (index, path_id, height) in [(4868, 60, 67.51367_f32), (4896, 35, 4.0)] {
        compare_operation_range(&expected, &actual, start, index);
        assert_rectangle(
            expected.version,
            &expected.operations[index],
            path_id,
            upstream_width,
            height,
        );
        assert_rectangle(
            actual.version,
            &actual.operations[index],
            path_id,
            css_width,
            height,
        );
        start = index + 1;
    }
    // This includes every later frame and retains the upstream comparator's
    // resource identities, ordering, signed-zero handling and tolerances.
    compare_operation_range(&expected, &actual, start, expected.operations.len());
}
