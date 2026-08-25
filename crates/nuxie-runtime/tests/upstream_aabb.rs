// Complete direct port of pinned `tests/unit_tests/runtime/aabb_test.cpp`.
// Missing integer/join/intersection owners remain explicit expected-red tests;
// the literal vectors are retained here for the source-correspondence phase.

use nuxie_render_api::{Aabb, Vec2D};
use nuxie_runtime::{IntegerAabb, SemanticBounds, TypedAabb};

#[test]
fn iaabb_join_direct_port() {
    assert_eq!(
        IntegerAabb::new(1, -2, 99, 101).join(IntegerAabb::new(0, 0, 100, 100)),
        IntegerAabb::new(0, -2, 100, 101)
    );
    assert_eq!(
        IntegerAabb::new(1, -2, 99, 101).join(IntegerAabb::new(2, -3, 98, 103)),
        IntegerAabb::new(1, -3, 99, 103)
    );
}

#[test]
fn iaabb_intersect_direct_port() {
    assert_eq!(
        IntegerAabb::new(1, -2, 99, 101)
            .intersect(IntegerAabb::new(0, 0, 100, 100)),
        IntegerAabb::new(1, 0, 99, 100)
    );
    assert_eq!(
        IntegerAabb::new(1, -2, 99, 101)
            .intersect(IntegerAabb::new(2, -3, 98, 103)),
        IntegerAabb::new(2, -2, 98, 101)
    );
}

#[test]
fn iaabb_empty_direct_port() {
    let cases = [
        (IntegerAabb::new(0, 0, 0, 0), true),
        (IntegerAabb::new(0, 0, 0, 1), true),
        (IntegerAabb::new(0, 0, 1, 0), true),
        (IntegerAabb::new(0, 0, 1, 1), false),
        (IntegerAabb::new(0, 0, -1, -1), true),
        (
            IntegerAabb::new(i32::MAX, i32::MAX, i32::MIN, i32::MIN),
            true,
        ),
    ];
    for (bounds, expected) in cases {
        assert_eq!(bounds.empty(), expected);
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
    (IntegerAabb::new(10, 10, 90, 90), true),
    (IntegerAabb::new(0, 0, 100, 100), true),
    (IntegerAabb::new(-1000, 10, 90, 90), true),
    (IntegerAabb::new(10, -1000, 90, 90), true),
    (IntegerAabb::new(10, 10, 1000, 90), true),
    (IntegerAabb::new(10, 10, 90, 1000), true),
    (IntegerAabb::new(-1000, -1000, 1000, 90), true),
    (IntegerAabb::new(-1000, -1000, 90, 1000), true),
    (IntegerAabb::new(-1000, 10, 1000, 1000), true),
    (IntegerAabb::new(10, -1000, 1000, 1000), true),
    (IntegerAabb::new(110, 10, 190, 90), false),
    (IntegerAabb::new(10, 110, 90, 190), false),
    (IntegerAabb::new(-110, 10, -10, 90), false),
    (IntegerAabb::new(10, -110, 90, -10), false),
    (IntegerAabb::new(-10, 10, 0, 90), false),
    (IntegerAabb::new(10, -10, 90, 0), false),
    (IntegerAabb::new(100, 10, 190, 90), false),
    (IntegerAabb::new(10, 100, 190, 90), false),
];

#[test]
fn iaabb_overlaps_direct_port() {
    let bounds = IntegerAabb::new(0, 0, 100, 100);
    for (other, expected) in OVERLAP_CASES {
        assert_eq!(bounds.overlaps(other), expected);
    }
}

#[test]
fn aabb_overlaps_direct_port() {
    let bounds = Aabb::new(0.0, 0.0, 100.0, 100.0);
    for (other, expected) in OVERLAP_CASES {
        assert_eq!(
            bounds.overlaps(Aabb::new(
                other.left as f32,
                other.top as f32,
                other.right as f32,
                other.bottom as f32,
            )),
            expected
        );
    }
}

macro_rules! assert_maximal {
    ($($ty:ty),+ $(,)?) => {
        $(assert_eq!(
            TypedAabb::<$ty>::make_maximal(),
            TypedAabb::new(<$ty>::MIN, <$ty>::MIN, <$ty>::MAX, <$ty>::MAX),
        );)+
    };
}

macro_rules! assert_maximally_negative {
    ($($ty:ty),+ $(,)?) => {
        $(assert_eq!(
            TypedAabb::<$ty>::make_maximally_negative(),
            TypedAabb::new(<$ty>::MAX, <$ty>::MAX, <$ty>::MIN, <$ty>::MIN),
        );)+
    };
}

#[test]
fn taabb_make_maximal_direct_port() {
    assert_maximal!(i16, u16, i32, u32, i64, u64);
}

#[test]
fn taabb_make_maximally_negative_direct_port() {
    assert_maximally_negative!(i16, u16, i32, u32, i64, u64);
}
