use nuxie_render_api::{Aabb, FillRule, PathDirection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitTestArea {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl HitTestArea {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn around(x: f32, y: f32, radius: f32) -> Self {
        Self::new(
            graphics_round(x - radius),
            graphics_round(y - radius),
            graphics_round(x + radius),
            graphics_round(y + radius),
        )
    }

    fn width(self) -> i32 {
        self.right.saturating_sub(self.left)
    }

    fn height(self) -> i32 {
        self.bottom.saturating_sub(self.top)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn midpoint(self, other: Self) -> Self {
        Self {
            // C++ `ave` calls `lerp(a, b, .5)`, grouped as
            // `a * (1 - t) + b * t`.
            x: self.x * 0.5 + other.x * 0.5,
            y: self.y * 0.5 + other.y * 0.5,
        }
    }
}

/// Direct owner for pinned C++ `src/math/hit_test.cpp` state and path raster.
#[derive(Debug)]
pub(crate) struct HitTester {
    delta_windings: Vec<i32>,
    first: Point,
    previous: Point,
    offset: Point,
    height: f32,
    width_i32: i32,
    height_i32: i32,
    expects_move: bool,
}

impl HitTester {
    pub(crate) fn new(area: HitTestArea) -> Self {
        let mut tester = Self {
            delta_windings: Vec::new(),
            first: Point::default(),
            previous: Point::default(),
            offset: Point::default(),
            height: 0.0,
            width_i32: 0,
            height_i32: 0,
            expects_move: true,
        };
        tester.reset(area);
        tester
    }

    pub(crate) fn reset(&mut self, area: HitTestArea) {
        self.offset = Point {
            x: area.left as f32,
            y: area.top as f32,
        };
        self.height = area.height() as f32;
        self.width_i32 = area.width();
        self.height_i32 = area.height();
        let length = usize::try_from(self.width_i32)
            .ok()
            .zip(usize::try_from(self.height_i32).ok())
            .and_then(|(width, height)| width.checked_mul(height))
            .unwrap_or(0);
        self.delta_windings.clear();
        self.delta_windings.resize(length, 0);
        self.expects_move = true;
    }

    /// Pinned no-argument `HitTester::reset` clears only the accumulated
    /// winding storage.
    pub(crate) fn clear_windings(&mut self) {
        self.delta_windings.clear();
    }

    pub(crate) fn move_to(&mut self, point: (f32, f32)) {
        if !self.expects_move {
            self.close();
        }
        let point = Point {
            x: point.0 - self.offset.x,
            y: point.1 - self.offset.y,
        };
        self.first = point;
        self.previous = point;
        self.expects_move = false;
    }

    pub(crate) fn line_to(&mut self, point: (f32, f32)) {
        if self.expects_move {
            return;
        }
        let point = Point {
            x: point.0 - self.offset.x,
            y: point.1 - self.offset.y,
        };
        clip_line(
            self.height,
            self.previous,
            point,
            &mut self.delta_windings,
            self.width_i32,
        );
        self.previous = point;
    }

    /// Literal owner for pinned `HitTester::quad`.
    ///
    /// The upstream implementation intentionally does not rasterize the
    /// quadratic; it only advances `m_Prev` to the un-offset endpoint. Keep
    /// that unusual behavior instead of replacing it with a curve flattener.
    pub(crate) fn quad_to(&mut self, _control: (f32, f32), end: (f32, f32)) {
        if self.expects_move {
            return;
        }
        self.previous = Point { x: end.0, y: end.1 };
    }

    pub(crate) fn cubic_to(&mut self, control1: (f32, f32), control2: (f32, f32), end: (f32, f32)) {
        if self.expects_move {
            return;
        }
        let control1 = self.offset_point(control1);
        let control2 = self.offset_point(control2);
        let end = self.offset_point(end);
        if quick_reject_cubic(self.height, self.previous, control1, control2, end) {
            self.previous = end;
            return;
        }
        let count = compute_cubic_segments(self.previous, control1, control2, end);
        self.recurse_cubic(control1, control2, end, count);
    }

    fn recurse_cubic(&mut self, control1: Point, control2: Point, end: Point, count: i32) {
        if quick_reject_cubic(self.height, self.previous, control1, control2, end) {
            self.previous = end;
            return;
        }
        if count > 16 {
            let ab = self.previous.midpoint(control1);
            let bc = control1.midpoint(control2);
            let cd = control2.midpoint(end);
            let abc = ab.midpoint(bc);
            let bcd = bc.midpoint(cd);
            let midpoint = abc.midpoint(bcd);
            let next_count = count.saturating_add(1) >> 1;
            self.recurse_cubic(ab, abc, midpoint, next_count);
            self.recurse_cubic(bcd, cd, end, next_count);
            return;
        }

        let coefficient = CubicCoefficient::new(self.previous, control1, control2, end);
        let delta = 1.0 / count as f32;
        let mut t = delta;
        let mut previous = self.previous;
        for _ in 1..count.saturating_sub(1) {
            let next = coefficient.evaluate(t);
            clip_line(
                self.height,
                previous,
                next,
                &mut self.delta_windings,
                self.width_i32,
            );
            previous = next;
            t += delta;
        }
        clip_line(
            self.height,
            previous,
            end,
            &mut self.delta_windings,
            self.width_i32,
        );
        self.previous = end;
    }

    pub(crate) fn close(&mut self) {
        if self.expects_move {
            return;
        }
        clip_line(
            self.height,
            self.previous,
            self.first,
            &mut self.delta_windings,
            self.width_i32,
        );
        self.expects_move = true;
    }

    pub(crate) fn add_rect(
        &mut self,
        bounds: Aabb,
        transform: crate::Mat2D,
        direction: PathDirection,
    ) {
        let points = [
            transform.transform_point(bounds.min_x, bounds.min_y),
            transform.transform_point(bounds.max_x, bounds.min_y),
            transform.transform_point(bounds.max_x, bounds.max_y),
            transform.transform_point(bounds.min_x, bounds.max_y),
        ];
        self.move_to(points[0]);
        if direction == PathDirection::Clockwise {
            self.line_to(points[1]);
            self.line_to(points[2]);
            self.line_to(points[3]);
        } else {
            self.line_to(points[3]);
            self.line_to(points[2]);
            self.line_to(points[1]);
        }
        self.close();
    }

    pub(crate) fn test(&mut self, fill_rule: FillRule) -> bool {
        if !self.expects_move {
            self.close();
        }
        let mask = if fill_rule == FillRule::NonZero {
            -1
        } else {
            1
        };
        self.delta_windings
            .iter()
            .fold(0, |nonzero, winding| nonzero | (winding & mask))
            != 0
    }

    fn offset_point(&self, point: (f32, f32)) -> Point {
        Point {
            x: point.0 - self.offset.x,
            y: point.1 - self.offset.y,
        }
    }

    pub(crate) fn test_mesh_point(
        point: (f32, f32),
        vertices: &[(f32, f32)],
        indices: &[u16],
    ) -> bool {
        if vertices.len() < 3 {
            return false;
        }
        let Some((left, top, right, bottom)) = mesh_bounds(vertices) else {
            return false;
        };
        if bottom < point.1 || point.1 < top || right < point.0 || point.0 < left {
            return false;
        }

        for triangle in indices.chunks_exact(3) {
            let Some(&a) = vertices.get(usize::from(triangle[0])) else {
                return false;
            };
            let Some(&b) = vertices.get(usize::from(triangle[1])) else {
                return false;
            };
            let Some(&c) = vertices.get(usize::from(triangle[2])) else {
                return false;
            };
            let pa = (a.0 - point.0, a.1 - point.1);
            let pb = (b.0 - point.0, b.1 - point.1);
            let pc = (c.0 - point.0, c.1 - point.1);
            let ab = cross_lt(pa, pb);
            let bc = cross_lt(pb, pc);
            let ca = cross_lt(pc, pa);
            if ab == bc && ab == ca {
                return true;
            }
        }
        false
    }

    pub(crate) fn test_mesh_area(
        area: HitTestArea,
        vertices: &[(f32, f32)],
        indices: &[u16],
    ) -> bool {
        if area.width().wrapping_mul(area.height()) == 1 {
            return Self::test_mesh_point((area.left as f32, area.top as f32), vertices, indices);
        }
        if vertices.len() < 3 {
            return false;
        }
        let Some((left, top, right, bottom)) = mesh_bounds(vertices) else {
            return false;
        };
        if bottom <= area.top as f32
            || area.bottom as f32 <= top
            || right <= area.left as f32
            || area.right as f32 <= left
        {
            return false;
        }

        let length = usize::try_from(area.width())
            .ok()
            .zip(usize::try_from(area.height()).ok())
            .and_then(|(width, height)| width.checked_mul(height))
            .unwrap_or(0);
        let mut windings = vec![0_i32; length];
        let offset = (area.left as f32, area.top as f32);
        for triangle in indices.chunks_exact(3) {
            let Some(&a) = vertices.get(usize::from(triangle[0])) else {
                return false;
            };
            let Some(&b) = vertices.get(usize::from(triangle[1])) else {
                return false;
            };
            let Some(&c) = vertices.get(usize::from(triangle[2])) else {
                return false;
            };
            let a = Point {
                x: a.0 - offset.0,
                y: a.1 - offset.1,
            };
            let b = Point {
                x: b.0 - offset.0,
                y: b.1 - offset.1,
            };
            let c = Point {
                x: c.0 - offset.0,
                y: c.1 - offset.1,
            };
            clip_line(area.height() as f32, a, b, &mut windings, area.width());
            clip_line(area.height() as f32, b, c, &mut windings, area.width());
            clip_line(area.height() as f32, c, a, &mut windings, area.width());
            if windings
                .iter()
                .fold(0_i32, |value, winding| value | winding)
                != 0
            {
                return true;
            }
        }
        false
    }
}

fn cross_lt(a: (f32, f32), b: (f32, f32)) -> bool {
    a.0 * b.1 < a.1 * b.0
}

fn mesh_bounds(vertices: &[(f32, f32)]) -> Option<(f32, f32, f32, f32)> {
    let &(mut left, mut top) = vertices.first()?;
    let mut right = left;
    let mut bottom = top;
    for &(x, y) in &vertices[1..] {
        // `AABB(Span<Vec2D>)` uses `std::min/std::max`, whose first-operand
        // NaN behavior differs from Rust's `f32::{min,max}`.
        left = cpp_min(left, x);
        top = cpp_min(top, y);
        right = cpp_max(right, x);
        bottom = cpp_max(bottom, y);
    }
    Some((left, top, right, bottom))
}

fn cpp_min(first: f32, second: f32) -> f32 {
    if second < first { second } else { first }
}

fn cpp_max(first: f32, second: f32) -> f32 {
    if first < second { second } else { first }
}

fn graphics_round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

fn clip_line(height: f32, mut start: Point, mut end: Point, delta: &mut [i32], width: i32) {
    if start.y == end.y || width <= 0 {
        return;
    }
    let mut winding = 1;
    if start.y > end.y {
        winding = -1;
        std::mem::swap(&mut start, &mut end);
    }
    if end.y <= 0.0 || start.y >= height {
        return;
    }
    let slope = (end.x - start.x) / (end.y - start.y);
    if start.y < 0.0 {
        start.x += slope * -start.y;
        start.y = 0.0;
    }
    if end.y > height {
        end.x += slope * (height - end.y);
        end.y = height;
    }
    append_line(height, start, end, slope, winding, delta, width);
}

fn append_line(
    height: f32,
    start: Point,
    end: Point,
    slope: f32,
    winding: i32,
    delta: &mut [i32],
    width: i32,
) {
    let top = graphics_round(start.y);
    let bottom = graphics_round(end.y);
    if top == bottom || top < 0 || bottom as f32 > height {
        return;
    }
    let mut x = start.x + slope * (top as f32 - start.y + 0.5) + 0.5;
    for y in top..bottom {
        let column = x.max(0.0) as i32;
        if column < width
            && let Some(index) = y
                .checked_mul(width)
                .and_then(|row| row.checked_add(column))
                .and_then(|index| usize::try_from(index).ok())
            && let Some(cell) = delta.get_mut(index)
        {
            *cell = cell.wrapping_add(winding);
        }
        x += slope;
    }
}

fn quick_reject_cubic(height: f32, a: Point, b: Point, c: Point, d: Point) -> bool {
    (a.y <= 0.0 && b.y <= 0.0 && c.y <= 0.0 && d.y <= 0.0)
        || (a.y >= height && b.y >= height && c.y >= height && d.y >= height)
}

fn compute_cubic_segments(a: Point, b: Point, c: Point, d: Point) -> i32 {
    let abc = Point {
        x: a.x - b.x - b.x + c.x,
        y: a.y - b.y - b.y + c.y,
    };
    let bcd = Point {
        x: b.x - c.x - c.x + d.x,
        y: b.y - c.y - c.y + d.y,
    };
    let dx = abc.x.abs().max(bcd.x.abs());
    let dy = abc.y.abs().max(bcd.y.abs());
    // Pinned runtime tests disable FP contraction for this owner. Keep the
    // multiply and add separately rounded: `mul_add` changes a finite witness
    // from 36 to 37 segments.
    let squared_distance = dx * dx + dy * dy;
    ((3.0 * squared_distance.sqrt()).sqrt().ceil() as i32).clamp(1, 1 << 8)
}

struct CubicCoefficient {
    a: Point,
    b: Point,
    c: Point,
    d: Point,
}

impl CubicCoefficient {
    fn new(a: Point, b: Point, c: Point, d: Point) -> Self {
        Self {
            a: Point {
                x: (d.x - a.x) + 3.0 * (b.x - c.x),
                y: (d.y - a.y) + 3.0 * (b.y - c.y),
            },
            b: Point {
                x: 3.0 * ((c.x - b.x) + (a.x - b.x)),
                y: 3.0 * ((c.y - b.y) + (a.y - b.y)),
            },
            c: Point {
                x: 3.0 * (b.x - a.x),
                y: 3.0 * (b.y - a.y),
            },
            d: a,
        }
    }

    fn evaluate(&self, t: f32) -> Point {
        Point {
            x: ((self.a.x * t + self.b.x) * t + self.c.x) * t + self.d.x,
            y: ((self.a.y * t + self.b.y) * t + self.c.y) * t + self.d.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Mat2D;

    #[test]
    fn pinned_add_rect_uses_authored_direction_and_transform() {
        for direction in [PathDirection::Clockwise, PathDirection::Counterclockwise] {
            let mut tester = HitTester::new(HitTestArea::new(10, 20, 14, 24));
            tester.add_rect(
                Aabb::new(0.0, 0.0, 4.0, 4.0),
                Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0]),
                direction,
            );
            assert!(tester.test(FillRule::NonZero), "direction {direction:?}");
        }
    }

    #[test]
    fn pinned_quad_only_advances_the_unoffset_endpoint() {
        let mut tester = HitTester::new(HitTestArea::new(10, 10, 12, 12));
        tester.move_to((10.0, 10.0));
        tester.quad_to((10.5, 11.0), (1.0, 1.0));
        assert_eq!((tester.previous.x, tester.previous.y), (1.0, 1.0));
    }

    #[test]
    fn pinned_mesh_point_and_area_overloads_cover_both_windings() {
        let vertices = [(0.0, 0.0), (4.0, 0.0), (0.0, 4.0)];
        assert!(HitTester::test_mesh_point(
            (1.0, 1.0),
            &vertices,
            &[0, 1, 2]
        ));
        assert!(HitTester::test_mesh_point(
            (1.0, 1.0),
            &vertices,
            &[2, 1, 0]
        ));
        assert!(!HitTester::test_mesh_point(
            (5.0, 5.0),
            &vertices,
            &[0, 1, 2]
        ));

        assert!(HitTester::test_mesh_area(
            HitTestArea::new(0, 0, 2, 2),
            &vertices,
            &[0, 1, 2]
        ));
        assert!(!HitTester::test_mesh_area(
            HitTestArea::new(5, 5, 7, 7),
            &vertices,
            &[0, 1, 2]
        ));
    }

    #[test]
    fn safe_mesh_owner_fails_closed_for_invalid_cpp_topology() {
        let vertices = [(0.0, 0.0), (4.0, 0.0), (0.0, 4.0)];
        assert!(!HitTester::test_mesh_point(
            (1.0, 1.0),
            &vertices,
            &[0, 1, 9]
        ));
        assert!(!HitTester::test_mesh_area(
            HitTestArea::new(0, 0, 2, 2),
            &vertices,
            &[0, 1, 9]
        ));
    }

    #[test]
    fn no_argument_reset_only_clears_windings() {
        let mut tester = HitTester::new(HitTestArea::new(0, 0, 2, 2));
        tester.move_to((0.0, 0.0));
        tester.line_to((2.0, 0.0));
        tester.clear_windings();
        assert!(tester.delta_windings.is_empty());
        assert!(!tester.expects_move);
        assert_eq!(tester.width_i32, 2);
        assert_eq!(tester.height_i32, 2);
    }

    #[test]
    fn cubic_segment_count_preserves_pinned_noncontracted_rounding() {
        let a = Point {
            x: f32::from_bits(0x43d7_ffe2),
            y: f32::from_bits(0x3f69_e89d),
        };
        let zero = Point::default();
        let squared_distance = a.x * a.x + a.y * a.y;
        assert_eq!(squared_distance.to_bits(), 0x4836_4002);
        assert_eq!(compute_cubic_segments(a, zero, zero, zero), 36);
    }
}
