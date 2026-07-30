//! Rectangle-union contour construction.
//!
//! Literal Rust port of pinned C++ `src/math/rectangles_to_contour.cpp`.
//! The retained scratch vectors correspond one-for-one with the C++ owner so
//! repeated computations reuse allocation. A small ordered vector map stands
//! in for C++'s TESTING `std::map`; rectangle contours are small and this keeps
//! cross-platform contour order deterministic without introducing a second
//! geometry subsystem.

use std::cmp::Ordering;

use nuxie_render_api::{Aabb, Vec2D};

#[derive(Debug, Clone, Copy)]
struct RectEvent {
    index: usize,
    event_type: u8,
    x: f32,
    y: f32,
}

impl RectEvent {
    fn value(self, axis: u8) -> f32 {
        if axis == 0 { self.x } else { self.y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ContourPoint {
    point: Vec2D,
    direction: u8,
}

type EdgeMap = Vec<(Vec2D, Vec2D)>;

/// Clone-owned counterpart of C++ `RectanglesToContour`.
#[derive(Debug, Default)]
pub(crate) struct RuntimeRectanglesToContour {
    rect_events: Vec<RectEvent>,
    edges_h: EdgeMap,
    edges_v: EdgeMap,
    unique_points: Vec<Vec2D>,
    rects: Vec<Aabb>,
    subdivided_rects: Vec<Aabb>,
    rect_inclusion_bits: Vec<u8>,
    sorted_points_x: Vec<Vec2D>,
    sorted_points_y: Vec<Vec2D>,
    contour_points: Vec<ContourPoint>,
    contour_offsets: Vec<usize>,
}

impl RuntimeRectanglesToContour {
    /// Mirrors `RectanglesToContour::addRect`, including adjacent-row
    /// coalescing (`rectangles_to_contour.cpp:134-154`).
    pub(crate) fn add_rect(&mut self, rect: Aabb) {
        if let Some(last) = self.rects.last_mut()
            && last.min_y == rect.min_y
            && last.max_y == rect.max_y
            && last.max_x == rect.min_x
        {
            last.max_x = rect.max_x;
            return;
        }
        self.rects.push(rect);
    }

    /// Clears authored rectangles while retaining every scratch allocation.
    pub(crate) fn reset(&mut self) {
        self.rects.clear();
    }

    pub(crate) fn compute_contours(&mut self) {
        self.subdivide_rectangles();
        self.unique_points.clear();
        for rect in self.subdivided_rects.iter().copied() {
            for point in [
                Vec2D::new(rect.min_x, rect.min_y),
                Vec2D::new(rect.max_x, rect.min_y),
                Vec2D::new(rect.max_x, rect.max_y),
                Vec2D::new(rect.min_x, rect.max_y),
            ] {
                toggle_unique_point(&mut self.unique_points, point);
            }
        }

        self.sorted_points_x.clone_from(&self.unique_points);
        self.sorted_points_y.clone_from(&self.unique_points);
        self.sorted_points_x.sort_by(compare_x_then_y);
        self.sorted_points_y.sort_by(compare_y_then_x);
        self.edges_h.clear();
        self.edges_v.clear();

        let mut index = 0;
        while index < self.sorted_points_y.len() {
            let current_y = self.sorted_points_y[index].y;
            while index < self.sorted_points_y.len() && self.sorted_points_y[index].y == current_y {
                let first = self.sorted_points_y[index];
                let second = self.sorted_points_y[index + 1];
                insert_edge(&mut self.edges_h, first, second);
                insert_edge(&mut self.edges_h, second, first);
                index += 2;
            }
        }

        index = 0;
        while index < self.sorted_points_x.len() {
            let current_x = self.sorted_points_x[index].x;
            while index < self.sorted_points_x.len() && self.sorted_points_x[index].x == current_x {
                let first = self.sorted_points_x[index];
                let second = self.sorted_points_x[index + 1];
                insert_edge(&mut self.edges_v, first, second);
                insert_edge(&mut self.edges_v, second, first);
                index += 2;
            }
        }

        self.contour_points.clear();
        self.contour_offsets.clear();
        extract_polygons(
            &mut self.contour_points,
            &mut self.contour_offsets,
            &mut self.edges_h,
            &mut self.edges_v,
        );
    }

    pub(crate) fn contour_count(&self) -> usize {
        self.contour_offsets.len()
    }

    pub(crate) fn contour(&self, index: usize) -> RuntimeRectangleContour<'_> {
        let end = self.contour_offsets[index];
        let start = index
            .checked_sub(1)
            .map_or(0, |previous| self.contour_offsets[previous]);
        RuntimeRectangleContour {
            points: &self.contour_points[start..end],
        }
    }

    fn subdivide_rectangles(&mut self) {
        self.subdivided_rects.clear();
        self.unique_points.clear();
        self.rect_events.clear();
        if self.rects.is_empty() {
            return;
        }

        let vertical = sort_rect_events(&self.rects, 0, 1);
        let horizontal = sort_rect_events(&self.rects, 1, 0);
        self.rect_events.reserve(vertical.len() + horizontal.len());
        self.rect_events.extend_from_slice(&vertical);
        self.rect_events.extend_from_slice(&horizontal);

        self.rect_inclusion_bits.resize(self.rects.len() / 8 + 1, 0);
        self.rect_inclusion_bits.fill(0);
        mark_rect_included(&mut self.rect_inclusion_bits, vertical[0].index, true);

        let mut opened = 0i32;
        let mut begin_y = 0.0;
        for pair in vertical.windows(2) {
            let event_v = pair[0];
            mark_rect_included(
                &mut self.rect_inclusion_bits,
                event_v.index,
                event_v.event_type == 0,
            );
            let begin_x = event_v.x;
            let end_x = pair[1].x;
            if end_x - begin_x == 0.0 {
                continue;
            }

            for (horizontal_index, event_h) in horizontal.iter().copied().enumerate() {
                if !is_rect_included(&self.rect_inclusion_bits, event_h.index) {
                    continue;
                }
                if event_h.event_type == 0 {
                    opened += 1;
                    if opened == 1 {
                        begin_y = event_h.y;
                    }
                    continue;
                }

                opened -= 1;
                let next = horizontal[horizontal_index + 1..]
                    .iter()
                    .copied()
                    .find(|candidate| is_rect_included(&self.rect_inclusion_bits, candidate.index));
                if next.is_none_or(|next| opened == 0 && next.y != event_h.y) {
                    self.subdivided_rects
                        .push(Aabb::new(begin_x, begin_y, end_x, event_h.y));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeRectangleContour<'a> {
    points: &'a [ContourPoint],
}

impl RuntimeRectangleContour<'_> {
    pub(crate) fn len(self) -> usize {
        self.points.len()
    }

    pub(crate) fn point(self, index: usize) -> Vec2D {
        self.points[index].point
    }

    pub(crate) fn point_reversed(self, index: usize) -> Vec2D {
        self.points[self.points.len() - 1 - index].point
    }

    pub(crate) fn is_clockwise(self) -> bool {
        if self.points.is_empty() {
            return true;
        }
        let mut area = 0.0;
        for pair in self.points.windows(2) {
            area += cross(pair[0].point, pair[1].point);
        }
        area += cross(
            self.points[self.points.len() - 1].point,
            self.points[0].point,
        );
        area * 0.5 >= 0.0
    }

    /// Mirrors `ContourPointItr`: consecutive equal points are yielded once.
    pub(crate) fn points(&self) -> impl Iterator<Item = Vec2D> + '_ {
        self.points
            .iter()
            .map(|point| point.point)
            .enumerate()
            .filter_map(|(index, point)| {
                (index == 0 || self.points[index - 1].point != point).then_some(point)
            })
    }
}

fn sort_rect_events(rects: &[Aabb], axis_a: u8, axis_b: u8) -> Vec<RectEvent> {
    let mut result = Vec::with_capacity(rects.len() * 2);
    for (index, rect) in rects.iter().copied().enumerate() {
        for point_index in 0..2 {
            let point = if point_index == 0 {
                Vec2D::new(rect.min_x, rect.min_y)
            } else {
                Vec2D::new(rect.max_x, rect.max_y)
            };
            let coordinate = |axis| if axis == 0 { point.x } else { point.y };
            result.push(RectEvent {
                index,
                event_type: point_index,
                y: if axis_b == 0 {
                    coordinate(axis_a)
                } else {
                    coordinate(axis_b)
                },
                x: if axis_b == 0 {
                    coordinate(axis_b)
                } else {
                    coordinate(axis_a)
                },
            });
        }
    }
    result.sort_by(|left, right| compare_float(left.value(axis_b), right.value(axis_b)));
    result.sort_by(|left, right| compare_float(left.value(axis_a), right.value(axis_a)));
    result
}

fn extract_polygons(
    contour_points: &mut Vec<ContourPoint>,
    contour_offsets: &mut Vec<usize>,
    edges_h: &mut EdgeMap,
    edges_v: &mut EdgeMap,
) {
    while let Some(start) = edges_h.first().map(|edge| edge.0) {
        let contour_start = contour_points.len();
        remove_edge(edges_h, start);
        let first = ContourPoint {
            point: start,
            direction: 0,
        };
        contour_points.push(first);

        loop {
            let current = contour_points[contour_points.len() - 1];
            let edge = if current.direction == 0 {
                take_edge(edges_v, current.point).map(|point| ContourPoint {
                    point,
                    direction: 1,
                })
            } else {
                take_edge(edges_h, current.point).map(|point| ContourPoint {
                    point,
                    direction: 0,
                })
            };
            let Some(next) = edge else {
                break;
            };
            contour_points.push(next);
            if next == first {
                contour_points.pop();
                break;
            }
        }

        contour_offsets.push(contour_points.len());
        for point in contour_points[contour_start..]
            .iter()
            .map(|point| point.point)
        {
            remove_edge(edges_h, point);
            remove_edge(edges_v, point);
        }
    }
}

fn insert_edge(edges: &mut EdgeMap, key: Vec2D, value: Vec2D) {
    remove_edge(edges, key);
    let index = edges
        .binary_search_by(|edge| compare_x_then_y(&edge.0, &key))
        .unwrap_or_else(|index| index);
    edges.insert(index, (key, value));
}

fn take_edge(edges: &mut EdgeMap, key: Vec2D) -> Option<Vec2D> {
    let index = edges
        .binary_search_by(|edge| compare_x_then_y(&edge.0, &key))
        .ok()?;
    Some(edges.remove(index).1)
}

fn remove_edge(edges: &mut EdgeMap, key: Vec2D) {
    if let Ok(index) = edges.binary_search_by(|edge| compare_x_then_y(&edge.0, &key)) {
        edges.remove(index);
    }
}

fn toggle_unique_point(points: &mut Vec<Vec2D>, point: Vec2D) {
    if let Some(index) = points.iter().position(|candidate| *candidate == point) {
        points.swap_remove(index);
    } else {
        points.push(point);
    }
}

fn mark_rect_included(bits: &mut [u8], index: usize, included: bool) {
    let mask = 1 << (index % 8);
    if included {
        bits[index / 8] |= mask;
    } else {
        bits[index / 8] &= !mask;
    }
}

fn is_rect_included(bits: &[u8], index: usize) -> bool {
    bits[index / 8] & (1 << (index % 8)) != 0
}

fn compare_x_then_y(left: &Vec2D, right: &Vec2D) -> Ordering {
    compare_float(left.x, right.x).then_with(|| compare_float(left.y, right.y))
}

fn compare_y_then_x(left: &Vec2D, right: &Vec2D) -> Ordering {
    compare_float(left.y, right.y).then_with(|| compare_float(left.x, right.x))
}

fn compare_float(left: f32, right: f32) -> Ordering {
    left.total_cmp(&right)
}

fn cross(left: Vec2D, right: Vec2D) -> f32 {
    left.x * right.y - left.y * right.x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_and_separate_rectangles_match_cpp_contours() {
        // Direct fixture from pinned C++
        // `tests/unit_tests/runtime/rectangles_to_contour_test.cpp:7-38`.
        let mut converter = RuntimeRectanglesToContour::default();
        converter.add_rect(Aabb::new(10.0, 10.0, 20.0, 20.0));
        converter.add_rect(Aabb::new(20.0, 10.0, 30.0, 20.0));
        converter.compute_contours();
        assert_contour(
            converter.contour(0),
            &[(10.0, 10.0), (10.0, 20.0), (30.0, 20.0), (30.0, 10.0)],
        );

        converter.reset();
        converter.add_rect(Aabb::new(10.0, 10.0, 20.0, 20.0));
        converter.add_rect(Aabb::new(20.0, 10.0, 30.0, 20.0));
        converter.add_rect(Aabb::new(20.0, 40.0, 30.0, 50.0));
        converter.compute_contours();
        assert_eq!(converter.contour_count(), 2);
        assert_contour(
            converter.contour(0),
            &[(10.0, 10.0), (10.0, 20.0), (30.0, 20.0), (30.0, 10.0)],
        );
        assert_contour(
            converter.contour(1),
            &[(20.0, 40.0), (20.0, 50.0), (30.0, 50.0), (30.0, 40.0)],
        );
    }

    #[test]
    fn contour_iteration_orientation_and_reversal_match_cpp_owner() {
        let mut converter = RuntimeRectanglesToContour::default();
        converter.add_rect(Aabb::new(0.0, 0.0, 4.0, 2.0));
        converter.compute_contours();
        let contour = converter.contour(0);
        assert_eq!(contour.len(), 4);
        // C++ uses the mathematical cross-product sign even though Rive's
        // screen-space Y axis points down, so this emitted order is false.
        assert!(!contour.is_clockwise());
        assert_eq!(contour.point_reversed(0), contour.point(3));
        assert_eq!(contour.points().count(), 4);
    }

    fn assert_contour(contour: RuntimeRectangleContour<'_>, expected: &[(f32, f32)]) {
        assert_eq!(contour.len(), expected.len());
        for (index, expected) in expected.iter().copied().enumerate() {
            assert_eq!(contour.point(index), Vec2D::new(expected.0, expected.1));
        }
    }
}
