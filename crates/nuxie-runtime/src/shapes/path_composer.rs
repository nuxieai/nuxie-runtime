//! Embedded PathComposer callback owner. The draw coordinator invokes its
//! dependency-ordered update to rebuild local, local-clockwise, and world
//! ShapePaintPaths; drawing never reconstructs those paths.

pub(crate) fn needs_clockwise_reversal(
    determinant: f32,
    designed_clockwise: bool,
    is_hole: bool,
) -> bool {
    let winding = if designed_clockwise { 1.0 } else { -1.0 };
    (determinant * winding < 0.0) != is_hole
}
