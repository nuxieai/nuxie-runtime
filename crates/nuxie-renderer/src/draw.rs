//! CPU path preparation translated from `renderer/src/draw.cpp`.

use crate::gpu::{
    AtlasTransform, ContourData, CoverageBufferRange, PathData, TessVertexSpan, TriangleVertex,
    BEVEL_JOIN_CONTOUR_FLAG, CONTOUR_ID_MASK, CULL_EXCESS_TESSELLATION_SEGMENTS_CONTOUR_FLAG,
    EMULATED_STROKE_CAP_CONTOUR_FLAG, MAX_PARAMETRIC_SEGMENTS, MIDPOINT_FAN_PATCH_SEGMENT_SPAN,
    MITER_CLIP_JOIN_CONTOUR_FLAG, MITER_REVERT_JOIN_CONTOUR_FLAG, OUTER_CURVE_PATCH_SEGMENT_SPAN,
    PARAMETRIC_PRECISION, POLAR_PRECISION, ROUND_JOIN_CONTOUR_FLAG,
};
use bytemuck::Zeroable;
use nuxie_render_api::{Mat2D, PathVerb, RawPath, StrokeCap, StrokeJoin, Vec2D};

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

pub(crate) struct InteriorTessellation {
    pub spans: Vec<TessVertexSpan>,
    pub path: PathData,
    pub contours: Vec<ContourData>,
    pub triangles: Vec<TriangleVertex>,
    pub base_instance: u32,
    pub instance_count: u32,
}

#[derive(Clone)]
struct StrokeCurve {
    cubic: [Vec2D; 4],
    is_line: bool,
}

struct StrokeContour {
    curves: Vec<StrokeCurve>,
    first: Vec2D,
    current: Vec2D,
    closed: bool,
}

pub(crate) fn build_stroke_tessellation(
    path: &RawPath,
    transform: Mat2D,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
) -> Option<FillTessellation> {
    let contours = stroke_contours(path)?;
    let stroke_radius = thickness * 0.5;
    if stroke_radius <= 0.0 || contours.is_empty() {
        return None;
    }
    let matrix_scale = max_matrix_scale(transform);
    let polar_segments_per_radian = polar_segments_per_radian(stroke_radius * matrix_scale);
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
    let join_flags = match join {
        StrokeJoin::Miter => MITER_REVERT_JOIN_CONTOUR_FLAG,
        StrokeJoin::Round => ROUND_JOIN_CONTOUR_FLAG,
        StrokeJoin::Bevel => BEVEL_JOIN_CONTOUR_FLAG,
    };
    let mut spans = Vec::new();
    let mut contour_data = Vec::with_capacity(contours.len());
    let mut location = MIDPOINT_FAN_PATCH_SEGMENT_SPAN as i32;
    push_padding_span(&mut spans, 0, location);
    let path_start = location;
    for (contour_index, contour) in contours.iter().enumerate() {
        let mut curves = contour.curves.clone();
        if contour.closed && !same_point(contour.first, contour.current) {
            curves.push(StrokeCurve {
                cubic: line_cubic(contour.current, contour.first),
                is_line: true,
            });
        }
        curves.retain(|curve| !curve.is_line || !same_point(curve.cubic[0], curve.cubic[3]));
        if curves.is_empty() {
            continue;
        }
        let prepared = curves
            .iter()
            .map(|curve| {
                let tangents = cubic_tangents(curve.cubic);
                let (parametric, polar) = if curve.is_line {
                    (1, 1)
                } else {
                    if cubic_requires_convex_180_chop(curve.cubic) {
                        return None;
                    }
                    let transformed = curve.cubic.map(|point| transform.transform_point(point));
                    (
                        cubic_segment_count(transformed),
                        round_join_segment_count(
                            tangents[0],
                            tangents[1],
                            polar_segments_per_radian,
                        ),
                    )
                };
                Some((curve.cubic, tangents, parametric, polar))
            })
            .collect::<Option<Vec<_>>>()?;
        let contour_start = location as u32;
        let contour_id = (contour_index as u32 + 1) & CONTOUR_ID_MASK;
        let mut pending = Vec::new();
        if !contour.closed {
            let (cubic, tangents, _, _) = prepared[0];
            pending.push((
                [cubic[3], cubic[2], cubic[1], cubic[0]],
                tangents[0],
                0,
                0,
                cap_segments,
                contour_id | cap_flags,
            ));
        }
        for (index, (cubic, tangents, parametric, polar)) in prepared.iter().copied().enumerate() {
            let final_open = !contour.closed && index + 1 == prepared.len();
            let (join_tangent, join_segments, flags) = if final_open {
                (negate(tangents[1]), cap_segments, contour_id | cap_flags)
            } else {
                let next_tangent = prepared[(index + 1) % prepared.len()].1[0];
                let segment_count = if join == StrokeJoin::Round {
                    round_join_segment_count(tangents[1], next_tangent, polar_segments_per_radian)
                } else {
                    5
                };
                (next_tangent, segment_count, contour_id | join_flags)
            };
            pending.push((cubic, join_tangent, parametric, polar, join_segments, flags));
        }
        let vertex_count = pending
            .iter()
            .map(|(_, _, parametric, polar, join, _)| parametric + polar + join - 1)
            .sum::<u32>() as i32;
        let padding = align_up(vertex_count, MIDPOINT_FAN_PATCH_SEGMENT_SPAN as i32) - vertex_count;
        contour_data.push(ContourData::new(
            [if contour.closed { 1.0 } else { 0.0 }, 0.0],
            0,
            contour_start,
        ));
        for (index, (curve, tangent, parametric, polar, join, flags)) in
            pending.into_iter().enumerate()
        {
            let x0 = location;
            location += parametric as i32 + polar as i32 + join as i32 - 1
                + i32::from(index == 0) * padding;
            spans.push(TessVertexSpan::without_reflection(
                curve.map(|point| [point.x, point.y]),
                [tangent.x, tangent.y],
                0.0,
                x0,
                location,
                parametric,
                polar,
                join,
                flags,
            ));
        }
    }
    if contour_data.is_empty() {
        return None;
    }
    Some(FillTessellation {
        spans,
        path: PathData::new(
            transform,
            stroke_radius,
            0.0,
            0,
            AtlasTransform::zeroed(),
            CoverageBufferRange::zeroed(),
        ),
        contours: contour_data,
        base_instance: 1,
        instance_count: (location - path_start) as u32 / MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32,
    })
}

fn stroke_contours(path: &RawPath) -> Option<Vec<StrokeContour>> {
    let mut contours = Vec::new();
    let mut contour = None::<StrokeContour>;
    let mut point_index = 0;
    for verb in path.verbs() {
        match verb {
            PathVerb::Move => {
                if let Some(contour) = contour.take() {
                    contours.push(contour);
                }
                let point = path.points()[point_index];
                contour = Some(StrokeContour {
                    curves: Vec::new(),
                    first: point,
                    current: point,
                    closed: false,
                });
                point_index += 1;
            }
            PathVerb::Line => {
                let end = path.points()[point_index];
                let contour = contour.as_mut()?;
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
                let contour = contour.as_mut()?;
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
                let contour = contour.as_mut()?;
                contour.curves.push(StrokeCurve {
                    cubic: [contour.current, control0, control1, end],
                    is_line: false,
                });
                contour.current = end;
                point_index += 3;
            }
            PathVerb::Close => contour.as_mut()?.closed = true,
        }
    }
    if let Some(contour) = contour {
        contours.push(contour);
    }
    Some(contours)
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

fn cubic_requires_convex_180_chop(points: [Vec2D; 4]) -> bool {
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
        return inside(c / b_over_minus_2);
    }
    if discriminant_over_4 <= cusp_threshold {
        if a != 0.0 || b_over_minus_2 != 0.0 || c != 0.0 {
            return inside(b_over_minus_2 / a);
        }
        let base = subtract(points[3], points[0]);
        let ordered = points
            .windows(2)
            .all(|points| dot(points[1], base) > dot(points[0], base));
        if ordered {
            return false;
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
    inside(q / a) || inside(c / q)
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

fn round_join_segment_count(incoming: Vec2D, outgoing: Vec2D, per_radian: f32) -> u32 {
    let denominator = ((incoming.x * incoming.x + incoming.y * incoming.y)
        * (outgoing.x * outgoing.x + outgoing.y * outgoing.y))
        .sqrt();
    let cosine =
        ((incoming.x * outgoing.x + incoming.y * outgoing.y) / denominator).clamp(-1.0, 1.0);
    (cosine.acos() * per_radian)
        .ceil()
        .clamp(1.0, crate::gpu::MAX_POLAR_SEGMENTS as f32) as u32
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

pub(crate) fn build_interior_tessellation(
    path: &RawPath,
    transform: Mat2D,
) -> Option<InteriorTessellation> {
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
    let base = OUTER_CURVE_PATCH_SEGMENT_SPAN as i32;
    let curve_count = cubic_contours.iter().map(Vec::len).sum::<usize>();
    let half_vertex_count = (curve_count * OUTER_CURVE_PATCH_SEGMENT_SPAN) as i32;
    let mut spans = Vec::with_capacity(curve_count + 1);
    push_padding_span(&mut spans, 0, base);
    let mut contours = Vec::with_capacity(cubic_contours.len());
    let mut triangles = Vec::new();
    let mut curve_offset = 0i32;
    for (contour_index, curves) in cubic_contours.iter().enumerate() {
        let points = curves.iter().map(|curve| curve[0]).collect::<Vec<_>>();
        let indices = triangulate_contour(&points)?;
        let transformed = points
            .iter()
            .copied()
            .map(|point| transform.transform_point(point))
            .collect::<Vec<_>>();
        let winding = if signed_area(&transformed) >= 0.0 {
            -1
        } else {
            1
        };
        triangles.extend(indices.into_iter().map(|index| {
            let point = points[index as usize];
            TriangleVertex::new([point.x, point.y], winding, 1)
        }));
        contours.push(ContourData::new(
            [0.0, 0.0],
            1,
            (base + half_vertex_count + curve_offset) as u32,
        ));
        for curve in curves {
            let reflection_x0 = base + half_vertex_count - curve_offset;
            let reflection_x1 = reflection_x0 - OUTER_CURVE_PATCH_SEGMENT_SPAN as i32;
            let x0 = base + half_vertex_count + curve_offset;
            let x1 = x0 + OUTER_CURVE_PATCH_SEGMENT_SPAN as i32;
            let mut span = TessVertexSpan::without_reflection(
                curve.map(|point| [point.x, point.y]),
                [0.0, 0.0],
                0.0,
                x0,
                x1,
                16,
                1,
                1,
                ((contour_index as u32 + 1) & CONTOUR_ID_MASK)
                    | CULL_EXCESS_TESSELLATION_SEGMENTS_CONTOUR_FLAG,
            );
            span.set_ranges(x0, x1, reflection_x0, reflection_x1, 0.0);
            spans.push(span);
            curve_offset += OUTER_CURVE_PATCH_SEGMENT_SPAN as i32;
        }
    }
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
        base_instance: 1,
        instance_count: (curve_count * 2) as u32,
    })
}

fn outer_cubic_subdivision_count(points: [Vec2D; 4], transform: Mat2D) -> u32 {
    let [xx, yx, xy, yy, _, _] = transform.0;
    let transformed_second_difference = |a: Vec2D, b: Vec2D, c: Vec2D| {
        let x = a.x - 2.0 * b.x + c.x;
        let y = a.y - 2.0 * b.y + c.y;
        let transformed_x = xx * x + xy * y;
        let transformed_y = yx * x + yy * y;
        transformed_x * transformed_x + transformed_y * transformed_y
    };
    let max_length_squared = transformed_second_difference(points[0], points[1], points[2]).max(
        transformed_second_difference(points[1], points[2], points[3]),
    );
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
        let base = self.base_instance * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let half_vertex_count = self.instance_count * MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        for span in &mut self.spans {
            if span.contour_id_with_flags & CONTOUR_ID_MASK == 0 {
                continue;
            }
            let (x0, x1) = span.x_range();
            let offset = x0 - base as i32;
            let reflection_x0 = (base + half_vertex_count) as i32 - offset;
            let reflection_x1 = reflection_x0 - (x1 - x0);
            span.set_ranges(
                x0 + half_vertex_count as i32,
                x1 + half_vertex_count as i32,
                reflection_x0,
                reflection_x1,
                0.0,
            );
        }
        for contour in &mut self.contours {
            contour.vertex_index0 += half_vertex_count;
        }
        self.instance_count *= 2;
    }
}

pub(crate) fn build_fill_tessellation(
    path: &RawPath,
    transform: Mat2D,
) -> Option<FillTessellation> {
    let contours = cubic_contours(path);
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
            .map(|curve| cubic_segment_count(curve.map(|point| transform.transform_point(point))))
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
            spans.push(TessVertexSpan::without_reflection(
                curve.map(|point| [point.x, point.y]),
                [0.0, 0.0],
                0.0,
                x0,
                location,
                segments,
                1,
                1,
                (index as u32 + 1) & CONTOUR_ID_MASK,
            ));
        }
    }
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

fn cubic_contours(path: &RawPath) -> Vec<Vec<[Vec2D; 4]>> {
    let mut result = Vec::new();
    let mut curves = Vec::new();
    let mut first = None;
    let mut current = None;
    let mut point_index = 0;
    let finish = |result: &mut Vec<Vec<[Vec2D; 4]>>,
                  curves: &mut Vec<[Vec2D; 4]>,
                  first: &mut Option<Vec2D>,
                  current: &mut Option<Vec2D>| {
        if let (Some(start), Some(end)) = (*first, *current) {
            if !same_point(start, end) {
                curves.push(line_cubic(end, start));
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
                    curves.push(line_cubic(start, end));
                }
                current = Some(end);
            }
            PathVerb::Quad => {
                let control = path.points()[point_index];
                let end = path.points()[point_index + 1];
                point_index += 2;
                if let Some(start) = current {
                    curves.push([
                        start,
                        lerp(start, control, 2.0 / 3.0),
                        lerp(end, control, 2.0 / 3.0),
                        end,
                    ]);
                }
                current = Some(end);
            }
            PathVerb::Cubic => {
                let control0 = path.points()[point_index];
                let control1 = path.points()[point_index + 1];
                let end = path.points()[point_index + 2];
                point_index += 3;
                if let Some(start) = current {
                    curves.push([start, control0, control1, end]);
                }
                current = Some(end);
            }
            PathVerb::Close => {
                if let (Some(start), Some(end)) = (first, current) {
                    if !same_point(start, end) {
                        curves.push(line_cubic(end, start));
                    }
                    current = Some(start);
                }
            }
        }
    }
    finish(&mut result, &mut curves, &mut first, &mut current);
    result
}

fn line_cubic(start: Vec2D, end: Vec2D) -> [Vec2D; 4] {
    [
        start,
        lerp(start, end, 1.0 / 3.0),
        lerp(start, end, 2.0 / 3.0),
        end,
    ]
}

fn contour_midpoint(curves: &[[Vec2D; 4]]) -> Vec2D {
    let mut sum = Vec2D::new(0.0, 0.0);
    for curve in curves {
        sum.x += curve[0].x;
        sum.y += curve[0].y;
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

fn append_cubic(contour: &mut Contour, cubic: [Vec2D; 4]) {
    let segment_count = cubic_segment_count(cubic);
    for segment in 1..=segment_count {
        contour
            .points
            .push(eval_cubic(cubic, segment as f32 / segment_count as f32));
    }
}

pub(crate) fn cubic_segment_count(points: [Vec2D; 4]) -> u32 {
    let second_difference = |a: Vec2D, b: Vec2D, c: Vec2D| {
        let x = a.x - 2.0 * b.x + c.x;
        let y = a.y - 2.0 * b.y + c.y;
        x * x + y * y
    };
    let max_length_squared = second_difference(points[0], points[1], points[2])
        .max(second_difference(points[1], points[2], points[3]));
    let length_term_squared = (9.0 / 16.0) * (PARAMETRIC_PRECISION as f32).powi(2);
    (max_length_squared * length_term_squared)
        .sqrt()
        .sqrt()
        .ceil()
        .clamp(1.0, MAX_PARAMETRIC_SEGMENTS as f32) as u32
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
    Vec2D::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn removes_repeated_endpoint_from_closed_cubic_contour() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.cubic_to(10.0, 0.0, 10.0, 10.0, 0.0, 0.0);
        path.close();
        let contour = flatten_path(&path, Mat2D::IDENTITY).remove(0);
        assert_ne!(contour.points.first(), contour.points.last());
        assert!(triangulate_contour(&contour.points).is_some());
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
        assert_eq!(tessellation.spans.len(), 4);
        assert_eq!(tessellation.spans[0].x0_x1 as u32, 0x0008_0000);
        assert_eq!(tessellation.spans[1].x0_x1 as u32, 0x000c_0008);
        assert_eq!(tessellation.spans[3].x0_x1 as u32, 0x0010_000e);
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
        assert_eq!(tessellation.spans[1].x_range(), (16, 20));
        assert_eq!(tessellation.spans[1].reflection_x0_x1 as u32, 0x000c_0010);
    }

    #[test]
    fn interior_layout_emits_outer_patches_and_weighted_triangles() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(100.0, 0.0);
        path.line_to(100.0, 100.0);
        path.line_to(0.0, 100.0);
        path.close();
        let tessellation = build_interior_tessellation(&path, Mat2D::IDENTITY).unwrap();
        assert_eq!(tessellation.spans.len(), 5);
        assert_eq!(tessellation.base_instance, 1);
        assert_eq!(tessellation.instance_count, 8);
        assert_eq!(tessellation.triangles.len(), 6);
        assert_eq!(tessellation.triangles[0].weight_path_id >> 16, -1);
        assert_eq!(tessellation.triangles[0].weight_path_id as u16, 1);
        assert_eq!(tessellation.contours[0].vertex_index0, 85);
        assert_eq!(
            tessellation.spans[1].contour_id_with_flags,
            CULL_EXCESS_TESSELLATION_SEGMENTS_CONTOUR_FLAG | 1
        );
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
        let tessellation =
            build_interior_tessellation(&path, Mat2D([100.0, 0.0, 0.0, 100.0, 0.0, 0.0])).unwrap();
        assert!(tessellation.spans.len() > 3);
        assert_eq!(
            tessellation.instance_count as usize,
            (tessellation.spans.len() - 1) * 2
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
        assert_eq!(tessellation.spans.len(), 3);
        assert_eq!(tessellation.spans[1].x_range(), (8, 18));
        assert_eq!(tessellation.spans[2].x_range(), (18, 24));
        assert_eq!(
            tessellation.spans[1].contour_id_with_flags,
            1 | BEVEL_JOIN_CONTOUR_FLAG | EMULATED_STROKE_CAP_CONTOUR_FLAG
        );
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
        assert!(tessellation.spans[2].segment_counts & 1023 > 1);
        assert!(tessellation.spans[2].segment_counts >> 10 & 1023 > 1);
    }

    #[test]
    fn cusp_cubic_stroke_waits_for_straddled_chop_port() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.cubic_to(100.0, 0.0, -100.0, 0.0, 0.0, 0.0);
        assert!(build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            2.0,
            StrokeJoin::Round,
            StrokeCap::Round,
        )
        .is_none());
    }

    #[test]
    fn stroke_budget_uses_maximum_singular_scale_under_shear() {
        let scale = max_matrix_scale(Mat2D([1.0, 0.0, 1.0, 1.0, 0.0, 0.0]));
        assert!((scale - 1.618_034).abs() < 1e-5);
    }
}
