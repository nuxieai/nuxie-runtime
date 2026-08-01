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
        Self {
            active_target_index,
        }
    }
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

    let active_target_local = artboard
        .uint_property(local_id, property_key)
        .and_then(|value| usize::try_from(value).ok());
    artboard
        .runtime_drawables
        .set_draw_rules_active_target(local_id, active_target_local);
    Some(artboard.add_dirt(0, ComponentDirt::DRAW_ORDER, false))
}
