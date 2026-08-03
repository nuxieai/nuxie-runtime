//! Font-asset source compatibility owned by C++ `ContextValueAssetFont`.

use nuxie_binary::RuntimeFile;

use crate::data_bind_graph::RuntimeDataBindGraphValue;
use crate::{ArtboardInstance, RuntimeFontAssetValue};

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    crate::context_value_enum::integer_payload(next).map(RuntimeDataBindGraphValue::Asset)
}

/// An empty font source must not replace the TextStyle's authored font.
pub(crate) fn resolves_to_font(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    value: &RuntimeFontAssetValue,
) -> bool {
    crate::text::runtime_font_asset_bytes(runtime, instance, value).is_some()
}
