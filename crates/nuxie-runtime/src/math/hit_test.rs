use nuxie_render_api::FillRule;

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
    ((3.0 * dx.mul_add(dx, dy * dy).sqrt()).sqrt().ceil() as i32).clamp(1, 1 << 8)
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
