use std::sync::OnceLock;

use crate::components::TransformProperty;
use crate::properties::cached_property_key_for_name;

/// Concrete type discriminator used by the mechanical
/// `RootBone::onAddedClean` translation. The occurrence clean phase skips
/// `Bone::onAddedClean` for this type and retains only the relationship
/// established by `TransformComponent`, allowing any WorldTransformComponent
/// parent.
pub(crate) fn is_root_bone(type_name: &str) -> bool {
    type_name == "RootBone"
}

pub(crate) fn x_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "RootBone", "x")
}

pub(crate) fn y_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "RootBone", "y")
}

/// Import-time devirtualization of `RootBone::xChanged`; the returned transform
/// property routes the generated callback to
/// `TransformComponent::markTransformDirty`.
fn x_changed(property_key: u16) -> Option<TransformProperty> {
    (x_property_key() == Some(property_key)).then_some(TransformProperty::X)
}

/// Import-time devirtualization of `RootBone::yChanged`; the returned transform
/// property routes the generated callback to
/// `TransformComponent::markTransformDirty`.
fn y_changed(property_key: u16) -> Option<TransformProperty> {
    (y_property_key() == Some(property_key)).then_some(TransformProperty::Y)
}

pub(crate) fn transform_property_for_changed_key(property_key: u16) -> Option<TransformProperty> {
    x_changed(property_key).or_else(|| y_changed(property_key))
}
