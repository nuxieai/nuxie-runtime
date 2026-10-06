use nuxie::render_api::{Aabb, Fit, Mat2D, Vec2D, compute_alignment};

pub(super) fn layout_transform(
    bounds: (f32, f32, f32, f32),
    viewport: (u32, u32),
    scale: f32,
) -> Result<Mat2D, &'static str> {
    let (x, y, width, height) = bounds;
    if ![x, y, width, height, scale].into_iter().all(f32::is_finite)
        || width <= 0.0
        || height <= 0.0
        || scale <= 0.0
        || viewport.0 == 0
        || viewport.1 == 0
    {
        return Err("layout bounds, surface dimensions and scale must be finite and positive");
    }
    let transform = compute_alignment(
        Fit::Layout,
        Vec2D::new(0.0, 0.0),
        Aabb::new(0.0, 0.0, viewport.0 as f32, viewport.1 as f32),
        Aabb::new(x, y, x + width, y + height),
        scale,
    );
    if !transform.0.into_iter().all(f32::is_finite) {
        return Err("layout transform must be finite");
    }
    Ok(transform)
}
