use nuxie_graph::{ParametricPathNode, PathGeometryNode};
use std::sync::OnceLock;

use crate::ArtboardInstance;
use crate::properties::{cached_property_key_for_name, property_key_for_name};

/// Live generated fields shared by every concrete `ParametricPath`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeParametricPathFields {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) origin_x: f32,
    pub(crate) origin_y: f32,
}

macro_rules! cached_parametric_property_key {
    ($type_name:literal, $property_name:literal) => {{
        static KEY: OnceLock<Option<u16>> = OnceLock::new();
        cached_property_key_for_name(&KEY, $type_name, $property_name)
    }};
}

fn property_key(type_name: &str, property_name: &str) -> Option<u16> {
    // Keep the same concrete-type cache partitioning as the pre-split
    // `draw.rs` implementation. Besides avoiding a structural performance
    // regression, the fallback preserves malformed type/variant behavior.
    match (type_name, property_name) {
        ("Ellipse", "width") => cached_parametric_property_key!("Ellipse", "width"),
        ("Ellipse", "height") => cached_parametric_property_key!("Ellipse", "height"),
        ("Ellipse", "originX") => cached_parametric_property_key!("Ellipse", "originX"),
        ("Ellipse", "originY") => cached_parametric_property_key!("Ellipse", "originY"),
        ("Polygon", "width") => cached_parametric_property_key!("Polygon", "width"),
        ("Polygon", "height") => cached_parametric_property_key!("Polygon", "height"),
        ("Polygon", "originX") => cached_parametric_property_key!("Polygon", "originX"),
        ("Polygon", "originY") => cached_parametric_property_key!("Polygon", "originY"),
        ("Polygon", "points") => cached_parametric_property_key!("Polygon", "points"),
        ("Polygon", "cornerRadius") => {
            cached_parametric_property_key!("Polygon", "cornerRadius")
        }
        ("Star", "width") => cached_parametric_property_key!("Star", "width"),
        ("Star", "height") => cached_parametric_property_key!("Star", "height"),
        ("Star", "originX") => cached_parametric_property_key!("Star", "originX"),
        ("Star", "originY") => cached_parametric_property_key!("Star", "originY"),
        ("Star", "points") => cached_parametric_property_key!("Star", "points"),
        ("Star", "cornerRadius") => cached_parametric_property_key!("Star", "cornerRadius"),
        ("Star", "innerRadius") => cached_parametric_property_key!("Star", "innerRadius"),
        ("Rectangle", "width") => cached_parametric_property_key!("Rectangle", "width"),
        ("Rectangle", "height") => cached_parametric_property_key!("Rectangle", "height"),
        ("Rectangle", "originX") => cached_parametric_property_key!("Rectangle", "originX"),
        ("Rectangle", "originY") => cached_parametric_property_key!("Rectangle", "originY"),
        ("Rectangle", "linkCornerRadius") => {
            cached_parametric_property_key!("Rectangle", "linkCornerRadius")
        }
        ("Rectangle", "cornerRadiusTL") => {
            cached_parametric_property_key!("Rectangle", "cornerRadiusTL")
        }
        ("Rectangle", "cornerRadiusTR") => {
            cached_parametric_property_key!("Rectangle", "cornerRadiusTR")
        }
        ("Rectangle", "cornerRadiusBL") => {
            cached_parametric_property_key!("Rectangle", "cornerRadiusBL")
        }
        ("Rectangle", "cornerRadiusBR") => {
            cached_parametric_property_key!("Rectangle", "cornerRadiusBR")
        }
        ("Triangle", "width") => cached_parametric_property_key!("Triangle", "width"),
        ("Triangle", "height") => cached_parametric_property_key!("Triangle", "height"),
        ("Triangle", "originX") => cached_parametric_property_key!("Triangle", "originX"),
        ("Triangle", "originY") => cached_parametric_property_key!("Triangle", "originY"),
        _ => property_key_for_name(type_name, property_name),
    }
}

pub(crate) fn live_double(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: f32,
) -> f32 {
    property_key(type_name, property_name)
        .and_then(|property_key| artboard.double_property(local_id, property_key))
        .unwrap_or(default)
}

pub(crate) fn live_bool(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: bool,
) -> bool {
    property_key(type_name, property_name)
        .and_then(|property_key| artboard.bool_property(local_id, property_key))
        .unwrap_or(default)
}

pub(crate) fn live_uint(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: u64,
) -> u64 {
    property_key(type_name, property_name)
        .and_then(|property_key| artboard.uint_property(local_id, property_key))
        .unwrap_or(default)
}

/// Read the generated `ParametricPathBase` fields from the live occurrence.
///
/// The property keys belong to the generated base even when the concrete
/// object is an Ellipse/Rectangle/Polygon/Star/Triangle. This is the same
/// occurrence storage previously read from `draw.rs`.
pub(crate) fn live_fields(
    artboard: &ArtboardInstance,
    local_id: usize,
    concrete_type_name: &str,
    width: f32,
    height: f32,
    origin_x: f32,
    origin_y: f32,
) -> RuntimeParametricPathFields {
    RuntimeParametricPathFields {
        width: live_double(artboard, local_id, concrete_type_name, "width", width),
        height: live_double(artboard, local_id, concrete_type_name, "height", height),
        origin_x: live_double(artboard, local_id, concrete_type_name, "originX", origin_x),
        origin_y: live_double(artboard, local_id, concrete_type_name, "originY", origin_y),
    }
}

/// Resolve one concrete parametric-path occurrence through its direct owner.
pub(crate) fn resolve(
    artboard: &ArtboardInstance,
    path: &PathGeometryNode,
) -> Option<ParametricPathNode> {
    let authored = path.parametric.as_ref()?;
    match authored {
        ParametricPathNode::Ellipse { .. } => super::ellipse::resolve(artboard, path, authored),
        ParametricPathNode::Polygon { .. } => super::polygon::resolve(artboard, path, authored),
        ParametricPathNode::Rectangle { .. } => super::rectangle::resolve(artboard, path, authored),
        ParametricPathNode::Star { .. } => super::star::resolve(artboard, path, authored),
        ParametricPathNode::Triangle { .. } => super::triangle::resolve(artboard, path, authored),
    }
}

/// Existing Rust equivalent of `ParametricPath::measureLayout`.
///
/// The caller has already converted the layout measure modes into numeric
/// maxima, so preserving `f32::min` here retains the staging Rust NaN behavior.
pub(crate) fn measure_layout(
    path: &PathGeometryNode,
    max_width: f32,
    max_height: f32,
) -> Option<(f32, f32)> {
    let (width, height) = match path.parametric.as_ref()? {
        ParametricPathNode::Ellipse { width, height, .. }
        | ParametricPathNode::Polygon { width, height, .. }
        | ParametricPathNode::Rectangle { width, height, .. }
        | ParametricPathNode::Star { width, height, .. }
        | ParametricPathNode::Triangle { width, height, .. } => (*width, *height),
    };
    Some((max_width.min(width), max_height.min(height)))
}

/// Existing layout-control projection for `ParametricPath::controlSize`.
///
/// C++ also marks world/path dirt from this setter. Those callbacks and their
/// owner-local layout propagation are intentionally not added by this
/// structural extraction; they remain part of the FL-E semantic closure.
pub(crate) fn with_control_size(
    parametric: ParametricPathNode,
    width: f32,
    height: f32,
) -> ParametricPathNode {
    match parametric {
        ParametricPathNode::Ellipse {
            origin_x, origin_y, ..
        } => ParametricPathNode::Ellipse {
            width,
            height,
            origin_x,
            origin_y,
        },
        ParametricPathNode::Polygon {
            origin_x,
            origin_y,
            points,
            corner_radius,
            ..
        } => ParametricPathNode::Polygon {
            width,
            height,
            origin_x,
            origin_y,
            points,
            corner_radius,
        },
        ParametricPathNode::Star {
            origin_x,
            origin_y,
            points,
            corner_radius,
            inner_radius,
            ..
        } => ParametricPathNode::Star {
            width,
            height,
            origin_x,
            origin_y,
            points,
            corner_radius,
            inner_radius,
        },
        ParametricPathNode::Triangle {
            origin_x, origin_y, ..
        } => ParametricPathNode::Triangle {
            width,
            height,
            origin_x,
            origin_y,
        },
        ParametricPathNode::Rectangle {
            origin_x,
            origin_y,
            link_corner_radius,
            corner_radius_tl,
            corner_radius_tr,
            corner_radius_bl,
            corner_radius_br,
            ..
        } => ParametricPathNode::Rectangle {
            width,
            height,
            origin_x,
            origin_y,
            link_corner_radius,
            corner_radius_tl,
            corner_radius_tr,
            corner_radius_bl,
            corner_radius_br,
        },
    }
}
