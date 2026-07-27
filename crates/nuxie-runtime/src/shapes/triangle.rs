use nuxie_graph::{ParametricPathNode, PathGeometryNode, ShapePaintPathKind};

use crate::{ArtboardInstance, Mat2D};

pub(crate) fn resolve(
    artboard: &ArtboardInstance,
    path: &PathGeometryNode,
    authored: &ParametricPathNode,
) -> Option<ParametricPathNode> {
    let ParametricPathNode::Triangle {
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
    Some(ParametricPathNode::Triangle {
        width: fields.width,
        height: fields.height,
        origin_x: fields.origin_x,
        origin_y: fields.origin_y,
    })
}

/// Current Rust geometry projection for `Triangle::update`.
///
/// C++ retains three StraightVertex members. The shared synthetic-vertex
/// adapter remains in place until the semantic owner-family port.
pub(crate) fn path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<crate::draw::RuntimePathCommand> {
    let Some(ParametricPathNode::Triangle {
        width,
        height,
        origin_x,
        origin_y,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let ox = -*origin_x * *width;
    let oy = -*origin_y * *height;
    crate::draw::closed_straight_vertices_path_commands(
        path,
        path_kind,
        transform,
        vec![
            crate::draw::virtual_straight_vertex(ox + *width / 2.0, oy, 0.0),
            crate::draw::virtual_straight_vertex(ox + *width, oy + *height, 0.0),
            crate::draw::virtual_straight_vertex(ox, oy + *height, 0.0),
        ],
    )
}
