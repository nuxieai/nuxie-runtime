//! Selection geometry ported from `src/text/text_selection_path.cpp`.

use crate::RuntimePathCommand;
use crate::rectangles_to_contour::{RuntimeRectangleContour, RuntimeRectanglesToContour};
use nuxie_render_api::{Aabb, Vec2D};

#[derive(Debug, Default)]
pub(crate) struct TextSelectionPath {
    converter: RuntimeRectanglesToContour,
    commands: Vec<RuntimePathCommand>,
}

impl Clone for TextSelectionPath {
    fn clone(&self) -> Self {
        // Generated occurrences rebuild the contour scratch owner cold, while
        // preserving the last CPU path until the next selection update.
        Self {
            converter: RuntimeRectanglesToContour::default(),
            commands: self.commands.clone(),
        }
    }
}

impl TextSelectionPath {
    pub(crate) fn update(&mut self, rects: &[Aabb], corner_radius: f32) {
        self.converter.reset();
        self.commands.clear();
        for rect in rects.iter().copied() {
            self.converter.add_rect(rect);
        }
        self.converter.compute_contours();

        for index in 0..self.converter.contour_count() {
            append_rounded_path(
                &mut self.commands,
                self.converter.contour(index),
                corner_radius,
            );
        }
    }

    pub(crate) fn commands(&self) -> &[RuntimePathCommand] {
        &self.commands
    }
}

#[cfg(test)]
fn update(rects: &[Aabb], corner_radius: f32) -> Vec<RuntimePathCommand> {
    let mut selection_path = TextSelectionPath::default();
    selection_path.update(rects, corner_radius);
    selection_path.commands
}

fn append_rounded_path(
    commands: &mut Vec<RuntimePathCommand>,
    contour: RuntimeRectangleContour<'_>,
    radius: f32,
) {
    let length = contour.len();
    if length < 2 {
        return;
    }
    let reversed = contour.is_clockwise();
    let point = |index: usize| {
        if reversed {
            contour.point_reversed(index)
        } else {
            contour.point(index)
        }
    };
    if radius > 0.0 {
        let (translation, out_point, in_point, pos_next) =
            rounded_corner(point(length - 1), point(0), point(1), radius);
        commands.push(RuntimePathCommand::Move {
            x: translation.x,
            y: translation.y,
        });
        commands.push(RuntimePathCommand::Cubic {
            x1: out_point.x,
            y1: out_point.y,
            x2: in_point.x,
            y2: in_point.y,
            x3: pos_next.x,
            y3: pos_next.y,
        });
    } else {
        let first = point(0);
        commands.push(RuntimePathCommand::Move {
            x: first.x,
            y: first.y,
        });
    }

    for index in 1..length {
        let current = point(index);
        if radius > 0.0 {
            let (translation, out_point, in_point, pos_next) = rounded_corner(
                point(index - 1),
                current,
                point((index + 1) % length),
                radius,
            );
            commands.push(RuntimePathCommand::Line {
                x: translation.x,
                y: translation.y,
            });
            commands.push(RuntimePathCommand::Cubic {
                x1: out_point.x,
                y1: out_point.y,
                x2: in_point.x,
                y2: in_point.y,
                x3: pos_next.x,
                y3: pos_next.y,
            });
        } else {
            commands.push(RuntimePathCommand::Line {
                x: current.x,
                y: current.y,
            });
        }
    }
    commands.push(RuntimePathCommand::Close);
}

fn rounded_corner(
    prev: Vec2D,
    pos: Vec2D,
    next: Vec2D,
    radius: f32,
) -> (Vec2D, Vec2D, Vec2D, Vec2D) {
    let (to_prev, to_prev_length) = normalized(prev.x - pos.x, prev.y - pos.y);
    let (to_next, to_next_length) = normalized(next.x - pos.x, next.y - pos.y);
    let render_radius = (to_prev_length * 0.5).min(to_next_length * 0.5).min(radius);
    let ideal_distance = ideal_control_point_distance(to_prev, to_next, render_radius);
    (
        scale_and_add(pos, to_prev, render_radius),
        scale_and_add(pos, to_prev, render_radius - ideal_distance),
        scale_and_add(pos, to_next, render_radius - ideal_distance),
        scale_and_add(pos, to_next, render_radius),
    )
}

fn normalized(x: f32, y: f32) -> (Vec2D, f32) {
    let length = x.mul_add(x, y * y).sqrt();
    (Vec2D::new(x / length, y / length), length)
}

fn scale_and_add(point: Vec2D, vector: Vec2D, scale: f32) -> Vec2D {
    Vec2D::new(
        vector.x.mul_add(scale, point.x),
        vector.y.mul_add(scale, point.y),
    )
}

fn ideal_control_point_distance(to_prev: Vec2D, to_next: Vec2D, radius: f32) -> f32 {
    let cross = to_prev.x.mul_add(to_next.y, -(to_prev.y * to_next.x));
    let dot = to_prev.x.mul_add(to_next.x, to_prev.y * to_next.y);
    let angle = cross.atan2(dot).abs();
    radius.min(
        (4.0 / 3.0)
            * (std::f32::consts::PI / (2.0 * ((2.0 * std::f32::consts::PI) / angle))).tan()
            * radius
            * if angle < std::f32::consts::FRAC_PI_2 {
                1.0 + angle.cos()
            } else {
                2.0 - angle.sin()
            },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_empty_square_and_rounded_selection_paths_are_ported() {
        assert!(update(&[], 5.0).is_empty());
        let rect = Aabb::new(0.0, 0.0, 10.0, 8.0);
        assert_eq!(update(&[rect], 0.0).len(), 5);
        let rounded = update(&[rect], 100.0);
        assert_eq!(rounded.len(), 9);
        assert!(matches!(rounded.last(), Some(RuntimePathCommand::Close)));
    }

    #[test]
    fn adjacent_selection_rectangles_are_unioned_before_rounding() {
        let commands = update(
            &[
                Aabb::new(0.0, 0.0, 10.0, 8.0),
                Aabb::new(10.0, 0.0, 20.0, 8.0),
            ],
            0.0,
        );
        assert_eq!(commands.len(), 5);
    }
}
