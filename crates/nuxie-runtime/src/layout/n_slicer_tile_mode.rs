use super::*;
use nuxie_graph::NSlicerTileModeNode;

/// Direct `NSlicerTileMode::onAddedDirty`: reject a non-details parent and
/// install by patch index, replacing an earlier duplicate like C++ map
/// assignment.
pub(crate) fn on_added_dirty(
    owner: &mut n_slicer_details::RuntimeNSlicerDetailsOwner,
    mode: &NSlicerTileModeNode,
    parent_local: Option<usize>,
) -> Option<()> {
    (mode.type_name == "NSlicerTileMode" && parent_local == Some(owner.local_id)).then(|| {
        owner.add_tile_mode(mode);
    })
}
