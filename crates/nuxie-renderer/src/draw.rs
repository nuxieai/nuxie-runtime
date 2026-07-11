//! CPU path preparation translated from `renderer/src/draw.cpp`.

use crate::gpu::{
    AtlasTransform, ContourData, CoverageBufferRange, PathData, TessVertexSpan, CONTOUR_ID_MASK,
    MAX_PARAMETRIC_SEGMENTS, MIDPOINT_FAN_PATCH_SEGMENT_SPAN, PARAMETRIC_PRECISION,
};
use bytemuck::Zeroable;
use nuxie_render_api::{Mat2D, PathVerb, RawPath, Vec2D};

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
        let vertex_index0 = (location - path_start) as u32;
        let midpoint = contour_midpoint(curves);
        contour_data.push(ContourData::new([midpoint.x, midpoint.y], 0, vertex_index0));
        for curve in curves {
            let transformed = curve.map(|point| transform.transform_point(point));
            let segments = cubic_segment_count(transformed);
            let x0 = location;
            location += segments as i32;
            spans.push(TessVertexSpan::without_reflection(
                curve.map(|point| [point.x, point.y]),
                [0.0, 0.0],
                0.0,
                x0,
                location,
                segments,
                0,
                1,
                (index as u32 + 1) & CONTOUR_ID_MASK,
            ));
        }
    }
    let used = location - path_start;
    let aligned = align_up(used, MIDPOINT_FAN_PATCH_SEGMENT_SPAN as i32);
    if aligned > used {
        push_padding_span(&mut spans, location, location + aligned - used);
        location += aligned - used;
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
        assert_eq!(tessellation.spans.len(), 5);
        assert_eq!(tessellation.spans[0].x0_x1 as u32, 0x0008_0000);
        assert_eq!(tessellation.spans[1].x0_x1 as u32, 0x0009_0008);
        assert_eq!(tessellation.spans[4].x0_x1 as u32, 0x0010_000b);
    }
}
