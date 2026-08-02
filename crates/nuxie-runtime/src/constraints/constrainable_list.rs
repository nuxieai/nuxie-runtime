//! Direct owner for pinned `src/constraints/constrainable_list.cpp`.

use super::*;

/// Runtime constraint application for the C++ `src/constraints/` path.
pub(crate) fn apply_list_constraints(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
) -> bool {
    if artboard
        .objects
        .component(component_index)
        .is_none_or(|component| component.concrete.constrainable_list.is_none())
    {
        return false;
    }

    let list_local = artboard.component_at(component_index).local_id;
    let Some(mut item_transforms) = artboard
        .component_list_state_mut(list_local)
        .map(|list| std::mem::take(&mut list.item_transforms))
    else {
        return false;
    };
    let changed = list_constraint::constrain_component_list_item_transforms(
        artboard,
        list_local,
        component_index,
        &mut item_transforms,
    );
    if let Some(list) = artboard.component_list_state_mut(list_local) {
        list.item_transforms = item_transforms;
    }
    changed
}
