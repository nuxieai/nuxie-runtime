// Complete direct port of pinned `tests/unit_tests/runtime/aabb_test.cpp`.
// Missing integer/join/intersection owners remain explicit expected-red tests;
// the literal vectors are retained here for the source-correspondence phase.

use nuxie_render_api::{Aabb, Vec2D};
use nuxie_runtime::SemanticBounds;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntegerAabb(i32, i32, i32, i32);

fn missing_integer_aabb_owner(_: &str, _: IntegerAabb, _: IntegerAabb) -> IntegerAabb {
    panic!("Rust runtime has no primary integer AABB owner yet")
}

fn missing_integer_aabb_predicate(_: &str, _: IntegerAabb, _: IntegerAabb) -> bool {
    panic!("Rust runtime has no primary integer AABB owner yet")
}

fn missing_float_aabb_overlap(_: Aabb, _: Aabb) -> bool {
    panic!("Rust runtime Aabb does not expose the pinned overlap operation yet")
}

#[test]
#[ignore = "expected red: source correspondence must supply the integer AABB join owner"]
fn iaabb_join_direct_port() {
    assert_eq!(
        missing_integer_aabb_owner(
            "join",
            IntegerAabb(1, -2, 99, 101),
            IntegerAabb(0, 0, 100, 100),
        ),
        IntegerAabb(0, -2, 100, 101)
    );
    assert_eq!(
        missing_integer_aabb_owner(
            "join",
            IntegerAabb(1, -2, 99, 101),
            IntegerAabb(2, -3, 98, 103),
        ),
        IntegerAabb(1, -3, 99, 103)
    );
}

#[test]
#[ignore = "expected red: source correspondence must supply the integer AABB intersection owner"]
fn iaabb_intersect_direct_port() {
    assert_eq!(
        missing_integer_aabb_owner(
            "intersect",
            IntegerAabb(1, -2, 99, 101),
            IntegerAabb(0, 0, 100, 100),
        ),
        IntegerAabb(1, 0, 99, 100)
    );
    assert_eq!(
        missing_integer_aabb_owner(
            "intersect",
            IntegerAabb(1, -2, 99, 101),
            IntegerAabb(2, -3, 98, 103),
        ),
        IntegerAabb(2, -2, 98, 101)
    );
}

#[test]
#[ignore = "expected red: source correspondence must supply the integer AABB empty owner"]
fn iaabb_empty_direct_port() {
    let cases = [
        (IntegerAabb(0, 0, 0, 0), true),
        (IntegerAabb(0, 0, 0, 1), true),
        (IntegerAabb(0, 0, 1, 0), true),
        (IntegerAabb(0, 0, 1, 1), false),
        (IntegerAabb(0, 0, -1, -1), true),
        (IntegerAabb(i32::MAX, i32::MAX, i32::MIN, i32::MIN), true),
    ];
    for (bounds, expected) in cases {
        assert_eq!(
            missing_integer_aabb_predicate("empty", bounds, bounds),
            expected
        );
    }
}

#[test]
fn is_empty_or_nan_direct_port() {
    let infinity = f32::INFINITY;
    let nan = f32::NAN;
    let cases = [
        (SemanticBounds::new(0.0, 0.0, 1.0, 1.0), false),
        (
            SemanticBounds::new(-infinity, -infinity, infinity, infinity),
            false,
        ),
        (SemanticBounds::new(0.0, 0.0, 0.0, 0.0), true),
        (SemanticBounds::new(0.0, 0.0, -1.0, -2.0), true),
        (
            SemanticBounds::new(infinity, infinity, -infinity, -infinity),
            true,
        ),
        (
            SemanticBounds::new(infinity, -infinity, -infinity, infinity),
            true,
        ),
        (
            SemanticBounds::new(-infinity, infinity, infinity, -infinity),
            true,
        ),
        (SemanticBounds::new(nan, 0.0, 10.0, 10.0), true),
        (SemanticBounds::new(0.0, nan, 10.0, 10.0), true),
        (SemanticBounds::new(0.0, 0.0, nan, 10.0), true),
        (SemanticBounds::new(0.0, 0.0, 10.0, nan), true),
        (SemanticBounds::new(nan, nan, 10.0, 10.0), true),
        (SemanticBounds::new(nan, nan, nan, 10.0), true),
        (SemanticBounds::new(nan, nan, nan, nan), true),
    ];
    for (bounds, expected) in cases {
        assert_eq!(bounds.is_empty_or_nan(), expected);
    }
}

#[test]
fn aabb_contains_direct_port() {
    let bounds = Aabb::new(0.0, 0.0, 100.0, 100.0);
    assert!(bounds.contains(Vec2D::new(20.0, 20.0)));
    assert!(bounds.contains(Vec2D::new(0.0, 0.0)));
    assert!(bounds.contains(Vec2D::new(100.0, 100.0)));
    assert!(!bounds.contains(Vec2D::new(200.0, 200.0)));
    assert!(!bounds.contains(Vec2D::new(-200.0, -200.0)));
    assert!(!bounds.contains(Vec2D::new(-f32::EPSILON, 50.0)));
    assert!(!bounds.contains(Vec2D::new(100.0 + 100.0 * f32::EPSILON, 50.0)));
}

const OVERLAP_CASES: [(IntegerAabb, bool); 18] = [
    (IntegerAabb(10, 10, 90, 90), true),
    (IntegerAabb(0, 0, 100, 100), true),
    (IntegerAabb(-1000, 10, 90, 90), true),
    (IntegerAabb(10, -1000, 90, 90), true),
    (IntegerAabb(10, 10, 1000, 90), true),
    (IntegerAabb(10, 10, 90, 1000), true),
    (IntegerAabb(-1000, -1000, 1000, 90), true),
    (IntegerAabb(-1000, -1000, 90, 1000), true),
    (IntegerAabb(-1000, 10, 1000, 1000), true),
    (IntegerAabb(10, -1000, 1000, 1000), true),
    (IntegerAabb(110, 10, 190, 90), false),
    (IntegerAabb(10, 110, 90, 190), false),
    (IntegerAabb(-110, 10, -10, 90), false),
    (IntegerAabb(10, -110, 90, -10), false),
    (IntegerAabb(-10, 10, 0, 90), false),
    (IntegerAabb(10, -10, 90, 0), false),
    (IntegerAabb(100, 10, 190, 90), false),
    (IntegerAabb(10, 100, 190, 90), false),
];

#[test]
#[ignore = "expected red: source correspondence must supply the integer AABB overlap owner"]
fn iaabb_overlaps_direct_port() {
    let bounds = IntegerAabb(0, 0, 100, 100);
    for (other, expected) in OVERLAP_CASES {
        assert_eq!(
            missing_integer_aabb_predicate("overlaps", bounds, other),
            expected
        );
    }
}

#[test]
#[ignore = "expected red: source correspondence must expose float AABB overlap"]
fn aabb_overlaps_direct_port() {
    let bounds = Aabb::new(0.0, 0.0, 100.0, 100.0);
    for (other, expected) in OVERLAP_CASES {
        assert_eq!(
            missing_float_aabb_overlap(
                bounds,
                Aabb::new(
                    other.0 as f32,
                    other.1 as f32,
                    other.2 as f32,
                    other.3 as f32,
                ),
            ),
            expected
        );
    }
}

fn missing_generic_aabb_owner<T>(_: &str) -> (T, T, T, T) {
    panic!("Rust runtime has no primary generic AABB owner yet")
}

macro_rules! assert_maximal {
    ($($ty:ty),+ $(,)?) => {
        $(assert_eq!(
            missing_generic_aabb_owner::<$ty>("makeMaximal"),
            (<$ty>::MIN, <$ty>::MIN, <$ty>::MAX, <$ty>::MAX),
        );)+
    };
}

macro_rules! assert_maximally_negative {
    ($($ty:ty),+ $(,)?) => {
        $(assert_eq!(
            missing_generic_aabb_owner::<$ty>("makeMaximallyNegative"),
            (<$ty>::MAX, <$ty>::MAX, <$ty>::MIN, <$ty>::MIN),
        );)+
    };
}

#[test]
#[ignore = "expected red: source correspondence must supply the generic AABB maximal owner"]
fn taabb_make_maximal_direct_port() {
    assert_maximal!(i16, u16, i32, u32, i64, u64);
}

#[test]
#[ignore = "expected red: source correspondence must supply the generic AABB negative owner"]
fn taabb_make_maximally_negative_direct_port() {
    assert_maximally_negative!(i16, u16, i32, u32, i64, u64);
}
