use super::semantic_snapshot::Bounds;
use crate::mechanical_port::source::math::{path_types::PathVerb, raw_path::RawPath, vec2d::Vec2D};
use i_overlay::{
    core::{fill_rule::FillRule, overlay_rule::OverlayRule},
    float::{overlay::FloatOverlay, single::SingleFloatOverlay},
};

type Point = [f64; 2];
const CURVE_TOLERANCE: f64 = 0.01;
const MAX_POINTS: usize = 16_384;
const MAX_DEPTH: u8 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticGeometryError {
    InvalidPath,
    LimitExceeded,
}

/// Filled root-space contours, including holes, retained until all clips apply.
#[derive(Default)]
pub(super) struct SemanticClipRegion {
    contours: Vec<Vec<Point>>,
    error: Option<SemanticGeometryError>,
}

impl SemanticClipRegion {
    pub fn from_polygon(polygon: &[Vec2D]) -> Self {
        Self {
            contours: vec![
                polygon
                    .iter()
                    .map(|p| [f64::from(p.x), f64::from(p.y)])
                    .collect(),
            ],
            error: None,
        }
    }
    pub fn status(&self) -> Result<(), SemanticGeometryError> {
        self.error.map_or(Ok(()), Err)
    }
    pub fn is_empty(&self) -> bool {
        self.contours.is_empty()
    }
    pub fn intersect_polygon(&mut self, polygon: &[Vec2D]) {
        let clip = polygon
            .iter()
            .map(|p| [f64::from(p.x), f64::from(p.y)])
            .collect::<Vec<_>>();
        self.contours = self
            .contours
            .overlay_as::<i64>(&clip, OverlayRule::Intersect, FillRule::NonZero)
            .into_iter()
            .flatten()
            .collect();
    }
    pub fn intersect_path(&mut self, path: &RawPath, rule: nuxie_render_api::FillRule) {
        let contours = match flatten_path(path) {
            Ok(contours) => contours,
            Err(error) => {
                self.error = Some(error);
                self.contours.clear();
                return;
            }
        };
        let rule = match rule {
            nuxie_render_api::FillRule::EvenOdd => FillRule::EvenOdd,
            _ => FillRule::NonZero,
        };
        // Normalize each clip with its own fill rule. The intersection then uses
        // nonzero contours for both inputs, preserving holes across mixed rules.
        let clip =
            FloatOverlay::<Point, i64>::from_subj(&contours).overlay(OverlayRule::Subject, rule);
        self.contours = self
            .contours
            .overlay_as::<i64>(&clip, OverlayRule::Intersect, FillRule::NonZero)
            .into_iter()
            .flatten()
            .collect();
    }
    pub fn bounds(&self) -> Bounds {
        let mut result = Bounds::for_expansion();
        for point in self.contours.iter().flatten() {
            result.expand((point[0] as f32, point[1] as f32));
        }
        result
    }
}

fn flatten_path(path: &RawPath) -> Result<Vec<Vec<Point>>, SemanticGeometryError> {
    if path.points().len() > MAX_POINTS {
        return Err(SemanticGeometryError::LimitExceeded);
    }
    if path
        .points()
        .iter()
        .any(|p| !p.x.is_finite() || !p.y.is_finite())
    {
        return Err(SemanticGeometryError::InvalidPath);
    }
    let mut contours = Vec::new();
    let mut contour = Vec::new();
    let mut remaining = MAX_POINTS;
    let point = |p: Vec2D| [f64::from(p.x), f64::from(p.y)];
    for segment in path.segments() {
        match segment.verb {
            PathVerb::Move => {
                if !contour.is_empty() {
                    contours.push(std::mem::take(&mut contour));
                }
                push(&mut contour, point(segment.points[0]), &mut remaining)?;
            }
            PathVerb::Line => push(&mut contour, point(segment.points[1]), &mut remaining)?,
            PathVerb::Quad => {
                let a = point(segment.points[0]);
                let b = point(segment.points[1]);
                let c = point(segment.points[2]);
                cubic(
                    [a, lerp(a, b, 2.0 / 3.0), lerp(c, b, 2.0 / 3.0), c],
                    0,
                    &mut contour,
                    &mut remaining,
                )?;
            }
            PathVerb::Cubic => cubic(
                [
                    point(segment.points[0]),
                    point(segment.points[1]),
                    point(segment.points[2]),
                    point(segment.points[3]),
                ],
                0,
                &mut contour,
                &mut remaining,
            )?,
            PathVerb::Close => {
                if !contour.is_empty() {
                    contours.push(std::mem::take(&mut contour));
                }
            }
        }
    }
    if !contour.is_empty() {
        contours.push(contour);
    }
    Ok(contours)
}
fn push(
    points: &mut Vec<Point>,
    point: Point,
    remaining: &mut usize,
) -> Result<(), SemanticGeometryError> {
    *remaining = remaining
        .checked_sub(1)
        .ok_or(SemanticGeometryError::LimitExceeded)?;
    points.push(point);
    Ok(())
}
fn lerp(a: Point, b: Point, t: f64) -> Point {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
fn distance_to_segment_squared(p: Point, a: Point, b: Point) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let length = dx * dx + dy * dy;
    let t = if length == 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length).clamp(0.0, 1.0)
    };
    let q = lerp(a, b, t);
    (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)
}
fn cubic(
    p: [Point; 4],
    depth: u8,
    points: &mut Vec<Point>,
    remaining: &mut usize,
) -> Result<(), SemanticGeometryError> {
    if distance_to_segment_squared(p[1], p[0], p[3])
        .max(distance_to_segment_squared(p[2], p[0], p[3]))
        <= CURVE_TOLERANCE * CURVE_TOLERANCE
    {
        return push(points, p[3], remaining);
    }
    if depth == MAX_DEPTH {
        return Err(SemanticGeometryError::LimitExceeded);
    }
    let a = lerp(p[0], p[1], 0.5);
    let b = lerp(p[1], p[2], 0.5);
    let c = lerp(p[2], p[3], 0.5);
    let d = lerp(a, b, 0.5);
    let e = lerp(b, c, 0.5);
    let middle = lerp(d, e, 0.5);
    cubic([p[0], a, d, middle], depth + 1, points, remaining)?;
    cubic([middle, e, c, p[3]], depth + 1, points, remaining)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::math::{aabb::Aabb, path_types::PathDirection};

    #[test]
    fn rotated_clip_rejects_empty_corners_in_both_windings() {
        let mut diamond = [
            Vec2D::new(0.0, 1.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(2.0, 1.0),
            Vec2D::new(1.0, 2.0),
        ];
        let corner = [
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.25, 0.0),
            Vec2D::new(0.25, 0.25),
            Vec2D::new(0.0, 0.25),
        ];
        for _ in 0..2 {
            let mut region = SemanticClipRegion::from_polygon(&corner);
            region.intersect_polygon(&diamond);
            assert!(region.is_empty());
            diamond.reverse();
        }
    }

    #[test]
    fn rotated_clip_preserves_only_the_visible_triangle() {
        let diamond = [
            Vec2D::new(0.0, 1.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(2.0, 1.0),
            Vec2D::new(1.0, 2.0),
        ];
        let square = [
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.0, 1.0),
        ];
        let mut region = SemanticClipRegion::from_polygon(&square);
        region.intersect_polygon(&diamond);
        let area: f64 = region
            .contours
            .iter()
            .map(|contour| {
                contour
                    .iter()
                    .zip(contour.iter().cycle().skip(1))
                    .take(contour.len())
                    .map(|(a, b)| a[0] * b[1] - a[1] * b[0])
                    .sum::<f64>()
            })
            .sum();
        assert!((area.abs() - 1.0).abs() < 0.0001);
        assert!(
            region
                .contours
                .iter()
                .flatten()
                .all(|p| p[0] + p[1] >= 1.0 && p[0] <= 1.0 && p[1] <= 1.0)
        );
    }

    #[test]
    fn path_fill_rules_preserve_holes() {
        let target = [
            Vec2D::new(1.5, 1.5),
            Vec2D::new(2.5, 1.5),
            Vec2D::new(2.5, 2.5),
            Vec2D::new(1.5, 2.5),
        ];
        for (inner_direction, rule, empty) in [
            (
                PathDirection::Clockwise,
                nuxie_render_api::FillRule::EvenOdd,
                true,
            ),
            (
                PathDirection::Clockwise,
                nuxie_render_api::FillRule::NonZero,
                false,
            ),
            (
                PathDirection::Counterclockwise,
                nuxie_render_api::FillRule::NonZero,
                true,
            ),
        ] {
            let mut path = RawPath::default();
            path.add_rect(Aabb::new(0.0, 0.0, 4.0, 4.0), PathDirection::Clockwise);
            path.add_rect(Aabb::new(1.0, 1.0, 3.0, 3.0), inner_direction);
            let mut region = SemanticClipRegion::from_polygon(&target);
            region.intersect_path(&path, rule);
            assert_eq!(region.is_empty(), empty);
        }
    }

    #[test]
    fn collinear_reversing_curve_is_not_replaced_by_its_zero_chord() {
        let mut path = RawPath::default();
        path.move_to(0.0, 0.0);
        path.cubic_to(10.0, 0.0, -10.0, 0.0, 0.0, 0.0);
        let points = flatten_path(&path).unwrap();
        assert!(points.iter().flatten().any(|p| p[0] > 2.0));
        assert!(points.iter().flatten().any(|p| p[0] < -2.0));
    }

    #[test]
    fn valid_over_budget_path_reports_limit_instead_of_empty_geometry() {
        let mut path = RawPath::default();
        for _ in 0..MAX_POINTS {
            path.add_rect(Aabb::new(0.0, 0.0, 4.0, 4.0), PathDirection::Clockwise);
        }
        let target = [
            Vec2D::new(1.0, 1.0),
            Vec2D::new(2.0, 1.0),
            Vec2D::new(2.0, 2.0),
            Vec2D::new(1.0, 2.0),
        ];
        let mut region = SemanticClipRegion::from_polygon(&target);
        region.intersect_path(&path, nuxie_render_api::FillRule::NonZero);
        assert_eq!(region.status(), Err(SemanticGeometryError::LimitExceeded));
        let mut empty = SemanticClipRegion::from_polygon(&target);
        empty.intersect_path(&RawPath::default(), nuxie_render_api::FillRule::NonZero);
        assert!(empty.is_empty());
        assert_eq!(empty.status(), Ok(()));
    }

    #[test]
    fn nonfinite_path_is_rejected() {
        let mut path = RawPath::default();
        path.move_to(f32::NAN, 0.0);
        assert_eq!(flatten_path(&path), Err(SemanticGeometryError::InvalidPath));
    }
}
