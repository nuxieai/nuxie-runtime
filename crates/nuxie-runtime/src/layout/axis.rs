use crate::ArtboardInstance;

/// Direct `Axis::onAddedDirty` body after Component Super has linked the
/// occurrence to its retained parent. Only the two concrete
/// `NSlicerDetails` owners are valid.
pub(crate) fn on_added_dirty(
    owner: &super::n_slicer_details::RuntimeNSlicerDetailsOwner,
    parent_local: Option<usize>,
) -> Option<()> {
    (super::n_slicer_details::is_details(owner.type_name) && parent_local == Some(owner.local_id))
        .then_some(())
}

/// Direct port of `Axis::offsetChanged`: an Axis never owns the mesh dirt;
/// its `NSlicerDetails` parent does.
pub(crate) fn offset_changed(instance: &mut ArtboardInstance, axis_local: usize) -> bool {
    instance
        .component_parent_local(axis_local)
        .is_some_and(|details_local| {
            match instance
                .component(details_local)
                .map(|component| component.type_name)
            {
                Some("NSlicedNode") => super::n_sliced_node::axis_changed(instance, details_local),
                Some("NSlicer") => super::n_slicer::axis_changed(instance, details_local),
                _ => false,
            }
        })
}
