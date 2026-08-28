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
    pub fn update(&mut self, rects: &[Aabb], radius: f32) {
        self.path.rewind();
        self.rectangles.reset();
        for rect in rects {
            self.rectangles.add_rect(*rect);
        }
        self.rectangles.compute_contours();
        let contours: Vec<_> = self.rectangles.iter().cloned().collect();
        for contour in &contours {
            Self::add_rounded_path(contour, radius, self.path.mutable_raw_path());
        }
    }
    fn add_rounded_path(contour: &Contour, radius: f32, raw: &mut RawPath) {
        let clockwise = contour.is_clockwise();
        let len = contour.len();
        if len < 2 {
            return;
        }
        for i in 0..len {
            let pos = contour.point(i, !clockwise);
            if radius > 0.0 {
                let prev = contour.point((i + len - 1) % len, !clockwise);
                let next = contour.point((i + 1) % len, !clockwise);
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
