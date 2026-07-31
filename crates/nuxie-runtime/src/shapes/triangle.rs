//! Triangle retains three virtual StraightVertex members and rebuilds them
//! only when inherited ParametricPath dirt reaches `update(Path)`.

use crate::ArtboardInstance;

pub(crate) fn property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    super::parametric_path::property_changed(artboard, local_id, property_key)
}
