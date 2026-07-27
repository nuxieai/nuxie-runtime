use crate::artboard::ArtboardInstance;
use crate::components::ComponentDirt;
use crate::properties::property_key_for_name;

/// Direct generated-property callback counterpart of
/// `DrawRules::drawTargetIdChanged`.
///
/// C++ resolves `m_ActiveTarget` in the concrete owner and dirties the owning
/// Artboard's draw order (`src/draw_rules.cpp:30-43`). Rust's retained
/// drawable list resolves the live target while sorting; the generic property
/// dispatcher delegates this owner's callback here so the Artboard dirt
/// boundary is source-local while the remaining target-resolution work stays
/// explicitly queued for FL-E.
pub(crate) fn apply_uint_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> bool {
    if artboard.slot(local_id).and_then(|slot| slot.type_name) != Some("DrawRules")
        || property_key_for_name("DrawRules", "drawTargetId") != Some(property_key)
    {
        return false;
    }
    artboard.add_dirt(0, ComponentDirt::DRAW_ORDER, false)
}
