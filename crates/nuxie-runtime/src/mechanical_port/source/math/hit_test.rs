use super::aabb::{Aabb, IAabb};
use super::mat2d::Mat2D;
use super::path_types::{FillRule, PathDirection};
use super::vec2d::Vec2D;

const MAX_CURVE_SEGMENTS: i32 = 1 << 8;
const MAX_LOCAL_SEGMENTS: i32 = 16;

#[derive(Clone, Debug, Default)]
pub struct HitTester {
    delta_windings: Vec<i32>,
    first: Vec2D,
    previous: Vec2D,
    offset: Vec2D,
    height: f32,
    integer_width: i32,
    integer_height: i32,
    expects_move: bool,
}

impl HitTester {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_area(area: IAabb) -> Self {
        let mut result = Self::new();
        result.reset_area(area);
        result
    }
    pub fn reset(&mut self) {
        self.delta_windings.clear();
    }
    pub fn reset_area(&mut self, clip: IAabb) {
        self.offset = Vec2D::new(clip.left as f32, clip.top as f32);
        self.height = clip.height() as f32;
        self.integer_width = clip.width();
        self.integer_height = clip.height();
        self.delta_windings
            .resize((self.integer_width * self.integer_height) as usize, 0);
        self.delta_windings.fill(0);
        self.expects_move = true;
    }
    pub fn move_to(&mut self, point: Vec2D) {
        if !self.expects_move {
            self.close();
        }
        self.first = point - self.offset;
        self.previous = self.first;
        self.expects_move = false;
    }
    pub fn line_to(&mut self, point: Vec2D) {
        assert!(!self.expects_move);
        let point = point - self.offset;
        clip_line(
            self.height,
            self.previous,
            point,
            &mut self.delta_windings,
            self.integer_width,
        );
        self.previous = point;
    }
    pub fn quad_to(&mut self, _control: Vec2D, end: Vec2D) {
        assert!(!self.expects_move);
        self.previous = end;
    }
    pub fn cubic_to(&mut self, mut b: Vec2D, mut c: Vec2D, mut d: Vec2D) {
        assert!(!self.expects_move);
        b -= self.offset;
        c -= self.offset;
        d -= self.offset;
        if quick_reject_cubic(self.height, self.previous, b, c, d) {
            self.previous = d;
            return;
        }
        let count = compute_cubic_segments(self.previous, b, c, d);
        self.recurse_cubic(b, c, d, count);
    }
    fn recurse_cubic(&mut self, b: Vec2D, c: Vec2D, d: Vec2D, count: i32) {
        if quick_reject_cubic(self.height, self.previous, b, c, d) {
            self.previous = d;
            return;
        }
        if count > MAX_LOCAL_SEGMENTS {
            let chop = CubicChop::new(self.previous, b, c, d);
            let new_count = (count + 1) >> 1;
            assert!(new_count < count);
            self.recurse_cubic(chop[1], chop[2], chop[3], new_count);
            self.recurse_cubic(chop[4], chop[5], chop[6], new_count);
        } else {
            let delta = 1.0 / count as f32;
            let mut t = delta;
            let cubic = CubicCoeff::new(self.previous, b, c, d);
            let mut previous = self.previous;
            for _ in 1..count - 1 {
                let next = cubic.eval(t);
                clip_line(
                    self.height,
                    previous,
                    next,
                    &mut self.delta_windings,
                    self.integer_width,
                );
                previous = next;
                t += delta;
            }
            clip_line(
                self.height,
                previous,
                d,
                &mut self.delta_windings,
                self.integer_width,
            );
            self.previous = d;
        }
    }
    pub fn close(&mut self) {
        assert!(!self.expects_move);
        clip_line(
            self.height,
            self.previous,
            self.first,
            &mut self.delta_windings,
            self.integer_width,
        );
        self.expects_move = true;
    }
    pub fn add_rect(&mut self, rect: Aabb, transform: Mat2D, direction: PathDirection) {
        let points = [
            transform * Vec2D::new(rect.left(), rect.top()),
            transform * Vec2D::new(rect.right(), rect.top()),
            transform * Vec2D::new(rect.right(), rect.bottom()),
            transform * Vec2D::new(rect.left(), rect.bottom()),
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
    pub fn test(&mut self, rule: FillRule) -> bool {
        if !self.expects_move {
            self.close();
        }
        let mask = if rule == FillRule::NonZero { -1 } else { 1 };
        let mut nonzero = 0;
        for winding in &self.delta_windings {
            nonzero |= winding & mask;
        }
        nonzero != 0
    }
    pub fn test_mesh_point(point: Vec2D, vertices: &[Vec2D], indices: &[u16]) -> bool {
        if vertices.len() < 3 {
            return false;
        }
        let bounds = Aabb::from_points(vertices);
        if bounds.bottom() < point.y
            || point.y < bounds.top()
            || bounds.right() < point.x
            || point.x < bounds.left()
        {
            return false;
        }
        for triangle in indices.chunks_exact(3) {
            let a = vertices[triangle[0] as usize] - point;
            let b = vertices[triangle[1] as usize] - point;
            let c = vertices[triangle[2] as usize] - point;
            let ab = cross_less(a, b);
            let bc = cross_less(b, c);
            let ca = cross_less(c, a);
            if ab == bc && ab == ca {
                return true;
            }
        }
        false
    }
    pub fn test_mesh_area(area: IAabb, vertices: &[Vec2D], indices: &[u16]) -> bool {
        if area.width() * area.height() == 1 {
            return Self::test_mesh_point(
                Vec2D::new(area.left as f32, area.top as f32),
                vertices,
                indices,
            );
        }
        if vertices.len() < 3 {
            return false;
        }
        let bounds = Aabb::from_points(vertices);
        if bounds.bottom() <= area.top as f32
            || area.bottom as f32 <= bounds.top()
            || bounds.right() <= area.left as f32
            || area.right as f32 <= bounds.left()
        {
            return false;
        }
        let mut windings = vec![0; (area.width() * area.height()) as usize];
        let offset = Vec2D::new(area.left as f32, area.top as f32);
        for triangle in indices.chunks_exact(3) {
            let a = vertices[triangle[0] as usize] - offset;
            let b = vertices[triangle[1] as usize] - offset;
            let c = vertices[triangle[2] as usize] - offset;
            clip_line(area.height() as f32, a, b, &mut windings, area.width());
            clip_line(area.height() as f32, b, c, &mut windings, area.width());
            clip_line(area.height() as f32, c, a, &mut windings, area.width());
            let mut nonzero = 0;
            for winding in &windings {
                nonzero |= winding;
            }
            if nonzero != 0 {
                return true;
            }
        }
        false
    }
}

fn graphics_round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}
fn append_line(
    height: f32,
    p0: Vec2D,
    p1: Vec2D,
    slope: f32,
    winding: i32,
    delta: &mut [i32],
    width: i32,
) {
    assert!(winding == 1 || winding == -1);
    let top = graphics_round(p0.y);
    let bottom = graphics_round(p1.y);
    if top == bottom {
        return;
    }
    assert!(top < bottom && top >= 0 && bottom as f32 <= height);
    let mut x = p0.x + slope * (top as f32 - p0.y + 0.5) + 0.5;
    let mut row = (top * width) as usize;
    for _ in top..bottom {
        let ix = x.max(0.0) as i32;
        if ix < width {
            delta[row + ix as usize] += winding;
        }
        x += slope;
        row += width as usize;
    }
}
fn clip_line(height: f32, mut p0: Vec2D, mut p1: Vec2D, delta: &mut [i32], width: i32) {
    if p0.y == p1.y {
        return;
    }
    let mut winding = 1;
    if p0.y > p1.y {
        winding = -1;
        core::mem::swap(&mut p0, &mut p1);
    }
    if p1.y <= 0.0 || p0.y >= height {
        return;
    }
    let slope = (p1.x - p0.x) / (p1.y - p0.y);
    if p0.y < 0.0 {
        p0.x += slope * -p0.y;
        p0.y = 0.0;
    }
    if p1.y > height {
        p1.x += slope * (height - p1.y);
        p1.y = height;
    }
    append_line(height, p0, p1, slope, winding, delta, width);
}
fn compute_cubic_segments(a: Vec2D, b: Vec2D, c: Vec2D, d: Vec2D) -> i32 {
    let abc = a - b - b + c;
    let bcd = b - c - c + d;
    let dx = abc.x.abs().max(bcd.x.abs());
    let dy = abc.y.abs().max(bcd.y.abs());
    let distance = (dx * dx + dy * dy).sqrt();
    ((3.0 * distance).sqrt().ceil() as i32).clamp(1, MAX_CURVE_SEGMENTS)
}
#[derive(Clone, Copy)]
struct CubicCoeff {
    a: Vec2D,
    b: Vec2D,
    c: Vec2D,
    d: Vec2D,
}
impl CubicCoeff {
    fn new(a: Vec2D, b: Vec2D, c: Vec2D, d: Vec2D) -> Self {
        Self {
            a: (d - a) + 3.0 * (b - c),
            b: 3.0 * ((c - b) + (a - b)),
            c: 3.0 * (b - a),
            d: a,
        }
    }
    fn eval(self, t: f32) -> Vec2D {
        ((self.a * t + self.b) * t + self.c) * t + self.d
    }
}
fn quick_reject_cubic(height: f32, a: Vec2D, b: Vec2D, c: Vec2D, d: Vec2D) -> bool {
    (a.y <= 0.0 && b.y <= 0.0 && c.y <= 0.0 && d.y <= 0.0)
        || (a.y >= height && b.y >= height && c.y >= height && d.y >= height)
}
#[derive(Clone, Copy)]
struct CubicChop([Vec2D; 7]);
impl CubicChop {
    fn new(a: Vec2D, b: Vec2D, c: Vec2D, d: Vec2D) -> Self {
        let ab = average(a, b);
        let bc = average(b, c);
        let cd = average(c, d);
        let abc = average(ab, bc);
        let bcd = average(bc, cd);
        Self([a, ab, abc, average(abc, bcd), bcd, cd, d])
    }
}

fn average(a: Vec2D, b: Vec2D) -> Vec2D {
    // Pinned `ave` instantiates generic `lerp(a, b, .5)`, whose expression is
    // `a * (1 - t) + b * t`, not `Vec2D::lerp`'s `a + (b - a) * t`.
    Vec2D::new(a.x * 0.5 + b.x * 0.5, a.y * 0.5 + b.y * 0.5)
}
impl core::ops::Index<usize> for CubicChop {
    type Output = Vec2D;
    fn index(&self, index: usize) -> &Vec2D {
        &self.0[index]
    }
}
fn cross_less(a: Vec2D, b: Vec2D) -> bool {
    a.x * b.y < a.y * b.x
}
