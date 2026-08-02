//! Direct owner for pinned `src/constraints/list_constraint.cpp`.

use super::*;

pub(super) fn constrain_component_list_item_transforms(
    artboard: &ArtboardInstance,
    list_local: usize,
    list_component_index: ComponentHandle,
    item_transforms: &mut [Mat2D],
) -> bool {
    // C++ explicitly skips list constraints while the component list is
    // virtualized. The scroll virtualizer owns row positions in that mode.
    if scrolling::scroll_virtualizer::component_list_virtualization(artboard, list_local).is_some()
    {
        return false;
    }

    let constraint_count = artboard
        .objects
        .component(list_component_index)
        .and_then(|component| component.concrete.constrainable_list.as_ref())
        .map_or(0, |list| list.constraints.len());
    let mut changed = false;
    for index in 0..constraint_count {
        let Some(constraint) = artboard
            .objects
            .component(list_component_index)
            .and_then(|component| component.concrete.constrainable_list.as_ref())
            .and_then(|list| list.constraints.get(index))
            .copied()
        else {
            continue;
        };
        changed |= list_follow_path_constraint::apply_to_transforms(
            artboard,
            list_component_index,
            constraint,
            item_transforms,
        );
    }
    changed
}
