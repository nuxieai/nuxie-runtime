//! Direct owner for pinned `src/draw_rules.cpp`.

use crate::properties::property_key_for_name;
use crate::{ArtboardInstance, ComponentDirt};

/// Rust counterpart of `DrawRules::m_ActiveTarget`.
///
/// The serialized id is resolved at construction and refreshed only by the
/// generated `drawTargetIdChanged` callback. Draw-order sorting follows this
/// retained owner; it never resolves the id again.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeDrawRules {
    pub(crate) active_target_index: Option<usize>,
}

impl RuntimeDrawRules {
    pub(crate) fn new(active_target_index: Option<usize>) -> Self {
        let rules = Self::on_added_dirty(active_target_index);
        rules.on_added_clean();
        rules
    }

    /// Mechanical counterpart of pinned C++ `DrawRules::onAddedDirty` after
    /// the graph's approved `CoreContext::resolve` adaptation has converted
    /// the retained `DrawTarget*` to an arena index. The graph only supplies
    /// `Some` for a resolved object whose concrete type is `DrawTarget`.
    fn on_added_dirty(active_target_index: Option<usize>) -> Self {
        Self {
            active_target_index,
        }
    }

    /// Pinned C++ `DrawRules::onAddedClean` is an unconditional success.
    fn on_added_clean(&self) {}
}

pub(crate) fn uint_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("DrawRules")
        || property_key_for_name("DrawRules", "drawTargetId") != Some(property_key)
    {
        return None;
    }

    draw_target_id_changed(artboard, local_id, property_key)
}

/// Mechanical counterpart of pinned C++ `DrawRules::drawTargetIdChanged`.
fn draw_target_id_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    let active_target_local = artboard
        .uint_property(local_id, property_key)
        .and_then(|value| usize::try_from(value).ok())
        .filter(|target_local| {
            artboard.runtime_object_type_name(*target_local) == Some("DrawTarget")
        });
    artboard
        .runtime_drawables
        .set_draw_rules_active_target(local_id, active_target_local);
    Some(artboard.add_dirt(0, ComponentDirt::DRAW_ORDER, false))
}
