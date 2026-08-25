// Complete direct port of pinned `tests/unit_tests/runtime/aabb_test.cpp`, plus
// focused source-authority coverage for operations absent from that test file.

use nuxie_render_api::{Aabb, Vec2D};
use nuxie_runtime::{FloatAabb, IntegerAabb, SemanticBounds, TypedAabb};

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
        IntegerAabb::new(1, -2, 99, 101).intersect(IntegerAabb::new(0, 0, 100, 100)),
        IntegerAabb::new(1, 0, 99, 100)
    );
    assert_eq!(
        IntegerAabb::new(1, -2, 99, 101).intersect(IntegerAabb::new(2, -3, 98, 103)),
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

#[test]
fn taabb_complete_cross_type_surface_matches_pinned_integer_contracts() {
    let signed = TypedAabb::<i32>::new(-2, 3, 12, 19);
    assert_eq!((signed.width(), signed.height()), (14, 16));
    assert_eq!(signed.inset(2, 3), TypedAabb::new(0, 6, 10, 16));
    assert_eq!(signed.outset(2, 3), TypedAabb::new(-4, 0, 14, 22));
    assert_eq!(signed.offset(4, -2), TypedAabb::new(2, 1, 16, 17));

    let unsigned = TypedAabb::<u16>::new(0, 4, 20, 30);
    assert_eq!(signed.intersect(unsigned), TypedAabb::new(0, 4, 12, 19),);
    assert_eq!(
        TypedAabb::<i16>::new(-10, -10, -1, -1).intersect_or_empty(unsigned),
        TypedAabb::default(),
    );
    assert!(TypedAabb::<i32>::new(0, 0, 20, 30).contains(unsigned));
    assert!(TypedAabb::<i32>::new(0, 4, 20, 30).equals(TypedAabb::<u16>::new(0, 4, 20, 30)));
    assert!(TypedAabb::<i32>::new(-1, 0, 1, 1).overlaps(TypedAabb::<u64>::new(0, 0, 2, 2,)));

    assert_eq!(
        TypedAabb::<i16>::new(-1, 0, i16::MAX, 7).clamp_cast::<u16>(),
        TypedAabb::new(0, 0, i16::MAX as u16, 7),
    );
    assert_eq!(
        TypedAabb::<u16>::new(1, 2, 3, 4).lossless_numeric_cast::<i32>(),
        Some(TypedAabb::new(1, 2, 3, 4)),
    );
    assert_eq!(
        TypedAabb::<i32>::new(-1, 2, 3, 4).lossless_numeric_cast::<u16>(),
        None,
    );
    assert_eq!(
        TypedAabb::<i32>::make_wh(7_u16, 9_u16),
        Some(TypedAabb::new(0, 0, 7, 9)),
    );
}

#[test]
fn float_aabb_complete_surface_preserves_pinned_grouping_and_order() {
    let bounds = FloatAabb::from_min_max(Vec2D::new(1.25, 2.5), Vec2D::new(8.75, 12.5));
    assert_eq!(bounds.min(), Vec2D::new(1.25, 2.5));
    assert_eq!(bounds.max(), Vec2D::new(8.75, 12.5));
    assert_eq!(bounds.size(), Vec2D::new(7.5, 10.0));
    assert_eq!(bounds.center(), Vec2D::new(5.0, 7.5));
    assert_eq!(bounds.pad(1.0), FloatAabb::new(0.25, 1.5, 9.75, 13.5));
    assert_eq!(
        bounds.inset(1.0, 2.0),
        FloatAabb::new(2.25, 4.5, 7.75, 10.5)
    );
    assert_eq!(
        bounds.offset(-1.0, 3.0),
        FloatAabb::new(0.25, 5.5, 7.75, 15.5)
    );
    assert_eq!(bounds.corner(0), Some(Vec2D::new(1.25, 2.5)));
    assert_eq!(bounds.corner(1), Some(Vec2D::new(8.75, 12.5)));
    assert_eq!(bounds.corner(2), None);
    assert_eq!(bounds.round(), IntegerAabb::new(1, 3, 9, 13));
    assert_eq!(bounds.round_out(), IntegerAabb::new(1, 2, 9, 13));
    assert_eq!(
        FloatAabb::from_integer(IntegerAabb::new(1, 2, 9, 13)),
        FloatAabb::new(1.0, 2.0, 9.0, 13.0),
    );
    assert!(bounds.contains(Vec2D::new(1.25, 12.5)));
    assert!(bounds.overlaps(FloatAabb::new(8.0, 12.0, 20.0, 20.0)));

    let points = FloatAabb::from_points(&[
        Vec2D::new(2.0, 3.0),
        Vec2D::new(-1.0, 8.0),
        Vec2D::new(4.0, -5.0),
    ]);
    assert_eq!(points, FloatAabb::new(-1.0, -5.0, 4.0, 8.0));
    assert_eq!(FloatAabb::from_points(&[]), FloatAabb::default());

    let first_zero = FloatAabb::new(0.0, 0.0, 0.0, 0.0);
    let second_zero = FloatAabb::new(-0.0, -0.0, -0.0, -0.0);
    let joined_zero = FloatAabb::join(first_zero, second_zero);
    assert_eq!(joined_zero.min_x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(joined_zero.max_x.to_bits(), 0.0_f32.to_bits());

    let first_nan = FloatAabb::new(f32::NAN, 0.0, f32::NAN, 1.0);
    let joined_nan = FloatAabb::join(first_nan, FloatAabb::new(2.0, 2.0, 3.0, 3.0));
    assert!(joined_nan.min_x.is_nan());
    assert!(joined_nan.max_x.is_nan());

    let mut expansion = FloatAabb::for_expansion();
    expansion.expand_to(Vec2D::new(f32::NAN, f32::NAN));
    assert_eq!(expansion, FloatAabb::for_expansion());
    expansion.expand(FloatAabb::new(0.0, 0.0, 0.0, 0.0));
    assert_eq!(expansion, FloatAabb::new(0.0, 0.0, 0.0, 0.0));

    let factor = FloatAabb::new(1.0, 2.0, 1.0, 2.0).factor_from(Vec2D::new(1.0, 2.0));
    assert_eq!(factor.x, 0.0);
    assert!(factor.y.is_nan());
}

#[test]
fn semantic_bounds_expand_uses_the_shared_pinned_join_owner() {
    let mut bounds = SemanticBounds::for_expansion();
    bounds.expand(SemanticBounds::new(0.0, 0.0, 0.0, 0.0));
    assert_eq!(bounds, SemanticBounds::new(0.0, 0.0, 0.0, 0.0));

    let mut signed_zero = SemanticBounds::new(0.0, 0.0, 0.0, 0.0);
    signed_zero.expand(SemanticBounds::new(-0.0, -0.0, -0.0, -0.0));
    assert_eq!(signed_zero.min_x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(signed_zero.max_x.to_bits(), 0.0_f32.to_bits());
}
