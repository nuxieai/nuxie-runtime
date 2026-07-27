//! Direct Rust home for `include/rive/text/text_style_axis.hpp` and
//! `src/text/text_style_axis.cpp`.

use crate::ArtboardInstance;
use crate::components::ComponentDirt;
use crate::properties::property_key_for_name;

/// Behavior-preserving extraction of the existing axis-value callback.
/// Pinned C++ dirties the owning TextStyle, which then forwards TextShape dirt
/// through its retained text/helper topology (`src/text/text_style_axis.cpp:
/// 27-30`).
///
/// Retained axis ownership, tag changes, and inherited-parent validation stay
/// pending for the complete text semantic wave.
pub(crate) fn apply_double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> bool {
    if type_name != Some("TextStyleAxis")
        || property_key_for_name("TextStyleAxis", "axisValue") != Some(property_key)
    {
        return false;
    }
    instance
        .component_parent_local(local_id)
        .is_some_and(|style| instance.add_dirt(style, ComponentDirt::TEXT_SHAPE, false))
}
