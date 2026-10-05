use std::{borrow::Cow, rc::Rc};

use super::math_types;
use super::path_types::PathVerb;
use super::raw_path::{RawPath, RawPathCursor};
use super::raw_path_utils::{EvalCubic, EvalQuad, cubic_extract, line_extract, quad_extract};
use super::vec2d::Vec2D;
use super::wangs_formula::{self, VectorXform};

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
#[repr(C)]
pub struct Segment {
    distance: f32,
    point_index: u32,
    // Upstream stores a 30-bit t value and a 2-bit type in one word.
    packed_t_and_type: u32,
}
impl Segment {
    fn new(distance: f32, point_index: u32, t_value: u32, segment_type: SegmentType) -> Self {
        Self {
            distance,
            point_index,
            packed_t_and_type: (t_value & MAX_DOT30) | ((segment_type as u32) << 30),
        }
    }
    fn t_value(self) -> u32 {
        self.packed_t_and_type & MAX_DOT30
    }
    fn segment_type(self) -> SegmentType {
        match self.packed_t_and_type >> 30 {
            0 => SegmentType::Line,
            1 => SegmentType::Quad,
            2 => SegmentType::Cubic,
            _ => unreachable!("invalid segment type"),
        }
    }
    pub fn get_t(self) -> f32 {
        self.t_value() as f32 * INV_SCALE_D30
    }
    fn extract_all(self, dst: &mut RawPath, points: &[Vec2D]) {
        let points = &points[self.point_index as usize..];
        match self.segment_type() {
            SegmentType::Line => dst.line_to_point(points[1]),
            SegmentType::Quad => dst.quad_to_points(points[1], points[2]),
            SegmentType::Cubic => dst.cubic_to_points(points[1], points[2], points[3]),
        }
    }
    fn extract(self, dst: &mut RawPath, from_t: f32, to_t: f32, points: &[Vec2D], move_to: bool) {
        assert!(from_t <= to_t);
        let points = &points[self.point_index as usize..];
        match self.segment_type() {
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
    fn segment_packs_the_source_t_value_and_type_in_twelve_bytes() {
        assert_eq!(std::mem::size_of::<Segment>(), 12);
        for kind in [SegmentType::Line, SegmentType::Quad, SegmentType::Cubic] {
            for t_value in [0, 1, MAX_DOT30 / 2, MAX_DOT30] {
                let segment = Segment::new(5.0, 7, t_value, kind);
                assert_eq!(segment.distance, 5.0);
                assert_eq!(segment.point_index, 7);
                assert_eq!(segment.t_value(), t_value);
                assert_eq!(segment.segment_type(), kind);
                assert_eq!(
                    segment.get_t().to_bits(),
                    (t_value as f32 * INV_SCALE_D30).to_bits()
                );
            }
        }
    }

    #[test]
    fn line_segment_extract_matches_pinned_exact_lerp_grouping() {
        let segment = Segment::new(1.0, 0, MAX_DOT30, SegmentType::Line);
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
            [0x41eb_e53a, 0x4152_996b]
        );
        assert_eq!(
            path.points()
                .iter()
                .map(|point| point.y.to_bits())
                .collect::<Vec<_>>(),
            [0xc2da_38af, 0xc335_cd98]
        );
    }

    #[test]
    fn line_pos_tan_keeps_the_pinned_separate_lerp() {
        let p0 = Vec2D::new(39.608627, -64.03908);
        let p1 = Vec2D::new(12.428378, -185.07193);
        let mut path = RawPath::default();
        path.move_to_point(p0);
        path.line_to_point(p1);
        let contour = ContourMeasureIter::new(&path, ContourMeasureIter::DEFAULT_TOLERANCE)
            .next()
            .unwrap();
        let distance = contour.length() * 0.37239173;
        let ratio = distance / contour.length();
        let expected = p0 + (p1 - p0) * ratio;

        let actual = contour.get_pos_tan(distance);

        assert_eq!(actual.pos.x.to_bits(), expected.x.to_bits());
        assert_eq!(actual.pos.y.to_bits(), expected.y.to_bits());
    }

    #[test]
    fn curve_distance_interpolation_obeys_its_pinned_scalar_mode() {
        let points = [
            Vec2D::new(39.608627, -64.03908),
            Vec2D::new(12.428378, -185.07193),
            Vec2D::new(96.2807, 71.687096),
        ];
        let mut path = RawPath::default();
        path.move_to_point(points[0]);
        path.quad_to_points(points[1], points[2]);
        let contour = ContourMeasureIter::new(&path, ContourMeasureIter::DEFAULT_TOLERANCE)
            .next()
            .unwrap();
        assert!(contour.segments.len() > 1);
        for index in 1..contour.segments.len() {
            let previous = contour.segments[index - 1];
            let segment = contour.segments[index];
            let distance = previous.distance + (segment.distance - previous.distance) * 0.37239173;
            let ratio = (distance - previous.distance) / (segment.distance - previous.distance);
            #[cfg(feature = "strict-fp")]
            let t = previous.get_t() * (1.0 - ratio) + segment.get_t() * ratio;
            #[cfg(not(feature = "strict-fp"))]
            let t = previous
                .get_t()
                .mul_add(1.0 - ratio, segment.get_t() * ratio);
            let eval = EvalQuad::new(&points);
            let expected = (eval.a * t + eval.b) * t + eval.c;
            let expected_tangent = ((eval.a + eval.a) * t + eval.b).normalized();

            assert_eq!(contour.compute_t(index, distance).to_bits(), t.to_bits());
            let actual = contour.get_pos_tan(distance);
            assert_eq!(actual.pos.x.to_bits(), expected.x.to_bits());
            assert_eq!(actual.pos.y.to_bits(), expected.y.to_bits());
            assert_eq!(actual.tan.x.to_bits(), expected_tangent.x.to_bits());
            assert_eq!(actual.tan.y.to_bits(), expected_tangent.y.to_bits());
        }
    }

    #[test]
    fn borrowed_iterator_uses_the_source_but_returns_independent_geometry() {
        let contour = {
            let mut path = RawPath::default();
            path.move_to(1.0, 2.0);
            path.line_to(4.0, 6.0);
            let mut iter = ContourMeasureIter::new(&path, 0.5);
            assert!(matches!(&iter.path, Cow::Borrowed(_)));
            assert!(std::ptr::eq(iter.path.as_ref(), &path));
            iter.next().unwrap()
        };
        assert_eq!(contour.length(), 5.0);
        assert_eq!(contour.get_pos_tan(5.0).pos, Vec2D::new(4.0, 6.0));
        let mut extracted = RawPath::default();
        contour.get_segment(0.0, 5.0, &mut extracted, true);
        assert_eq!(
            extracted.points(),
            &[Vec2D::new(1.0, 2.0), Vec2D::new(4.0, 6.0)]
        );
    }

    #[test]
    fn owning_iterator_survives_source_changes_and_clones_its_current_position() {
        let mut owner = {
            let mut path = RawPath::default();
            path.move_to(1.0, 2.0);
            path.line_to(4.0, 6.0);
            path.move_to(10.0, 10.0);
            path.line_to(13.0, 10.0);
            let owner = RefCntContourMeasureIter::new(&path, 0.5);
            assert!(matches!(&owner.iterator.path, Cow::Owned(_)));
            path.rewind();
            owner
        };
        assert_eq!(owner.get().next().unwrap().length(), 5.0);
        let mut clone = owner.clone();
        drop(owner);
        let second = clone.get().next().unwrap();
        assert_eq!(second.length(), 3.0);
        assert_eq!(second.get_pos_tan(0.0).pos, Vec2D::new(10.0, 10.0));
        assert!(clone.get().next().is_none());
    }

    #[test]
    fn contour_passes_reuse_scratch_and_preserve_degenerate_and_closing_points() {
        let mut path = RawPath::default();
        path.move_to(0.0, 0.0);
        path.line_to(0.0, 0.0);
        path.quad_to(0.0, 0.0, 0.0, 0.0);
        path.cubic_to(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        path.line_to(3.0, 0.0);
        path.close();
        path.move_to(10.0, 0.0);
        path.quad_to(11.0, 2.0, 12.0, 0.0);
        path.cubic_to(13.0, 3.0, 14.0, -3.0, 15.0, 0.0);
        path.line_to(10.0, 0.0);
        path.close();
        let mut iter = ContourMeasureIter::new(&path, 0.5);
        let scratch = iter.segment_counts.as_ptr();
        let capacity = iter.segment_counts.capacity();
        let first = iter.next().unwrap();
        assert_eq!(&iter.segment_counts[..2], &[0, 0]);
        assert_eq!(first.length(), 6.0);
        assert!(first.is_closed());
        assert_eq!(first.segments.len(), 2);
        assert_eq!(first.points.len(), 9);
        assert_eq!(first.points.last(), Some(&Vec2D::new(0.0, 0.0)));
        let second = iter.next().unwrap();
        assert!(iter.segment_counts[..2].iter().all(|&count| count > 0));
        assert!(second.is_closed());
        assert_eq!(second.points.len(), 7);
        assert_eq!(second.points.first(), second.points.last());
        assert_eq!(iter.segment_counts.as_ptr(), scratch);
        assert_eq!(iter.segment_counts.capacity(), capacity);
        assert!(iter.next().is_none());
        iter.rewind(&path, 0.5);
        assert_eq!(iter.segment_counts.as_ptr(), scratch);
        assert_eq!(iter.segment_counts.capacity(), capacity);
        assert_eq!(iter.next().unwrap().points, first.points);
    }

    #[test]
    fn contour_counts_apply_the_source_segment_cap_and_tolerance_floor() {
        let mut path = RawPath::default();
        path.move_to(0.0, 0.0);
        path.quad_to(1_000_000.0, 1_000_000.0, 2_000_000.0, 0.0);
        path.cubic_to(
            3_000_000.0,
            1_000_000.0,
            4_000_000.0,
            -1_000_000.0,
            5_000_000.0,
            0.0,
        );
        let mut iter = ContourMeasureIter::new(&path, 0.0);
        let contour = iter.next().unwrap();
        assert_eq!(&iter.segment_counts[..2], &[100, 100]);
        assert_eq!(contour.segments.len(), 200);
        let at_floor = ContourMeasureIter::new(&path, 1.0 / 16.0).next().unwrap();
        assert_eq!(contour.length().to_bits(), at_floor.length().to_bits());
        assert_eq!(contour.points, at_floor.points);
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
        if segment.segment_type() == SegmentType::Line {
            let p0 = self.points[point_index];
            let p1 = self.points[point_index + 1];
            return PosTan {
                // This Vec2D::lerp call remains separate in the pinned
                // shipping owner; the shared geometry helper contracts.
                pos: p0 + (p1 - p0) * relative_distance,
                tan: (p1 - p0).normalized(),
            };
        }
        let previous_t = if index > 0 && self.segments[index - 1].point_index == segment.point_index
        {
            self.segments[index - 1].get_t()
        } else {
            0.0
        };
        let t = interpolate_segment_t(previous_t, segment.get_t(), relative_distance);
        assert!((0.0..=1.0).contains(&t));
        if segment.segment_type() == SegmentType::Quad {
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
        let t = interpolate_segment_t(previous_t, segment.get_t(), ratio);
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
                    segment.segment_type() as i32,
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
                    segment.segment_type() as i32
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
    let a = eval.a + eval.a;
    PosTan {
        pos: (eval.a * t + eval.b) * t + eval.c,
        tan: (a * t + eval.b).normalized(),
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
    let a = eval.a * 3.0;
    let b = eval.b + eval.b;
    PosTan {
        pos: eval.at(t),
        tan: ((a * t + b) * t + eval.c).normalized(),
    }
}

#[inline]
fn interpolate_segment_t(previous: f32, next: f32, ratio: f32) -> f32 {
    #[cfg(feature = "strict-fp")]
    {
        previous * (1.0 - ratio) + next * ratio
    }
    #[cfg(not(feature = "strict-fp"))]
    {
        // C++ rive::lerp rounds its right product, then contracts the left
        // product and final sum in this shipping scalar call context.
        previous.mul_add(1.0 - ratio, next * ratio)
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
pub struct ContourMeasureIter<'a> {
    path: Cow<'a, RawPath>,
    cursor: RawPathCursor,
    inverse_tolerance: f32,
    pub segment_counts: Vec<u32>,
}
impl<'a> ContourMeasureIter<'a> {
    pub const DEFAULT_TOLERANCE: f32 = 0.5;

    /// Upstream's pointer constructor borrows the source for iteration. Each
    /// returned measure owns its points and can outlive both source and iterator.
    pub fn new(path: &'a RawPath, tolerance: f32) -> Self {
        Self::from_path(Cow::Borrowed(path), tolerance)
    }

    fn from_path(path: Cow<'a, RawPath>, tolerance: f32) -> Self {
        let segment_counts = vec![0; path.verbs().len()];
        Self {
            path,
            cursor: RawPathCursor::default(),
            inverse_tolerance: 1.0 / cpp_max(tolerance, 1.0 / 16.0),
            segment_counts,
        }
    }

    pub fn rewind(&mut self, path: &'a RawPath, tolerance: f32) {
        self.path = Cow::Borrowed(path);
        self.cursor = RawPathCursor::default();
        self.inverse_tolerance = 1.0 / cpp_max(tolerance, 1.0 / 16.0);
        self.segment_counts.resize(path.verbs().len(), 0);
    }

    fn try_next(&mut self) -> Option<Rc<ContourMeasure>> {
        let path = self.path.as_ref();
        let mut iter = path.iter_from(self.cursor);
        let end = path.end();
        assert!(iter == end || iter.verb() == PathVerb::Move);

        // Skip empty contours, retaining the most recent move point.
        let mut start = Vec2D::default();
        loop {
            if iter == end {
                self.cursor = iter.cursor();
                return None;
            }
            match iter.verb() {
                PathVerb::Move => start = iter.move_point(),
                PathVerb::Close => {}
                _ => break,
            }
            let _ = iter.next();
        }

        // Pass 1 counts directly from the source path and reuses the scratch
        // buffer sized by rewind, including any closing line in the reservation.
        let first = iter.clone();
        let mut end_of_contour = end.clone();
        let mut curve_segments = 0usize;
        let mut line_count = 0usize;
        let mut curve_index = 0usize;
        let mut closed = false;
        let mut scan = first.clone();
        while scan != end {
            match scan.verb() {
                PathVerb::Move => {
                    end_of_contour = scan;
                    break;
                }
                PathVerb::Line => {
                    let points = scan.line_points();
                    line_count += usize::from(Vec2D::distance_squared(points[1], points[0]) > 0.0);
                }
                PathVerb::Quad => {
                    let count = wangs_formula::quadratic(
                        scan.quad_points().try_into().unwrap(),
                        self.inverse_tolerance,
                        VectorXform::default(),
                    )
                    .ceil() as u32;
                    let count = count.min(100);
                    curve_segments += count as usize;
                    self.segment_counts[curve_index] = count;
                    curve_index += 1;
                }
                PathVerb::Cubic => {
                    let count = wangs_formula::cubic(
                        scan.cubic_points().try_into().unwrap(),
                        self.inverse_tolerance,
                        VectorXform::default(),
                    )
                    .ceil()
                    .ceil() as u32;
                    let count = count.min(100);
                    curve_segments += count as usize;
                    self.segment_counts[curve_index] = count;
                    curve_index += 1;
                }
                PathVerb::Close => {
                    line_count += usize::from(scan.point_before_close() != start);
                    closed = true;
                }
            }
            let _ = scan.next();
        }

        // Pass 2 emits into the fully reserved segment buffer. The math and
        // source point indices follow the same path traversal as pass 1.
        let mut segments = Vec::with_capacity(curve_segments + line_count);
        let mut distance = 0.0;
        let mut point_index = 0u32;
        let mut curve_index = 0;
        let mut duplicate_start = false;
        while iter != end_of_contour {
            match iter.verb() {
                PathVerb::Move => unreachable!("move begins the next contour"),
                PathVerb::Line => {
                    let line = iter.line_points();
                    if Vec2D::distance_squared(line[1], line[0]) > 0.0 {
                        distance += (line[1] - line[0]).length();
                        segments.push(Segment::new(
                            distance,
                            point_index,
                            MAX_DOT30,
                            SegmentType::Line,
                        ));
                    }
                    point_index += 1;
                }
                PathVerb::Quad => {
                    let count = self.segment_counts[curve_index];
                    curve_index += 1;
                    if count > 0 {
                        distance = add_quad_segments(
                            &mut segments,
                            iter.quad_points().try_into().unwrap(),
                            count,
                            point_index,
                            distance,
                        );
                    }
                    point_index += 2;
                }
                PathVerb::Cubic => {
                    let count = self.segment_counts[curve_index];
                    curve_index += 1;
                    if count > 0 {
                        distance = add_cubic_segments(
                            &mut segments,
                            iter.cubic_points().try_into().unwrap(),
                            count,
                            point_index,
                            distance,
                        );
                    }
                    point_index += 3;
                }
                PathVerb::Close => {
                    let last = iter.point_before_close();
                    if last != start {
                        distance += (start - last).length();
                        segments.push(Segment::new(
                            distance,
                            point_index,
                            MAX_DOT30,
                            SegmentType::Line,
                        ));
                        point_index += 1;
                        duplicate_start = true;
                    }
                    assert!(closed);
                }
            }
            let _ = iter.next();
        }
        assert_eq!(segments.len(), curve_segments + line_count);

        // Measures retain their own contiguous point range, independent of the
        // borrowed or copied source that this iterator is currently reading.
        let mut points = Vec::with_capacity(1 + point_index as usize);
        points.extend_from_slice(
            &path.points()[first.cursor().point_index() - 1..end_of_contour.cursor().point_index()],
        );
        if duplicate_start {
            points.push(start);
        }
        assert_eq!(points.len(), 1 + point_index as usize);
        self.cursor = end_of_contour.cursor();

        if distance > 0.0 && points.len() >= 2 {
            assert!(!distance.is_nan());
            Some(Rc::new(ContourMeasure::new(
                segments, points, distance, closed,
            )))
        } else {
            assert!(distance == 0.0 || distance.is_nan());
            None
        }
    }
    pub fn next(&mut self) -> Option<Rc<ContourMeasure>> {
        loop {
            let result = self.try_next();
            if result.is_some() || self.cursor == self.path.end().cursor() {
                return result;
            }
        }
    }
}

impl ContourMeasureIter<'static> {
    /// Upstream's reference constructor copies the source for an iterator that
    /// can outlive it, as required by the scripting wrapper.
    pub fn from_path_copy(path: &RawPath, tolerance: f32) -> Self {
        Self::from_path(Cow::Owned(path.clone()), tolerance)
    }
}

#[derive(Clone, Debug)]
pub struct RefCntContourMeasureIter {
    iterator: ContourMeasureIter<'static>,
}
impl RefCntContourMeasureIter {
    pub fn new(path: &RawPath, tolerance: f32) -> Self {
        Self {
            iterator: ContourMeasureIter::from_path_copy(path, tolerance),
        }
    }
    pub fn get(&mut self) -> &mut ContourMeasureIter<'static> {
        &mut self.iterator
    }
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
        let next = (eval.a * t + eval.b) * t + eval.c;
        distance += (next - previous).length();
        output.push(Segment::new(
            distance,
            point_index,
            to_dot30(t),
            SegmentType::Quad,
        ));
        previous = next;
        t += delta;
    }
    distance += (points[2] - previous).length();
    output.push(Segment::new(
        distance,
        point_index,
        MAX_DOT30,
        SegmentType::Quad,
    ));
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
        output.push(Segment::new(
            distance,
            point_index,
            to_dot30(t),
            SegmentType::Cubic,
        ));
        previous = next;
        t += delta;
    }
    distance += (points[3] - previous).length();
    output.push(Segment::new(
        distance,
        point_index,
        MAX_DOT30,
        SegmentType::Cubic,
    ));
    distance
}
fn cpp_min(first: f32, second: f32) -> f32 {
    if second < first { second } else { first }
}
fn cpp_max(first: f32, second: f32) -> f32 {
    if first < second { second } else { first }
}
