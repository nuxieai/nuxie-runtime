//! Pinned deformer dispatch is a closed NSlicedNode conversion. E2 owns the
//! concrete NSlicer; Path and LinearGradient consume it through retained
//! world/local deformation state.

pub(crate) fn supports_component(type_name: &str) -> bool {
    type_name == "NSlicedNode"
}
