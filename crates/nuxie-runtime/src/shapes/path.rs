//! Retained Path lifecycle owner.
//!
//! CPU `RawPath` state lives on the occurrence-owned `RuntimePathOwner` in the
//! draw coordinator. Setter callbacks enter through the concrete vertex and
//! parametric modules, then schedule this owner with `ComponentDirt::PATH`.

use crate::{ArtboardInstance, properties::property_key_for_name};

pub(crate) fn bool_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("Path", "isHole") != Some(property_key) {
        return None;
    }
    Some(super::mark_path_dirty(artboard, local_id))
}
