use crate::mechanical_port::source::{
    math::{
        aabb::Aabb,
        raw_path::RawPath,
        rectangles_to_contour::{Contour, RectanglesToContour},
        vec2d::Vec2D,
    },
    shapes::{paint::shape_paint_path::ShapePaintPath, path::Path},
};
#[derive(Default)]
pub struct TextSelectionPath {
    pub path: ShapePaintPath,
    rectangles: RectanglesToContour,
}
impl TextSelectionPath {
    pub fn new(is_local: bool, fill_rule: nuxie_render_api::FillRule) -> Self {
        Self {
            path: ShapePaintPath::with_fill_rule(is_local, fill_rule),
            rectangles: RectanglesToContour::default(),
        }
    }
    pub fn update(&mut self, rects: &[Aabb], radius: f32) {
        self.path.rewind();
        self.rectangles.reset();
        for rect in rects {
            self.rectangles.add_rect(*rect);
        }
        self.rectangles.compute_contours();
        let count = self.rectangles.contour_count();
        for i in 0..count {
            let contour = self.rectangles.contour(i);
            if contour.size() < 2 {
                continue;
            }
            let probe = contour.point(0);
            let mut depth = 0;
            for j in 0..count {
                if j != i && Self::contour_contains(&self.rectangles.contour(j), probe) {
                    depth += 1;
                }
            }
            Self::add_rounded_path(
                &contour,
                radius,
                self.path.mutable_raw_path(),
                depth & 1 == 0,
            );
        }
    }
    // Crossing-number test. RectanglesToContour contours never touch, so a
    // vertex of another contour is a safe containment probe.
    fn contour_contains(contour: &Contour, point: Vec2D) -> bool {
        let size = contour.size();
        if size < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = size - 1;
        for i in 0..size {
            let a = contour.point(i);
            let b = contour.point(j);
            if (a.y > point.y) != (b.y > point.y)
                && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    }
    fn add_rounded_path(contour: &Contour, radius: f32, raw: &mut RawPath, clockwise: bool) {
        let reversed = contour.is_clockwise() != clockwise;
        let len = contour.size();
        if len < 2 {
            return;
        }
        for i in 0..len {
            let pos = contour.point_reversed(i, reversed);
            if radius > 0.0 {
                let prev = contour.point_reversed((i + len - 1) % len, reversed);
                let next = contour.point_reversed((i + 1) % len, reversed);
                let mut to_prev = prev - pos;
                let lp = to_prev.length();
                to_prev /= lp;
                let mut to_next = next - pos;
                let ln = to_next.length();
                to_next /= ln;
                let rr = (lp / 2.0).min((ln / 2.0).min(radius));
                let d = Path::compute_ideal_control_point_distance(to_prev, to_next, rr);
                let begin = Vec2D::scale_and_add(pos, to_prev, rr);
                if i == 0 {
                    raw.move_to(begin.x, begin.y);
                } else {
                    raw.line_to(begin.x, begin.y);
                }
                let out = Vec2D::scale_and_add(pos, to_prev, rr - d);
                let inside = Vec2D::scale_and_add(pos, to_next, rr - d);
                let end = Vec2D::scale_and_add(pos, to_next, rr);
                raw.cubic_to(out.x, out.y, inside.x, inside.y, end.x, end.y);
            } else if i == 0 {
                raw.move_to(pos.x, pos.y);
            } else {
                raw.line_to(pos.x, pos.y);
            }
        }
        raw.close();
    }
}
