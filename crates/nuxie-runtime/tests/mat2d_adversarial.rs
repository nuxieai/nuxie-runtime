use nuxie_runtime::Mat2D;

/// Pinned `Mat2D::mapBoundingBox` evaluates its SIMD affine expression before
/// adding translation. This cancellation-heavy case distinguishes that order
/// from `mapPoints`, where translation participates in the inner expression.
#[test]
fn map_bounding_box_preserves_pinned_affine_grouping_bits() {
    let matrix = Mat2D([
        1.000_000_1,
        -std::f32::consts::PI,
        std::f32::consts::E,
        -f32::EPSILON,
        16_777_216.0,
        -16_777_216.0,
    ]);

    let actual = matrix.map_bounding_box(&[(16_777_215.0, -16_777_215.0)]);
    assert_eq!(
        [actual.0.to_bits(), actual.1.to_bits()],
        [0xcb37_e14c, 0xcc84_87ed],
        "expected bits captured from pinned C++ Mat2D::mapBoundingBox"
    );
}

/// Pinned SIMD lane reduction produces positive zero for this corner order.
#[test]
fn map_bounding_box_preserves_pinned_signed_zero_lane_order() {
    let actual = Mat2D::IDENTITY.map_bounding_box(&[(0.0, 0.0), (-0.0, -0.0)]);
    assert_eq!(
        [actual.0.to_bits(), actual.1.to_bits()],
        [0x0000_0000, 0x0000_0000],
        "expected bits captured from pinned C++ SIMD min reduction"
    );
}
