use super::aabb::Aabb;
use super::bezier_utils::EvalCubic;
use super::mat2d::Mat2D;
use super::path_types::{PathDirection, PathVerb, path_verb_to_point_count};
use super::simd::{self, Float2, Float4};
use super::vec2d::Vec2D;

use crate::mechanical_port::source::command_path::CommandPath;

#[derive(Clone, Copy, Debug)]
pub struct PathSegment<'a> {
    pub verb: PathVerb,
    pub points: &'a [Vec2D],
}

/// Position of appended geometry without borrowing the growable path buffers.
#[derive(Clone, Copy, Debug)]
pub struct RawPathCursor {
    verb: usize,
    point: usize,
}

#[derive(Clone, Debug, Default)]
pub struct RawPath {
    points: Vec<Vec2D>,
    verbs: Vec<PathVerb>,
    last_move_index: usize,
    contour_is_open: bool,
}

impl PartialEq for RawPath {
    fn eq(&self, other: &Self) -> bool {
        self.points == other.points && self.verbs == other.verbs
    }
}

impl RawPath {
    pub fn empty(&self) -> bool {
        self.points.is_empty()
    }
    pub fn points(&self) -> &[Vec2D] {
        &self.points
    }
    pub fn points_mut(&mut self) -> &mut [Vec2D] {
        &mut self.points
    }
    pub fn verbs(&self) -> &[PathVerb] {
        &self.verbs
    }
    pub fn verbs_mut(&mut self) -> &mut [PathVerb] {
        &mut self.verbs
    }
    pub fn verbs_u8(&self) -> &[u8] {
        // SAFETY: PathVerb is `repr(u8)`, so each initialized enum occupies
        // exactly one byte with byte alignment. The returned slice shares the
        // source slice's lifetime and cannot mutate its discriminants.
        unsafe { core::slice::from_raw_parts(self.verbs.as_ptr().cast(), self.verbs.len()) }
    }

    pub fn bounds(&self) -> Aabb {
        let (mut mins, mut maxes, mut index) = if self.points.len() & 1 != 0 {
            let first = self.points[0];
            let first = Float2::from_array([first.x, first.y]).xyxy();
            (first, first, 1)
        } else if self.points.is_empty() {
            let zero = Float4::default();
            (zero, zero, 2)
        } else {
            let first = self.points[0];
            let second = self.points[1];
            let pair = Float4::from_array([first.x, first.y, second.x, second.y]);
            (pair, pair, 2)
        };

        while index < self.points.len() {
            let first = self.points[index];
            let second = self.points[index + 1];
            let points = Float4::from_array([first.x, first.y, second.x, second.y]);
            mins = simd::min(mins, points);
            maxes = simd::max(maxes, points);
            index += 2;
        }

        let mins = simd::min(mins.xy(), mins.zw());
        let maxes = simd::max(maxes.xy(), maxes.zw());
        Aabb::new(mins.x(), mins.y(), maxes.x(), maxes.y())
    }
    pub fn count_move_tos(&self) -> usize {
        self.verbs
            .iter()
            .filter(|verb| **verb == PathVerb::Move)
            .count()
    }
    fn inject_implicit_move_if_needed(&mut self) {
        if !self.contour_is_open {
            let point = if self.points.is_empty() {
                Vec2D::default()
            } else {
                self.points[self.last_move_index]
            };
            self.move_to_point(point);
        }
    }
    pub fn move_to_point(&mut self, point: Vec2D) {
        self.contour_is_open = true;
        self.last_move_index = self.points.len();
        self.points.push(point);
        self.verbs.push(PathVerb::Move);
    }
    pub fn line_to_point(&mut self, point: Vec2D) {
        self.inject_implicit_move_if_needed();
        self.points.push(point);
        self.verbs.push(PathVerb::Line);
    }
    pub fn quad_to_points(&mut self, control: Vec2D, end: Vec2D) {
        self.inject_implicit_move_if_needed();
        self.points.push(control);
        self.points.push(end);
        self.verbs.push(PathVerb::Quad);
    }
    pub fn cubic_to_points(&mut self, control1: Vec2D, control2: Vec2D, end: Vec2D) {
        self.inject_implicit_move_if_needed();
        self.points.extend([control1, control2, end]);
        self.verbs.push(PathVerb::Cubic);
    }
    pub fn close(&mut self) {
        if self.contour_is_open {
            self.verbs.push(PathVerb::Close);
            self.contour_is_open = false;
        }
    }
    pub fn is_closed(&self) -> bool {
        self.verbs.last() == Some(&PathVerb::Close)
    }
    pub fn swap(&mut self, other: &mut Self) {
        core::mem::swap(&mut self.points, &mut other.points);
        core::mem::swap(&mut self.verbs, &mut other.verbs);
    }
    pub fn reset(&mut self) {
        self.points.clear();
        self.points.shrink_to_fit();
        self.verbs.clear();
        self.verbs.shrink_to_fit();
        self.contour_is_open = false;
    }
    pub fn rewind(&mut self) {
        self.points.clear();
        self.verbs.clear();
        self.contour_is_open = false;
    }
    pub fn transform(&self, matrix: Mat2D) -> Self {
        let mut path = Self {
            verbs: self.verbs.clone(),
            points: vec![Vec2D::default(); self.points.len()],
            ..Self::default()
        };
        matrix.map_points(&mut path.points, &self.points);
        path
    }
    pub fn transform_in_place(&mut self, matrix: Mat2D) {
        let source = self.points.clone();
        matrix.map_points(&mut self.points, &source);
    }
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.move_to_point(Vec2D::new(x, y));
    }
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.line_to_point(Vec2D::new(x, y));
    }
    pub fn quad_to(&mut self, x: f32, y: f32, x1: f32, y1: f32) {
        self.quad_to_points(Vec2D::new(x, y), Vec2D::new(x1, y1));
    }
    pub fn cubic_to(&mut self, x: f32, y: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.cubic_to_points(Vec2D::new(x, y), Vec2D::new(x1, y1), Vec2D::new(x2, y2));
    }
    pub fn quad_to_cubic(&mut self, x: f32, y: f32, x1: f32, y1: f32) {
        assert!(!self.points.is_empty());
        if self.points.is_empty() {
            return;
        }
        let p0 = *self.points.last().unwrap();
        let p1 = Vec2D::new(x, y);
        let p2 = Vec2D::new(x1, y1);
        self.cubic_to_points(
            Vec2D::lerp(p0, p1, 2.0 / 3.0),
            Vec2D::lerp(p2, p1, 2.0 / 3.0),
            p2,
        );
    }
    pub fn add_rect(&mut self, rect: Aabb, direction: PathDirection) {
        self.points.reserve(5);
        self.verbs.reserve(6);
        self.move_to(rect.left(), rect.top());
        if direction == PathDirection::Clockwise {
            self.line_to(rect.right(), rect.top());
            self.line_to(rect.right(), rect.bottom());
            self.line_to(rect.left(), rect.bottom());
        } else {
            self.line_to(rect.left(), rect.bottom());
            self.line_to(rect.right(), rect.bottom());
            self.line_to(rect.right(), rect.top());
        }
        self.close();
    }
    pub fn add_oval(&mut self, rect: Aabb, direction: PathDirection) {
        const C: f32 = 0.551_915_05;
        const UNIT: [Vec2D; 13] = [
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, C),
            Vec2D::new(C, 1.0),
            Vec2D::new(0.0, 1.0),
            Vec2D::new(-C, 1.0),
            Vec2D::new(-1.0, C),
            Vec2D::new(-1.0, 0.0),
            Vec2D::new(-1.0, -C),
            Vec2D::new(-C, -1.0),
            Vec2D::new(0.0, -1.0),
            Vec2D::new(C, -1.0),
            Vec2D::new(1.0, -C),
            Vec2D::new(1.0, 0.0),
        ];
        let center = rect.center();
        let sx = rect.width() * 0.5;
        let sy = rect.height() * 0.5;
        let map = |p: Vec2D| Vec2D::new(p.x * sx + center.x, p.y * sy + center.y);
        self.points.reserve(13);
        self.verbs.reserve(6);
        if direction == PathDirection::Clockwise {
            self.move_to_point(map(UNIT[0]));
            for index in (1..=10).step_by(3) {
                self.cubic_to_points(map(UNIT[index]), map(UNIT[index + 1]), map(UNIT[index + 2]));
            }
        } else {
            self.move_to_point(map(UNIT[12]));
            for index in [11usize, 8, 5, 2] {
                self.cubic_to_points(map(UNIT[index]), map(UNIT[index - 1]), map(UNIT[index - 2]));
            }
        }
        self.close();
    }
    pub fn add_poly(&mut self, points: &[Vec2D], is_closed: bool) {
        let Some(first) = points.first().copied() else {
            return;
        };
        self.points.reserve(points.len() + usize::from(is_closed));
        self.verbs.reserve(points.len() + usize::from(is_closed));
        self.move_to_point(first);
        for point in &points[1..] {
            self.line_to_point(*point);
        }
        if is_closed {
            self.close();
        }
    }

    pub fn segments(&self) -> Vec<PathSegment<'_>> {
        let mut result = Vec::with_capacity(self.verbs.len());
        let mut point_index: usize = 0;
        for verb in &self.verbs {
            let count = path_verb_to_point_count(*verb);
            let start = if *verb == PathVerb::Move {
                point_index
            } else {
                point_index.saturating_sub(1)
            };
            let end = point_index + count;
            result.push(PathSegment {
                verb: *verb,
                points: &self.points[start..end],
            });
            point_index += count;
        }
        result
    }
    pub fn morph(&self, mut procedure: impl FnMut(Vec2D) -> Vec2D) -> Self {
        let mut dst = Self::default();
        for segment in self.segments() {
            match segment.verb {
                PathVerb::Move => dst.move_to_point(procedure(segment.points[0])),
                PathVerb::Line => dst.line_to_point(procedure(segment.points[1])),
                PathVerb::Quad => {
                    dst.quad_to_points(procedure(segment.points[1]), procedure(segment.points[2]))
                }
                PathVerb::Cubic => dst.cubic_to_points(
                    procedure(segment.points[1]),
                    procedure(segment.points[2]),
                    procedure(segment.points[3]),
                ),
                PathVerb::Close => dst.close(),
            }
        }
        dst
    }
    pub fn add_path(&mut self, source: &Self, matrix: Option<&Mat2D>) -> RawPathCursor {
        let initial_verb_count = self.verbs.len();
        let initial_point_count = self.points.len();
        self.verbs.extend_from_slice(&source.verbs);
        if let Some(matrix) = matrix {
            let start = self.points.len();
            self.points
                .resize(start + source.points.len(), Vec2D::default());
            matrix.map_points(&mut self.points[start..], &source.points);
        } else {
            self.points.extend_from_slice(&source.points);
        }
        RawPathCursor {
            verb: initial_verb_count,
            point: initial_point_count,
        }
    }
    pub fn add_path_backwards(&mut self, source: &Self, matrix: Option<&Mat2D>) -> RawPathCursor {
        if source.empty() {
            return RawPathCursor {
                verb: self.verbs.len(),
                point: self.points.len(),
            };
        }
        let initial_point_count = self.points.len();
        self.points.extend(source.points.iter().rev().copied());
        let initial_verb_count = self.verbs.len();
        assert_eq!(source.verbs.first(), Some(&PathVerb::Move));
        self.verbs.push(PathVerb::Move);
        let mut closed = false;
        for (reverse_index, verb) in source.verbs.iter().rev().copied().enumerate() {
            if verb == PathVerb::Close {
                assert!(!closed);
                closed = true;
                continue;
            }
            if verb == PathVerb::Move && closed {
                self.verbs.push(PathVerb::Close);
                closed = false;
            }
            if reverse_index + 1 != source.verbs.len() {
                self.verbs.push(verb);
            } else {
                assert_eq!(verb, PathVerb::Move);
            }
        }
        assert!(!closed);
        assert_eq!(self.verbs.len(), initial_verb_count + source.verbs.len());
        assert_eq!(self.points.len(), initial_point_count + source.points.len());
        if let Some(matrix) = matrix {
            let source_points = self.points[initial_point_count..].to_vec();
            matrix.map_points(&mut self.points[initial_point_count..], &source_points);
            self.prune_empty_segments_from(RawPathCursor {
                verb: initial_verb_count,
                point: initial_point_count,
            });
        }
        RawPathCursor {
            verb: initial_verb_count,
            point: initial_point_count,
        }
    }
    pub fn prune_empty_segments(&mut self) {
        self.prune_empty_segments_from(RawPathCursor { verb: 0, point: 0 });
    }
    pub fn prune_empty_segments_from(&mut self, start: RawPathCursor) {
        let RawPathCursor {
            verb: start_verb,
            point: start_point,
        } = start;
        let mut kept_verbs = self.verbs[..start_verb].to_vec();
        let mut kept_points = self.points[..start_point].to_vec();
        let mut point_index = start_point;
        for verb in self.verbs[start_verb..].iter().copied() {
            let advance = path_verb_to_point_count(verb);
            let keep = match verb {
                PathVerb::Move | PathVerb::Close => true,
                PathVerb::Line => self.points[point_index] != self.points[point_index - 1],
                PathVerb::Quad => {
                    self.points[point_index + 1] != self.points[point_index]
                        || self.points[point_index] != self.points[point_index - 1]
                }
                PathVerb::Cubic => {
                    self.points[point_index + 2] != self.points[point_index + 1]
                        || self.points[point_index + 1] != self.points[point_index]
                        || self.points[point_index] != self.points[point_index - 1]
                }
            };
            if keep {
                kept_verbs.push(verb);
                kept_points.extend_from_slice(&self.points[point_index..point_index + advance]);
            }
            point_index += advance;
        }
        self.verbs = kept_verbs;
        self.points = kept_points;
    }
    pub fn add_to(&self, result: &mut dyn CommandPath) {
        for segment in self.segments() {
            match segment.verb {
                PathVerb::Move => result.move_(segment.points[0]),
                PathVerb::Line => result.line(segment.points[1]),
                PathVerb::Cubic => {
                    result.cubic(segment.points[1], segment.points[2], segment.points[3])
                }
                PathVerb::Close => result.close(),
                PathVerb::Quad => result.cubic(
                    Vec2D::lerp(segment.points[0], segment.points[1], 2.0 / 3.0),
                    Vec2D::lerp(segment.points[2], segment.points[1], 2.0 / 3.0),
                    segment.points[2],
                ),
            }
        }
    }
    #[cfg(debug_assertions)]
    pub fn print_code(&self) {
        eprintln!("RawPath path;");
        for segment in self.segments() {
            match segment.verb {
                PathVerb::Move => eprintln!(
                    "path.moveTo({:.6}, {:.6});",
                    segment.points[0].x, segment.points[0].y
                ),
                PathVerb::Line => eprintln!(
                    "path.lineTo({:.6}, {:.6});",
                    segment.points[1].x, segment.points[1].y
                ),
                PathVerb::Cubic => eprintln!(
                    "path.cubicTo({:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6});",
                    segment.points[1].x,
                    segment.points[1].y,
                    segment.points[2].x,
                    segment.points[2].y,
                    segment.points[3].x,
                    segment.points[3].y
                ),
                PathVerb::Close => eprintln!("path.close();"),
                PathVerb::Quad => eprintln!(
                    "path.quadTo({:.6}, {:.6}, {:.6}, {:.6});",
                    segment.points[1].x,
                    segment.points[1].y,
                    segment.points[2].x,
                    segment.points[2].y
                ),
            }
        }
        eprintln!();
    }
    pub fn reserve(&mut self, verbs: usize, points: usize) {
        self.verbs.reserve(verbs);
        self.points.reserve(points);
    }
    pub fn precise_bounds(&self) -> Aabb {
        let mut bounds = Aabb::for_expansion();
        for segment in self.segments() {
            match segment.verb {
                PathVerb::Move => Aabb::expand_to_point(&mut bounds, segment.points[0]),
                PathVerb::Line => Aabb::expand_to_point(&mut bounds, segment.points[1]),
                PathVerb::Cubic => {
                    expand_cubic_bounds_for_axis(
                        &mut bounds,
                        0,
                        segment.points[0].x,
                        segment.points[1].x,
                        segment.points[2].x,
                        segment.points[3].x,
                    );
                    expand_cubic_bounds_for_axis(
                        &mut bounds,
                        1,
                        segment.points[0].y,
                        segment.points[1].y,
                        segment.points[2].y,
                        segment.points[3].y,
                    );
                }
                PathVerb::Quad => {
                    let p1 = Vec2D::lerp(segment.points[0], segment.points[1], 2.0 / 3.0);
                    let p2 = Vec2D::lerp(segment.points[2], segment.points[1], 2.0 / 3.0);
                    expand_cubic_bounds_for_axis(
                        &mut bounds,
                        0,
                        segment.points[0].x,
                        p1.x,
                        p2.x,
                        segment.points[2].x,
                    );
                    expand_cubic_bounds_for_axis(
                        &mut bounds,
                        1,
                        segment.points[0].y,
                        p1.y,
                        p2.y,
                        segment.points[2].y,
                    );
                }
                PathVerb::Close => {}
            }
        }
        bounds
    }
    pub fn compute_coarse_area(&self) -> f32 {
        let mut area = 0.0;
        let mut contour_start = Vec2D::default();
        let mut last = Vec2D::default();
        for segment in self.segments() {
            match segment.verb {
                PathVerb::Move => {
                    area += Vec2D::cross(last, contour_start);
                    contour_start = segment.points[0];
                    last = segment.points[0];
                }
                PathVerb::Close => {}
                PathVerb::Line => {
                    area += Vec2D::cross(last, segment.points[1]);
                    last = segment.points[1];
                }
                PathVerb::Quad => unreachable!(),
                PathVerb::Cubic => {
                    let points: &[Vec2D; 4] = segment.points.try_into().unwrap();
                    let mut count = cubic_wangs_formula(points, 1.0 / 8.0).ceil();
                    if count > 1.0 {
                        count = if 64.0 < count { 64.0 } else { count };
                        let eval = EvalCubic::new(points);
                        let mut low_t = 1.0 / count;
                        let mut high_t = 2.0 / count;
                        let delta_t = high_t;
                        while low_t < 1.0 {
                            let point = eval.at(low_t);
                            area += Vec2D::cross(last, point);
                            last = point;
                            if high_t < 1.0 {
                                let point = eval.at(high_t);
                                area += Vec2D::cross(last, point);
                                last = point;
                            }
                            low_t += delta_t;
                            high_t += delta_t;
                        }
                    }
                    area += Vec2D::cross(last, points[3]);
                    last = points[3];
                }
            }
        }
        area += Vec2D::cross(last, contour_start);
        area * 0.5
    }
}

fn expand_axis_bounds(bounds: &mut Aabb, axis: usize, value: f32) {
    match axis {
        0 => {
            if value < bounds.min_x {
                bounds.min_x = value;
            }
            if value > bounds.max_x {
                bounds.max_x = value;
            }
        }
        1 => {
            if value < bounds.min_y {
                bounds.min_y = value;
            }
            if value > bounds.max_y {
                bounds.max_y = value;
            }
        }
        _ => unreachable!(),
    }
}
fn expand_bounds_to_cubic_point(
    bounds: &mut Aabb,
    axis: usize,
    t: f32,
    a: f32,
    b: f32,
    c: f32,
    d: f32,
) {
    if (0.0..=1.0).contains(&t) {
        let inverse = 1.0 - t;
        let point = inverse * inverse * inverse * a
            + 3.0 * inverse * inverse * t * b
            + 3.0 * inverse * t * t * c
            + t * t * t * d;
        expand_axis_bounds(bounds, axis, point);
    }
}
fn expand_cubic_bounds_for_axis(
    bounds: &mut Aabb,
    axis: usize,
    start: f32,
    cp1: f32,
    cp2: f32,
    end: f32,
) {
    expand_axis_bounds(bounds, axis, start);
    expand_axis_bounds(bounds, axis, end);
    let a = 3.0 * (cp1 - start);
    let b = 3.0 * (cp2 - cp1);
    let c = 3.0 * (end - cp2);
    let d = a - 2.0 * b + c;
    if d != 0.0 {
        let m1 = -(b * b - a * c).sqrt();
        let m2 = -a + b;
        expand_bounds_to_cubic_point(bounds, axis, -(m1 + m2) / d, start, cp1, cp2, end);
        expand_bounds_to_cubic_point(bounds, axis, -(-m1 + m2) / d, start, cp1, cp2, end);
    } else if b != c {
        expand_bounds_to_cubic_point(
            bounds,
            axis,
            (2.0 * b - c) / (2.0 * (b - c)),
            start,
            cp1,
            cp2,
            end,
        );
    }
    let d2a = 2.0 * (b - a);
    let d2b = 2.0 * (c - b);
    if d2a != b {
        expand_bounds_to_cubic_point(bounds, axis, d2a / (d2a - d2b), start, cp1, cp2, end);
    }
}
fn cubic_wangs_formula(points: &[Vec2D; 4], precision: f32) -> f32 {
    let v0 = points[0] - 2.0 * points[1] + points[2];
    let v1 = points[1] - 2.0 * points[2] + points[3];
    let first = v0.length_squared();
    let second = v1.length_squared();
    let maximum = if first < second { second } else { first };
    (maximum * (9.0 / 16.0) * precision * precision)
        .sqrt()
        .sqrt()
}
