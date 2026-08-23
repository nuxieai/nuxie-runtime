//! Executable bridge for pinned `renderer/src/draw.cpp`.
//!
//! Path subdivision, stroke chopping, feathering, Wang counts, and inner-fan
//! triangulation bind to `crate::draw`, whose source correspondence is already
//! recorded for this exact pinned file. This module owns the source `Draw`
//! construction and callback dispatch which had previously been represented
//! only by nullable/default function slots in `render_context_hpp`.

#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer as MechanicalRenderBuffer;
use crate::mechanical_port::source::renderer::include::rive::renderer::draw_hpp::{
    ClipReset, ClipResetAction, Draw, DrawObjectType, FillTessellation, ImageMeshDraw,
    ImageRectDraw, InteriorTessellation, PathCoverageType, PathDraw, RiveRenderPaintContract,
    StrokeTessellation,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    AABBu16, LogicalFlush, RenderContext, IAABB,
};
use crate::mechanical_port::source::renderer::src::rive_render_path_hpp::RiveRenderPath;
use nuxie_render_api::{
    BlendMode, FillRule, Mat2D, PathVerb, RawPath, StrokeCap, StrokeJoin, Vec2D,
};

pub fn resolve_path_pixel_bounds(
    path: &nuxie_render_api::RawPath,
    matrix: Mat2D,
    precomputed_pixel_bounds: Option<IAABB>,
    paint_feather: f32,
    stroke: Option<(f32, StrokeJoin, StrokeCap)>,
) -> Option<IAABB> {
    #[cfg(not(debug_assertions))]
    if let Some(pixel_bounds) = precomputed_pixel_bounds {
        return Some(pixel_bounds);
    }

    let [left, top, right, bottom] = if stroke.is_some() || paint_feather != 0.0 {
        crate::draw::prepared_feather_pixel_bounds(path, matrix, paint_feather, stroke)?
    } else {
        crate::draw::path_pixel_bounds(path, matrix)?
    };
    let pixel_bounds = IAABB {
        left,
        top,
        right,
        bottom,
    };
    #[cfg(debug_assertions)]
    if let Some(precomputed) = precomputed_pixel_bounds {
        debug_assert_eq!(pixel_bounds, precomputed);
    }
    Some(pixel_bounds)
}

pub fn select_path_coverage_type(
    paint_feather: f32,
    matrix: Mat2D,
    platform_features: &gpu::PlatformFeatures,
    interlock_mode: gpu::InterlockMode,
) -> PathCoverageType {
    if paint_feather != 0.0
        && crate::draw::feather_requires_atlas(
            paint_feather,
            matrix,
            platform_features.alwaysFeatherToAtlas || interlock_mode == gpu::InterlockMode::msaa,
        )
    {
        return PathCoverageType::featherAtlas;
    }
    match interlock_mode {
        gpu::InterlockMode::rasterOrdering | gpu::InterlockMode::atomics => {
            PathCoverageType::pixelLocalStorage
        }
        gpu::InterlockMode::clockwise => PathCoverageType::clockwise,
        gpu::InterlockMode::clockwiseAtomic => PathCoverageType::clockwiseAtomic,
        gpu::InterlockMode::msaa => PathCoverageType::msaa,
    }
}

pub enum PreparedPathGeometry {
    MidpointFan(FillTessellation),
    Stroke(StrokeTessellation),
    Interior(InteriorTessellation),
}

impl PreparedPathGeometry {
    fn fill(&self) -> Option<&FillTessellation> {
        match self {
            Self::MidpointFan(fill) => Some(fill),
            Self::Stroke(stroke) => Some(&stroke.tessellation),
            Self::Interior(_) => None,
        }
    }

    fn tess_vertex_count(&self) -> u32 {
        match self {
            Self::MidpointFan(fill) => {
                fill.instance_count * gpu::kMidpointFanPatchSegmentSpan as u32
            }
            Self::Stroke(stroke) => {
                stroke.tessellation.instance_count * gpu::kMidpointFanPatchSegmentSpan as u32
            }
            Self::Interior(interior) => {
                interior.instance_count * gpu::kOuterCurvePatchSegmentSpan as u32
            }
        }
    }

    fn relocate_to(&mut self, tess_location: u32) {
        let (spans, base_instance, contours, segment_span) = match self {
            Self::MidpointFan(fill) => (
                &mut fill.spans,
                &mut fill.base_instance,
                &mut fill.contours,
                gpu::kMidpointFanPatchSegmentSpan as u32,
            ),
            Self::Stroke(stroke) => (
                &mut stroke.tessellation.spans,
                &mut stroke.tessellation.base_instance,
                &mut stroke.tessellation.contours,
                gpu::kMidpointFanPatchSegmentSpan as u32,
            ),
            Self::Interior(interior) => (
                &mut interior.spans,
                &mut interior.base_instance,
                &mut interior.contours,
                gpu::kOuterCurvePatchSegmentSpan as u32,
            ),
        };
        debug_assert_eq!(tess_location % segment_span, 0);
        crate::relocate_tessellation_logically(
            spans,
            base_instance,
            contours,
            tess_location / segment_span,
            segment_span,
        );
    }

    fn contour_count(&self) -> usize {
        match self {
            Self::MidpointFan(fill) => fill.contours.len(),
            Self::Stroke(stroke) => stroke.tessellation.contours.len(),
            Self::Interior(interior) => interior.contours.len(),
        }
    }

    fn span_count(&self) -> usize {
        match self {
            Self::MidpointFan(fill) => fill.spans.len(),
            Self::Stroke(stroke) => stroke.tessellation.spans.len(),
            Self::Interior(interior) => interior.spans.len(),
        }
    }

    fn tessellated_segment_record_count(&self) -> usize {
        let spans = match self {
            Self::MidpointFan(fill) => fill.spans.as_slice(),
            Self::Stroke(stroke) => stroke.tessellation.spans.as_slice(),
            Self::Interior(interior) => interior.spans.as_slice(),
        };
        // Count every emitted authored segment record. Row-wrapped fragments
        // are distinct source records and must not be collapsed merely because
        // adjacent line/curve endpoints and flags compare identically.
        spans
            .iter()
            .filter(|span| span.contour_id_with_flags & crate::gpu::CONTOUR_ID_MASK != 0)
            .count()
    }

    fn parts_mut(
        &mut self,
    ) -> (
        &mut Vec<crate::gpu::TessVertexSpan>,
        &mut u32,
        &mut Vec<crate::gpu::ContourData>,
        u32,
    ) {
        match self {
            Self::MidpointFan(fill) => (
                &mut fill.spans,
                &mut fill.base_instance,
                &mut fill.contours,
                gpu::kMidpointFanPatchSegmentSpan as u32,
            ),
            Self::Stroke(stroke) => (
                &mut stroke.tessellation.spans,
                &mut stroke.tessellation.base_instance,
                &mut stroke.tessellation.contours,
                gpu::kMidpointFanPatchSegmentSpan as u32,
            ),
            Self::Interior(interior) => (
                &mut interior.spans,
                &mut interior.base_instance,
                &mut interior.contours,
                gpu::kOuterCurvePatchSegmentSpan as u32,
            ),
        }
    }

    /// Source `pushTessellationData()` maps the two sides to separately
    /// allocated clockwise-atomic prepass and main-pass ranges. Which side is
    /// borrowed is direction-dependent; a contiguous relocation cannot
    /// represent this branch.
    fn relocate_split_borrowed_coverage(
        &mut self,
        main_location: u32,
        borrowed_location: u32,
        half_vertex_count: u32,
        directions: gpu::ContourDirections,
    ) {
        debug_assert!(matches!(
            directions,
            gpu::ContourDirections::reverseThenForward | gpu::ContourDirections::forwardThenReverse
        ));
        let (spans, base_instance, contours, segment_span) = self.parts_mut();
        let old_base = base_instance
            .checked_mul(segment_span)
            .expect("tessellation source base overflow");
        let (source_primary_base, source_reflection_end, primary_target, reflection_target_end) =
            match directions {
                gpu::ContourDirections::reverseThenForward => (
                    old_base + half_vertex_count,
                    old_base + half_vertex_count,
                    main_location,
                    borrowed_location + half_vertex_count,
                ),
                gpu::ContourDirections::forwardThenReverse => (
                    old_base,
                    old_base + half_vertex_count * 2,
                    borrowed_location,
                    main_location + half_vertex_count,
                ),
                _ => unreachable!(),
            };
        let texture_width = crate::gpu::TESS_TEXTURE_WIDTH as u32;
        let mut relocated = Vec::with_capacity(spans.len());
        for mut span in core::mem::take(spans) {
            let (x0, x1) = span.x_range();
            let reverse_only = span.reflection_y.is_nan() && x1 < x0;
            let (logical_x0, logical_x1) = if reverse_only {
                (x1, x0)
            } else {
                debug_assert!(x1 >= x0);
                (x0, x1)
            };
            let width =
                u32::try_from(logical_x1 - logical_x0).expect("nonnegative tessellation span");
            let source_primary = (span.y as u32)
                .checked_mul(texture_width)
                .and_then(|row| row.checked_add_signed(logical_x0))
                .expect("tessellation primary location overflow");
            let primary = if span.contour_id_with_flags & crate::gpu::CONTOUR_ID_MASK == 0 {
                source_primary
                    .checked_add(
                        main_location
                            .checked_sub(old_base)
                            .expect("padding relocation must move forward"),
                    )
                    .expect("padding relocation overflow")
            } else {
                primary_target
                    .checked_add(source_primary - source_primary_base)
                    .expect("split primary relocation overflow")
            };
            let mut y = primary / texture_width;
            let mut relocated_x0 = (primary % texture_width) as i32;
            let mut relocated_x1 = relocated_x0 + width as i32;
            if span.reflection_y.is_finite() {
                let source_reflection_x0 = span.reflection_x0_x1 as i16 as i32;
                let source_reflection = (span.reflection_y as u32)
                    .wrapping_mul(texture_width)
                    .wrapping_add_signed(source_reflection_x0);
                let reflection = reflection_target_end
                    .checked_sub(source_reflection_end - source_reflection)
                    .expect("split reflection relocation underflow");
                let mut reflection_y = reflection.wrapping_sub(1) / texture_width;
                let mut reflection_x0 = (reflection.wrapping_sub(1) % texture_width + 1) as i32;
                let mut reflection_x1 = reflection_x0 - width as i32;
                loop {
                    span.y = y as f32;
                    span.set_ranges(
                        relocated_x0,
                        relocated_x1,
                        reflection_x0,
                        reflection_x1,
                        reflection_y as f32,
                    );
                    relocated.push(span);
                    if relocated_x1 <= crate::gpu::TESS_TEXTURE_WIDTH && reflection_x1 >= 0 {
                        break;
                    }
                    y += 1;
                    relocated_x0 -= crate::gpu::TESS_TEXTURE_WIDTH;
                    relocated_x1 -= crate::gpu::TESS_TEXTURE_WIDTH;
                    reflection_y = reflection_y.wrapping_sub(1);
                    reflection_x0 += crate::gpu::TESS_TEXTURE_WIDTH;
                    reflection_x1 += crate::gpu::TESS_TEXTURE_WIDTH;
                }
            } else if reverse_only {
                let reverse_last = primary
                    .checked_add(width)
                    .and_then(|location| location.checked_sub(1))
                    .expect("reverse tessellation span end overflow");
                let mut reverse_y = reverse_last / texture_width;
                let mut reverse_x0 = (reverse_last % texture_width + 1) as i32;
                let mut reverse_x1 = reverse_x0 - width as i32;
                loop {
                    span.y = reverse_y as f32;
                    span.set_ranges(reverse_x0, reverse_x1, -1, -1, f32::NAN);
                    relocated.push(span);
                    if reverse_x1 >= 0 {
                        break;
                    }
                    reverse_y = reverse_y
                        .checked_sub(1)
                        .expect("reverse tessellation row underflow");
                    reverse_x0 = reverse_x0
                        .checked_add(texture_width as i32)
                        .expect("reverse tessellation x overflow");
                    reverse_x1 = reverse_x1
                        .checked_add(texture_width as i32)
                        .expect("reverse tessellation x overflow");
                }
            } else {
                loop {
                    span.y = y as f32;
                    span.set_ranges(relocated_x0, relocated_x1, -1, -1, f32::NAN);
                    relocated.push(span);
                    if relocated_x1 <= crate::gpu::TESS_TEXTURE_WIDTH {
                        break;
                    }
                    y += 1;
                    relocated_x0 -= crate::gpu::TESS_TEXTURE_WIDTH;
                    relocated_x1 -= crate::gpu::TESS_TEXTURE_WIDTH;
                }
            }
        }
        for contour in contours {
            contour.vertex_index0 = primary_target
                .checked_add(contour.vertex_index0 - source_primary_base)
                .expect("split contour relocation overflow");
        }
        *base_instance = primary_target / segment_span;
        *spans = relocated;
    }
}

/// Complete allocation owner for the source `PathDraw` base plus the exact
/// already-translated draw.cpp geometry. `draw` is first so all source virtual
/// callbacks can recover the most-derived allocation from a `Draw*`.
#[repr(C)]
pub struct PathDrawAllocation {
    pub draw: PathDraw,
    path_ref: rcp<RiveRenderPath>,
    #[cfg(debug_assertions)]
    raw_path_mutation_id: u64,
    geometry: PreparedPathGeometry,
    image_texture: rcp<gpu::Texture>,
    gradient: rcp<crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::Gradient>,
    coverage_type: PathCoverageType,
    contour_directions: gpu::ContourDirections,
    path_fill_rule: FillRule,
    triangulator_fill_rule: FillRule,
    triangulator_reverse_triangles: bool,
    triangulator_negate_winding: bool,
    atlas_scale_factor: f32,
    path_id: u32,
    prepass_tess_location: u32,
    tess_location: u32,
    geometry_written: bool,
}

fn push_reverse_span_fragments(
    output: &mut Vec<crate::gpu::TessVertexSpan>,
    mut span: crate::gpu::TessVertexSpan,
    logical_x0: i32,
    logical_x1: i32,
    source_base: i32,
    reverse_end: i32,
) {
    let vertex_count = logical_x1 - logical_x0;
    let reverse_location = reverse_end - (logical_x0 - source_base);
    let mut y = ((reverse_location - 1) / crate::gpu::TESS_TEXTURE_WIDTH) as u32;
    let mut x0 = (reverse_location - 1) % crate::gpu::TESS_TEXTURE_WIDTH + 1;
    let mut x1 = x0 - vertex_count;
    loop {
        span.y = y as f32;
        span.set_ranges(x0, x1, -1, -1, f32::NAN);
        output.push(span);
        if x1 >= 0 {
            break;
        }
        y = y.wrapping_sub(1);
        x0 += crate::gpu::TESS_TEXTURE_WIDTH;
        x1 += crate::gpu::TESS_TEXTURE_WIDTH;
    }
}

fn push_forward_span_fragments(
    output: &mut Vec<crate::gpu::TessVertexSpan>,
    points: [[f32; 2]; 4],
    join_tangent: [f32; 2],
    logical_x0: i32,
    logical_x1: i32,
    parametric_segments: u32,
    contour_id: u32,
) {
    let mut y = logical_x0 / crate::gpu::TESS_TEXTURE_WIDTH;
    let mut x0 = logical_x0 % crate::gpu::TESS_TEXTURE_WIDTH;
    let mut x1 = x0 + logical_x1 - logical_x0;
    loop {
        output.push(crate::gpu::TessVertexSpan::without_reflection(
            points,
            join_tangent,
            y as f32,
            x0,
            x1,
            parametric_segments,
            1,
            1,
            contour_id,
        ));
        if x1 <= crate::gpu::TESS_TEXTURE_WIDTH {
            break;
        }
        y += 1;
        x0 -= crate::gpu::TESS_TEXTURE_WIDTH;
        x1 -= crate::gpu::TESS_TEXTURE_WIDTH;
    }
}

#[derive(Clone, Copy)]
struct SourceFillCurve {
    points: [Vec2D; 4],
    is_line: bool,
}

fn source_points_equal(a: Vec2D, b: Vec2D) -> bool {
    a.x.to_bits() == b.x.to_bits() && a.y.to_bits() == b.y.to_bits()
}

fn source_stroke_midpoint(closed: bool) -> [f32; 2] {
    [if closed { 1.0 } else { 0.0 }, 0.0]
}

fn source_line_cubic(start: Vec2D, end: Vec2D) -> [Vec2D; 4] {
    let one_third = |a: Vec2D, b: Vec2D| {
        Vec2D::new(
            (b.x - a.x).mul_add(1.0 / 3.0, a.x),
            (b.y - a.y).mul_add(1.0 / 3.0, a.y),
        )
    };
    [start, one_third(start, end), one_third(end, start), end]
}

fn transformed_cubic_segment_count(points: [Vec2D; 4], matrix: Mat2D) -> u32 {
    let [xx, yx, xy, yy, _, _] = matrix.0;
    let second_difference = |a: Vec2D, b: Vec2D, c: Vec2D| {
        let x = -2.0 * b.x + a.x + c.x;
        let y = -2.0 * b.y + a.y + c.y;
        let mapped_x = xx * x + xy * y;
        let mapped_y = yx * x + yy * y;
        mapped_x * mapped_x + mapped_y * mapped_y
    };
    let max_length_squared = second_difference(points[0], points[1], points[2])
        .max(second_difference(points[1], points[2], points[3]));
    let length_term_squared = (9.0 / 16.0) * (crate::gpu::PARAMETRIC_PRECISION as f32).powi(2);
    (max_length_squared * length_term_squared)
        .sqrt()
        .sqrt()
        .ceil()
        .clamp(1.0, crate::gpu::MAX_PARAMETRIC_SEGMENTS as f32) as u32
}

/// Forward midpoint-fan preparation with the source's bitwise contour-close
/// comparison. This intentionally retains zero-length lines and distinguishes
/// `-0` and NaN payloads exactly like the `uint64_t` bit-cast in draw.cpp.
fn build_source_fill_tessellation(path: &RawPath, matrix: Mat2D) -> Option<FillTessellation> {
    let mut contours = Vec::<Vec<SourceFillCurve>>::new();
    let mut current_curves = Vec::new();
    let mut first = None;
    let mut current = None;
    let mut point_index = 0usize;
    let finish = |contours: &mut Vec<Vec<SourceFillCurve>>,
                  curves: &mut Vec<SourceFillCurve>,
                  first: &mut Option<Vec2D>,
                  current: &mut Option<Vec2D>| {
        if let (Some(start), Some(end)) = (*first, *current) {
            if !source_points_equal(start, end) {
                curves.push(SourceFillCurve {
                    points: source_line_cubic(end, start),
                    is_line: true,
                });
            }
            contours.push(core::mem::take(curves));
        }
        *first = None;
        *current = None;
    };
    for verb in path.verbs() {
        match verb {
            PathVerb::Move => {
                finish(&mut contours, &mut current_curves, &mut first, &mut current);
                let point = path.points()[point_index];
                point_index += 1;
                first = Some(point);
                current = Some(point);
            }
            PathVerb::Line => {
                let end = path.points()[point_index];
                point_index += 1;
                let start = current?;
                current_curves.push(SourceFillCurve {
                    points: source_line_cubic(start, end),
                    is_line: true,
                });
                current = Some(end);
            }
            PathVerb::Quad => {
                unreachable!("pinned draw.cpp requires RawPath quads converted to cubics")
            }
            PathVerb::Cubic => {
                let start = current?;
                let points = [
                    start,
                    path.points()[point_index],
                    path.points()[point_index + 1],
                    path.points()[point_index + 2],
                ];
                point_index += 3;
                current_curves.push(SourceFillCurve {
                    points,
                    is_line: false,
                });
                current = Some(points[3]);
            }
            PathVerb::Close => {}
        }
    }
    finish(&mut contours, &mut current_curves, &mut first, &mut current);
    if contours.is_empty() {
        return None;
    }
    let patch_span = crate::gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
    let mut spans = vec![crate::gpu::TessVertexSpan::without_reflection(
        [[0.0; 2]; 4],
        [0.0; 2],
        0.0,
        0,
        patch_span as i32,
        0,
        0,
        1,
        0,
    )];
    let mut contour_data = Vec::with_capacity(contours.len());
    let mut location = patch_span;
    let path_start = location;
    for (index, curves) in contours.iter().enumerate() {
        let contour_start = location;
        let mut endpoint_sum = Vec2D::new(0.0, 0.0);
        let counts = curves
            .iter()
            .map(|curve| {
                endpoint_sum.x += curve.points[3].x;
                endpoint_sum.y += curve.points[3].y;
                if curve.is_line {
                    1
                } else {
                    transformed_cubic_segment_count(curve.points, matrix)
                }
            })
            .collect::<Vec<_>>();
        let reciprocal = 1.0 / curves.len() as f32;
        contour_data.push(crate::gpu::ContourData::new(
            [endpoint_sum.x * reciprocal, endpoint_sum.y * reciprocal],
            0,
            contour_start,
        ));
        let raw_vertex_count = counts.iter().sum::<u32>() + curves.len() as u32;
        let padding = (patch_span - raw_vertex_count % patch_span) % patch_span;
        for (curve_index, (curve, segments)) in curves.iter().zip(counts).enumerate() {
            let x0 = location;
            location += segments + 1 + u32::from(curve_index == 0) * padding;
            push_forward_span_fragments(
                &mut spans,
                curve.points.map(|point| [point.x, point.y]),
                if curve.is_line {
                    [0.0, 1.0]
                } else {
                    [0.0, 0.0]
                },
                x0 as i32,
                location as i32,
                segments,
                (index as u32 + 1) & crate::gpu::CONTOUR_ID_MASK,
            );
        }
    }
    if location == path_start {
        return None;
    }
    let geometry_spans = spans.split_off(1);
    let outer_aligned_location =
        location.next_multiple_of(crate::gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32);
    if outer_aligned_location != location {
        push_forward_span_fragments(
            &mut spans,
            [[0.0; 2]; 4],
            [0.0, 0.0],
            location as i32,
            outer_aligned_location as i32,
            0,
            0,
        );
    }
    push_forward_span_fragments(
        &mut spans,
        [[0.0; 2]; 4],
        [0.0, 0.0],
        outer_aligned_location as i32,
        outer_aligned_location as i32 + 1,
        0,
        0,
    );
    spans.extend(geometry_spans);
    Some(FillTessellation {
        spans,
        path: crate::gpu::PathData::new(
            matrix,
            0.0,
            0.0,
            0,
            crate::gpu::AtlasTransform {
                scale_factor: 0.0,
                translate_x: 0.0,
                translate_y: 0.0,
            },
            crate::gpu::CoverageBufferRange {
                offset: 0,
                pitch: 0,
                offset_x: 0.0,
                offset_y: 0.0,
            },
        ),
        contours: contour_data,
        base_instance: 1,
        instance_count: (location - path_start) / patch_span,
    })
}

fn make_fill_single_sided_reverse(fill: &mut FillTessellation, negate_coverage: bool) {
    let base = fill.base_instance * gpu::kMidpointFanPatchSegmentSpan as u32;
    let vertex_count = fill.instance_count * gpu::kMidpointFanPatchSegmentSpan as u32;
    let end = base + vertex_count;
    let mut reversed_geometry = Vec::with_capacity(fill.spans.len());
    let mut previous = None;
    for mut span in fill
        .spans
        .iter()
        .copied()
        .filter(|span| span.contour_id_with_flags & crate::gpu::CONTOUR_ID_MASK != 0)
    {
        let (x0, x1) = span.x_range();
        let logical_x0 = span.y as i32 * crate::gpu::TESS_TEXTURE_WIDTH + x0;
        if previous == Some(logical_x0) {
            continue;
        }
        previous = Some(logical_x0);
        if negate_coverage {
            span.contour_id_with_flags |= crate::gpu::NEGATE_PATH_FILL_COVERAGE_FLAG;
        }
        push_reverse_span_fragments(
            &mut reversed_geometry,
            span,
            logical_x0,
            logical_x0 + x1 - x0,
            base as i32,
            end as i32,
        );
    }
    for contour in &mut fill.contours {
        contour.vertex_index0 = base + end - contour.vertex_index0 - 1;
    }
    let mut reversed = Vec::with_capacity(reversed_geometry.len() + 3);
    reversed.push(crate::gpu::TessVertexSpan::without_reflection(
        [[0.0; 2]; 4],
        [0.0; 2],
        0.0,
        0,
        base as i32,
        0,
        0,
        1,
        0,
    ));
    let outer_aligned_end = end.next_multiple_of(gpu::kOuterCurvePatchSegmentSpan as u32);
    if outer_aligned_end != end {
        push_forward_span_fragments(
            &mut reversed,
            [[0.0; 2]; 4],
            [0.0; 2],
            end as i32,
            outer_aligned_end as i32,
            0,
            0,
        );
    }
    push_forward_span_fragments(
        &mut reversed,
        [[0.0; 2]; 4],
        [0.0; 2],
        outer_aligned_end as i32,
        outer_aligned_end as i32 + 1,
        0,
        0,
    );
    reversed.extend(reversed_geometry);
    fill.spans = reversed;
}

#[repr(C)]
pub struct ImageRectDrawAllocation {
    pub draw: ImageRectDraw,
    image_texture: rcp<gpu::Texture>,
}

#[repr(C)]
pub struct ImageMeshDrawAllocation {
    pub draw: ImageMeshDraw,
    image_texture: rcp<gpu::Texture>,
    vertex_buffer: rcp<MechanicalRenderBuffer>,
    uv_buffer: rcp<MechanicalRenderBuffer>,
    index_buffer: rcp<MechanicalRenderBuffer>,
}

/// Frame-ordered complete allocations corresponding to source block-allocated
/// `Draw` objects. The enum owns the most-derived allocation while the
/// mechanical RenderContext receives only its source `DrawUniquePtr` view.
pub enum MechanicalDrawOwner {
    Path(Box<PathDrawAllocation>),
    ImageRect(Box<ImageRectDrawAllocation>),
    ImageMesh(Box<ImageMeshDrawAllocation>),
    ClipReset(Box<ClipReset>),
}

impl MechanicalDrawOwner {
    pub fn draw_ptr(&mut self) -> *mut Draw {
        match self {
            Self::Path(owner) => owner.draw_ptr(),
            Self::ImageRect(owner) => owner.draw_ptr(),
            Self::ImageMesh(owner) => owner.draw_ptr(),
            Self::ClipReset(owner) => core::ptr::addr_of_mut!(owner.base),
        }
    }
}

impl PathDrawAllocation {
    pub fn draw_ptr(&mut self) -> *mut Draw {
        core::ptr::addr_of_mut!(self.draw.base)
    }

    pub fn coverageType(&self) -> PathCoverageType {
        self.coverage_type
    }
    pub fn contourDirections(&self) -> gpu::ContourDirections {
        self.contour_directions
    }
    pub fn isFeatheredFill(&self) -> bool {
        self.draw.featherRadius() != 0.0 && !self.draw.isStroke()
    }
    pub fn isStrokeOrFeather(&self) -> bool {
        self.draw.strokeRadius().to_bits() | self.draw.featherRadius().to_bits() != 0
    }
    pub fn isOutermostClipUpdate(&self) -> bool {
        (self.draw.base.draw_contents
            & (gpu::DrawContents::clipUpdate | gpu::DrawContents::activeClip))
            == gpu::DrawContents::clipUpdate
    }
    pub fn needsBorrowedCoveragePrepass(&self) -> bool {
        debug_assert_eq!(self.coverage_type, PathCoverageType::clockwiseAtomic);
        !self.draw.isStroke() && !self.isOutermostClipUpdate()
    }
    pub fn triangulatorFillRule(&self) -> FillRule {
        self.triangulator_fill_rule
    }
    pub fn triangulatorReverseTriangles(&self) -> bool {
        self.triangulator_reverse_triangles
    }
    pub fn triangulatorNegateWinding(&self) -> bool {
        self.triangulator_negate_winding
    }
    pub fn triangulator(&self) -> Option<&InteriorTessellation> {
        match &self.geometry {
            PreparedPathGeometry::Interior(interior) => Some(interior),
            PreparedPathGeometry::MidpointFan(_) | PreparedPathGeometry::Stroke(_) => None,
        }
    }
}

// These queries are public on source `PathDraw`, including after a `Draw*` has
// been concretely downcast to `PathDraw*`. Every PathDraw constructed by this
// module is the offset-zero first field of PathDrawAllocation, so recover the
// complete source-shaped owner for state that is not part of the GPU-facing
// prefix.
impl PathDraw {
    fn allocation(&self) -> &PathDrawAllocation {
        unsafe { &*(self as *const PathDraw).cast::<PathDrawAllocation>() }
    }

    pub fn coverageType(&self) -> PathCoverageType {
        self.allocation().coverageType()
    }
    pub fn contourDirections(&self) -> gpu::ContourDirections {
        self.allocation().contourDirections()
    }
    pub fn isFeatheredFill(&self) -> bool {
        self.allocation().isFeatheredFill()
    }
    pub fn isStrokeOrFeather(&self) -> bool {
        self.allocation().isStrokeOrFeather()
    }
    pub fn isOutermostClipUpdate(&self) -> bool {
        self.allocation().isOutermostClipUpdate()
    }
    pub fn needsBorrowedCoveragePrepass(&self) -> bool {
        self.allocation().needsBorrowedCoveragePrepass()
    }
    pub fn triangulator(&self) -> Option<&InteriorTessellation> {
        self.allocation().triangulator()
    }
    pub fn triangulatorFillRule(&self) -> FillRule {
        self.allocation().triangulatorFillRule()
    }
    pub fn triangulatorReverseTriangles(&self) -> bool {
        self.allocation().triangulatorReverseTriangles()
    }
    pub fn triangulatorNegateWinding(&self) -> bool {
        self.allocation().triangulatorNegateWinding()
    }
}

impl ImageRectDrawAllocation {
    pub fn draw_ptr(&mut self) -> *mut Draw {
        core::ptr::addr_of_mut!(self.draw.base)
    }
}

impl ImageMeshDrawAllocation {
    pub fn draw_ptr(&mut self) -> *mut Draw {
        core::ptr::addr_of_mut!(self.draw.base)
    }
}

unsafe fn release_plain_draw(_: *mut Draw) {}

unsafe fn release_path_draw(draw: *mut Draw) {
    let owner = unsafe { &mut *path_allocation(draw) };
    owner.image_texture.operator_assign_null();
    unsafe {
        (&*owner.path_ref.get()).unlockRawPathMutations();
    }
    owner.path_ref.operator_assign_null();
    owner.gradient.operator_assign_null();
}

unsafe fn release_image_rect(draw: *mut Draw) {
    let owner = unsafe { &mut *draw.cast::<ImageRectDrawAllocation>() };
    owner.image_texture.operator_assign_null();
}

unsafe fn release_image_mesh(draw: *mut Draw) {
    let owner = unsafe { &mut *draw.cast::<ImageMeshDrawAllocation>() };
    owner.image_texture.operator_assign_null();
    owner.vertex_buffer.operator_assign_null();
    owner.uv_buffer.operator_assign_null();
    owner.index_buffer.operator_assign_null();
}

unsafe fn count_one_subpass(draw: *mut Draw, _: &gpu::PlatformFeatures) {
    unsafe {
        (*draw).prepass_count = 0;
        (*draw).subpass_count = 1;
    }
}

unsafe fn allocate_plain_draw(_: *mut Draw, _: *mut LogicalFlush) -> bool {
    true
}

unsafe fn allocate_path_resources(draw: *mut Draw, flush: *mut LogicalFlush) -> bool {
    let owner = unsafe { &mut *path_allocation(draw) };
    let flush_ref = unsafe { &mut *flush };
    #[cfg(debug_assertions)]
    debug_assert_eq!(owner.raw_path_mutation_id, unsafe {
        (&*owner.path_ref.get()).getRawPathMutationID()
    });
    debug_assert!(!unsafe { (&*owner.path_ref.get()).getRawPath() }
        .verbs()
        .is_empty());
    if !owner.gradient.get().is_null()
        && !unsafe {
            flush_ref.allocateGradientExecutable(
                owner.gradient.get(),
                core::ptr::addr_of_mut!(owner.draw.base.simple_paint_value.colorRampLocation),
            )
        }
    {
        return false;
    }

    let is_outermost_clip_update =
        (owner.draw.base.draw_contents.0 & gpu::DrawContents::clipUpdate.0) != 0
            && (owner.draw.base.draw_contents.0 & gpu::DrawContents::activeClip.0) == 0;
    if owner.coverage_type != PathCoverageType::featherAtlas
        && (owner.coverage_type != PathCoverageType::clockwiseAtomic || is_outermost_clip_update)
    {
        return true;
    }

    const PADDING: i32 = 2;
    let frame = flush_ref.frameDescriptor();
    let visible = IAABB {
        left: owner.draw.base.pixel_bounds.left.max(0),
        top: owner.draw.base.pixel_bounds.top.max(0),
        right: owner
            .draw
            .base
            .pixel_bounds
            .right
            .min(frame.renderTargetWidth as i32),
        bottom: owner
            .draw
            .base
            .pixel_bounds
            .bottom
            .min(frame.renderTargetHeight as i32),
    };
    let width = (visible.right - visible.left).max(0) as u32;
    let height = (visible.bottom - visible.top).max(0) as u32;
    if owner.coverage_type == PathCoverageType::featherAtlas {
        let atlas_width = (width as f32 * owner.atlas_scale_factor).ceil() as u16;
        let atlas_height = (height as f32 * owner.atlas_scale_factor).ceil() as u16;
        let mut x = 0;
        let mut y = 0;
        let mut scissor = AABBu16::default();
        if !unsafe {
            flush_ref.allocateFeatherAtlasDrawExecutable(
                &mut owner.draw,
                atlas_width,
                atlas_height,
                PADDING as u16,
                &mut x,
                &mut y,
                &mut scissor,
            )
        } {
            return false;
        }
        owner.draw.feather_atlas_transform.scaleFactor = owner.atlas_scale_factor;
        owner.draw.feather_atlas_transform.translateX =
            x as f32 - visible.left as f32 * owner.atlas_scale_factor;
        owner.draw.feather_atlas_transform.translateY =
            y as f32 - visible.top as f32 * owner.atlas_scale_factor;
        owner.draw.feather_atlas_scissor = scissor;
        owner.draw.feather_atlas_scissor_enabled = visible != owner.draw.base.pixel_bounds;
    } else {
        let coverage_width = (width + (PADDING * 2) as u32).next_multiple_of(32);
        let coverage_height = (height + (PADDING * 2) as u32).next_multiple_of(32);
        let length = coverage_width as usize * coverage_height as usize;
        let offset = flush_ref.allocateCoverageBufferRangeExecutable(length);
        if offset == usize::MAX {
            return false;
        }
        owner.draw.coverage_buffer_range.offset = offset as u32;
        owner.draw.coverage_buffer_range.pitch = coverage_width;
        owner.draw.coverage_buffer_range.offsetX = -visible.left as f32 + PADDING as f32;
        owner.draw.coverage_buffer_range.offsetY = -visible.top as f32 + PADDING as f32;
    }
    true
}

unsafe fn path_allocation(draw: *mut Draw) -> *mut PathDrawAllocation {
    draw.cast::<PathDrawAllocation>()
}

unsafe fn count_path_subpasses(draw: *mut Draw, features: &gpu::PlatformFeatures) {
    let owner = unsafe { &mut *path_allocation(draw) };
    let interior = matches!(&owner.geometry, PreparedPathGeometry::Interior(_));
    let is_outermost_clip_update = (owner.draw.base.draw_contents
        & (gpu::DrawContents::clipUpdate | gpu::DrawContents::activeClip))
        == gpu::DrawContents::clipUpdate;
    owner.draw.base.prepass_count = 0;
    owner.draw.base.subpass_count = match owner.coverage_type {
        PathCoverageType::featherAtlas => 1,
        PathCoverageType::pixelLocalStorage => {
            if interior {
                2
            } else {
                1
            }
        }
        PathCoverageType::clockwise => {
            if interior {
                3
            } else {
                1
            }
        }
        PathCoverageType::clockwiseAtomic => {
            let subpass_count = if interior { 2 } else { 1 };
            if !owner.draw.isStroke() && !is_outermost_clip_update {
                owner.draw.base.prepass_count = subpass_count;
            }
            subpass_count
        }
        PathCoverageType::msaa => {
            if owner.draw.isStroke()
                || ((owner.draw.base.draw_contents
                    & (gpu::DrawContents::clipUpdate | gpu::DrawContents::activeClip))
                    == (gpu::DrawContents::clipUpdate | gpu::DrawContents::activeClip))
            {
                1
            } else if (owner.draw.base.draw_contents.0 & gpu::DrawContents::evenOddFill.0) != 0 {
                2
            } else if features.supportsPipelineDynamicState {
                1
            } else {
                3
            }
        }
    };
    if owner.coverage_type == PathCoverageType::msaa
        && owner.draw.base.isOpaque()
        && (owner.draw.base.draw_contents.0
            & (gpu::DrawContents::activeClip | gpu::DrawContents::clipUpdate).0)
            == 0
    {
        owner.draw.base.prepass_count = owner.draw.base.subpass_count;
        owner.draw.base.subpass_count = 0;
    }
}

fn convert_span(span: crate::gpu::TessVertexSpan) -> gpu::TessVertexSpan {
    gpu::TessVertexSpan {
        pts: span
            .points
            .map(|point| nuxie_render_api::Vec2D::new(point[0], point[1])),
        joinTangent: nuxie_render_api::Vec2D::new(span.join_tangent[0], span.join_tangent[1]),
        y: span.y,
        reflectionY: span.reflection_y,
        x0x1: span.x0_x1,
        reflectionX0X1: span.reflection_x0_x1,
        segmentCounts: span.segment_counts,
        contourIDWithFlags: span.contour_id_with_flags,
    }
}

unsafe fn write_path_geometry(owner: &mut PathDrawAllocation, flush: *mut LogicalFlush) {
    if owner.geometry_written {
        return;
    }
    let flush = unsafe { &mut *flush };
    let contour_id_offset = flush.m_current_contour_id;
    let contours = match &owner.geometry {
        PreparedPathGeometry::MidpointFan(fill) => fill.contours.as_slice(),
        PreparedPathGeometry::Stroke(stroke) => stroke.tessellation.contours.as_slice(),
        PreparedPathGeometry::Interior(interior) => interior.contours.as_slice(),
    };
    let contour_count = contours.len() as u32;
    flush.m_current_contour_id = flush
        .m_current_contour_id
        .checked_add(contour_count)
        .expect("contour ID overflow");
    let context = unsafe { flush.m_ctx.as_mut() };
    for contour in contours {
        unsafe {
            context.m_contour_data.emplace_back(gpu::ContourData::new(
                nuxie_render_api::Vec2D::new(contour.midpoint[0], contour.midpoint[1]),
                owner.path_id,
                contour.vertex_index0,
            ))
        };
    }

    let spans = match &mut owner.geometry {
        PreparedPathGeometry::MidpointFan(fill) => &mut fill.spans,
        PreparedPathGeometry::Stroke(stroke) => &mut stroke.tessellation.spans,
        PreparedPathGeometry::Interior(interior) => &mut interior.spans,
    };
    // Source LogicalFlush owns the three global tessellation-padding spans.
    // Geometry owners carry contour-ID-0 spans for the generic CPU path, but
    // publishing them here once per draw would exceed the source-sized mapping.
    for span in spans
        .iter_mut()
        .filter(|span| span.contour_id_with_flags & crate::gpu::CONTOUR_ID_MASK != 0)
    {
        let local_contour_id = span.contour_id_with_flags & crate::gpu::CONTOUR_ID_MASK;
        if local_contour_id != 0 {
            let global_contour_id = local_contour_id
                .checked_add(contour_id_offset)
                .expect("contour ID relocation overflow");
            debug_assert!(global_contour_id as usize <= gpu::kMaxContourID);
            span.contour_id_with_flags =
                (span.contour_id_with_flags & !crate::gpu::CONTOUR_ID_MASK) | global_contour_id;
        }
        unsafe { context.m_tess_span_data.emplace_back(convert_span(*span)) };
    }
    owner.geometry_written = true;
}

unsafe fn push_interior_triangles(
    draw: *const PathDraw,
    path_id: u32,
    winding: gpu::WindingFaces,
    writer: *mut gpu::WriteOnlyMappedMemory<gpu::TriangleVertex>,
) -> usize {
    let owner = unsafe { &*draw.cast::<PathDrawAllocation>() };
    let PreparedPathGeometry::Interior(interior) = &owner.geometry else {
        return 0;
    };
    let mut written = 0;
    for triangle in interior.triangles.chunks_exact(3) {
        let weight = (triangle[0].weight_path_id >> 16) as i16;
        let included = (weight < 0 && (winding.0 & gpu::WindingFaces::negative.0) != 0)
            || (weight >= 0 && (winding.0 & gpu::WindingFaces::positive.0) != 0);
        if !included {
            continue;
        }
        for vertex in triangle {
            unsafe {
                (&mut *writer).emplace_back(gpu::TriangleVertex {
                    m_point: nuxie_render_api::Vec2D::new(vertex.point[0], vertex.point[1]),
                    m_weight_pathID: (vertex.weight_path_id & !0xffff) | path_id as i32,
                })
            };
            written += 1;
        }
    }
    written
}

unsafe fn push_path(
    draw: *mut Draw,
    flush: *mut LogicalFlush,
    subpass: i32,
) -> *mut gpu::DrawBatch {
    let owner = unsafe { &mut *path_allocation(draw) };
    let flush_ref = unsafe { &mut *flush };
    let mut tess_vertex_count = owner.geometry.tess_vertex_count();
    if tess_vertex_count == 0 {
        return core::ptr::null_mut();
    }
    if owner.path_id == 0 {
        owner.path_id = unsafe { flush_ref.pushPathExecutable(&owner.draw) };
    }
    let interior = matches!(&owner.geometry, PreparedPathGeometry::Interior(_));
    match owner.coverage_type {
        PathCoverageType::pixelLocalStorage | PathCoverageType::clockwise => {
            let main_subpass = if owner.coverage_type == PathCoverageType::clockwise && interior {
                1
            } else {
                0
            };
            if subpass == main_subpass {
                owner.tess_location = if interior {
                    flush_ref.allocateOuterCubicTessVerticesExecutable(tess_vertex_count)
                } else {
                    flush_ref.allocateMidpointFanTessVerticesExecutable(tess_vertex_count)
                };
                owner.geometry.relocate_to(owner.tess_location);
                unsafe { write_path_geometry(owner, flush) };
                if interior {
                    unsafe {
                        flush_ref.pushOuterCubicsDrawExecutable(
                            &owner.draw,
                            gpu::DrawType::outerCurvePatches,
                            tess_vertex_count,
                            owner.tess_location,
                            gpu::ShaderMiscFlags::none,
                        )
                    }
                } else {
                    let draw_type = if owner.draw.featherRadius() != 0.0 && !owner.draw.isStroke() {
                        gpu::DrawType::midpointFanCenterAAPatches
                    } else {
                        gpu::DrawType::midpointFanPatches
                    };
                    unsafe {
                        flush_ref.pushMidpointFanDrawExecutable(
                            &owner.draw,
                            draw_type,
                            tess_vertex_count,
                            owner.tess_location,
                            gpu::ShaderMiscFlags::none,
                        )
                    }
                }
            } else {
                let winding = if owner.coverage_type == PathCoverageType::clockwise {
                    if subpass == 0 {
                        gpu::WindingFaces::negative
                    } else {
                        gpu::WindingFaces::positive
                    }
                } else {
                    gpu::WindingFaces::all
                };
                unsafe {
                    flush_ref.pushInteriorTriangulationDrawExecutable(
                        &owner.draw,
                        owner.path_id,
                        winding,
                        if subpass == 0 {
                            gpu::ShaderMiscFlags::borrowedCoveragePass
                        } else {
                            gpu::ShaderMiscFlags::none
                        },
                        #[cfg(debug_assertions)]
                        core::ptr::null_mut(),
                    )
                }
            }
        }
        PathCoverageType::clockwiseAtomic => {
            if owner.draw.base.prepass_count != 0 {
                debug_assert_eq!(owner.draw.base.prepass_count, owner.draw.base.subpass_count);
                debug_assert_eq!(tess_vertex_count & 1, 0);
                tess_vertex_count /= 2;
            }
            match subpass {
                -1 => {
                    debug_assert!(!owner.draw.isStroke());
                    owner.prepass_tess_location = if interior {
                        flush_ref.allocateOuterCubicTessVerticesExecutable(tess_vertex_count)
                    } else {
                        flush_ref.allocateMidpointFanTessVerticesExecutable(tess_vertex_count)
                    };
                    if interior {
                        unsafe {
                            flush_ref.pushOuterCubicsDrawExecutable(
                                &owner.draw,
                                gpu::DrawType::outerCurvePatches,
                                tess_vertex_count,
                                owner.prepass_tess_location,
                                gpu::ShaderMiscFlags::borrowedCoveragePass,
                            )
                        }
                    } else {
                        unsafe {
                            flush_ref.pushMidpointFanDrawExecutable(
                                &owner.draw,
                                gpu::DrawType::midpointFanPatches,
                                tess_vertex_count,
                                owner.prepass_tess_location,
                                gpu::ShaderMiscFlags::borrowedCoveragePass,
                            )
                        }
                    }
                }
                0 => {
                    owner.tess_location = if interior {
                        flush_ref.allocateOuterCubicTessVerticesExecutable(tess_vertex_count)
                    } else {
                        flush_ref.allocateMidpointFanTessVerticesExecutable(tess_vertex_count)
                    };
                    if owner.draw.base.prepass_count != 0 {
                        debug_assert_ne!(owner.prepass_tess_location, 0);
                        owner.geometry.relocate_split_borrowed_coverage(
                            owner.tess_location,
                            owner.prepass_tess_location,
                            tess_vertex_count,
                            owner.contour_directions,
                        );
                    } else {
                        owner.geometry.relocate_to(owner.tess_location);
                    }
                    unsafe { write_path_geometry(owner, flush) };
                    if interior {
                        unsafe {
                            flush_ref.pushOuterCubicsDrawExecutable(
                                &owner.draw,
                                gpu::DrawType::outerCurvePatches,
                                tess_vertex_count,
                                owner.tess_location,
                                gpu::ShaderMiscFlags::none,
                            )
                        }
                    } else {
                        let draw_type =
                            if owner.draw.featherRadius() != 0.0 && !owner.draw.isStroke() {
                                gpu::DrawType::midpointFanCenterAAPatches
                            } else {
                                gpu::DrawType::midpointFanPatches
                            };
                        unsafe {
                            flush_ref.pushMidpointFanDrawExecutable(
                                &owner.draw,
                                draw_type,
                                tess_vertex_count,
                                owner.tess_location,
                                gpu::ShaderMiscFlags::none,
                            )
                        }
                    }
                }
                -2 | 1 => unsafe {
                    flush_ref.pushInteriorTriangulationDrawExecutable(
                        &owner.draw,
                        owner.path_id,
                        if owner.draw.base.prepass_count == 0 {
                            gpu::WindingFaces::all
                        } else if subpass < 0 {
                            gpu::WindingFaces::negative
                        } else {
                            gpu::WindingFaces::positive
                        },
                        if subpass < 0 {
                            gpu::ShaderMiscFlags::borrowedCoveragePass
                        } else {
                            gpu::ShaderMiscFlags::none
                        },
                        #[cfg(debug_assertions)]
                        core::ptr::null_mut(),
                    )
                },
                _ => core::ptr::null_mut(),
            }
        }
        PathCoverageType::msaa => {
            let pass_count = owner.draw.base.prepass_count | owner.draw.base.subpass_count;
            let pass_index = subpass + owner.draw.base.prepass_count;
            if pass_index == 0 {
                owner.tess_location =
                    flush_ref.allocateMidpointFanTessVerticesExecutable(tess_vertex_count);
                owner.geometry.relocate_to(owner.tess_location);
                unsafe { write_path_geometry(owner, flush) };
            }
            let draw_type = if pass_count == 1 {
                if owner.draw.isStroke() {
                    gpu::DrawType::msaaStrokes
                } else if (owner.draw.base.draw_contents
                    & (gpu::DrawContents::clipUpdate | gpu::DrawContents::activeClip))
                    == (gpu::DrawContents::clipUpdate | gpu::DrawContents::activeClip)
                {
                    gpu::DrawType::msaaMidpointFanPathsStencil
                } else {
                    gpu::DrawType::msaaDynamicMidpointFans
                }
            } else if pass_count == 2 {
                [
                    gpu::DrawType::msaaMidpointFanPathsStencil,
                    gpu::DrawType::msaaMidpointFanPathsCover,
                ][pass_index as usize]
            } else {
                [
                    gpu::DrawType::msaaMidpointFanBorrowedCoverage,
                    gpu::DrawType::msaaMidpointFans,
                    gpu::DrawType::msaaMidpointFanStencilReset,
                ][pass_index as usize]
            };
            unsafe {
                flush_ref.pushMidpointFanDrawExecutable(
                    &owner.draw,
                    draw_type,
                    tess_vertex_count,
                    owner.tess_location,
                    gpu::ShaderMiscFlags::none,
                )
            }
        }
        PathCoverageType::featherAtlas => unsafe {
            flush_ref.pushFeatherAtlasBlitExecutable(&mut owner.draw, owner.path_id)
        },
    }
}

unsafe fn push_feather_atlas(
    draw: *mut PathDraw,
    flush: *mut LogicalFlush,
    tess_vertex_count: *mut u32,
    tess_base_vertex: *mut u32,
) {
    let owner = unsafe { &mut *draw.cast::<PathDrawAllocation>() };
    let count = owner.geometry.tess_vertex_count();
    unsafe { *tess_vertex_count = count };
    if count == 0 {
        debug_assert_eq!(owner.path_id, 0);
        return;
    }
    let flush_ref = unsafe { &mut *flush };
    let location = if matches!(&owner.geometry, PreparedPathGeometry::Interior(_)) {
        flush_ref.allocateOuterCubicTessVerticesExecutable(count)
    } else {
        flush_ref.allocateMidpointFanTessVerticesExecutable(count)
    };
    owner.tess_location = location;
    unsafe { *tess_base_vertex = location };
    owner.geometry.relocate_to(location);
    unsafe { write_path_geometry(owner, flush) };
}

unsafe fn push_image_rect(
    draw: *mut Draw,
    flush: *mut LogicalFlush,
    subpass: i32,
) -> *mut gpu::DrawBatch {
    debug_assert_eq!(subpass, 0);
    unsafe { (&mut *flush).pushImageRectDrawExecutable(draw.cast::<ImageRectDraw>()) }
}

unsafe fn push_image_mesh(
    draw: *mut Draw,
    flush: *mut LogicalFlush,
    subpass: i32,
) -> *mut gpu::DrawBatch {
    debug_assert_eq!(subpass, 0);
    unsafe { (&mut *flush).pushImageMeshDrawExecutable(draw.cast::<ImageMeshDraw>()) }
}

unsafe fn push_clip_reset(
    draw: *mut Draw,
    flush: *mut LogicalFlush,
    subpass: i32,
) -> *mut gpu::DrawBatch {
    debug_assert_eq!(subpass, 0);
    unsafe { (&mut *flush).pushClipResetDrawExecutable(draw.cast::<ClipReset>()) }
}

fn base_draw(
    draw_type: DrawObjectType,
    pixel_bounds: IAABB,
    matrix: Mat2D,
    blend_mode: BlendMode,
    image_texture: *mut gpu::Texture,
    image_sampler: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler,
    draw_contents: gpu::DrawContents,
    clip_id: u32,
    scissor: Option<AABBu16>,
) -> Draw {
    let mut draw = Draw::new();
    draw.release_refs = release_plain_draw;
    draw.count_subpasses = count_one_subpass;
    draw.allocate_resources = allocate_plain_draw;
    draw.draw_type = draw_type;
    draw.pixel_bounds = pixel_bounds;
    draw.clipped_pixel_bounds = pixel_bounds;
    draw.matrix = matrix;
    draw.blend_mode = blend_mode;
    draw.image_texture = image_texture;
    draw.image_sampler = image_sampler;
    draw.draw_contents = draw_contents;
    if blend_mode != BlendMode::SrcOver {
        draw.draw_contents |= gpu::DrawContents::advancedBlend;
    }
    draw.setClipID(clip_id);
    draw.scissor_rect = scissor;
    draw
}

fn color_modulate_opacity(value: u32, opacity: f32) -> u32 {
    let source_alpha = (value >> 24) as f32 / 255.0;
    let alpha = (source_alpha * opacity).clamp(0.0, 1.0);
    (value & 0x00ff_ffff) | ((alpha.mul_add(255.0, 0.5).floor() as u32) << 24)
}

fn contour_directions_for_path(
    path: &RawPath,
    matrix: Mat2D,
    initial_fill_rule: FillRule,
    is_stroke: bool,
    coverage_type: PathCoverageType,
    clockwise_fill_override: bool,
) -> gpu::ContourDirections {
    if is_stroke {
        return gpu::ContourDirections::forward;
    }
    let [xx, yx, xy, yy, _, _] = matrix.0;
    let determinant = xx * yy - yx * xy;
    if initial_fill_rule == FillRule::Clockwise {
        if determinant < 0.0 {
            if matches!(
                coverage_type,
                PathCoverageType::msaa | PathCoverageType::featherAtlas
            ) {
                gpu::ContourDirections::reverse
            } else {
                gpu::ContourDirections::forwardThenReverse
            }
        } else if matches!(
            coverage_type,
            PathCoverageType::msaa | PathCoverageType::featherAtlas
        ) {
            gpu::ContourDirections::forward
        } else {
            gpu::ContourDirections::reverseThenForward
        }
    } else if coverage_type != PathCoverageType::msaa {
        if clockwise_fill_override && crate::draw::path_coarse_area(path) * determinant < 0.0 {
            if coverage_type == PathCoverageType::featherAtlas {
                gpu::ContourDirections::reverse
            } else {
                gpu::ContourDirections::forwardThenReverse
            }
        } else if coverage_type == PathCoverageType::featherAtlas {
            gpu::ContourDirections::forward
        } else {
            gpu::ContourDirections::reverseThenForward
        }
    } else if initial_fill_rule == FillRule::NonZero || clockwise_fill_override {
        if crate::draw::path_coarse_area(path) * determinant >= 0.0 {
            gpu::ContourDirections::forward
        } else {
            gpu::ContourDirections::reverse
        }
    } else {
        gpu::ContourDirections::forward
    }
}

fn apply_fill_directions(fill: &mut FillTessellation, directions: gpu::ContourDirections) {
    match directions {
        gpu::ContourDirections::forward => {}
        gpu::ContourDirections::reverse => make_fill_single_sided_reverse(fill, false),
        gpu::ContourDirections::reverseThenForward => fill.make_double_sided(),
        gpu::ContourDirections::forwardThenReverse => fill.make_double_sided_with_direction(true),
    }
}

/// Literal executable `PathDraw::Make` plus the `PathDraw` constructor. Paint
/// state is read from its source getter contract here so callers cannot bypass
/// opacity modulation, fill flags, coverage selection, or geometry admission.
pub unsafe fn make_path_draw_from_source(
    context: &RenderContext,
    matrix: Mat2D,
    path_ref: rcp<RiveRenderPath>,
    initial_fill_rule: FillRule,
    paint: &dyn RiveRenderPaintContract,
    modulated_opacity: f32,
    precomputed_pixel_bounds: Option<IAABB>,
) -> Option<Box<PathDrawAllocation>> {
    let path = unsafe { (&*path_ref.get()).getRawPath() };
    debug_assert!(!path.verbs().is_empty());
    let coverage_type = select_path_coverage_type(
        paint.getFeather(),
        matrix,
        context.platformFeatures(),
        context.frameInterlockMode(),
    );
    let pixel_bounds = resolve_path_pixel_bounds(
        path,
        matrix,
        precomputed_pixel_bounds,
        paint.getFeather(),
        paint
            .getIsStroked()
            .then_some((paint.getThickness(), paint.getJoin(), paint.getCap())),
    )?;
    if context.isOutsideCurrentFrameExecutable(&pixel_bounds) {
        debug_assert!(precomputed_pixel_bounds.is_none());
        return None;
    }

    let frame = context.frameDescriptor();
    let directions = contour_directions_for_path(
        path,
        matrix,
        initial_fill_rule,
        paint.getIsStroked(),
        coverage_type,
        frame.clockwiseFillOverride,
    );
    let do_interior = !paint.getIsStroked()
        && paint.getFeather() == 0.0
        && context.frameInterlockMode() != gpu::InterlockMode::msaa
        && crate::draw::should_use_interior_tessellation(path, matrix);
    let mut geometry = if do_interior {
        PreparedPathGeometry::Interior(crate::draw::build_interior_tessellation(
            path,
            matrix,
            initial_fill_rule,
            frame.clockwiseFillOverride,
        )?)
    } else if paint.getFeather() != 0.0 {
        let direction = match directions {
            gpu::ContourDirections::forward => crate::draw::FeatherFillDirection::Forward,
            gpu::ContourDirections::reverse => crate::draw::FeatherFillDirection::Reverse,
            gpu::ContourDirections::reverseThenForward => {
                crate::draw::FeatherFillDirection::ReverseThenForward
            }
            gpu::ContourDirections::forwardThenReverse => {
                crate::draw::FeatherFillDirection::ForwardThenReverse
            }
        };
        PreparedPathGeometry::Stroke(
            crate::draw::build_prepared_feather_tessellation_with_direction(
                path,
                matrix,
                paint.getFeather(),
                paint.getIsStroked().then_some((
                    paint.getThickness(),
                    paint.getJoin(),
                    paint.getCap(),
                )),
                direction,
            )?,
        )
    } else if paint.getIsStroked() {
        PreparedPathGeometry::Stroke(crate::draw::build_stroke_tessellation_with_layout(
            path,
            matrix,
            paint.getThickness(),
            paint.getJoin(),
            paint.getCap(),
        )?)
    } else {
        let mut fill = build_source_fill_tessellation(path, matrix)?;
        apply_fill_directions(&mut fill, directions);
        PreparedPathGeometry::MidpointFan(fill)
    };
    if paint.getIsStroked() {
        if let PreparedPathGeometry::Stroke(stroke) = &mut geometry {
            for contour in &mut stroke.tessellation.contours {
                contour.midpoint = source_stroke_midpoint(contour.midpoint[0] != 0.0);
            }
        }
    }

    let feather_radius = if paint.getFeather() != 0.0 {
        let radius = paint.getFeather() * 1.5;
        debug_assert!(!radius.is_nan() && radius > 0.0);
        radius
    } else {
        0.0
    };
    let stroke_radius = if paint.getIsStroked() {
        (paint.getThickness() * 0.5).max(f32::MIN_POSITIVE)
    } else {
        0.0
    };
    let mut draw_contents = gpu::DrawContents::none;
    if paint.getIsOpaque() {
        draw_contents |= gpu::DrawContents::opaquePaint;
    }
    if coverage_type != PathCoverageType::featherAtlas {
        if stroke_radius != 0.0 {
            draw_contents |= gpu::DrawContents::stroke;
        } else {
            if feather_radius != 0.0 {
                draw_contents |= gpu::DrawContents::featheredFill;
            }
            if initial_fill_rule == FillRule::Clockwise || frame.clockwiseFillOverride {
                draw_contents |= gpu::DrawContents::clockwiseFill;
            } else if initial_fill_rule == FillRule::NonZero {
                draw_contents |= gpu::DrawContents::nonZeroFill;
            } else {
                draw_contents |= gpu::DrawContents::evenOddFill;
            }
        }
    }

    let paint_type = paint.getType();
    let mut simple_paint_value = paint.getSimpleValue();
    if paint_type == gpu::PaintType::clipUpdate {
        draw_contents |= gpu::DrawContents::clipUpdate;
        if unsafe { simple_paint_value.outerClipID } != 0 {
            draw_contents |= gpu::DrawContents::activeClip;
        }
    }
    if modulated_opacity != 1.0 {
        match paint_type {
            gpu::PaintType::solidColor => {
                simple_paint_value.color =
                    color_modulate_opacity(unsafe { simple_paint_value.color }, modulated_opacity)
            }
            gpu::PaintType::image => {
                simple_paint_value.imageOpacity =
                    unsafe { simple_paint_value.imageOpacity } * modulated_opacity
            }
            gpu::PaintType::linearGradient
            | gpu::PaintType::radialGradient
            | gpu::PaintType::clipUpdate => {}
        }
    }
    let image_texture = paint.getImageTexture();
    let gradient = paint.getGradientWithOpacity(modulated_opacity);
    let mut owner = unsafe {
        make_path_draw(
            pixel_bounds,
            path_ref,
            matrix,
            paint.getBlendMode(),
            image_texture,
            paint.getImageSampler(),
            draw_contents,
            0,
            None,
            geometry,
            coverage_type,
            directions,
            crate::draw::feather_atlas_scale(paint.getFeather(), matrix),
            gradient,
            paint_type,
            simple_paint_value,
            stroke_radius,
            feather_radius,
            gpu::AtlasTransform::default(),
            AABBu16::default(),
            false,
            gpu::CoverageBufferRange::default(),
        )
    };
    owner.path_fill_rule = if frame.clockwiseFillOverride {
        FillRule::Clockwise
    } else {
        initial_fill_rule
    };
    owner.triangulator_fill_rule = if owner.path_fill_rule == FillRule::EvenOdd {
        FillRule::EvenOdd
    } else {
        FillRule::NonZero
    };
    let [xx, yx, xy, yy, _, _] = matrix.0;
    owner.triangulator_reverse_triangles = xx * yy - yx * xy < 0.0;
    owner.triangulator_negate_winding = owner.triangulator_reverse_triangles
        != (directions == gpu::ContourDirections::forwardThenReverse);
    Some(owner)
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn make_path_draw(
    pixel_bounds: IAABB,
    path_ref: rcp<RiveRenderPath>,
    matrix: Mat2D,
    blend_mode: BlendMode,
    image_texture: rcp<gpu::Texture>,
    image_sampler: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler,
    draw_contents: gpu::DrawContents,
    clip_id: u32,
    scissor: Option<AABBu16>,
    geometry: PreparedPathGeometry,
    coverage_type: PathCoverageType,
    contour_directions: gpu::ContourDirections,
    atlas_scale_factor: f32,
    gradient: rcp<crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::Gradient>,
    paint_type: gpu::PaintType,
    simple_paint_value: gpu::SimplePaintValue,
    stroke_radius: f32,
    feather_radius: f32,
    feather_atlas_transform: gpu::AtlasTransform,
    feather_atlas_scissor: AABBu16,
    feather_atlas_scissor_enabled: bool,
    coverage_buffer_range: gpu::CoverageBufferRange,
) -> Box<PathDrawAllocation> {
    unsafe {
        (&*path_ref.get()).lockRawPathMutations();
    }
    #[cfg(debug_assertions)]
    let raw_path_mutation_id = unsafe { (&*path_ref.get()).getRawPathMutationID() };
    let image_texture_ptr = image_texture.get();
    let gradient_ptr = gradient.get();
    let tess_vertex_count = geometry.tess_vertex_count() as usize;
    let contour_count = geometry.contour_count();
    let tessellated_segment_record_count = geometry.tessellated_segment_record_count();
    let max_triangle_vertex_count = if coverage_type == PathCoverageType::featherAtlas {
        6
    } else {
        match &geometry {
            PreparedPathGeometry::Interior(interior) => interior.max_triangle_vertex_count,
            PreparedPathGeometry::MidpointFan(_) | PreparedPathGeometry::Stroke(_) => 0,
        }
    };
    let mut base = base_draw(
        DrawObjectType::path,
        pixel_bounds,
        matrix,
        blend_mode,
        image_texture_ptr,
        image_sampler,
        draw_contents,
        clip_id,
        scissor,
    );
    base.count_subpasses = count_path_subpasses;
    base.release_refs = release_path_draw;
    base.allocate_resources = allocate_path_resources;
    base.push_to_render_context = push_path;
    base.simple_paint_value = simple_paint_value;
    base.resource_counts.maxTriangleVertexCount = max_triangle_vertex_count;
    // Source leaves path/contour/tessellation counts at zero when an admitted
    // draw (for example an empty butt-cap stroke) has no tessellation vertices.
    if tess_vertex_count != 0 {
        base.resource_counts.pathCount = 1;
        base.resource_counts.contourCount = contour_count;
        base.resource_counts.maxTessellatedSegmentCount = tessellated_segment_record_count;
        if matches!(&geometry, PreparedPathGeometry::Interior(_)) {
            base.resource_counts.outerCubicTessVertexCount = tess_vertex_count;
        } else {
            base.resource_counts.midpointFanTessVertexCount = tess_vertex_count;
        }
    }
    Box::new(PathDrawAllocation {
        draw: PathDraw {
            base,
            is_stroke: stroke_radius != 0.0,
            feather_atlas_scissor_enabled,
            feather_atlas_scissor,
            push_feather_atlas,
            gradient: gradient_ptr,
            paint_type,
            stroke_radius,
            feather_radius,
            feather_atlas_transform,
            coverage_buffer_range,
            push_interior_triangles,
        },
        path_ref,
        #[cfg(debug_assertions)]
        raw_path_mutation_id,
        geometry,
        image_texture,
        gradient,
        coverage_type,
        contour_directions,
        path_fill_rule: FillRule::NonZero,
        triangulator_fill_rule: FillRule::NonZero,
        triangulator_reverse_triangles: false,
        triangulator_negate_winding: false,
        atlas_scale_factor,
        path_id: 0,
        prepass_tess_location: 0,
        tess_location: 0,
        geometry_written: false,
    })
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn make_image_rect_draw(
    pixel_bounds: IAABB,
    matrix: Mat2D,
    blend_mode: BlendMode,
    opacity: f32,
    image_texture: rcp<gpu::Texture>,
    image_sampler: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler,
    draw_contents: gpu::DrawContents,
    clip_id: u32,
    scissor: Option<AABBu16>,
) -> Box<ImageRectDrawAllocation> {
    let image_texture_ptr = image_texture.get();
    let mut base = base_draw(
        DrawObjectType::imageRect,
        pixel_bounds,
        matrix,
        blend_mode,
        image_texture_ptr,
        image_sampler,
        draw_contents,
        clip_id,
        scissor,
    );
    base.push_to_render_context = push_image_rect;
    base.release_refs = release_image_rect;
    base.resource_counts.imageDrawCount = 1;
    Box::new(ImageRectDrawAllocation {
        draw: ImageRectDraw { base, opacity },
        image_texture,
    })
}

/// Source `ImageRectDraw` constructor used by renderer call sites. The context
/// assertion is part of the constructor contract: image rectangles are only
/// legal when path image paints are unavailable.
#[allow(clippy::too_many_arguments)]
pub unsafe fn make_image_rect_draw_from_source(
    context: &RenderContext,
    pixel_bounds: IAABB,
    matrix: Mat2D,
    blend_mode: BlendMode,
    image_texture: rcp<gpu::Texture>,
    image_sampler: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler,
    opacity: f32,
) -> Box<ImageRectDrawAllocation> {
    debug_assert!(!context.frameSupportsImagePaintForPathsExecutable());
    unsafe {
        make_image_rect_draw(
            pixel_bounds,
            matrix,
            blend_mode,
            opacity,
            image_texture,
            image_sampler,
            gpu::DrawContents::none,
            0,
            None,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn make_image_mesh_draw(
    pixel_bounds: IAABB,
    matrix: Mat2D,
    blend_mode: BlendMode,
    opacity: f32,
    image_texture: rcp<gpu::Texture>,
    image_sampler: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler,
    draw_contents: gpu::DrawContents,
    clip_id: u32,
    scissor: Option<AABBu16>,
    vertex_buffer: rcp<MechanicalRenderBuffer>,
    uv_buffer: rcp<MechanicalRenderBuffer>,
    index_buffer: rcp<MechanicalRenderBuffer>,
    index_count: u32,
) -> Box<ImageMeshDrawAllocation> {
    let image_texture_ptr = image_texture.get();
    let vertex_buffer_ptr = vertex_buffer.get();
    let uv_buffer_ptr = uv_buffer.get();
    let index_buffer_ptr = index_buffer.get();
    let mut base = base_draw(
        DrawObjectType::imageMesh,
        pixel_bounds,
        matrix,
        blend_mode,
        image_texture_ptr,
        image_sampler,
        draw_contents,
        clip_id,
        scissor,
    );
    base.push_to_render_context = push_image_mesh;
    base.release_refs = release_image_mesh;
    base.resource_counts.imageDrawCount = 1;
    Box::new(ImageMeshDrawAllocation {
        draw: ImageMeshDraw {
            base,
            opacity,
            index_count,
            vertex_buffer: vertex_buffer_ptr,
            uv_buffer: uv_buffer_ptr,
            index_buffer: index_buffer_ptr,
        },
        image_texture,
        vertex_buffer,
        uv_buffer,
        index_buffer,
    })
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn make_image_mesh_draw_from_source(
    pixel_bounds: IAABB,
    matrix: Mat2D,
    blend_mode: BlendMode,
    image_texture: rcp<gpu::Texture>,
    image_sampler: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler,
    vertex_buffer: rcp<MechanicalRenderBuffer>,
    uv_buffer: rcp<MechanicalRenderBuffer>,
    index_buffer: rcp<MechanicalRenderBuffer>,
    index_count: u32,
    opacity: f32,
) -> Box<ImageMeshDrawAllocation> {
    debug_assert!(!vertex_buffer.get().is_null());
    debug_assert!(!uv_buffer.get().is_null());
    debug_assert!(!index_buffer.get().is_null());
    unsafe {
        make_image_mesh_draw(
            pixel_bounds,
            matrix,
            blend_mode,
            opacity,
            image_texture,
            image_sampler,
            gpu::DrawContents::none,
            0,
            None,
            vertex_buffer,
            uv_buffer,
            index_buffer,
            index_count,
        )
    }
}

pub fn make_clip_reset(
    pixel_bounds: IAABB,
    previous_clip_id: u32,
    previous_clip_draw_contents: gpu::DrawContents,
    reset_action: ClipResetAction,
) -> Box<ClipReset> {
    let fill_rule_flags = gpu::DrawContents::nonZeroFill
        | gpu::DrawContents::evenOddFill
        | gpu::DrawContents::clockwiseFill;
    let mut draw_contents = previous_clip_draw_contents & fill_rule_flags;
    if reset_action == ClipResetAction::intersectPreviousClip {
        draw_contents |= gpu::DrawContents::activeClip;
    }
    draw_contents |= gpu::DrawContents::clipUpdate;
    let mut base = base_draw(
        DrawObjectType::stencilClipReset,
        pixel_bounds,
        Mat2D::IDENTITY,
        BlendMode::SrcOver,
        core::ptr::null_mut(),
        crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler::LinearClamp(),
        draw_contents,
        0,
        None,
    );
    base.push_to_render_context = push_clip_reset;
    base.resource_counts.maxTriangleVertexCount = 6;
    Box::new(ClipReset {
        base,
        previous_clip_id,
    })
}

/// Source `ClipReset` constructor call site: bounds are owned by the current
/// clip record and must not be supplied independently by a caller.
pub fn make_clip_reset_from_source(
    context: &RenderContext,
    previous_clip_id: u32,
    previous_clip_draw_contents: gpu::DrawContents,
    reset_action: ClipResetAction,
) -> Box<ClipReset> {
    make_clip_reset(
        *context.getClipContentBounds(previous_clip_id),
        previous_clip_id,
        previous_clip_draw_contents,
        reset_action,
    )
}
