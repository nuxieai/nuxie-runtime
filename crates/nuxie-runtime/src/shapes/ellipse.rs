use nuxie_graph::{ParametricPathNode, PathGeometryNode, ShapePaintPathKind};

use crate::draw::RuntimePathCommand;
use crate::{ArtboardInstance, Mat2D};

const CIRCLE_CONSTANT: f32 = 0.552_284_8;

pub(crate) fn resolve(
    artboard: &ArtboardInstance,
    path: &PathGeometryNode,
    authored: &ParametricPathNode,
) -> Option<ParametricPathNode> {
    let ParametricPathNode::Ellipse {
        width,
        height,
        origin_x,
        origin_y,
    } = authored
    else {
        return None;
    };
    let fields = super::parametric_path::live_fields(
        artboard,
        path.local_id,
        path.type_name,
        *width,
        *height,
        *origin_x,
        *origin_y,
    );
    Some(ParametricPathNode::Ellipse {
        width: fields.width,
        height: fields.height,
        origin_x: fields.origin_x,
        origin_y: fields.origin_y,
    })
}

/// Current Rust geometry projection for `Ellipse::update`.
///
/// Pinned C++ retains four CubicDetachedVertex members and updates them under
/// Path dirt before `Path::update`. This structural move deliberately keeps
/// the existing on-demand command materialization and exact current constant,
/// ordering, transform, and reversal behavior.
pub(crate) fn path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Ellipse {
        width,
        height,
        origin_x,
        origin_y,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };
    let reverse_for_clockwise_fill = path_kind == ShapePaintPathKind::LocalClockwise
        && crate::draw::path_needs_clockwise_reversal(path, transform);

    let radius_x = *width / 2.0;
    let radius_y = *height / 2.0;
    let ox = -*origin_x * *width + radius_x;
    let oy = -*origin_y * *height + radius_y;
    let top = (ox, oy - radius_y);
    let right = (ox + radius_x, oy);
    let bottom = (ox, oy + radius_y);
    let left = (ox - radius_x, oy);

    let mut commands = Vec::new();
    push_move(&mut commands, transform, top);
    push_cubic(
        &mut commands,
        transform,
        (ox + radius_x * CIRCLE_CONSTANT, oy - radius_y),
        (ox + radius_x, oy - CIRCLE_CONSTANT * radius_y),
        right,
    );
    push_cubic(
        &mut commands,
        transform,
        (ox + radius_x, oy + CIRCLE_CONSTANT * radius_y),
        (ox + radius_x * CIRCLE_CONSTANT, oy + radius_y),
        bottom,
    );
    push_cubic(
        &mut commands,
        transform,
        (ox - radius_x * CIRCLE_CONSTANT, oy + radius_y),
        (ox - radius_x, oy + radius_y * CIRCLE_CONSTANT),
        left,
    );
    push_cubic(
        &mut commands,
        transform,
        (ox - radius_x, oy - radius_y * CIRCLE_CONSTANT),
        (ox - radius_x * CIRCLE_CONSTANT, oy - radius_y),
        top,
    );
    commands.push(RuntimePathCommand::Close);
    if reverse_for_clockwise_fill {
        crate::draw::path_commands_backwards(&commands)
    } else {
        commands
    }
}

fn push_move(commands: &mut Vec<RuntimePathCommand>, transform: Mat2D, point: (f32, f32)) {
    let (x, y) = transform.map_point(point.0, point.1);
    commands.push(RuntimePathCommand::Move { x, y });
}

fn push_cubic(
    commands: &mut Vec<RuntimePathCommand>,
    transform: Mat2D,
    point1: (f32, f32),
    point2: (f32, f32),
    point3: (f32, f32),
) {
    let (x1, y1) = transform.map_point(point1.0, point1.1);
    let (x2, y2) = transform.map_point(point2.0, point2.1);
    let (x3, y3) = transform.map_point(point3.0, point3.1);
    commands.push(RuntimePathCommand::Cubic {
        x1,
        y1,
        x2,
        y2,
        x3,
        y3,
    });
}
