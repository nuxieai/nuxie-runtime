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
/// corner to the matching inner corner.
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
            clip.outline.line_to_point(inner_corner[next]);
            clip.outline.line_to_point(inner_corner[side]);
            clip.outline.close();
        }
        geometry.clip = Some(clip);
    }
    geometry
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
}
