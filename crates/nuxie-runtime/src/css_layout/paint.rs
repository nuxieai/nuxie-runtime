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
        let wedges = side_wedges(border_box, radii, padding);
        let mut clip = Region::default();
        // Each wedge winds clockwise, as the box does.
        for (side, wedge) in wedges.iter().enumerate() {
            if sides & (1 << side) == 0 {
                continue;
            }
            clip.outline.move_to_point(wedge[0]);
            for &point in &wedge[1..] {
                clip.outline.line_to_point(point);
            }
            clip.outline.close();
        }
        geometry.clip = Some(clip);
    }
    geometry
}

/// Each side's join wedge (top, right, bottom, left): from its two outer
/// corners to where their join lines leave the corners' regions, along
/// the line from each outer corner through its inner corner, continued to
/// the end of the corner's curve and at least the border widths.
fn side_wedges(border_box: Aabb, radii: Radii, padding: Aabb) -> [[Vec2D; 4]; 4] {
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
    let join_end: [Vec2D; 4] = std::array::from_fn(|corner| {
        let outer = outer[corner];
        let step = inner_corner[corner] - outer;
        let (width, height) = (step.x.abs(), step.y.abs());
        let reach = |extent: f32, border: f32| (border > 0.0).then(|| extent.max(border) / border);
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
    // Side n runs from corner n to corner n + 1.
    std::array::from_fn(|side| {
        let next = (side + 1) % 4;
        [outer[side], outer[next], join_end[next], join_end[side]]
    })
}

/// Whether `point` is in a polygon (even-odd).
fn polygon_contains(polygon: &[Vec2D], point: Vec2D) -> bool {
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        if (current.y > point.y) != (previous.y > point.y) {
            let x = previous.x
                + (point.y - previous.y) / (current.y - previous.y) * (current.x - previous.x);
            if point.x < x {
                inside = !inside;
            }
        }
        previous = current;
    }
    inside
}

/// Whether `point` is on a border ring as it is painted (docs/CSS_LAYOUT.md,
/// 4.5): between the rounded border box and the padding box with its inner
/// radii, and on one of `sides` (their join wedges).
pub fn border_ring_contains(
    point: Vec2D,
    border_box: Aabb,
    radii: Radii,
    padding: Aabb,
    inner: Radii,
    sides: u8,
) -> bool {
    overflow_clip_contains(point, border_box, radii, (true, true))
        && !overflow_clip_contains(point, padding, inner, (true, true))
        && (sides & 15 == 15
            || side_wedges(border_box, radii, padding)
                .iter()
                .enumerate()
                .any(|(side, wedge)| sides & (1 << side) != 0 && polygon_contains(wedge, point)))
}

/// A border's style (`BoxBorder.style`, docs/CSS_LAYOUT.md, 4.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
}

impl BorderStyle {
    pub fn from_value(value: u8) -> Self {
        match value {
            1 => Self::Dashed,
            2 => Self::Dotted,
            _ => Self::Solid,
        }
    }
}

/// One group of a dashed or dotted border's marks, drawn inside the
/// border's ring and the join wedges of `sides` (as [`box_border`] makes
/// them), as Chrome clips each side of such a border.
#[derive(Default, Debug)]
pub struct BorderMarks {
    pub sides: u8,
    /// Open sub-paths stroked with butt caps, each list at its thickness:
    /// one per side of a square box (top, right, bottom, left), one around
    /// the centre line of a rounded one.
    pub strokes: Vec<(RawPath, f32)>,
    /// Filled marks: round dots, and the whole square dots at the ends of
    /// a thin dotted side.
    pub fills: RawPath,
}

/// Chrome's dash fit (Blink's StyledStrokeData): the gap that gives the
/// number of dashes whose gaps are closest to `gap`.
fn best_dash_gap(length: f32, dash: f32, gap: f32, closed: bool) -> f32 {
    let available = if closed { length } else { length + gap };
    let fewer = (available / (dash + gap)).floor();
    let more = fewer + 1.0;
    let gaps = |dashes: f32| if closed { dashes } else { dashes - 1.0 };
    let fewer_gap = (length - fewer * dash) / gaps(fewer);
    let more_gap = (length - more * dash) / gaps(more);
    if more_gap <= 0.0 || (fewer_gap - gap).abs() < (more_gap - gap).abs() {
        fewer_gap
    } else {
        more_gap
    }
}

/// Whether Chrome strokes a border of `width` in `style` as dashes (with
/// butt ends): dashed, and dotted up to 3 points wide (square dots).
fn draws_as_dashes(width: f32, style: BorderStyle) -> bool {
    style == BorderStyle::Dashed || (style == BorderStyle::Dotted && width <= 3.0)
}

/// The dash and gap lengths Chrome strokes a `length`-long line or closed
/// centre line of `width` with, or None for a solid line (no room for
/// dashes).
fn dash_pattern(length: f32, width: f32, style: BorderStyle, closed: bool) -> Option<(f32, f32)> {
    let dashed = style == BorderStyle::Dashed;
    // Thin dashes are longer, with longer gaps, so they read as dashes.
    let (dash, gap) = if dashed {
        if width >= 3.0 {
            (width * 2.0, width)
        } else {
            (width * 3.0, width * 2.0)
        }
    } else {
        (width, width)
    };
    if length <= dash * 2.0 {
        return None;
    }
    let two_dashes = 2.0 * dash + gap + if closed { gap } else { 0.0 };
    if length <= two_dashes {
        let scale = length / two_dashes;
        return Some((dash * scale, gap * scale));
    }
    Some((
        dash,
        if dashed {
            best_dash_gap(length, dash, gap, closed)
        } else {
            gap
        },
    ))
}

/// Chrome draws round dots this far short of their period, so the last
/// dot of a line is whole.
const DOT_EPSILON: f32 = 1.0e-2;

/// A polyline with its points' distances along it, and how far inward each
/// point may be offset before the offset folds over (the radius of
/// curvature there; unbounded on straight parts).
struct Line {
    points: Vec<Vec2D>,
    distances: Vec<f32>,
    limits: Vec<f32>,
}

impl Line {
    fn new(points: Vec<Vec2D>) -> Self {
        let limits = vec![f32::INFINITY; points.len()];
        Self::with_limits(points, limits)
    }
    fn with_limits(points: Vec<Vec2D>, limits: Vec<f32>) -> Self {
        let mut distances = vec![0.0];
        for pair in points.windows(2) {
            let step = pair[1] - pair[0];
            distances.push(distances.last().unwrap() + (step.x * step.x + step.y * step.y).sqrt());
        }
        Self {
            points,
            distances,
            limits,
        }
    }
    fn length(&self) -> f32 {
        *self.distances.last().unwrap_or(&0.0)
    }
    /// The point at `distance`, and the segment it lies on.
    fn locate(&self, distance: f32) -> (Vec2D, usize) {
        let distance = distance.clamp(0.0, self.length());
        let index = self
            .distances
            .partition_point(|&d| d < distance)
            .clamp(1, self.points.len() - 1);
        let (from, to) = (self.distances[index - 1], self.distances[index]);
        let t = if to > from {
            (distance - from) / (to - from)
        } else {
            0.0
        };
        let (a, b) = (self.points[index - 1], self.points[index]);
        (
            Vec2D::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t),
            index,
        )
    }
    fn at(&self, distance: f32) -> Vec2D {
        self.locate(distance).0
    }
    /// The stretch from `start` to `end`: its points, each with the normal
    /// to its left (outward on a clockwise line, y down) and its limit.
    fn stretch(&self, start: f32, end: f32) -> Vec<(Vec2D, Vec2D, f32)> {
        let end = end.min(self.length());
        if !(end > start) {
            return Vec::new();
        }
        let normal = |segment: usize| {
            let step = self.points[segment] - self.points[segment - 1];
            let length = (step.x * step.x + step.y * step.y).sqrt();
            if length > 0.0 {
                Vec2D::new(step.y / length, -step.x / length)
            } else {
                Vec2D::default()
            }
        };
        let (first, first_segment) = self.locate(start);
        let (last, last_segment) = self.locate(end);
        let limit = |segment: usize| self.limits[segment - 1].min(self.limits[segment]);
        let mut points = vec![(first, normal(first_segment), limit(first_segment))];
        for index in first_segment..last_segment {
            if self.distances[index] > start && self.distances[index] < end {
                let (a, b) = (normal(index), normal(index + 1));
                let sum = Vec2D::new(a.x + b.x, a.y + b.y);
                let length = (sum.x * sum.x + sum.y * sum.y).sqrt();
                let normal = if length > 0.0 {
                    Vec2D::new(sum.x / length, sum.y / length)
                } else {
                    a
                };
                points.push((self.points[index], normal, self.limits[index]));
            }
        }
        points.push((last, normal(last_segment), limit(last_segment)));
        points
    }
    /// The stretch from `start` to `end` as one open sub-path of `centres`,
    /// and its outline `half` wide on either side as a filled shape (the
    /// inward side no further than each point's limit), as a butt-ended
    /// stroke covers it.
    fn add_stretch(
        &self,
        centres: &mut RawPath,
        fills: &mut RawPath,
        half: f32,
        start: f32,
        end: f32,
    ) {
        let points = self.stretch(start, end);
        if points.len() < 2 {
            return;
        }
        centres.move_to_point(points[0].0);
        for &(point, ..) in &points[1..] {
            centres.line_to_point(point);
        }
        let out = |&(point, normal, _): &(Vec2D, Vec2D, f32)| {
            Vec2D::new(point.x + normal.x * half, point.y + normal.y * half)
        };
        let inward = |&(point, normal, limit): &(Vec2D, Vec2D, f32)| {
            let depth = half.min(limit);
            Vec2D::new(point.x - normal.x * depth, point.y - normal.y * depth)
        };
        fills.move_to_point(out(&points[0]));
        for point in &points[1..] {
            fills.line_to_point(out(point));
        }
        for point in points.iter().rev() {
            fills.line_to_point(inward(point));
        }
        fills.close();
    }
    /// Dashes along the line from its start, or the whole line when
    /// `pattern` is None.
    fn add_dashes(
        &self,
        centres: &mut RawPath,
        fills: &mut RawPath,
        half: f32,
        pattern: Option<(f32, f32)>,
    ) {
        let Some((dash, gap)) = pattern else {
            self.add_stretch(centres, fills, half, 0.0, self.length());
            return;
        };
        let period = dash + gap;
        if !(period > 0.0) {
            return;
        }
        let mut start = 0.0;
        while start < self.length() {
            self.add_stretch(centres, fills, half, start, start + dash);
            start += period;
        }
    }
    /// Round dots of `diameter` every `period` from the line's start, up to
    /// its end.
    fn add_dots(&self, path: &mut RawPath, diameter: f32, period: f32) {
        let mut at = 0.0;
        loop {
            add_circle(path, self.at(at), diameter / 2.0);
            at += period;
            if !(period > 0.0) || at > self.length() + 1.0e-4 {
                break;
            }
        }
    }
}

/// A round dot, clockwise from its top: a fine polygon of the circle's
/// area, so a renderer's own curve tessellation does not shrink small dots.
fn add_circle(path: &mut RawPath, center: Vec2D, radius: f32) {
    const SIDES: usize = 48;
    let step = std::f32::consts::TAU / SIDES as f32;
    // A regular polygon of the circle's area.
    let radius = radius * (std::f32::consts::TAU / (SIDES as f32 * step.sin())).sqrt();
    for side in 0..SIDES {
        let angle = side as f32 * step;
        let point = Vec2D::new(
            center.x + radius * angle.sin(),
            center.y - radius * angle.cos(),
        );
        if side == 0 {
            path.move_to_point(point);
        } else {
            path.line_to_point(point);
        }
    }
    path.close();
}

fn add_rect(path: &mut RawPath, left: f32, top: f32, right: f32, bottom: f32) {
    path.move_to_point(Vec2D::new(left, top));
    path.line_to_point(Vec2D::new(right, top));
    path.line_to_point(Vec2D::new(right, bottom));
    path.line_to_point(Vec2D::new(left, bottom));
    path.close();
}

/// Chrome's whole dots at the ends of a thin dotted line (Blink's
/// EnforceDotsAtEndpoints): for whole widths 1 to 3, by the line's length
/// in whole points, it draws a square dot at either end itself, growing it
/// or moving the line's start and end, so the dots meet the corners.
/// Returns the line's new ends.
fn enforce_end_dots(
    fills: &mut RawPath,
    start: Vec2D,
    end: Vec2D,
    length: f32,
    width: f32,
) -> (Vec2D, Vec2D) {
    let (width, length) = (width.round() as i32, length.round() as i32);
    let (mod_4, mod_6) = (length % 4, length % 6);
    let (mut start_dot, mut start_growth, mut start_offset) = (false, 0, 0);
    let (mut end_dot, mut end_growth) = (false, 0);
    if (width == 1 && length % 2 == 0) || (width == 3 && mod_6 == 0) {
        start_dot = true;
        start_growth = 1;
        start_offset = 1;
    }
    if (width == 2 && (mod_4 == 0 || mod_4 == 1)) || (width == 3 && (mod_6 == 1 || mod_6 == 2)) {
        start_dot = true;
        start_offset = -1;
    }
    if (width == 2 && mod_4 == 0) || (width == 3 && mod_6 == 1) {
        end_dot = true;
    }
    if (width == 2 && mod_4 == 3) || (width == 3 && (mod_6 == 4 || mod_6 == 5)) {
        start_dot = true;
        start_offset = 1;
    }
    if width == 3 && mod_6 == 5 {
        end_dot = true;
    } else if width == 3 && mod_6 == 0 {
        end_dot = true;
        end_growth = 1;
    }
    let vertical = start.x == end.x;
    let w = width as f32;
    let half = w / 2.0;
    let (mut start, mut end) = (start, end);
    if start_dot {
        let grow = start_growth as f32;
        if vertical {
            add_rect(
                fills,
                start.x - half,
                start.y,
                start.x + half,
                start.y + w + grow,
            );
            start.y += 2.0 * w + start_offset as f32;
        } else {
            add_rect(
                fills,
                start.x,
                start.y - half,
                start.x + w + grow,
                start.y + half,
            );
            start.x += 2.0 * w + start_offset as f32;
        }
    }
    if end_dot {
        let grow = end_growth as f32;
        if vertical {
            add_rect(fills, end.x - half, end.y - w - grow, end.x + half, end.y);
            end.y -= w + grow + 1.0;
        } else {
            add_rect(fills, end.x - w - grow, end.y - half, end.x, end.y + half);
            end.x -= w + grow + 1.0;
        }
    }
    (start, end)
}

/// One straight side's marks, as Chrome strokes a side of a square box: a
/// line down the side's middle across its whole outer length, from its
/// top or left end, `width` wide.
fn add_side_line(
    strokes: &mut RawPath,
    fills: &mut RawPath,
    start: Vec2D,
    end: Vec2D,
    width: f32,
    style: BorderStyle,
) {
    let length = (end.x - start.x) + (end.y - start.y);
    if length <= 0.0 || width <= 0.0 {
        return;
    }
    if draws_as_dashes(width, style) {
        let pattern = dash_pattern(length, width, style, false);
        let (start, end) = if style == BorderStyle::Dotted && pattern.is_some() {
            enforce_end_dots(fills, start, end, length, width)
        } else {
            (start, end)
        };
        Line::new(vec![start, end]).add_dashes(strokes, fills, width / 2.0, pattern);
        return;
    }
    // Round dots: their caps reach past the ends, so the line starts and
    // ends half a width in.
    let along = Vec2D::new((end.x - start.x) / length, (end.y - start.y) / length);
    let half = width / 2.0;
    let line = Line::new(vec![
        Vec2D::new(start.x + along.x * half, start.y + along.y * half),
        Vec2D::new(end.x - along.x * half, end.y - along.y * half),
    ]);
    let period = if length < 2.0 * width {
        2.0 * width
    } else {
        best_dash_gap(length, width, width, false) + width - DOT_EPSILON
    };
    line.add_dots(fills, width, period);
}

/// A rounded box's border centre line: the border box inset by half of
/// each side's width, its radii reduced to match, as one closed polyline
/// starting at the top side's left end and running clockwise, where
/// Chrome's rounded border path starts.
fn centre_line(border_box: Aabb, radii: Radii, border: [f32; 4]) -> Line {
    let [top, right, bottom, left] = border;
    let rect = Aabb::new(
        border_box.left() + left / 2.0,
        border_box.top() + top / 2.0,
        border_box.right() - right / 2.0,
        border_box.bottom() - bottom / 2.0,
    );
    let radii = inner_radii(radii, border.map(|width| width / 2.0));
    // Each point, how far inward it may be offset (unbounded on the straight
    // sides, the corner's least radius of curvature on its curve), and its
    // distance along the line as Skia measures it: a curve by the chords
    // Skia's path measure splits it into (halves until a chord's middle is
    // within half a point of the curve's), which is what Chrome's dashes are
    // fitted and placed by.
    let mut points: Vec<(Vec2D, f32, f32)> = Vec::new();
    let add_straight = |points: &mut Vec<(Vec2D, f32, f32)>, point: Vec2D| {
        let distance = points
            .last()
            .map_or(0.0, |&(last, _, distance): &(Vec2D, f32, f32)| {
                let step = point - last;
                distance + (step.x * step.x + step.y * step.y).sqrt()
            });
        points.push((point, f32::INFINITY, distance));
    };
    // A quarter ellipse about `center` from angle `from` (radians, y down).
    let arc = |points: &mut Vec<(Vec2D, f32, f32)>, center: Vec2D, radius: Vec2D, from: f32| {
        let reach = radius.x.max(radius.y);
        if !(reach > 0.0) {
            return;
        }
        let limit = radius.x.min(radius.y).powi(2) / reach;
        let on = |angle: f32| {
            Vec2D::new(
                center.x + radius.x * angle.cos(),
                center.y + radius.y * angle.sin(),
            )
        };
        let mut pieces = Vec::new();
        let mut pending = vec![(from, from + std::f32::consts::FRAC_PI_2)];
        while let Some((start, end)) = pending.pop() {
            let (a, b, middle) = (on(start), on(end), on((start + end) / 2.0));
            let deviation = (middle.x - (a.x + b.x) / 2.0)
                .abs()
                .max((middle.y - (a.y + b.y) / 2.0).abs());
            if deviation > 0.5 && end - start > 1.0e-3 {
                let half = (start + end) / 2.0;
                pending.push((half, end));
                pending.push((start, half));
            } else {
                pieces.push((start, end));
            }
        }
        for (start, end) in pieces {
            let (a, b) = (on(start), on(end));
            let chord = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
            let base = points.last().map_or(0.0, |point| point.2);
            let steps = ((chord / 0.25).ceil() as usize).max(2);
            for step in 1..=steps {
                let t = step as f32 / steps as f32;
                points.push((on(start + (end - start) * t), limit, base + chord * t));
            }
        }
    };
    let pi = std::f32::consts::PI;
    let [tl, tr, br, bl] = radii;
    add_straight(&mut points, Vec2D::new(rect.left() + tl.x, rect.top()));
    add_straight(&mut points, Vec2D::new(rect.right() - tr.x, rect.top()));
    arc(
        &mut points,
        Vec2D::new(rect.right() - tr.x, rect.top() + tr.y),
        tr,
        1.5 * pi,
    );
    add_straight(&mut points, Vec2D::new(rect.right(), rect.bottom() - br.y));
    arc(
        &mut points,
        Vec2D::new(rect.right() - br.x, rect.bottom() - br.y),
        br,
        0.0,
    );
    add_straight(&mut points, Vec2D::new(rect.left() + bl.x, rect.bottom()));
    arc(
        &mut points,
        Vec2D::new(rect.left() + bl.x, rect.bottom() - bl.y),
        bl,
        0.5 * pi,
    );
    add_straight(&mut points, Vec2D::new(rect.left(), rect.top() + tl.y));
    arc(
        &mut points,
        Vec2D::new(rect.left() + tl.x, rect.top() + tl.y),
        tl,
        pi,
    );
    points.dedup_by(|a, b| {
        let same = (a.0.x - b.0.x).abs() < 1.0e-5 && (a.0.y - b.0.y).abs() < 1.0e-5;
        if same {
            b.1 = b.1.min(a.1);
        }
        same
    });
    let mut line = Line {
        points: Vec::with_capacity(points.len()),
        distances: Vec::with_capacity(points.len()),
        limits: Vec::with_capacity(points.len()),
    };
    for (point, limit, distance) in points {
        line.points.push(point);
        line.limits.push(limit);
        line.distances.push(distance);
    }
    line
}

/// A dashed or dotted border's marks over `sides` (docs/CSS_LAYOUT.md,
/// 4.2), as Chrome 155 draws them. A side of a square box is a line down
/// its middle across its outer length, with dashes or dots sized from its
/// width and fitted to its length. On a rounded box, a side whose inner
/// corners are rounded follows the whole centre line instead, fitted to
/// its length, drawn thick and cut to the ring. Dashes are two widths long
/// with one-width gaps (three and two below 3 points), dots up to 3 points
/// wide are square and wider ones round. None for a solid border.
pub fn border_marks(
    border_box: Aabb,
    radii: Radii,
    border: [f32; 4],
    sides: u8,
    style: BorderStyle,
) -> Vec<BorderMarks> {
    if style == BorderStyle::Solid {
        return Vec::new();
    }
    let rounded_corner = |radius: Vec2D| radius.x > 0.0 && radius.y > 0.0;
    let rounded = radii.iter().any(|&radius| rounded_corner(radius));
    let inner = inner_radii(radii, border);
    let inner_rounded = inner.iter().any(|&radius| rounded_corner(radius));
    let (left, top, right, bottom) = (
        border_box.left(),
        border_box.top(),
        border_box.right(),
        border_box.bottom(),
    );
    let [top_width, right_width, bottom_width, left_width] = border;
    let mut straight = BorderMarks::default();
    // The curved sides by width: their sides and the thickest of their
    // cuts. A group's marks are built once that thickness is known.
    let mut curved: Vec<(f32, u8, f32)> = Vec::new();
    for side in 0..4 {
        let width = border[side];
        if sides & (1 << side) == 0 || !(width > 0.0) {
            continue;
        }
        // Side n runs from corner n to corner n + 1.
        let is_curved = rounded
            && (!inner_rounded
                || rounded_corner(inner[side])
                || rounded_corner(inner[(side + 1) % 4]));
        if !is_curved {
            straight.sides |= 1 << side;
            let (start, end) = match side {
                0 => (
                    Vec2D::new(left, top + top_width / 2.0),
                    Vec2D::new(right, top + top_width / 2.0),
                ),
                1 => (
                    Vec2D::new(right - right_width / 2.0, top),
                    Vec2D::new(right - right_width / 2.0, bottom),
                ),
                2 => (
                    Vec2D::new(left, bottom - bottom_width / 2.0),
                    Vec2D::new(right, bottom - bottom_width / 2.0),
                ),
                _ => (
                    Vec2D::new(left + left_width / 2.0, top),
                    Vec2D::new(left + left_width / 2.0, bottom),
                ),
            };
            let mut path = RawPath::default();
            add_side_line(&mut path, &mut straight.fills, start, end, width, style);
            straight.strokes.push((path, width));
            continue;
        }
        // The thick stroke Chrome cuts to the ring: 2.2 times the widest of
        // the side and its neighbours.
        let thickness = 2.2
            * width
                .max(border[(side + 1) % 4])
                .max(border[(side + 3) % 4]);
        match curved.iter_mut().find(|(group, _, _)| *group == width) {
            Some((_, group_sides, group_thickness)) => {
                *group_sides |= 1 << side;
                *group_thickness = group_thickness.max(thickness);
            }
            None => curved.push((width, 1 << side, thickness)),
        }
    }
    let curved = curved.into_iter().map(|(width, sides, thickness)| {
        let line = centre_line(border_box, radii, border);
        let mut marks = BorderMarks {
            sides,
            ..BorderMarks::default()
        };
        let length = line.length().floor();
        if draws_as_dashes(width, style) {
            let mut path = RawPath::default();
            line.add_dashes(
                &mut path,
                &mut marks.fills,
                thickness / 2.0,
                dash_pattern(length, width, style, true),
            );
            marks.strokes.push((path, thickness));
        } else {
            let period = if length < 2.0 * width {
                2.0 * width
            } else {
                best_dash_gap(length, width, width, true) + width - DOT_EPSILON
            };
            line.add_dots(&mut marks.fills, width, period);
        }
        marks
    });
    let mut groups = Vec::new();
    if straight.sides != 0 {
        groups.push(straight);
    }
    groups.extend(curved);
    groups
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
    // Until mixed values arrive (docs/CSS_LAYOUT.md, 1.2), a value made
    // mixed at run time resolves as its percentage alone, as layout's do.
    let part = |key: u16, basis: f32| {
        let percent = values.number(key + 1);
        if percent != 0.0 {
            percent / 100.0 * basis
        } else {
            values.number(key)
        }
    };
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
    // The ellipse through `corner` with the aspect ratio of `side`. A side
    // with no width or no height gives an ellipse with none either, which
    // the degenerate rule below draws.
    let through = |corner: Vec2D, side: Vec2D| {
        if circle {
            let radius = corner_distance(corner);
            Vec2D::new(radius, radius)
        } else if side.x <= 0.0 {
            Vec2D::new(0.0, corner.y)
        } else if side.y <= 0.0 {
            Vec2D::new(corner.x, 0.0)
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
    if !radii.x.is_finite() || !radii.y.is_finite() {
        return None;
    }
    // A zero-sized ending shape draws as CSS draws it (CSS Images 3,
    // "Degenerate Radial Gradients"): a circle of a vanishing radius, or an
    // ellipse vanishingly thin on its zero axis and very long on the other.
    const VANISHING: f32 = 1.0e-3;
    const VAST: f32 = 1.0e6;
    let radii = match (radii.x > 0.0, radii.y > 0.0) {
        (true, true) => radii,
        (false, false) if circle => Vec2D::new(VANISHING, VANISHING),
        // Zero width, whatever the height, before zero height.
        (false, _) => Vec2D::new(VANISHING, VAST),
        (true, false) => Vec2D::new(VAST, VANISHING),
    };
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

    /// A radial gradient whose ending shape is zero-sized is drawn as CSS
    /// draws it, a vanishingly small circle (the last stop's colour), not
    /// with the Rive gradient's authored ends.
    #[test]
    fn zero_radius_gradients_stay_box_gradients() {
        use crate::css_layout::{CssValues, keys};
        let (first, count) = keys::range(keys::BOX_GRADIENT).unwrap();
        let mut values: Vec<f32> = (first..first + count as u16)
            .map(keys::default_value)
            .collect();
        // An explicit circle of radius 0.
        values[usize::from(keys::GRADIENT_SHAPE - first)] = 1.0;
        values[usize::from(keys::GRADIENT_SIZE - first)] = 4.0;
        let geometry = gradient_geometry(
            CssValues {
                first,
                values: &values,
            },
            Aabb::new(0.0, 0.0, 100.0, 100.0),
            true,
        )
        .expect("a box gradient");
        assert_eq!(geometry.start, Vec2D::new(50.0, 50.0));
        let radius = geometry.end.x - geometry.start.x;
        assert!(radius > 0.0 && radius < 0.01, "{radius}");
    }

    /// A closest-corner ellipse whose closest side has no width (its centre
    /// on the left edge) is CSS's zero-width degenerate ellipse: vanishingly
    /// thin and very tall, not the Rive gradient's authored ends.
    #[test]
    fn zero_width_closest_corner_ellipses_are_degenerate() {
        use crate::css_layout::{CssValues, keys};
        let (first, count) = keys::range(keys::BOX_GRADIENT).unwrap();
        let mut values: Vec<f32> = (first..first + count as u16)
            .map(keys::default_value)
            .collect();
        values[usize::from(keys::GRADIENT_SIZE - first)] = 2.0;
        values[usize::from(keys::GRADIENT_CENTER_X + 1 - first)] = 0.0;
        let geometry = gradient_geometry(
            CssValues {
                first,
                values: &values,
            },
            Aabb::new(0.0, 0.0, 100.0, 100.0),
            true,
        )
        .expect("a box gradient");
        assert_eq!(geometry.start, Vec2D::new(0.0, 50.0));
        let radius = geometry.end.x - geometry.start.x;
        assert!(radius > 0.0 && radius < 0.01, "{radius}");
        // Scaled vertically to the very long axis.
        assert!(
            geometry.transform.values()[3] > 1.0e6,
            "{:?}",
            geometry.transform
        );
    }

    /// Each stroked sub-path of `path` as its first and last points.
    fn runs(path: &RawPath) -> Vec<(Vec2D, Vec2D)> {
        let mut runs = Vec::new();
        let mut start: Option<Vec2D> = None;
        let mut last = Vec2D::default();
        for segment in path.segments() {
            let point = segment.points.last().copied().unwrap_or_default();
            if matches!(
                segment.verb,
                crate::source::math::path_types::PathVerb::Move
            ) {
                if let Some(start) = start {
                    runs.push((start, last));
                }
                start = Some(point);
            }
            last = point;
        }
        if let Some(start) = start {
            runs.push((start, last));
        }
        runs
    }
    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.02
    }

    /// `border: 6px dashed` on a 112 by 72 border box, as Chrome measures
    /// it: 12-point dashes, seven along the top with 4.67-point gaps, four
    /// down the left with 8-point gaps, each side from its outer corner.
    #[test]
    fn dashes_are_fitted_to_each_side_as_chrome_fits_them() {
        let bounds = Aabb::new(0.0, 0.0, 112.0, 72.0);
        let marks = border_marks(
            bounds,
            [Vec2D::default(); 4],
            [6.0; 4],
            15,
            BorderStyle::Dashed,
        );
        let strokes = &marks[0].strokes;
        assert_eq!(strokes.len(), 4);
        let top = runs(&strokes[0].0);
        assert_eq!(strokes[0].1, 6.0);
        assert_eq!(top.len(), 7);
        for (index, (start, end)) in top.iter().enumerate() {
            let x = index as f32 * (12.0 + 28.0 / 6.0);
            assert!(near(start.x, x) && near(end.x, x + 12.0), "{top:?}");
            assert!(near(start.y, 3.0) && near(end.y, 3.0));
        }
        let left = runs(&strokes[3].0);
        assert_eq!(left.len(), 4);
        for (index, (start, end)) in left.iter().enumerate() {
            let y = index as f32 * 20.0;
            assert!(near(start.y, y) && near(end.y, y + 12.0), "{left:?}");
            assert!(near(start.x, 3.0));
        }
    }

    /// `border: 2px dashed`: dashes three widths long, two-width gaps
    /// fitted (eleven 6-point dashes along a 104-point top).
    #[test]
    fn thin_dashes_are_longer_than_thick_ones() {
        let bounds = Aabb::new(0.0, 0.0, 104.0, 64.0);
        let marks = border_marks(
            bounds,
            [Vec2D::default(); 4],
            [2.0; 4],
            15,
            BorderStyle::Dashed,
        );
        let top = runs(&marks[0].strokes[0].0);
        assert_eq!(top.len(), 11);
        assert!(near(top[1].0.x, 9.8) && near(top[10].1.x, 104.0), "{top:?}");
    }

    /// `border: 2px dotted`: square dots, the first and last drawn whole at
    /// the ends (Chrome's endpoint rule for a 104-point line), the rest
    /// two on, two off from x 3 to 101.
    #[test]
    fn thin_dots_keep_whole_dots_at_the_ends() {
        let bounds = Aabb::new(0.0, 0.0, 104.0, 64.0);
        let marks = border_marks(
            bounds,
            [Vec2D::default(); 4],
            [2.0; 4],
            15,
            BorderStyle::Dotted,
        );
        let top = runs(&marks[0].strokes[0].0);
        assert!(near(top[0].0.x, 3.0) && near(top[0].1.x, 5.0), "{top:?}");
        assert!(near(top.last().unwrap().1.x, 101.0), "{top:?}");
        let fills = marks[0].fills.bounds();
        assert!(
            near(fills.left(), 0.0) && near(fills.right(), 104.0),
            "{fills:?}"
        );
    }

    /// `border: 8px dotted`: round 8-point dots, eight along a 116-point
    /// top, centred from x 4 to x 112.
    #[test]
    fn thick_dots_are_round_and_evenly_spaced() {
        let bounds = Aabb::new(0.0, 0.0, 116.0, 76.0);
        let marks = border_marks(
            bounds,
            [Vec2D::default(); 4],
            [8.0; 4],
            15,
            BorderStyle::Dotted,
        );
        assert!(marks[0].strokes.iter().all(|(path, _)| path.empty()));
        // Every dot is a circle: count the top row's by their centres.
        let dots = runs(&marks[0].fills);
        // The left and right sides' first dots sit on the top's end dots.
        let mut top: Vec<f32> = dots
            .iter()
            .filter(|(start, _)| near(start.y, 0.0))
            .map(|(start, _)| start.x)
            .collect();
        top.sort_by(f32::total_cmp);
        top.dedup_by(|a, b| (*a - *b).abs() < 0.1);
        assert_eq!(top.len(), 8, "{dots:?}");
        assert!(near(top[0], 4.0) && (top[7] - 112.0).abs() < 0.1, "{top:?}");
    }

    /// `border: 4px dashed` with 20-point corners: 8-point dashes fitted
    /// around the whole centre line, 304 points as Skia measures it (25
    /// dashes, 4.16-point gaps), from the top edge's left end, clockwise,
    /// where Chrome starts.
    #[test]
    fn rounded_dashes_follow_the_whole_centre_line() {
        let bounds = Aabb::new(0.0, 0.0, 108.0, 68.0);
        let marks = border_marks(
            bounds,
            [Vec2D::new(20.0, 20.0); 4],
            [4.0; 4],
            15,
            BorderStyle::Dashed,
        );
        assert_eq!(marks.len(), 1);
        let (path, thickness) = &marks[0].strokes[0];
        assert!(near(*thickness, 4.0 * 2.2));
        // The fit uses the length in whole points, as Chrome's does, so a
        // sliver of a 26th dash may close the line where the first starts.
        let dashes: Vec<_> = runs(path)
            .into_iter()
            .filter(|(start, end)| (end.x - start.x).abs() + (end.y - start.y).abs() > 1.0)
            .collect();
        assert_eq!(dashes.len(), 25);
        assert!(
            near(dashes[0].0.x, 20.0) && near(dashes[0].0.y, 2.0),
            "{:?}",
            dashes[0]
        );
        assert!(
            near(dashes[0].1.x, 28.0) && near(dashes[0].1.y, 2.0),
            "{:?}",
            dashes[0]
        );
        assert!(near(dashes[1].0.x, 32.16), "{:?}", dashes[1]);
    }

    /// Equal-width curved sides share one centre line, drawn at the thickest
    /// of their cuts: with top and right 4 points wide beside a 40-point
    /// bottom, the right side's cut (2.2 times 40) sets the group's
    /// thickness, and its dash outlines are built at it, not at the top's.
    #[test]
    fn grouped_rounded_dashes_are_built_at_the_group_thickness() {
        let marks = border_marks(
            Aabb::new(0.0, 0.0, 100.0, 100.0),
            [Vec2D::new(45.0, 45.0); 4],
            [4.0, 4.0, 40.0, 1.0],
            15,
            BorderStyle::Dashed,
        );
        let group = marks
            .iter()
            .find(|marks| marks.sides == 0b0011)
            .expect("top and right share a group");
        let (centres, thickness) = &group.strokes[0];
        assert!(near(*thickness, 88.0), "{thickness}");
        // A dash outline reaches half the thickness out from the centre line
        // at the line's extremes, on every side of the box.
        let (centre, outline) = (centres.bounds(), group.fills.bounds());
        let reach = 0.9 * thickness / 2.0;
        assert!(
            outline.left() <= centre.left() - reach
                && outline.top() <= centre.top() - reach
                && outline.right() >= centre.right() + reach
                && outline.bottom() >= centre.bottom() + reach,
            "outline {outline:?}, centre line {centre:?}"
        );
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
