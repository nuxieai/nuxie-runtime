use nuxie_graph::{ParametricPathNode, PathGeometryNode, ShapePaintPathKind};

use crate::{ArtboardInstance, Mat2D};

pub(crate) fn live_polygon_fields(
    artboard: &ArtboardInstance,
    local_id: usize,
    concrete_type_name: &str,
    points: u32,
    corner_radius: f32,
) -> (u32, f32) {
    let points = super::parametric_path::live_uint(
        artboard,
        local_id,
        concrete_type_name,
        "points",
        u64::from(points),
    ) as u32;
    let corner_radius = super::parametric_path::live_double(
        artboard,
        local_id,
        concrete_type_name,
        "cornerRadius",
        corner_radius,
    );
    (points, corner_radius)
}

pub(crate) fn resolve(
    artboard: &ArtboardInstance,
    path: &PathGeometryNode,
    authored: &ParametricPathNode,
) -> Option<ParametricPathNode> {
    let ParametricPathNode::Polygon {
        width,
        height,
        origin_x,
        origin_y,
        points,
        corner_radius,
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
    let (points, corner_radius) = live_polygon_fields(
        artboard,
        path.local_id,
        path.type_name,
        *points,
        *corner_radius,
    );
    Some(ParametricPathNode::Polygon {
        width: fields.width,
        height: fields.height,
        origin_x: fields.origin_x,
        origin_y: fields.origin_y,
        points,
        corner_radius,
    })
}

/// Current Rust geometry projection for `Polygon::update/buildPolygon`.
///
/// The count conversion and `< 2` early return are deliberately retained even
/// though the pinned C++ vector-resize loop has different malformed-value
/// edges. This extraction does not make a semantic decision for FL-E.
pub(crate) fn path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<crate::draw::RuntimePathCommand> {
    let Some(ParametricPathNode::Polygon {
        width,
        height,
        origin_x,
        origin_y,
        points,
        corner_radius,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let Ok(count) = usize::try_from(*points) else {
        return Vec::new();
    };
    if count < 2 {
        return Vec::new();
    }

    let half_width = *width / 2.0;
    let half_height = *height / 2.0;
    let ox = -*origin_x * *width + half_width;
    let oy = -*origin_y * *height + half_height;
    let mut angle = -std::f32::consts::FRAC_PI_2;
    let inc = 2.0 * std::f32::consts::PI / *points as f32;
    let mut vertices = Vec::with_capacity(count);
    for _ in 0..count {
        vertices.push(crate::draw::virtual_straight_vertex(
            ox + angle.cos() * half_width,
            oy + angle.sin() * half_height,
            *corner_radius,
        ));
        angle += inc;
    }

    crate::draw::closed_straight_vertices_path_commands(path, path_kind, transform, vertices)
}
