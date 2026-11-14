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
        let Ok(measure) =
            Self::from_path_admitted(path, tolerance, |_| Ok::<(), std::convert::Infallible>(()));
        measure
    }
    /// [`Self::from_path`], after `admit` accepts the bytes the measure will
    /// hold once each next contour is in it ([`Self::heap_bytes`]), counted
    /// before that contour's tables and the contour list grow. The first
    /// refusal stops the measure and is returned.
    pub fn from_path_admitted<E>(
        path: &RawPath,
        tolerance: f32,
        mut admit: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Self, E> {
        let mut result = Self::new();
        let mut iterator = ContourMeasureIter::new(path, tolerance);
        // The contours' bytes so far, kept as they are added.
        let mut held = 0usize;
        loop {
            let list = Self::list_bytes(Self::grown_list_capacity(
                result.contours.len(),
                result.contours.capacity(),
            ));
            let next = iterator.next_admitted(|contour| {
                admit(held.saturating_add(list).saturating_add(contour))
            })?;
            let Some(contour) = next else {
                return Ok(result);
            };
            held = held.saturating_add(contour.heap_bytes());
            result.length += contour.length();
            result.contours.push(contour);
        }
    }
    /// Bytes the measure holds: its contour list (its capacity) and each
    /// contour's tables.
    pub fn heap_bytes(&self) -> usize {
        Self::list_bytes(self.contours.capacity()).saturating_add(self.contour_bytes())
    }
    fn contour_bytes(&self) -> usize {
        self.contours.iter().fold(0, |bytes, contour| {
            bytes.saturating_add(contour.heap_bytes())
        })
    }
    fn list_bytes(capacity: usize) -> usize {
        std::mem::size_of::<Rc<ContourMeasure>>().saturating_mul(capacity)
    }
    /// The list's capacity after one more push, as `Vec` grows.
    fn grown_list_capacity(len: usize, capacity: usize) -> usize {
        if len < capacity {
            capacity
        } else {
            capacity.saturating_mul(2).max(4)
        }
    }
    /// The most verbs and points [`Self::get_segment`] appends for this
    /// interval, without appending ([`ContourMeasure::segment_room`] for
    /// each contour it reaches).
    pub fn segment_room(&self, start_distance: f32, end_distance: f32) -> (usize, usize) {
        if self.contours.is_empty() {
            return (0, 0);
        }
        let start_distance = cpp_max(0.0, cpp_min(start_distance, self.length));
        let end_distance = cpp_max(0.0, cpp_min(end_distance, self.length));
        if start_distance >= end_distance {
            return (0, 0);
        }
        let (mut verbs, mut points) = (0usize, 0usize);
        let mut current = 0.0;
        for contour in &self.contours {
            let contour_length = contour.length();
            let contour_start = current;
            let contour_end = current + contour_length;
            if contour_end > start_distance && contour_start < end_distance {
                let local_start = cpp_max(0.0, start_distance - contour_start);
                let local_end = cpp_min(contour_length, end_distance - contour_start);
                let (contour_verbs, contour_points) = contour.segment_room(local_start, local_end);
                verbs = verbs.saturating_add(contour_verbs);
                points = points.saturating_add(contour_points);
            }
            current += contour_length;
            if current >= end_distance {
                break;
            }
        }
        (verbs, points)
    }
    pub fn at_distance(&self, distance: f32) -> PosTanDistance {
        let mut current = distance;
        for (index, contour) in self.contours.iter().enumerate() {
            let length = contour.length();
            // The last contour clamps a distance rounded beyond the end.
            if current - length <= 0.0 || index + 1 == self.contours.len() {
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

#[cfg(test)]
mod size_tests {
    use super::*;
    use crate::mechanical_port::source::math::contour_measure::RefCntContourMeasureIter;

    /// Two contours: a closed square and an open cubic.
    fn two_contours() -> RawPath {
        let mut path = RawPath::default();
        path.move_to(0.0, 0.0);
        path.line_to(100.0, 0.0);
        path.line_to(100.0, 100.0);
        path.line_to(0.0, 100.0);
        path.close();
        path.move_to(200.0, 0.0);
        path.cubic_to(300.0, 0.0, 300.0, 100.0, 200.0, 100.0);
        path
    }

    #[test]
    fn the_last_admitted_size_is_what_the_measure_holds() {
        let path = two_contours();
        let mut asked = Vec::new();
        let measure = PathMeasure::from_path_admitted(&path, 0.5, |bytes| {
            asked.push(bytes);
            Ok::<(), ()>(())
        })
        .unwrap();
        assert_eq!(asked.len(), 2, "one admission per contour");
        assert!(asked[0] < asked[1]);
        assert_eq!(asked.last().copied(), Some(measure.heap_bytes()));
        let plain = PathMeasure::from_path(&path, 0.5);
        assert_eq!(plain.length().to_bits(), measure.length().to_bits());
    }

    #[test]
    fn a_refused_contour_stops_the_measure_before_it_is_built() {
        let path = two_contours();
        let mut calls = 0;
        let refused = PathMeasure::from_path_admitted(&path, 0.5, |bytes| {
            calls += 1;
            if calls == 2 { Err(bytes) } else { Ok(()) }
        });
        assert!(matches!(refused, Err(bytes) if bytes > 0));
        assert_eq!(calls, 2);
    }

    #[test]
    fn the_contour_iterator_admits_each_measure_at_its_size() {
        let path = two_contours();
        let mut iterator = ContourMeasureIter::new(&path, 0.5);
        let mut asked = 0;
        let first = iterator
            .next_admitted(|bytes| {
                asked = bytes;
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(asked, first.heap_bytes());
        assert!(matches!(iterator.next_admitted(|_| Err(())), Err(())));
        // A refusal leaves the iterator on the same contour.
        let second = iterator.next().unwrap();
        assert!(!second.is_closed());
        assert!(iterator.next().is_none());
        assert_eq!(
            ContourMeasureIter::copy_bytes(&path),
            RefCntContourMeasureIter::new(&path, 0.5).heap_bytes()
        );
    }

    #[test]
    fn segment_room_bounds_every_extract() {
        let path = two_contours();
        let measure = PathMeasure::from_path(&path, 0.5);
        let length = measure.length();
        let steps = 37;
        for i in 0..=steps {
            for j in 0..=steps {
                let start = length * (i as f32 / steps as f32) - 10.0;
                let end = length * (j as f32 / steps as f32) + 5.0;
                for start_with_move in [true, false] {
                    let (verbs, points) = measure.segment_room(start, end);
                    let mut dst = RawPath::default();
                    measure.get_segment(start, end, Some(&mut dst), start_with_move);
                    assert!(
                        dst.verbs().len() <= verbs && dst.points().len() <= points,
                        "[{start}, {end}]: appended {}/{}, room {verbs}/{points}",
                        dst.verbs().len(),
                        dst.points().len()
                    );
                }
            }
        }
        assert_eq!(measure.segment_room(50.0, 50.0), (0, 0));
        assert_eq!(measure.segment_room(f32::NAN, -1.0), (0, 0));
    }

    #[test]
    fn segment_room_follows_the_interval_not_the_whole_source() {
        // A circle-like contour of many cubic segments.
        let mut path = RawPath::default();
        path.move_to(0.0, 0.0);
        for i in 0..200 {
            let x = (i + 1) as f32 * 10.0;
            path.cubic_to(x - 7.0, 30.0, x - 3.0, -30.0, x, 0.0);
        }
        let measure = PathMeasure::from_path(&path, 0.5);
        let (whole_verbs, _) = measure.segment_room(0.0, measure.length());
        let (short_verbs, short_points) = measure.segment_room(0.0, 1.0);
        assert!(
            short_verbs < 8 && short_points < 24,
            "{short_verbs}/{short_points}"
        );
        assert!(whole_verbs > 200);
    }
}
