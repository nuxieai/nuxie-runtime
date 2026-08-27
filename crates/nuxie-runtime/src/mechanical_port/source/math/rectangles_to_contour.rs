use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
#[cfg(any(test, feature = "tools"))]
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};

use super::aabb::Aabb;
use super::vec2d::Vec2D;

#[derive(Clone, Copy, Debug)]
struct PointKey(Vec2D);
impl PartialEq for PointKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for PointKey {}
impl Hash for PointKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let x = if self.0.x == 0.0 {
            0
        } else {
            self.0.x.to_bits()
        } as usize;
        let y = if self.0.y == 0.0 {
            0
        } else {
            self.0.y.to_bits()
        } as usize;
        (x ^ y.wrapping_shl(1)).hash(state);
    }
}
impl PartialOrd for PointKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PointKey {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_points(self.0, other.0, 0, 1)
    }
}

#[cfg(any(test, feature = "tools"))]
type EdgeMap = BTreeMap<PointKey, Vec2D>;
#[cfg(not(any(test, feature = "tools")))]
type EdgeMap = HashMap<PointKey, Vec2D>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContourPoint {
    pub vector: Vec2D,
    pub direction: i32,
}
impl ContourPoint {
    pub fn new(vector: Vec2D, direction: i32) -> Self {
        Self { vector, direction }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Contour<'a> {
    points: &'a [ContourPoint],
}
impl<'a> Contour<'a> {
    pub fn new(points: &'a [ContourPoint]) -> Self {
        Self { points }
    }
    pub fn size(self) -> usize {
        self.points.len()
    }
    pub fn point(self, index: usize) -> Vec2D {
        self.points[index].vector
    }
    pub fn point_reversed(self, index: usize, reversed: bool) -> Vec2D {
        if reversed {
            self.points[self.points.len() - 1 - index].vector
        } else {
            self.points[index].vector
        }
    }
    pub fn iter(self) -> ContourPointIter<'a> {
        ContourPointIter {
            contour: self.points,
            point_index: 0,
        }
    }
    pub fn is_clockwise(self) -> bool {
        let size = self.points.len();
        if size < 1 {
            return true;
        }
        let mut area = 0.0;
        for index in 1..size {
            area += Vec2D::cross(self.points[index - 1].vector, self.points[index].vector);
        }
        area += Vec2D::cross(self.points[size - 1].vector, self.points[0].vector);
        area * 0.5 >= 0.0
    }
}
pub struct ContourPointIter<'a> {
    contour: &'a [ContourPoint],
    point_index: usize,
}
impl<'a> Iterator for ContourPointIter<'a> {
    type Item = Vec2D;
    fn next(&mut self) -> Option<Vec2D> {
        let current = self.contour.get(self.point_index)?.vector;
        self.point_index += 1;
        while self.point_index < self.contour.len()
            && self.contour[self.point_index].vector == current
        {
            self.point_index += 1;
        }
        Some(current)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RectEvent {
    pub index: usize,
    pub size: f32,
    pub event_type: u8,
    pub x: f32,
    pub y: f32,
}
impl RectEvent {
    pub fn get_value(self, axis: u8) -> f32 {
        if axis == 0 { self.x } else { self.y }
    }
}

#[derive(Debug, Default)]
pub struct RectanglesToContour {
    rect_events: Vec<RectEvent>,
    edges_horizontal: EdgeMap,
    edges_vertical: EdgeMap,
    unique_points: HashSet<PointKey>,
    rectangles: Vec<Aabb>,
    subdivided_rectangles: Vec<Aabb>,
    rectangle_inclusion_bits: Vec<u8>,
    sorted_points_x: Vec<Vec2D>,
    sorted_points_y: Vec<Vec2D>,
    contour_points: Vec<ContourPoint>,
    contour_offsets: Vec<usize>,
}

impl RectanglesToContour {
    pub fn add_rect(&mut self, rect: Aabb) {
        if let Some(last) = self.rectangles.last().copied() {
            if last.min_y == rect.min_y && last.max_y == rect.max_y && last.max_x == rect.min_x {
                self.rectangles.pop();
                self.rectangles
                    .push(Aabb::new(last.min_x, last.min_y, rect.max_x, last.max_y));
                return;
            }
        }
        self.rectangles.push(rect);
    }
    pub fn reset(&mut self) {
        self.rectangles.clear();
    }
    pub fn contour_count(&self) -> usize {
        self.contour_offsets.len()
    }
    pub fn contour(&self, index: usize) -> Contour<'_> {
        assert!(index < self.contour_offsets.len());
        let end = self.contour_offsets[index];
        let start = if index == 0 {
            0
        } else {
            self.contour_offsets[index - 1]
        };
        Contour::new(&self.contour_points[start..end])
    }
    pub fn contours(&self) -> impl Iterator<Item = Contour<'_>> {
        (0..self.contour_count()).map(|index| self.contour(index))
    }
    fn is_rect_included(&self, index: usize) -> bool {
        self.rectangle_inclusion_bits[index / 8] & (1 << (index % 8)) != 0
    }
    fn mark_rect_included(&mut self, index: usize, included: bool) {
        if included {
            self.rectangle_inclusion_bits[index / 8] |= 1 << (index % 8);
        } else {
            self.rectangle_inclusion_bits[index / 8] &= !(1 << (index % 8));
        }
    }
    fn subdivide_rectangles(&mut self) {
        self.subdivided_rectangles.clear();
        self.unique_points.clear();
        self.rect_events.clear();
        if self.rectangles.is_empty() {
            return;
        }
        let vertical = sort_rect_events(&self.rectangles, &mut self.rect_events, 0, 1);
        let horizontal = sort_rect_events(&self.rectangles, &mut self.rect_events, 1, 0);
        let vertical_events = self.rect_events[vertical.clone()].to_vec();
        let horizontal_events = self.rect_events[horizontal].to_vec();
        self.rectangle_inclusion_bits
            .resize(self.rectangles.len() / 8 + 1, 0);
        self.rectangle_inclusion_bits.fill(0);
        self.mark_rect_included(vertical_events[0].index, true);
        let mut opened = 0;
        let mut begin_y = 0.0;
        for index in 0..vertical_events.len() - 1 {
            let event = vertical_events[index];
            self.mark_rect_included(event.index, event.event_type == 0);
            let next = vertical_events[index + 1];
            let begin_x = event.x;
            let end_x = next.x;
            if end_x - begin_x == 0.0 {
                continue;
            }
            for horizontal_index in 0..horizontal_events.len() {
                let horizontal = horizontal_events[horizontal_index];
                if self.is_rect_included(horizontal.index) {
                    if horizontal.event_type == 0 {
                        opened += 1;
                        if opened == 1 {
                            begin_y = horizontal.y;
                        }
                    } else {
                        opened -= 1;
                        let mut distance = 1;
                        while horizontal_index + distance < horizontal_events.len()
                            && !self.is_rect_included(
                                horizontal_events[horizontal_index + distance].index,
                            )
                        {
                            distance += 1;
                        }
                        let next = horizontal_events.get(horizontal_index + distance);
                        if next.is_none() || (opened == 0 && next.unwrap().y != horizontal.y) {
                            self.subdivided_rectangles.push(Aabb::new(
                                begin_x,
                                begin_y,
                                end_x,
                                horizontal.y,
                            ));
                        }
                    }
                }
            }
        }
    }
    fn add_unique_point(&mut self, point: Vec2D) {
        let key = PointKey(point);
        if !self.unique_points.insert(key) {
            self.unique_points.remove(&key);
        }
    }
    pub fn compute_contours(&mut self) {
        self.subdivide_rectangles();
        let rectangles = self.subdivided_rectangles.clone();
        for rect in rectangles {
            self.add_unique_point(Vec2D::new(rect.min_x, rect.min_y));
            self.add_unique_point(Vec2D::new(rect.max_x, rect.min_y));
            self.add_unique_point(Vec2D::new(rect.max_x, rect.max_y));
            self.add_unique_point(Vec2D::new(rect.min_x, rect.max_y));
        }
        self.sorted_points_x.clear();
        self.sorted_points_y.clear();
        for point in &self.unique_points {
            self.sorted_points_x.push(point.0);
            self.sorted_points_y.push(point.0);
        }
        self.sorted_points_x
            .sort_unstable_by(|a, b| compare_points(*a, *b, 0, 1));
        self.sorted_points_y
            .sort_unstable_by(|a, b| compare_points(*a, *b, 1, 0));
        self.edges_horizontal.clear();
        self.edges_vertical.clear();
        let mut index = 0;
        while index < self.sorted_points_y.len() {
            let current_y = self.sorted_points_y[index].y;
            while index < self.sorted_points_y.len() && self.sorted_points_y[index].y == current_y {
                let a = self.sorted_points_y[index];
                let b = self.sorted_points_y[index + 1];
                self.edges_horizontal.insert(PointKey(a), b);
                self.edges_horizontal.insert(PointKey(b), a);
                index += 2;
            }
        }
        index = 0;
        while index < self.sorted_points_x.len() {
            let current_x = self.sorted_points_x[index].x;
            while index < self.sorted_points_x.len() && self.sorted_points_x[index].x == current_x {
                let a = self.sorted_points_x[index];
                let b = self.sorted_points_x[index + 1];
                self.edges_vertical.insert(PointKey(a), b);
                self.edges_vertical.insert(PointKey(b), a);
                index += 2;
            }
        }
        self.contour_points.clear();
        self.contour_offsets.clear();
        extract_polygons(
            &mut self.contour_points,
            &mut self.contour_offsets,
            &mut self.edges_horizontal,
            &mut self.edges_vertical,
        );
    }
}

fn compare_points(a: Vec2D, b: Vec2D, axis_a: usize, axis_b: usize) -> Ordering {
    let less = a[axis_a] < b[axis_a] || (a[axis_a] == b[axis_a] && a[axis_b] < b[axis_b]);
    let greater = b[axis_a] < a[axis_a] || (b[axis_a] == a[axis_a] && b[axis_b] < a[axis_b]);
    if less {
        Ordering::Less
    } else if greater {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

fn sort_rect_events(
    rectangles: &[Aabb],
    output: &mut Vec<RectEvent>,
    axis_a: u8,
    axis_b: u8,
) -> core::ops::Range<usize> {
    let start = output.len();
    for (index, rect) in rectangles.iter().enumerate() {
        for point_index in 0..2 {
            let point = rect.corner(point_index);
            let event = RectEvent {
                event_type: point_index as u8,
                index,
                y: if axis_b == 0 {
                    point[axis_a as usize]
                } else {
                    point[axis_b as usize]
                },
                x: if axis_b == 0 {
                    point[axis_b as usize]
                } else {
                    point[axis_a as usize]
                },
                size: if point_index == 0 {
                    rect.corner(1)[axis_b as usize] - point[axis_b as usize]
                } else {
                    point[axis_b as usize] - rect.corner(0)[axis_b as usize]
                },
            };
            output.push(event);
        }
    }
    output[start..].sort_unstable_by(|a, b| {
        let a = a.get_value(axis_b);
        let b = b.get_value(axis_b);
        if a < b {
            Ordering::Less
        } else if b < a {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    output[start..].sort_unstable_by(|a, b| {
        let a = a.get_value(axis_a);
        let b = b.get_value(axis_a);
        if a < b {
            Ordering::Less
        } else if b < a {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    start..output.len()
}
fn first_horizontal_key(map: &EdgeMap) -> Option<PointKey> {
    #[cfg(any(test, feature = "tools"))]
    {
        map.keys().next().copied()
    }
    #[cfg(not(any(test, feature = "tools")))]
    {
        map.keys().next().copied()
    }
}
fn extract_polygons(
    points: &mut Vec<ContourPoint>,
    offsets: &mut Vec<usize>,
    horizontal: &mut EdgeMap,
    vertical: &mut EdgeMap,
) {
    while let Some(start) = first_horizontal_key(horizontal) {
        let contour_start = points.len();
        horizontal.remove(&start);
        let first = ContourPoint::new(start.0, 0);
        points.push(first);
        loop {
            let current = *points.last().unwrap();
            let map = if current.direction == 0 {
                &mut *vertical
            } else {
                &mut *horizontal
            };
            let Some(next) = map.remove(&PointKey(current.vector)) else {
                break;
            };
            points.push(ContourPoint::new(
                next,
                if current.direction == 0 { 1 } else { 0 },
            ));
            if points.last() == Some(&first) {
                points.pop();
                break;
            }
        }
        offsets.push(points.len());
        for point in &points[contour_start..] {
            horizontal.remove(&PointKey(point.vector));
            vertical.remove(&PointKey(point.vector));
        }
    }
}
