use crate::ArtboardInstance;
use crate::properties::cached_property_key_for_name;
use std::sync::OnceLock;

const COUNTER_CLOCKWISE_PATH_FLAG: u64 = 1 << 1;

/// Generated `PointsCommonPathBase::isClosed` storage read.
///
/// Rust retains generated fields in the occurrence object arena rather than
/// duplicating them in a renderer sidecar. Looking the key up through the
/// concrete type preserves the generated base-field inheritance used by
/// `PointsPath` and `ListPath`
/// (`include/rive/generated/shapes/points_common_path_base.hpp:34-67`).
pub(crate) fn is_closed(
    artboard: &ArtboardInstance,
    local_id: usize,
    concrete_type_name: &str,
    default: bool,
) -> bool {
    if !matches!(
        concrete_type_name,
        "PointsCommonPath" | "PointsPath" | "ListPath"
    ) {
        return default;
    }
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "PointsCommonPath", "isClosed")
        .and_then(|key| artboard.bool_property(local_id, key))
        .unwrap_or(default)
}

/// Direct port of pinned C++ `PointsCommonPath::isClockwise`
/// (`src/shapes/points_common_path.cpp:12-15`).
pub(crate) fn is_clockwise(artboard: &ArtboardInstance, local_id: usize, default: bool) -> bool {
    let default_flags = if default {
        0
    } else {
        COUNTER_CLOCKWISE_PATH_FLAG
    };
    super::path::path_flags(artboard, local_id, default_flags) & COUNTER_CLOCKWISE_PATH_FLAG == 0
}
