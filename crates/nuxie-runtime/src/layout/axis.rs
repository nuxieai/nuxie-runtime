use crate::{ArtboardInstance, ComponentDirt};

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
                Some("NSlicer") => instance.add_dirt(details_local, ComponentDirt::N_SLICER, false),
                _ => false,
            }
        })
}
