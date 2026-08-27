//! Pinned deformer dispatch is a closed NSlicedNode conversion. E2 owns the
//! concrete NSlicer; Path and LinearGradient consume it through retained
//! world/local deformation state.

/// Rust retains the component separately, so the non-null result of C++
/// `RenderPathDeformer::from(Component*)` is represented by this classifier.
pub(crate) fn render_path_deformer_from_component(type_name: &str) -> bool {
    type_name == "NSlicedNode"
}

/// Rust retains the component separately, so the non-null result of C++
/// `PointDeformer::from(Component*)` is represented by this classifier.
pub(crate) fn point_deformer_from_component(type_name: &str) -> bool {
    type_name == "NSlicedNode"
}

pub(crate) fn supports_component(type_name: &str) -> bool {
    render_path_deformer_from_component(type_name) || point_deformer_from_component(type_name)
}
