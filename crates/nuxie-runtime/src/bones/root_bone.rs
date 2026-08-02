use std::sync::OnceLock;

use crate::properties::cached_property_key_for_name;

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
