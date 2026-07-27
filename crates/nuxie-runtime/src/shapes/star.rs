use nuxie_graph::{ParametricPathNode, PathGeometryNode, ShapePaintPathKind};

use crate::{ArtboardInstance, Mat2D};

pub(crate) fn resolve(
    artboard: &ArtboardInstance,
    path: &PathGeometryNode,
    authored: &ParametricPathNode,
) -> Option<ParametricPathNode> {
    let ParametricPathNode::Star {
        width,
        height,
        origin_x,
        origin_y,
        points,
        corner_radius,
        inner_radius,
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
    let (points, corner_radius) = super::polygon::live_polygon_fields(
        artboard,
        path.local_id,
        path.type_name,
        *points,
        *corner_radius,
    );
    let inner_radius = super::parametric_path::live_double(
        artboard,
        path.local_id,
        path.type_name,
        "innerRadius",
        *inner_radius,
    );
    Some(ParametricPathNode::Star {
        width: fields.width,
        height: fields.height,
        origin_x: fields.origin_x,
        origin_y: fields.origin_y,
        points,
        corner_radius,
        inner_radius,
    })
}

/// Current Rust geometry projection for `Star::buildPolygon`.
///
/// Preserve the checked multiplication and `< 2` malformed-value guards from
/// the staging Rust implementation. C++ retained-vector sizing and exact edge
/// behavior remain semantic closure work.
pub(crate) fn path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<crate::draw::RuntimePathCommand> {
    let Some(ParametricPathNode::Star {
        width,
        height,
        origin_x,
        origin_y,
        points,
        corner_radius,
        inner_radius,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let Ok(point_count) = usize::try_from(*points) else {
        return Vec::new();
    };
    let Some(count) = point_count.checked_mul(2) else {
        return Vec::new();
    };
    if count < 2 {
        return Vec::new();
    }

    let half_width = *width / 2.0;
    let half_height = *height / 2.0;
    let inner_half_width = *width * *inner_radius / 2.0;
    let inner_half_height = *height * *inner_radius / 2.0;
    let ox = -*origin_x * *width + half_width;
    let oy = -*origin_y * *height + half_height;
    let mut angle = -std::f32::consts::FRAC_PI_2;
    let inc = 2.0 * std::f32::consts::PI / count as f32;
    let mut vertices = Vec::with_capacity(count);
    for _ in 0..point_count {
        vertices.push(crate::draw::virtual_straight_vertex(
            ox + angle.cos() * half_width,
            oy + angle.sin() * half_height,
            *corner_radius,
        ));
        angle += inc;
        vertices.push(crate::draw::virtual_straight_vertex(
            ox + angle.cos() * inner_half_width,
            oy + angle.sin() * inner_half_height,
            *corner_radius,
        ));
        angle += inc;
    }

    crate::draw::closed_straight_vertices_path_commands(path, path_kind, transform, vertices)
}
