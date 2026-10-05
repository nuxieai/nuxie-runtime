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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawPathCursor {
    verb: usize,
    point: usize,
}

/// Borrowed upstream RawPath::Iter. No segment list or point storage is copied.
#[derive(Clone, Debug, Default)]
pub struct RawPathIter<'a> {
    verbs: &'a [PathVerb],
    points: &'a [Vec2D],
    cursor: RawPathCursor,
}

impl<'a> RawPathIter<'a> {
    pub fn cursor(&self) -> RawPathCursor {
        self.cursor
    }
    pub fn points_advance_after_verb(verb: PathVerb) -> usize {
        path_verb_to_point_count(verb)
    }
    pub fn points_backset_for_verb(verb: PathVerb) -> isize {
        if verb == PathVerb::Move { 0 } else { -1 }
    }
    pub fn raw_verbs_ptr(&self) -> *const PathVerb {
        self.verbs.as_ptr().wrapping_add(self.cursor.verb)
    }
    pub fn raw_points_ptr(&self) -> *const Vec2D {
        self.points.as_ptr().wrapping_add(self.cursor.point)
    }
    pub fn verb(&self) -> PathVerb {
        self.verbs[self.cursor.verb]
    }
    pub fn points(&self) -> &'a [Vec2D] {
        let verb = self.verb();
        let start = if verb == PathVerb::Move {
            self.cursor.point
        } else {
            self.cursor.point - 1
        };
        &self.points[start..self.cursor.point + path_verb_to_point_count(verb)]
    }
    pub fn move_point(&self) -> Vec2D {
        debug_assert_eq!(self.verb(), PathVerb::Move);
        self.points[self.cursor.point]
    }
    pub fn line_points(&self) -> &'a [Vec2D] {
        debug_assert_eq!(self.verb(), PathVerb::Line);
        self.points()
    }
    pub fn quad_points(&self) -> &'a [Vec2D] {
        debug_assert_eq!(self.verb(), PathVerb::Quad);
        self.points()
    }
    pub fn cubic_points(&self) -> &'a [Vec2D] {
        debug_assert_eq!(self.verb(), PathVerb::Cubic);
        self.points()
    }
    pub fn point_before_close(&self) -> Vec2D {
        debug_assert_eq!(self.verb(), PathVerb::Close);
        self.points[self.cursor.point - 1]
    }
}

impl<'a> Iterator for RawPathIter<'a> {
    type Item = PathSegment<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let verb = *self.verbs.get(self.cursor.verb)?;
        let points = self.points();
        self.cursor.verb += 1;
        self.cursor.point += path_verb_to_point_count(verb);
        Some(PathSegment { verb, points })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.verbs.len() - self.cursor.verb;
        (remaining, Some(remaining))
    }
}
impl ExactSizeIterator for RawPathIter<'_> {}
impl std::iter::FusedIterator for RawPathIter<'_> {}
impl PartialEq for RawPathIter<'_> {
    fn eq(&self, other: &Self) -> bool {
        let same_verb = self.raw_verbs_ptr() == other.raw_verbs_ptr();
        debug_assert!(!same_verb || self.raw_points_ptr() == other.raw_points_ptr());
        same_verb
    }
}
impl Eq for RawPathIter<'_> {}

impl Default for RawPathCursor {
    fn default() -> Self {
        Self { verb: 0, point: 0 }
    }
}

impl RawPathCursor {
    pub(crate) fn point_index(self) -> usize {
        self.point
    }
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
    /// Upstream trusted-span constructor; bookkeeping stays default.
    pub fn from_slices(verbs: &[PathVerb], points: &[Vec2D]) -> Self {
        Self {
            verbs: verbs.to_vec(),
            points: points.to_vec(),
            ..Self::default()
        }
    }
    pub const COARSE_AREA_TOLERANCE: f32 = 8.0;
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
    /// Borrow the original path buffers at the renderer boundary, without
    /// replaying geometry or changing contour construction state.
    pub fn as_render_path_ref(&self) -> nuxie_render_api::RawPathRef<'_> {
        nuxie_render_api::RawPathRef::new(&self.verbs, bytemuck::cast_slice(&self.points))
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
    pub fn inject_implicit_move_if_needed(&mut self) {
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
        map_points_in_place(&matrix, &mut self.points);
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
        debug_assert!(!self.points.is_empty());
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
        self.reserve(6, 5);
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
        self.reserve(6, 13);
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
        let capacity = points.len() + usize::from(is_closed);
        self.reserve(capacity, capacity);
        self.move_to_point(first);
        for point in &points[1..] {
            self.line_to_point(*point);
        }
        if is_closed {
            self.close();
        }
    }

    pub fn segments(&self) -> RawPathIter<'_> {
        self.begin()
    }
    pub fn begin(&self) -> RawPathIter<'_> {
        RawPathIter {
            verbs: &self.verbs,
            points: &self.points,
            cursor: RawPathCursor::default(),
        }
    }
    pub(crate) fn iter_from(&self, cursor: RawPathCursor) -> RawPathIter<'_> {
        RawPathIter {
            verbs: &self.verbs,
            points: &self.points,
            cursor,
        }
    }
    pub fn end(&self) -> RawPathIter<'_> {
        RawPathIter {
            verbs: &self.verbs,
            points: &self.points,
            cursor: RawPathCursor {
                verb: self.verbs.len(),
                point: self.points.len(),
            },
        }
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
    // Upstream retains this private reverse-point helper although current
    // addPathBackwards performs the bulk reverse directly.
    #[allow(dead_code)]
    fn add_points<'a>(
        &mut self,
        points: &mut impl Iterator<Item = &'a Vec2D>,
        count: usize,
        matrix: Option<&Mat2D>,
    ) {
        for _ in 0..count {
            let point = *points
                .next()
                .expect("source reverse iterator has enough points");
            self.points.push(match matrix {
                Some(matrix) => Vec2D::transform_mat2d(point, matrix),
                None => point,
            });
        }
    }
    /// Safe-Rust entry for upstream's `addPath(*this, matrix)` aliasing branch.
    /// The source must be copied before extending either backing vector.
    pub fn add_self_path(&mut self, matrix: Option<&Mat2D>) -> RawPathCursor {
        let copy = self.clone();
        self.add_path(&copy, matrix)
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
        self.points.reserve(source.points.len());
        self.points.extend(source.points.iter().rev().copied());
        let initial_verb_count = self.verbs.len();
        self.verbs.reserve(source.verbs.len());
        debug_assert_eq!(source.verbs.first(), Some(&PathVerb::Move));
        self.verbs.push(PathVerb::Move);
        let mut closed = false;
        for (reverse_index, verb) in source.verbs.iter().rev().copied().enumerate() {
            if verb == PathVerb::Close {
                debug_assert!(!closed);
                closed = true;
                debug_assert_ne!(reverse_index + 1, source.verbs.len());
                continue;
            }
            if verb == PathVerb::Move && closed {
                self.verbs.push(PathVerb::Close);
                closed = false;
            }
            if reverse_index + 1 != source.verbs.len() {
                self.verbs.push(verb);
            } else {
                debug_assert_eq!(verb, PathVerb::Move);
            }
        }
        debug_assert!(!closed);
        debug_assert_eq!(self.verbs.len(), initial_verb_count + source.verbs.len());
        debug_assert_eq!(self.points.len(), initial_point_count + source.points.len());
        if let Some(matrix) = matrix {
            map_points_in_place(matrix, &mut self.points[initial_point_count..]);
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
        let mut dst_verb = start_verb;
        let mut dst_point = start_point;
        let mut point_index = start_point;
        for src_verb in start_verb..self.verbs.len() {
            let verb = self.verbs[src_verb];
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
                if src_verb != dst_verb {
                    self.verbs[dst_verb] = verb;
                    self.points
                        .copy_within(point_index..point_index + advance, dst_point);
                }
                dst_verb += 1;
                dst_point += advance;
            }
            point_index += advance;
        }
        if dst_verb != self.verbs.len() {
            self.verbs.truncate(dst_verb);
            self.points.truncate(dst_point);
        }
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
        // C++ reserve takes total capacity; Vec::reserve takes additional length.
        self.verbs.reserve(verbs.saturating_sub(self.verbs.len()));
        self.points
            .reserve(points.saturating_sub(self.points.len()));
    }
    pub fn precise_bounds(&self) -> Aabb {
        self.precise_bounds_with_transform(Mat2D::identity())
    }

    /// Tight bounds after applying an affine transform, without copying the path.
    pub fn precise_bounds_with_transform(&self, xform: Mat2D) -> Aabb {
        let mut bounds = Aabb::for_expansion();
        for segment in self.segments() {
            match segment.verb {
                PathVerb::Move => Aabb::expand_to_point(&mut bounds, xform * segment.points[0]),
                PathVerb::Line => Aabb::expand_to_point(&mut bounds, xform * segment.points[1]),
                PathVerb::Cubic => {
                    let p0 = xform * segment.points[0];
                    let p1 = xform * segment.points[1];
                    let p2 = xform * segment.points[2];
                    let p3 = xform * segment.points[3];
                    expand_cubic_bounds_for_axis(&mut bounds, 0, p0.x, p1.x, p2.x, p3.x);
                    expand_cubic_bounds_for_axis(&mut bounds, 1, p0.y, p1.y, p2.y, p3.y);
                }
                PathVerb::Quad => {
                    let p0 = xform * segment.points[0];
                    let p1 = xform * segment.points[1];
                    let p2 = xform * segment.points[2];
                    let pt1 = Vec2D::lerp(p0, p1, 2.0 / 3.0);
                    let pt2 = Vec2D::lerp(p2, p1, 2.0 / 3.0);
                    expand_cubic_bounds_for_axis(&mut bounds, 0, p0.x, pt1.x, pt2.x, p2.x);
                    expand_cubic_bounds_for_axis(&mut bounds, 1, p0.y, pt1.y, pt2.y, p2.y);
                }
                PathVerb::Close => {}
            }
        }
        bounds
    }
    pub fn compute_coarse_area(&self) -> f32 {
        self.compute_coarse_area_with_origin(Vec2D::default())
    }

    /// An origin near the path avoids cancellation in the area products.
    /// The default remains zero so existing renderer decisions are unchanged.
    pub fn compute_coarse_area_with_origin(&self, origin: Vec2D) -> f32 {
        let mut area = 0.0;
        let mut contour_start = Vec2D::default();
        let mut last = Vec2D::default();
        for segment in self.segments() {
            match segment.verb {
                PathVerb::Move => {
                    area += Vec2D::cross(last, contour_start);
                    contour_start = segment.points[0] - origin;
                    last = segment.points[0] - origin;
                }
                PathVerb::Close => {}
                PathVerb::Line => {
                    area += Vec2D::cross(last, segment.points[1] - origin);
                    last = segment.points[1] - origin;
                }
                PathVerb::Quad => unreachable!(),
                PathVerb::Cubic => {
                    let points: &[Vec2D; 4] = segment.points.try_into().unwrap();
                    let mut count = super::wangs_formula::cubic(
                        points,
                        1.0 / Self::COARSE_AREA_TOLERANCE,
                        super::wangs_formula::VectorXform::default(),
                    )
                    .ceil();
                    if count > 1.0 {
                        count = if 64.0 < count { 64.0 } else { count };
                        let eval = EvalCubic::new(points);
                        let inverse_count = 1.0 / count;
                        let mut low_t = inverse_count;
                        let mut high_t = 2.0 * inverse_count;
                        let delta_t = high_t;
                        while low_t < 1.0 {
                            let point = eval.at(low_t) - origin;
                            area += Vec2D::cross(last, point);
                            last = point;
                            // Source t is [lo, lo, hi, hi], and checks t.y.
                            if low_t < 1.0 {
                                let point = eval.at(high_t) - origin;
                                area += Vec2D::cross(last, point);
                                last = point;
                            }
                            low_t += delta_t;
                            high_t += delta_t;
                        }
                    }
                    area += Vec2D::cross(last, points[3] - origin);
                    last = points[3] - origin;
                }
            }
        }
        area += Vec2D::cross(last, contour_start);
        area * 0.5
    }
}

// Safe aliasing adaptation of Mat2D::mapPoints(dst, dst, count). Retain the
// existing map_points no-skew branch and FMA stages, rather than Mul<Vec2D>.
fn map_points_in_place(matrix: &Mat2D, points: &mut [Vec2D]) {
    for point in points {
        let source = [*point];
        matrix.map_points(std::slice::from_mut(point), &source);
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
#[cfg(test)]
mod render_view_tests {
    use super::{PathVerb, RawPath, Vec2D};

    #[test]
    fn render_view_borrows_original_buffers_and_preserves_order_and_float_bits() {
        let verbs = [
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Quad,
            PathVerb::Cubic,
            PathVerb::Close,
        ];
        let points = [
            Vec2D::new(-0.0, 0.0),
            Vec2D::new(f32::from_bits(0x7fc0_1234), f32::NEG_INFINITY),
            Vec2D::new(f32::INFINITY, f32::from_bits(1)),
            Vec2D::new(1.0, -2.0),
            Vec2D::new(3.0, -4.0),
            Vec2D::new(5.0, -6.0),
            Vec2D::new(7.0, -8.0),
        ];
        let path = RawPath::from_slices(&verbs, &points);
        let view = path.as_render_path_ref();

        assert!(!view.empty());
        assert_eq!(view.verbs().as_ptr(), path.verbs().as_ptr());
        assert_eq!(
            view.points().as_ptr().cast::<u8>(),
            path.points().as_ptr().cast::<u8>()
        );
        assert_eq!(view.verbs(), verbs);
        assert_eq!(
            view.verbs()
                .iter()
                .map(|verb| *verb as u8)
                .collect::<Vec<_>>(),
            [0, 1, 2, 4, 5]
        );
        assert_eq!(view.points().len(), points.len());
        for (actual, expected) in view.points().iter().zip(points) {
            assert_eq!(actual.x.to_bits(), expected.x.to_bits());
            assert_eq!(actual.y.to_bits(), expected.y.to_bits());
        }
    }

    #[test]
    fn render_view_preserves_empty_storage() {
        let path = RawPath::default();
        let view = path.as_render_path_ref();

        assert!(view.empty());
        assert!(view.verbs().is_empty());
        assert!(view.points().is_empty());
        assert_eq!(view.verbs().as_ptr(), path.verbs().as_ptr());
        assert_eq!(
            view.points().as_ptr().cast::<u8>(),
            path.points().as_ptr().cast::<u8>()
        );
    }
}

#[cfg(test)]
mod prune_tests {
    use super::*;

    #[test]
    fn trusted_arrays_and_borrowed_iterator_match_source() {
        let verbs = [
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Quad,
            PathVerb::Cubic,
            PathVerb::Close,
        ];
        let points = [
            (1.0, 2.0),
            (3.0, 4.0),
            (5.0, 6.0),
            (7.0, 8.0),
            (9.0, 10.0),
            (11.0, 12.0),
            (13.0, 14.0),
        ]
        .map(|(x, y)| point(x, y));
        let path = RawPath::from_slices(&verbs, &points);
        assert!(!path.contour_is_open);
        assert_eq!(path.last_move_index, 0);
        assert_eq!(path.verbs(), &verbs);
        assert_eq!(path.points(), &points);
        let mut iter = path.begin();
        assert_eq!(iter.len(), 5);
        assert_eq!(iter.move_point(), points[0]);
        assert_eq!(iter.next().unwrap().points.as_ptr(), path.points.as_ptr());
        assert_eq!(iter.line_points(), &points[0..2]);
        iter.next();
        assert_eq!(iter.quad_points(), &points[1..4]);
        iter.next();
        assert_eq!(iter.cubic_points(), &points[3..7]);
        iter.next();
        assert_eq!(iter.point_before_close(), points[6]);
        assert_eq!(iter.next().unwrap().points, &points[6..7]);
        assert_eq!(iter, path.end());
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn reserve_is_absolute_not_additional() {
        let mut path = RawPath::default();
        path.reserve(64, 64);
        for index in 0..40 {
            path.move_to(index as f32, 0.0);
        }
        let storage = buffers(&path);
        path.reserve(64, 64);
        assert_eq!(buffers(&path), storage);
        path.add_rect(Aabb::new(0.0, 0.0, 2.0, 3.0), PathDirection::Clockwise);
        assert_eq!(buffers(&path), storage);
    }

    #[test]
    fn in_place_mapping_keeps_fused_stages_and_storage() {
        for matrix in [
            Mat2D::new(1.234567, 0.0, 0.0, -2.345678, 3.456789, 4.567891),
            Mat2D::new(1.234567, 2.345678, -3.456789, 4.567891, 5.678912, -6.789123),
        ] {
            let mut path = RawPath::default();
            path.move_to(-0.0, 16777216.0);
            path.line_to(3.1415927, -0.0000001);
            path.close();
            let expected = path.transform(matrix);
            let storage = buffers(&path);
            let metadata = (path.last_move_index, path.contour_is_open);
            path.transform_in_place(matrix);
            assert_eq!(path.verbs, expected.verbs);
            for (actual, expected) in path.points.iter().zip(&expected.points) {
                assert_eq!(
                    (actual.x.to_bits(), actual.y.to_bits()),
                    (expected.x.to_bits(), expected.y.to_bits())
                );
            }
            assert_eq!(buffers(&path), storage);
            assert_eq!((path.last_move_index, path.contour_is_open), metadata);
        }
    }

    #[test]
    fn reverse_append_mapping_and_cursor_match_source() {
        let mut source = RawPath::default();
        source.move_to(1.0, 2.0);
        source.cubic_to(3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
        source.close();
        let matrix = Mat2D::new(1.234567, 2.345678, -3.456789, 4.567891, 5.678912, -6.789123);
        let expected = source.transform(matrix);
        let mut path = RawPath::default();
        path.move_to(100.0, 200.0);
        path.reserve(32, 32);
        let storage = buffers(&path);
        let cursor = path.add_path_backwards(&source, Some(&matrix));
        assert_eq!((cursor.verb, cursor.point), (1, 1));
        assert_eq!(
            path.verbs,
            [
                PathVerb::Move,
                PathVerb::Move,
                PathVerb::Cubic,
                PathVerb::Close
            ]
        );
        for (actual, expected) in path.points[1..].iter().zip(expected.points.iter().rev()) {
            assert_eq!(
                (actual.x.to_bits(), actual.y.to_bits()),
                (expected.x.to_bits(), expected.y.to_bits())
            );
        }
        assert_eq!(buffers(&path), storage);
    }

    #[test]
    fn source_wang_identity_stage_preserves_nonfinite_behavior() {
        // Upstream VectorXform is evaluated even for the identity. Infinity
        // in one coordinate poisons the opposite 0*infinity lane, so this
        // cannot be replaced by Vec2D::length_squared on the raw differences.
        let points = [
            point(0.0, 0.0),
            point(f32::INFINITY, 1.0),
            point(2.0, 2.0),
            point(3.0, 3.0),
        ];
        assert!(
            super::super::wangs_formula::cubic(
                &points,
                1.0 / RawPath::COARSE_AREA_TOLERANCE,
                super::super::wangs_formula::VectorXform::default()
            )
            .is_nan()
        );
        // Exercise the owner, not just its dependency: ceil(NaN) fails the
        // upstream subdivision guard, leaving only the finite endpoint edge.
        let mut path = RawPath::default();
        path.move_to_point(points[0]);
        path.cubic_to_points(points[1], points[2], points[3]);
        assert_eq!(path.compute_coarse_area().to_bits(), 0.0f32.to_bits());
    }

    fn point(x: f32, y: f32) -> Vec2D {
        Vec2D { x, y }
    }

    fn buffers(path: &RawPath) -> (*const PathVerb, usize, *const Vec2D, usize) {
        (
            path.verbs.as_ptr(),
            path.verbs.capacity(),
            path.points.as_ptr(),
            path.points.capacity(),
        )
    }

    // Pinned raw_path_test.cpp: prune-empty-segments, implicit move cases.
    #[test]
    fn prune_empty_and_implicit_moves() {
        let mut empty = RawPath::default();
        empty.prune_empty_segments();
        assert!(empty.verbs.is_empty());
        assert!(empty.points.is_empty());
        for verb in [PathVerb::Line, PathVerb::Quad, PathVerb::Cubic] {
            let mut path = RawPath::default();
            let zero = Vec2D::default();
            match verb {
                PathVerb::Line => path.line_to_point(zero),
                PathVerb::Quad => path.quad_to_points(zero, zero),
                PathVerb::Cubic => path.cubic_to_points(zero, zero, zero),
                _ => unreachable!(),
            }
            let storage = buffers(&path);
            path.prune_empty_segments();
            assert_eq!(path.verbs, [PathVerb::Move]);
            assert_eq!(path.points, [zero]);
            assert_eq!(buffers(&path), storage);
        }
    }

    // Same mixed sequence and expected kept segments as upstream's test.
    #[test]
    fn prune_upstream_mixed_segments() {
        let mut path = RawPath::default();
        path.move_to_point(point(1.0, 2.0));
        path.line_to_point(point(3.0, 4.0));
        path.line_to_point(point(3.0, 4.0));
        for [cx, cy, x, y] in [
            [5.0, 6.0, 7.0, 8.0],
            [7.0, 8.0, 7.0, 8.0],
            [7.0, 8.0, 7.0, 9.0],
            [7.0, 9.0, 7.0, 9.0],
            [7.0, 9.0, 7.0, 8.0],
            [7.0, 8.0, 7.0, 8.0],
        ] {
            path.quad_to_points(point(cx, cy), point(x, y));
        }
        for [ax, ay, bx, by, x, y] in [
            [9.0, 10.0, 11.0, 12.0, 13.0, 14.0],
            [13.0, 14.0, 13.0, 14.0, 13.0, 14.0],
            [13.0, 14.0, 13.0, 14.0, 13.0, 15.0],
            [13.0, 15.0, 13.0, 15.0, 13.0, 15.0],
            [13.0, 16.0, 13.0, 15.0, 13.0, 15.0],
            [13.0, 15.0, 13.0, 15.0, 13.0, 15.0],
            [13.0, 15.0, 13.0, 16.0, 13.0, 15.0],
            [13.0, 15.0, 13.0, 15.0, 13.0, 15.0],
            [13.0, 15.0, 13.0, 15.0, 13.0, 16.0],
        ] {
            path.cubic_to_points(point(ax, ay), point(bx, by), point(x, y));
        }
        path.close();
        let storage = buffers(&path);
        path.prune_empty_segments();
        use PathVerb::{Close, Cubic, Line, Move, Quad};
        assert_eq!(
            path.verbs,
            [
                Move, Line, Quad, Quad, Quad, Cubic, Cubic, Cubic, Cubic, Cubic, Close
            ]
        );
        let expected = [
            (1.0, 2.0),
            (3.0, 4.0),
            (5.0, 6.0),
            (7.0, 8.0),
            (7.0, 8.0),
            (7.0, 9.0),
            (7.0, 9.0),
            (7.0, 8.0),
            (9.0, 10.0),
            (11.0, 12.0),
            (13.0, 14.0),
            (13.0, 14.0),
            (13.0, 14.0),
            (13.0, 15.0),
            (13.0, 16.0),
            (13.0, 15.0),
            (13.0, 15.0),
            (13.0, 15.0),
            (13.0, 16.0),
            (13.0, 15.0),
            (13.0, 15.0),
            (13.0, 15.0),
            (13.0, 16.0),
        ]
        .map(|(x, y)| point(x, y));
        assert_eq!(path.points, expected);
        assert_eq!(buffers(&path), storage);
    }

    #[test]
    fn prune_upstream_suffix_and_end_cursor() {
        let mut path = RawPath::default();
        path.move_to_point(point(1.0, 2.0));
        path.line_to_point(point(1.0, 2.0));
        path.line_to_point(point(3.0, 4.0));
        let mut added = RawPath::default();
        added.move_to_point(point(5.0, 6.0));
        added.quad_to_points(point(7.0, 8.0), point(9.0, 10.0));
        added.close();
        added.move_to_point(point(11.0, 12.0));
        added.cubic_to_points(point(13.0, 14.0), point(15.0, 16.0), point(17.0, 18.0));
        let matrix = Mat2D::new(0.0, 0.0, 0.0, 0.0, 19.0, 20.0);
        let cursor = path.add_path(&added, Some(&matrix));
        let original = path.clone();
        let storage = buffers(&path);
        path.prune_empty_segments_from(RawPathCursor {
            verb: path.verbs.len(),
            point: path.points.len(),
        });
        assert_eq!(path, original);
        assert_eq!(buffers(&path), storage);
        path.prune_empty_segments_from(cursor);
        use PathVerb::{Close, Line, Move};
        assert_eq!(path.verbs, [Move, Line, Line, Move, Close, Move]);
        assert_eq!(
            path.points,
            [
                point(1.0, 2.0),
                point(1.0, 2.0),
                point(3.0, 4.0),
                point(19.0, 20.0),
                point(19.0, 20.0)
            ]
        );
        path.prune_empty_segments();
        assert_eq!(path.verbs, [Move, Line, Move, Close, Move]);
        assert_eq!(
            path.points,
            [
                point(1.0, 2.0),
                point(3.0, 4.0),
                point(19.0, 20.0),
                point(19.0, 20.0)
            ]
        );
        assert_eq!(buffers(&path), storage);
    }

    #[test]
    fn prune_overlapping_copy_preserves_float_bits_and_storage() {
        let mut path = RawPath::default();
        path.reserve(32, 64);
        path.move_to_point(point(-0.0, 0.0));
        path.line_to_point(point(0.0, -0.0)); // Numerically empty, despite different bits.
        let controls = [
            point(f32::from_bits(0x7fc0_1234), -0.0),
            point(f32::INFINITY, f32::NEG_INFINITY),
            point(2.0, -0.0),
        ];
        path.cubic_to_points(controls[0], controls[1], controls[2]);
        path.line_to_point(controls[2]);
        path.close();
        path.move_to_point(controls[0]);
        path.line_to_point(controls[0]); // NaN != itself: this segment must survive.
        let storage = buffers(&path);
        let metadata = (path.last_move_index, path.contour_is_open);
        path.prune_empty_segments();
        use PathVerb::{Close, Cubic, Line, Move};
        assert_eq!(path.verbs, [Move, Cubic, Close, Move, Line]);
        let expected = [
            point(-0.0, 0.0),
            controls[0],
            controls[1],
            controls[2],
            controls[0],
            controls[0],
        ];
        assert_eq!(path.points.len(), expected.len());
        for (actual, expected) in path.points.iter().zip(expected) {
            assert_eq!(
                (actual.x.to_bits(), actual.y.to_bits()),
                (expected.x.to_bits(), expected.y.to_bits())
            );
        }
        assert_eq!(buffers(&path), storage);
        assert_eq!((path.last_move_index, path.contour_is_open), metadata);
    }
}
