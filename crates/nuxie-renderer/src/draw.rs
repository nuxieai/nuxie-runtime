//! CPU path preparation translated from `renderer/src/draw.cpp`.

use crate::gpu::{
    AtlasTransform, ContourData, CoverageBufferRange, PathData, TessVertexSpan, TriangleVertex,
    BEVEL_JOIN_CONTOUR_FLAG, CONTOUR_ID_MASK, CULL_EXCESS_TESSELLATION_SEGMENTS_CONTOUR_FLAG,
    EMULATED_STROKE_CAP_CONTOUR_FLAG, FEATHER_JOIN_CONTOUR_FLAG, MAX_PARAMETRIC_SEGMENTS,
    MIDPOINT_FAN_PATCH_SEGMENT_SPAN, MITER_CLIP_JOIN_CONTOUR_FLAG, MITER_REVERT_JOIN_CONTOUR_FLAG,
    NEGATE_PATH_FILL_COVERAGE_FLAG, OUTER_CURVE_PATCH_SEGMENT_SPAN, PARAMETRIC_PRECISION,
    POLAR_PRECISION, RETROFITTED_TRIANGLE_CONTOUR_FLAG, ROUND_JOIN_CONTOUR_FLAG,
    TESS_TEXTURE_WIDTH,
};
use crate::gr_triangulator::{InnerFanTriangulator, SweepDirection, WindingFaces};
use bytemuck::Zeroable;
use nuxie_render_api::{FillRule, Mat2D, PathVerb, RawPath, StrokeCap, StrokeJoin, Vec2D};
use smallvec::SmallVec;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Contour {
    pub points: Vec<Vec2D>,
    pub closed: bool,
}

pub(crate) struct FillTessellation {
    pub spans: Vec<TessVertexSpan>,
    pub path: PathData,
    pub contours: Vec<ContourData>,
    pub base_instance: u32,
    pub instance_count: u32,
}

impl Clone for FillTessellation {
    fn clone(&self) -> Self {
        #[cfg(test)]
        FILL_TESSELLATION_CLONE_COUNT.with(|count| count.set(count.get() + 1));
        Self {
            spans: self.spans.clone(),
            path: self.path,
            contours: self.contours.clone(),
            base_instance: self.base_instance,
            instance_count: self.instance_count,
        }
    }
}

pub(crate) struct StrokeTessellation {
    pub tessellation: FillTessellation,
    pub local_contour_ids_are_dense: bool,
}

pub(crate) struct InteriorTessellation {
    pub spans: Vec<TessVertexSpan>,
    pub path: PathData,
    pub contours: Vec<ContourData>,
    pub triangles: Vec<TriangleVertex>,
    pub max_triangle_vertex_count: usize,
    pub base_instance: u32,
    pub instance_count: u32,
}

impl Clone for InteriorTessellation {
    fn clone(&self) -> Self {
        #[cfg(test)]
        INTERIOR_TESSELLATION_CLONE_COUNT.with(|count| count.set(count.get() + 1));
        Self {
            spans: self.spans.clone(),
            path: self.path,
            contours: self.contours.clone(),
            triangles: self.triangles.clone(),
            max_triangle_vertex_count: self.max_triangle_vertex_count,
            base_instance: self.base_instance,
            instance_count: self.instance_count,
        }
    }
}

impl InteriorTessellation {
    pub(crate) fn visit_triangles(
        &self,
        path_id: u16,
        faces: WindingFaces,
        mut visit: impl FnMut(i16, [TriangleVertex; 3]),
    ) {
        #[cfg(test)]
        INTERIOR_TRIANGLE_VISIT_COUNT.with(|count| count.set(count.get() + 1));
        for triangle in self.triangles.chunks_exact(3) {
            let weight = i16::try_from(triangle[0].weight_path_id >> 16)
                .expect("interior triangle winding fits i16");
            debug_assert!(triangle
                .iter()
                .all(|vertex| vertex.weight_path_id >> 16 == i32::from(weight)));
            if !faces.includes(weight) {
                continue;
            }
            let mut emitted = [triangle[0], triangle[1], triangle[2]];
            for vertex in &mut emitted {
                vertex.weight_path_id = (vertex.weight_path_id & !0xffff) | i32::from(path_id);
            }
            visit(weight, emitted);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FeatherFillDirection {
    Forward,
    Reverse,
    ReverseThenForward,
    ForwardThenReverse,
}

pub(crate) fn feather_atlas_fill_direction(
    transform: Mat2D,
    fill_rule: FillRule,
    is_stroke: bool,
) -> FeatherFillDirection {
    let [xx, yx, xy, yy, _, _] = transform.0;
    if !is_stroke && fill_rule == FillRule::Clockwise && xx * yy - xy * yx < 0.0 {
        FeatherFillDirection::Reverse
    } else {
        FeatherFillDirection::Forward
    }
}

#[derive(Clone)]
struct StrokeCurve {
    cubic: [Vec2D; 4],
    is_line: bool,
}

struct StrokeContour {
    curves: SmallVec<[StrokeCurve; 1]>,
    first: Vec2D,
    current: Vec2D,
    closed: bool,
}

#[derive(Clone, Copy)]
struct PreparedStrokeCurve {
    cubic: [Vec2D; 4],
    tangents: [Vec2D; 2],
    original_start_tangent: Vec2D,
    parametric_segments: u32,
    polar_segments: u32,
    ends_original_curve: bool,
}

type PendingStrokeCurve = ([Vec2D; 4], Vec2D, u32, u32, u32, u32);

#[derive(Default)]
pub(crate) struct StrokePreparationScratch {
    // Keep contour slots alive so both the outer Vec and spilled per-contour
    // SmallVec storage survive until the frame ends.
    contours: Vec<StrokeContour>,
    prepared: Vec<PreparedStrokeCurve>,
    pending: Vec<PendingStrokeCurve>,
    #[cfg(test)]
    stats: StrokePreparationStats,
}

impl StrokePreparationScratch {
    pub(crate) fn retained_capacity_bytes(&self) -> usize {
        let contour_bytes = self
            .contours
            .capacity()
            .saturating_mul(std::mem::size_of::<StrokeContour>());
        let spilled_curve_bytes = self
            .contours
            .iter()
            .filter(|contour| contour.curves.spilled())
            .map(|contour| {
                contour
                    .curves
                    .capacity()
                    .saturating_mul(std::mem::size_of::<StrokeCurve>())
            })
            .fold(0usize, usize::saturating_add);
        contour_bytes
            .saturating_add(spilled_curve_bytes)
            .saturating_add(
                self.prepared
                    .capacity()
                    .saturating_mul(std::mem::size_of::<PreparedStrokeCurve>()),
            )
            .saturating_add(
                self.pending
                    .capacity()
                    .saturating_mul(std::mem::size_of::<PendingStrokeCurve>()),
            )
    }

    pub(crate) fn reset_for_reuse(&mut self) {
        for contour in &mut self.contours {
            contour.curves.clear();
        }
        self.prepared.clear();
        self.pending.clear();
    }
}

#[cfg(test)]
impl StrokePreparationScratch {
    pub(crate) fn stats(&self) -> StrokePreparationStats {
        self.stats
    }

    pub(crate) fn reserve_retained_bytes_for_test(&mut self, bytes: usize) {
        let entries = (bytes / std::mem::size_of::<PendingStrokeCurve>()).saturating_add(1);
        self.pending.reserve_exact(entries);
    }
}

#[cfg(test)]
pub(crate) fn build_stroke_tessellation(
    path: &RawPath,
    transform: Mat2D,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
) -> Option<FillTessellation> {
    build_stroke_tessellation_with_layout(path, transform, thickness, join, cap)
        .map(|built| built.tessellation)
}

#[allow(dead_code)] // Retain the fresh-scratch entry point for standalone callers.
pub(crate) fn build_stroke_tessellation_with_layout(
    path: &RawPath,
    transform: Mat2D,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
) -> Option<StrokeTessellation> {
    let mut scratch = StrokePreparationScratch::default();
    build_stroke_tessellation_with_layout_using_scratch(
        path,
        transform,
        thickness,
        join,
        cap,
        &mut scratch,
    )
}

pub(crate) fn build_stroke_tessellation_with_layout_using_scratch(
    path: &RawPath,
    transform: Mat2D,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    scratch: &mut StrokePreparationScratch,
) -> Option<StrokeTessellation> {
    build_stroke_or_feather_tessellation_using_scratch(
        path,
        transform,
        Some((thickness, join, cap)),
        0.0,
        scratch,
    )
}

pub(crate) fn build_feather_tessellation(
    path: &RawPath,
    transform: Mat2D,
    paint_feather: f32,
    stroke: Option<(f32, StrokeJoin, StrokeCap)>,
) -> Option<FillTessellation> {
    build_feather_tessellation_with_direction(
        path,
        transform,
        paint_feather,
        stroke,
        FeatherFillDirection::ReverseThenForward,
    )
}

#[cfg(test)]
pub(crate) fn build_feather_atlas_tessellation(
    path: &RawPath,
    transform: Mat2D,
    paint_feather: f32,
    stroke: Option<(f32, StrokeJoin, StrokeCap)>,
) -> Option<FillTessellation> {
    build_feather_tessellation_with_direction(
        path,
        transform,
        paint_feather,
        stroke,
        FeatherFillDirection::Forward,
    )
}

pub(crate) fn build_feather_tessellation_with_direction(
    path: &RawPath,
    transform: Mat2D,
    paint_feather: f32,
    stroke: Option<(f32, StrokeJoin, StrokeCap)>,
    fill_direction: FeatherFillDirection,
) -> Option<FillTessellation> {
    let feather_radius = paint_feather * 1.5;
    if feather_radius <= 0.0 || !feather_radius.is_finite() {
        return None;
    }
    let mut tessellation =
        build_stroke_or_feather_tessellation(path, transform, stroke, paint_feather)?;
    if stroke.is_none() {
        match fill_direction {
            FeatherFillDirection::Forward => {}
            FeatherFillDirection::Reverse => {
                tessellation.tessellation.make_single_sided_reverse(true)
            }
            FeatherFillDirection::ReverseThenForward => {
                tessellation.tessellation.make_double_sided()
            }
            FeatherFillDirection::ForwardThenReverse => {
                tessellation
                    .tessellation
                    .make_double_sided_with_direction(true);
            }
        }
    }
    Some(tessellation.tessellation)
}

pub(crate) fn feather_requires_atlas(
    paint_feather: f32,
    transform: Mat2D,
    force_atlas: bool,
) -> bool {
    force_atlas || feather_atlas_scale(paint_feather, transform) <= 0.5
}

pub(crate) fn feather_atlas_scale(paint_feather: f32, transform: Mat2D) -> f32 {
    let device_radius = paint_feather * 1.5 * max_matrix_scale(transform);
    16.0 / device_radius.max(16.0)
}

pub(crate) fn feather_pixel_bounds(
    path: &RawPath,
    transform: Mat2D,
    paint_feather: f32,
    stroke: Option<(f32, StrokeJoin, StrokeCap)>,
) -> Option<[i32; 4]> {
    let matrix_scale = max_matrix_scale(transform);
    let softened_path = (stroke.is_none()
        && feather_fill_requires_softening(paint_feather, matrix_scale))
    .then(|| softened_path_for_feathering(path, paint_feather * 1.5, matrix_scale));
    let path = softened_path.as_ref().unwrap_or(path);
    let (min, max) = transformed_control_bounds(path, transform)?;
    let mut radius = stroke.map_or(0.0, |(thickness, join, cap)| {
        let stroke_radius = thickness * 0.5;
        if join == StrokeJoin::Miter {
            stroke_radius * 4.0
        } else if cap == StrokeCap::Square {
            stroke_radius * std::f32::consts::SQRT_2
        } else {
            stroke_radius
        }
    });
    radius += paint_feather * 1.5;
    let [xx, yx, xy, yy, _, _] = transform.0;
    let outset_x = radius * (xx.abs() + xy.abs()) + 1.0;
    let outset_y = radius * (yx.abs() + yy.abs()) + 1.0;
    Some([
        (min.x - outset_x).floor() as i32,
        (min.y - outset_y).floor() as i32,
        (max.x + outset_x).ceil() as i32,
        (max.y + outset_y).ceil() as i32,
    ])
}

pub(crate) fn path_pixel_bounds(path: &RawPath, transform: Mat2D) -> Option<[i32; 4]> {
    let (min, max) = transformed_control_bounds(path, transform)?;
    Some([
        min.x.floor() as i32,
        min.y.floor() as i32,
        max.x.ceil() as i32,
        max.y.ceil() as i32,
    ])
}

fn transformed_control_bounds(path: &RawPath, transform: Mat2D) -> Option<(Vec2D, Vec2D)> {
    let mut min = Vec2D::new(f32::INFINITY, f32::INFINITY);
    let mut max = Vec2D::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    for point in path.points() {
        let point = transform.transform_point(*point);
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }
    if !min.x.is_finite() || !min.y.is_finite() || !max.x.is_finite() || !max.y.is_finite() {
        return None;
    }
    Some((min, max))
}

pub(crate) fn clockwise_atomic_coverage_range(
    path: &RawPath,
    transform: Mat2D,
    viewport_width: u32,
    viewport_height: u32,
    offset: usize,
) -> Option<(CoverageBufferRange, usize)> {
    let bounds = path_pixel_bounds(path, transform)?;
    clockwise_atomic_coverage_range_from_bounds(bounds, viewport_width, viewport_height, offset)
}

pub(crate) fn clockwise_atomic_coverage_range_from_bounds(
    [left, top, right, bottom]: [i32; 4],
    viewport_width: u32,
    viewport_height: u32,
    offset: usize,
) -> Option<(CoverageBufferRange, usize)> {
    const PADDING: i32 = 2;
    const TILE_SIZE: u32 = 32;

    let left = left.clamp(0, viewport_width as i32);
    let top = top.clamp(0, viewport_height as i32);
    let right = right.clamp(0, viewport_width as i32);
    let bottom = bottom.clamp(0, viewport_height as i32);
    if right <= left || bottom <= top {
        return None;
    }
    let padded_width = u32::try_from(right - left + PADDING * 2).ok()?;
    let padded_height = u32::try_from(bottom - top + PADDING * 2).ok()?;
    let coverage_width = padded_width.checked_next_multiple_of(TILE_SIZE)?;
    let coverage_height = padded_height.checked_next_multiple_of(TILE_SIZE)?;
    let word_count = (coverage_width as usize).checked_mul(coverage_height as usize)?;
    Some((
        CoverageBufferRange {
            offset: u32::try_from(offset).ok()?,
            pitch: coverage_width,
            offset_x: (-left + PADDING) as f32,
            offset_y: (-top + PADDING) as f32,
        },
        word_count,
    ))
}

fn build_stroke_or_feather_tessellation(
    path: &RawPath,
    transform: Mat2D,
    stroke: Option<(f32, StrokeJoin, StrokeCap)>,
    paint_feather: f32,
) -> Option<StrokeTessellation> {
    let mut scratch = StrokePreparationScratch::default();
    build_stroke_or_feather_tessellation_using_scratch(
        path,
        transform,
        stroke,
        paint_feather,
        &mut scratch,
    )
}

fn build_stroke_or_feather_tessellation_using_scratch(
    path: &RawPath,
    transform: Mat2D,
    stroke: Option<(f32, StrokeJoin, StrokeCap)>,
    paint_feather: f32,
    scratch: &mut StrokePreparationScratch,
) -> Option<StrokeTessellation> {
    let matrix_scale = max_matrix_scale(transform);
    let feather_radius = paint_feather * 1.5;
    let softened_path = (stroke.is_none()
        && feather_fill_requires_softening(paint_feather, matrix_scale))
    .then(|| softened_path_for_feathering(path, feather_radius, matrix_scale));
    let path = softened_path.as_ref().unwrap_or(path);
    #[cfg(test)]
    let StrokePreparationScratch {
        contours,
        prepared,
        pending,
        stats,
    } = scratch;
    #[cfg(not(test))]
    let StrokePreparationScratch {
        contours,
        prepared,
        pending,
    } = scratch;
    prepared.clear();
    pending.clear();
    #[cfg(test)]
    let contour_capacity_before = contours.capacity();
    let contour_count = stroke_contours(path, contours)?;
    #[cfg(test)]
    let contour_capacity_grew = contours.capacity() != contour_capacity_before;
    let contours = &mut contours[..contour_count];
    let is_stroke = stroke.is_some();
    if !is_stroke {
        for contour in &mut *contours {
            contour.closed = true;
        }
    }
    let (thickness, join, cap) = stroke.unwrap_or((0.0, StrokeJoin::Bevel, StrokeCap::Butt));
    let stroke_radius = thickness * 0.5;
    if (is_stroke && stroke_radius <= 0.0) || contours.is_empty() {
        return None;
    }
    for contour in &mut *contours {
        normalize_stroke_contour_curves(contour);
    }
    let feather_screen_radius = (feather_radius * matrix_scale).min(feather_max_screen_radius());
    let parametric_precision = if feather_radius > 1.0 {
        (PARAMETRIC_PRECISION as f32 * 100.0 / (feather_radius * matrix_scale))
            .min(PARAMETRIC_PRECISION as f32)
    } else {
        PARAMETRIC_PRECISION as f32
    };
    let polar_segments_per_radian =
        polar_segments_per_radian(feather_screen_radius + stroke_radius * matrix_scale);
    let cap_segments = match cap {
        StrokeCap::Round => ((polar_segments_per_radian * std::f32::consts::PI).ceil() + 2.0)
            .min(crate::gpu::MAX_POLAR_SEGMENTS as f32) as u32,
        StrokeCap::Butt | StrokeCap::Square => 5,
    };
    let cap_flags = match cap {
        StrokeCap::Butt => BEVEL_JOIN_CONTOUR_FLAG,
        StrokeCap::Round => ROUND_JOIN_CONTOUR_FLAG,
        StrokeCap::Square => MITER_CLIP_JOIN_CONTOUR_FLAG,
    } | EMULATED_STROKE_CAP_CONTOUR_FLAG;
    let join_flags = if is_stroke {
        match join {
            StrokeJoin::Miter => MITER_REVERT_JOIN_CONTOUR_FLAG,
            StrokeJoin::Round => ROUND_JOIN_CONTOUR_FLAG,
            StrokeJoin::Bevel => BEVEL_JOIN_CONTOUR_FLAG,
        }
    } else {
        FEATHER_JOIN_CONTOUR_FLAG
    };
    let feather_join_segments = ((polar_segments_per_radian * std::f32::consts::PI).ceil() + 4.0)
        .clamp(6.0, crate::gpu::MAX_POLAR_SEGMENTS as f32) as u32;
    // Match C++ PathDraw::initForMidpointFan's path-wide sizing while retaining
    // the largest allocation in the frame scratch. A stroked cubic can produce
    // at most five convex/180-degree pieces.
    let prepared_capacity = contours
        .iter()
        .map(|contour| {
            contour
                .curves
                .len()
                .checked_mul(5)
                .expect("stroke preparation capacity overflow")
        })
        .max()
        .unwrap_or(0);
    let pending_capacity = prepared_capacity
        .checked_add(1)
        .expect("stroke pending capacity overflow")
        .max(2);
    #[cfg(test)]
    let prepared_capacity_before = prepared.capacity();
    #[cfg(test)]
    let pending_capacity_before = pending.capacity();
    prepared.reserve(prepared_capacity);
    pending.reserve(pending_capacity);
    #[cfg(test)]
    {
        stats.builds += 1;
        stats.contours += contours.len();
        stats.inline_one_curve_contours += contours
            .iter()
            .filter(|contour| contour.curves.len() == 1 && !contour.curves.spilled())
            .count();
        stats.spilled_curve_contours += contours
            .iter()
            .filter(|contour| contour.curves.spilled())
            .count();
        stats.contour_capacity_growths += usize::from(contour_capacity_grew);
        stats.prepared_capacity_growths +=
            usize::from(prepared.capacity() != prepared_capacity_before);
        stats.pending_capacity_growths +=
            usize::from(pending.capacity() != pending_capacity_before);
    }
    let mut spans = Vec::new();
    let mut contour_data = Vec::with_capacity(contours.len());
    let mut local_contour_ids_are_dense = true;
    let mut location = MIDPOINT_FAN_PATCH_SEGMENT_SPAN as i32;
    push_padding_span(&mut spans, 0, location);
    let path_start = location;
    for (contour_index, contour) in contours.iter_mut().enumerate() {
        let curves = &contour.curves;
        let contour_start = location as u32;
        let contour_id = (contour_index as u32 + 1) & CONTOUR_ID_MASK;
        prepared.clear();
        pending.clear();
        if curves.is_empty() && !is_stroke {
            // There is a contour record but no span carrying this local ID.
            // Preserve the generic validation/fallback for this sparse layout.
            local_contour_ids_are_dense = false;
            contour_data.push(ContourData::new([f32::NAN, f32::NAN], 0, contour_start));
            continue;
        }
        if curves.is_empty() {
            let empty_cap = if contour.closed {
                match join {
                    StrokeJoin::Round => StrokeCap::Round,
                    StrokeJoin::Miter => StrokeCap::Square,
                    StrokeJoin::Bevel => StrokeCap::Butt,
                }
            } else {
                cap
            };
            if empty_cap == StrokeCap::Butt {
                continue;
            }
            let empty_cap_segments = match empty_cap {
                StrokeCap::Round => {
                    ((polar_segments_per_radian * std::f32::consts::PI).ceil() + 2.0)
                        .min(crate::gpu::MAX_POLAR_SEGMENTS as f32) as u32
                }
                StrokeCap::Square => 5,
                StrokeCap::Butt => unreachable!(),
            };
            let empty_cap_flags = (match empty_cap {
                StrokeCap::Round => ROUND_JOIN_CONTOUR_FLAG,
                StrokeCap::Square => MITER_CLIP_JOIN_CONTOUR_FLAG,
                StrokeCap::Butt => unreachable!(),
            }) | EMULATED_STROKE_CAP_CONTOUR_FLAG;
            for direction in [1.0, -1.0] {
                let pivot = Vec2D::new(contour.first.x + direction, contour.first.y);
                pending.push((
                    [pivot, pivot, pivot, contour.first],
                    Vec2D::new(direction, 0.0),
                    0,
                    0,
                    empty_cap_segments,
                    contour_id | empty_cap_flags,
                ));
            }
        } else {
            for curve in curves {
                if curve.is_line {
                    prepared.push(prepare_line_curve(curve));
                    continue;
                }
                let original_tangents = cubic_tangents(curve.cubic);
                let (roots, are_cusps) = find_cubic_convex_180_chops(curve.cubic);
                let chopped = if are_cusps {
                    chop_cubic_around_cusps(curve.cubic, &roots, matrix_scale)
                } else {
                    chop_cubic_at_values(curve.cubic, &roots)
                };
                let chopped_count = chopped.len();
                for (index, cubic) in chopped.into_iter().enumerate() {
                    let tangents = cubic_tangents(cubic);
                    let (parametric_segments, polar_segments) = (
                        cubic_segment_count_with_precision_and_transform(
                            cubic,
                            parametric_precision,
                            transform,
                        ),
                        if is_stroke {
                            round_join_segment_count(
                                tangents[0],
                                tangents[1],
                                polar_segments_per_radian,
                            )
                        } else {
                            1
                        },
                    );
                    prepared.push(PreparedStrokeCurve {
                        cubic,
                        tangents,
                        original_start_tangent: original_tangents[0],
                        parametric_segments,
                        polar_segments,
                        ends_original_curve: index + 1 == chopped_count,
                    });
                }
            }
            if !contour.closed {
                let PreparedStrokeCurve {
                    cubic, tangents, ..
                } = prepared[0];
                pending.push((
                    [cubic[3], cubic[2], cubic[1], cubic[0]],
                    tangents[0],
                    0,
                    0,
                    cap_segments,
                    contour_id | cap_flags,
                ));
            }
            let original_end_tangent = if let Some(curve) = curves.last() {
                if curve.is_line {
                    subtract(curve.cubic[3], curve.cubic[0])
                } else {
                    cubic_tangents(curve.cubic)[1]
                }
            } else {
                Vec2D::new(-1.0, 0.0)
            };
            let mut carried_join_tangent = Vec2D::new(0.0, 1.0);
            for (index, curve) in prepared.iter().copied().enumerate() {
                let final_open = !contour.closed && index + 1 == prepared.len();
                let (join_tangent, join_segments, flags) = if final_open {
                    carried_join_tangent = negate(original_end_tangent);
                    (carried_join_tangent, cap_segments, contour_id | cap_flags)
                } else if !curve.ends_original_curve {
                    (carried_join_tangent, 1, contour_id | join_flags)
                } else {
                    let next_tangent =
                        prepared[(index + 1) % prepared.len()].original_start_tangent;
                    carried_join_tangent = next_tangent;
                    let segment_count = if !is_stroke {
                        feather_join_segments
                    } else if join == StrokeJoin::Round {
                        round_join_segment_count(
                            curve.tangents[1],
                            next_tangent,
                            polar_segments_per_radian,
                        )
                    } else {
                        5
                    };
                    (carried_join_tangent, segment_count, contour_id | join_flags)
                };
                pending.push((
                    curve.cubic,
                    join_tangent,
                    curve.parametric_segments,
                    curve.polar_segments,
                    join_segments,
                    flags,
                ));
            }
        }
        let vertex_count = pending
            .iter()
            .map(|(_, _, parametric, polar, join, _)| parametric + polar + join - 1)
            .sum::<u32>() as i32;
        let padding = align_up(vertex_count, MIDPOINT_FAN_PATCH_SEGMENT_SPAN as i32) - vertex_count;
        let midpoint = if is_stroke {
            Vec2D::new(if contour.closed { 1.0 } else { 0.0 }, 0.0)
        } else {
            contour_midpoint(curves)
        };
        local_contour_ids_are_dense &=
            u32::try_from(contour_data.len() + 1).ok() == Some(contour_id);
        contour_data.push(ContourData::new([midpoint.x, midpoint.y], 0, contour_start));
        for (index, (curve, tangent, parametric, polar, join, flags)) in
            pending.drain(..).enumerate()
        {
            let x0 = location;
            location += parametric as i32 + polar as i32 + join as i32 - 1
                + i32::from(index == 0) * padding;
            push_forward_tessellation_spans(
                &mut spans,
                curve.map(|point| [point.x, point.y]),
                [tangent.x, tangent.y],
                x0,
                location,
                parametric,
                polar,
                join,
                flags,
            );
        }
    }
    if contour_data.is_empty() {
        return None;
    }
    // C++ LogicalFlush writes every padding span before PathDraw geometry.
    let geometry_spans = spans.split_off(1);
    push_midpoint_tail_padding(&mut spans, location);
    spans.extend(geometry_spans);
    Some(StrokeTessellation {
        tessellation: FillTessellation {
            spans,
            path: PathData::new(
                transform,
                stroke_radius,
                feather_radius,
                0,
                AtlasTransform::zeroed(),
                CoverageBufferRange::zeroed(),
            ),
            contours: contour_data,
            base_instance: 1,
            instance_count: (location - path_start) as u32 / MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32,
        },
        local_contour_ids_are_dense,
    })
}

fn normalize_stroke_contour_curves(contour: &mut StrokeContour) {
    contour.curves.retain(|curve| {
        let [p0, p1, p2, p3] = curve.cubic;
        !(points_equal(p0, p1) && points_equal(p1, p2) && points_equal(p2, p3))
    });
    if contour.closed && !same_point(contour.first, contour.current) {
        contour.curves.push(StrokeCurve {
            cubic: line_cubic(contour.current, contour.first),
            is_line: true,
        });
    }
}

#[inline]
fn prepare_line_curve(curve: &StrokeCurve) -> PreparedStrokeCurve {
    debug_assert!(curve.is_line);
    let tangent = subtract(curve.cubic[3], curve.cubic[0]);
    PreparedStrokeCurve {
        cubic: curve.cubic,
        tangents: [tangent, tangent],
        original_start_tangent: tangent,
        parametric_segments: 1,
        polar_segments: 1,
        ends_original_curve: true,
    }
}

fn feather_fill_requires_softening(paint_feather: f32, matrix_scale: f32) -> bool {
    paint_feather * matrix_scale > 1.0
}

pub(crate) fn softened_path_for_feathering(
    path: &RawPath,
    feather_radius: f32,
    matrix_scale: f32,
) -> RawPath {
    const POLAR_JOIN_PRECISION: f32 = 2.0;
    const MIN_POLAR_ANGLE: f32 = std::f32::consts::PI / 16.0;
    let radius = feather_radius * matrix_scale * 0.25;
    let cos_theta = 1.0 - (1.0 / POLAR_JOIN_PRECISION) / radius;
    let mut rotation_between_joins = 2.0 * cos_theta.max(-1.0).acos();
    if rotation_between_joins < MIN_POLAR_ANGLE {
        let delta = (MIN_POLAR_ANGLE - rotation_between_joins) * 5.0;
        rotation_between_joins = MIN_POLAR_ANGLE + delta * (delta * delta);
    }
    rotation_between_joins = rotation_between_joins.min(std::f32::consts::FRAC_PI_2);

    let mut softened = RawPath::new();
    let mut point_index = 0;
    let mut current = Vec2D::new(0.0, 0.0);
    for verb in path.verbs() {
        match verb {
            PathVerb::Move => {
                current = path.points()[point_index];
                softened.move_to(current.x, current.y);
                point_index += 1;
            }
            PathVerb::Line => {
                current = path.points()[point_index];
                softened.line_to(current.x, current.y);
                point_index += 1;
            }
            PathVerb::Quad => {
                let control = path.points()[point_index];
                let end = path.points()[point_index + 1];
                append_softened_cubic(
                    &mut softened,
                    [
                        current,
                        lerp(current, control, 2.0 / 3.0),
                        lerp(end, control, 2.0 / 3.0),
                        end,
                    ],
                    rotation_between_joins,
                );
                current = end;
                point_index += 2;
            }
            PathVerb::Cubic => {
                let cubic = [
                    current,
                    path.points()[point_index],
                    path.points()[point_index + 1],
                    path.points()[point_index + 2],
                ];
                append_softened_cubic(&mut softened, cubic, rotation_between_joins);
                current = cubic[3];
                point_index += 3;
            }
            PathVerb::Close => softened.close(),
        }
    }
    softened
}

fn append_softened_cubic(path: &mut RawPath, cubic: [Vec2D; 4], rotation_between_joins: f32) {
    const CUSP_PADDING: f32 = 1e-2;
    let (roots, are_cusps) = find_cubic_convex_180_chops(cubic);
    let roots = if are_cusps {
        let mut straddles = Vec::with_capacity(roots.len() * 2);
        for (index, root) in roots.iter().copied().enumerate() {
            let min_t = if index == 0 {
                0.0
            } else {
                (roots[index - 1] + root) * 0.5
            };
            let max_t = if index + 1 == roots.len() {
                1.0
            } else {
                (roots[index + 1] + root) * 0.5
            };
            straddles.extend([
                (root - CUSP_PADDING).max(min_t),
                (root + CUSP_PADDING).min(max_t),
            ]);
        }
        straddles
    } else {
        roots
    };
    for (index, segment) in chop_cubic_at_values(cubic, &roots).into_iter().enumerate() {
        if are_cusps && index & 1 == 1 {
            path.line_to(segment[3].x, segment[3].y);
            continue;
        }
        append_cubic_at_uniform_rotation(path, segment, rotation_between_joins);
    }
}

fn append_cubic_at_uniform_rotation(
    path: &mut RawPath,
    cubic: [Vec2D; 4],
    rotation_between_joins: f32,
) {
    let tangents = cubic_tangents(cubic);
    let rotation = angle_between(tangents[0], tangents[1]);
    let chop_count = (rotation / rotation_between_joins) as usize;
    if chop_count == 0 {
        path.cubic_to(
            cubic[1].x, cubic[1].y, cubic[2].x, cubic[2].y, cubic[3].x, cubic[3].y,
        );
        return;
    }

    let mut turn = vector_cross(subtract(cubic[2], cubic[0]), subtract(cubic[3], cubic[1]));
    if turn == 0.0 {
        turn = vector_cross(tangents[0], tangents[1]);
    }
    let signed_rotation = if turn >= 0.0 {
        rotation_between_joins
    } else {
        -rotation_between_joins
    };
    let sin_rotation = signed_rotation.sin();
    let cos_rotation = signed_rotation.cos();
    let c = subtract(cubic[1], cubic[0]);
    let d = subtract(cubic[2], cubic[1]);
    let b = subtract(d, c);
    let a = subtract(subtract(cubic[3], cubic[0]), scale(d, 3.0));
    let mut tangent = tangents[0];
    let mut max_t = 0.0;
    let mut roots = Vec::with_capacity(chop_count);
    for _ in 0..chop_count {
        tangent = Vec2D::new(
            cos_rotation * tangent.x - sin_rotation * tangent.y,
            sin_rotation * tangent.x + cos_rotation * tangent.y,
        );
        let qa = a.x * tangent.y - a.y * tangent.x;
        let qb = b.x * tangent.y - b.y * tangent.x;
        let qc = c.x * tangent.y - c.y * tangent.x;
        let discriminant = qb * qb - qa * qc;
        let q = -qb - discriminant.sqrt().copysign(qb);
        let root = qc / q;
        if root > max_t + 1e-4 && root < 1.0 - 1e-4 {
            max_t = root;
            roots.push(root);
        }
    }
    for segment in chop_cubic_at_values(cubic, &roots) {
        path.cubic_to(
            segment[1].x,
            segment[1].y,
            segment[2].x,
            segment[2].y,
            segment[3].x,
            segment[3].y,
        );
    }
}

fn angle_between(a: Vec2D, b: Vec2D) -> f32 {
    let denominator = ((a.x * a.x + a.y * a.y) * (b.x * b.x + b.y * b.y)).sqrt();
    let cosine = (dot(a, b) / denominator).clamp(-1.0, 1.0);
    if cosine.is_nan() {
        0.0
    } else {
        cosine.acos()
    }
}

fn feather_max_screen_radius() -> f32 {
    1.0 / (POLAR_PRECISION as f32 * (1.0 - (std::f32::consts::PI / 32.0).cos()))
}

fn stroke_contours(path: &RawPath, contours: &mut Vec<StrokeContour>) -> Option<usize> {
    let contour_count = path
        .verbs()
        .iter()
        .filter(|verb| **verb == PathVerb::Move)
        .count();
    if contours.capacity() < contour_count {
        contours.reserve(contour_count - contours.len());
    }
    let mut contour_count = 0;
    let mut point_index = 0;
    for verb in path.verbs() {
        match verb {
            PathVerb::Move => {
                let point = path.points()[point_index];
                if contour_count == contours.len() {
                    contours.push(StrokeContour {
                        curves: SmallVec::new(),
                        first: point,
                        current: point,
                        closed: false,
                    });
                } else {
                    let contour = &mut contours[contour_count];
                    contour.curves.clear();
                    contour.first = point;
                    contour.current = point;
                    contour.closed = false;
                }
                contour_count += 1;
                point_index += 1;
            }
            PathVerb::Line => {
                let end = path.points()[point_index];
                let contour = contours.get_mut(contour_count.checked_sub(1)?)?;
                contour.curves.push(StrokeCurve {
                    cubic: line_cubic(contour.current, end),
                    is_line: true,
                });
                contour.current = end;
                point_index += 1;
            }
            PathVerb::Quad => {
                let control = path.points()[point_index];
                let end = path.points()[point_index + 1];
                let contour = contours.get_mut(contour_count.checked_sub(1)?)?;
                contour.curves.push(StrokeCurve {
                    cubic: [
                        contour.current,
                        lerp(contour.current, control, 2.0 / 3.0),
                        lerp(end, control, 2.0 / 3.0),
                        end,
                    ],
                    is_line: false,
                });
                contour.current = end;
                point_index += 2;
            }
            PathVerb::Cubic => {
                let control0 = path.points()[point_index];
                let control1 = path.points()[point_index + 1];
                let end = path.points()[point_index + 2];
                let contour = contours.get_mut(contour_count.checked_sub(1)?)?;
                contour.curves.push(StrokeCurve {
                    cubic: [contour.current, control0, control1, end],
                    is_line: false,
                });
                contour.current = end;
                point_index += 3;
            }
            PathVerb::Close => {
                contours.get_mut(contour_count.checked_sub(1)?)?.closed = true;
            }
        }
    }
    Some(contour_count)
}

fn cubic_tangents(curve: [Vec2D; 4]) -> [Vec2D; 2] {
    let start_control = if !same_point(curve[0], curve[1]) {
        curve[1]
    } else if !same_point(curve[1], curve[2]) {
        curve[2]
    } else {
        curve[3]
    };
    let end_control = if !same_point(curve[3], curve[2]) {
        curve[2]
    } else if !same_point(curve[2], curve[1]) {
        curve[1]
    } else {
        curve[0]
    };
    [
        subtract(start_control, curve[0]),
        subtract(curve[3], end_control),
    ]
}

fn find_cubic_convex_180_chops(points: [Vec2D; 4]) -> (Vec<f32>, bool) {
    const TESS_EPSILON: f32 = 1.0 / 1024.0;
    let c_vector = subtract(points[1], points[0]);
    let d = subtract(points[2], points[1]);
    let e = subtract(points[3], points[0]);
    let b_vector = subtract(d, c_vector);
    let a_vector = subtract(e, scale(d, 3.0));
    let mut a = vector_cross(a_vector, b_vector);
    let b = vector_cross(a_vector, c_vector);
    let mut c = vector_cross(b_vector, c_vector);
    let mut b_over_minus_2 = -0.5 * b;
    let mut discriminant_over_4 = b_over_minus_2 * b_over_minus_2 - a * c;
    let cusp_threshold = (a * (TESS_EPSILON * 0.5)).powi(2);
    let inside = |root: f32| root.is_finite() && root >= TESS_EPSILON && root < 1.0 - TESS_EPSILON;
    if discriminant_over_4 < -cusp_threshold {
        let root = c / b_over_minus_2;
        return (inside(root).then_some(root).into_iter().collect(), false);
    }
    let are_cusps = discriminant_over_4 <= cusp_threshold;
    if are_cusps {
        if a != 0.0 || b_over_minus_2 != 0.0 || c != 0.0 {
            let root = b_over_minus_2 / a;
            return (inside(root).then_some(root).into_iter().collect(), true);
        }
        let base = subtract(points[3], points[0]);
        let ordered = points
            .windows(2)
            .all(|points| dot(points[1], base) > dot(points[0], base));
        if ordered {
            return (Vec::new(), false);
        }
        let tangent0 = if c_vector.x != 0.0 || c_vector.y != 0.0 {
            c_vector
        } else {
            subtract(points[2], points[0])
        };
        a = dot(tangent0, a_vector);
        b_over_minus_2 = -dot(tangent0, b_vector);
        c = dot(tangent0, c_vector);
        discriminant_over_4 = (b_over_minus_2 * b_over_minus_2 - a * c).max(0.0);
    }
    let q = discriminant_over_4.sqrt().copysign(b_over_minus_2) + b_over_minus_2;
    let mut roots = [q / a, c / q]
        .into_iter()
        .filter(|root| inside(*root))
        .collect::<Vec<_>>();
    roots.sort_by(f32::total_cmp);
    roots.dedup_by(|a, b| a.to_bits() == b.to_bits());
    (roots, are_cusps)
}

fn chop_cubic_at_values(curve: [Vec2D; 4], roots: &[f32]) -> Vec<[Vec2D; 4]> {
    let mut result = Vec::with_capacity(roots.len() + 1);
    let mut remaining = curve;
    let mut last_t = 0.0;
    let mut roots = roots.chunks_exact(2);
    for pair in &mut roots {
        let denominator = 1.0 - last_t;
        let t0 = ((pair[0] - last_t) / denominator).clamp(0.0, 1.0);
        let t1 = ((pair[1] - last_t) / denominator).clamp(0.0, 1.0);
        let [first, middle, last] = split_cubic_at_two(remaining, t0, t1);
        result.extend([first, middle]);
        remaining = last;
        last_t = pair[1];
    }
    if let Some(&root) = roots.remainder().first() {
        let local_t = ((root - last_t) / (1.0 - last_t)).clamp(0.0, 1.0);
        let (left, right) = split_cubic(remaining, local_t);
        result.push(left);
        remaining = right;
    }
    result.push(remaining);
    result
}

fn split_cubic_at_two(curve: [Vec2D; 4], t0: f32, t1: f32) -> [[Vec2D; 4]; 3] {
    debug_assert!((0.0..=t1).contains(&t0));
    debug_assert!(t1 <= 1.0);
    if t1 == 1.0 {
        let (first, remaining) = split_cubic(curve, t0);
        return [first, remaining, [curve[3]; 4]];
    }

    let ab0 = lerp(curve[0], curve[1], t0);
    let bc0 = lerp(curve[1], curve[2], t0);
    let cd0 = lerp(curve[2], curve[3], t0);
    let abc0 = lerp(ab0, bc0, t0);
    let bcd0 = lerp(bc0, cd0, t0);
    let split0 = lerp(abc0, bcd0, t0);

    let ab1 = lerp(curve[0], curve[1], t1);
    let bc1 = lerp(curve[1], curve[2], t1);
    let cd1 = lerp(curve[2], curve[3], t1);
    let abc1 = lerp(ab1, bc1, t1);
    let bcd1 = lerp(bc1, cd1, t1);
    let split1 = lerp(abc1, bcd1, t1);

    [
        [curve[0], ab0, abc0, split0],
        [split0, lerp(abc0, bcd0, t1), lerp(abc1, bcd1, t0), split1],
        [split1, bcd1, cd1, curve[3]],
    ]
}

fn chop_cubic_around_cusps(
    curve: [Vec2D; 4],
    cusp_roots: &[f32],
    matrix_scale: f32,
) -> Vec<[Vec2D; 4]> {
    const EPSILON: f32 = 1.0 / 4096.0;
    let mut straddles = Vec::with_capacity(cusp_roots.len() * 2);
    for (index, &root) in cusp_roots.iter().enumerate() {
        let min_t = if index == 0 {
            0.0
        } else {
            (cusp_roots[index - 1] + root) * 0.5
        };
        let max_t = if index + 1 == cusp_roots.len() {
            1.0
        } else {
            (cusp_roots[index + 1] + root) * 0.5
        };
        straddles.extend([(root - EPSILON).max(min_t), (root + EPSILON).min(max_t)]);
    }
    let mut chopped = chop_cubic_at_values(curve, &straddles);
    for (index, &root) in cusp_roots.iter().enumerate() {
        let cusp = eval_cubic(curve, root);
        let offset = index * 2;
        chopped[offset][3] = cusp;
        chopped[offset + 1][0] = cusp;
        chopped[offset + 1][3] = cusp;
        chopped[offset + 2][0] = cusp;
        let neighboring_midpoint = lerp(chopped[offset][2], chopped[offset + 2][1], 0.5);
        let direction = subtract(cusp, neighboring_midpoint);
        let length = (direction.x * direction.x + direction.y * direction.y).sqrt();
        let pivot = if length > 0.0 && matrix_scale > 0.0 {
            let amount = 1.0 / (length * matrix_scale * POLAR_PRECISION as f32 * 2.0);
            Vec2D::new(cusp.x + direction.x * amount, cusp.y + direction.y * amount)
        } else {
            cusp
        };
        chopped[offset + 1][1] = pivot;
        chopped[offset + 1][2] = pivot;
    }
    chopped
}

fn split_cubic(curve: [Vec2D; 4], t: f32) -> ([Vec2D; 4], [Vec2D; 4]) {
    let ab = lerp(curve[0], curve[1], t);
    let bc = lerp(curve[1], curve[2], t);
    let cd = lerp(curve[2], curve[3], t);
    let abc = lerp(ab, bc, t);
    let bcd = lerp(bc, cd, t);
    let split = lerp(abc, bcd, t);
    ([curve[0], ab, abc, split], [split, bcd, cd, curve[3]])
}

fn max_matrix_scale(transform: Mat2D) -> f32 {
    let [xx, yx, xy, yy, _, _] = transform.0;
    if xy == 0.0 && yx == 0.0 {
        return xx.abs().max(yy.abs());
    }
    let a = xx * xx + xy * xy;
    let b = xx * yx + yy * xy;
    let c = yx * yx + yy * yy;
    let result = if b * b <= f32::EPSILON * f32::EPSILON {
        a.max(c)
    } else {
        (a + c) * 0.5 + ((a - c) * (a - c) + 4.0 * b * b).sqrt() * 0.5
    };
    if result.is_finite() {
        result.max(0.0).sqrt()
    } else {
        0.0
    }
}

fn polar_segments_per_radian(radius: f32) -> f32 {
    let cos_theta = 1.0 - (1.0 / POLAR_PRECISION as f32) / radius;
    0.5 / cos_theta.max(-1.0).acos()
}

fn fast_acos(x: f32) -> f32 {
    const A: f32 = -0.939115566365855;
    const B: f32 = 0.9217841528914573;
    const C: f32 = -1.2845906244690837;
    const D: f32 = 0.295624144969963174;
    const PI_OVER_2: f32 = 1.5707963267948966;

    let xx = x * x;
    let numer = B * xx + A;
    let denom = xx * (D * xx + C) + 1.0;
    x * (numer / denom) + PI_OVER_2
}

fn round_join_segment_count(incoming: Vec2D, outgoing: Vec2D, per_radian: f32) -> u32 {
    let denominator = ((incoming.x * incoming.x + incoming.y * incoming.y)
        * (outgoing.x * outgoing.x + outgoing.y * outgoing.y))
        .sqrt();
    let cosine =
        ((incoming.x * outgoing.x + incoming.y * outgoing.y) / denominator).clamp(-1.0, 1.0);
    (fast_acos(cosine) * per_radian)
        .ceil()
        .clamp(1.0, crate::gpu::MAX_POLAR_SEGMENTS as f32) as u32
}

#[cfg(test)]
mod fast_acos_tests {
    use super::*;

    #[test]
    fn round_join_segments_follow_cpp_fast_acos_at_threshold_edges() {
        let per_radian = 5.5;
        let cases = [
            (Vec2D::new(0.934_609_65, 0.355_672_72), 2),
            (Vec2D::new(0.614_463_2, 0.788_945_5), 6),
            (Vec2D::new(-0.827_510_06, 0.561_448_9), 15),
        ];

        for (outgoing, expected) in cases {
            assert_eq!(
                round_join_segment_count(Vec2D::new(1.0, 0.0), outgoing, per_radian),
                expected
            );
        }
    }
}

fn subtract(a: Vec2D, b: Vec2D) -> Vec2D {
    Vec2D::new(a.x - b.x, a.y - b.y)
}

fn negate(vector: Vec2D) -> Vec2D {
    Vec2D::new(-vector.x, -vector.y)
}

fn scale(vector: Vec2D, amount: f32) -> Vec2D {
    Vec2D::new(vector.x * amount, vector.y * amount)
}

fn dot(a: Vec2D, b: Vec2D) -> f32 {
    a.x * b.x + a.y * b.y
}

fn vector_cross(a: Vec2D, b: Vec2D) -> f32 {
    a.x * b.y - a.y * b.x
}

pub(crate) fn should_use_interior_tessellation(path: &RawPath, transform: Mat2D) -> bool {
    if path.verbs().len() >= 1000 || path.points().is_empty() {
        return false;
    }
    let mut min = path.points()[0];
    let mut max = min;
    for point in &path.points()[1..] {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }
    let [xx, yx, xy, yy, _, _] = transform.0;
    let transformed_area = (xx * yy - xy * yx).abs() * (max.x - min.x) * (max.y - min.y);
    transformed_area > 512.0 * 512.0
}

pub(crate) fn path_coarse_area(path: &RawPath) -> f32 {
    let mut area = 0.0;
    let mut contour_start = Vec2D::new(0.0, 0.0);
    let mut last = Vec2D::new(0.0, 0.0);
    let mut point_index = 0;
    for verb in path.verbs() {
        match verb {
            PathVerb::Move => {
                area += vector_cross(last, contour_start);
                contour_start = path.points()[point_index];
                last = contour_start;
                point_index += 1;
            }
            PathVerb::Line => {
                let end = path.points()[point_index];
                area += vector_cross(last, end);
                last = end;
                point_index += 1;
            }
            PathVerb::Quad => {
                let control = path.points()[point_index];
                let end = path.points()[point_index + 1];
                let cubic = [
                    last,
                    lerp(last, control, 2.0 / 3.0),
                    lerp(end, control, 2.0 / 3.0),
                    end,
                ];
                accumulate_coarse_cubic_area(&mut area, &mut last, cubic);
                point_index += 2;
            }
            PathVerb::Cubic => {
                let cubic = [
                    last,
                    path.points()[point_index],
                    path.points()[point_index + 1],
                    path.points()[point_index + 2],
                ];
                accumulate_coarse_cubic_area(&mut area, &mut last, cubic);
                point_index += 3;
            }
            PathVerb::Close => {}
        }
    }
    area += vector_cross(last, contour_start);
    area * 0.5
}

fn accumulate_coarse_cubic_area(area: &mut f32, last: &mut Vec2D, cubic: [Vec2D; 4]) {
    let segment_count = coarse_cubic_segment_count(cubic);
    for segment in 1..segment_count {
        let point = eval_cubic(cubic, segment as f32 / segment_count as f32);
        *area += vector_cross(*last, point);
        *last = point;
    }
    *area += vector_cross(*last, cubic[3]);
    *last = cubic[3];
}

fn coarse_cubic_segment_count(points: [Vec2D; 4]) -> u32 {
    let second_difference = |a: Vec2D, b: Vec2D, c: Vec2D| {
        let x = a.x - 2.0 * b.x + c.x;
        let y = a.y - 2.0 * b.y + c.y;
        x * x + y * y
    };
    let max_length_squared = second_difference(points[0], points[1], points[2])
        .max(second_difference(points[1], points[2], points[3]));
    let length_term_squared = (9.0 / 16.0) * (1.0 / 8.0f32).powi(2);
    (max_length_squared * length_term_squared)
        .sqrt()
        .sqrt()
        .ceil()
        .clamp(1.0, 64.0) as u32
}

pub(crate) fn build_interior_tessellation(
    path: &RawPath,
    transform: Mat2D,
    fill_rule: FillRule,
    clockwise_override: bool,
) -> Option<InteriorTessellation> {
    #[cfg(test)]
    INTERIOR_TESSELLATION_BUILD_COUNT.with(|count| count.set(count.get() + 1));
    let cubic_contours = cubic_contours(path)
        .into_iter()
        .map(|curves| {
            curves
                .into_iter()
                .flat_map(|curve| {
                    let subdivision_count = outer_cubic_subdivision_count(curve, transform);
                    subdivide_cubic(curve, subdivision_count)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if cubic_contours.is_empty() {
        return None;
    }
    let mut scratch = RawPath::new();
    for curves in &cubic_contours {
        let first = curves.first()?;
        scratch.move_to(first[0].x, first[0].y);
        for curve in curves {
            scratch.line_to(curve[3].x, curve[3].y);
        }
    }
    let mut min = path.points()[0];
    let mut max = min;
    for point in &path.points()[1..] {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }
    let direction = if max.x - min.x > max.y - min.y {
        SweepDirection::Horizontal
    } else {
        SweepDirection::Vertical
    };
    let determinant = transform.0[0] * transform.0[3] - transform.0[2] * transform.0[1];
    let coarse_area = path_coarse_area(path);
    let negate_coverage = clockwise_atomic_negate_coverage_from_area(
        coarse_area,
        determinant,
        fill_rule,
        clockwise_override,
    );
    let effective_fill_rule = if clockwise_override {
        FillRule::Clockwise
    } else {
        fill_rule
    };
    let mut triangulator =
        InnerFanTriangulator::new(&scratch, transform, direction, effective_fill_rule);
    if (determinant < 0.0) != negate_coverage {
        triangulator.negate_winding();
    }
    let triangles = triangulator.triangles(1, WindingFaces::All);
    // Count the NonZero superset from the already-built mesh. This is the same
    // allocation-free polygon bound as C++ and stays conservative if admission
    // selects a different fill variant from the final atomic run.
    let max_triangle_vertex_count = triangulator.non_zero_max_triangle_vertex_count();
    let grout = triangulator.grout_triangles().to_vec();
    let base = OUTER_CURVE_PATCH_SEGMENT_SPAN as i32;
    let curve_count = cubic_contours.iter().map(Vec::len).sum::<usize>();
    let patch_count = curve_count + grout.len();
    let half_vertex_count = (patch_count * OUTER_CURVE_PATCH_SEGMENT_SPAN) as i32;
    let mut spans = Vec::with_capacity(patch_count + 1);
    push_padding_span(&mut spans, 0, base);
    let mut contours = Vec::with_capacity(cubic_contours.len());
    let mut curve_offset = 0i32;
    for (contour_index, curves) in cubic_contours.iter().enumerate() {
        let forward_base = if negate_coverage {
            base
        } else {
            base + half_vertex_count
        };
        contours.push(ContourData::new(
            [0.0, 0.0],
            1,
            (forward_base + curve_offset) as u32,
        ));
        for curve in curves {
            let span = TessVertexSpan::without_reflection(
                curve.map(|point| [point.x, point.y]),
                [0.0, 0.0],
                0.0,
                0,
                OUTER_CURVE_PATCH_SEGMENT_SPAN as i32,
                16,
                1,
                1,
                ((contour_index as u32 + 1) & CONTOUR_ID_MASK)
                    | CULL_EXCESS_TESSELLATION_SEGMENTS_CONTOUR_FLAG
                    | u32::from(negate_coverage) * NEGATE_PATH_FILL_COVERAGE_FLAG,
            );
            push_double_sided_tessellation_spans(
                &mut spans,
                span,
                base + curve_offset,
                base + curve_offset + OUTER_CURVE_PATCH_SEGMENT_SPAN as i32,
                base,
                half_vertex_count,
                negate_coverage,
            );
            curve_offset += OUTER_CURVE_PATCH_SEGMENT_SPAN as i32;
        }
    }
    let grout_contour_id = (cubic_contours.len() as u32) & CONTOUR_ID_MASK;
    for triangle in &grout {
        let cubic = [triangle[0], triangle[1], Vec2D::new(0.0, 0.0), triangle[2]];
        let span = TessVertexSpan::without_reflection(
            cubic.map(|point| [point.x, point.y]),
            [0.0, 0.0],
            0.0,
            0,
            OUTER_CURVE_PATCH_SEGMENT_SPAN as i32,
            16,
            1,
            1,
            grout_contour_id
                | RETROFITTED_TRIANGLE_CONTOUR_FLAG
                | u32::from(negate_coverage) * NEGATE_PATH_FILL_COVERAGE_FLAG,
        );
        push_double_sided_tessellation_spans(
            &mut spans,
            span,
            base + curve_offset,
            base + curve_offset + OUTER_CURVE_PATCH_SEGMENT_SPAN as i32,
            base,
            half_vertex_count,
            negate_coverage,
        );
        curve_offset += OUTER_CURVE_PATCH_SEGMENT_SPAN as i32;
    }
    push_final_padding(&mut spans, base + half_vertex_count * 2);
    Some(InteriorTessellation {
        spans,
        path: PathData::new(
            transform,
            0.0,
            0.0,
            0,
            AtlasTransform::zeroed(),
            CoverageBufferRange::zeroed(),
        ),
        contours,
        triangles,
        max_triangle_vertex_count,
        base_instance: 1,
        instance_count: (patch_count * 2) as u32,
    })
}

fn outer_cubic_subdivision_count(points: [Vec2D; 4], transform: Mat2D) -> u32 {
    let max_length_squared = max_transformed_cubic_second_difference(points, transform);
    let length_term_squared = (9.0 / 16.0) * (PARAMETRIC_PRECISION as f32).powi(2);
    let wang_segments = (max_length_squared * length_term_squared).sqrt().sqrt();
    (wang_segments / 16.0)
        .ceil()
        .clamp(1.0, MAX_PARAMETRIC_SEGMENTS.div_ceil(16) as f32) as u32
}

fn subdivide_cubic(mut curve: [Vec2D; 4], subdivision_count: u32) -> Vec<[Vec2D; 4]> {
    let mut result = Vec::with_capacity(subdivision_count as usize);
    let mut remaining = subdivision_count;
    while remaining >= 3 {
        let t0 = 1.0 / remaining as f32;
        let t1 = 2.0 / remaining as f32;
        let ab0 = lerp(curve[0], curve[1], t0);
        let bc0 = lerp(curve[1], curve[2], t0);
        let cd0 = lerp(curve[2], curve[3], t0);
        let abc0 = lerp(ab0, bc0, t0);
        let bcd0 = lerp(bc0, cd0, t0);
        let split0 = lerp(abc0, bcd0, t0);
        let ab1 = lerp(curve[0], curve[1], t1);
        let bc1 = lerp(curve[1], curve[2], t1);
        let cd1 = lerp(curve[2], curve[3], t1);
        let abc1 = lerp(ab1, bc1, t1);
        let bcd1 = lerp(bc1, cd1, t1);
        let split1 = lerp(abc1, bcd1, t1);
        result.push([curve[0], ab0, abc0, split0]);
        result.push([split0, lerp(abc0, bcd0, t1), lerp(abc1, bcd1, t0), split1]);
        curve = [split1, bcd1, cd1, curve[3]];
        remaining -= 2;
    }
    if remaining == 2 {
        let ab = lerp(curve[0], curve[1], 0.5);
        let bc = lerp(curve[1], curve[2], 0.5);
        let cd = lerp(curve[2], curve[3], 0.5);
        let abc = lerp(ab, bc, 0.5);
        let bcd = lerp(bc, cd, 0.5);
        let split = lerp(abc, bcd, 0.5);
        result.push([curve[0], ab, abc, split]);
        curve = [split, bcd, cd, curve[3]];
    }
    result.push(curve);
    result
}

impl FillTessellation {
    pub(crate) fn make_double_sided(&mut self) {
        self.make_double_sided_with_direction(false);
    }

    pub(crate) fn make_double_sided_with_direction(&mut self, forward_then_reverse: bool) {
        let base = self.base_instance * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let half_vertex_count = self.instance_count * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let mut geometry_spans = Vec::with_capacity(self.spans.len());
        let mut previous_logical_x0 = None;
        for mut span in self
            .spans
            .iter()
            .copied()
            .filter(|span| span.contour_id_with_flags & CONTOUR_ID_MASK != 0)
        {
            let (x0, x1) = span.x_range();
            let logical_x0 = span.y as i32 * TESS_TEXTURE_WIDTH + x0;
            if previous_logical_x0 == Some(logical_x0) {
                continue;
            }
            previous_logical_x0 = Some(logical_x0);
            if forward_then_reverse {
                span.contour_id_with_flags |= NEGATE_PATH_FILL_COVERAGE_FLAG;
            }
            push_double_sided_tessellation_spans(
                &mut geometry_spans,
                span,
                logical_x0,
                logical_x0 + x1 - x0,
                base as i32,
                half_vertex_count as i32,
                forward_then_reverse,
            );
        }
        if !forward_then_reverse {
            for contour in &mut self.contours {
                contour.vertex_index0 += half_vertex_count;
            }
        }
        self.instance_count *= 2;
        let location =
            (self.base_instance + self.instance_count) * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let mut ordered_spans = Vec::with_capacity(geometry_spans.len() + 3);
        push_padding_span(&mut ordered_spans, 0, base as i32);
        push_midpoint_tail_padding(&mut ordered_spans, location as i32);
        ordered_spans.extend(geometry_spans);
        self.spans = ordered_spans;
    }

    fn make_single_sided_reverse(&mut self, negate_coverage: bool) {
        let base = self.base_instance * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let vertex_count = self.instance_count * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let end = base + vertex_count;
        let mut geometry_spans = Vec::with_capacity(self.spans.len());
        let mut previous_logical_x0 = None;
        for mut span in self
            .spans
            .iter()
            .copied()
            .filter(|span| span.contour_id_with_flags & CONTOUR_ID_MASK != 0)
        {
            let (x0, x1) = span.x_range();
            let logical_x0 = span.y as i32 * TESS_TEXTURE_WIDTH + x0;
            if previous_logical_x0 == Some(logical_x0) {
                continue;
            }
            previous_logical_x0 = Some(logical_x0);
            if negate_coverage {
                span.contour_id_with_flags |= NEGATE_PATH_FILL_COVERAGE_FLAG;
            }
            push_reverse_tessellation_spans(
                &mut geometry_spans,
                span,
                logical_x0,
                logical_x0 + x1 - x0,
                base as i32,
                end as i32,
            );
        }
        for contour in &mut self.contours {
            contour.vertex_index0 = base + end - contour.vertex_index0 - 1;
        }
        let mut ordered_spans = Vec::with_capacity(geometry_spans.len() + 3);
        push_padding_span(&mut ordered_spans, 0, base as i32);
        push_midpoint_tail_padding(&mut ordered_spans, end as i32);
        ordered_spans.extend(geometry_spans);
        self.spans = ordered_spans;
    }
}

pub(crate) fn clockwise_atomic_negate_coverage(
    path: &RawPath,
    transform: Mat2D,
    fill_rule: FillRule,
    clockwise_override: bool,
) -> bool {
    let determinant = transform.0[0] * transform.0[3] - transform.0[2] * transform.0[1];
    let coarse_area = path_coarse_area(path);
    clockwise_atomic_negate_coverage_from_area(
        coarse_area,
        determinant,
        fill_rule,
        clockwise_override,
    )
}

#[cfg(test)]
pub(crate) fn msaa_fill_requires_reverse(
    path: &RawPath,
    transform: Mat2D,
    fill_rule: FillRule,
) -> bool {
    msaa_fill_requires_reverse_from_area(path_coarse_area(path), transform, fill_rule)
}

pub(crate) fn msaa_fill_requires_reverse_from_area(
    coarse_area: f32,
    transform: Mat2D,
    fill_rule: FillRule,
) -> bool {
    let determinant = transform.0[0] * transform.0[3] - transform.0[2] * transform.0[1];
    match fill_rule {
        FillRule::EvenOdd => false,
        FillRule::Clockwise => determinant < 0.0,
        FillRule::NonZero => coarse_area * determinant < 0.0,
    }
}

pub(crate) fn clockwise_atomic_negate_coverage_from_area(
    coarse_area: f32,
    determinant: f32,
    fill_rule: FillRule,
    clockwise_override: bool,
) -> bool {
    if fill_rule == FillRule::Clockwise {
        determinant < 0.0
    } else {
        clockwise_override && coarse_area * determinant < 0.0
    }
}

fn push_double_sided_tessellation_spans(
    spans: &mut Vec<TessVertexSpan>,
    mut span: TessVertexSpan,
    logical_x0: i32,
    logical_x1: i32,
    base: i32,
    half_vertex_count: i32,
    forward_then_reverse: bool,
) {
    let offset = logical_x0 - base;
    let vertex_count = logical_x1 - logical_x0;
    let (forward_location, reflection_location) = if forward_then_reverse {
        (base + offset, base + half_vertex_count * 2 - offset)
    } else {
        (
            base + half_vertex_count + offset,
            base + half_vertex_count - offset,
        )
    };
    let mut y = forward_location / TESS_TEXTURE_WIDTH;
    let mut x0 = forward_location % TESS_TEXTURE_WIDTH;
    let mut x1 = x0 + vertex_count;
    let mut reflection_y = ((reflection_location - 1) / TESS_TEXTURE_WIDTH) as u32;
    let mut reflection_x0 = (reflection_location - 1) % TESS_TEXTURE_WIDTH + 1;
    let mut reflection_x1 = reflection_x0 - vertex_count;

    loop {
        span.y = y as f32;
        span.set_ranges(x0, x1, reflection_x0, reflection_x1, reflection_y as f32);
        spans.push(span);
        if x1 <= TESS_TEXTURE_WIDTH && reflection_x1 >= 0 {
            break;
        }
        y += 1;
        x0 -= TESS_TEXTURE_WIDTH;
        x1 -= TESS_TEXTURE_WIDTH;
        reflection_y = reflection_y.wrapping_sub(1);
        reflection_x0 += TESS_TEXTURE_WIDTH;
        reflection_x1 += TESS_TEXTURE_WIDTH;
    }
}

fn push_reverse_tessellation_spans(
    spans: &mut Vec<TessVertexSpan>,
    mut span: TessVertexSpan,
    logical_x0: i32,
    logical_x1: i32,
    base: i32,
    end: i32,
) {
    let vertex_count = logical_x1 - logical_x0;
    let reverse_location = end - (logical_x0 - base);
    let mut y = ((reverse_location - 1) / TESS_TEXTURE_WIDTH) as u32;
    let mut x0 = (reverse_location - 1) % TESS_TEXTURE_WIDTH + 1;
    let mut x1 = x0 - vertex_count;
    loop {
        span.y = y as f32;
        span.set_ranges(x0, x1, -1, -1, f32::NAN);
        spans.push(span);
        if x1 >= 0 {
            break;
        }
        y = y.wrapping_sub(1);
        x0 += TESS_TEXTURE_WIDTH;
        x1 += TESS_TEXTURE_WIDTH;
    }
}

pub(crate) fn build_fill_tessellation(
    path: &RawPath,
    transform: Mat2D,
) -> Option<FillTessellation> {
    #[cfg(test)]
    FILL_TESSELLATION_BUILD_COUNT.with(|count| count.set(count.get() + 1));
    let contours = fill_cubic_contours(path);
    if contours.is_empty() {
        return None;
    }
    let mut spans = Vec::new();
    let mut location = MIDPOINT_FAN_PATCH_SEGMENT_SPAN as i32;
    push_padding_span(&mut spans, 0, location);
    let base_instance = location as u32 / MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
    let mut contour_data = Vec::with_capacity(contours.len());
    let path_start = location;
    for (index, curves) in contours.iter().enumerate() {
        let vertex_index0 = location as u32;
        let midpoint = contour_midpoint(curves);
        contour_data.push(ContourData::new([midpoint.x, midpoint.y], 0, vertex_index0));
        let segment_counts = curves
            .iter()
            .map(|curve| {
                // C++ PathDraw counts every line as exactly two tessellation vertices.
                if curve.is_line {
                    1
                } else {
                    cubic_segment_count_with_precision_and_transform(
                        curve.cubic,
                        PARAMETRIC_PRECISION as f32,
                        transform,
                    )
                }
            })
            .collect::<Vec<_>>();
        let raw_vertex_count = segment_counts.iter().sum::<u32>() + curves.len() as u32;
        let padding = align_up(
            raw_vertex_count as i32,
            MIDPOINT_FAN_PATCH_SEGMENT_SPAN as i32,
        ) - raw_vertex_count as i32;
        for (curve_index, (curve, segments)) in
            curves.iter().zip(segment_counts.into_iter()).enumerate()
        {
            let x0 = location;
            location += segments as i32 + 1 + i32::from(curve_index == 0) * padding;
            push_forward_tessellation_spans(
                &mut spans,
                curve.cubic.map(|point| [point.x, point.y]),
                if curve.is_line {
                    [0.0, 1.0]
                } else {
                    [0.0, 0.0]
                },
                x0,
                location,
                segments,
                1,
                1,
                (index as u32 + 1) & CONTOUR_ID_MASK,
            );
        }
    }
    // C++ LogicalFlush writes every padding span before PathDraw geometry.
    let geometry_spans = spans.split_off(1);
    push_midpoint_tail_padding(&mut spans, location);
    spans.extend(geometry_spans);
    Some(FillTessellation {
        spans,
        path: PathData::new(
            transform,
            0.0,
            0.0,
            0,
            AtlasTransform::zeroed(),
            CoverageBufferRange::zeroed(),
        ),
        contours: contour_data,
        base_instance,
        instance_count: (location - path_start) as u32 / MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32,
    })
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct StrokePreparationStats {
    pub builds: usize,
    pub contours: usize,
    pub inline_one_curve_contours: usize,
    pub spilled_curve_contours: usize,
    pub contour_capacity_growths: usize,
    pub prepared_capacity_growths: usize,
    pub pending_capacity_growths: usize,
}

#[cfg(test)]
thread_local! {
    static FILL_TESSELLATION_BUILD_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FILL_TESSELLATION_CLONE_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static INTERIOR_TESSELLATION_BUILD_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static INTERIOR_TESSELLATION_CLONE_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static INTERIOR_TRIANGLE_VISIT_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_fill_tessellation_build_count() {
    FILL_TESSELLATION_BUILD_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn fill_tessellation_build_count() -> usize {
    FILL_TESSELLATION_BUILD_COUNT.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_fill_tessellation_clone_count() {
    FILL_TESSELLATION_CLONE_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn fill_tessellation_clone_count() -> usize {
    FILL_TESSELLATION_CLONE_COUNT.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_interior_tessellation_build_count() {
    INTERIOR_TESSELLATION_BUILD_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn interior_tessellation_build_count() -> usize {
    INTERIOR_TESSELLATION_BUILD_COUNT.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_interior_tessellation_clone_count() {
    INTERIOR_TESSELLATION_CLONE_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn interior_tessellation_clone_count() -> usize {
    INTERIOR_TESSELLATION_CLONE_COUNT.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_interior_triangle_visit_count() {
    INTERIOR_TRIANGLE_VISIT_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn interior_triangle_visit_count() -> usize {
    INTERIOR_TRIANGLE_VISIT_COUNT.with(std::cell::Cell::get)
}

fn fill_cubic_contours(path: &RawPath) -> Vec<Vec<StrokeCurve>> {
    let mut result = Vec::new();
    let mut curves = Vec::new();
    let mut first = None;
    let mut current = None;
    let mut point_index = 0;
    let finish = |result: &mut Vec<Vec<StrokeCurve>>,
                  curves: &mut Vec<StrokeCurve>,
                  first: &mut Option<Vec2D>,
                  current: &mut Option<Vec2D>| {
        if let (Some(start), Some(end)) = (*first, *current) {
            if !points_equal(start, end) {
                curves.push(StrokeCurve {
                    cubic: line_cubic(end, start),
                    is_line: true,
                });
            }
        }
        if !curves.is_empty() {
            result.push(std::mem::take(curves));
        }
        *first = None;
        *current = None;
    };
    for verb in path.verbs() {
        match verb {
            PathVerb::Move => {
                finish(&mut result, &mut curves, &mut first, &mut current);
                let point = path.points()[point_index];
                point_index += 1;
                first = Some(point);
                current = Some(point);
            }
            PathVerb::Line => {
                let end = path.points()[point_index];
                point_index += 1;
                if let Some(start) = current {
                    if !points_equal(start, end) {
                        curves.push(StrokeCurve {
                            cubic: line_cubic(start, end),
                            is_line: true,
                        });
                    }
                }
                current = Some(end);
            }
            PathVerb::Quad => {
                let control = path.points()[point_index];
                let end = path.points()[point_index + 1];
                point_index += 2;
                if let Some(start) = current {
                    curves.push(StrokeCurve {
                        cubic: [
                            start,
                            lerp(start, control, 2.0 / 3.0),
                            lerp(end, control, 2.0 / 3.0),
                            end,
                        ],
                        is_line: false,
                    });
                }
                current = Some(end);
            }
            PathVerb::Cubic => {
                let control0 = path.points()[point_index];
                let control1 = path.points()[point_index + 1];
                let end = path.points()[point_index + 2];
                point_index += 3;
                if let Some(start) = current {
                    curves.push(StrokeCurve {
                        cubic: [start, control0, control1, end],
                        is_line: false,
                    });
                }
                current = Some(end);
            }
            PathVerb::Close => {
                if let (Some(start), Some(end)) = (first, current) {
                    if !points_equal(start, end) {
                        curves.push(StrokeCurve {
                            cubic: line_cubic(end, start),
                            is_line: true,
                        });
                    }
                    current = Some(start);
                }
            }
        }
    }
    finish(&mut result, &mut curves, &mut first, &mut current);
    result
}

fn cubic_contours(path: &RawPath) -> Vec<Vec<[Vec2D; 4]>> {
    fill_cubic_contours(path)
        .into_iter()
        .map(|curves| curves.into_iter().map(|curve| curve.cubic).collect())
        .collect()
}

fn line_cubic(start: Vec2D, end: Vec2D) -> [Vec2D; 4] {
    let mix_one_third = |a: Vec2D, b: Vec2D| {
        let t = 1.0 / 3.0;
        Vec2D::new((b.x - a.x).mul_add(t, a.x), (b.y - a.y).mul_add(t, a.y))
    };
    [
        start,
        mix_one_third(start, end),
        mix_one_third(end, start),
        end,
    ]
}

fn contour_midpoint(curves: &[StrokeCurve]) -> Vec2D {
    let mut sum = Vec2D::new(0.0, 0.0);
    for curve in curves {
        sum.x += curve.cubic[3].x;
        sum.y += curve.cubic[3].y;
    }
    let scale = 1.0 / curves.len() as f32;
    Vec2D::new(sum.x * scale, sum.y * scale)
}

fn push_padding_span(spans: &mut Vec<TessVertexSpan>, x0: i32, x1: i32) {
    spans.push(TessVertexSpan::without_reflection(
        [[0.0; 2]; 4],
        [0.0; 2],
        0.0,
        x0,
        x1,
        0,
        0,
        1,
        0,
    ));
}

fn push_final_padding(spans: &mut Vec<TessVertexSpan>, location: i32) {
    push_forward_tessellation_spans(
        spans,
        [[0.0; 2]; 4],
        [0.0; 2],
        location,
        location + 1,
        0,
        0,
        1,
        0,
    );
}

fn push_midpoint_tail_padding(spans: &mut Vec<TessVertexSpan>, location: i32) {
    let outer_curve_start = align_up(location, OUTER_CURVE_PATCH_SEGMENT_SPAN as i32);
    if outer_curve_start != location {
        push_forward_tessellation_spans(
            spans,
            [[0.0; 2]; 4],
            [0.0; 2],
            location,
            outer_curve_start,
            0,
            0,
            1,
            0,
        );
    }
    push_final_padding(spans, outer_curve_start);
}

#[allow(clippy::too_many_arguments)]
fn push_forward_tessellation_spans(
    spans: &mut Vec<TessVertexSpan>,
    points: [[f32; 2]; 4],
    join_tangent: [f32; 2],
    logical_x0: i32,
    logical_x1: i32,
    parametric_segments: u32,
    polar_segments: u32,
    join_segments: u32,
    contour_id_with_flags: u32,
) {
    let mut y = logical_x0 / TESS_TEXTURE_WIDTH;
    let mut x0 = logical_x0 % TESS_TEXTURE_WIDTH;
    let mut x1 = x0 + logical_x1 - logical_x0;
    loop {
        spans.push(TessVertexSpan::without_reflection(
            points,
            join_tangent,
            y as f32,
            x0,
            x1,
            parametric_segments,
            polar_segments,
            join_segments,
            contour_id_with_flags,
        ));
        if x1 <= TESS_TEXTURE_WIDTH {
            break;
        }
        y += 1;
        x0 -= TESS_TEXTURE_WIDTH;
        x1 -= TESS_TEXTURE_WIDTH;
    }
}

pub(crate) fn tessellation_texture_height(spans: &[TessVertexSpan]) -> u32 {
    spans.iter().map(|span| span.y as u32).max().unwrap_or(0) + 1
}

fn align_up(value: i32, alignment: i32) -> i32 {
    ((value + alignment - 1) / alignment) * alignment
}

pub(crate) fn flatten_path(path: &RawPath, transform: Mat2D) -> Vec<Contour> {
    let mut contours = Vec::new();
    let mut contour = None::<Contour>;
    let mut point_index = 0;

    for verb in path.verbs() {
        match verb {
            PathVerb::Move => {
                finish_contour(&mut contours, contour.take());
                let point = path.points()[point_index];
                point_index += 1;
                contour = Some(Contour {
                    points: vec![transform.transform_point(point)],
                    closed: false,
                });
            }
            PathVerb::Line => {
                let point = path.points()[point_index];
                point_index += 1;
                ensure_contour(&mut contour)
                    .points
                    .push(transform.transform_point(point));
            }
            PathVerb::Quad => {
                let control = path.points()[point_index];
                let end = path.points()[point_index + 1];
                point_index += 2;
                let contour = ensure_contour(&mut contour);
                let start = contour
                    .points
                    .last()
                    .copied()
                    .unwrap_or(Vec2D::new(0.0, 0.0));
                let control = transform.transform_point(control);
                let end = transform.transform_point(end);
                let cubic = [
                    start,
                    lerp(start, control, 2.0 / 3.0),
                    lerp(end, control, 2.0 / 3.0),
                    end,
                ];
                append_cubic(contour, cubic);
            }
            PathVerb::Cubic => {
                let control0 = path.points()[point_index];
                let control1 = path.points()[point_index + 1];
                let end = path.points()[point_index + 2];
                point_index += 3;
                let contour = ensure_contour(&mut contour);
                let start = contour
                    .points
                    .last()
                    .copied()
                    .unwrap_or(Vec2D::new(0.0, 0.0));
                append_cubic(
                    contour,
                    [
                        start,
                        transform.transform_point(control0),
                        transform.transform_point(control1),
                        transform.transform_point(end),
                    ],
                );
            }
            PathVerb::Close => {
                if let Some(contour) = contour.as_mut() {
                    contour.closed = true;
                }
            }
        }
    }
    finish_contour(&mut contours, contour);
    contours
}

fn ensure_contour(contour: &mut Option<Contour>) -> &mut Contour {
    contour.get_or_insert_with(|| Contour {
        points: vec![Vec2D::new(0.0, 0.0)],
        closed: false,
    })
}

fn finish_contour(contours: &mut Vec<Contour>, contour: Option<Contour>) {
    if let Some(mut contour) = contour {
        contour.points.dedup_by(|a, b| same_point(*a, *b));
        if contour.closed
            && contour.points.len() > 1
            && same_point(contour.points[0], *contour.points.last().unwrap())
        {
            contour.points.pop();
        }
        if contour.points.len() >= 2 {
            contours.push(contour);
        }
    }
}

fn same_point(a: Vec2D, b: Vec2D) -> bool {
    a.x.to_bits() == b.x.to_bits() && a.y.to_bits() == b.y.to_bits()
}

fn points_equal(a: Vec2D, b: Vec2D) -> bool {
    a.x == b.x && a.y == b.y
}

fn append_cubic(contour: &mut Contour, cubic: [Vec2D; 4]) {
    let segment_count = cubic_segment_count(cubic);
    for segment in 1..=segment_count {
        contour
            .points
            .push(eval_cubic(cubic, segment as f32 / segment_count as f32));
    }
}

pub(crate) fn cubic_segment_count(points: [Vec2D; 4]) -> u32 {
    cubic_segment_count_with_precision(points, PARAMETRIC_PRECISION as f32)
}

fn cubic_segment_count_with_precision(points: [Vec2D; 4], precision: f32) -> u32 {
    cubic_segment_count_with_precision_and_transform(points, precision, Mat2D::IDENTITY)
}

fn cubic_segment_count_with_precision_and_transform(
    points: [Vec2D; 4],
    precision: f32,
    transform: Mat2D,
) -> u32 {
    let max_length_squared = max_transformed_cubic_second_difference(points, transform);
    let length_term_squared = (9.0 / 16.0) * precision.powi(2);
    (max_length_squared * length_term_squared)
        .sqrt()
        .sqrt()
        .ceil()
        .clamp(1.0, MAX_PARAMETRIC_SEGMENTS as f32) as u32
}

fn max_transformed_cubic_second_difference(points: [Vec2D; 4], transform: Mat2D) -> f32 {
    let [xx, yx, xy, yy, _, _] = transform.0;
    let transformed_second_difference = |a: Vec2D, b: Vec2D, c: Vec2D| {
        let x = -2.0 * b.x + a.x + c.x;
        let y = -2.0 * b.y + a.y + c.y;
        let transformed_x = xx * x + xy * y;
        let transformed_y = yx * x + yy * y;
        transformed_x * transformed_x + transformed_y * transformed_y
    };
    transformed_second_difference(points[0], points[1], points[2]).max(
        transformed_second_difference(points[1], points[2], points[3]),
    )
}

pub(crate) fn triangulate_contour(points: &[Vec2D]) -> Option<Vec<u32>> {
    if points.len() < 3 {
        return None;
    }
    let mut remaining = (0..points.len()).collect::<Vec<_>>();
    let winding = signed_area(points).signum();
    if winding == 0.0 {
        return None;
    }
    let mut indices = Vec::with_capacity((points.len() - 2) * 3);
    while remaining.len() > 3 {
        let mut ear = None;
        for current in 0..remaining.len() {
            let previous = remaining[(current + remaining.len() - 1) % remaining.len()];
            let vertex = remaining[current];
            let next = remaining[(current + 1) % remaining.len()];
            if cross(points[previous], points[vertex], points[next]) * winding <= 0.0 {
                continue;
            }
            if remaining.iter().copied().any(|candidate| {
                candidate != previous
                    && candidate != vertex
                    && candidate != next
                    && point_in_triangle(
                        points[candidate],
                        points[previous],
                        points[vertex],
                        points[next],
                        winding,
                    )
            }) {
                continue;
            }
            ear = Some((current, previous, vertex, next));
            break;
        }
        let (current, previous, vertex, next) = ear?;
        indices.extend([previous as u32, vertex as u32, next as u32]);
        remaining.remove(current);
    }
    indices.extend([
        remaining[0] as u32,
        remaining[1] as u32,
        remaining[2] as u32,
    ]);
    Some(indices)
}

fn signed_area(points: &[Vec2D]) -> f32 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum::<f32>()
        * 0.5
}

fn cross(a: Vec2D, b: Vec2D, c: Vec2D) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn point_in_triangle(point: Vec2D, a: Vec2D, b: Vec2D, c: Vec2D, winding: f32) -> bool {
    cross(a, b, point) * winding >= 0.0
        && cross(b, c, point) * winding >= 0.0
        && cross(c, a, point) * winding >= 0.0
}

fn eval_cubic(points: [Vec2D; 4], t: f32) -> Vec2D {
    let ab = lerp(points[0], points[1], t);
    let bc = lerp(points[1], points[2], t);
    let cd = lerp(points[2], points[3], t);
    let abc = lerp(ab, bc, t);
    let bcd = lerp(bc, cd, t);
    lerp(abc, bcd, t)
}

fn lerp(a: Vec2D, b: Vec2D, t: f32) -> Vec2D {
    Vec2D::new((b.x - a.x).mul_add(t, a.x), (b.y - a.y).mul_add(t, a.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_stroke_tessellation_bytes_eq(
        actual: &StrokeTessellation,
        expected: &StrokeTessellation,
    ) {
        assert_eq!(
            actual.local_contour_ids_are_dense,
            expected.local_contour_ids_are_dense
        );
        let actual = &actual.tessellation;
        let expected = &expected.tessellation;
        assert_eq!(actual.base_instance, expected.base_instance);
        assert_eq!(actual.instance_count, expected.instance_count);
        assert_eq!(
            bytemuck::bytes_of(&actual.path),
            bytemuck::bytes_of(&expected.path)
        );
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&actual.spans),
            bytemuck::cast_slice::<_, u8>(&expected.spans)
        );
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&actual.contours),
            bytemuck::cast_slice::<_, u8>(&expected.contours)
        );
    }

    #[test]
    fn shared_stroke_preparation_scratch_is_byte_identical_to_fresh_scratch() {
        let mut path = RawPath::new();
        path.move_to(2.0, 3.0);
        path.cubic_to(20.0, -7.0, -4.0, 31.0, 38.0, 17.0);
        path.line_to(9.0, 41.0);
        path.close();
        path.move_to(50.0, 11.0);
        path.cubic_to(75.0, 44.0, 23.0, -19.0, 91.0, 37.0);

        let expected = build_stroke_tessellation_with_layout(
            &path,
            Mat2D([1.25, 0.125, -0.25, 0.75, 4.0, -3.0]),
            5.5,
            StrokeJoin::Round,
            StrokeCap::Square,
        )
        .unwrap();
        let mut scratch = StrokePreparationScratch::default();
        let mut previous_path = RawPath::new();
        previous_path.move_to(-100.0, -90.0);
        previous_path.line_to(-80.0, -70.0);
        previous_path.cubic_to(-60.0, -50.0, -40.0, -30.0, -20.0, -10.0);
        previous_path.move_to(110.0, 120.0);
        previous_path.line_to(130.0, 140.0);
        previous_path.move_to(210.0, 220.0);
        previous_path.cubic_to(230.0, 240.0, 250.0, 260.0, 270.0, 280.0);
        build_stroke_tessellation_with_layout_using_scratch(
            &previous_path,
            Mat2D::IDENTITY,
            13.0,
            StrokeJoin::Miter,
            StrokeCap::Round,
            &mut scratch,
        )
        .unwrap();
        let actual = build_stroke_tessellation_with_layout_using_scratch(
            &path,
            Mat2D([1.25, 0.125, -0.25, 0.75, 4.0, -3.0]),
            5.5,
            StrokeJoin::Round,
            StrokeCap::Square,
            &mut scratch,
        )
        .unwrap();

        assert_stroke_tessellation_bytes_eq(&actual, &expected);
    }

    fn assert_stroke_curve_bits_eq(actual: &StrokeCurve, expected: &StrokeCurve) {
        assert_eq!(actual.is_line, expected.is_line);
        for (actual, expected) in actual.cubic.iter().zip(expected.cubic.iter()) {
            assert_eq!(actual.x.to_bits(), expected.x.to_bits());
            assert_eq!(actual.y.to_bits(), expected.y.to_bits());
        }
    }

    fn assert_prepared_stroke_curve_bits_eq(
        actual: &PreparedStrokeCurve,
        expected: &PreparedStrokeCurve,
    ) {
        for (actual, expected) in actual.cubic.iter().zip(expected.cubic.iter()) {
            assert_eq!(actual.x.to_bits(), expected.x.to_bits());
            assert_eq!(actual.y.to_bits(), expected.y.to_bits());
        }
        for (actual, expected) in actual.tangents.iter().zip(expected.tangents.iter()) {
            assert_eq!(actual.x.to_bits(), expected.x.to_bits());
            assert_eq!(actual.y.to_bits(), expected.y.to_bits());
        }
        assert_eq!(
            actual.original_start_tangent.x.to_bits(),
            expected.original_start_tangent.x.to_bits()
        );
        assert_eq!(
            actual.original_start_tangent.y.to_bits(),
            expected.original_start_tangent.y.to_bits()
        );
        assert_eq!(actual.parametric_segments, expected.parametric_segments);
        assert_eq!(actual.polar_segments, expected.polar_segments);
        assert_eq!(actual.ends_original_curve, expected.ends_original_curve);
    }

    #[test]
    fn stroke_contour_normalization_matches_clone_filter_and_close_bit_for_bit() {
        let nan = f32::from_bits(0x7fc0_0042);
        let first = Vec2D::new(-0.0, 11.0);
        let current = Vec2D::new(0.0, 11.0);
        let mut contour = StrokeContour {
            curves: vec![
                StrokeCurve {
                    cubic: line_cubic(Vec2D::new(2.0, 3.0), Vec2D::new(5.0, 7.0)),
                    is_line: true,
                },
                StrokeCurve {
                    cubic: [Vec2D::new(9.0, -0.0); 4],
                    is_line: false,
                },
                StrokeCurve {
                    cubic: [Vec2D::new(nan, 13.0); 4],
                    is_line: false,
                },
            ]
            .into(),
            first,
            current,
            closed: true,
        };

        // This is the allocation-heavy sequence used before normalization was
        // moved in place.
        let mut expected = contour.curves.clone();
        expected.retain(|curve| {
            let [p0, p1, p2, p3] = curve.cubic;
            !(points_equal(p0, p1) && points_equal(p1, p2) && points_equal(p2, p3))
        });
        if contour.closed && !same_point(contour.first, contour.current) {
            expected.push(StrokeCurve {
                cubic: line_cubic(contour.current, contour.first),
                is_line: true,
            });
        }

        normalize_stroke_contour_curves(&mut contour);

        assert_eq!(contour.curves.len(), expected.len());
        for (actual, expected) in contour.curves.iter().zip(expected.iter()) {
            assert_stroke_curve_bits_eq(actual, expected);
        }
    }

    #[test]
    fn scalar_line_preparation_matches_single_element_chop_bit_for_bit() {
        let curve = StrokeCurve {
            cubic: line_cubic(Vec2D::new(-0.0, 1.0e20), Vec2D::new(0.0, -1.0e20)),
            is_line: true,
        };

        // Mirror the old vec![curve.cubic] path, including its two independent
        // tangent calculations.
        let original_tangent = subtract(curve.cubic[3], curve.cubic[0]);
        let chopped = vec![curve.cubic];
        let cubic = chopped.into_iter().next().unwrap();
        let tangent = subtract(cubic[3], cubic[0]);
        let expected = PreparedStrokeCurve {
            cubic,
            tangents: [tangent, tangent],
            original_start_tangent: original_tangent,
            parametric_segments: 1,
            polar_segments: 1,
            ends_original_curve: true,
        };

        let actual = prepare_line_curve(&curve);
        assert_prepared_stroke_curve_bits_eq(&actual, &expected);
    }

    #[test]
    fn contour_midpoint_matches_projected_cubic_endpoints_bit_for_bit() {
        let curves = [
            StrokeCurve {
                cubic: [
                    Vec2D::new(-2.0, 4.0),
                    Vec2D::new(8.0, -16.0),
                    Vec2D::new(32.0, 64.0),
                    Vec2D::new(1.0e20, -1.0e20),
                ],
                is_line: false,
            },
            StrokeCurve {
                cubic: [
                    Vec2D::new(3.0, 5.0),
                    Vec2D::new(7.0, 11.0),
                    Vec2D::new(13.0, 17.0),
                    Vec2D::new(-1.0e20, 1.0e20),
                ],
                is_line: false,
            },
            StrokeCurve {
                cubic: [
                    Vec2D::new(19.0, 23.0),
                    Vec2D::new(29.0, 31.0),
                    Vec2D::new(37.0, 41.0),
                    Vec2D::new(3.25, -7.5),
                ],
                is_line: true,
            },
        ];
        let projected = curves.iter().map(|curve| curve.cubic).collect::<Vec<_>>();
        let mut old_sum = Vec2D::new(0.0, 0.0);
        for cubic in &projected {
            old_sum.x += cubic[3].x;
            old_sum.y += cubic[3].y;
        }
        let old_scale = 1.0 / projected.len() as f32;
        let expected = Vec2D::new(old_sum.x * old_scale, old_sum.y * old_scale);

        let actual = contour_midpoint(&curves);
        assert_eq!(actual.x.to_bits(), expected.x.to_bits());
        assert_eq!(actual.y.to_bits(), expected.y.to_bits());
    }

    #[test]
    fn wang_segment_count_matches_cpp_formula() {
        let line = [
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(2.0, 0.0),
            Vec2D::new(3.0, 0.0),
        ];
        assert_eq!(cubic_segment_count(line), 1);

        let curve = [
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.0, 100.0),
            Vec2D::new(100.0, 100.0),
            Vec2D::new(100.0, 0.0),
        ];
        assert_eq!(cubic_segment_count(curve), 21);
        assert_eq!(coarse_cubic_segment_count(curve), 4);
    }

    #[test]
    fn wang_segment_count_applies_only_the_linear_transform() {
        let overstroke_quad = [
            Vec2D::new(100.0, 0.0),
            Vec2D::new(66.666_664, -26.666_668),
            Vec2D::new(33.333_336, -26.666_668),
            Vec2D::new(0.0, 0.0),
        ];
        let scaled = Mat2D([0.2, 0.0, 0.0, 0.2, 0.0, 0.0]);
        let translated = Mat2D([0.2, 0.0, 0.0, 0.2, 290.0, 80.0]);
        let skewed = Mat2D([0.0, 0.1, 0.2, 0.0, 0.0, 0.0]);
        let translated_skew = Mat2D([0.0, 0.1, 0.2, 0.0, -410.0, 730.0]);

        for transform in [scaled, translated, skewed, translated_skew] {
            assert_eq!(
                cubic_segment_count_with_precision_and_transform(
                    overstroke_quad,
                    PARAMETRIC_PRECISION as f32,
                    transform,
                ),
                4
            );
        }
    }

    #[test]
    fn coarse_area_preserves_cpp_stream_order_for_cancelling_contours() {
        let mut path = RawPath::new();
        path.move_to(1.0, 0.0);
        path.line_to(-0.500_000_06, -0.866_025_4);
        path.line_to(-0.499_999_9, 0.866_025_45);
        path.move_to(-1.0, -8.742_278e-8);
        path.line_to(0.499_999_9, -0.866_025_45);
        path.line_to(0.500_000_06, 0.866_025_4);

        assert!(path_coarse_area(&path) < 0.0);
        assert!(clockwise_atomic_negate_coverage(
            &path,
            Mat2D::IDENTITY,
            FillRule::EvenOdd,
            true,
        ));
    }

    #[test]
    fn msaa_fill_orientation_matches_cpp_fill_rule_contract() {
        let mut clockwise = RawPath::new();
        clockwise.move_to(0.0, 0.0);
        clockwise.line_to(10.0, 0.0);
        clockwise.line_to(10.0, 10.0);
        clockwise.line_to(0.0, 10.0);
        clockwise.close();
        let mirrored = Mat2D([-1.0, 0.0, 0.0, 1.0, 10.0, 0.0]);
        let mut counterclockwise = RawPath::new();
        counterclockwise.add_path_backwards(&clockwise, Mat2D::IDENTITY);

        assert!(!msaa_fill_requires_reverse(
            &clockwise,
            Mat2D::IDENTITY,
            FillRule::NonZero,
        ));
        assert!(msaa_fill_requires_reverse(
            &counterclockwise,
            Mat2D::IDENTITY,
            FillRule::NonZero,
        ));
        assert!(msaa_fill_requires_reverse(
            &clockwise,
            mirrored,
            FillRule::NonZero,
        ));
        assert!(!msaa_fill_requires_reverse(
            &clockwise,
            Mat2D::IDENTITY,
            FillRule::Clockwise,
        ));
        assert!(msaa_fill_requires_reverse(
            &clockwise,
            mirrored,
            FillRule::Clockwise,
        ));
        assert!(!msaa_fill_requires_reverse(
            &counterclockwise,
            mirrored,
            FillRule::EvenOdd,
        ));
    }

    #[test]
    fn flatten_path_preserves_contours_closure_and_transform() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(2.0, 0.0);
        path.close();
        path.move_to(3.0, 4.0);
        path.cubic_to(3.0, 4.0, 5.0, 6.0, 7.0, 8.0);

        let contours = flatten_path(&path, Mat2D([2.0, 0.0, 0.0, 2.0, 10.0, 20.0]));
        assert_eq!(contours.len(), 2);
        assert!(contours[0].closed);
        assert!(!contours[1].closed);
        assert_eq!(contours[0].points[0], Vec2D::new(10.0, 20.0));
        assert_eq!(contours[0].points[1], Vec2D::new(14.0, 20.0));
        assert_eq!(contours[1].points.last(), Some(&Vec2D::new(24.0, 36.0)));
    }

    #[test]
    fn triangulates_concave_contours_in_either_winding() {
        let points = [
            Vec2D::new(0.0, 0.0),
            Vec2D::new(4.0, 0.0),
            Vec2D::new(4.0, 4.0),
            Vec2D::new(2.0, 2.0),
            Vec2D::new(0.0, 4.0),
        ];
        let indices = triangulate_contour(&points).unwrap();
        assert_eq!(indices.len(), 9);
        assert!(indices.iter().all(|index| *index < points.len() as u32));

        let reversed = points.iter().copied().rev().collect::<Vec<_>>();
        assert_eq!(triangulate_contour(&reversed).unwrap().len(), 9);
    }

    #[test]
    fn fill_preparation_prunes_numeric_zero_length_lines() {
        let mut path = RawPath::new();
        path.move_to(0.0, -0.0);
        path.line_to(-0.0, 0.0);
        path.line_to(1.0, 0.0);
        path.line_to(1.0, 0.0);
        path.close();

        let contours = fill_cubic_contours(&path);
        assert_eq!(contours.len(), 1);
        assert_eq!(contours[0].len(), 2);
    }

    #[test]
    fn removes_repeated_endpoint_from_closed_cubic_contour() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.cubic_to(10.0, 0.0, 10.0, 10.0, 0.0, 0.0);
        path.close();
        let contour = flatten_path(&path, Mat2D::IDENTITY).remove(0);
        assert_ne!(contour.points.first(), contour.points.last());
        assert!(triangulate_contour(&contour.points).is_some());
    }

    fn geometry_spans(tessellation: &FillTessellation) -> impl Iterator<Item = &TessVertexSpan> {
        tessellation
            .spans
            .iter()
            .filter(|span| span.contour_id_with_flags & CONTOUR_ID_MASK != 0)
    }

    fn assert_post_contour_padding(tessellation: &FillTessellation) {
        let logical_end = (tessellation.base_instance + tessellation.instance_count)
            * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let location = align_up(logical_end as i32, OUTER_CURVE_PATCH_SEGMENT_SPAN as i32) as u32;
        let expected_range = (
            location as i32 % TESS_TEXTURE_WIDTH,
            location as i32 % TESS_TEXTURE_WIDTH + 1,
        );
        let padding = tessellation
            .spans
            .iter()
            .find(|span| {
                span.contour_id_with_flags == 0
                    && span.segment_counts == 0x0010_0000
                    && span.x_range() == expected_range
            })
            .expect("final C++-ordered padding span");
        assert_eq!(padding.points, [[0.0; 2]; 4]);
        assert_eq!(padding.join_tangent, [0.0; 2]);
        assert_eq!(padding.y, (location as i32 / TESS_TEXTURE_WIDTH) as f32);
        assert_eq!(padding.x_range(), expected_range);
        assert_eq!(padding.segment_counts, 0x0010_0000);
        assert_eq!(padding.contour_id_with_flags, 0);
        assert!(
            tessellation
                .spans
                .iter()
                .filter(|span| {
                    span.segment_counts == 0x0010_0000 && span.contour_id_with_flags == 0
                })
                .count()
                >= 1
        );
    }

    #[test]
    fn fill_tessellation_obeys_eight_vertex_patch_layout() {
        let mut path = RawPath::new();
        path.move_to(4.0, 4.0);
        path.line_to(60.0, 4.0);
        path.line_to(32.0, 60.0);
        path.close();
        let tessellation = build_fill_tessellation(&path, Mat2D::IDENTITY).unwrap();
        assert_eq!(tessellation.base_instance, 1);
        assert_eq!(tessellation.instance_count, 1);
        assert_eq!(tessellation.contours.len(), 1);
        assert_eq!(tessellation.contours[0].vertex_index0, 8);
        assert_eq!(tessellation.spans.len(), 6);
        assert_eq!(tessellation.spans[0].x0_x1 as u32, 0x0008_0000);
        let geometry = geometry_spans(&tessellation).collect::<Vec<_>>();
        assert_eq!(geometry[0].x0_x1 as u32, 0x000c_0008);
        assert_eq!(geometry[2].x0_x1 as u32, 0x0010_000e);
        assert_post_contour_padding(&tessellation);
    }

    #[test]
    fn fill_tessellation_does_not_subdivide_large_lines() {
        let mut path = RawPath::new();
        path.move_to(-1.0e9, -1.0e9);
        path.line_to(1.0e9, -1.0e9);
        path.line_to(1.0e9, 1.0e9);
        path.line_to(-1.0e9, 1.0e9);
        path.close();

        let tessellation =
            build_fill_tessellation(&path, Mat2D([1.0, 0.0, 0.0, 1.0, 258.0, 10_365_663.0]))
                .unwrap();

        assert_eq!(tessellation.instance_count, 1);
    }

    #[test]
    fn fill_tessellation_wraps_locations_before_i16_packing() {
        let mut path = RawPath::new();
        for x in 0..4_100 {
            let x = x as f32;
            path.move_to(x, 0.0);
            path.line_to(x + 0.5, 1.0);
            path.line_to(x + 1.0, 0.0);
            path.close();
        }
        let tessellation = build_fill_tessellation(&path, Mat2D::IDENTITY).unwrap();

        assert!(tessellation.spans.iter().any(|span| span.y >= 16.0));
        assert!(tessellation
            .spans
            .iter()
            .all(|span| span.x_range().0 >= -TESS_TEXTURE_WIDTH));
    }

    #[test]
    fn atomic_fill_layout_packs_reverse_then_forward_halves() {
        let mut path = RawPath::new();
        path.move_to(4.0, 4.0);
        path.line_to(60.0, 4.0);
        path.line_to(32.0, 60.0);
        path.close();
        let mut tessellation = build_fill_tessellation(&path, Mat2D::IDENTITY).unwrap();
        tessellation.make_double_sided();
        assert_eq!(tessellation.base_instance, 1);
        assert_eq!(tessellation.instance_count, 2);
        assert_eq!(tessellation.contours[0].vertex_index0, 16);
        let geometry = geometry_spans(&tessellation).collect::<Vec<_>>();
        assert_eq!(geometry[0].x_range(), (16, 20));
        assert_eq!(geometry[0].reflection_x0_x1 as u32, 0x000c_0010);
        assert_post_contour_padding(&tessellation);
    }

    #[test]
    fn mirrored_clockwise_fill_packs_forward_then_reverse_halves() {
        let mut path = RawPath::new();
        path.move_to(4.0, 4.0);
        path.line_to(60.0, 4.0);
        path.line_to(32.0, 60.0);
        path.close();
        let mut tessellation =
            build_fill_tessellation(&path, Mat2D([-1.0, 0.0, 0.0, 1.0, 64.0, 0.0])).unwrap();
        tessellation.make_double_sided_with_direction(true);

        assert_eq!(tessellation.instance_count, 2);
        assert_eq!(tessellation.contours[0].vertex_index0, 8);
        let geometry = geometry_spans(&tessellation).collect::<Vec<_>>();
        assert_eq!(geometry[0].x_range(), (8, 12));
        assert_eq!(geometry[0].reflection_x0_x1 as u32, 0x0014_0018);
        assert_ne!(
            geometry[0].contour_id_with_flags & NEGATE_PATH_FILL_COVERAGE_FLAG,
            0
        );
        assert_post_contour_padding(&tessellation);
    }

    #[test]
    fn clockwise_atomic_coverage_range_matches_cpp_visible_bounds_tiling() {
        let mut path = RawPath::new();
        path.move_to(10.25, 20.75);
        path.cubic_to(12.0, -8.5, 80.25, 75.5, 50.1, 70.2);
        path.close();
        let (range, word_count) =
            clockwise_atomic_coverage_range(&path, Mat2D::IDENTITY, 64, 64, 1024).unwrap();
        assert_eq!(range.offset, 1024);
        assert_eq!(range.pitch, 64);
        assert_eq!(range.offset_x, -8.0);
        assert_eq!(range.offset_y, 2.0);
        assert_eq!(word_count, 64 * 96);
    }

    #[test]
    fn clockwise_atomic_coverage_range_rejects_offscreen_paths() {
        let mut path = RawPath::new();
        path.move_to(-20.0, -20.0);
        path.line_to(-10.0, -20.0);
        path.line_to(-10.0, -10.0);
        path.close();
        assert!(clockwise_atomic_coverage_range(&path, Mat2D::IDENTITY, 64, 64, 0).is_none());
    }

    #[test]
    fn interior_layout_emits_outer_patches_and_weighted_triangles() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(100.0, 0.0);
        path.line_to(100.0, 100.0);
        path.line_to(0.0, 100.0);
        path.close();
        let tessellation =
            build_interior_tessellation(&path, Mat2D::IDENTITY, FillRule::NonZero, false).unwrap();
        assert_eq!(tessellation.spans.len(), 6);
        assert_eq!(tessellation.base_instance, 1);
        assert_eq!(tessellation.instance_count, 8);
        assert_eq!(tessellation.triangles.len(), 6);
        assert_eq!(tessellation.triangles[0].weight_path_id >> 16, 1);
        assert_eq!(tessellation.triangles[0].weight_path_id as u16, 1);
        assert_eq!(tessellation.contours[0].vertex_index0, 85);
        assert_eq!(
            tessellation.spans[1].contour_id_with_flags,
            CULL_EXCESS_TESSELLATION_SEGMENTS_CONTOUR_FLAG | 1
        );
    }

    #[test]
    fn mirrored_clockwise_interior_uses_forward_then_reverse_layout() {
        let mut path = RawPath::new();
        path.move_to(1600.0, 0.0);
        path.line_to(0.0, 0.0);
        path.line_to(0.0, 1600.0);
        path.line_to(1600.0, 1600.0);
        path.close();
        for x in [800.0, 0.0, 800.0] {
            path.move_to(x + 50.0, 640.0);
            path.cubic_to(x + 50.0, 0.0, x + 750.0, 0.0, x + 750.0, 640.0);
            path.cubic_to(x + 750.0, 1600.0, x + 50.0, 1600.0, x + 50.0, 640.0);
        }

        let positive = build_interior_tessellation(
            &path,
            Mat2D([1.0, 0.0, 0.0, 1.0, 29.0, -100.0]),
            FillRule::Clockwise,
            false,
        )
        .unwrap();
        let mirrored = build_interior_tessellation(
            &path,
            Mat2D([-1.0, 0.0, 0.0, 1.0, 1593.0, 207.0]),
            FillRule::Clockwise,
            false,
        )
        .unwrap();

        assert_eq!(positive.contours[0].vertex_index0, 493);
        assert_eq!(mirrored.contours[0].vertex_index0, 17);
        assert_ne!(
            positive.spans[1].reflection_x0_x1,
            mirrored.spans[1].reflection_x0_x1
        );
    }

    #[test]
    fn interior_triangle_visitor_assigns_path_ids_and_preserves_face_order() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(20.0, 0.0);
        path.line_to(20.0, 20.0);
        path.line_to(0.0, 20.0);
        path.close();
        let mut tessellation =
            build_interior_tessellation(&path, Mat2D::IDENTITY, FillRule::NonZero, false).unwrap();
        let triangle = |weight, source_path_id, x| {
            [
                TriangleVertex::new([x, 0.0], weight, source_path_id),
                TriangleVertex::new([x + 1.0, 0.0], weight, source_path_id),
                TriangleVertex::new([x, 1.0], weight, source_path_id),
            ]
        };
        tessellation.triangles = [
            triangle(-2, 1, 0.0),
            triangle(0, 2, 2.0),
            triangle(3, 3, 4.0),
            triangle(-1, 4, 6.0),
            triangle(1, 5, 8.0),
        ]
        .concat();

        for faces in [
            WindingFaces::Negative,
            WindingFaces::Positive,
            WindingFaces::All,
        ] {
            let mut actual = Vec::new();
            tessellation
                .visit_triangles(23, faces, |_, triangle| actual.extend_from_slice(&triangle));
            let expected = tessellation
                .triangles
                .chunks_exact(3)
                .filter(|triangle| faces.includes((triangle[0].weight_path_id >> 16) as i16))
                .flat_map(|triangle| {
                    triangle.iter().copied().map(|mut vertex| {
                        vertex.weight_path_id = (vertex.weight_path_id & !0xffff) | 23;
                        vertex
                    })
                })
                .collect::<Vec<_>>();
            assert_eq!(
                bytemuck::cast_slice::<_, u8>(&actual),
                bytemuck::cast_slice::<_, u8>(&expected),
                "{faces:?}"
            );
        }
    }

    #[test]
    fn interior_selection_matches_upstream_area_threshold() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(512.0, 0.0);
        path.line_to(512.0, 512.0);
        path.close();
        assert!(!should_use_interior_tessellation(&path, Mat2D::IDENTITY));
        assert!(should_use_interior_tessellation(
            &path,
            Mat2D([1.01, 0.0, 0.0, 1.0, 0.0, 0.0])
        ));
    }

    #[test]
    fn interior_layout_chops_large_cubics_into_outer_patches() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.cubic_to(0.0, 100.0, 100.0, 100.0, 100.0, 0.0);
        path.close();
        let tessellation = build_interior_tessellation(
            &path,
            Mat2D([100.0, 0.0, 0.0, 100.0, 0.0, 0.0]),
            FillRule::NonZero,
            false,
        )
        .unwrap();
        assert!(tessellation.spans.len() > 3);
        assert_eq!(
            tessellation.instance_count as usize,
            (tessellation.spans.len() - 2) * 2
        );
    }

    #[test]
    fn open_butt_line_stroke_packs_caps_into_two_midpoint_patches() {
        let mut path = RawPath::new();
        path.move_to(10.0, 20.0);
        path.line_to(50.0, 20.0);
        let tessellation = build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            20.0,
            StrokeJoin::Miter,
            StrokeCap::Butt,
        )
        .unwrap();
        assert_eq!(tessellation.path.stroke_radius, 10.0);
        assert_eq!(tessellation.instance_count, 2);
        assert_eq!(tessellation.contours[0].midpoint, [0.0, 0.0]);
        assert_eq!(tessellation.contours[0].vertex_index0, 8);
        assert_eq!(tessellation.spans.len(), 5);
        let geometry = geometry_spans(&tessellation).collect::<Vec<_>>();
        assert_eq!(geometry.len(), 2);
        assert_eq!(geometry[0].x_range(), (8, 18));
        assert_eq!(geometry[1].x_range(), (18, 24));
        assert_eq!(
            geometry[0].contour_id_with_flags,
            1 | BEVEL_JOIN_CONTOUR_FLAG | EMULATED_STROKE_CAP_CONTOUR_FLAG
        );
        assert_post_contour_padding(&tessellation);
    }

    #[test]
    fn simple_cubic_stroke_uses_analytic_curve_budgets() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.cubic_to(0.0, 10.0, 10.0, 10.0, 10.0, 0.0);
        let tessellation = build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            2.0,
            StrokeJoin::Round,
            StrokeCap::Round,
        )
        .unwrap();
        let cubic = geometry_spans(&tessellation)
            .find(|span| span.segment_counts & 1023 > 1)
            .expect("analytic cubic span");
        assert!(cubic.segment_counts >> 10 & 1023 > 1);
    }

    #[test]
    fn feather_fill_uses_closed_double_sided_join_geometry() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(12.0, 0.0);
        path.line_to(12.0, 6.0);
        let tessellation = build_feather_tessellation(&path, Mat2D::IDENTITY, 10.0, None).unwrap();

        assert_eq!(tessellation.path.stroke_radius, 0.0);
        assert_eq!(tessellation.path.feather_radius, 15.0);
        assert_eq!(tessellation.contours[0].midpoint, [8.0, 2.0]);
        assert_eq!(tessellation.contours[0].vertex_index0, 64);
        assert_eq!(tessellation.instance_count % 2, 0);
        assert!(geometry_spans(&tessellation)
            .all(|span| span.contour_id_with_flags & FEATHER_JOIN_CONTOUR_FLAG != 0));
    }

    #[test]
    fn feather_fill_preserves_empty_move_contours_for_gpu_ids() {
        let mut path = RawPath::new();
        path.move_to(0.0, 100.0);
        path.move_to(0.0, 100.0);
        path.cubic_to(133.635864, 0.0, -33.6358566, 0.0, 100.0, 100.0);

        let tessellation = build_feather_tessellation(
            &path,
            Mat2D([1.46300006, 0.0, 0.0, 1.46300006, 0.0, 0.0]),
            1.0,
            None,
        )
        .unwrap();

        assert_eq!(tessellation.contours.len(), 2);
        assert!(tessellation.contours[0].midpoint[0].is_nan());
        assert!(tessellation.contours[0].midpoint[1].is_nan());
        assert_eq!(tessellation.contours[0].vertex_index0, 88);
        assert!(tessellation
            .spans
            .iter()
            .any(|span| { span.contour_id_with_flags & CONTOUR_ID_MASK == 2 }));
    }

    #[test]
    fn double_sided_feather_wraps_forward_and_mirrored_rows_together() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        for index in 1..=320 {
            path.line_to(index as f32, (index & 1) as f32);
        }
        path.close();

        let tessellation = build_feather_tessellation(&path, Mat2D::IDENTITY, 1.0, None).unwrap();
        let base = tessellation.base_instance * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let half_vertex_count =
            tessellation.instance_count / 2 * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        assert!(half_vertex_count > TESS_TEXTURE_WIDTH as u32);

        let center = (base + half_vertex_count) as i32;
        let mut saw_wrapped_record = false;
        let mut saw_wrapped_reflection_row = false;
        for span in tessellation
            .spans
            .iter()
            .filter(|span| span.contour_id_with_flags & CONTOUR_ID_MASK != 0)
        {
            let (x0, x1) = span.x_range();
            let reflection_x0 = span.reflection_x0_x1 as i16 as i32;
            let reflection_x1 = (span.reflection_x0_x1 >> 16) as i16 as i32;
            let forward_location = span.y as i32 * TESS_TEXTURE_WIDTH + x0;
            let reflection_y = if span.reflection_y == u32::MAX as f32 {
                saw_wrapped_reflection_row = true;
                -1
            } else {
                span.reflection_y as i32
            };
            let reflection_location = reflection_y * TESS_TEXTURE_WIDTH + reflection_x0;
            assert_eq!(forward_location + reflection_location, center * 2);
            assert_eq!(x1 - x0, reflection_x0 - reflection_x1);
            saw_wrapped_record |= x0 < 0
                || x1 > TESS_TEXTURE_WIDTH
                || reflection_x0 > TESS_TEXTURE_WIDTH
                || reflection_x1 < 0;
        }
        assert!(saw_wrapped_record);
        assert!(saw_wrapped_reflection_row);
    }

    #[test]
    fn feather_atlas_fill_keeps_forward_contour_only() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(12.0, 0.0);
        path.line_to(12.0, 6.0);
        let direct = build_feather_tessellation(&path, Mat2D::IDENTITY, 10.0, None).unwrap();
        let atlas = build_feather_atlas_tessellation(&path, Mat2D::IDENTITY, 10.0, None).unwrap();
        assert_eq!(atlas.contours[0].vertex_index0, 8);
        assert_eq!(atlas.instance_count * 2, direct.instance_count);
    }

    #[test]
    fn mirrored_feather_fill_uses_cpp_contour_directions() {
        let mut path = RawPath::new();
        path.move_to(4.0, 4.0);
        path.line_to(60.0, 4.0);
        path.line_to(32.0, 60.0);
        path.close();
        let transform = Mat2D([-1.0, 0.0, 0.0, 1.0, 64.0, 0.0]);

        assert_eq!(
            feather_atlas_fill_direction(transform, FillRule::Clockwise, false),
            FeatherFillDirection::Reverse
        );
        assert_eq!(
            feather_atlas_fill_direction(transform, FillRule::NonZero, false),
            FeatherFillDirection::Forward
        );
        assert_eq!(
            feather_atlas_fill_direction(transform, FillRule::Clockwise, true),
            FeatherFillDirection::Forward
        );

        let direct = build_feather_tessellation_with_direction(
            &path,
            transform,
            2.0,
            None,
            FeatherFillDirection::ForwardThenReverse,
        )
        .unwrap();
        assert_eq!(direct.contours[0].vertex_index0, 8);
        assert!(direct
            .spans
            .iter()
            .filter(|span| span.contour_id_with_flags & CONTOUR_ID_MASK != 0)
            .all(|span| span.contour_id_with_flags & NEGATE_PATH_FILL_COVERAGE_FLAG != 0));

        let atlas = build_feather_tessellation_with_direction(
            &path,
            transform,
            40.0,
            None,
            FeatherFillDirection::Reverse,
        )
        .unwrap();
        let base = atlas.base_instance * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let end = base + atlas.instance_count * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        assert_eq!(atlas.contours[0].vertex_index0, end - 1);
        assert!(atlas
            .spans
            .iter()
            .filter(|span| span.contour_id_with_flags & CONTOUR_ID_MASK != 0)
            .all(|span| {
                let (x0, x1) = span.x_range();
                x0 > x1 && span.contour_id_with_flags & NEGATE_PATH_FILL_COVERAGE_FLAG != 0
            }));
    }

    #[test]
    fn reverse_feather_atlas_wraps_descending_spans_across_rows() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        for index in 1..=320 {
            path.line_to(index as f32, (index & 1) as f32);
        }
        path.close();

        let atlas = build_feather_tessellation_with_direction(
            &path,
            Mat2D::IDENTITY,
            1.0,
            None,
            FeatherFillDirection::Reverse,
        )
        .unwrap();
        assert!(
            atlas.instance_count * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32
                > TESS_TEXTURE_WIDTH as u32
        );
        assert!(atlas
            .spans
            .iter()
            .filter(|span| span.contour_id_with_flags & CONTOUR_ID_MASK != 0)
            .all(|span| span.x_range().0 > span.x_range().1));
        assert!(atlas
            .spans
            .iter()
            .any(|span| span.x_range().1 < 0 || span.x_range().0 > TESS_TEXTURE_WIDTH));
    }

    #[test]
    fn feather_fill_softens_circle_at_uniform_tangent_rotations() {
        const CONTROL_OFFSET: f32 = 8.83064;
        let mut path = RawPath::new();
        path.move_to(48.0, 32.0);
        path.cubic_to(48.0, 40.83064, 40.83064, 48.0, 32.0, 48.0);
        path.cubic_to(32.0 - CONTROL_OFFSET, 48.0, 16.0, 40.83064, 16.0, 32.0);
        path.cubic_to(16.0, 32.0 - CONTROL_OFFSET, 23.16936, 16.0, 32.0, 16.0);
        path.cubic_to(40.83064, 16.0, 48.0, 23.16936, 48.0, 32.0);
        path.close();

        let softened = softened_path_for_feathering(&path, 30.0, 1.0);
        assert_eq!(
            softened
                .verbs()
                .iter()
                .filter(|verb| **verb == PathVerb::Cubic)
                .count(),
            12
        );
        let atlas = build_feather_atlas_tessellation(&path, Mat2D::IDENTITY, 20.0, None).unwrap();
        assert_eq!(atlas.instance_count, 34);
    }

    #[test]
    fn feather_atlas_boundary_matches_cpp_scale_factor() {
        assert!(!feather_requires_atlas(8.0, Mat2D::IDENTITY, false));
        assert!(!feather_requires_atlas(
            10.0,
            Mat2D([2.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            false
        ));
        assert!(feather_requires_atlas(
            32.0 / 3.0,
            Mat2D([2.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            false
        ));
        assert!(feather_requires_atlas(0.01, Mat2D::IDENTITY, true));
    }

    #[test]
    fn feather_softening_threshold_uses_authored_feather_without_radius_round_trip() {
        let paint_feather = 5.389_884_f32;
        let matrix_scale = 0.185_532_76_f32;

        assert!(paint_feather * matrix_scale > 1.0);
        assert_eq!((paint_feather * 1.5) / 1.5 * matrix_scale, 1.0);
        assert!(feather_fill_requires_softening(paint_feather, matrix_scale));
    }

    #[test]
    fn path_pixel_bounds_round_out_without_aa_outset() {
        let mut path = RawPath::new();
        path.move_to(1.25, 2.75);
        path.line_to(6.5, 8.125);

        assert_eq!(
            path_pixel_bounds(&path, Mat2D::IDENTITY),
            Some([1, 2, 7, 9])
        );
    }

    #[test]
    fn feather_pixel_bounds_include_transformed_radius_and_aa() {
        let mut path = RawPath::new();
        path.move_to(16.0, 16.0);
        path.line_to(48.0, 48.0);
        assert_eq!(
            feather_pixel_bounds(&path, Mat2D::IDENTITY, 80.0, None),
            Some([-105, -105, 169, 169])
        );
        assert_eq!(
            feather_pixel_bounds(&path, Mat2D([2.0, 0.0, 0.0, 0.5, 10.0, -4.0]), 4.0, None,),
            Some([29, 0, 119, 24])
        );
    }

    #[test]
    fn feather_pixel_bounds_include_cpp_stroke_outsets() {
        let mut path = RawPath::new();
        path.move_to(16.0, 16.0);
        path.line_to(48.0, 48.0);

        assert_eq!(
            feather_pixel_bounds(
                &path,
                Mat2D::IDENTITY,
                4.0,
                Some((10.0, StrokeJoin::Bevel, StrokeCap::Butt)),
            ),
            Some([4, 4, 60, 60])
        );
        assert_eq!(
            feather_pixel_bounds(
                &path,
                Mat2D::IDENTITY,
                4.0,
                Some((10.0, StrokeJoin::Miter, StrokeCap::Butt)),
            ),
            Some([-11, -11, 75, 75])
        );
        assert_eq!(
            feather_pixel_bounds(
                &path,
                Mat2D::IDENTITY,
                4.0,
                Some((10.0, StrokeJoin::Bevel, StrokeCap::Square)),
            ),
            Some([1, 1, 63, 63])
        );
    }

    #[test]
    fn feathered_stroke_keeps_stroke_join_flags_and_both_radii() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(12.0, 0.0);
        let tessellation = build_feather_tessellation(
            &path,
            Mat2D::IDENTITY,
            4.0,
            Some((10.0, StrokeJoin::Round, StrokeCap::Round)),
        )
        .unwrap();

        assert_eq!(tessellation.path.stroke_radius, 5.0);
        assert_eq!(tessellation.path.feather_radius, 6.0);
        assert_eq!(tessellation.contours[0].vertex_index0, 8);
        assert!(geometry_spans(&tessellation)
            .all(|span| { span.contour_id_with_flags & FEATHER_JOIN_CONTOUR_FLAG == 0 }));
    }

    #[test]
    fn smooth_miter_stroke_joins_keep_cpp_five_segment_budget() {
        let mut path = RawPath::new();
        path.move_to(100.0, 50.0);
        path.cubic_to(100.0, 75.0, 75.0, 100.0, 50.0, 100.0);
        path.cubic_to(25.0, 100.0, 0.0, 75.0, 0.0, 50.0);
        path.cubic_to(0.0, 25.0, 25.0, 0.0, 50.0, 0.0);
        path.cubic_to(75.0, 0.0, 100.0, 25.0, 100.0, 50.0);
        path.close();

        let tessellation = build_feather_atlas_tessellation(
            &path,
            Mat2D::IDENTITY,
            4.0,
            Some((10.0, StrokeJoin::Miter, StrokeCap::Butt)),
        )
        .unwrap();
        let first_geometry = tessellation
            .spans
            .iter()
            .position(|span| span.contour_id_with_flags & CONTOUR_ID_MASK != 0)
            .unwrap();
        assert!(tessellation.spans[..first_geometry]
            .iter()
            .all(|span| span.contour_id_with_flags == 0));
        assert!(tessellation.spans[first_geometry..]
            .iter()
            .all(|span| span.contour_id_with_flags & CONTOUR_ID_MASK != 0));
        assert!(tessellation
            .spans
            .iter()
            .filter(|span| span.contour_id_with_flags & CONTOUR_ID_MASK != 0)
            .all(|span| span.segment_counts >> 20 == 5));
    }

    #[test]
    fn cusp_cubic_stroke_emits_straddled_pivot_curve() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.cubic_to(100.0, 0.0, -100.0, 0.0, 0.0, 0.0);
        let tessellation = build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            2.0,
            StrokeJoin::Round,
            StrokeCap::Round,
        )
        .unwrap();
        assert_eq!(tessellation.spans.len(), 9);
        let pivots = geometry_spans(&tessellation)
            .filter(|span| span.points[0] == span.points[3] && span.points[1] == span.points[2])
            .count();
        assert_eq!(pivots, 2);
    }

    #[test]
    fn degenerate_cubic_chops_carry_cpp_join_tangents() {
        let mut tricky = RawPath::new();
        tricky.move_to(1.0, 1.0);
        tricky.cubic_to(1.66666675, 1.0, 1.66666675, 1.0, 1.0, 1.0);
        let tricky = build_stroke_tessellation(
            &tricky,
            Mat2D([3.32997298, 0.0, 0.0, 3.32997298, 0.0, 0.0]),
            9.00908184,
            StrokeJoin::Miter,
            StrokeCap::Butt,
        )
        .unwrap();
        let tricky_geometry = geometry_spans(&tricky).collect::<Vec<_>>();
        assert_eq!(tricky_geometry.len(), 4);
        assert_eq!(tricky_geometry[1].join_tangent, [0.0, 1.0]);
        assert_eq!(tricky_geometry[2].join_tangent, [0.0, 1.0]);
        assert_eq!(tricky_geometry[3].join_tangent, [0.66666675, 0.0]);

        let mut turnaround = RawPath::new();
        turnaround.move_to(0.0, 0.0);
        turnaround.cubic_to(0.0, -10.0, 0.0, -10.0, 0.0, 10.0);
        let turnaround = build_stroke_tessellation(
            &turnaround,
            Mat2D::IDENTITY,
            100.0,
            StrokeJoin::Miter,
            StrokeCap::Butt,
        )
        .unwrap();
        let turnaround_geometry = geometry_spans(&turnaround).collect::<Vec<_>>();
        assert_eq!(turnaround_geometry.len(), 4);
        assert_eq!(turnaround_geometry[1].join_tangent, [0.0, 1.0]);
        assert_eq!(turnaround_geometry[2].join_tangent, [0.0, 1.0]);
        assert_eq!(turnaround_geometry[3].join_tangent, [0.0, -20.0]);
    }

    #[test]
    fn degenerate_cubic_single_chop_matches_cpp_mix_bits() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.cubic_to(0.0, -10.0, 10.0, 10.0, 0.0, 10.0);
        let tessellation = build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            100.0,
            StrokeJoin::Miter,
            StrokeCap::Butt,
        )
        .unwrap();
        let geometry = geometry_spans(&tessellation).collect::<Vec<_>>();
        assert_eq!(geometry.len(), 3);
        assert_eq!(geometry[0].points[0][1].to_bits(), 0x40a5ed0a);
        assert_eq!(geometry[0].points[1][1].to_bits(), 0x350aaaab);
        assert_eq!(geometry[1].points[2][1].to_bits(), 0x350aaaab);
        assert_eq!(geometry[2].join_tangent, [10.0, 0.0]);
    }

    #[test]
    fn empty_round_stroke_emits_opposed_cap_joins() {
        let mut path = RawPath::new();
        path.move_to(20.0, 30.0);
        let tessellation = build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            10.0,
            StrokeJoin::Bevel,
            StrokeCap::Round,
        )
        .unwrap();
        assert_eq!(tessellation.spans.len(), 5);
        let geometry = geometry_spans(&tessellation).collect::<Vec<_>>();
        assert_eq!(geometry.len(), 2);
        assert_eq!(geometry[0].points[3], [20.0, 30.0]);
        assert_eq!(geometry[1].points[3], [20.0, 30.0]);
        assert_eq!(
            geometry[0].contour_id_with_flags,
            1 | ROUND_JOIN_CONTOUR_FLAG | EMULATED_STROKE_CAP_CONTOUR_FLAG
        );
    }

    #[test]
    fn zero_length_cubic_stroke_uses_empty_cap_geometry() {
        let mut path = RawPath::new();
        path.move_to(20.0, 30.0);
        path.cubic_to(20.0, 30.0, 20.0, 30.0, 20.0, 30.0);

        for (cap, flags) in [
            (StrokeCap::Round, ROUND_JOIN_CONTOUR_FLAG),
            (StrokeCap::Square, MITER_CLIP_JOIN_CONTOUR_FLAG),
        ] {
            let tessellation =
                build_stroke_tessellation(&path, Mat2D::IDENTITY, 10.0, StrokeJoin::Bevel, cap)
                    .unwrap();

            assert_eq!(tessellation.spans.len(), 5);
            let geometry = geometry_spans(&tessellation).collect::<Vec<_>>();
            assert_eq!(geometry.len(), 2);
            assert_eq!(geometry[0].points[0], [21.0, 30.0]);
            assert_eq!(geometry[0].points[3], [20.0, 30.0]);
            assert_eq!(geometry[1].points[0], [19.0, 30.0]);
            assert_eq!(geometry[1].points[3], [20.0, 30.0]);
            assert_eq!(
                geometry[0].contour_id_with_flags,
                1 | flags | EMULATED_STROKE_CAP_CONTOUR_FLAG
            );
        }
    }

    #[test]
    fn signed_zero_implicit_close_is_not_pruned() {
        let mut path = RawPath::new();
        path.move_to(-0.0, 0.0);
        path.line_to(0.0, 0.0);
        path.close();

        let tessellation = build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            10.0,
            StrokeJoin::Bevel,
            StrokeCap::Butt,
        )
        .unwrap();

        let close = geometry_spans(&tessellation).next().unwrap();
        assert_eq!(close.points[0][0].to_bits(), 0.0f32.to_bits());
        assert_eq!(close.points[3][0].to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn stroke_budget_uses_maximum_singular_scale_under_shear() {
        let scale = max_matrix_scale(Mat2D([1.0, 0.0, 1.0, 1.0, 0.0, 0.0]));
        assert!((scale - 1.618_034).abs() < 1e-5);
    }

    #[test]
    fn stroke_spans_wrap_across_tessellation_texture_rows() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        for index in 1..600 {
            path.line_to(index as f32, (index % 2) as f32);
        }
        let tessellation = build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            2.0,
            StrokeJoin::Miter,
            StrokeCap::Butt,
        )
        .unwrap();
        assert!(tessellation_texture_height(&tessellation.spans) > 1);
        assert!(tessellation
            .spans
            .iter()
            .all(|span| span.x_range().0 < TESS_TEXTURE_WIDTH && span.x_range().1 > 0));
        assert_post_contour_padding(&tessellation);
    }

    #[test]
    fn overstroke_reuses_frame_scoped_stroke_preparation_capacity() {
        use nuxie_render_stream::{Command, RenderStream};

        let stream = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/gm/OverStroke.rive-stream"
        )))
        .unwrap();
        let mut scratch = StrokePreparationScratch::default();

        for command in &stream.frames[0].commands {
            let Command::DrawPath { path, paint } = command else {
                continue;
            };
            if paint.style != nuxie_render_api::RenderPaintStyle::Stroke {
                continue;
            }
            build_stroke_tessellation_with_layout_using_scratch(
                &path.raw_path,
                Mat2D::IDENTITY,
                paint.thickness,
                paint.join,
                paint.cap,
                &mut scratch,
            )
            .unwrap();
        }

        let stats = scratch.stats();
        assert_eq!(stats.builds, 12);
        assert_eq!(stats.contours, 240);
        assert_eq!(
            stats.inline_one_curve_contours + stats.spilled_curve_contours,
            stats.contours
        );
        assert!(stats.spilled_curve_contours >= 4);
        assert!(stats.contour_capacity_growths < stats.builds);
        assert!(stats.prepared_capacity_growths < stats.builds);
        assert!(stats.pending_capacity_growths < stats.builds);
    }
}
