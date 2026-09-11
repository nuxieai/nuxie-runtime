//! Device-space, binary rectangle exclusion for the optional hard-clip API.
use nuxie_render_api::{Aabb, IntegerAabb, Mat2D, RawPath, Vec2D};

/// Apply a mask to a live source renderer. Its owning frame retains the context.
#[cfg(feature = "renderer-metal")]
pub(crate) fn apply(
    renderer: &mut crate::mechanical_port::source::renderer::include::rive::renderer::rive_renderer_hpp::RiveRenderer,
    rect: Aabb,
) -> bool {
    use crate::mechanical_port::source::include::rive::renderer_hpp::RendererContract;
    use crate::mechanical_port::source::renderer::include::rive::renderer::rive_renderer_hpp::RiveRenderer;
    use nuxie_render_api::FillRule;
    let state = renderer.current_state().clone();
    let Some(mut mask) =
        crate::hard_clip::difference_mask(rect, state.matrix, state.overallClipPixelBounds)
    else {
        return false;
    };
    let path = unsafe { &mut *renderer.m_context }
        .riveRenderFactoryMut()
        .makeRenderPathHandle(&mut mask, FillRule::NonZero);
    let Some(path) = path else {
        return false;
    };
    // Store the clip in device space without restoring the new clip away.
    // The source clip stack retains the path and captures this matrix.
    renderer.current_state_mut().matrix = Mat2D::IDENTITY;
    unsafe {
        <RiveRenderer as RendererContract>::clipPath(
            renderer,
            path.source_base() as *const _ as *mut _,
        );
    }
    renderer.current_state_mut().matrix = state.matrix;
    true
}

/// Build the remaining clip as integer-aligned device rectangles. Ordinary path
/// coverage is then binary at every pixel boundary. Work is bounded by the
/// current clip height, not by unbounded source coordinates.
pub(crate) fn difference_mask(rect: Aabb, matrix: Mat2D, bounds: IntegerAabb) -> Option<RawPath> {
    if matrix
        .0
        .iter()
        .chain([rect.min_x, rect.min_y, rect.max_x, rect.max_y].iter())
        .any(|v| !v.is_finite())
    {
        return None;
    }
    let mut path = RawPath::default();
    if bounds.empty() {
        return Some(path);
    }
    let outer = Aabb::new(
        bounds.left as f32,
        bounds.top as f32,
        bounds.right as f32,
        bounds.bottom as f32,
    );
    if rect.min_x >= rect.max_x || rect.min_y >= rect.max_y {
        path.add_rect(outer);
        return Some(path);
    }
    let mut corners = [
        Vec2D::new(rect.min_x, rect.min_y),
        Vec2D::new(rect.max_x, rect.min_y),
        Vec2D::new(rect.max_x, rect.max_y),
        Vec2D::new(rect.min_x, rect.max_y),
    ];
    matrix.map_points_in_place(&mut corners);
    if corners.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return None;
    }
    let low = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let high = corners
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);
    // Skia rounds non-AA scale/translate rectangles to the device lattice.
    // For general affine rectangles, intersect the convex edges at row centers.
    let first = ((f64::from(low) + 0.5).floor() as i32).clamp(bounds.top, bounds.bottom);
    let last = ((f64::from(high) + 0.5).floor() as i32).clamp(bounds.top, bounds.bottom);
    let mut add = |l: f32, t: f32, r: f32, b: f32| {
        if l < r && t < b {
            path.add_rect(Aabb::new(l, t, r, b));
        }
    };
    add(outer.min_x, outer.min_y, outer.max_x, first as f32);
    for y in first..last {
        let center = f64::from(y) + 0.5;
        let mut left = f64::INFINITY;
        let mut right = f64::NEG_INFINITY;
        for i in 0..4 {
            let a = corners[i];
            let b = corners[(i + 1) % 4];
            let (ay, by) = (f64::from(a.y), f64::from(b.y));
            if center > ay.min(by) && center <= ay.max(by) {
                let x = f64::from(a.x) + (center - ay) * f64::from(b.x - a.x) / (by - ay);
                left = left.min(x);
                right = right.max(x);
            }
        }
        let (l, r) = (
            ((left + 0.5).floor() as i32).clamp(bounds.left, bounds.right),
            ((right + 0.5).floor() as i32).clamp(bounds.left, bounds.right),
        );
        if l < r {
            add(outer.min_x, y as f32, l as f32, (y + 1) as f32);
            add(r as f32, y as f32, outer.max_x, (y + 1) as f32);
        } else {
            add(outer.min_x, y as f32, outer.max_x, (y + 1) as f32);
        }
    }
    add(outer.min_x, last as f32, outer.max_x, outer.max_y);
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn covered(path: &RawPath, x: i32, y: i32) -> bool {
        path.points().chunks_exact(4).any(|p| {
            let x = x as f32 + 0.5;
            let y = y as f32 + 0.5;
            x >= p[0].x && x < p[2].x && y >= p[0].y && y < p[2].y
        })
    }
    #[test]
    fn device_half_pixel_edges_round_after_transform() {
        let bounds = IntegerAabb::new(-4, -4, 8, 8);
        for edge in [0.49999997, 0.5, 0.50000006, -0.5] {
            let mask = difference_mask(
                Aabb::new(edge, edge, edge + 2.0, edge + 2.0),
                Mat2D::IDENTITY,
                bounds,
            )
            .unwrap();
            let start = (f64::from(edge) + 0.5).floor() as i32;
            let end = (f64::from(edge + 2.0) + 0.5).floor() as i32;
            for y in -4..8 {
                for x in -4..8 {
                    assert_eq!(
                        covered(&mask, x, y),
                        !(x >= start && x < end && y >= start && y < end),
                        "{edge}: {x},{y}"
                    );
                }
            }
        }
    }
    #[test]
    fn transformed_rect_matches_equivalent_device_rect_and_reflection() {
        let bounds = IntegerAabb::new(0, 0, 20, 20);
        let local = Aabb::new(0.25, 1.25, 4.25, 6.25);
        let matrix = Mat2D([2.0, 0.0, 0.0, -1.0, 0.25, 10.0]);
        let mapped = matrix.map_bounds(local);
        let a = difference_mask(local, matrix, bounds).unwrap();
        let b = difference_mask(mapped, Mat2D::IDENTITY, bounds).unwrap();
        for y in 0..20 {
            for x in 0..20 {
                assert_eq!(covered(&a, x, y), covered(&b, x, y));
            }
        }
    }
    #[test]
    fn shear_excludes_only_pixel_centers_inside_the_transformed_rectangle() {
        let bounds = IntegerAabb::new(0, 0, 12, 12);
        let mask = difference_mask(
            Aabb::new(2.0, 2.0, 6.0, 6.0),
            Mat2D([1.0, 0.0, 0.25, 1.0, 0.0, 0.0]),
            bounds,
        )
        .unwrap();
        for y in 0..12 {
            for x in 0..12 {
                let local_x = x as f32 + 0.5 - (y as f32 + 0.5) * 0.25;
                let excluded = (2..6).contains(&y) && (2.0..6.0).contains(&local_x);
                assert_eq!(covered(&mask, x, y), !excluded, "{x},{y}");
            }
        }
    }
    #[test]
    fn invalid_input_declines_and_empty_or_disjoint_rectangles_preserve_clip() {
        let bounds = IntegerAabb::new(0, 0, 4, 4);
        assert!(
            difference_mask(Aabb::new(f32::NAN, 0.0, 1.0, 1.0), Mat2D::IDENTITY, bounds).is_none()
        );
        for rect in [
            Aabb::new(2.0, 1.0, 2.0, 3.0),
            Aabb::new(10.0, 10.0, 12.0, 12.0),
        ] {
            let mask = difference_mask(rect, Mat2D::IDENTITY, bounds).unwrap();
            for y in 0..4 {
                for x in 0..4 {
                    assert!(covered(&mask, x, y));
                }
            }
        }
    }
}
