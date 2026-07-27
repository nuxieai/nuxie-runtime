//! Direct Rust owner for pinned C++ `include/rive/bones/root_bone.hpp` and
//! `src/bones/root_bone.cpp`.

use std::sync::OnceLock;

use crate::ArtboardInstance;
use crate::artboard::transform_component;
use crate::components::{ComponentHandle, TransformProperty};
use crate::objects::InstanceObjectArena;
use crate::properties::cached_property_key_for_name;

/// Literal `RootBone::onAddedClean`: bypass `Bone::onAddedClean` and delegate
/// directly to `TransformComponent::onAddedClean`.
pub(crate) fn on_added_clean(objects: &mut InstanceObjectArena, handle: ComponentHandle) {
    transform_component::retain_parent_transform_component(objects, handle);
}

pub(crate) fn x_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "RootBone", "x")
}

pub(crate) fn y_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "RootBone", "y")
}

/// Direct `RootBone::xChanged` / `RootBone::yChanged` dispatch.
pub(crate) fn apply_position_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> bool {
    if artboard
        .component(local_id)
        .is_none_or(|component| component.type_name != "RootBone")
        || ![x_property_key(), y_property_key()].contains(&Some(property_key))
    {
        return false;
    }
    let Some(handle) = artboard.component_handle(local_id) else {
        return false;
    };
    artboard.mark_transform_dirty_handle(handle);
    true
}

pub(crate) fn is_position_property(property: TransformProperty) -> bool {
    matches!(property, TransformProperty::X | TransformProperty::Y)
}
