//! Source matching and referencer application owned by C++
//! `DataBindContextValueArtboard`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;
use crate::{ArtboardInstance, RuntimeBindableArtboard};

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    super_integer(next).map(RuntimeDataBindGraphValue::Artboard)
}

fn super_integer(value: &RuntimeDataBindGraphValue) -> Option<u64> {
    crate::context_value_enum::integer_payload(value)
}

/// Apply the `ArtboardReferencer` branch of pinned C++ `apply`.
///
/// The generated integer property and the mounted child are deliberately
/// separate. A live source replaces the mounted occurrence without changing
/// `artboardId`; a file-backed source uses the same replacement path on its
/// first application and the generated setter on later applications.
pub(crate) fn apply_to_nested_host(
    target: &mut ArtboardInstance,
    target_local_id: usize,
    value: &RuntimeDataBindGraphValue,
    runtime_artboard: Option<&RuntimeBindableArtboard>,
    first_apply: bool,
) -> Option<bool> {
    let RuntimeDataBindGraphValue::Artboard(value) = value else {
        return None;
    };
    Some(if let Some(runtime_artboard) = runtime_artboard {
        let Some(source) = runtime_artboard.artboard_instance() else {
            return Some(false);
        };
        target.replace_nested_artboard_artboard_instance(target_local_id, source)
    } else if first_apply {
        target.replace_nested_artboard_artboard_id(target_local_id, *value)
    } else {
        target.set_nested_artboard_artboard_id(target_local_id, *value)
    })
}
