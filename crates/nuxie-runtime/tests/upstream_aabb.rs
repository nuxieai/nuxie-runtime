// Complete direct port of pinned `tests/unit_tests/runtime/aabb_test.cpp`, plus
// focused source-authority coverage for operations absent from that test file.

use nuxie_runtime::source::math::{
    aabb::{Aabb, IAabb, TAabb},
    vec2d::Vec2D,
};

#[test]
fn iaabb_join_direct_port() {
    assert_eq!(
        IAabb {
            left: 1,
            top: -2,
            right: 99,
            bottom: 101
        }
        .join(IAabb {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100
        }),
        IAabb {
            left: 0,
            top: -2,
            right: 100,
            bottom: 101
        }
    );
    assert_eq!(
        IAabb {
            left: 1,
            top: -2,
            right: 99,
            bottom: 101
        }
        .join(IAabb {
            left: 2,
            top: -3,
            right: 98,
            bottom: 103
        }),
        IAabb {
            left: 1,
            top: -3,
            right: 99,
            bottom: 103
        }
    );
}

#[test]
fn iaabb_intersect_direct_port() {
    assert_eq!(
        IAabb {
            left: 1,
            top: -2,
            right: 99,
            bottom: 101
        }
        .intersect(IAabb {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100
        }),
        IAabb {
            left: 1,
            top: 0,
            right: 99,
            bottom: 100
        }
    );
    assert_eq!(
        IAabb {
            left: 1,
            top: -2,
            right: 99,
            bottom: 101
        }
        .intersect(IAabb {
            left: 2,
            top: -3,
            right: 98,
            bottom: 103
        }),
        IAabb {
            left: 2,
            top: -2,
            right: 98,
            bottom: 101
        }
    );
}

#[test]
fn iaabb_empty_direct_port() {
    let cases = [
        (
            IAabb {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            true,
        ),
        (
            IAabb {
                left: 0,
                top: 0,
                right: 0,
                bottom: 1,
            },
            true,
        ),
        (
            IAabb {
                left: 0,
                top: 0,
                right: 1,
                bottom: 0,
            },
            true,
        ),
        (
            IAabb {
                left: 0,
                top: 0,
                right: 1,
                bottom: 1,
            },
            false,
        ),
        (
            IAabb {
                left: 0,
                top: 0,
                right: -1,
                bottom: -1,
            },
            true,
        ),
        (
            IAabb {
                left: i32::MAX,
                top: i32::MAX,
                right: i32::MIN,
                bottom: i32::MIN,
            },
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
        (Aabb::new(0.0, 0.0, 1.0, 1.0), false),
        (Aabb::new(-infinity, -infinity, infinity, infinity), false),
        (Aabb::new(0.0, 0.0, 0.0, 0.0), true),
        (Aabb::new(0.0, 0.0, -1.0, -2.0), true),
        (Aabb::new(infinity, infinity, -infinity, -infinity), true),
        (Aabb::new(infinity, -infinity, -infinity, infinity), true),
        (Aabb::new(-infinity, infinity, infinity, -infinity), true),
        (Aabb::new(nan, 0.0, 10.0, 10.0), true),
        (Aabb::new(0.0, nan, 10.0, 10.0), true),
        (Aabb::new(0.0, 0.0, nan, 10.0), true),
        (Aabb::new(0.0, 0.0, 10.0, nan), true),
        (Aabb::new(nan, nan, 10.0, 10.0), true),
        (Aabb::new(nan, nan, nan, 10.0), true),
        (Aabb::new(nan, nan, nan, nan), true),
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

const OVERLAP_CASES: [(IAabb, bool); 18] = [
    (
        IAabb {
            left: 10,
            top: 10,
            right: 90,
            bottom: 90,
        },
        true,
    ),
    (
        IAabb {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100,
        },
        true,
    ),
    (
        IAabb {
            left: -1000,
            top: 10,
            right: 90,
            bottom: 90,
        },
        true,
    ),
    (
        IAabb {
            left: 10,
            top: -1000,
            right: 90,
            bottom: 90,
        },
        true,
    ),
    (
        IAabb {
            left: 10,
            top: 10,
            right: 1000,
            bottom: 90,
        },
        true,
    ),
    (
        IAabb {
            left: 10,
            top: 10,
            right: 90,
            bottom: 1000,
        },
        true,
    ),
    (
        IAabb {
            left: -1000,
            top: -1000,
            right: 1000,
            bottom: 90,
        },
        true,
    ),
    (
        IAabb {
            left: -1000,
            top: -1000,
            right: 90,
            bottom: 1000,
        },
        true,
    ),
    (
        IAabb {
            left: -1000,
            top: 10,
            right: 1000,
            bottom: 1000,
        },
        true,
    ),
    (
        IAabb {
            left: 10,
            top: -1000,
            right: 1000,
            bottom: 1000,
        },
        true,
    ),
    (
        IAabb {
            left: 110,
            top: 10,
            right: 190,
            bottom: 90,
        },
        false,
    ),
    (
        IAabb {
            left: 10,
            top: 110,
            right: 90,
            bottom: 190,
        },
        false,
    ),
    (
        IAabb {
            left: -110,
            top: 10,
            right: -10,
            bottom: 90,
        },
        false,
    ),
    (
        IAabb {
            left: 10,
            top: -110,
            right: 90,
            bottom: -10,
        },
        false,
    ),
    (
        IAabb {
            left: -10,
            top: 10,
            right: 0,
            bottom: 90,
        },
        false,
    ),
    (
        IAabb {
            left: 10,
            top: -10,
            right: 90,
            bottom: 0,
        },
        false,
    ),
    (
        IAabb {
            left: 100,
            top: 10,
            right: 190,
            bottom: 90,
        },
        false,
    ),
    (
        IAabb {
            left: 10,
            top: 100,
            right: 190,
            bottom: 90,
        },
        false,
    ),
];

#[test]
fn iaabb_overlaps_direct_port() {
    let bounds = IAabb {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
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
            TAabb::<$ty>::make_maximal(),
            TAabb { left: <$ty>::MIN, top: <$ty>::MIN, right: <$ty>::MAX, bottom: <$ty>::MAX },
        );)+
    };
}

macro_rules! assert_maximally_negative {
    ($($ty:ty),+ $(,)?) => {
        $(assert_eq!(
            TAabb::<$ty>::make_maximally_negative(),
            TAabb { left: <$ty>::MAX, top: <$ty>::MAX, right: <$ty>::MIN, bottom: <$ty>::MIN },
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
    let signed = TAabb::<i32> {
        left: -2,
        top: 3,
        right: 12,
        bottom: 19,
    };
    assert_eq!((signed.width(), signed.height()), (14, 16));
    assert_eq!(
        signed.inset(2, 3),
        TAabb {
            left: 0,
            top: 6,
            right: 10,
            bottom: 16
        }
    );
    assert_eq!(
        signed.outset(2, 3),
        TAabb {
            left: -4,
            top: 0,
            right: 14,
            bottom: 22
        }
    );
    assert_eq!(
        signed.offset(4, -2),
        TAabb {
            left: 2,
            top: 1,
            right: 16,
            bottom: 17
        }
    );

    let unsigned = TAabb::<u16> {
        left: 0,
        top: 4,
        right: 20,
        bottom: 30,
    };
    assert_eq!(
        signed.intersect(unsigned),
        TAabb {
            left: 0,
            top: 4,
            right: 12,
            bottom: 19
        },
    );
    assert_eq!(
        TAabb::<i16> {
            left: -10,
            top: -10,
            right: -1,
            bottom: -1
        }
        .intersect_or_empty(unsigned),
        TAabb::<i16>::default(),
    );
    assert!(
        TAabb::<i32> {
            left: 0,
            top: 0,
            right: 20,
            bottom: 30
        }
        .contains(unsigned)
    );
    assert!(
        TAabb::<i32> {
            left: 0,
            top: 4,
            right: 20,
            bottom: 30
        } == (TAabb::<u16> {
            left: 0,
            top: 4,
            right: 20,
            bottom: 30
        })
    );
    assert!(
        TAabb::<i32> {
            left: -1,
            top: 0,
            right: 1,
            bottom: 1
        }
        .overlaps(TAabb::<u64> {
            left: 0,
            top: 0,
            right: 2,
            bottom: 2
        })
    );

    assert_eq!(
        TAabb::<i16> {
            left: -1,
            top: 0,
            right: i16::MAX,
            bottom: 7
        }
        .clamp_cast::<u16>(),
        TAabb {
            left: 0,
            top: 0,
            right: i16::MAX as u16,
            bottom: 7
        },
    );
    assert_eq!(
        TAabb::<u16> {
            left: 1,
            top: 2,
            right: 3,
            bottom: 4
        }
        .lossless_numeric_cast::<i32>(),
        TAabb {
            left: 1,
            top: 2,
            right: 3,
            bottom: 4
        },
    );
    assert!(
        std::panic::catch_unwind(|| {
            TAabb::<i32> {
                left: -1,
                top: 2,
                right: 3,
                bottom: 4,
            }
            .lossless_numeric_cast::<u16>()
        })
        .is_err()
    );
    assert_eq!(
        TAabb::<i32>::make_wh(7_u16, 9_u16),
        TAabb {
            left: 0,
            top: 0,
            right: 7,
            bottom: 9
        },
    );
}

#[test]
fn float_aabb_complete_surface_preserves_pinned_grouping_and_order() {
    let bounds = Aabb::from_min_max(Vec2D::new(1.25, 2.5), Vec2D::new(8.75, 12.5));
    assert_eq!(bounds.min(), Vec2D::new(1.25, 2.5));
    assert_eq!(bounds.max(), Vec2D::new(8.75, 12.5));
    assert_eq!(bounds.size(), Vec2D::new(7.5, 10.0));
    assert_eq!(bounds.center(), Vec2D::new(5.0, 7.5));
    assert_eq!(bounds.pad(1.0), Aabb::new(0.25, 1.5, 9.75, 13.5));
    assert_eq!(bounds.inset(1.0, 2.0), Aabb::new(2.25, 4.5, 7.75, 10.5));
    assert_eq!(bounds.offset(-1.0, 3.0), Aabb::new(0.25, 5.5, 7.75, 15.5));
    assert_eq!(bounds.corner(0), Vec2D::new(1.25, 2.5));
    assert_eq!(bounds.corner(1), Vec2D::new(8.75, 12.5));
    assert!(std::panic::catch_unwind(|| bounds.corner(2)).is_err());
    assert_eq!(
        bounds.round(),
        IAabb {
            left: 1,
            top: 3,
            right: 9,
            bottom: 13
        }
    );
    assert_eq!(
        bounds.round_out(),
        IAabb {
            left: 1,
            top: 2,
            right: 9,
            bottom: 13
        }
    );
    assert_eq!(
        Aabb::from_iaabb(IAabb {
            left: 1,
            top: 2,
            right: 9,
            bottom: 13
        }),
        Aabb::new(1.0, 2.0, 9.0, 13.0),
    );
    assert!(bounds.contains(Vec2D::new(1.25, 12.5)));
    assert!(bounds.overlaps(Aabb::new(8.0, 12.0, 20.0, 20.0)));

    let points = Aabb::from_points(&[
        Vec2D::new(2.0, 3.0),
        Vec2D::new(-1.0, 8.0),
        Vec2D::new(4.0, -5.0),
    ]);
    assert_eq!(points, Aabb::new(-1.0, -5.0, 4.0, 8.0));
    assert_eq!(Aabb::from_points(&[]), Aabb::default());

    let first_zero = Aabb::new(0.0, 0.0, 0.0, 0.0);
    let second_zero = Aabb::new(-0.0, -0.0, -0.0, -0.0);
    let mut joined_zero = Aabb::default();
    Aabb::join(&mut joined_zero, first_zero, second_zero);
    assert_eq!(joined_zero.min_x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(joined_zero.max_x.to_bits(), 0.0_f32.to_bits());

    let first_nan = Aabb::new(f32::NAN, 0.0, f32::NAN, 1.0);
    let mut joined_nan = Aabb::default();
    Aabb::join(&mut joined_nan, first_nan, Aabb::new(2.0, 2.0, 3.0, 3.0));
    assert!(joined_nan.min_x.is_nan());
    assert!(joined_nan.max_x.is_nan());

    let mut expansion = Aabb::for_expansion();
    Aabb::expand_to_point(&mut expansion, Vec2D::new(f32::NAN, f32::NAN));
    assert_eq!(expansion, Aabb::for_expansion());
    expansion.expand(Aabb::new(0.0, 0.0, 0.0, 0.0));
    assert_eq!(expansion, Aabb::new(0.0, 0.0, 0.0, 0.0));

    let factor = Aabb::new(1.0, 2.0, 1.0, 2.0).factor_from(Vec2D::new(1.0, 2.0));
    assert_eq!(factor.x, 0.0);
    assert!(factor.y.is_nan());
}

#[test]
fn semantic_bounds_expand_uses_the_shared_pinned_join_owner() {
    let mut bounds = Aabb::for_expansion();
    bounds.expand(Aabb::new(0.0, 0.0, 0.0, 0.0));
    assert_eq!(bounds, Aabb::new(0.0, 0.0, 0.0, 0.0));

    let mut signed_zero = Aabb::new(0.0, 0.0, 0.0, 0.0);
    signed_zero.expand(Aabb::new(-0.0, -0.0, -0.0, -0.0));
    assert_eq!(signed_zero.min_x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(signed_zero.max_x.to_bits(), 0.0_f32.to_bits());
}
