//! Direct owners for the pinned `src/shapes` callback family.
//!
//! The generated C++ hierarchy dispatches a setter directly to the concrete
//! `...Changed` override. Rust stores generated objects in a flat arena, so
//! this module is the one type dispatch at that generated boundary; each
//! concrete file owns its property-key interpretation and dirt callback.

pub(crate) mod clipping_shape;
pub(crate) mod cubic_asymmetric_vertex;
pub(crate) mod cubic_detached_vertex;
pub(crate) mod cubic_mirrored_vertex;
pub(crate) mod cubic_vertex;
pub(crate) mod deformer;
pub(crate) mod ellipse;
pub(crate) mod list_path;
pub(crate) mod paint;
pub(crate) mod parametric_path;
pub(crate) mod path;
pub(crate) mod path_composer;
pub(crate) mod path_vertex;
pub(crate) mod points_common_path;
pub(crate) mod points_path;
pub(crate) mod polygon;
pub(crate) mod rectangle;
pub(crate) mod shape;
pub(crate) mod shape_paint_container;
pub(crate) mod star;
pub(crate) mod straight_vertex;
pub(crate) mod triangle;
pub(crate) mod vertex;

use crate::{ArtboardInstance, ComponentDirt};

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    match type_name {
        Some("StraightVertex") => {
            straight_vertex::property_changed(artboard, local_id, property_key)
        }
        Some("CubicMirroredVertex") => {
            cubic_mirrored_vertex::property_changed(artboard, local_id, property_key)
        }
        Some("CubicAsymmetricVertex") => {
            cubic_asymmetric_vertex::property_changed(artboard, local_id, property_key)
        }
        Some("CubicDetachedVertex") => {
            cubic_detached_vertex::property_changed(artboard, local_id, property_key)
        }
        Some("Ellipse") => ellipse::property_changed(artboard, local_id, property_key),
        Some("ParametricPath") => {
            parametric_path::property_changed(artboard, local_id, property_key)
        }
        Some("Polygon") => polygon::property_changed(artboard, local_id, property_key),
        Some("Rectangle") => rectangle::property_changed(artboard, local_id, property_key),
        Some("Star") => star::property_changed(artboard, local_id, property_key),
        Some("Triangle") => triangle::property_changed(artboard, local_id, property_key),
        Some("Dash") => paint::dash::double_property_changed(artboard, local_id, property_key),
        Some("DashPath") => {
            paint::dash_path::double_property_changed(artboard, local_id, property_key)
        }
        Some("Feather") => {
            paint::feather::double_property_changed(artboard, local_id, property_key)
        }
        Some("LinearGradient" | "RadialGradient") => {
            paint::linear_gradient::double_property_changed(artboard, local_id, property_key)
        }
        Some("GradientStop") => {
            paint::gradient_stop::double_property_changed(artboard, local_id, property_key)
        }
        Some("Stroke") => paint::stroke::double_property_changed(artboard, local_id, property_key),
        Some("TrimPath") => {
            paint::trim_path::double_property_changed(artboard, local_id, property_key)
        }
        _ => None,
    }
}

pub(crate) fn bool_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    match type_name {
        Some(
            "Path" | "PointsPath" | "ListPath" | "Ellipse" | "Polygon" | "Rectangle" | "Star"
            | "Triangle",
        ) => path::bool_property_changed(artboard, local_id, property_key),
        Some("ClippingShape") => {
            clipping_shape::bool_property_changed(artboard, local_id, property_key)
        }
        Some("Dash") => paint::dash::bool_property_changed(artboard, local_id, property_key),
        Some("DashPath") => {
            paint::dash_path::bool_property_changed(artboard, local_id, property_key)
        }
        Some("Feather") => paint::feather::bool_property_changed(artboard, local_id, property_key),
        _ => None,
    }
}

pub(crate) fn uint_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    match type_name {
        Some("Feather") => paint::feather::uint_property_changed(artboard, local_id, property_key),
        Some("Stroke") => paint::stroke::uint_property_changed(artboard, local_id, property_key),
        Some("TrimPath") => {
            paint::trim_path::uint_property_changed(artboard, local_id, property_key)
        }
        _ => None,
    }
}

pub(crate) fn color_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    match type_name {
        Some("GradientStop") => {
            paint::gradient_stop::color_property_changed(artboard, local_id, property_key)
        }
        _ => None,
    }
}

pub(crate) fn mark_path_dirty(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    artboard.add_dirt(local_id, ComponentDirt::PATH, false)
}
