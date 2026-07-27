use nuxie_graph::{ParametricPathNode, PathGeometryNode, ShapePaintPathKind};

use crate::{ArtboardInstance, Mat2D};

pub(crate) fn resolve(
    artboard: &ArtboardInstance,
    path: &PathGeometryNode,
    authored: &ParametricPathNode,
) -> Option<ParametricPathNode> {
    let ParametricPathNode::Rectangle {
        width,
        height,
        origin_x,
        origin_y,
        link_corner_radius,
        corner_radius_tl,
        corner_radius_tr,
        corner_radius_bl,
        corner_radius_br,
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
    let link_corner_radius = super::parametric_path::live_bool(
        artboard,
        path.local_id,
        path.type_name,
        "linkCornerRadius",
        *link_corner_radius,
    );

    Some(ParametricPathNode::Rectangle {
        width: fields.width,
        height: fields.height,
        origin_x: fields.origin_x,
        origin_y: fields.origin_y,
        link_corner_radius,
        corner_radius_tl: super::parametric_path::live_double(
            artboard,
            path.local_id,
            path.type_name,
            "cornerRadiusTL",
            *corner_radius_tl,
        ),
        corner_radius_tr: super::parametric_path::live_double(
            artboard,
            path.local_id,
            path.type_name,
            "cornerRadiusTR",
            *corner_radius_tr,
        ),
        corner_radius_bl: super::parametric_path::live_double(
            artboard,
            path.local_id,
            path.type_name,
            "cornerRadiusBL",
            *corner_radius_bl,
        ),
        corner_radius_br: super::parametric_path::live_double(
            artboard,
            path.local_id,
            path.type_name,
            "cornerRadiusBR",
            *corner_radius_br,
        ),
    })
}

/// Current Rust geometry projection for `Rectangle::update`.
///
/// Keep the existing linked-radius selection and synthetic-vertex adapter
/// unchanged. The generated radius callbacks and C++ retained four-vertex
/// lifecycle remain for FL-E.
pub(crate) fn path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<crate::draw::RuntimePathCommand> {
    let Some(ParametricPathNode::Rectangle {
        width,
        height,
        origin_x,
        origin_y,
        link_corner_radius,
        corner_radius_tl,
        corner_radius_tr,
        corner_radius_bl,
        corner_radius_br,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let width = *width;
    let height = *height;
    let left = -*origin_x * width;
    let top = -*origin_y * height;
    let right = left + width;
    let bottom = top + height;
    let top_left_radius = *corner_radius_tl;
    let top_right_radius = if *link_corner_radius {
        top_left_radius
    } else {
        *corner_radius_tr
    };
    let bottom_right_radius = if *link_corner_radius {
        top_left_radius
    } else {
        *corner_radius_br
    };
    let bottom_left_radius = if *link_corner_radius {
        top_left_radius
    } else {
        *corner_radius_bl
    };

    crate::draw::closed_straight_vertices_path_commands(
        path,
        path_kind,
        transform,
        vec![
            crate::draw::virtual_straight_vertex(left, top, top_left_radius),
            crate::draw::virtual_straight_vertex(right, top, top_right_radius),
            crate::draw::virtual_straight_vertex(right, bottom, bottom_right_radius),
            crate::draw::virtual_straight_vertex(left, bottom, bottom_left_radius),
        ],
    )
}
