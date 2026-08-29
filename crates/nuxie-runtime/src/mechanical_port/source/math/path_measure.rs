use std::rc::Rc;

use super::contour_measure::{ContourMeasure, ContourMeasureIter, PosTanDistance};
use super::raw_path::RawPath;

#[derive(Clone, Debug, Default)]
pub struct PathMeasure {
    length: f32,
    contours: Vec<Rc<ContourMeasure>>,
}

impl PathMeasure {
    pub fn from_path_default(path: &RawPath) -> Self {
        Self::from_path(path, ContourMeasureIter::DEFAULT_TOLERANCE)
    }

    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_path(path: &RawPath, tolerance: f32) -> Self {
        let mut result = Self::new();
        let mut iterator = ContourMeasureIter::new(path, tolerance);
        while let Some(contour) = iterator.next() {
            result.length += contour.length();
            result.contours.push(contour);
        }
        result
    }
    pub fn at_distance(&self, distance: f32) -> PosTanDistance {
        let mut current = distance;
        for contour in &self.contours {
            let length = contour.length();
            if current - length <= 0.0 {
                return PosTanDistance::new(contour.get_pos_tan(current), distance);
            }
            current -= length;
        }
        PosTanDistance::default()
    }
    pub fn at_percentage(&self, percentage_distance: f32) -> PosTanDistance {
        let mut in_range = percentage_distance % 1.0;
        if in_range < 0.0 {
            in_range += 1.0;
        }
        if percentage_distance != 0.0 && in_range == 0.0 {
            in_range = 1.0;
        }
        self.at_distance(self.length * in_range)
    }
    pub fn get_segment(
        &self,
        mut start_distance: f32,
        mut end_distance: f32,
        dst: Option<&mut RawPath>,
        start_with_move: bool,
    ) {
        let Some(dst) = dst else {
            return;
        };
        if self.contours.is_empty() {
            return;
        }
        start_distance = cpp_max(0.0, cpp_min(start_distance, self.length));
        end_distance = cpp_max(0.0, cpp_min(end_distance, self.length));
        if start_distance >= end_distance {
            return;
        }
        let mut current = 0.0;
        let mut first = true;
        for contour in &self.contours {
            let contour_length = contour.length();
            let contour_start = current;
            let contour_end = current + contour_length;
            if contour_end > start_distance && contour_start < end_distance {
                let local_start = cpp_max(0.0, start_distance - contour_start);
                let local_end = cpp_min(contour_length, end_distance - contour_start);
                contour.get_segment(local_start, local_end, dst, !first || start_with_move);
                first = false;
            }
            current += contour_length;
            if current >= end_distance {
                break;
            }
        }
    }
    pub fn length(&self) -> f32 {
        self.length
    }
    pub fn is_closed(&self) -> bool {
        self.contours.len() == 1 && self.contours[0].is_closed()
    }
}
fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}
fn cpp_max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}
