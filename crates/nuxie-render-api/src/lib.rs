// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/include/rive/renderer.hpp
// /Users/levi/dev/oss/rive-runtime/include/rive/factory.hpp
// /Users/levi/dev/rive-rust/tools/golden-runner/recording_renderer.cpp
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Write;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

mod serializing;
pub use nuxie_audio::{AudioDecodeError, AudioSource};
pub use serializing::{SerializingFactory, SerializingRenderer};

pub type ColorInt = u32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

impl Vec2D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Axis-aligned bounds in the same coordinate space as the queried geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Aabb {
    pub const fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn width(self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(self) -> f32 {
        self.max_y - self.min_y
    }

    /// Inclusive containment, including points on the maximum edges.
    pub fn contains(self, point: Vec2D) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
    }
}

/// Pinned C++ `rive::Fit` values from `include/rive/layout.hpp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Fit {
    Fill = 0,
    Contain = 1,
    Cover = 2,
    FitWidth = 3,
    FitHeight = 4,
    None = 5,
    ScaleDown = 6,
    Layout = 7,
}

impl Fit {
    pub const fn from_u64(value: u64) -> Option<Self> {
        match value {
            0 => Some(Self::Fill),
            1 => Some(Self::Contain),
            2 => Some(Self::Cover),
            3 => Some(Self::FitWidth),
            4 => Some(Self::FitHeight),
            5 => Some(Self::None),
            6 => Some(Self::ScaleDown),
            7 => Some(Self::Layout),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2D(pub [f32; 6]);

impl Mat2D {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub fn transform_point(self, point: Vec2D) -> Vec2D {
        let [xx, yx, xy, yy, tx, ty] = self.0;
        Vec2D {
            x: xx * point.x + xy * point.y + tx,
            y: yx * point.x + yy * point.y + ty,
        }
    }
}

fn multiply_mat2d(lhs: Mat2D, rhs: Mat2D) -> Mat2D {
    let a = lhs.0;
    let b = rhs.0;
    Mat2D([
        a[0].mul_add(b[0], a[2] * b[1]),
        a[1].mul_add(b[0], a[3] * b[1]),
        a[0].mul_add(b[2], a[2] * b[3]),
        a[1].mul_add(b[2], a[3] * b[3]),
        a[0].mul_add(b[4], a[2] * b[5]) + a[4],
        a[1].mul_add(b[4], a[3] * b[5]) + a[5],
    ])
}

/// Align content into a destination frame with pinned Rive fit semantics.
///
/// Direct port of `src/renderer.cpp:7-70`. `alignment` is the two-float
/// `rive::Alignment` value; `Vec2D` is its backend-neutral seam equivalent.
pub fn compute_alignment(
    fit: Fit,
    alignment: Vec2D,
    frame: Aabb,
    content: Aabb,
    scale_factor: f32,
) -> Mat2D {
    compute_alignment_from_origin_size(
        fit,
        alignment,
        Vec2D::new(frame.min_x, frame.min_y),
        Vec2D::new(frame.width(), frame.height()),
        Vec2D::new(content.min_x, content.min_y),
        Vec2D::new(content.width(), content.height()),
        scale_factor,
    )
}

/// `compute_alignment` for callers that already retain bounds as origin and
/// size. This avoids a lossy origin-plus-size-to-maximum round trip.
pub fn compute_alignment_from_origin_size(
    fit: Fit,
    alignment: Vec2D,
    frame_origin: Vec2D,
    frame_size: Vec2D,
    content_origin: Vec2D,
    content_size: Vec2D,
    scale_factor: f32,
) -> Mat2D {
    let x = -content_origin.x - content_size.x * 0.5 - alignment.x * content_size.x * 0.5;
    let y = -content_origin.y - content_size.y * 0.5 - alignment.y * content_size.y * 0.5;

    let (scale_x, scale_y) = match fit {
        Fit::Fill => (frame_size.x / content_size.x, frame_size.y / content_size.y),
        Fit::Contain => {
            let scale = (frame_size.x / content_size.x).min(frame_size.y / content_size.y);
            (scale, scale)
        }
        Fit::Cover => {
            let scale = (frame_size.x / content_size.x).max(frame_size.y / content_size.y);
            (scale, scale)
        }
        Fit::FitHeight => {
            let scale = frame_size.y / content_size.y;
            (scale, scale)
        }
        Fit::FitWidth => {
            let scale = frame_size.x / content_size.x;
            (scale, scale)
        }
        Fit::Layout => (scale_factor, scale_factor),
        Fit::None => (1.0, 1.0),
        Fit::ScaleDown => {
            let scale = (frame_size.x / content_size.x).min(frame_size.y / content_size.y);
            let scale = if scale < 1.0 { scale } else { 1.0 };
            (scale, scale)
        }
    };

    let translation = Mat2D([
        1.0,
        0.0,
        0.0,
        1.0,
        frame_origin.x + frame_size.x * 0.5 + alignment.x * frame_size.x * 0.5,
        frame_origin.y + frame_size.y * 0.5 + alignment.y * frame_size.y * 0.5,
    ]);
    multiply_mat2d(
        multiply_mat2d(translation, Mat2D([scale_x, 0.0, 0.0, scale_y, 0.0, 0.0])),
        Mat2D([1.0, 0.0, 0.0, 1.0, x, y]),
    )
}

/// Rive's intentionally narrow whitespace classification.
///
/// Direct port of pinned `isWhiteSpace` (`src/renderer.cpp:142-147`). In
/// particular, U+200B is whitespace while most Unicode space characters are
/// not part of this renderer contract.
pub fn is_white_space(character: char) -> bool {
    let character = character as u32;
    character <= u32::from(b' ') || matches!(character, 0x2028 | 0x200b)
}

/// Renderer annotations attached to one shaped glyph run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphRunAnnotations {
    pub breaks: Vec<u32>,
    pub joiners: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphRunAnnotationError;

impl std::fmt::Display for GlyphRunAnnotationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("shaped glyph run contains an invalid text index")
    }
}

impl std::error::Error for GlyphRunAnnotationError {}

/// Attach pinned line-break and word-joiner annotations to shaped runs.
///
/// Direct port of the post-shape loop in `Font::shapeText`
/// (`src/renderer.cpp:149-229`). Each text index addresses `text`; run order
/// and glyph order remain exactly as supplied by the shaping adapter.
pub fn annotate_glyph_runs(
    text: &[char],
    run_text_indices: &[&[u32]],
) -> Result<Vec<GlyphRunAnnotations>, GlyphRunAnnotationError> {
    let mut want_white_space = false;
    let mut annotations = Vec::with_capacity(run_text_indices.len());

    for text_indices in run_text_indices {
        let mut breaks = Vec::with_capacity(text.len() / 4);
        let mut joiners = Vec::with_capacity(text.len() / 4);
        for (glyph_index, offset) in text_indices.iter().copied().enumerate() {
            let character = usize::try_from(offset)
                .ok()
                .and_then(|offset| text.get(offset))
                .copied()
                .ok_or(GlyphRunAnnotationError)?;
            let glyph_index = u32::try_from(glyph_index).map_err(|_| GlyphRunAnnotationError)?;
            if matches!(character, '\n' | '\u{2028}') {
                breaks.push(glyph_index);
                breaks.push(glyph_index);
            }
            if character == '\u{2060}' {
                joiners.push(offset);
            }
            if want_white_space == is_white_space(character) {
                breaks.push(glyph_index);
                want_white_space = !want_white_space;
            }
        }
        annotations.push(GlyphRunAnnotations { breaks, joiners });
    }

    if let Some((annotation, text_indices)) = annotations.last_mut().zip(run_text_indices.last()) {
        let glyph_count = u32::try_from(text_indices.len()).map_err(|_| GlyphRunAnnotationError)?;
        if want_white_space {
            annotation.breaks.push(glyph_count);
        } else {
            annotation
                .breaks
                .push(annotation.breaks.last().copied().unwrap_or(0));
            annotation.breaks.push(glyph_count);
        }
    }

    Ok(annotations)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FillRule {
    NonZero = 0,
    EvenOdd = 1,
    Clockwise = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PathVerb {
    Move = 0,
    Line = 1,
    Quad = 2,
    Cubic = 4,
    Close = 5,
}

static NEXT_RAW_PATH_MUTATION_ID: AtomicU64 = AtomicU64::new(1);

fn next_raw_path_mutation_id() -> u64 {
    NEXT_RAW_PATH_MUTATION_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone)]
pub struct RawPath {
    verbs: Vec<PathVerb>,
    points: Vec<Vec2D>,
    mutation_id: u64,
}

/// Exclusive builder for replacing a [`RawPath`] as one logical mutation.
///
/// The builder only exposes geometry mutators, so a partially rebuilt path
/// cannot be observed or cloned while its mutation identity remains stable.
pub struct RawPathBuilder<'a> {
    raw_path: &'a mut RawPath,
}

impl RawPathBuilder<'_> {
    #[inline]
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.raw_path.verbs.push(PathVerb::Move);
        self.raw_path.points.push(Vec2D::new(x, y));
    }

    #[inline]
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.raw_path.inject_implicit_move_if_needed();
        self.raw_path.verbs.push(PathVerb::Line);
        self.raw_path.points.push(Vec2D::new(x, y));
    }

    #[inline]
    pub fn quad_to(&mut self, ox: f32, oy: f32, x: f32, y: f32) {
        self.raw_path.inject_implicit_move_if_needed();
        self.raw_path.verbs.push(PathVerb::Quad);
        self.raw_path.points.push(Vec2D::new(ox, oy));
        self.raw_path.points.push(Vec2D::new(x, y));
    }

    #[inline]
    pub fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.raw_path.inject_implicit_move_if_needed();
        self.raw_path.verbs.push(PathVerb::Cubic);
        self.raw_path.points.push(Vec2D::new(ox, oy));
        self.raw_path.points.push(Vec2D::new(ix, iy));
        self.raw_path.points.push(Vec2D::new(x, y));
    }

    #[inline]
    pub fn close(&mut self) {
        if !self.raw_path.verbs.is_empty() && self.raw_path.verbs.last() != Some(&PathVerb::Close) {
            self.raw_path.verbs.push(PathVerb::Close);
        }
    }
}

impl PartialEq for RawPath {
    fn eq(&self, other: &Self) -> bool {
        self.verbs == other.verbs && self.points == other.points
    }
}

impl RawPath {
    pub fn new() -> Self {
        Self {
            verbs: Vec::new(),
            points: Vec::new(),
            mutation_id: next_raw_path_mutation_id(),
        }
    }

    /// Identifies the current geometry snapshot, matching C++ `RiveRenderPath` mutation IDs.
    pub fn mutation_id(&self) -> u64 {
        self.mutation_id
    }

    /// Assigns a new identity when this geometry becomes a distinct render-path object.
    pub fn renew_mutation_id(&mut self) {
        self.mark_mutated();
    }

    pub fn verbs(&self) -> &[PathVerb] {
        &self.verbs
    }

    pub fn points(&self) -> &[Vec2D] {
        &self.points
    }

    /// Coarse control-point bounds, matching C++ `RawPath::bounds()`.
    pub fn bounds(&self) -> Option<Aabb> {
        let first = *self.points.first()?;
        Some(self.points.iter().copied().fold(
            Aabb::new(first.x, first.y, first.x, first.y),
            |mut bounds, point| {
                bounds.min_x = bounds.min_x.min(point.x);
                bounds.min_y = bounds.min_y.min(point.y);
                bounds.max_x = bounds.max_x.max(point.x);
                bounds.max_y = bounds.max_y.max(point.y);
                bounds
            },
        ))
    }

    /// Exact Bézier extrema bounds, matching C++ `RawPath::preciseBounds()`.
    pub fn precise_bounds(&self) -> Option<Aabb> {
        let mut point_index = 0;
        let mut current = Vec2D::new(0.0, 0.0);
        let mut contour_start = current;
        let mut bounds = None;
        for verb in &self.verbs {
            match verb {
                PathVerb::Move => {
                    current = self.points[point_index];
                    point_index += 1;
                    contour_start = current;
                    include_raw_path_point(&mut bounds, current);
                }
                PathVerb::Line => {
                    current = self.points[point_index];
                    point_index += 1;
                    include_raw_path_point(&mut bounds, current);
                }
                PathVerb::Quad => {
                    let control = self.points[point_index];
                    let end = self.points[point_index + 1];
                    point_index += 2;
                    include_raw_path_point(&mut bounds, current);
                    include_raw_path_point(&mut bounds, end);
                    for t in raw_path_quad_extrema(current.x, control.x, end.x)
                        .into_iter()
                        .chain(raw_path_quad_extrema(current.y, control.y, end.y))
                    {
                        include_raw_path_point(
                            &mut bounds,
                            Vec2D::new(
                                raw_path_quad_value(current.x, control.x, end.x, t),
                                raw_path_quad_value(current.y, control.y, end.y, t),
                            ),
                        );
                    }
                    current = end;
                }
                PathVerb::Cubic => {
                    let outer = self.points[point_index];
                    let inner = self.points[point_index + 1];
                    let end = self.points[point_index + 2];
                    point_index += 3;
                    include_raw_path_point(&mut bounds, current);
                    include_raw_path_point(&mut bounds, end);
                    for t in raw_path_cubic_extrema(current.x, outer.x, inner.x, end.x)
                        .into_iter()
                        .chain(raw_path_cubic_extrema(current.y, outer.y, inner.y, end.y))
                    {
                        include_raw_path_point(
                            &mut bounds,
                            Vec2D::new(
                                raw_path_cubic_value(current.x, outer.x, inner.x, end.x, t),
                                raw_path_cubic_value(current.y, outer.y, inner.y, end.y, t),
                            ),
                        );
                    }
                    current = end;
                }
                PathVerb::Close => current = contour_start,
            }
        }
        bounds
    }

    /// Replaces this path's geometry as one logical mutation.
    ///
    /// C++ `RawPath` does not assign mutation identities while commands are
    /// appended. Its owning `RiveRenderPath` is merely marked dirty and lazily
    /// receives one new identity when a renderer next consumes it. This scoped
    /// builder provides the equivalent contract: it renews the snapshot
    /// identity once, preserves the path allocations, and prevents observers
    /// from seeing the individual command appends.
    #[inline]
    pub fn rebuild(
        &mut self,
        verbs: usize,
        points: usize,
        build: impl FnOnce(&mut RawPathBuilder<'_>),
    ) {
        self.mark_mutated();
        self.verbs.clear();
        self.points.clear();
        self.verbs.reserve(verbs);
        self.points.reserve(points);
        build(&mut RawPathBuilder { raw_path: self });
    }

    pub fn rewind(&mut self) {
        self.mark_mutated();
        self.verbs.clear();
        self.points.clear();
    }

    pub fn reserve(&mut self, verbs: usize, points: usize) {
        self.verbs.reserve(verbs);
        self.points.reserve(points);
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.mark_mutated();
        self.verbs.push(PathVerb::Move);
        self.points.push(Vec2D::new(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.inject_implicit_move_if_needed();
        self.mark_mutated();
        self.verbs.push(PathVerb::Line);
        self.points.push(Vec2D::new(x, y));
    }

    pub fn quad_to(&mut self, ox: f32, oy: f32, x: f32, y: f32) {
        self.inject_implicit_move_if_needed();
        self.mark_mutated();
        self.verbs.push(PathVerb::Quad);
        self.points.push(Vec2D::new(ox, oy));
        self.points.push(Vec2D::new(x, y));
    }

    pub fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.inject_implicit_move_if_needed();
        self.mark_mutated();
        self.verbs.push(PathVerb::Cubic);
        self.points.push(Vec2D::new(ox, oy));
        self.points.push(Vec2D::new(ix, iy));
        self.points.push(Vec2D::new(x, y));
    }

    pub fn close(&mut self) {
        if !self.verbs.is_empty() && self.verbs.last() != Some(&PathVerb::Close) {
            self.mark_mutated();
            self.verbs.push(PathVerb::Close);
        }
    }

    /// Append a clockwise rectangle, matching pinned C++ `RawPath::addRect`.
    pub fn add_rect(&mut self, bounds: Aabb) {
        self.reserve(6, 5);
        self.move_to(bounds.min_x, bounds.min_y);
        self.line_to(bounds.max_x, bounds.min_y);
        self.line_to(bounds.max_x, bounds.max_y);
        self.line_to(bounds.min_x, bounds.max_y);
        self.close();
    }

    pub fn add_path(&mut self, path: &RawPath, transform: Mat2D) {
        if path.verbs.is_empty() {
            return;
        }
        self.mark_mutated();
        self.verbs.extend_from_slice(&path.verbs);
        if transform == Mat2D::IDENTITY {
            // C++ passes a null matrix when RenderPath::addRawPath appends an
            // untransformed path, which copies the points verbatim. Besides
            // avoiding needless affine work, this preserves signed zero.
            self.points.extend_from_slice(&path.points);
        } else {
            self.points.extend(
                path.points
                    .iter()
                    .copied()
                    .map(|point| map_raw_path_point(transform, point)),
            );
        }
    }

    pub fn add_path_backwards(&mut self, path: &RawPath, transform: Mat2D) {
        if path.verbs.is_empty() {
            return;
        }
        self.mark_mutated();

        let initial_verb_count = self.verbs.len();
        let initial_point_count = self.points.len();
        self.points.reserve(path.points.len());
        if transform == Mat2D::IDENTITY {
            self.points.extend(path.points.iter().rev().copied());
        } else {
            self.points.extend(
                path.points
                    .iter()
                    .rev()
                    .copied()
                    .map(|point| map_raw_path_point(transform, point)),
            );
        }

        // Reverse the verbs while moving each close from the end of its
        // original contour to the end of the reversed contour.
        self.verbs.reserve(path.verbs.len());
        self.verbs.push(PathVerb::Move);
        let mut closed = false;
        for (index, verb) in path.verbs.iter().enumerate().rev() {
            if *verb == PathVerb::Close {
                debug_assert!(!closed, "a contour may contain only one close verb");
                closed = true;
                continue;
            }

            if *verb == PathVerb::Move && closed {
                self.verbs.push(PathVerb::Close);
                closed = false;
            }

            if index == 0 {
                debug_assert_eq!(*verb, PathVerb::Move);
                break;
            }

            self.verbs.push(*verb);
        }
        debug_assert!(!closed, "every close verb must have a preceding move verb");

        self.prune_empty_segments_from(initial_verb_count, initial_point_count);
    }

    fn prune_empty_segments_from(&mut self, verb_start: usize, point_start: usize) {
        let mut source_point = point_start;
        let mut destination_verb = verb_start;
        let mut destination_point = point_start;

        for source_verb in verb_start..self.verbs.len() {
            let verb = self.verbs[source_verb];
            let point_count = match verb {
                PathVerb::Move | PathVerb::Line => 1,
                PathVerb::Quad => 2,
                PathVerb::Cubic => 3,
                PathVerb::Close => 0,
            };
            let has_geometry = match verb {
                PathVerb::Move | PathVerb::Close => true,
                PathVerb::Line => self.points[source_point] != self.points[source_point - 1],
                PathVerb::Quad => {
                    self.points[source_point + 1] != self.points[source_point]
                        || self.points[source_point] != self.points[source_point - 1]
                }
                PathVerb::Cubic => {
                    self.points[source_point + 2] != self.points[source_point + 1]
                        || self.points[source_point + 1] != self.points[source_point]
                        || self.points[source_point] != self.points[source_point - 1]
                }
            };

            if has_geometry {
                if source_verb != destination_verb {
                    self.verbs[destination_verb] = verb;
                    for point in 0..point_count {
                        self.points[destination_point + point] = self.points[source_point + point];
                    }
                }
                destination_verb += 1;
                destination_point += point_count;
            }
            source_point += point_count;
        }

        self.verbs.truncate(destination_verb);
        self.points.truncate(destination_point);
    }

    #[inline]
    fn inject_implicit_move_if_needed(&mut self) {
        if !self.verbs.is_empty() && self.verbs.last() != Some(&PathVerb::Close) {
            return;
        }

        let mut point_index = 0;
        let mut last_move = Vec2D::new(0.0, 0.0);
        for verb in &self.verbs {
            match verb {
                PathVerb::Move => {
                    last_move = self.points[point_index];
                    point_index += 1;
                }
                PathVerb::Line => point_index += 1,
                PathVerb::Quad => point_index += 2,
                PathVerb::Cubic => point_index += 3,
                PathVerb::Close => {}
            }
        }
        self.verbs.push(PathVerb::Move);
        self.points.push(last_move);
    }

    fn mark_mutated(&mut self) {
        self.mutation_id = next_raw_path_mutation_id();
    }
}

fn include_raw_path_point(bounds: &mut Option<Aabb>, point: Vec2D) {
    match bounds {
        Some(bounds) => {
            bounds.min_x = bounds.min_x.min(point.x);
            bounds.min_y = bounds.min_y.min(point.y);
            bounds.max_x = bounds.max_x.max(point.x);
            bounds.max_y = bounds.max_y.max(point.y);
        }
        None => *bounds = Some(Aabb::new(point.x, point.y, point.x, point.y)),
    }
}

fn raw_path_quad_extrema(start: f32, control: f32, end: f32) -> Option<f32> {
    let denominator = start - 2.0 * control + end;
    (denominator != 0.0)
        .then_some((start - control) / denominator)
        .filter(|t| *t > 0.0 && *t < 1.0)
}

fn raw_path_quad_value(start: f32, control: f32, end: f32, t: f32) -> f32 {
    let one_minus_t = 1.0 - t;
    one_minus_t * one_minus_t * start + 2.0 * one_minus_t * t * control + t * t * end
}

fn raw_path_cubic_extrema(start: f32, outer: f32, inner: f32, end: f32) -> Vec<f32> {
    let a = -start + 3.0 * outer - 3.0 * inner + end;
    let b = 2.0 * (start - 2.0 * outer + inner);
    let c = outer - start;
    if a.abs() <= f32::EPSILON {
        return if b.abs() <= f32::EPSILON {
            Vec::new()
        } else {
            let t = -c / b;
            (t > 0.0 && t < 1.0).then_some(t).into_iter().collect()
        };
    }
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return Vec::new();
    }
    let root = discriminant.sqrt();
    [(-b + root) / (2.0 * a), (-b - root) / (2.0 * a)]
        .into_iter()
        .filter(|t| *t > 0.0 && *t < 1.0)
        .collect()
}

fn raw_path_cubic_value(start: f32, outer: f32, inner: f32, end: f32, t: f32) -> f32 {
    let one_minus_t = 1.0 - t;
    one_minus_t * one_minus_t * one_minus_t * start
        + 3.0 * one_minus_t * one_minus_t * t * outer
        + 3.0 * one_minus_t * t * t * inner
        + t * t * t * end
}

fn map_raw_path_point(transform: Mat2D, point: Vec2D) -> Vec2D {
    let [xx, yx, xy, yy, tx, ty] = transform.0;
    // C++ RawPath::addPath maps in batches through Mat2D::mapPoints. Its SIMD
    // affine branch groups skew with translation before adding scale and uses
    // fused multiply-adds on supported targets.
    Vec2D {
        x: xx.mul_add(point.x, xy.mul_add(point.y, tx)),
        y: yy.mul_add(point.y, yx.mul_add(point.x, ty)),
    }
}

impl Default for RawPath {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlendMode {
    #[default]
    SrcOver = 3,
    Screen = 14,
    Overlay = 15,
    Darken = 16,
    Lighten = 17,
    ColorDodge = 18,
    ColorBurn = 19,
    HardLight = 20,
    SoftLight = 21,
    Difference = 22,
    Exclusion = 23,
    Multiply = 24,
    Hue = 25,
    Saturation = 26,
    Color = 27,
    Luminosity = 28,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum StrokeJoin {
    #[default]
    Miter = 0,
    Round = 1,
    Bevel = 2,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum StrokeCap {
    #[default]
    Butt = 0,
    Round = 1,
    Square = 2,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RenderPaintStyle {
    Stroke,
    #[default]
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImageFilter {
    Bilinear = 0,
    Nearest = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImageWrap {
    Clamp = 0,
    Repeat = 1,
    Mirror = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSampler {
    pub wrap_x: ImageWrap,
    pub wrap_y: ImageWrap,
    pub filter: ImageFilter,
}

impl ImageSampler {
    pub const LINEAR_CLAMP: Self = Self {
        wrap_x: ImageWrap::Clamp,
        wrap_y: ImageWrap::Clamp,
        filter: ImageFilter::Bilinear,
    };

    pub fn as_key(self) -> u8 {
        self.wrap_x as u8 + (self.wrap_y as u8 * 3) + (self.filter as u8 * 3 * 3)
    }
}

impl Default for ImageSampler {
    fn default() -> Self {
        Self::LINEAR_CLAMP
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RenderBufferType {
    Index = 0,
    Vertex = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RenderBufferFlags {
    None = 0,
    MappedOnceAtInitialization = 1,
}

pub trait RenderBuffer: Any {
    fn as_any(&self) -> &dyn Any;
    fn buffer_type(&self) -> RenderBufferType;
    fn flags(&self) -> RenderBufferFlags;
    fn size_in_bytes(&self) -> usize;
    fn map_mut(&mut self) -> &mut [u8];
    fn unmap(&mut self);
}

pub trait RenderShader: Any {
    fn as_any(&self) -> &dyn Any;
}

/// One opaque, lookup-owned authored GPU-canvas shader module.
///
/// Each `context:shader` occurrence owns one of these handles. Backends retain
/// their device/domain identity and physical shader module behind this seam.
pub trait RenderGpuCanvasShader: Any {
    fn as_any(&self) -> &dyn Any;
}

pub trait RenderImage: Any {
    fn as_any(&self) -> &dyn Any;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn uv_transform(&self) -> Mat2D {
        Mat2D::IDENTITY
    }
}

/// One entry-point stage in a Rive whole-module shader source container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GpuCanvasShaderStage {
    Vertex = 0,
    Fragment = 1,
    Compute = 2,
}

/// One authored entry record from a Rive whole-module shader source container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasShaderEntry {
    pub stage: GpuCanvasShaderStage,
    pub logical_entry_point: String,
    pub physical_entry_point: String,
}

/// One resource kind from Rive's frozen `BindingMap` v2 wire schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GpuCanvasShaderResourceKind {
    UniformBuffer = 0,
    StorageBufferReadOnly = 1,
    StorageBufferReadWrite = 2,
    SampledTexture = 3,
    StorageTexture = 4,
    Sampler = 5,
    ComparisonSampler = 6,
}

/// Texture view dimension reflected into a Rive `BindingMap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GpuCanvasShaderTextureViewDimension {
    Undefined = 0,
    D1 = 1,
    D2 = 2,
    D2Array = 3,
    Cube = 4,
    CubeArray = 5,
    D3 = 6,
}

/// Texture sample type reflected into a Rive `BindingMap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GpuCanvasShaderTextureSampleType {
    Undefined = 0,
    Float = 1,
    UnfilterableFloat = 2,
    Depth = 3,
    Sint = 4,
    Uint = 5,
}

/// One decoded row from the mandatory Rive WebGPU `BindingMap` sidecar.
///
/// `backend_slots` is ordered vertex, fragment, compute. `None` preserves
/// Rive's `BindingMap::kAbsent` sentinel without leaking it to callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasShaderBinding {
    pub group: u8,
    pub binding: u8,
    pub kind: GpuCanvasShaderResourceKind,
    pub stage_mask: u8,
    pub backend_space: u8,
    pub backend_slots: [Option<u16>; 3],
    pub texture_view_dimension: GpuCanvasShaderTextureViewDimension,
    pub texture_sample_type: GpuCanvasShaderTextureSampleType,
    pub texture_multisampled: bool,
}

/// The authored WGSL module selected by WebGPU from a Rive `ShaderAsset`.
///
/// Rive target 0 stores one source module shared by every entry. Target 16
/// stores the binding metadata that accompanies that exact module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasShader {
    pub source: String,
    pub entries: Vec<GpuCanvasShaderEntry>,
    pub bindings: Vec<GpuCanvasShaderBinding>,
}

impl GpuCanvasShader {
    pub fn entry(
        &self,
        stage: GpuCanvasShaderStage,
        logical_entry_point: &str,
    ) -> Option<&GpuCanvasShaderEntry> {
        self.entries
            .iter()
            .find(|entry| entry.stage == stage && entry.logical_entry_point == logical_entry_point)
    }
}

/// The exact logical/physical entry pair selected by one authored pipeline
/// stage. Bare shader descriptors select the first declaration of that stage;
/// named descriptors retain both names so the renderer can reject stale or
/// mismatched records before creating a backend object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasShaderEntrySelection {
    pub logical_entry_point: String,
    pub physical_entry_point: String,
}

/// One uniform binding produced by an authored GPU-canvas frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasUniformBuffer {
    pub group: u32,
    pub binding: u32,
    pub bytes: Vec<u8>,
}

/// One vertex attribute in an authored GPU-canvas pipeline layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasVertexAttribute {
    pub shader_location: u32,
    pub offset: u64,
    pub format: String,
}

/// One vertex-buffer layout in an authored GPU-canvas pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasVertexLayout {
    pub stride: u64,
    pub step_mode: String,
    pub attributes: Vec<GpuCanvasVertexAttribute>,
}

/// One vertex buffer bound by an authored GPU-canvas render pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasVertexBuffer {
    pub slot: u32,
    pub bytes: Vec<u8>,
}

/// One index buffer and format selected by an authored render pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasIndexBuffer {
    pub bytes: Vec<u8>,
    pub format: String,
}

/// One uploaded region retained by an authored GPU texture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasTextureUpload {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub mip_level: u32,
    pub array_layer: u32,
    pub bytes_per_row: u32,
    pub rows_per_image: u32,
}

/// One sampled texture view bound by an authored GPU-canvas pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasTextureBinding {
    pub group: u32,
    pub binding: u32,
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
    pub format: String,
    pub texture_type: String,
    pub render_target: bool,
    pub sample_count: u32,
    pub mip_level_count: u32,
    pub view_dimension: String,
    pub base_mip_level: u32,
    pub mip_level_count_in_view: u32,
    pub base_array_layer: u32,
    pub array_layer_count: u32,
    pub uploads: Vec<GpuCanvasTextureUpload>,
}

/// One sampler bound by an authored GPU-canvas pass.
#[derive(Debug, Clone, PartialEq)]
pub struct GpuCanvasSamplerBinding {
    pub group: u32,
    pub binding: u32,
    pub min_filter: String,
    pub mag_filter: String,
    pub mipmap_filter: String,
    pub address_mode_u: String,
    pub address_mode_v: String,
    pub address_mode_w: String,
    pub compare: Option<String>,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub max_anisotropy: u16,
}

/// Blend state for one color target in the backend-neutral pipeline contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasBlendState {
    pub src_color: String,
    pub dst_color: String,
    pub color_op: String,
    pub src_alpha: String,
    pub dst_alpha: String,
    pub alpha_op: String,
}

/// One authored color-target declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasColorTarget {
    pub format: String,
    pub write_mask: String,
    pub blend: Option<GpuCanvasBlendState>,
}

/// One authored stencil face.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasStencilFace {
    pub compare: String,
    pub fail_op: String,
    pub depth_fail_op: String,
    pub pass_op: String,
}

/// Authored depth/stencil state carried to the wgpu pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct GpuCanvasDepthStencilState {
    pub format: String,
    pub depth_compare: String,
    pub depth_write_enabled: bool,
    pub depth_bias: i32,
    pub depth_bias_slope_scale: f32,
    pub depth_bias_clamp: f32,
    pub stencil_front: GpuCanvasStencilFace,
    pub stencil_back: GpuCanvasStencilFace,
    pub stencil_read_mask: u32,
    pub stencil_write_mask: u32,
}

/// Pipeline state that is not encoded in shader modules or vertex layouts.
#[derive(Debug, Clone, PartialEq)]
pub struct GpuCanvasPipelineState {
    pub color_targets: Vec<GpuCanvasColorTarget>,
    pub depth_stencil: Option<GpuCanvasDepthStencilState>,
    pub cull_mode: String,
    pub winding: String,
    pub topology: String,
    pub sample_count: u32,
}

impl Default for GpuCanvasPipelineState {
    fn default() -> Self {
        Self {
            color_targets: vec![GpuCanvasColorTarget {
                format: "rgba8unorm".into(),
                write_mask: "rgba".into(),
                blend: None,
            }],
            depth_stencil: None,
            cull_mode: "none".into(),
            winding: "ccw".into(),
            topology: "triangle-list".into(),
            sample_count: 1,
        }
    }
}

/// Dynamic pass state retained at the draw site.
#[derive(Debug, Clone, PartialEq)]
pub struct GpuCanvasPassState {
    pub viewport: Option<[f32; 4]>,
    pub scissor_rect: Option<[u32; 4]>,
    pub stencil_reference: u32,
    pub blend_color: [f64; 4],
}

impl Default for GpuCanvasPassState {
    fn default() -> Self {
        Self {
            viewport: None,
            scissor_rect: None,
            stencil_reference: 0,
            blend_color: [0.0; 4],
        }
    }
}

/// Indexed draw arguments. When absent, the legacy non-indexed fields on
/// [`GpuCanvasPlan`] describe the draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasIndexedDraw {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

/// Backend-neutral result of executing one imported script's `drawCanvas`.
#[derive(Debug, Clone, PartialEq)]
pub struct GpuCanvasPlan {
    pub vertex_entry: Option<GpuCanvasShaderEntrySelection>,
    pub fragment_entry: Option<GpuCanvasShaderEntrySelection>,
    pub width: u32,
    pub height: u32,
    pub clear_color: [f64; 4],
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
    pub uniform_buffers: Vec<GpuCanvasUniformBuffer>,
    pub vertex_layouts: Vec<GpuCanvasVertexLayout>,
    pub vertex_buffers: Vec<GpuCanvasVertexBuffer>,
    pub index_buffer: Option<GpuCanvasIndexBuffer>,
    pub indexed_draw: Option<GpuCanvasIndexedDraw>,
    pub texture_bindings: Vec<GpuCanvasTextureBinding>,
    pub sampler_bindings: Vec<GpuCanvasSamplerBinding>,
    pub pipeline_state: GpuCanvasPipelineState,
    pub pass_state: GpuCanvasPassState,
}

/// A render factory cannot turn an authored GPU-canvas plan into an image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasError {
    message: String,
}

impl GpuCanvasError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn unsupported() -> Self {
        Self::new("render factory does not support imported GPU-canvas images")
    }
}

impl std::fmt::Display for GpuCanvasError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for GpuCanvasError {}

/// A renderer adapter could not decode encoded image bytes into a render image.
///
/// Image codecs and backend limits are adapter-specific, so this error deliberately
/// does not expose a renderer dialect. Callers can recover by fixing the source bytes,
/// choosing another adapter, or retrying a later frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageDecodeError;

impl std::fmt::Display for ImageDecodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("render factory could not decode image")
    }
}

impl std::error::Error for ImageDecodeError {}

/// Factory-owned encoded font data validated by the runtime's HarfBuzz port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedFont {
    bytes: Arc<[u8]>,
}

impl DecodedFont {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Arc<[u8]> {
        self.bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontDecodeError;

impl std::fmt::Display for FontDecodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("render factory could not decode font")
    }
}

impl std::error::Error for FontDecodeError {}

pub trait RenderPaint: Any {
    fn as_any(&self) -> &dyn Any;
    fn style(&mut self, style: RenderPaintStyle);
    fn color(&mut self, value: ColorInt);
    fn thickness(&mut self, value: f32);
    fn join(&mut self, value: StrokeJoin);
    fn cap(&mut self, value: StrokeCap);
    fn feather(&mut self, value: f32);
    fn blend_mode(&mut self, value: BlendMode);
    fn shader(&mut self, shader: Option<&dyn RenderShader>);
    fn invalidate_stroke(&mut self);
}

pub trait RenderPath: Any {
    fn as_any(&self) -> &dyn Any;
    fn rewind(&mut self);
    fn reserve(&mut self, _verbs: usize, _points: usize) {}
    fn fill_rule(&mut self, value: FillRule);
    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D);
    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D);
    fn add_raw_path(&mut self, path: &RawPath);
    fn move_to(&mut self, x: f32, y: f32);
    fn line_to(&mut self, x: f32, y: f32);
    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32);
    fn close(&mut self);
}

pub trait Renderer {
    fn save(&mut self);
    fn restore(&mut self);
    fn transform(&mut self, transform: Mat2D);

    /// Direct port of pinned `Renderer::translate` (`src/renderer.cpp:72-75`).
    fn translate(&mut self, tx: f32, ty: f32) {
        self.transform(Mat2D([1.0, 0.0, 0.0, 1.0, tx, ty]));
    }

    /// Direct port of pinned `Renderer::scale` (`src/renderer.cpp:77-80`).
    fn scale(&mut self, sx: f32, sy: f32) {
        self.transform(Mat2D([sx, 0.0, 0.0, sy, 0.0, 0.0]));
    }

    /// Direct port of pinned `Renderer::rotate` (`src/renderer.cpp:82-88`).
    fn rotate(&mut self, radians: f32) {
        let sin = radians.sin();
        let cos = radians.cos();
        self.transform(Mat2D([cos, sin, -sin, cos, 0.0, 0.0]));
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint);
    fn clip_path(&mut self, path: &dyn RenderPath);
    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    );
    #[allow(clippy::too_many_arguments)]
    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    );
    fn modulate_opacity(&mut self, opacity: f32);
}

trait PersistentFactoryAccess {
    fn with_factory(&self, callback: &mut dyn FnMut(&mut dyn Factory));
}

struct PersistentFactoryCell<F> {
    factory: RefCell<F>,
}

impl<F: Factory + 'static> PersistentFactoryAccess for PersistentFactoryCell<F> {
    fn with_factory(&self, callback: &mut dyn FnMut(&mut dyn Factory)) {
        callback(&mut *self.factory.borrow_mut());
    }
}

/// An owned, stable-identity renderer factory suitable for retained scripting
/// contexts.
///
/// Clones are lightweight proxies for the same concrete factory. Each factory
/// operation borrows the underlying object only for that operation, so a
/// scripting callback can safely reach the same factory without lifetime
/// erasure or independently manufactured mutable references.
pub struct PersistentFactory<F> {
    access: Rc<PersistentFactoryCell<F>>,
}

impl<F> Clone for PersistentFactory<F> {
    fn clone(&self) -> Self {
        Self {
            access: Rc::clone(&self.access),
        }
    }
}

impl<F> PersistentFactory<F> {
    pub fn new(factory: F) -> Self {
        Self {
            access: Rc::new(PersistentFactoryCell {
                factory: RefCell::new(factory),
            }),
        }
    }

    pub fn borrow(&self) -> Ref<'_, F> {
        self.access.factory.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, F> {
        self.access.factory.borrow_mut()
    }
}

/// Type-erased retained handle shared by one scripting VM and its factory
/// proxies.
#[derive(Clone)]
pub struct PersistentFactoryContext {
    access: Rc<dyn PersistentFactoryAccess>,
    identity: *const (),
}

impl PersistentFactoryContext {
    pub fn identity(&self) -> *const () {
        self.identity
    }

    pub fn with_factory<R>(&self, callback: impl FnOnce(&mut dyn Factory) -> R) -> R {
        let mut callback = Some(callback);
        let mut result = None;
        self.access.with_factory(&mut |factory| {
            let callback = callback
                .take()
                .expect("persistent factory callback executes exactly once");
            result = Some(callback(factory));
        });
        result.expect("persistent factory callback executes exactly once")
    }
}

pub trait Factory {
    /// Return the stable owned context used by scripting VMs, when this
    /// factory is a [`PersistentFactory`] proxy.
    fn persistent_context(&self) -> Option<PersistentFactoryContext> {
        None
    }

    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer>;
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader>;
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader>;
    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath>;

    /// Build the pinned nonzero clockwise rectangle-path helper.
    ///
    /// Direct port of `src/factory.cpp:15-20`; the default keeps the pinned
    /// nonvirtual behavior layered on the adapter-specific raw-path constructor.
    fn make_render_path_from_aabb(&mut self, bounds: Aabb) -> Box<dyn RenderPath> {
        let mut raw_path = RawPath::new();
        raw_path.add_rect(bounds);
        self.make_render_path(raw_path, FillRule::NonZero)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath>;
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint>;
    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError>;

    /// Validate and take ownership of encoded font bytes.
    ///
    /// Direct port of pinned `Factory::decodeFont` (`src/factory.cpp:22-29`).
    /// HarfRust is the project's verified HarfBuzz port, and the owned byte
    /// snapshot supplies the backend views that C++ retains through `HBFont`.
    fn decode_font(&mut self, data: &[u8]) -> Result<DecodedFont, FontDecodeError> {
        harfrust::FontRef::new(data).map_err(|_| FontDecodeError)?;
        Ok(DecodedFont {
            bytes: Arc::from(data),
        })
    }

    /// Validate and take ownership of encoded audio bytes.
    ///
    /// This is the Rust counterpart of C++ `Factory::decodeAudio`: a
    /// non-renderer-specific helper on the Factory seam, not a backend hook.
    fn decode_audio(&mut self, data: &[u8]) -> Result<Arc<AudioSource>, AudioDecodeError> {
        AudioSource::from_encoded(data.to_vec()).map(Arc::new)
    }

    /// Parse, validate, and materialize one fresh authored shader occurrence
    /// in this factory's backend/device domain.
    fn make_gpu_canvas_shader(
        &mut self,
        _shader: &GpuCanvasShader,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        Err(GpuCanvasError::unsupported())
    }

    /// Execute one imported GPU-canvas plan and retain its result as a normal
    /// render image suitable for `Renderer::draw_image`.
    ///
    /// The default is intentionally fail-closed. Recording, callback, and
    /// other factories do not silently claim support merely because the
    /// scripting surface exists.
    fn make_gpu_canvas_image(
        &mut self,
        _vertex_shader: &Arc<dyn RenderGpuCanvasShader>,
        _fragment_shader: &Arc<dyn RenderGpuCanvasShader>,
        _plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        Err(GpuCanvasError::unsupported())
    }
}

impl<F: Factory + 'static> Factory for PersistentFactory<F> {
    fn persistent_context(&self) -> Option<PersistentFactoryContext> {
        let identity = Rc::as_ptr(&self.access).cast::<()>();
        let access: Rc<dyn PersistentFactoryAccess> = self.access.clone();
        Some(PersistentFactoryContext { access, identity })
    }

    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        self.borrow_mut()
            .make_render_buffer(buffer_type, flags, size_in_bytes)
    }

    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.borrow_mut()
            .make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.borrow_mut()
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        self.borrow_mut().make_render_path(raw_path, fill_rule)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.borrow_mut().make_empty_render_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.borrow_mut().make_render_paint()
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.borrow_mut().decode_image(data)
    }

    fn make_gpu_canvas_shader(
        &mut self,
        shader: &GpuCanvasShader,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.borrow_mut().make_gpu_canvas_shader(shader)
    }

    fn make_gpu_canvas_image(
        &mut self,
        vertex_shader: &Arc<dyn RenderGpuCanvasShader>,
        fragment_shader: &Arc<dyn RenderGpuCanvasShader>,
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        self.borrow_mut()
            .make_gpu_canvas_image(vertex_shader, fragment_shader, plan)
    }
}

#[derive(Debug, Default)]
struct RecordingStream {
    lines: String,
    semantic_commands: Vec<SemanticRecordingCommand>,
}

impl RecordingStream {
    fn line(&mut self, value: impl AsRef<str>) {
        self.lines.push_str(value.as_ref());
        self.lines.push('\n');
    }

    fn line_with(&mut self, write_line: impl FnOnce(&mut String)) {
        write_line(&mut self.lines);
        self.lines.push('\n');
    }

    fn semantic_line(&mut self, value: impl Into<String>) {
        let value = value.into();
        self.line(&value);
        self.semantic_commands
            .push(SemanticRecordingCommand::Line(value));
    }

    fn semantic(&mut self, command: SemanticRecordingCommand) {
        self.semantic_commands.push(command);
    }

    fn clear(&mut self) {
        self.lines.clear();
        self.semantic_commands.clear();
    }
}

#[derive(Debug, Clone)]
enum SemanticRecordingCommand {
    Line(String),
    DrawPath {
        path: RecordingPathSnapshot,
        paint: RecordingPaintSnapshot,
    },
    ClipPath(RecordingPathSnapshot),
    DrawImage {
        image: Option<RecordingImageSnapshot>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    },
    DrawImageMesh {
        image: Option<RecordingImageSnapshot>,
        sampler: ImageSampler,
        vertices: Option<RecordingBufferSnapshot>,
        uv_coords: Option<RecordingBufferSnapshot>,
        indices: Option<RecordingBufferSnapshot>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    },
}

#[derive(Debug, Clone)]
struct RecordingPathSnapshot {
    id: u64,
    raw_path: RawPath,
    fill_rule: FillRule,
}

#[derive(Debug, Clone)]
struct RecordingPaintSnapshot {
    id: u64,
    style: RenderPaintStyle,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
    blend_mode: BlendMode,
    shader: Option<RecordingShaderSnapshot>,
}

#[derive(Debug, Clone)]
struct RecordingShaderSnapshot {
    id: u64,
    gradient: RecordingGradientSnapshot,
}

#[derive(Debug, Clone)]
enum RecordingGradientSnapshot {
    Linear {
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: Vec<ColorInt>,
        stops: Vec<f32>,
    },
    Radial {
        cx: f32,
        cy: f32,
        radius: f32,
        colors: Vec<ColorInt>,
        stops: Vec<f32>,
    },
}

#[derive(Debug, Clone)]
struct RecordingImageSnapshot {
    id: u64,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct RecordingBufferSnapshot {
    id: u64,
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
}

pub struct RecordingRenderer {
    stream: Rc<RefCell<RecordingStream>>,
}

impl RecordingRenderer {
    fn new(stream: Rc<RefCell<RecordingStream>>) -> Self {
        Self { stream }
    }
}

pub struct RecordingFactory {
    stream: Rc<RefCell<RecordingStream>>,
    next_image_id: u64,
    next_paint_id: u64,
    next_path_id: u64,
    next_buffer_id: u64,
    next_shader_id: u64,
}

/// An allocator- and render-cache-independent semantic recording.
///
/// Resource identities are alpha-renamed independently for paths, paints,
/// shaders, images, and buffers. References retain their relationships while
/// incidental allocator numbers disappear from comparisons and fingerprints.
/// Draw commands snapshot complete retained resource state, so the result does
/// not depend on whether the raw golden stream included resource creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalRecording {
    stream: String,
    fnv1a64: u64,
}

impl CanonicalRecording {
    pub fn stream(&self) -> &str {
        &self.stream
    }

    pub fn fnv1a64(&self) -> u64 {
        self.fnv1a64
    }

    pub fn fnv1a64_hex(&self) -> String {
        format!("{:016x}", self.fnv1a64)
    }
}

pub struct NullRenderer;

impl NullRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct NullFactory;

impl NullFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn make_renderer(&self) -> NullRenderer {
        NullRenderer::new()
    }
}

impl RecordingFactory {
    pub fn new() -> Self {
        let stream = Rc::new(RefCell::new(RecordingStream::default()));
        stream.borrow_mut().line("rive-golden-stream-v1");
        Self {
            stream,
            next_image_id: 1,
            next_paint_id: 1,
            next_path_id: 1,
            next_buffer_id: 1,
            next_shader_id: 1,
        }
    }

    pub fn make_renderer(&self) -> RecordingRenderer {
        RecordingRenderer::new(Rc::clone(&self.stream))
    }

    pub fn source(&mut self, file: &str, artboard: &str, scene: &str) {
        self.stream.borrow_mut().semantic_line(format!(
            "source file={} artboard={} scene={}",
            quoted_string(file),
            quoted_string(artboard),
            quoted_string(scene)
        ));
    }

    pub fn add_sample(&mut self, seconds: f32) {
        self.stream
            .borrow_mut()
            .semantic_line(format!("sample seconds={}", float_to_string(seconds)));
    }

    pub fn add_input_event(&mut self, kind: &str, seconds: f32, x: f32, y: f32, pointer_id: i32) {
        self.stream.borrow_mut().semantic_line(format!(
            "input kind={kind} seconds={} position=({},{}) pointerId={pointer_id}",
            float_to_string(seconds),
            float_to_string(x),
            float_to_string(y)
        ));
    }

    // Event/state side-channel lines (docs/side-channel-format.md). Formats
    // must stay byte-compatible with the C++ runner's RecordingFactory
    // emitters in tools/golden-runner/recording_renderer.cpp.

    pub fn add_advance(&mut self, seconds: f32, settled: bool) {
        self.stream.borrow_mut().semantic_line(format!(
            "advance seconds={} settled={settled}",
            float_to_string(seconds)
        ));
    }

    pub fn add_advance_with_states(&mut self, seconds: f32, settled: bool, states_changed: usize) {
        self.stream.borrow_mut().semantic_line(format!(
            "advance seconds={} settled={settled} statesChanged={states_changed}",
            float_to_string(seconds)
        ));
    }

    pub fn add_side_channel_event(&mut self, event: &SideChannelEvent) {
        let mut line = format!(
            "event type={} name={} delay={}",
            event.core_type,
            quoted_string(&event.name),
            float_to_string(event.delay)
        );
        if let Some((url, target)) = &event.url_target {
            line.push_str(&format!(" url={} target={target}", quoted_string(url)));
        }
        line.push_str(" props=[");
        for (index, property) in event.properties.iter().enumerate() {
            if index != 0 {
                line.push(',');
            }
            line.push_str(&format!("{{name={},value=", quoted_string(&property.name)));
            match &property.value {
                SideChannelEventPropertyValue::Number(value) => {
                    line.push_str(&float_to_string(*value));
                }
                SideChannelEventPropertyValue::Bool(value) => {
                    line.push_str(if *value { "true" } else { "false" });
                }
                SideChannelEventPropertyValue::String(value) => {
                    line.push_str(&quoted_string(value));
                }
                SideChannelEventPropertyValue::Color(value) => {
                    line.push_str(&format!("0x{value:08x}"));
                }
                SideChannelEventPropertyValue::Uint(value) => {
                    line.push_str(&value.to_string());
                }
            }
            line.push('}');
        }
        line.push(']');
        self.stream.borrow_mut().semantic_line(line);
    }

    pub fn add_hit_result(&mut self, result: &str) {
        self.stream
            .borrow_mut()
            .semantic_line(format!("hit result={result}"));
    }

    pub fn add_frame(&mut self) {
        self.stream.borrow_mut().semantic_line("frame");
    }

    pub fn frame_size(&mut self, width: u32, height: u32) {
        self.stream
            .borrow_mut()
            .semantic_line(format!("frameSize width={width} height={height}"));
    }

    pub fn clear_color(&mut self, color: ColorInt) {
        self.stream
            .borrow_mut()
            .semantic_line(format!("clearColor value=0x{color:08x}"));
    }

    pub fn stream(&self) -> String {
        self.stream.borrow().lines.clone()
    }

    pub fn canonical_recording(&self) -> CanonicalRecording {
        let stream = canonicalize_recording_commands(&self.stream.borrow().semantic_commands);
        let fnv1a64 = fnv1a64(stream.as_bytes());
        CanonicalRecording { stream, fnv1a64 }
    }

    pub fn clear(&mut self) {
        let mut stream = self.stream.borrow_mut();
        stream.clear();
        stream.line("rive-golden-stream-v1");
    }
}

impl Default for RecordingFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl Factory for RecordingFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        let id = self.next_buffer_id;
        self.next_buffer_id += 1;
        self.stream.borrow_mut().line(format!(
            "makeRenderBuffer id={id} type={} flags={} size={size_in_bytes}",
            buffer_type as u8, flags as u8
        ));
        Box::new(RecordingRenderBuffer {
            stream: Rc::clone(&self.stream),
            id,
            buffer_type,
            flags,
            bytes: vec![0; size_in_bytes],
        })
    }

    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        assert_eq!(colors.len(), stops.len());
        let id = self.next_shader_id;
        self.next_shader_id += 1;
        let mut line = format!(
            "makeLinearGradient id={id} start=({},{}) end=({},{}) stops=[",
            float_to_string(sx),
            float_to_string(sy),
            float_to_string(ex),
            float_to_string(ey)
        );
        write_stops(&mut line, colors, stops);
        line.push(']');
        self.stream.borrow_mut().line(line);
        Box::new(RecordingRenderShader {
            id,
            gradient: RecordingGradientSnapshot::Linear {
                sx,
                sy,
                ex,
                ey,
                colors: colors.to_vec(),
                stops: stops.to_vec(),
            },
        })
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        assert_eq!(colors.len(), stops.len());
        let id = self.next_shader_id;
        self.next_shader_id += 1;
        let mut line = format!(
            "makeRadialGradient id={id} center=({},{}) radius={} stops=[",
            float_to_string(cx),
            float_to_string(cy),
            float_to_string(radius)
        );
        write_stops(&mut line, colors, stops);
        line.push(']');
        self.stream.borrow_mut().line(line);
        Box::new(RecordingRenderShader {
            id,
            gradient: RecordingGradientSnapshot::Radial {
                cx,
                cy,
                radius,
                colors: colors.to_vec(),
                stops: stops.to_vec(),
            },
        })
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        let id = self.next_path_id;
        self.next_path_id += 1;
        let path = RecordingRenderPath {
            id,
            raw_path,
            fill_rule,
        };
        self.stream.borrow_mut().line_with(|line| {
            line.push_str("makeRenderPath ");
            path.write_snapshot(line);
        });
        Box::new(path)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        let id = self.next_path_id;
        self.next_path_id += 1;
        let path = RecordingRenderPath {
            id,
            raw_path: RawPath::new(),
            fill_rule: FillRule::NonZero,
        };
        self.stream.borrow_mut().line_with(|line| {
            line.push_str("makeEmptyRenderPath ");
            path.write_snapshot(line);
        });
        Box::new(path)
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        let id = self.next_paint_id;
        self.next_paint_id += 1;
        let paint = RecordingRenderPaint {
            id,
            style: RenderPaintStyle::Fill,
            color: 0xff000000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
            shader: None,
        };
        self.stream.borrow_mut().line_with(|line| {
            line.push_str("makeRenderPaint ");
            paint.write_snapshot(line);
        });
        Box::new(paint)
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let id = self.next_image_id;
        self.next_image_id += 1;
        let (width, height) = encoded_image_dimensions(data);
        self.stream.borrow_mut().line(format!(
            "decodeImage id={id} width={width} height={height} data={}",
            hex_bytes(data)
        ));
        Ok(Box::new(RecordingRenderImage {
            id,
            width,
            height,
            data: data.to_vec(),
        }))
    }
}

impl Factory for NullFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        Box::new(NullRenderBuffer {
            buffer_type,
            flags,
            bytes: vec![0; size_in_bytes],
        })
    }

    fn make_linear_gradient(
        &mut self,
        _sx: f32,
        _sy: f32,
        _ex: f32,
        _ey: f32,
        _colors: &[ColorInt],
        _stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(NullRenderShader)
    }

    fn make_radial_gradient(
        &mut self,
        _cx: f32,
        _cy: f32,
        _radius: f32,
        _colors: &[ColorInt],
        _stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(NullRenderShader)
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        Box::new(NullRenderPath {
            raw_path,
            fill_rule,
        })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        Box::new(NullRenderPath {
            raw_path: RawPath::new(),
            fill_rule: FillRule::NonZero,
        })
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        Box::new(NullRenderPaint {
            style: RenderPaintStyle::Fill,
            color: 0xff000000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
        })
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let (width, height) = encoded_image_dimensions(data);
        Ok(Box::new(NullRenderImage { width, height }))
    }
}

struct NullRenderShader;

impl RenderShader for NullRenderShader {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct NullRenderImage {
    width: u32,
    height: u32,
}

impl RenderImage for NullRenderImage {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

struct NullRenderPaint {
    style: RenderPaintStyle,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
    blend_mode: BlendMode,
}

impl RenderPaint for NullRenderPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn style(&mut self, style: RenderPaintStyle) {
        self.style = style;
    }

    fn color(&mut self, value: ColorInt) {
        self.color = value;
    }

    fn thickness(&mut self, value: f32) {
        self.thickness = value;
    }

    fn join(&mut self, value: StrokeJoin) {
        self.join = value;
    }

    fn cap(&mut self, value: StrokeCap) {
        self.cap = value;
    }

    fn feather(&mut self, value: f32) {
        self.feather = value;
    }

    fn blend_mode(&mut self, value: BlendMode) {
        self.blend_mode = value;
    }

    fn shader(&mut self, _shader: Option<&dyn RenderShader>) {}

    fn invalidate_stroke(&mut self) {}
}

struct NullRenderPath {
    raw_path: RawPath,
    fill_rule: FillRule,
}

impl RenderPath for NullRenderPath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn rewind(&mut self) {
        self.raw_path.rewind();
    }

    fn reserve(&mut self, verbs: usize, points: usize) {
        self.raw_path.reserve(verbs, points);
    }

    fn fill_rule(&mut self, value: FillRule) {
        self.fill_rule = value;
    }

    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = null_path(path);
        self.raw_path.add_path(&path.raw_path, transform);
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = null_path(path);
        self.raw_path.add_path_backwards(&path.raw_path, transform);
    }

    fn add_raw_path(&mut self, path: &RawPath) {
        self.raw_path.add_path(path, Mat2D::IDENTITY);
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.raw_path.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.raw_path.line_to(x, y);
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.raw_path.cubic_to(ox, oy, ix, iy, x, y);
    }

    fn close(&mut self) {
        self.raw_path.close();
    }
}

struct NullRenderBuffer {
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
}

impl RenderBuffer for NullRenderBuffer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn buffer_type(&self) -> RenderBufferType {
        self.buffer_type
    }

    fn flags(&self) -> RenderBufferFlags {
        self.flags
    }

    fn size_in_bytes(&self) -> usize {
        self.bytes.len()
    }

    fn map_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    fn unmap(&mut self) {}
}

impl Renderer for NullRenderer {
    fn save(&mut self) {}

    fn restore(&mut self) {}

    fn transform(&mut self, _transform: Mat2D) {}

    fn draw_path(&mut self, _path: &dyn RenderPath, _paint: &dyn RenderPaint) {}

    fn clip_path(&mut self, _path: &dyn RenderPath) {}

    fn draw_image(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
    }

    fn draw_image_mesh(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _vertices: Option<&dyn RenderBuffer>,
        _uv_coords: Option<&dyn RenderBuffer>,
        _indices: Option<&dyn RenderBuffer>,
        _vertex_count: u32,
        _index_count: u32,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
    }

    fn modulate_opacity(&mut self, _opacity: f32) {}
}

fn null_path(path: &dyn RenderPath) -> &NullRenderPath {
    path.as_any()
        .downcast_ref::<NullRenderPath>()
        .expect("NullFactory requires NullRenderPath")
}

struct RecordingRenderShader {
    id: u64,
    gradient: RecordingGradientSnapshot,
}

impl RecordingRenderShader {
    fn snapshot(&self) -> RecordingShaderSnapshot {
        RecordingShaderSnapshot {
            id: self.id,
            gradient: self.gradient.clone(),
        }
    }
}

impl RenderShader for RecordingRenderShader {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct RecordingRenderImage {
    id: u64,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl RecordingRenderImage {
    fn snapshot(&self) -> RecordingImageSnapshot {
        RecordingImageSnapshot {
            id: self.id,
            width: self.width,
            height: self.height,
            data: self.data.clone(),
        }
    }
}

impl RenderImage for RecordingRenderImage {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

struct RecordingRenderPaint {
    id: u64,
    style: RenderPaintStyle,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
    blend_mode: BlendMode,
    shader: Option<RecordingShaderSnapshot>,
}

impl RecordingRenderPaint {
    fn snapshot(&self) -> RecordingPaintSnapshot {
        RecordingPaintSnapshot {
            id: self.id,
            style: self.style,
            color: self.color,
            thickness: self.thickness,
            join: self.join,
            cap: self.cap,
            feather: self.feather,
            blend_mode: self.blend_mode,
            shader: self.shader.clone(),
        }
    }

    fn write_snapshot(&self, out: &mut String) {
        write!(out, "{{id={},style=", self.id).expect("writing to a String cannot fail");
        out.push_str(match self.style {
            RenderPaintStyle::Stroke => "stroke",
            RenderPaintStyle::Fill => "fill",
        });
        out.push_str(",color=");
        write_color(out, self.color);
        out.push_str(",thickness=");
        write_float(out, self.thickness);
        write!(
            out,
            ",join={},cap={},feather=",
            self.join as u32, self.cap as u32
        )
        .expect("writing to a String cannot fail");
        write_float(out, self.feather);
        write!(
            out,
            ",blendMode={},shader={}}}",
            self.blend_mode as u8,
            self.shader.as_ref().map_or(0, |shader| shader.id)
        )
        .expect("writing to a String cannot fail");
    }
}

impl RenderPaint for RecordingRenderPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn style(&mut self, style: RenderPaintStyle) {
        self.style = style;
    }

    fn color(&mut self, value: ColorInt) {
        self.color = value;
    }

    fn thickness(&mut self, value: f32) {
        self.thickness = value;
    }

    fn join(&mut self, value: StrokeJoin) {
        self.join = value;
    }

    fn cap(&mut self, value: StrokeCap) {
        self.cap = value;
    }

    fn feather(&mut self, value: f32) {
        self.feather = value;
    }

    fn blend_mode(&mut self, value: BlendMode) {
        self.blend_mode = value;
    }

    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        self.shader = shader
            .and_then(|shader| shader.as_any().downcast_ref::<RecordingRenderShader>())
            .map(RecordingRenderShader::snapshot);
    }

    fn invalidate_stroke(&mut self) {}
}

struct RecordingRenderPath {
    id: u64,
    raw_path: RawPath,
    fill_rule: FillRule,
}

impl RecordingRenderPath {
    fn snapshot(&self) -> RecordingPathSnapshot {
        RecordingPathSnapshot {
            id: self.id,
            raw_path: self.raw_path.clone(),
            fill_rule: self.fill_rule,
        }
    }

    fn write_snapshot(&self, out: &mut String) {
        write!(
            out,
            "{{id={},fillRule={},path=",
            self.id, self.fill_rule as u8
        )
        .expect("writing to a String cannot fail");
        write_raw_path(out, &self.raw_path);
        out.push('}');
    }
}

impl RenderPath for RecordingRenderPath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn rewind(&mut self) {
        self.raw_path.rewind();
    }

    fn reserve(&mut self, verbs: usize, points: usize) {
        self.raw_path.reserve(verbs, points);
    }

    fn fill_rule(&mut self, value: FillRule) {
        self.fill_rule = value;
    }

    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = recording_path(path);
        self.raw_path.add_path(&path.raw_path, transform);
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = recording_path(path);
        self.raw_path.add_path_backwards(&path.raw_path, transform);
    }

    fn add_raw_path(&mut self, path: &RawPath) {
        self.raw_path.add_path(path, Mat2D::IDENTITY);
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.raw_path.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.raw_path.line_to(x, y);
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.raw_path.cubic_to(ox, oy, ix, iy, x, y);
    }

    fn close(&mut self) {
        self.raw_path.close();
    }
}

struct RecordingRenderBuffer {
    stream: Rc<RefCell<RecordingStream>>,
    id: u64,
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
}

impl RecordingRenderBuffer {
    fn snapshot(&self) -> RecordingBufferSnapshot {
        RecordingBufferSnapshot {
            id: self.id,
            buffer_type: self.buffer_type,
            flags: self.flags,
            bytes: self.bytes.clone(),
        }
    }
}

impl RenderBuffer for RecordingRenderBuffer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn buffer_type(&self) -> RenderBufferType {
        self.buffer_type
    }

    fn flags(&self) -> RenderBufferFlags {
        self.flags
    }

    fn size_in_bytes(&self) -> usize {
        self.bytes.len()
    }

    fn map_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    fn unmap(&mut self) {
        self.stream.borrow_mut().line(format!(
            "bufferData id={} type={} size={} data={}",
            self.id,
            self.buffer_type as u8,
            self.bytes.len(),
            hex_bytes(&self.bytes)
        ));
    }
}

impl Renderer for RecordingRenderer {
    fn save(&mut self) {
        self.stream.borrow_mut().semantic_line("save");
    }

    fn restore(&mut self) {
        self.stream.borrow_mut().semantic_line("restore");
    }

    fn transform(&mut self, transform: Mat2D) {
        self.stream
            .borrow_mut()
            .semantic_line(format!("transform matrix={}", mat_to_string(transform)));
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        let path = recording_path(path);
        let paint = recording_paint(paint);
        let mut stream = self.stream.borrow_mut();
        stream.line_with(|line| {
            line.push_str("drawPath path=");
            path.write_snapshot(line);
            line.push_str(" paint=");
            paint.write_snapshot(line);
        });
        stream.semantic(SemanticRecordingCommand::DrawPath {
            path: path.snapshot(),
            paint: paint.snapshot(),
        });
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        let path = recording_path(path);
        let mut stream = self.stream.borrow_mut();
        stream.line_with(|line| {
            line.push_str("clipPath path=");
            path.write_snapshot(line);
        });
        stream.semantic(SemanticRecordingCommand::ClipPath(path.snapshot()));
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let mut stream = self.stream.borrow_mut();
        stream.line(format!(
            "drawImage image={} sampler={} blendMode={} opacity={}",
            image_id(image),
            sampler_to_string(sampler),
            blend_mode as u8,
            float_to_string(opacity)
        ));
        stream.semantic(SemanticRecordingCommand::DrawImage {
            image: image_snapshot(image),
            sampler,
            blend_mode,
            opacity,
        });
    }

    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let mut stream = self.stream.borrow_mut();
        stream.line(format!(
            "drawImageMesh image={} sampler={} vertices={} uvs={} indices={} vertexCount={} indexCount={} blendMode={} opacity={}",
            image_id(image),
            sampler_to_string(sampler),
            buffer_id(vertices),
            buffer_id(uv_coords),
            buffer_id(indices),
            vertex_count,
            index_count,
            blend_mode as u8,
            float_to_string(opacity)
        ));
        stream.semantic(SemanticRecordingCommand::DrawImageMesh {
            image: image_snapshot(image),
            sampler,
            vertices: buffer_snapshot(vertices),
            uv_coords: buffer_snapshot(uv_coords),
            indices: buffer_snapshot(indices),
            vertex_count,
            index_count,
            blend_mode,
            opacity,
        });
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.stream.borrow_mut().semantic_line(format!(
            "modulateOpacity opacity={}",
            float_to_string(opacity)
        ));
    }
}

fn recording_path(path: &dyn RenderPath) -> &RecordingRenderPath {
    path.as_any()
        .downcast_ref::<RecordingRenderPath>()
        .expect("RecordingRenderer requires RecordingRenderPath")
}

fn recording_paint(paint: &dyn RenderPaint) -> &RecordingRenderPaint {
    paint
        .as_any()
        .downcast_ref::<RecordingRenderPaint>()
        .expect("RecordingRenderer requires RecordingRenderPaint")
}

fn image_id(image: Option<&dyn RenderImage>) -> u64 {
    image
        .and_then(|image| image.as_any().downcast_ref::<RecordingRenderImage>())
        .map(|image| image.id)
        .unwrap_or(0)
}

fn image_snapshot(image: Option<&dyn RenderImage>) -> Option<RecordingImageSnapshot> {
    image
        .and_then(|image| image.as_any().downcast_ref::<RecordingRenderImage>())
        .map(RecordingRenderImage::snapshot)
}

fn buffer_id(buffer: Option<&dyn RenderBuffer>) -> u64 {
    buffer
        .and_then(|buffer| buffer.as_any().downcast_ref::<RecordingRenderBuffer>())
        .map(|buffer| buffer.id)
        .unwrap_or(0)
}

fn buffer_snapshot(buffer: Option<&dyn RenderBuffer>) -> Option<RecordingBufferSnapshot> {
    buffer
        .and_then(|buffer| buffer.as_any().downcast_ref::<RecordingRenderBuffer>())
        .map(RecordingRenderBuffer::snapshot)
}

#[derive(Debug, Default)]
struct CanonicalResourceIds {
    raw_to_canonical: HashMap<u64, u64>,
}

impl CanonicalResourceIds {
    fn canonicalize(&mut self, raw: u64) -> u64 {
        if raw == 0 {
            return 0;
        }
        if let Some(canonical) = self.raw_to_canonical.get(&raw) {
            return *canonical;
        }

        let canonical = self.raw_to_canonical.len() as u64 + 1;
        self.raw_to_canonical.insert(raw, canonical);
        canonical
    }
}

#[derive(Debug, Default)]
struct CanonicalRecordingIds {
    paths: CanonicalResourceIds,
    paints: CanonicalResourceIds,
    shaders: CanonicalResourceIds,
    images: CanonicalResourceIds,
    buffers: CanonicalResourceIds,
}

fn canonicalize_recording_commands(commands: &[SemanticRecordingCommand]) -> String {
    let mut ids = CanonicalRecordingIds::default();
    let mut canonical = String::from("nuxie-canonical-recording-v1\n");

    for command in commands {
        match command {
            SemanticRecordingCommand::Line(line) => canonical.push_str(line),
            SemanticRecordingCommand::DrawPath { path, paint } => {
                canonical.push_str("drawPath path=");
                write_canonical_path(&mut canonical, path, &mut ids.paths);
                canonical.push_str(" paint=");
                write_canonical_paint(&mut canonical, paint, &mut ids.paints, &mut ids.shaders);
            }
            SemanticRecordingCommand::ClipPath(path) => {
                canonical.push_str("clipPath path=");
                write_canonical_path(&mut canonical, path, &mut ids.paths);
            }
            SemanticRecordingCommand::DrawImage {
                image,
                sampler,
                blend_mode,
                opacity,
            } => {
                canonical.push_str("drawImage image=");
                write_canonical_image(&mut canonical, image.as_ref(), &mut ids.images);
                canonical.push_str(" sampler=");
                canonical.push_str(&sampler_to_string(*sampler));
                write!(canonical, " blendMode={} opacity=", *blend_mode as u8)
                    .expect("writing to a String cannot fail");
                write_float(&mut canonical, *opacity);
            }
            SemanticRecordingCommand::DrawImageMesh {
                image,
                sampler,
                vertices,
                uv_coords,
                indices,
                vertex_count,
                index_count,
                blend_mode,
                opacity,
            } => {
                canonical.push_str("drawImageMesh image=");
                write_canonical_image(&mut canonical, image.as_ref(), &mut ids.images);
                canonical.push_str(" sampler=");
                canonical.push_str(&sampler_to_string(*sampler));
                canonical.push_str(" vertices=");
                write_canonical_buffer(&mut canonical, vertices.as_ref(), &mut ids.buffers);
                canonical.push_str(" uvs=");
                write_canonical_buffer(&mut canonical, uv_coords.as_ref(), &mut ids.buffers);
                canonical.push_str(" indices=");
                write_canonical_buffer(&mut canonical, indices.as_ref(), &mut ids.buffers);
                write!(
                    canonical,
                    " vertexCount={vertex_count} indexCount={index_count} blendMode={} opacity=",
                    *blend_mode as u8
                )
                .expect("writing to a String cannot fail");
                write_float(&mut canonical, *opacity);
            }
        }
        canonical.push('\n');
    }

    canonical
}

fn write_canonical_path(
    out: &mut String,
    path: &RecordingPathSnapshot,
    ids: &mut CanonicalResourceIds,
) {
    write!(
        out,
        "{{id={},fillRule={},path=",
        ids.canonicalize(path.id),
        path.fill_rule as u8
    )
    .expect("writing to a String cannot fail");
    write_raw_path(out, &path.raw_path);
    out.push('}');
}

fn write_canonical_paint(
    out: &mut String,
    paint: &RecordingPaintSnapshot,
    paint_ids: &mut CanonicalResourceIds,
    shader_ids: &mut CanonicalResourceIds,
) {
    write!(out, "{{id={},style=", paint_ids.canonicalize(paint.id))
        .expect("writing to a String cannot fail");
    out.push_str(match paint.style {
        RenderPaintStyle::Stroke => "stroke",
        RenderPaintStyle::Fill => "fill",
    });
    out.push_str(",color=");
    write_color(out, paint.color);
    out.push_str(",thickness=");
    write_float(out, paint.thickness);
    write!(
        out,
        ",join={},cap={},feather=",
        paint.join as u32, paint.cap as u32
    )
    .expect("writing to a String cannot fail");
    write_float(out, paint.feather);
    write!(out, ",blendMode={},shader=", paint.blend_mode as u8)
        .expect("writing to a String cannot fail");
    write_canonical_shader(out, paint.shader.as_ref(), shader_ids);
    out.push('}');
}

fn write_canonical_shader(
    out: &mut String,
    shader: Option<&RecordingShaderSnapshot>,
    ids: &mut CanonicalResourceIds,
) {
    let Some(shader) = shader else {
        out.push('0');
        return;
    };
    write!(out, "{{id={},", ids.canonicalize(shader.id)).expect("writing to a String cannot fail");
    match &shader.gradient {
        RecordingGradientSnapshot::Linear {
            sx,
            sy,
            ex,
            ey,
            colors,
            stops,
        } => {
            out.push_str("kind=linear,start=(");
            write_float(out, *sx);
            out.push(',');
            write_float(out, *sy);
            out.push_str("),end=(");
            write_float(out, *ex);
            out.push(',');
            write_float(out, *ey);
            out.push_str("),stops=[");
            write_stops(out, colors, stops);
        }
        RecordingGradientSnapshot::Radial {
            cx,
            cy,
            radius,
            colors,
            stops,
        } => {
            out.push_str("kind=radial,center=(");
            write_float(out, *cx);
            out.push(',');
            write_float(out, *cy);
            out.push_str("),radius=");
            write_float(out, *radius);
            out.push_str(",stops=[");
            write_stops(out, colors, stops);
        }
    }
    out.push_str("]}");
}

fn write_canonical_image(
    out: &mut String,
    image: Option<&RecordingImageSnapshot>,
    ids: &mut CanonicalResourceIds,
) {
    let Some(image) = image else {
        out.push('0');
        return;
    };
    write!(
        out,
        "{{id={},width={},height={},data={}}}",
        ids.canonicalize(image.id),
        image.width,
        image.height,
        hex_bytes(&image.data)
    )
    .expect("writing to a String cannot fail");
}

fn write_canonical_buffer(
    out: &mut String,
    buffer: Option<&RecordingBufferSnapshot>,
    ids: &mut CanonicalResourceIds,
) {
    let Some(buffer) = buffer else {
        out.push('0');
        return;
    };
    write!(
        out,
        "{{id={},type={},flags={},size={},data={}}}",
        ids.canonicalize(buffer.id),
        buffer.buffer_type as u8,
        buffer.flags as u8,
        buffer.bytes.len(),
        hex_bytes(&buffer.bytes)
    )
    .expect("writing to a String cannot fail");
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3)
    })
}

fn write_stops(out: &mut String, colors: &[ColorInt], stops: &[f32]) {
    for (index, (color, stop)) in colors.iter().zip(stops).enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push_str("{color=");
        write_color(out, *color);
        out.push_str(",stop=");
        write_float(out, *stop);
        out.push('}');
    }
}

fn write_raw_path(out: &mut String, path: &RawPath) {
    out.push_str("{verbs=[");
    for (index, verb) in path.verbs().iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push_str(match verb {
            PathVerb::Move => "move",
            PathVerb::Line => "line",
            PathVerb::Quad => "quad",
            PathVerb::Cubic => "cubic",
            PathVerb::Close => "close",
        });
    }
    out.push_str("],points=[");
    for (index, point) in path.points().iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push('(');
        write_float(out, point.x);
        out.push(',');
        write_float(out, point.y);
        out.push(')');
    }
    out.push_str("]}");
}

fn sampler_to_string(sampler: ImageSampler) -> String {
    format!(
        "{{wrapX={},wrapY={},filter={},key={}}}",
        sampler.wrap_x as u8,
        sampler.wrap_y as u8,
        sampler.filter as u8,
        sampler.as_key()
    )
}

fn mat_to_string(mat: Mat2D) -> String {
    let mut out = String::from("[");
    for (index, value) in mat.0.into_iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        write_float(&mut out, value);
    }
    out.push(']');
    out
}

fn write_color(out: &mut String, color: ColorInt) {
    write!(out, "0x{color:08x}").expect("writing to a String cannot fail");
}

/// One reported event for the golden side-channel
/// (docs/side-channel-format.md). Mirrors the C++ runner's SideChannelEvent
/// in tools/golden-runner/recording_renderer.hpp.
#[derive(Debug, Clone, Default)]
pub struct SideChannelEvent {
    pub core_type: u32,
    pub name: String,
    pub delay: f32,
    /// `Some((url, target))` only for OpenUrlEvent.
    pub url_target: Option<(String, String)>,
    pub properties: Vec<SideChannelEventProperty>,
}

#[derive(Debug, Clone)]
pub struct SideChannelEventProperty {
    pub name: String,
    pub value: SideChannelEventPropertyValue,
}

#[derive(Debug, Clone)]
pub enum SideChannelEventPropertyValue {
    Number(f32),
    Bool(bool),
    String(String),
    Color(u32),
    Uint(u64),
}

fn quoted_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::new();
    for byte in bytes {
        write!(out, "{byte:02x}").expect("writing to a String cannot fail");
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodedImageFormat {
    Png,
    Jpeg,
    WebP,
}

impl EncodedImageFormat {
    #[must_use]
    pub const fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::WebP => "image/webp",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodedImageMetadata {
    pub format: EncodedImageFormat,
    pub width: u32,
    pub height: u32,
}

/// Inspect the encoded raster formats supported by the pure-Rust renderer.
/// This reads only bounded header/chunk metadata and does not allocate a
/// decoded pixel buffer.
#[must_use]
pub fn encoded_image_metadata(bytes: &[u8]) -> Option<EncodedImageMetadata> {
    let (format, (width, height)) = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        (EncodedImageFormat::Png, png_dimensions(bytes)?)
    } else if bytes.starts_with(&[0xff, 0xd8]) {
        (EncodedImageFormat::Jpeg, jpeg_dimensions(bytes)?)
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        (EncodedImageFormat::WebP, webp_dimensions(bytes)?)
    } else {
        return None;
    };
    if width == 0 || height == 0 {
        return None;
    }
    Some(EncodedImageMetadata {
        format,
        width,
        height,
    })
}

fn encoded_image_dimensions(bytes: &[u8]) -> (u32, u32) {
    encoded_image_metadata(bytes)
        .map(|metadata| (metadata.width, metadata.height))
        .unwrap_or((0, 0))
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return None;
    }
    Some((read_be_u32(bytes, 16)?, read_be_u32(bytes, 20)?))
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }

    let mut offset = 2usize;
    while offset + 4 <= bytes.len() {
        while offset < bytes.len() && bytes[offset] != 0xff {
            offset += 1;
        }
        while offset < bytes.len() && bytes[offset] == 0xff {
            offset += 1;
        }
        let marker = *bytes.get(offset)?;
        offset += 1;
        if marker == 0xd9 || marker == 0xda {
            break;
        }
        if (0xd0..=0xd7).contains(&marker) {
            continue;
        }

        let segment_length = usize::from(read_be_u16(bytes, offset)?);
        if segment_length < 2 || offset + segment_length > bytes.len() {
            break;
        }
        if jpeg_start_of_frame(marker) && segment_length >= 7 {
            let height = u32::from(read_be_u16(bytes, offset + 3)?);
            let width = u32::from(read_be_u16(bytes, offset + 5)?);
            return Some((width, height));
        }
        offset += segment_length;
    }

    None
}

fn jpeg_start_of_frame(marker: u8) -> bool {
    (0xc0..=0xc3).contains(&marker)
        || (0xc5..=0xc7).contains(&marker)
        || (0xc9..=0xcb).contains(&marker)
        || (0xcd..=0xcf).contains(&marker)
}

fn webp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 20 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }

    let mut offset = 12usize;
    while offset + 8 <= bytes.len() {
        let chunk_data = offset + 8;
        let chunk_size = usize::try_from(read_le_u32(bytes, offset + 4)?).ok()?;
        let chunk_end = chunk_data.checked_add(chunk_size)?;
        if chunk_end > bytes.len() {
            break;
        }

        if &bytes[offset..offset + 4] == b"VP8X" && chunk_size >= 10 {
            return Some((
                read_le_u24(bytes, chunk_data + 4)? + 1,
                read_le_u24(bytes, chunk_data + 7)? + 1,
            ));
        }
        if &bytes[offset..offset + 4] == b"VP8L" && chunk_size >= 5 && bytes[chunk_data] == 0x2f {
            let width = 1
                + u32::from(bytes[chunk_data + 1])
                + (u32::from(bytes[chunk_data + 2] & 0x3f) << 8);
            let height = 1
                + u32::from(bytes[chunk_data + 2] >> 6)
                + (u32::from(bytes[chunk_data + 3]) << 2)
                + (u32::from(bytes[chunk_data + 4] & 0x0f) << 10);
            return Some((width, height));
        }
        if &bytes[offset..offset + 4] == b"VP8 "
            && chunk_size >= 10
            && &bytes[chunk_data + 3..chunk_data + 6] == b"\x9d\x01\x2a"
        {
            return Some((
                u32::from(read_le_u16(bytes, chunk_data + 6)? & 0x3fff),
                u32::from(read_le_u16(bytes, chunk_data + 8)? & 0x3fff),
            ));
        }

        offset = chunk_end.checked_add(chunk_size & 1)?;
    }

    None
}

fn read_be_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_be_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_le_u24(bytes: &[u8], offset: usize) -> Option<u32> {
    let bytes = bytes.get(offset..offset + 3)?;
    Some(u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16))
}

fn read_le_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn float_to_string(value: f32) -> String {
    let mut out = String::new();
    write_float(&mut out, value);
    out
}

fn write_float(out: &mut String, value: f32) {
    // C++ RecordingRenderer uses iostream defaultfloat with float max_digits10.
    // Nine significant digits round-trip every f32. Formatting one digit before
    // the decimal and eight after it first also pins the rounding carry before
    // applying defaultfloat's fixed/scientific threshold and zero trimming.
    if value.is_nan() {
        out.push_str("nan");
        return;
    }
    if value == f32::INFINITY {
        out.push_str("inf");
        return;
    }
    if value == f32::NEG_INFINITY {
        out.push_str("-inf");
        return;
    }

    let scientific = format!("{value:.8e}");
    let (mantissa, exponent) = scientific
        .split_once('e')
        .expect("Rust scientific float formatting always emits an exponent");
    let exponent = exponent
        .parse::<i32>()
        .expect("Rust scientific float formatting emits a decimal exponent");
    let negative = mantissa.starts_with('-');
    let mantissa = mantissa.strip_prefix('-').unwrap_or(mantissa);
    let mut digits = mantissa
        .bytes()
        .filter(|byte| *byte != b'.')
        .collect::<Vec<_>>();
    while digits.len() > 1 && digits.last() == Some(&b'0') {
        digits.pop();
    }

    if negative {
        out.push('-');
    }
    if (-4..9).contains(&exponent) {
        if exponent < 0 {
            out.push_str("0.");
            for _ in 0..(-exponent - 1) {
                out.push('0');
            }
            for digit in digits {
                out.push(char::from(digit));
            }
        } else {
            let integer_digits =
                usize::try_from(exponent + 1).expect("nonnegative decimal exponent fits usize");
            for (index, digit) in digits.iter().enumerate() {
                if index == integer_digits {
                    out.push('.');
                }
                out.push(char::from(*digit));
            }
            if digits.len() < integer_digits {
                for _ in digits.len()..integer_digits {
                    out.push('0');
                }
            }
        }
    } else {
        out.push(char::from(digits[0]));
        if digits.len() > 1 {
            out.push('.');
            for digit in &digits[1..] {
                out.push(char::from(*digit));
            }
        }
        out.push('e');
        if exponent < 0 {
            out.push('-');
        } else {
            out.push('+');
        }
        let magnitude = exponent.unsigned_abs();
        if magnitude < 10 {
            out.push('0');
        }
        write!(out, "{magnitude}").expect("writing to a String cannot fail");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    #[test]
    fn encoded_image_metadata_reports_supported_raster_identity() {
        let mut png = vec![0; 24];
        png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        png[12..16].copy_from_slice(b"IHDR");
        png[16..20].copy_from_slice(&3_u32.to_be_bytes());
        png[20..24].copy_from_slice(&5_u32.to_be_bytes());
        assert_eq!(
            encoded_image_metadata(&png),
            Some(EncodedImageMetadata {
                format: EncodedImageFormat::Png,
                width: 3,
                height: 5,
            })
        );
        assert_eq!(EncodedImageFormat::Png.content_type(), "image/png");
        assert_eq!(encoded_image_metadata(b"not an encoded image"), None);
    }

    #[test]
    fn raw_path_mutation_ids_track_object_snapshots() {
        let mut first = RawPath::new();
        let second = RawPath::new();
        assert_ne!(first.mutation_id(), second.mutation_id());

        first.move_to(1.0, 2.0);
        let snapshot = first.clone();
        assert_eq!(first.mutation_id(), snapshot.mutation_id());

        first.reserve(8, 8);
        assert_eq!(first.mutation_id(), snapshot.mutation_id());
        first.line_to(3.0, 4.0);
        assert_ne!(first.mutation_id(), snapshot.mutation_id());

        let mut distinct_object = snapshot.clone();
        distinct_object.renew_mutation_id();
        assert_eq!(distinct_object, snapshot);
        assert_ne!(distinct_object.mutation_id(), snapshot.mutation_id());
    }

    #[test]
    fn raw_path_rebuild_preserves_geometry_and_renews_snapshot_identity() {
        fn build_fixture(path: &mut RawPathBuilder<'_>) {
            path.line_to(1.0, 2.0);
            path.cubic_to(3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
            path.close();
            path.close();
            path.line_to(9.0, 10.0);
            path.quad_to(11.0, 12.0, 13.0, 14.0);
        }

        let mut expected = RawPath::new();
        expected.line_to(1.0, 2.0);
        expected.cubic_to(3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
        expected.close();
        expected.close();
        expected.line_to(9.0, 10.0);
        expected.quad_to(11.0, 12.0, 13.0, 14.0);

        let mut rebuilt = RawPath::new();
        let empty_mutation_id = rebuilt.mutation_id();
        rebuilt.rebuild(7, 9, build_fixture);

        assert_eq!(rebuilt, expected);
        assert_ne!(rebuilt.mutation_id(), empty_mutation_id);

        let first_populated_mutation_id = rebuilt.mutation_id();
        rebuilt.rebuild(7, 9, build_fixture);
        assert_eq!(rebuilt, expected);
        assert_ne!(rebuilt.mutation_id(), first_populated_mutation_id);

        let populated_snapshot = rebuilt.clone();
        rebuilt.rebuild(0, 0, |_| {});
        assert!(rebuilt.verbs().is_empty());
        assert!(rebuilt.points().is_empty());
        assert_ne!(rebuilt.mutation_id(), populated_snapshot.mutation_id());
        assert_eq!(populated_snapshot, expected);
    }

    #[test]
    fn identity_path_appends_preserve_point_bits() {
        let mut source = RawPath::new();
        source.move_to(-0.0, -0.0);
        source.line_to(1.0, -0.0);

        let mut forward = RawPath::new();
        forward.add_path(&source, Mat2D::IDENTITY);
        assert_eq!(forward.points()[0].x.to_bits(), (-0.0_f32).to_bits());
        assert_eq!(forward.points()[0].y.to_bits(), (-0.0_f32).to_bits());
        assert_eq!(forward.points()[1].y.to_bits(), (-0.0_f32).to_bits());

        let mut backwards = RawPath::new();
        backwards.add_path_backwards(&source, Mat2D::IDENTITY);
        assert_eq!(backwards.points()[0].y.to_bits(), (-0.0_f32).to_bits());
        assert_eq!(backwards.points()[1].x.to_bits(), (-0.0_f32).to_bits());
        assert_eq!(backwards.points()[1].y.to_bits(), (-0.0_f32).to_bits());
    }

    #[test]
    fn transformed_path_appends_match_cpp_simd_evaluation_order() {
        let mut source = RawPath::new();
        source.move_to(85.5, -61.0);

        let mut transformed = RawPath::new();
        transformed.add_path(
            &source,
            Mat2D([0.8660254, 0.5, -0.5, 0.8660254, 12.124355, 7.0]),
        );

        assert_eq!(transformed.points()[0].x.to_bits(), 0x42e9_56cc);
        assert_eq!(transformed.points()[0].y.to_bits(), 0xc044_f68f);
    }

    fn assert_backwards_round_trip(path: &RawPath) {
        let mut backwards = RawPath::new();
        backwards.add_path_backwards(path, Mat2D::IDENTITY);

        let mut restored = RawPath::new();
        restored.add_path_backwards(&backwards, Mat2D::IDENTITY);

        assert_eq!(&restored, path);
    }

    #[test]
    fn raw_path_mutators_normalize_contours_for_backwards_reversal() {
        let mut leading_line = RawPath::new();
        leading_line.line_to(1.0, 2.0);
        assert_eq!(leading_line.verbs(), &[PathVerb::Move, PathVerb::Line]);
        assert_eq!(
            leading_line.points(),
            &[Vec2D::new(0.0, 0.0), Vec2D::new(1.0, 2.0)]
        );
        assert_backwards_round_trip(&leading_line);

        let mut leading_quad = RawPath::new();
        leading_quad.quad_to(1.0, 2.0, 3.0, 4.0);
        assert_eq!(leading_quad.verbs(), &[PathVerb::Move, PathVerb::Quad]);
        assert_backwards_round_trip(&leading_quad);

        let mut path = RawPath::new();
        path.cubic_to(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        path.close();
        path.close();
        path.cubic_to(7.0, 8.0, 9.0, 10.0, 11.0, 12.0);
        assert_eq!(
            path.verbs(),
            &[
                PathVerb::Move,
                PathVerb::Cubic,
                PathVerb::Close,
                PathVerb::Move,
                PathVerb::Cubic,
            ]
        );
        assert_eq!(
            path.points(),
            &[
                Vec2D::new(0.0, 0.0),
                Vec2D::new(1.0, 2.0),
                Vec2D::new(3.0, 4.0),
                Vec2D::new(5.0, 6.0),
                Vec2D::new(0.0, 0.0),
                Vec2D::new(7.0, 8.0),
                Vec2D::new(9.0, 10.0),
                Vec2D::new(11.0, 12.0),
            ]
        );
        assert_backwards_round_trip(&path);
    }

    #[test]
    fn add_path_backwards_handles_empty_close_only_move_only_and_empty_contours() {
        let empty = RawPath::new();
        assert_backwards_round_trip(&empty);

        let mut close_only = RawPath::new();
        close_only.close();
        assert_eq!(close_only, empty);
        assert_backwards_round_trip(&close_only);

        let mut move_only = RawPath::new();
        move_only.move_to(1.0, 2.0);
        assert_backwards_round_trip(&move_only);

        let mut empty_contours = RawPath::new();
        empty_contours.move_to(1.0, 2.0);
        empty_contours.move_to(3.0, 4.0);
        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&empty_contours, Mat2D::IDENTITY);
        assert_eq!(reversed.verbs(), &[PathVerb::Move, PathVerb::Move]);
        assert_eq!(
            reversed.points(),
            &[Vec2D::new(3.0, 4.0), Vec2D::new(1.0, 2.0)]
        );
        assert_backwards_round_trip(&empty_contours);
    }

    #[test]
    fn add_path_backwards_reverses_open_line_quad_and_cubic_segments() {
        let mut source = RawPath::new();
        source.move_to(1.0, 2.0);
        source.line_to(3.0, 4.0);
        source.quad_to(5.0, 6.0, 7.0, 8.0);
        source.cubic_to(9.0, 10.0, 11.0, 12.0, 13.0, 14.0);

        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&source, Mat2D::IDENTITY);

        assert_eq!(
            reversed.verbs(),
            &[
                PathVerb::Move,
                PathVerb::Cubic,
                PathVerb::Quad,
                PathVerb::Line
            ]
        );
        assert_eq!(
            reversed.points(),
            &[
                Vec2D::new(13.0, 14.0),
                Vec2D::new(11.0, 12.0),
                Vec2D::new(9.0, 10.0),
                Vec2D::new(7.0, 8.0),
                Vec2D::new(5.0, 6.0),
                Vec2D::new(3.0, 4.0),
                Vec2D::new(1.0, 2.0),
            ]
        );
    }

    #[test]
    fn add_path_backwards_reverses_contour_order_and_preserves_closes() {
        let mut source = RawPath::new();
        source.move_to(0.0, 0.0);
        source.line_to(1.0, 0.0);
        source.quad_to(2.0, 0.0, 3.0, 0.0);
        source.close();
        source.move_to(10.0, 0.0);
        source.cubic_to(11.0, 0.0, 12.0, 0.0, 13.0, 0.0);
        source.move_to(20.0, 0.0);
        source.line_to(21.0, 0.0);
        source.close();

        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&source, Mat2D::IDENTITY);

        assert_eq!(
            reversed.verbs(),
            &[
                PathVerb::Move,
                PathVerb::Line,
                PathVerb::Close,
                PathVerb::Move,
                PathVerb::Cubic,
                PathVerb::Move,
                PathVerb::Quad,
                PathVerb::Line,
                PathVerb::Close,
            ]
        );
        assert_eq!(
            reversed.points(),
            &[
                Vec2D::new(21.0, 0.0),
                Vec2D::new(20.0, 0.0),
                Vec2D::new(13.0, 0.0),
                Vec2D::new(12.0, 0.0),
                Vec2D::new(11.0, 0.0),
                Vec2D::new(10.0, 0.0),
                Vec2D::new(3.0, 0.0),
                Vec2D::new(2.0, 0.0),
                Vec2D::new(1.0, 0.0),
                Vec2D::new(0.0, 0.0),
            ]
        );
    }

    #[test]
    fn add_path_backwards_transforms_only_the_appended_reversed_path() {
        let mut source = RawPath::new();
        source.move_to(1.0, 2.0);
        source.line_to(3.0, 4.0);

        let mut destination = RawPath::new();
        destination.move_to(-1.0, -2.0);
        destination.add_path_backwards(&source, Mat2D([2.0, 0.0, 0.0, 3.0, 5.0, 7.0]));

        assert_eq!(
            destination.verbs(),
            &[PathVerb::Move, PathVerb::Move, PathVerb::Line]
        );
        assert_eq!(
            destination.points(),
            &[
                Vec2D::new(-1.0, -2.0),
                Vec2D::new(11.0, 19.0),
                Vec2D::new(7.0, 13.0),
            ]
        );
    }

    #[test]
    fn add_path_backwards_prunes_segments_collapsed_by_transform() {
        let mut source = RawPath::new();
        source.move_to(1.0, 2.0);
        source.line_to(3.0, 4.0);

        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&source, Mat2D([0.0, 0.0, 0.0, 0.0, 5.0, 7.0]));

        assert_eq!(reversed.verbs(), &[PathVerb::Move]);
        assert_eq!(reversed.points(), &[Vec2D::new(5.0, 7.0)]);
    }

    #[test]
    fn add_path_backwards_keeps_transformed_curves_with_distinct_controls() {
        let mut source = RawPath::new();
        source.move_to(0.0, 0.0);
        source.quad_to(1.0, 2.0, 0.0, 0.0);
        source.cubic_to(3.0, 4.0, 5.0, 6.0, 0.0, 0.0);

        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&source, Mat2D([2.0, 0.0, 0.0, 3.0, 5.0, 7.0]));

        assert_eq!(
            reversed.verbs(),
            &[PathVerb::Move, PathVerb::Cubic, PathVerb::Quad]
        );
        assert_eq!(
            reversed.points(),
            &[
                Vec2D::new(5.0, 7.0),
                Vec2D::new(15.0, 25.0),
                Vec2D::new(11.0, 19.0),
                Vec2D::new(5.0, 7.0),
                Vec2D::new(7.0, 13.0),
                Vec2D::new(5.0, 7.0),
            ]
        );
    }

    #[test]
    fn add_path_backwards_prunes_fully_collapsed_transformed_curves() {
        let mut source = RawPath::new();
        source.move_to(1.0, 2.0);
        source.quad_to(3.0, 4.0, 5.0, 6.0);
        source.cubic_to(7.0, 8.0, 9.0, 10.0, 11.0, 12.0);

        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&source, Mat2D([0.0, 0.0, 0.0, 0.0, 5.0, 7.0]));

        assert_eq!(reversed.verbs(), &[PathVerb::Move]);
        assert_eq!(reversed.points(), &[Vec2D::new(5.0, 7.0)]);
    }

    #[test]
    fn recording_serializer_matches_cpp_smoke_stream() {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let mut path = factory.make_empty_render_path();
        let mut paint = factory.make_render_paint();

        path.move_to(0.0, 0.0);
        path.line_to(10.0, 0.0);
        path.line_to(10.0, 10.0);
        path.close();
        paint.color(0xff336699);

        factory.source("smoke", "", "manual");
        factory.frame_size(64, 64);
        factory.add_sample(0.0);
        renderer.save();
        renderer.draw_path(path.as_ref(), paint.as_ref());
        renderer.restore();
        factory.add_frame();

        assert_eq!(
            factory.stream(),
            concat!(
                "rive-golden-stream-v1\n",
                "makeEmptyRenderPath {id=1,fillRule=0,path={verbs=[],points=[]}}\n",
                "makeRenderPaint {id=1,style=fill,color=0xff000000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}\n",
                "source file=\"smoke\" artboard=\"\" scene=\"manual\"\n",
                "frameSize width=64 height=64\n",
                "sample seconds=0\n",
                "save\n",
                "drawPath path={id=1,fillRule=0,path={verbs=[move,line,line,close],points=[(0,0),(10,0),(10,10)]}} paint={id=1,style=fill,color=0xff336699,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}\n",
                "restore\n",
                "frame\n",
            )
        );
    }

    #[test]
    fn sriv_serializer_matches_cpp_smoke_stream() {
        let mut factory = SerializingFactory::new();
        let mut renderer = factory.make_renderer();
        let path = factory.make_empty_render_path();
        let mut paint = factory.make_render_paint();

        paint.color(0xff336699);
        factory.frame_size(64, 64);
        renderer.save();
        renderer.draw_path(path.as_ref(), paint.as_ref());
        renderer.restore();
        factory.add_frame();

        assert_eq!(
            &*factory.bytes(),
            &[
                b'S', b'R', b'I', b'V', 1, // header/version
                3, 0, // makeRenderPath 0
                5, 0, // makeRenderPaint 0
                21, 0, 0x99, 0xcd, 0xcd, 0xf9, 0x0f, // color
                29, 64, 64, // frameSize
                7,  // save
                10, 0, 0,  // drawPath
                8,  // restore
                28, // frame
            ]
        );
    }

    #[test]
    fn sriv_serializer_does_not_emit_constructor_fill_rule() {
        let mut factory = SerializingFactory::new();
        let mut raw_path = RawPath::new();
        raw_path.move_to(1.0, 2.0);

        let _path = factory.make_render_path(raw_path, FillRule::EvenOdd);

        assert_eq!(&factory.bytes()[5..8], &[3, 0, 16]);
    }

    #[test]
    fn p3g_factory_aabb_helper_builds_the_pinned_nonzero_rectangle_path() {
        let mut factory = RecordingFactory::new();

        let _path = factory.make_render_path_from_aabb(Aabb::new(-2.0, 3.0, 5.0, 11.0));

        assert_eq!(
            factory.stream(),
            concat!(
                "rive-golden-stream-v1\n",
                "makeRenderPath {id=1,fillRule=0,path={verbs=[move,line,line,line,close],points=[(-2,3),(5,3),(5,11),(-2,11)]}}\n",
            )
        );
    }

    #[test]
    fn p3g_factory_font_helper_validates_and_owns_the_encoded_font() {
        let mut factory = NullFactory::new();
        let encoded = include_bytes!("../../nuxie/tests/fixtures/roboto-a.ttf.base64")
            .iter()
            .copied()
            .filter(|byte| !byte.is_ascii_whitespace())
            .collect::<Vec<_>>();
        let mut bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .expect("fixture font base64 decodes");
        let expected = bytes.clone();

        let font = factory.decode_font(&bytes).expect("valid font decodes");
        bytes.fill(0);

        assert_eq!(font.bytes(), expected);
        assert_eq!(factory.decode_font(b"not a font"), Err(FontDecodeError));
    }

    #[test]
    fn p3g_renderer_transform_helpers_emit_the_pinned_matrices() {
        let factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let radians = 0.5_f32;

        renderer.translate(3.0, -4.0);
        renderer.scale(2.0, 5.0);
        renderer.rotate(radians);

        assert_eq!(
            factory.stream(),
            format!(
                concat!(
                    "rive-golden-stream-v1\n",
                    "transform matrix=[1,0,0,1,3,-4]\n",
                    "transform matrix=[2,0,0,5,0,0]\n",
                    "transform matrix={}\n",
                ),
                mat_to_string(Mat2D([
                    radians.cos(),
                    radians.sin(),
                    -radians.sin(),
                    radians.cos(),
                    0.0,
                    0.0,
                ])),
            )
        );
    }

    #[test]
    fn p3g_compute_alignment_matches_every_pinned_fit_case() {
        let frame = Aabb::new(10.0, 20.0, 210.0, 120.0);
        let content = Aabb::new(-5.0, -10.0, 45.0, 10.0);
        let alignment = Vec2D::new(0.0, 0.0);

        assert_eq!(
            [
                compute_alignment(Fit::Fill, alignment, frame, content, 0.5),
                compute_alignment(Fit::Contain, alignment, frame, content, 0.5),
                compute_alignment(Fit::Cover, alignment, frame, content, 0.5),
                compute_alignment(Fit::FitWidth, alignment, frame, content, 0.5),
                compute_alignment(Fit::FitHeight, alignment, frame, content, 0.5),
                compute_alignment(Fit::None, alignment, frame, content, 0.5),
                compute_alignment(Fit::ScaleDown, alignment, frame, content, 0.5),
                compute_alignment(Fit::Layout, alignment, frame, content, 0.5),
            ],
            [
                Mat2D([4.0, 0.0, 0.0, 5.0, 30.0, 70.0]),
                Mat2D([4.0, 0.0, 0.0, 4.0, 30.0, 70.0]),
                Mat2D([5.0, 0.0, 0.0, 5.0, 10.0, 70.0]),
                Mat2D([4.0, 0.0, 0.0, 4.0, 30.0, 70.0]),
                Mat2D([5.0, 0.0, 0.0, 5.0, 10.0, 70.0]),
                Mat2D([1.0, 0.0, 0.0, 1.0, 90.0, 70.0]),
                Mat2D([1.0, 0.0, 0.0, 1.0, 90.0, 70.0]),
                Mat2D([0.5, 0.0, 0.0, 0.5, 100.0, 70.0]),
            ]
        );
    }

    #[test]
    fn p3g_glyph_run_annotations_match_pinned_break_and_joiner_rules() {
        let text = "a b\nc\u{2028}d\u{200b}e\u{2060}f"
            .chars()
            .collect::<Vec<_>>();
        let text_indices = (0..text.len() as u32).collect::<Vec<_>>();

        let annotations = annotate_glyph_runs(&text, &[&text_indices])
            .expect("shaper-produced indices address the source text");

        assert_eq!(
            annotations,
            vec![GlyphRunAnnotations {
                breaks: vec![0, 1, 2, 3, 3, 3, 4, 5, 5, 5, 6, 7, 8, 11],
                joiners: vec![9],
            }]
        );
    }

    #[test]
    fn records_buffers_gradients_images_and_meshes() {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let shader = factory.make_linear_gradient(
            0.0,
            0.5,
            10.0,
            20.0,
            &[0xff000000, 0xffffffff],
            &[0.0, 1.0],
        );
        let mut paint = factory.make_render_paint();
        paint.shader(Some(shader.as_ref()));
        let image = factory.decode_image(&[1, 2, 3]).expect("image decodes");
        let mut vertices = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MappedOnceAtInitialization,
            4,
        );
        vertices.map_mut().copy_from_slice(&[1, 2, 3, 4]);
        vertices.unmap();

        renderer.draw_image(
            Some(image.as_ref()),
            ImageSampler {
                wrap_x: ImageWrap::Repeat,
                wrap_y: ImageWrap::Mirror,
                filter: ImageFilter::Nearest,
            },
            BlendMode::Multiply,
            0.5,
        );
        renderer.draw_image_mesh(
            Some(image.as_ref()),
            ImageSampler::LINEAR_CLAMP,
            Some(vertices.as_ref()),
            None,
            None,
            2,
            3,
            BlendMode::SrcOver,
            1.0,
        );

        let stream = factory.stream();
        assert!(stream.contains(
            "makeLinearGradient id=1 start=(0,0.5) end=(10,20) stops=[{color=0xff000000,stop=0},{color=0xffffffff,stop=1}]\n"
        ));
        assert!(stream.contains("makeRenderPaint {id=1,style=fill,color=0xff000000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}\n"));
        assert!(stream.contains("decodeImage id=1 width=0 height=0 data=010203\n"));
        assert!(stream.contains("makeRenderBuffer id=1 type=1 flags=1 size=4\n"));
        assert!(stream.contains("bufferData id=1 type=1 size=4 data=01020304\n"));
        assert!(stream.contains(
            "drawImage image=1 sampler={wrapX=1,wrapY=2,filter=1,key=16} blendMode=24 opacity=0.5\n"
        ));
        assert!(stream.contains(
            "drawImageMesh image=1 sampler={wrapX=0,wrapY=0,filter=0,key=0} vertices=1 uvs=0 indices=0 vertexCount=2 indexCount=3 blendMode=3 opacity=1\n"
        ));
    }

    #[test]
    fn pure_rust_float_formatter_pins_cpp_defaultfloat_boundaries() {
        let cases = [
            (0x0000_0000, "0"),
            (0x8000_0000, "-0"),
            (0x3dcc_cccd, "0.100000001"),
            (0x0000_0001, "1.40129846e-45"),
            (0x007f_ffff, "1.17549421e-38"),
            (0x0080_0000, "1.17549435e-38"),
            (0x7f7f_ffff, "3.40282347e+38"),
            (0x38d1_b716, "9.99999902e-05"),
            (0x38d1_b717, "9.99999975e-05"),
            (0x38d1_b718, "0.000100000005"),
            (0x4e6e_6b27, "999999936"),
            (0x4e6e_6b28, "1e+09"),
            (0x4e6e_6b29, "1.00000006e+09"),
            (0x411f_ffff, "9.99999905"),
            (0x3d4c_cccd, "0.0500000007"),
            (0x43c0_2f80, "384.371094"),
            (0x7f80_0000, "inf"),
            (0xff80_0000, "-inf"),
            (0x7fc0_0000, "nan"),
            (0xffc0_0000, "nan"),
        ];
        for (bits, expected) in cases {
            assert_eq!(float_to_string(f32::from_bits(bits)), expected);
        }
    }

    #[test]
    fn pure_rust_float_formatter_matches_c_oracle_corpus_digest() {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        let mut count = 0_u64;
        for_float_formatter_corpus(|value| {
            let formatted = float_to_string(value);
            for byte in formatted.bytes().chain(std::iter::once(0xff)) {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
            }
            count += 1;
        });
        assert_eq!(count, 1_050_928);
        assert_eq!(hash, 0x3e76_805d_71c7_9904);
    }

    fn for_float_formatter_corpus(mut visit: impl FnMut(f32)) {
        let boundary_bits = [
            0x0000_0000,
            0x8000_0000,
            0x0000_0001,
            0x007f_ffff,
            0x0080_0000,
            0x3dcc_cccd,
            0x7f7f_ffff,
            0xff7f_ffff,
            0x7f80_0000,
            0xff80_0000,
            0x7fc0_0000,
            0xffc0_0000,
        ];
        for bits in boundary_bits {
            visit(f32::from_bits(bits));
        }

        for center in [
            1e-5_f32,
            1e-4_f32,
            0.1_f32,
            1.0_f32,
            9.999_999_f32,
            1e8_f32,
            1e9_f32,
            f32::MIN_POSITIVE,
            f32::MAX,
        ] {
            let center = center.to_bits();
            for distance in 0..=64 {
                let below = center.saturating_sub(distance);
                let above = center.saturating_add(distance);
                visit(f32::from_bits(below));
                visit(f32::from_bits(above));
                visit(f32::from_bits(below | 0x8000_0000));
                visit(f32::from_bits(above | 0x8000_0000));
            }
        }

        let mut bits = 0x243f_6a88_u32;
        for _ in 0..1_048_576 {
            bits ^= bits << 13;
            bits ^= bits >> 17;
            bits ^= bits << 5;
            visit(f32::from_bits(bits));
        }
    }
}
