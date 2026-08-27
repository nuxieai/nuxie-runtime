use super::*;

pub(crate) fn is_axis(type_name: &str) -> bool {
    type_name == "AxisY"
}

/// Direct `AxisY::onAddedDirty`: validate its NSlicerDetails parent, then
/// register with the concrete Y-axis list.
pub(crate) fn on_added_dirty(
    owner: &mut n_slicer_details::RuntimeNSlicerDetailsOwner,
    axis: &NSlicerAxisNode,
    parent_local: Option<usize>,
) -> Option<()> {
    (is_axis(axis.type_name) && super::axis::on_added_dirty(owner, parent_local).is_some()).then(
        || {
            owner.add_axis_y(axis);
        },
    )
}
