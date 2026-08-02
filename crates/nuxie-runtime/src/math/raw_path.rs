// RawPath command ownership ported from pinned C++ `src/math/raw_path.cpp`.

use nuxie_render_api::{PathVerb as RenderPathVerb, RawPath, RawPathBuilder};

use crate::draw::RuntimePathCommand;

pub(crate) fn runtime_raw_path_from_commands(commands: &[RuntimePathCommand]) -> RawPath {
    let mut raw_path = RawPath::new();
    runtime_rebuild_raw_path_from_commands(&mut raw_path, commands);
    raw_path
}

pub(crate) fn runtime_rebuild_raw_path_from_commands(
    raw_path: &mut RawPath,
    commands: &[RuntimePathCommand],
) {
    let (verbs, points) = runtime_path_command_counts(commands);
    raw_path.rebuild(verbs, points, |raw_path| {
        runtime_append_commands_to_raw_path(raw_path, commands);
    });
}

fn runtime_path_command_counts(commands: &[RuntimePathCommand]) -> (usize, usize) {
    let mut points = 0;
    for command in commands {
        points += match command {
            RuntimePathCommand::Move { .. } | RuntimePathCommand::Line { .. } => 1,
            RuntimePathCommand::Cubic { .. } => 3,
            RuntimePathCommand::Close => 0,
        };
    }
    (commands.len(), points)
}

fn runtime_append_commands_to_raw_path(
    raw_path: &mut RawPathBuilder<'_>,
    commands: &[RuntimePathCommand],
) {
    for command in commands {
        match *command {
            RuntimePathCommand::Move { x, y } => raw_path.move_to(x, y),
            RuntimePathCommand::Line { x, y } => raw_path.line_to(x, y),
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => raw_path.cubic_to(x1, y1, x2, y2, x3, y3),
            RuntimePathCommand::Close => raw_path.close(),
        }
    }
}

pub fn runtime_path_commands_from_raw_path(path: &RawPath) -> Vec<RuntimePathCommand> {
    (|| {
        let mut commands = Vec::with_capacity(path.verbs().len());
        let mut point_index = 0usize;
        let mut current = None;
        for verb in path.verbs() {
            match verb {
                RenderPathVerb::Move => {
                    let point = path.points().get(point_index)?;
                    point_index += 1;
                    current = Some((point.x, point.y));
                    commands.push(RuntimePathCommand::Move {
                        x: point.x,
                        y: point.y,
                    });
                }
                RenderPathVerb::Line => {
                    let point = path.points().get(point_index)?;
                    point_index += 1;
                    current = Some((point.x, point.y));
                    commands.push(RuntimePathCommand::Line {
                        x: point.x,
                        y: point.y,
                    });
                }
                RenderPathVerb::Quad => {
                    let from = current?;
                    let control = path.points().get(point_index)?;
                    let point = path.points().get(point_index + 1)?;
                    point_index += 2;
                    let x1 = from.0 + (control.x - from.0) * (2.0 / 3.0);
                    let y1 = from.1 + (control.y - from.1) * (2.0 / 3.0);
                    let x2 = point.x + (control.x - point.x) * (2.0 / 3.0);
                    let y2 = point.y + (control.y - point.y) * (2.0 / 3.0);
                    current = Some((point.x, point.y));
                    commands.push(RuntimePathCommand::Cubic {
                        x1,
                        y1,
                        x2,
                        y2,
                        x3: point.x,
                        y3: point.y,
                    });
                }
                RenderPathVerb::Cubic => {
                    let p1 = path.points().get(point_index)?;
                    let p2 = path.points().get(point_index + 1)?;
                    let p3 = path.points().get(point_index + 2)?;
                    point_index += 3;
                    current = Some((p3.x, p3.y));
                    commands.push(RuntimePathCommand::Cubic {
                        x1: p1.x,
                        y1: p1.y,
                        x2: p2.x,
                        y2: p2.y,
                        x3: p3.x,
                        y3: p3.y,
                    });
                }
                RenderPathVerb::Close => commands.push(RuntimePathCommand::Close),
            }
        }
        Some(commands)
    })()
    .unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawPathVerb {
    Move,
    Line,
    Cubic,
    Close,
}

pub(crate) fn path_commands_backwards(commands: &[RuntimePathCommand]) -> Vec<RuntimePathCommand> {
    let (verbs, points) = raw_path_parts(commands);
    if verbs.is_empty() {
        return Vec::new();
    }

    let reversed_points = points.into_iter().rev().collect::<Vec<_>>();
    let mut reversed_verbs = Vec::with_capacity(verbs.len());
    reversed_verbs.push(RawPathVerb::Move);
    let mut closed = false;
    for index in (0..verbs.len()).rev() {
        let verb = verbs[index];
        if verb == RawPathVerb::Close {
            closed = true;
            continue;
        }
        if verb == RawPathVerb::Move && closed {
            reversed_verbs.push(RawPathVerb::Close);
            closed = false;
        }
        if index != 0 {
            reversed_verbs.push(verb);
        } else {
            break;
        }
    }

    let mut commands = raw_path_parts_to_commands(&reversed_verbs, &reversed_points);
    prune_empty_path_segments(&mut commands);
    commands
}

// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/src/math/raw_path.cpp RawPath::pruneEmptySegments
pub(crate) fn prune_empty_path_segments(commands: &mut Vec<RuntimePathCommand>) {
    prune_empty_path_segments_from(commands, 0);
}

pub(crate) fn prune_empty_path_segments_from(commands: &mut Vec<RuntimePathCommand>, start: usize) {
    let mut current = None::<(f32, f32)>;
    let mut write = start;
    let mut pruned = false;
    let mut multi_contour = None::<bool>;
    for read in start..commands.len() {
        let command = commands[read];
        let keep = match command {
            RuntimePathCommand::Move { x, y } => {
                current = Some((x, y));
                true
            }
            RuntimePathCommand::Line { x, y } => {
                let keep = current != Some((x, y));
                current = Some((x, y));
                keep
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                let exact_empty =
                    current == Some((x1, y1)) && (x1, y1) == (x2, y2) && (x2, y2) == (x3, y3);
                // Rust-side reverse/transform assembly can leave sub-ulp cancellation
                // noise in multi-contour paths that C++ has already collapsed.
                let near_empty = !exact_empty
                    && current.is_some_and(|current| {
                        path_points_match(current, (x1, y1))
                            && path_points_match(current, (x2, y2))
                            && path_points_match(current, (x3, y3))
                    })
                    && *multi_contour.get_or_insert_with(|| {
                        path_commands_have_multiple_contours(commands, start)
                    });
                if !exact_empty && !near_empty {
                    current = Some((x3, y3));
                    true
                } else {
                    current = Some((x3, y3));
                    false
                }
            }
            RuntimePathCommand::Close => true,
        };
        if keep {
            if pruned {
                commands[write] = command;
            }
            write += 1;
        } else {
            pruned = true;
        }
    }
    if pruned {
        commands.truncate(write);
    }
}

fn path_commands_have_multiple_contours(commands: &[RuntimePathCommand], start: usize) -> bool {
    commands
        .get(start..)
        .unwrap_or_default()
        .iter()
        .filter(|command| matches!(command, RuntimePathCommand::Move { .. }))
        .take(2)
        .count()
        >= 2
}

fn path_points_match(left: (f32, f32), right: (f32, f32)) -> bool {
    (left.0 - right.0).abs() <= f32::EPSILON && (left.1 - right.1).abs() <= f32::EPSILON
}

fn raw_path_parts(commands: &[RuntimePathCommand]) -> (Vec<RawPathVerb>, Vec<(f32, f32)>) {
    let mut verbs = Vec::with_capacity(commands.len());
    let mut points = Vec::new();
    for command in commands {
        match *command {
            RuntimePathCommand::Move { x, y } => {
                verbs.push(RawPathVerb::Move);
                points.push((x, y));
            }
            RuntimePathCommand::Line { x, y } => {
                verbs.push(RawPathVerb::Line);
                points.push((x, y));
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                verbs.push(RawPathVerb::Cubic);
                points.push((x1, y1));
                points.push((x2, y2));
                points.push((x3, y3));
            }
            RuntimePathCommand::Close => verbs.push(RawPathVerb::Close),
        }
    }
    (verbs, points)
}

fn raw_path_parts_to_commands(
    verbs: &[RawPathVerb],
    points: &[(f32, f32)],
) -> Vec<RuntimePathCommand> {
    let mut commands = Vec::with_capacity(verbs.len());
    let mut point_index = 0;
    for verb in verbs {
        match *verb {
            RawPathVerb::Move => {
                let Some((x, y)) = points.get(point_index).copied() else {
                    return Vec::new();
                };
                point_index += 1;
                commands.push(RuntimePathCommand::Move { x, y });
            }
            RawPathVerb::Line => {
                let Some((x, y)) = points.get(point_index).copied() else {
                    return Vec::new();
                };
                point_index += 1;
                commands.push(RuntimePathCommand::Line { x, y });
            }
            RawPathVerb::Cubic => {
                let Some((x1, y1)) = points.get(point_index).copied() else {
                    return Vec::new();
                };
                let Some((x2, y2)) = points.get(point_index + 1).copied() else {
                    return Vec::new();
                };
                let Some((x3, y3)) = points.get(point_index + 2).copied() else {
                    return Vec::new();
                };
                point_index += 3;
                commands.push(RuntimePathCommand::Cubic {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                });
            }
            RawPathVerb::Close => commands.push(RuntimePathCommand::Close),
        }
    }
    commands
}
