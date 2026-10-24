use crate::source::generated::core_registry::CoreCapabilities;
use crate::{
    CoreHandle,
    source::{
        artboard_component_list::ArtboardComponentList, layout_component::LayoutComponent,
        nested_artboard::NestedArtboard,
    },
};
use anyhow::{Result, ensure};
#[derive(Default)]
pub(crate) struct RowState {
    pub(crate) collapsed: std::collections::HashSet<CoreHandle>,
    pub(crate) mask: Vec<u64>,
    pub(crate) active: Vec<i32>,
    pub(crate) pending_collapse: Option<bool>,
}
impl Drop for RowState {
    fn drop(&mut self) {
        #[cfg(feature = "test-support")]
        crate::hosted_scene::bookkeeping_allocation(0, false);
    }
}
impl ArtboardComponentList {
    pub(crate) fn take_live_move_collapse(&mut self) -> Option<bool> {
        self.live_move_rows
            .as_mut()
            .and_then(|s| s.pending_collapse.take())
    }
    pub(crate) fn live_move_collapse_enabled(&self) -> bool {
        self.live_move_rows.is_some()
    }
    pub(crate) fn live_move_has_sleeping_rows(&self) -> bool {
        self.live_move_rows
            .as_ref()
            .is_some_and(|s| !s.collapsed.is_empty())
    }
    pub(crate) fn live_move_row_collapsed(&self, index: i32) -> bool {
        self.live_move_rows.as_ref().is_some_and(|s| {
            self.list_item(index)
                .is_some_and(|item| s.collapsed.contains(&item))
        })
    }
    pub(crate) fn live_move_collapsed_word(&self, word: usize) -> u64 {
        self.live_move_rows
            .as_ref()
            .and_then(|s| s.mask.get(word).copied())
            .unwrap_or(0)
    }
    pub(crate) fn live_move_rebuild_collapse_mask(&mut self) {
        // Nuxie: no temporary membership set or row walk for ordinary lists.
        if self.live_move_rows.is_none() {
            return;
        }
        let items = (0..self.artboard_count())
            .filter_map(|i| self.list_item(i as i32))
            .collect::<std::collections::HashSet<_>>();
        self.live_move_rows
            .as_mut()
            .unwrap()
            .collapsed
            .retain(|i| items.contains(i));
        let mut mask = Vec::new();
        let mut active = Vec::new();
        if self.live_move_has_sleeping_rows() {
            mask.resize(self.artboard_count().div_ceil(64), 0);
            for i in 0..self.artboard_count() {
                if self.live_move_row_collapsed(i as i32) {
                    mask[i / 64] |= 1 << (i % 64);
                } else {
                    active.push(i as i32);
                }
            }
        }
        let state = self.live_move_rows.as_mut().unwrap();
        state.mask = mask;
        state.active = active;
    }
}
pub fn set_row_collapsed(list: &CoreHandle, index: i32, collapsed: bool) -> Result<()> {
    let scene = crate::hosted_scene::scene_for_operation(list)
        .ok_or_else(|| anyhow::anyhow!("file has no live-move records"))?;
    ensure!(
        crate::hosted_scene::is_screen_list(list),
        "row sleep requires Page.screens"
    );
    // Apply list insertions, removals and moves before interpreting the row
    // index. This flush runs inside the caller's transaction, without playback.
    if let Some(root) = list
        .with(|o| o.as_component().and_then(|c| c.artboard_handle()))
        .flatten()
        .and_then(|b| b.runtime_arena_owner())
    {
        root.update_data_binds(false);
        root.update_pass(false);
    }
    scene.prepare()?;
    let (row, parent, changed) = list
        .with_downcast_mut::<ArtboardComponentList, _>(|l| -> Result<_> {
            ensure!(
                !l.virtualization_enabled(),
                "cannot collapse a virtualised row"
            );
            let item = l
                .list_item(index)
                .ok_or_else(|| anyhow::anyhow!("row not found"))?;
            let state = l.live_move_rows.get_or_insert_with(|| {
                #[cfg(feature = "test-support")]
                crate::hosted_scene::bookkeeping_allocation(0, true);
                Box::default()
            });
            let changed = if collapsed {
                state.collapsed.insert(item)
            } else {
                state.collapsed.remove(&item)
            };
            if changed {
                l.live_move_rebuild_collapse_mask();
                crate::hosted_scene::membership_changed(l);
                l.reset_quiet_row(index as usize);
                l.invalidate_ordered_list_indices_cache();
                l.mark_host_transform_dirty();
            }
            Ok((
                l.artboard_instance(index),
                l.layout_parent_handle(),
                changed,
            ))
        })
        .ok_or_else(|| anyhow::anyhow!("expected a list"))??;
    if !changed {
        return Ok(());
    }
    if let Some(row) = row {
        let hidden = ArtboardComponentList::reconcile_row_collapse_occurrence(list, index, &row);
        if !hidden {
            // Settle the bindings dirt retained by upstream collapse at this
            // mutation boundary. No playback advance or semantic rebuild.
            row.update_data_binds(true);
        }
    }
    if let Some(parent) = parent {
        LayoutComponent::sync_layout_children_occurrence(&parent);
        LayoutComponent::mark_layout_style_dirty_occurrence(&parent);
    }
    Ok(())
}

/// Extend upstream list collapse only after the host opts into row sleep.
/// The root stays awake; virtual component collapse owns every content rule.
pub(crate) fn apply_row_collapse(row: &crate::RuntimeArtboardInstanceHandle, hidden: bool) {
    let children = row.with_artboard(|b| {
        b.as_container_component()
            .unwrap()
            .component_children()
            .to_vec()
    });
    for child in children {
        child.collapse(hidden);
    }
}
/// The only feature visibility rule. `row` selects an item of `object` when it
/// is a list. At every artboard boundary, continue through its host and, for
/// a list host, that artboard's row identity. Root artboards stay uncollapsed.
/// Taking a borrowed object also supports upstream's direct &mut-self suffix.
pub(crate) fn effectively_hidden(
    object: &dyn crate::source::core::CoreObject,
    row: Option<i32>,
) -> bool {
    if object.component_is_collapsed() {
        return true;
    }
    if let Some(list) = object
        .as_registry_any()
        .downcast_ref::<ArtboardComponentList>()
    {
        if row.is_some_and(|i| list.live_move_row_collapsed(i)) {
            return true;
        }
    }
    if let Some(board) = object.as_artboard() {
        if let Some(host) = board.host() {
            let instance = board.runtime_weak_handle().upgrade();
            return host
                .with(|o| {
                    let index = o
                        .as_registry_any()
                        .downcast_ref::<ArtboardComponentList>()
                        .and_then(|list| {
                            instance
                                .as_ref()
                                .map(|b| list.index_of_artboard_instance(b))
                        });
                    effectively_hidden(o, index)
                })
                .unwrap_or(true);
        }
    }
    object
        .component_parent_handle()
        .is_some_and(|parent| parent.with(|o| effectively_hidden(o, None)).unwrap_or(true))
}
/// Use the native suffix on every attachment, even if the host bit did not change.
pub(crate) fn reconcile_slot_collapse(host: &CoreHandle) {
    let hidden = host.with(|o| effectively_hidden(o, None)).unwrap_or(true);
    NestedArtboard::collapse_after_super_occurrence(host, hidden);
    if let Some(unit) = host
        .with(|o| {
            o.as_nested_artboard()
                .and_then(NestedArtboard::artboard_instance_default)
        })
        .flatten()
    {
        reconcile_row_hosts(&unit);
    }
}
pub(crate) fn reconcile_row_hosts(board: &crate::RuntimeArtboardInstanceHandle) {
    for host in board.with_artboard(|b| b.nested_artboards()) {
        reconcile_slot_collapse(&host);
    }
    for list in board.with_artboard(|b| b.artboard_component_lists()) {
        let hidden = list.with(|o| effectively_hidden(o, None)).unwrap_or(true);
        ArtboardComponentList::collapse_after_super_occurrence(&list, hidden);
        let managed = list
            .with_downcast::<ArtboardComponentList, _>(|l| l.live_move_collapse_enabled())
            .unwrap_or(false);
        if !managed {
            ArtboardComponentList::for_each_realized_occurrence(&list, |_, row| {
                reconcile_row_hosts(&row);
            });
        }
    }
}
