use std::rc::Rc;

use super::math_types;
use super::path_types::PathVerb;
use super::raw_path::RawPath;
use super::raw_path_utils::{EvalCubic, EvalQuad, cubic_extract, line_extract, quad_extract};
use super::vec2d::Vec2D;

const MAX_DOT30: u32 = (1 << 30) - 1;
const INV_SCALE_D30: f32 = 1.0 / MAX_DOT30 as f32;
const EPSILON: f32 = 1.0 / 4096.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
enum SegmentType {
    Line,
    Quad,
    Cubic,
}

#[derive(Clone, Copy, Debug)]
pub struct Segment {
    distance: f32,
    point_index: u32,
    t_value: u32,
    segment_type: SegmentType,
}
impl Segment {
    pub fn get_t(self) -> f32 {
        self.t_value as f32 * INV_SCALE_D30
    }
    fn extract_all(self, dst: &mut RawPath, points: &[Vec2D]) {
        let points = &points[self.point_index as usize..];
        match self.segment_type {
            SegmentType::Line => dst.line_to_point(points[1]),
            SegmentType::Quad => dst.quad_to_points(points[1], points[2]),
            SegmentType::Cubic => dst.cubic_to_points(points[1], points[2], points[3]),
        }
    }
    fn extract(self, dst: &mut RawPath, from_t: f32, to_t: f32, points: &[Vec2D], move_to: bool) {
        assert!(from_t <= to_t);
        let points = &points[self.point_index as usize..];
        match self.segment_type {
            SegmentType::Line => {
                let source: &[Vec2D; 2] = points[..2].try_into().unwrap();
                let mut extracted = [Vec2D::default(); 2];
                line_extract(source, from_t, to_t, &mut extracted);
                if move_to {
                    dst.move_to_point(extracted[0]);
                }
                dst.line_to_point(extracted[1]);
            }
            SegmentType::Quad => {
                let source: &[Vec2D; 3] = points[..3].try_into().unwrap();
                let mut extracted = [Vec2D::default(); 3];
                quad_extract(source, from_t, to_t, &mut extracted);
                if move_to {
                    dst.move_to_point(extracted[0]);
                }
                dst.quad_to_points(extracted[1], extracted[2]);
            }
            SegmentType::Cubic => {
                let source: &[Vec2D; 4] = points[..4].try_into().unwrap();
                let mut extracted = [Vec2D::default(); 4];
                cubic_extract(source, from_t, to_t, &mut extracted);
                if move_to {
                    dst.move_to_point(extracted[0]);
                }
                dst.cubic_to_points(extracted[1], extracted[2], extracted[3]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_segment_extract_matches_pinned_generic_lerp_grouping() {
        let segment = Segment {
            distance: 1.0,
            point_index: 0,
            t_value: MAX_DOT30,
            segment_type: SegmentType::Line,
        };
        let points = [
            Vec2D::new(39.608627, -64.03908),
            Vec2D::new(12.428378, -185.07193),
        ];
        let mut path = RawPath::default();

        segment.extract(&mut path, 0.37239173, 0.97299224, &points, true);

        assert_eq!(
            path.points()
                .iter()
                .map(|point| point.x.to_bits())
                .collect::<Vec<_>>(),
            [0x41eb_e53b, 0x4152_996b]
        );
        assert_eq!(
            path.points()
                .iter()
                .map(|point| point.y.to_bits())
                .collect::<Vec<_>>(),
            [0xc2da_38b0, 0xc335_cd98]
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PosTan {
    pub pos: Vec2D,
    pub tan: Vec2D,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PosTanDistance {
    pub pos: Vec2D,
    pub tan: Vec2D,
    pub distance: f32,
    pub squared_distance_to_point: f32,
}
impl PosTanDistance {
    pub fn new(value: PosTan, distance: f32) -> Self {
        Self {
            pos: value.pos,
            tan: value.tan,
            distance,
            squared_distance_to_point: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ContourMeasure {
    segments: Vec<Segment>,
    points: Vec<Vec2D>,
    length: f32,
    is_closed: bool,
}
impl ContourMeasure {
    fn new(segments: Vec<Segment>, points: Vec<Vec2D>, length: f32, is_closed: bool) -> Self {
        Self {
            segments,
            points,
            length,
            is_closed,
        }
    }
    pub fn length(&self) -> f32 {
        self.length
    }
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }
    fn find_segment(&self, distance: f32) -> usize {
        assert!(
            self.segments[0].distance >= 0.0
                && self.segments.last().unwrap().distance == self.length
        );
        assert!(distance >= 0.0 && distance <= self.length);
        let mut index = self
            .segments
            .partition_point(|segment| segment.distance < distance);
        while index < self.segments.len() && self.segments[index].distance == 0.0 {
            index += 1;
        }
        assert!(index < self.segments.len());
        index
    }
    pub fn get_pos_tan(&self, mut distance: f32) -> PosTan {
        if distance > self.length {
            distance = self.length;
        }
        if distance < 0.0 {
            distance = 0.0;
        }
        let index = self.find_segment(distance);
        let segment = self.segments[index];
        let current_distance = segment.distance;
        let previous_distance = if index > 0 {
            self.segments[index - 1].distance
        } else {
            0.0
        };
        assert!(
            previous_distance < current_distance
                && distance <= current_distance
                && distance >= previous_distance
        );
        let relative_distance =
            (distance - previous_distance) / (current_distance - previous_distance);
        assert!((0.0..=1.0).contains(&relative_distance));
        let point_index = segment.point_index as usize;
        if segment.segment_type == SegmentType::Line {
            let p0 = self.points[point_index];
            let p1 = self.points[point_index + 1];
            return PosTan {
                pos: Vec2D::lerp(p0, p1, relative_distance),
                tan: (p1 - p0).normalized(),
            };
        }
        let previous_t = if index > 0 && self.segments[index - 1].point_index == segment.point_index
        {
            self.segments[index - 1].get_t()
        } else {
            0.0
        };
        let t = previous_t * (1.0 - relative_distance) + segment.get_t() * relative_distance;
        assert!((0.0..=1.0).contains(&t));
        if segment.segment_type == SegmentType::Quad {
            eval_quad(
                (&self.points[point_index..point_index + 3])
                    .try_into()
                    .unwrap(),
                t,
            )
        } else {
            eval_cubic(
                (&self.points[point_index..point_index + 4])
                    .try_into()
                    .unwrap(),
                t,
            )
        }
    }
    pub fn get_segment(
        &self,
        mut start_distance: f32,
        mut end_distance: f32,
        dst: &mut RawPath,
        start_with_move: bool,
    ) {
        start_distance = cpp_max(0.0, start_distance);
        end_distance = cpp_min(self.length, end_distance);
        if start_distance >= end_distance {
            return;
        }
        let mut start_index = self.find_segment(start_distance);
        let end_index = self.find_segment(end_distance);
        let mut start = self.segments[start_index];
        let end = self.segments[end_index];
        let mut start_t = self.compute_t(start_index, start_distance);
        let end_t = self.compute_t(end_index, end_distance);
        if 1.0 - start_t < EPSILON && start_index < end_index {
            start_index += 1;
            start = self.segments[start_index];
            start_t = 0.0;
        }
        if start.point_index == end.point_index {
            start.extract(dst, start_t, end_t, &self.points, start_with_move);
        } else {
            start.extract(dst, start_t, 1.0, &self.points, start_with_move);
            let mut index = next_segment_beginning(&self.segments, start_index);
            while self.segments[index].point_index != end.point_index {
                self.segments[index].extract_all(dst, &self.points);
                index = next_segment_beginning(&self.segments, index);
            }
            end.extract(dst, 0.0, end_t, &self.points, false);
        }
    }
    fn compute_t(&self, index: usize, distance: f32) -> f32 {
        let segment = self.segments[index];
        assert!(distance <= segment.distance);
        let (previous_distance, previous_t) = if index > 0 {
            let previous = self.segments[index - 1];
            (
                previous.distance,
                if previous.point_index == segment.point_index {
                    previous.get_t()
                } else {
                    0.0
                },
            )
        } else {
            (0.0, 0.0)
        };
        let ratio = (distance - previous_distance) / (segment.distance - previous_distance);
        let t = previous_t * (1.0 - ratio) + segment.get_t() * ratio;
        math_types::clamp(t, previous_t, segment.get_t())
    }
    pub fn warp(&self, source: Vec2D) -> Vec2D {
        let result = self.get_pos_tan(source.x);
        Vec2D::new(
            result.pos.x - result.tan.y * source.y,
            result.pos.y + result.tan.x * source.y,
        )
    }
    pub fn dump(&self) {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        unsafe {
            libc::printf(
                b"length %g pts %zu segs %zu\n\0".as_ptr().cast(),
                self.length as f64,
                self.points.len(),
                self.segments.len(),
            );
            for segment in &self.segments {
                libc::printf(
                    b" %g %d %g %d\n\0".as_ptr().cast(),
                    segment.distance as f64,
                    segment.point_index as i32,
                    segment.get_t() as f64,
                    segment.segment_type as i32,
                );
            }
        }
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            println!(
                "length {} pts {} segs {}",
                cpp_g(self.length),
                self.points.len(),
                self.segments.len()
            );
            for segment in &self.segments {
                println!(
                    " {} {} {} {}",
                    cpp_g(segment.distance),
                    segment.point_index as i32,
                    cpp_g(segment.get_t()),
                    segment.segment_type as i32
                );
            }
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn cpp_g(value: f32) -> String {
    if value.is_nan() {
        return if value.is_sign_negative() {
            "-nan".to_owned()
        } else {
            "nan".to_owned()
        };
    }
    if value == f32::INFINITY {
        return "inf".to_owned();
    }
    if value == f32::NEG_INFINITY {
        return "-inf".to_owned();
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0".to_owned()
        } else {
            "0".to_owned()
        };
    }
    let scientific = format!("{value:.5e}");
    let (mantissa, exponent) = scientific
        .split_once('e')
        .expect("Rust scientific formatting includes an exponent");
    let exponent: i32 = exponent.parse().expect("Rust formats a numeric exponent");
    if !(-4..6).contains(&exponent) {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        let sign = if exponent < 0 { '-' } else { '+' };
        return format!("{mantissa}e{sign}{:02}", exponent.unsigned_abs());
    }
    let precision = usize::try_from(5 - exponent).expect("fixed %g precision is nonnegative");
    let fixed = format!("{value:.precision$}");
    if fixed.contains('.') {
        fixed.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        fixed
    }
}

fn eval_quad(points: &[Vec2D; 3], t: f32) -> PosTan {
    assert!((0.0..=1.0).contains(&t));
    let eval = EvalQuad::new(points);
    PosTan {
        pos: eval.at(t),
        tan: Vec2D::scale_and_add(eval.b, 2.0 * eval.a, t).normalized(),
    }
}
fn eval_cubic(points: &[Vec2D; 4], t: f32) -> PosTan {
    assert!((0.0..=1.0).contains(&t));
    if t == 0.0 {
        return PosTan {
            pos: points[0],
            tan: (if points[0] != points[1] {
                points[1]
            } else if points[1] != points[2] {
                points[2]
            } else {
                points[3]
            }) - points[0],
        };
    }
    if t == 1.0 {
        return PosTan {
            pos: points[3],
            tan: points[3]
                - if points[3] != points[2] {
                    points[2]
                } else if points[2] != points[1] {
                    points[1]
                } else {
                    points[0]
                },
        };
    }
    let eval = EvalCubic::new(points);
    let tangent = Vec2D::scale_and_add(2.0 * eval.b, 3.0 * eval.a, t);
    PosTan {
        pos: eval.at(t),
        tan: Vec2D::scale_and_add(eval.c, tangent, t).normalized(),
    }
}
fn next_segment_beginning(segments: &[Segment], mut index: usize) -> usize {
    let point_index = segments[index].point_index;
    loop {
        index += 1;
        if segments[index].point_index != point_index {
            return index;
        }
    }
}

#[derive(Clone, Debug)]
struct SourceContour {
    start: Vec2D,
    elements: Vec<SourceElement>,
    closed: bool,
}
#[derive(Clone, Debug)]
enum SourceElement {
    Line([Vec2D; 2]),
    Quad([Vec2D; 3]),
    Cubic([Vec2D; 4]),
}

#[derive(Clone, Debug)]
pub struct ContourMeasureIter {
    contours: Vec<SourceContour>,
    index: usize,
    inverse_tolerance: f32,
    pub segment_counts: Vec<u32>,
}
impl ContourMeasureIter {
    pub const DEFAULT_TOLERANCE: f32 = 0.5;
    pub fn new(path: &RawPath, tolerance: f32) -> Self {
        let mut result = Self {
            contours: Vec::new(),
            index: 0,
            inverse_tolerance: 1.0,
            segment_counts: vec![0; path.verbs().len()],
        };
        result.rewind(path, tolerance);
        result
    }
    pub fn rewind(&mut self, path: &RawPath, tolerance: f32) {
        self.contours = collect_contours(path);
        self.index = 0;
        self.inverse_tolerance = 1.0 / cpp_max(tolerance, 1.0 / 16.0);
        self.segment_counts.resize(path.verbs().len(), 0);
    }
    fn try_next(&mut self) -> Option<Rc<ContourMeasure>> {
        let contour = self.contours.get(self.index)?.clone();
        self.index += 1;
        let mut segment_counts = Vec::new();
        let mut curve_segments = 0usize;
        let mut line_count = 0usize;
        for element in &contour.elements {
            match element {
                SourceElement::Line(points) => {
                    line_count += usize::from(Vec2D::distance_squared(points[1], points[0]) > 0.0)
                }
                SourceElement::Quad(points) => {
                    let count = quadratic_wangs(points, self.inverse_tolerance).ceil() as u32;
                    let count = count.min(100);
                    curve_segments += count as usize;
                    segment_counts.push(count);
                }
                SourceElement::Cubic(points) => {
                    let count = cubic_wangs(points, self.inverse_tolerance).ceil().ceil() as u32;
                    let count = count.min(100);
                    curve_segments += count as usize;
                    segment_counts.push(count);
                }
            }
        }
        self.segment_counts[..segment_counts.len()].copy_from_slice(&segment_counts);
        let mut segments = Vec::with_capacity(curve_segments + line_count);
        let mut points = vec![contour.start];
        let mut distance = 0.0;
        let mut point_index = 0u32;
        let mut curve_index = 0;
        for element in &contour.elements {
            match element {
                SourceElement::Line(line) => {
                    if Vec2D::distance_squared(line[1], line[0]) > 0.0 {
                        distance += (line[1] - line[0]).length();
                        segments.push(Segment {
                            distance,
                            point_index,
                            t_value: MAX_DOT30,
                            segment_type: SegmentType::Line,
                        });
                    }
                    points.push(line[1]);
                    point_index += 1;
                }
                SourceElement::Quad(quad) => {
                    let count = segment_counts[curve_index];
                    curve_index += 1;
                    if count > 0 {
                        distance =
                            add_quad_segments(&mut segments, quad, count, point_index, distance);
                    }
                    points.extend_from_slice(&quad[1..]);
                    point_index += 2;
                }
                SourceElement::Cubic(cubic) => {
                    let count = segment_counts[curve_index];
                    curve_index += 1;
                    if count > 0 {
                        distance =
                            add_cubic_segments(&mut segments, cubic, count, point_index, distance);
                    }
                    points.extend_from_slice(&cubic[1..]);
                    point_index += 3;
                }
            }
        }
        if contour.closed && points.last().copied() != Some(contour.start) {
            let last = *points.last().unwrap();
            distance += (contour.start - last).length();
            segments.push(Segment {
                distance,
                point_index,
                t_value: MAX_DOT30,
                segment_type: SegmentType::Line,
            });
            points.push(contour.start);
        }
        if distance > 0.0 && points.len() >= 2 {
            assert!(!distance.is_nan());
            Some(Rc::new(ContourMeasure::new(
                segments,
                points,
                distance,
                contour.closed,
            )))
        } else {
            assert!(distance == 0.0 || distance.is_nan());
            None
        }
    }
    pub fn next(&mut self) -> Option<Rc<ContourMeasure>> {
        loop {
            let result = self.try_next();
            if result.is_some() || self.index >= self.contours.len() {
                return result;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RefCntContourMeasureIter {
    iterator: ContourMeasureIter,
}
impl RefCntContourMeasureIter {
    pub fn new(path: &RawPath, tolerance: f32) -> Self {
        Self {
            iterator: ContourMeasureIter::new(path, tolerance),
        }
    }
    pub fn get(&mut self) -> &mut ContourMeasureIter {
        &mut self.iterator
    }
}

fn collect_contours(path: &RawPath) -> Vec<SourceContour> {
    let mut contours = Vec::new();
    let mut current: Option<SourceContour> = None;
    for segment in path.segments() {
        match segment.verb {
            PathVerb::Move => {
                if let Some(contour) = current.take() {
                    contours.push(contour);
                }
                current = Some(SourceContour {
                    start: segment.points[0],
                    elements: Vec::new(),
                    closed: false,
                });
            }
            PathVerb::Line => current
                .as_mut()
                .unwrap()
                .elements
                .push(SourceElement::Line([segment.points[0], segment.points[1]])),
            PathVerb::Quad => current
                .as_mut()
                .unwrap()
                .elements
                .push(SourceElement::Quad(segment.points.try_into().unwrap())),
            PathVerb::Cubic => current
                .as_mut()
                .unwrap()
                .elements
                .push(SourceElement::Cubic(segment.points.try_into().unwrap())),
            PathVerb::Close => current.as_mut().unwrap().closed = true,
        }
    }
    if let Some(contour) = current {
        contours.push(contour);
    }
    contours
}
fn to_dot30(value: f32) -> u32 {
    assert!(value >= 0.0 && value < 1.0);
    (value * (1u32 << 30) as f32) as u32
}
fn add_quad_segments(
    output: &mut Vec<Segment>,
    points: &[Vec2D; 3],
    count: u32,
    point_index: u32,
    mut distance: f32,
) -> f32 {
    let delta = 1.0 / count as f32;
    let eval = EvalQuad::new(points);
    let mut t = delta;
    let mut previous = points[0];
    for _ in 1..count {
        let next = eval.at(t);
        distance += (next - previous).length();
        output.push(Segment {
            distance,
            point_index,
            t_value: to_dot30(t),
            segment_type: SegmentType::Quad,
        });
        previous = next;
        t += delta;
    }
    distance += (points[2] - previous).length();
    output.push(Segment {
        distance,
        point_index,
        t_value: MAX_DOT30,
        segment_type: SegmentType::Quad,
    });
    distance
}
fn add_cubic_segments(
    output: &mut Vec<Segment>,
    points: &[Vec2D; 4],
    count: u32,
    point_index: u32,
    mut distance: f32,
) -> f32 {
    let delta = 1.0 / count as f32;
    let eval = EvalCubic::new(points);
    let mut t = delta;
    let mut previous = points[0];
    for _ in 1..count {
        let next = eval.at(t);
        distance += (next - previous).length();
        output.push(Segment {
            distance,
            point_index,
            t_value: to_dot30(t),
            segment_type: SegmentType::Cubic,
        });
        previous = next;
        t += delta;
    }
    distance += (points[3] - previous).length();
    output.push(Segment {
        distance,
        point_index,
        t_value: MAX_DOT30,
        segment_type: SegmentType::Cubic,
    });
    distance
}
fn quadratic_wangs(points: &[Vec2D; 3], precision: f32) -> f32 {
    let v = points[0] - 2.0 * points[1] + points[2];
    let length_term_pow2 = (1.0 / 16.0) * (precision * precision);
    (v.length_squared() * length_term_pow2).sqrt().sqrt()
}
fn cubic_wangs(points: &[Vec2D; 4], precision: f32) -> f32 {
    let v0 = points[0] - 2.0 * points[1] + points[2];
    let v1 = points[1] - 2.0 * points[2] + points[3];
    let length_term_pow2 = (9.0 / 16.0) * (precision * precision);
    (cpp_max(v0.length_squared(), v1.length_squared()) * length_term_pow2)
        .sqrt()
        .sqrt()
}
fn cpp_min(first: f32, second: f32) -> f32 {
    if second < first { second } else { first }
}
fn cpp_max(first: f32, second: f32) -> f32 {
    if first < second { second } else { first }
}
