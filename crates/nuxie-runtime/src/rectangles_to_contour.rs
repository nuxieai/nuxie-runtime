//! Rectangle-union contour construction.
//!
//! Literal Rust port of pinned C++ `src/math/rectangles_to_contour.cpp`.
//! The retained scratch vectors correspond one-for-one with the C++ owner so
//! repeated computations reuse allocation. A small ordered vector map stands
//! in for C++'s TESTING `std::map`; rectangle contours are small and this keeps
//! cross-platform contour order deterministic without introducing a second
//! geometry subsystem.

use std::cmp::Ordering;
#[cfg(not(test))]
use std::collections::HashMap;
use std::ops::Range;

use nuxie_render_api::{Aabb, Vec2D};

#[derive(Debug, Clone, Copy)]
struct RectEvent {
    index: usize,
    // Public on the pinned C++ helper and populated by `sortRectEvents`, even
    // though this pinned algorithm does not subsequently consume it.
    #[allow(dead_code)]
    size: f32,
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

/// Mirrors the header's `EdgeMap` switch: unit-test builds use an ordered map
/// for cross-platform fixture stability, while production uses a hash map.
#[derive(Debug, Default)]
struct EdgeMap {
    #[cfg(test)]
    entries: Vec<(Vec2D, Vec2D)>,
    #[cfg(not(test))]
    entries: HashMap<Vec2DKey, (Vec2D, Vec2D)>,
}

#[cfg(not(test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Vec2DKey {
    x: u32,
    y: u32,
}

#[cfg(not(test))]
impl From<Vec2D> for Vec2DKey {
    fn from(point: Vec2D) -> Self {
        // C++ `std::hash<float>` must hash equal signed zeroes alike.
        let bits = |value: f32| if value == 0.0 { 0 } else { value.to_bits() };
        Self {
            x: bits(point.x),
            y: bits(point.y),
        }
    }
}

impl EdgeMap {
    fn clear(&mut self) {
        self.entries.clear();
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn first_key(&self) -> Option<Vec2D> {
        #[cfg(test)]
        {
            self.entries.first().map(|edge| edge.0)
        }
        #[cfg(not(test))]
        {
            self.entries.values().next().map(|edge| edge.0)
        }
    }

    fn insert(&mut self, key: Vec2D, value: Vec2D) {
        #[cfg(test)]
        {
            match self
                .entries
                .binary_search_by(|edge| compare_edge_keys(&edge.0, &key))
            {
                Ok(index) => self.entries[index].1 = value,
                Err(index) => self.entries.insert(index, (key, value)),
            }
        }
        #[cfg(not(test))]
        {
            match self.entries.entry(key.into()) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    // `unordered_map::operator[]` preserves the originally
                    // inserted key object when an equal key is assigned.
                    entry.get_mut().1 = value;
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert((key, value));
                }
            }
        }
    }

    fn take(&mut self, key: Vec2D) -> Option<Vec2D> {
        #[cfg(test)]
        {
            let index = self
                .entries
                .binary_search_by(|edge| compare_edge_keys(&edge.0, &key))
                .ok()?;
            Some(self.entries.remove(index).1)
        }
        #[cfg(not(test))]
        {
            self.entries.remove(&key.into()).map(|edge| edge.1)
        }
    }

    fn remove(&mut self, key: Vec2D) {
        #[cfg(test)]
        {
            if let Ok(index) = self
                .entries
                .binary_search_by(|edge| compare_edge_keys(&edge.0, &key))
            {
                self.entries.remove(index);
            }
        }
        #[cfg(not(test))]
        {
            self.entries.remove(&key.into());
        }
    }
}

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
                self.edges_h.insert(first, second);
                self.edges_h.insert(second, first);
                index += 2;
            }
        }

        index = 0;
        while index < self.sorted_points_x.len() {
            let current_x = self.sorted_points_x[index].x;
            while index < self.sorted_points_x.len() && self.sorted_points_x[index].x == current_x {
                let first = self.sorted_points_x[index];
                let second = self.sorted_points_x[index + 1];
                self.edges_v.insert(first, second);
                self.edges_v.insert(second, first);
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

    /// Mirrors the header-owned `ContourItr` range surface. Keeping the range
    /// on the nominal owner lets production callers traverse the packed
    /// contour storage without rebuilding its offset arithmetic themselves.
    pub(crate) fn contours(&self) -> impl Iterator<Item = RuntimeRectangleContour<'_>> + '_ {
        (0..self.contour_count()).map(|index| self.contour(index))
    }

    fn subdivide_rectangles(&mut self) {
        self.subdivided_rects.clear();
        self.unique_points.clear();
        self.rect_events.clear();
        if self.rects.is_empty() {
            return;
        }

        let vertical_range = sort_rect_events(&self.rects, &mut self.rect_events, 0, 1);
        let horizontal_range = sort_rect_events(&self.rects, &mut self.rect_events, 1, 0);
        let vertical = &self.rect_events[vertical_range];
        let horizontal = &self.rect_events[horizontal_range];

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

fn sort_rect_events(
    rects: &[Aabb],
    result: &mut Vec<RectEvent>,
    axis_a: u8,
    axis_b: u8,
) -> Range<usize> {
    let result_start = result.len();
    result.reserve(rects.len() * 2);
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
                size: if point_index == 0 {
                    let opposite = if axis_b == 0 { rect.max_x } else { rect.max_y };
                    opposite - coordinate(axis_b)
                } else {
                    let opposite = if axis_b == 0 { rect.min_x } else { rect.min_y };
                    coordinate(axis_b) - opposite
                },
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
    result[result_start..]
        .sort_unstable_by(|left, right| compare_float(left.value(axis_b), right.value(axis_b)));
    result[result_start..]
        .sort_unstable_by(|left, right| compare_float(left.value(axis_a), right.value(axis_a)));
    result_start..result.len()
}

fn extract_polygons(
    contour_points: &mut Vec<ContourPoint>,
    contour_offsets: &mut Vec<usize>,
    edges_h: &mut EdgeMap,
    edges_v: &mut EdgeMap,
) {
    while !edges_h.is_empty() {
        let start = edges_h
            .first_key()
            .expect("non-empty EdgeMap must yield one entry");
        let contour_start = contour_points.len();
        edges_h.remove(start);
        let first = ContourPoint {
            point: start,
            direction: 0,
        };
        contour_points.push(first);

        loop {
            let current = contour_points[contour_points.len() - 1];
            let edge = if current.direction == 0 {
                edges_v.take(current.point).map(|point| ContourPoint {
                    point,
                    direction: 1,
                })
            } else {
                edges_h.take(current.point).map(|point| ContourPoint {
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
            edges_h.remove(point);
            edges_v.remove(point);
        }
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
    compare_lexicographic(left.x, right.x, left.y, right.y)
}

fn compare_y_then_x(left: &Vec2D, right: &Vec2D) -> Ordering {
    compare_lexicographic(left.y, right.y, left.x, right.x)
}

#[cfg(test)]
fn compare_edge_keys(left: &Vec2D, right: &Vec2D) -> Ordering {
    compare_lexicographic(left.x, right.x, left.y, right.y)
}

fn compare_lexicographic(
    left_primary: f32,
    right_primary: f32,
    left_secondary: f32,
    right_secondary: f32,
) -> Ordering {
    if left_primary < right_primary {
        Ordering::Less
    } else if right_primary < left_primary {
        Ordering::Greater
    } else if left_primary == right_primary {
        compare_float(left_secondary, right_secondary)
    } else {
        // The source comparator returns false in both directions for NaN.
        Ordering::Equal
    }
}

fn compare_float(left: f32, right: f32) -> Ordering {
    if left < right {
        Ordering::Less
    } else if right < left {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

fn cross(left: Vec2D, right: Vec2D) -> f32 {
    left.x.mul_add(right.y, -(left.y * right.x))
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
        assert_eq!(converter.contour_count(), 1);
        assert_eq!(converter.contours().count(), 1);
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
        assert_eq!(converter.contours().count(), 2);
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

    #[test]
    fn contour_winding_preserves_pinned_vec2d_cross_cancellation_sign() {
        let a = Vec2D::new(f32::from_bits(0x26cd_29b3), f32::from_bits(0xd01a_d4bb));
        let b = Vec2D::new(f32::from_bits(0x2533_fdc2), f32::from_bits(0xce87_d5a9));
        assert_eq!(cross(a, b).to_bits(), 0xa7ee_c560);

        let points = [
            ContourPoint {
                point: Vec2D::new(0.0, 0.0),
                direction: 0,
            },
            ContourPoint {
                point: a,
                direction: 0,
            },
            ContourPoint {
                point: b,
                direction: 0,
            },
        ];
        assert!(!RuntimeRectangleContour { points: &points }.is_clockwise());
    }

    #[test]
    fn rect_event_ranges_retain_the_pinned_shared_scratch_and_axis_sizes() {
        let rects = [Aabb::new(0.0, 1.0, 4.0, 7.0)];
        let mut events = Vec::new();
        let vertical = sort_rect_events(&rects, &mut events, 0, 1);
        let horizontal = sort_rect_events(&rects, &mut events, 1, 0);
        assert_eq!(vertical, 0..2);
        assert_eq!(horizontal, 2..4);
        assert!(events[vertical].iter().all(|event| event.size == 6.0));
        assert!(events[horizontal].iter().all(|event| event.size == 4.0));
    }

    #[test]
    fn testing_edge_map_and_sort_comparators_preserve_cpp_float_relations() {
        assert_eq!(compare_float(-0.0, 0.0), Ordering::Equal);
        assert_eq!(compare_float(f32::NAN, 1.0), Ordering::Equal);
        assert_eq!(
            compare_x_then_y(&Vec2D::new(f32::NAN, 0.0), &Vec2D::new(f32::NAN, 1.0)),
            Ordering::Equal
        );

        let negative_zero = Vec2D::new(-0.0, 2.0);
        let positive_zero = Vec2D::new(0.0, 2.0);
        let mut edges = EdgeMap::default();
        edges.insert(negative_zero, Vec2D::new(1.0, 1.0));
        edges.insert(positive_zero, Vec2D::new(2.0, 2.0));
        assert_eq!(edges.first_key().unwrap().x.to_bits(), (-0.0_f32).to_bits());
        assert_eq!(edges.take(positive_zero), Some(Vec2D::new(2.0, 2.0)));
    }

    fn assert_contour(contour: RuntimeRectangleContour<'_>, expected: &[(f32, f32)]) {
        assert_eq!(contour.len(), expected.len());
        for (index, expected) in expected.iter().copied().enumerate() {
            assert_eq!(contour.point(index), Vec2D::new(expected.0, expected.1));
        }
    }
}
