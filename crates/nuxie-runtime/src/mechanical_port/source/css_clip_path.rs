//! Paint-only elliptical clipping geometry for the pinned Chromium profile.
use super::math::{aabb::Aabb, raw_path::RawPath, vec2d::Vec2D};

/// Border paint between the outer and inner rounded contours. The caller must
/// use an even-odd fill, so the inner contour is a hole even for translucent
/// colors. Edges are left/top/right/bottom and must be solved used widths.
pub(crate) fn border_ring_path(
    path: &mut RawPath,
    bounds: Aabb,
    circular_radii: [f32; 4],
    borders: [f32; 4],
) -> bool {
    elliptical_border_ring_path(path, bounds, circular_radii.map(|r| [r, r]), borders)
}

pub(crate) fn elliptical_border_ring_path(
    path: &mut RawPath,
    bounds: Aabb,
    radii: [[f32; 2]; 4],
    borders: [f32; 4],
) -> bool {
    if !borders.into_iter().all(|edge| edge.is_finite() && edge >= 0.) {
        return false;
    }
    let mut outer = RawPath::default();
    let mut inner = RawPath::default();
    if !elliptical_outset_path(&mut outer, bounds, radii, [0.; 4])
        || !elliptical_outset_path(&mut inner, bounds, radii, borders.map(|edge| -edge)) {
        return false;
    }
    path.rewind();
    if borders == [0.; 4] { return true; }
    path.add_path(&outer, None);
    path.add_path(&inner, None);
    true
}

/// Experimental side partition for clipping the shared rounded border ring.
/// Sides are top/right/bottom/left; used widths are left/top/right/bottom.
/// This is not a public paint policy until Chrome pixel qualification passes.
pub(crate) fn border_side_clip_path(path: &mut RawPath, bounds: Aabb, circular_radii: [f32; 4], borders: [f32; 4], side: usize) -> bool {
    elliptical_border_side_clip_path(path, bounds, circular_radii.map(|r| [r, r]), borders, side)
}

pub(crate) fn elliptical_border_side_clip_path(path: &mut RawPath, bounds: Aabb, mut radii: [[f32; 2]; 4], borders: [f32; 4], side: usize) -> bool {
    if side >= 4 || ![bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y,
        bounds.width(), bounds.height()].into_iter().all(f32::is_finite)
        || !borders.into_iter().all(|v| v.is_finite() && v >= 0.)
        || !radii.iter().flatten().all(|v| v.is_finite() && *v >= 0.) { return false; }
    path.rewind();
    if bounds.width() <= 0. || bounds.height() <= 0. { return true; }
    let [left, top, right, bottom] = borders;
    // If opposing borders consume the box, their inner edges must meet,
    // not cross. Double intermediates prevent overflow for finite widths.
    let inset_pair = |extent: f32, a: f32, b: f32| {
        let total = a as f64 + b as f64;
        let scale = if total > extent as f64 { extent as f64 / total } else { 1. };
        ((a as f64 * scale) as f32, (b as f64 * scale) as f32)
    };
    let (left, right) = inset_pair(bounds.width(), left, right);
    let (top, bottom) = inset_pair(bounds.height(), top, bottom);
    let outer = [Vec2D::new(bounds.min_x,bounds.min_y), Vec2D::new(bounds.max_x,bounds.min_y),
        Vec2D::new(bounds.max_x,bounds.max_y), Vec2D::new(bounds.min_x,bounds.max_y)];
    let mut inner = [Vec2D::new(bounds.min_x+left,bounds.min_y+top), Vec2D::new(bounds.max_x-right,bounds.min_y+top),
        Vec2D::new(bounds.max_x-right,bounds.max_y-bottom), Vec2D::new(bounds.min_x+left,bounds.max_y-bottom)];
    constrain(bounds, &mut radii);
    for (radius, widths) in radii.iter_mut().zip([[left,top],[right,top],[right,bottom],[left,bottom]]) {
        radius[0]=(radius[0]-widths[0]).max(0.);
        radius[1]=(radius[1]-widths[1]).max(0.);
    }
    constrain(Aabb::new(inner[0].x,inner[0].y,inner[2].x,inner[2].y), &mut radii);
    // Continue each outer-to-inner miter ray until it reaches the chord
    // between the inner arc endpoints. Turning toward the box center here
    // would move the color boundary through the visible rounded ring.
    for i in 0..4 {
        let [rx,ry]=radii[i];
        if rx <= 0. || ry <= 0. { continue; }
        let dx=inner[i].x-outer[i].x;let dy=inner[i].y-outer[i].y;
        let denominator=(dx as f64).abs()/rx as f64+(dy as f64).abs()/ry as f64;
        if denominator > 0. {
            inner[i].x=(inner[i].x as f64+dx as f64/denominator) as f32;
            inner[i].y=(inner[i].y as f64+dy as f64/denominator) as f32;
        }
    }
    let next = (side+1)%4;
    path.move_to_point(outer[side]);
    for point in [outer[next],inner[next],inner[side]] { path.line_to_point(point); }
    path.close();
    true
}

/// Expand a padding-edge rounded rectangle. Insets are negative outsets;
/// order is left, top, right, bottom, and corners are TL, TR, BR, BL.
/// Chrome 153 enables ShadowContourFollowsBorder, including its coverage
/// factor. Keep this distinct from the legacy Rive rounded-rectangle path.
pub(crate) fn outset_path(
    path: &mut RawPath,
    bounds: Aabb,
    circular_radii: [f32; 4],
    outsets: [f32; 4],
) -> bool {
    elliptical_outset_path(path, bounds, circular_radii.map(|r| [r, r]), outsets)
}

/// Per-axis corner input for CSS ellipse geometry. Percentages must be resolved
/// against the live border box by the caller before entering this helper.
pub(crate) fn elliptical_outset_path(
    path: &mut RawPath,
    bounds: Aabb,
    mut radii: [[f32; 2]; 4],
    outsets: [f32; 4],
) -> bool {
    let [left, top, right, bottom] = outsets;
    let expanded = Aabb::new(bounds.min_x - left, bounds.min_y - top,
        bounds.max_x + right, bounds.max_y + bottom);
    if ![expanded.min_x, expanded.min_y, expanded.max_x, expanded.max_y,
        bounds.width(), bounds.height()].into_iter().all(f32::is_finite)
        || !radii.iter().flatten().all(|r| r.is_finite() && *r >= 0.) {
        return false;
    }
    path.rewind();
    if expanded.width() <= 0. || expanded.height() <= 0. { return true; }
    constrain(bounds, &mut radii);
    for (radius, outset) in radii.iter_mut().zip([
        [left, top], [right, top], [right, bottom], [left, bottom],
    ]) {
        if radius == &[0., 0.] { continue; }
        let coverage = if bounds.width() > 0. && bounds.height() > 0. {
            2. * (radius[0] / bounds.width()).min(radius[1] / bounds.height())
        } else { 0. };
        for axis in 0..2 {
            radius[axis] = adjusted_radius(radius[axis], outset[axis], coverage);
        }
    }
    if !radii.iter().flatten().all(|v| v.is_finite()) { return false; }
    constrain(expanded, &mut radii);
    add_elliptical_rect(path, expanded, radii);
    true
}

fn adjusted_radius(radius: f32, outset: f32, coverage: f32) -> f32 {
    if outset == 0. { return radius; }
    let result = if radius > outset || coverage > 1. {
        radius + outset
    } else {
        let ratio = radius / outset;
        radius + outset * (1. - (1. - ratio).powi(3) * (1. - coverage.powi(3)))
    };
    result.max(0.)
}

fn constrain(bounds: Aabb, radii: &mut [[f32; 2]; 4]) {
    let mut factor = 1.0_f32;
    for (extent, sum) in [
        (bounds.width(), radii[0][0] + radii[1][0]),
        (bounds.width(), radii[3][0] + radii[2][0]),
        (bounds.height(), radii[0][1] + radii[3][1]),
        (bounds.height(), radii[1][1] + radii[2][1]),
    ] {
        if sum > extent { factor = factor.min((extent / sum).max(0.)); }
    }
    for radius in radii { for component in radius { *component *= factor; } }
}

fn add_elliptical_rect(path: &mut RawPath, b: Aabb, radii: [[f32; 2]; 4]) {
    // Standard quarter-ellipse cubic approximation, matching the existing
    // renderer's circular path convention, independently scaled in x and y.
    const K: f32 = 0.552_284_8;
    let corners = [
        ([b.min_x, b.min_y], [0., 1.], [1., 0.]),
        ([b.max_x, b.min_y], [-1., 0.], [0., 1.]),
        ([b.max_x, b.max_y], [0., -1.], [-1., 0.]),
        ([b.min_x, b.max_y], [1., 0.], [0., -1.]),
    ];
    for (i, ((corner, prev, next), radius)) in corners.into_iter().zip(radii).enumerate() {
        let radius = if radius[0] == 0. || radius[1] == 0. { [0., 0.] } else { radius };
        let point = |direction: [f32; 2], scale: f32| Vec2D::new(
            corner[0] + direction[0] * radius[0] * scale,
            corner[1] + direction[1] * radius[1] * scale);
        let enter = point(prev, 1.);
        if i == 0 { path.move_to_point(enter); } else { path.line_to_point(enter); }
        if radius != [0., 0.] {
            path.cubic_to_points(point(prev, 1. - K), point(next, 1. - K), point(next, 1.));
        }
    }
    path.close();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elliptical_axes_share_one_overlap_reduction_factor() {
        let mut path = RawPath::default();
        assert!(elliptical_outset_path(&mut path, Aabb::new(0., 0., 100., 80.),
            [[80., 10.]; 4], [0.; 4]));
        // The horizontal sum is 160: both axes must scale by 100/160.
        assert_eq!(path.points()[0], Vec2D::new(0., 6.25));
        assert_eq!(path.points()[3], Vec2D::new(50., 0.));
        assert_eq!(path.bounds(), Aabb::new(0., 0., 100., 80.));
    }

    #[test]
    fn elliptical_inner_contour_subtracts_each_axis_and_rejects_invalid_axes() {
        let bounds = Aabb::new(0., 0., 100., 80.);
        let mut ring = RawPath::default();
        assert!(elliptical_border_ring_path(&mut ring, bounds, [[30.,20.];4], [8.,4.,10.,6.]));
        assert_eq!(ring.points()[0], Vec2D::new(0.,20.));
        assert_eq!(ring.points()[3], Vec2D::new(30.,0.));
        assert_eq!(ring.points()[16], Vec2D::new(8.,20.));
        assert_eq!(ring.points()[19], Vec2D::new(30.,4.));
        for invalid in [-1., f32::NAN, f32::INFINITY] {
            let mut path = RawPath::default();
            let mut radii = [[10.,20.];4];
            radii[2][1] = invalid;
            assert!(!elliptical_outset_path(&mut path, bounds, radii, [0.;4]));
        }
    }

    #[test]
    fn rounded_side_clip_preserves_miter_ray_and_reaches_inner_arc_chord() {
        let mut path=RawPath::default();
        assert!(border_side_clip_path(&mut path,Aabb::new(0.,0.,100.,80.),[30.;4],[20.,2.,8.,14.],0));
        let points=path.points();
        // Top-side path: outer TL, outer TR, extended inner TR, extended inner TL.
        for (outer, corner, endpoint, radius) in [
            (Vec2D::new(0.,0.),Vec2D::new(20.,2.),points[3],[10.,28.]),
            (Vec2D::new(100.,0.),Vec2D::new(92.,2.),points[2],[22.,28.]),
        ] {
            let dx=corner.x-outer.x;let dy=corner.y-outer.y;
            assert!(((endpoint.x-outer.x)*dy-(endpoint.y-outer.y)*dx).abs()<0.001);
            let chord=(endpoint.x-corner.x).abs()/radius[0]+(endpoint.y-corner.y).abs()/radius[1];
            assert!((chord-1.).abs()<0.00001);
        }
    }

    #[test]
    fn elliptical_side_partition_reaches_inner_chord_without_rotating_miter() {
        let mut path = RawPath::default();
        assert!(elliptical_border_side_clip_path(&mut path, Aabb::new(0.,0.,100.,80.),
            [[40.,20.];4], [10.,4.,8.,6.], 0));
        for (outer, corner, endpoint, radius) in [
            (Vec2D::new(0.,0.), Vec2D::new(10.,4.), path.points()[3], [30.,16.]),
            (Vec2D::new(100.,0.), Vec2D::new(92.,4.), path.points()[2], [32.,16.]),
        ] {
            let dx = corner.x - outer.x;
            let dy = corner.y - outer.y;
            assert!(((endpoint.x-outer.x)*dy-(endpoint.y-outer.y)*dx).abs()<0.001);
            let chord = (endpoint.x-corner.x).abs()/radius[0] + (endpoint.y-corner.y).abs()/radius[1];
            assert!((chord-1.).abs()<0.00001);
        }
    }

    #[test]
    fn physical_side_partitions_cover_square_ring() {
        let bounds = Aabb::new(10.,20.,110.,100.);
        for widths in [[20.,2.,8.,14.], [8.,0.,16.,4.], [60.,70.,80.,90.], [0.;4]] {
            let mut sum = 0.;
            for side in 0..4 {
                let mut path = RawPath::default();
                assert!(border_side_clip_path(&mut path,bounds,[0.;4],widths,side));
                let points = path.points();
                let area = (0..4).map(|i| {
                    let a=points[i];let b=points[(i+1)%4];a.x*b.y-b.x*a.y
                }).sum::<f32>() * 0.5;
                assert!(area >= 0., "inverted side {side}: {widths:?}");sum+=area;
            }
            let inner=(100.-widths[0]-widths[2]).max(0.)*(80.-widths[1]-widths[3]).max(0.);
            assert!((sum-(8000.-inner)).abs()<0.01, "{widths:?}");
        }
        let mut path=RawPath::default();
        assert!(!border_side_clip_path(&mut path,bounds,[0.;4],[f32::NAN;4],0));
        assert!(!border_side_clip_path(&mut path,bounds,[0.;4],[1.;4],4));
    }

    #[test]
    fn border_ring_contains_outer_and_inset_contours() {
        let mut ring = RawPath::default();
        let bounds = Aabb::new(0., 0., 100., 80.);
        assert!(border_ring_path(&mut ring, bounds, [0.; 4], [8., 9., 10., 11.]));
        assert_eq!(ring.bounds(), bounds);
        assert_eq!(ring.points(), &[
            Vec2D::new(0., 0.), Vec2D::new(100., 0.), Vec2D::new(100., 80.), Vec2D::new(0., 80.),
            Vec2D::new(8., 9.), Vec2D::new(90., 9.), Vec2D::new(90., 69.), Vec2D::new(8., 69.),
        ]);
        // Two contours with even-odd filling exclude the whole interior;
        // collapsing the inner box leaves only the outer contour.
        assert!(border_ring_path(&mut ring, bounds, [0.; 4], [60.; 4]));
        assert_eq!(ring.points().len(), 4);
        assert!(border_ring_path(&mut ring, bounds, [18.; 4], [0.; 4]));
        assert!(ring.points().is_empty());
    }

    #[test]
    fn border_ring_normalizes_outer_radius_before_insetting() {
        let mut ring = RawPath::default();
        let bounds = Aabb::new(0., 0., 100., 80.);
        assert!(border_ring_path(&mut ring, bounds, [90.; 4], [8.; 4]));
        // Outer90px is constrained to40px. The inner starts at x8,y40,
        // with32px radii, rather than renormalizing an authored82px radius.
        assert_eq!(ring.points()[0], Vec2D::new(0., 40.));
        assert_eq!(ring.points()[16], Vec2D::new(8., 40.));
        for edge in [f32::NAN, f32::INFINITY, -1.] {
            assert!(!border_ring_path(&mut ring, bounds, [18.; 4], [edge; 4]));
        }
    }

    #[test]
    fn coverage_adjustment_distinguishes_small_corners_and_capsules() {
        assert_eq!(adjusted_radius(10., 20., 0.), 27.5);
        assert_eq!(adjusted_radius(10., 20., 0.5), 27.8125);
        assert_eq!(adjusted_radius(10., 20., 1.), 30.);
        assert_eq!(adjusted_radius(10., -18., 0.5), 0.);
        assert_eq!(adjusted_radius(10., 0., 0.5), 10.);
    }

    #[test]
    fn content_insets_make_ellipses_without_changing_the_reference_box() {
        let bounds = Aabb::new(0., 0., 200., 100.);
        let mut path = RawPath::default();
        assert!(outset_path(&mut path, bounds, [24.; 4], [-10., -4., -10., -4.]));
        assert_eq!(path.bounds(), Aabb::new(10., 4., 190., 96.));
        assert_eq!(path.points()[0], Vec2D::new(10., 24.));
        assert_eq!(path.points()[3], Vec2D::new(24., 4.));
        assert_eq!(bounds, Aabb::new(0., 0., 200., 100.));
    }

    #[test]
    fn square_and_empty_clips_do_not_gain_round_corners() {
        let mut path = RawPath::default();
        assert!(outset_path(&mut path, Aabb::new(0., 0., 100., 80.), [0.; 4], [24.; 4]));
        assert_eq!(path.points().len(), 4);
        assert_eq!(path.bounds(), Aabb::new(-24., -24., 124., 104.));
        assert!(outset_path(&mut path, Aabb::new(0., 0., 100., 80.), [24.; 4], [-60.; 4]));
        assert!(path.points().is_empty());
    }
}
