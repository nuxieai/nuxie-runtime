//! Box paint geometry in CSS mode (docs/CSS_LAYOUT.md, section 4): the
//! border box with CSS corner radii, the padding box with its inner radii,
//! and the clip `overflow` implies. Our layer over Rive: Rive's own box path
//! has circular radii clamped per corner, while CSS radii are elliptical and
//! scale together when they overlap.
use crate::source::math::{aabb::Aabb, raw_path::RawPath, vec2d::Vec2D};

/// The cubic approximation of a quarter ellipse: 4/3 (sqrt(2) - 1).
const KAPPA: f32 = 0.552_284_8;

/// Corner radii in CSS order: top left, top right, bottom right, bottom
/// left. Each is (horizontal, vertical).
pub type Radii = [Vec2D; 4];

/// The used radii of a `width` by `height` border box: each corner's point
/// radius plus its percentage of the width (horizontal) and of the height
/// (vertical), all scaled down together when adjacent radii overlap
/// (CSS Backgrounds 3, "Overlapping Curves").
pub fn border_radii(width: f32, height: f32, points: [f32; 4], percents: [f32; 4]) -> Radii {
    let mut radii = [Vec2D::default(); 4];
    for corner in 0..4 {
        let point = points[corner].max(0.0);
        let percent = percents[corner].max(0.0) / 100.0;
        radii[corner] = Vec2D::new(point + percent * width, point + percent * height);
        // A zero on either axis makes the corner square.
        if radii[corner].x <= 0.0 || radii[corner].y <= 0.0 {
            radii[corner] = Vec2D::default();
        }
    }
    let ratio = |side: f32, a: f32, b: f32| {
        if a + b > 0.0 {
            side.max(0.0) / (a + b)
        } else {
            f32::INFINITY
        }
    };
    let scale = [
        ratio(width, radii[0].x, radii[1].x),
        ratio(width, radii[3].x, radii[2].x),
        ratio(height, radii[0].y, radii[3].y),
        ratio(height, radii[1].y, radii[2].y),
    ]
    .into_iter()
    .fold(1.0f32, f32::min);
    if scale < 1.0 {
        for radius in &mut radii {
            *radius = Vec2D::new(radius.x * scale, radius.y * scale);
        }
    }
    radii
}

/// The padding edge's radii: each outer radius less the adjacent border
/// widths, floored at zero. `border` is top, right, bottom, left.
pub fn inner_radii(outer: Radii, border: [f32; 4]) -> Radii {
    let [top, right, bottom, left] = border;
    let inset = |radius: Vec2D, x: f32, y: f32| {
        let inner = Vec2D::new((radius.x - x).max(0.0), (radius.y - y).max(0.0));
        if inner.x <= 0.0 || inner.y <= 0.0 {
            Vec2D::default()
        } else {
            inner
        }
    };
    [
        inset(outer[0], left, top),
        inset(outer[1], right, top),
        inset(outer[2], right, bottom),
        inset(outer[3], left, bottom),
    ]
}

/// `bounds` inset by `border` (top, right, bottom, left).
pub fn padding_box(bounds: Aabb, border: [f32; 4]) -> Aabb {
    let [top, right, bottom, left] = border;
    let min_x = bounds.left() + left;
    let min_y = bounds.top() + top;
    Aabb::new(
        min_x,
        min_y,
        (bounds.right() - right).max(min_x),
        (bounds.bottom() - bottom).max(min_y),
    )
}

/// A rounded rectangle with elliptical corners, clockwise from the top
/// left corner's end, as Rive's own rounded rectangle winds.
pub fn add_rounded_rect(path: &mut RawPath, bounds: Aabb, radii: Radii) {
    let (left, top, right, bottom) = (bounds.left(), bounds.top(), bounds.right(), bounds.bottom());
    let [tl, tr, br, bl] = radii;
    path.move_to(left + tl.x, top);
    path.line_to(right - tr.x, top);
    if tr.x > 0.0 {
        path.cubic_to(
            right - tr.x + KAPPA * tr.x,
            top,
            right,
            top + tr.y - KAPPA * tr.y,
            right,
            top + tr.y,
        );
    }
    path.line_to(right, bottom - br.y);
    if br.x > 0.0 {
        path.cubic_to(
            right,
            bottom - br.y + KAPPA * br.y,
            right - br.x + KAPPA * br.x,
            bottom,
            right - br.x,
            bottom,
        );
    }
    path.line_to(left + bl.x, bottom);
    if bl.x > 0.0 {
        path.cubic_to(
            left + bl.x - KAPPA * bl.x,
            bottom,
            left,
            bottom - bl.y + KAPPA * bl.y,
            left,
            bottom - bl.y,
        );
    }
    path.line_to(left, top + tl.y);
    if tl.x > 0.0 {
        path.cubic_to(
            left,
            top + tl.y - KAPPA * tl.y,
            left + tl.x - KAPPA * tl.x,
            top,
            left + tl.x,
            top,
        );
    }
    path.close();
}

/// How far a clip reaches on an axis that does not clip.
const UNCLIPPED: f32 = 1.0e7;

/// The clip `overflow` implies, or None when both axes are visible. Both
/// axes clipping: the padding box with its inner radii. One axis: a band
/// across that axis's padding edges, unbounded along the other.
pub fn add_overflow_clip(
    path: &mut RawPath,
    padding: Aabb,
    inner: Radii,
    clips: (bool, bool),
) -> bool {
    match clips {
        (false, false) => false,
        (true, true) => {
            add_rounded_rect(path, padding, inner);
            true
        }
        (x, _) => {
            let band = if x {
                Aabb::new(padding.left(), -UNCLIPPED, padding.right(), UNCLIPPED)
            } else {
                Aabb::new(-UNCLIPPED, padding.top(), UNCLIPPED, padding.bottom())
            };
            add_rounded_rect(path, band, [Vec2D::default(); 4]);
            true
        }
    }
}

/// Whether `point`, in the box's space, is inside the clip `overflow`
/// implies: the padding box on the clipped axes, with its inner radii when
/// both clip (as [`add_overflow_clip`] draws it).
pub fn overflow_clip_contains(
    point: Vec2D,
    padding: Aabb,
    inner: Radii,
    clips: (bool, bool),
) -> bool {
    let (x, y) = clips;
    if x && !(point.x >= padding.left() && point.x <= padding.right()) {
        return false;
    }
    if y && !(point.y >= padding.top() && point.y <= padding.bottom()) {
        return false;
    }
    if !(x && y) {
        return true;
    }
    // Each corner's ellipse centre and the quadrant it rounds.
    let corners = [
        (
            Vec2D::new(padding.left() + inner[0].x, padding.top() + inner[0].y),
            -1.0,
            -1.0,
        ),
        (
            Vec2D::new(padding.right() - inner[1].x, padding.top() + inner[1].y),
            1.0,
            -1.0,
        ),
        (
            Vec2D::new(padding.right() - inner[2].x, padding.bottom() - inner[2].y),
            1.0,
            1.0,
        ),
        (
            Vec2D::new(padding.left() + inner[3].x, padding.bottom() - inner[3].y),
            -1.0,
            1.0,
        ),
    ];
    corners
        .iter()
        .zip(inner)
        .all(|(&(centre, sx, sy), radius)| {
            let (dx, dy) = (point.x - centre.x, point.y - centre.y);
            if radius.x <= 0.0 || radius.y <= 0.0 || dx * sx <= 0.0 || dy * sy <= 0.0 {
                return true;
            }
            let (u, v) = (dx / radius.x, dy / radius.y);
            u * u + v * v <= 1.0
        })
}

/// A shadow's corner radii after CSS spread (CSS Backgrounds 3, "Shadow
/// Shape, Spread, and Knockout"): each radius grows by the spread, less so
/// while it is smaller than the spread, so a square corner stays square. A
/// negative spread shrinks radii, floored at zero.
pub fn spread_radii(radii: Radii, spread: f32) -> Radii {
    let grow = |radius: f32| {
        if spread <= 0.0 {
            (radius + spread).max(0.0)
        } else if radius >= spread {
            radius + spread
        } else {
            let ratio = radius / spread - 1.0;
            radius + spread * (1.0 + ratio * ratio * ratio)
        }
    };
    radii.map(|radius| {
        let grown = Vec2D::new(grow(radius.x), grow(radius.y));
        if grown.x <= 0.0 || grown.y <= 0.0 {
            Vec2D::default()
        } else {
            grown
        }
    })
}

/// `bounds` grown by `amount` on every side (shrunk when negative, never
/// past its centre).
pub fn outset(bounds: Aabb, amount: f32) -> Aabb {
    let (cx, cy) = (
        (bounds.left() + bounds.right()) / 2.0,
        (bounds.top() + bounds.bottom()) / 2.0,
    );
    Aabb::new(
        (bounds.left() - amount).min(cx),
        (bounds.top() - amount).min(cy),
        (bounds.right() + amount).max(cx),
        (bounds.bottom() + amount).max(cy),
    )
}

/// A region as an outline and holes: the holes wind the other way, so the
/// non-zero rule leaves them out whatever fill rules a renderer's clips
/// support.
#[derive(Default)]
pub struct Region {
    pub outline: RawPath,
    pub holes: RawPath,
}

/// The geometry a CSS box draws one of its special paints with, in the
/// box's own space: the region to draw and the region it is clipped to.
#[derive(Default)]
pub struct PaintGeometry {
    pub path: Region,
    pub clip: Option<Region>,
}

/// A CSS `box-shadow` drawn by a feathered fill (docs/CSS_LAYOUT.md, 4.2).
/// `reach` is how far the shadow can extend past its shape (blur and
/// offset), which bounds the region that stands in for "everywhere".
/// An outer shadow is the border box grown by the spread, drawn only
/// outside the border box. An inset shadow is everything outside the
/// padding box shrunk by the spread (its hole), drawn only inside the
/// padding box.
pub fn box_shadow(
    border_box: Aabb,
    radii: Radii,
    padding: Aabb,
    inner: Radii,
    spread: f32,
    inset: bool,
    reach: f32,
) -> PaintGeometry {
    let margin = reach.max(0.0) + spread.abs() + 2.0;
    let mut geometry = PaintGeometry::default();
    let mut clip = Region::default();
    if inset {
        let hole = outset(padding, -spread);
        add_rounded_rect(
            &mut geometry.path.outline,
            outset(padding, margin),
            [Vec2D::default(); 4],
        );
        add_rounded_rect(&mut geometry.path.holes, hole, spread_radii(inner, -spread));
        add_rounded_rect(&mut clip.outline, padding, inner);
    } else {
        add_rounded_rect(
            &mut geometry.path.outline,
            outset(border_box, spread),
            spread_radii(radii, spread),
        );
        let everywhere = outset(border_box, margin + spread.max(0.0));
        add_rounded_rect(&mut clip.outline, everywhere, [Vec2D::default(); 4]);
        add_rounded_rect(&mut clip.holes, border_box, radii);
    }
    geometry.clip = Some(clip);
    geometry
}

/// A per-side CSS border (`BoxBorder`): the ring between the border box and
/// the padding box, limited to the sides in `sides` (1 top, 2 right,
/// 4 bottom, 8 left). Adjacent sides meet on the line from each outer
/// corner through the matching inner corner, continued to the end of the
/// corner's curve, so a rounded ring is shared between its two sides.
pub fn box_border(
    border_box: Aabb,
    radii: Radii,
    padding: Aabb,
    inner: Radii,
    sides: u8,
) -> PaintGeometry {
    let mut geometry = PaintGeometry::default();
    add_rounded_rect(&mut geometry.path.outline, border_box, radii);
    add_rounded_rect(&mut geometry.path.holes, padding, inner);
    if sides & 15 != 15 {
        let outer = [
            Vec2D::new(border_box.left(), border_box.top()),
            Vec2D::new(border_box.right(), border_box.top()),
            Vec2D::new(border_box.right(), border_box.bottom()),
            Vec2D::new(border_box.left(), border_box.bottom()),
        ];
        let inner_corner = [
            Vec2D::new(padding.left(), padding.top()),
            Vec2D::new(padding.right(), padding.top()),
            Vec2D::new(padding.right(), padding.bottom()),
            Vec2D::new(padding.left(), padding.bottom()),
        ];
        // Where each corner's join line leaves the corner's region: its
        // curve, and at least its border widths.
        let join_end: [Vec2D; 4] = std::array::from_fn(|corner| {
            let outer = outer[corner];
            let step = inner_corner[corner] - outer;
            let (width, height) = (step.x.abs(), step.y.abs());
            let reach =
                |extent: f32, border: f32| (border > 0.0).then(|| extent.max(border) / border);
            let t = match (
                reach(radii[corner].x, width),
                reach(radii[corner].y, height),
            ) {
                (Some(x), Some(y)) => x.min(y),
                (Some(t), None) | (None, Some(t)) => t,
                (None, None) => 0.0,
            };
            Vec2D::new(outer.x + step.x * t, outer.y + step.y * t)
        });
        let mut clip = Region::default();
        // Side n runs from corner n to corner n + 1: top, right, bottom, left.
        // Each wedge winds clockwise, as the box does.
        for side in 0..4 {
            if sides & (1 << side) == 0 {
                continue;
            }
            let next = (side + 1) % 4;
            clip.outline.move_to_point(outer[side]);
            clip.outline.line_to_point(outer[next]);
            clip.outline.line_to_point(join_end[next]);
            clip.outline.line_to_point(join_end[side]);
            clip.outline.close();
        }
        geometry.clip = Some(clip);
    }
    geometry
}

/// Where a gradient goes over a box (`BoxGradient`, docs/CSS_LAYOUT.md,
/// 4.2): its start and end in the box's space, and the shader transform an
/// elliptical radial gradient needs (identity otherwise).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientGeometry {
    pub start: Vec2D,
    pub end: Vec2D,
    pub transform: crate::source::math::mat2d::Mat2D,
}

/// CSS's gradient geometry over `area`, the box's padding box (the
/// background positioning area). Linear: the gradient line through the
/// centre at `angle` (or toward a corner), as long as CSS makes it, so its
/// ends' perpendiculars touch the far corners. Radial: the ending shape
/// from `shape` and `size` at the `at` position; an ellipse is a circle of
/// its horizontal radius scaled vertically. None for a degenerate radius.
pub fn gradient_geometry(
    values: super::CssValues<'_>,
    area: Aabb,
    radial: bool,
) -> Option<GradientGeometry> {
    use super::keys;
    use crate::source::math::mat2d::Mat2D;
    let (width, height) = (area.width(), area.height());
    let identity = Mat2D::identity();
    if !radial {
        let direction = match values.choice(keys::GRADIENT_TO_CORNER) {
            1 => Vec2D::new(height, -width),
            2 => Vec2D::new(height, width),
            3 => Vec2D::new(-height, width),
            4 => Vec2D::new(-height, -width),
            _ => {
                let angle = values.number(keys::GRADIENT_ANGLE).to_radians();
                Vec2D::new(angle.sin(), -angle.cos())
            }
        };
        let length = (direction.x * direction.x + direction.y * direction.y).sqrt();
        if length <= 0.0 || !length.is_finite() {
            return None;
        }
        let unit = Vec2D::new(direction.x / length, direction.y / length);
        let half = (width * unit.x.abs() + height * unit.y.abs()) / 2.0;
        let center = Vec2D::new(area.left() + width / 2.0, area.top() + height / 2.0);
        return Some(GradientGeometry {
            start: Vec2D::new(center.x - unit.x * half, center.y - unit.y * half),
            end: Vec2D::new(center.x + unit.x * half, center.y + unit.y * half),
            transform: identity,
        });
    }
    let part = |key: u16, basis: f32| values.number(key) + values.number(key + 1) / 100.0 * basis;
    let center = Vec2D::new(
        area.left() + part(keys::GRADIENT_CENTER_X, width),
        area.top() + part(keys::GRADIENT_CENTER_Y, height),
    );
    let sides = (
        (center.x - area.left()).abs(),
        (area.right() - center.x).abs(),
        (center.y - area.top()).abs(),
        (area.bottom() - center.y).abs(),
    );
    let near = Vec2D::new(sides.0.min(sides.1), sides.2.min(sides.3));
    let far = Vec2D::new(sides.0.max(sides.1), sides.2.max(sides.3));
    let circle = values.choice(keys::GRADIENT_SHAPE) == 1;
    let corner_distance = |corner: Vec2D| corner.x.hypot(corner.y);
    // The ellipse through `corner` with the aspect ratio of `side`.
    let through = |corner: Vec2D, side: Vec2D| {
        if circle {
            let radius = corner_distance(corner);
            Vec2D::new(radius, radius)
        } else if side.y <= 0.0 {
            Vec2D::default()
        } else {
            let ratio = side.x / side.y;
            let y = ((corner.x / ratio).powi(2) + corner.y * corner.y).sqrt();
            Vec2D::new(ratio * y, y)
        }
    };
    let radii = match values.choice(keys::GRADIENT_SIZE) {
        1 if circle => Vec2D::new(far.x.max(far.y), far.x.max(far.y)),
        1 => far,
        2 => through(near, near),
        3 if circle => Vec2D::new(near.x.min(near.y), near.x.min(near.y)),
        3 => near,
        4 => {
            let x = part(keys::GRADIENT_RADIUS_X, width).max(0.0);
            let y = if circle {
                x
            } else {
                part(keys::GRADIENT_RADIUS_Y, height).max(0.0)
            };
            Vec2D::new(x, y)
        }
        // farthest-corner, the initial size.
        _ => through(far, far),
    };
    if radii.x <= 0.0 || radii.y <= 0.0 || !radii.x.is_finite() || !radii.y.is_finite() {
        return None;
    }
    let transform = if radii.x == radii.y {
        identity
    } else {
        Mat2D::from_translate(center.x, center.y)
            * Mat2D::from_scale(1.0, radii.y / radii.x)
            * Mat2D::from_translate(-center.x, -center.y)
    };
    Some(GradientGeometry {
        start: center,
        end: Vec2D::new(center.x + radii.x, center.y),
        transform,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_radii_are_elliptical_and_overlaps_scale_together() {
        // 50% on a 200 by 100 box: a 100 by 50 ellipse at each corner, the
        // `border-radius: 50%` pill. No overlap: 100 + 100 fits 200.
        let radii = border_radii(200.0, 100.0, [0.0; 4], [50.0; 4]);
        assert!(radii.iter().all(|r| *r == Vec2D::new(100.0, 50.0)));
        // 80 point radii on a 100 by 100 box overlap: scaled by 100 / 160.
        let radii = border_radii(100.0, 100.0, [80.0; 4], [0.0; 4]);
        assert!(radii.iter().all(|r| *r == Vec2D::new(50.0, 50.0)));
        // A point radius plus a percentage.
        let radii = border_radii(200.0, 100.0, [10.0, 0.0, 0.0, 0.0], [10.0, 0.0, 0.0, 0.0]);
        assert_eq!(radii[0], Vec2D::new(30.0, 20.0));
        assert_eq!(radii[1], Vec2D::default());
    }

    #[test]
    fn inner_radii_lose_the_border_widths() {
        let outer = [Vec2D::new(20.0, 10.0); 4];
        let inner = inner_radii(outer, [4.0, 8.0, 12.0, 2.0]);
        assert_eq!(inner[0], Vec2D::new(18.0, 6.0));
        assert_eq!(inner[1], Vec2D::new(12.0, 6.0));
        // The bottom border is wider than the vertical radius: square.
        assert_eq!(inner[2], Vec2D::default());
        assert_eq!(inner[3], Vec2D::default());
        assert_eq!(
            padding_box(Aabb::new(0.0, 0.0, 100.0, 50.0), [4.0, 8.0, 12.0, 2.0]),
            Aabb::new(2.0, 4.0, 92.0, 38.0)
        );
    }

    /// A centre or radius made mixed at run time resolves as its percentage
    /// alone, as layout lengths do (docs/CSS_LAYOUT.md, 2.1).
    #[test]
    fn mixed_gradient_values_use_their_percentage() {
        use crate::css_layout::{CssValues, keys};
        let (first, count) = keys::range(keys::BOX_GRADIENT).unwrap();
        let mut values: Vec<f32> = (first..first + count as u16)
            .map(keys::default_value)
            .collect();
        // centerX 10 points beside the default 50%.
        values[usize::from(keys::GRADIENT_CENTER_X - first)] = 10.0;
        let geometry = gradient_geometry(
            CssValues {
                first,
                values: &values,
            },
            Aabb::new(0.0, 0.0, 100.0, 100.0),
            true,
        )
        .unwrap();
        assert_eq!(geometry.start, Vec2D::new(50.0, 50.0));
    }

    #[test]
    fn gradient_lines_reach_the_far_corners_as_css_does() {
        use crate::css_layout::{CssValues, keys};
        let area = Aabb::new(10.0, 20.0, 210.0, 120.0);
        let values = |overrides: &[(u16, f32)]| {
            let (first, count) = keys::range(keys::BOX_GRADIENT).unwrap();
            let mut values: Vec<f32> = (first..first + count as u16)
                .map(keys::default_value)
                .collect();
            for (key, value) in overrides {
                values[usize::from(key - first)] = *value;
            }
            values
        };
        let geometry = |overrides: &[(u16, f32)], radial: bool| {
            let values = values(overrides);
            gradient_geometry(
                CssValues {
                    first: keys::GRADIENT_ANGLE,
                    values: &values,
                },
                area,
                radial,
            )
            .unwrap()
        };
        // 180deg (to bottom): the vertical centre line, top to bottom.
        let down = geometry(&[], false);
        let near = |a: Vec2D, b: Vec2D| (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3;
        assert!(
            near(down.start, Vec2D::new(110.0, 20.0)),
            "{:?}",
            down.start
        );
        assert!(near(down.end, Vec2D::new(110.0, 120.0)), "{:?}", down.end);
        // 90deg (to right).
        let right = geometry(&[(keys::GRADIENT_ANGLE, 90.0)], false);
        assert!((right.start.x - 10.0).abs() < 1e-4 && (right.end.x - 210.0).abs() < 1e-4);
        // Farthest-corner ellipse at the centre of a 200 by 100 area: the
        // farthest-side ellipse (100 by 50) scaled by sqrt(2).
        let ellipse = geometry(&[], true);
        assert_eq!(ellipse.start, Vec2D::new(110.0, 70.0));
        assert!((ellipse.end.x - 110.0 - 100.0 * 2f32.sqrt()).abs() < 1e-3);
        assert!(ellipse.transform != crate::source::math::mat2d::Mat2D::identity());
        // A closest-side circle.
        let circle = geometry(
            &[(keys::GRADIENT_SHAPE, 1.0), (keys::GRADIENT_SIZE, 3.0)],
            true,
        );
        assert_eq!(circle.end, Vec2D::new(160.0, 70.0));
        assert_eq!(
            circle.transform,
            crate::source::math::mat2d::Mat2D::identity()
        );
    }
}
