//! Font-asset source compatibility owned by C++
//! `DataBindContextValueAssetFont`.

use nuxie_binary::{RuntimeFile, RuntimeObject};

use crate::RuntimeFontAssetValue;
use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    crate::context_value_enum::integer_payload(next).map(RuntimeDataBindGraphValue::Asset)
}

/// Mechanical `DataBindContextValueAssetFont::fileAsset` translation.
/// A valid FontAsset is returned even before it has decoded a Font.
pub(crate) fn file_asset<'a>(
    runtime: &'a RuntimeFile,
    value: &RuntimeFontAssetValue,
) -> Option<&'a RuntimeObject> {
    usize::try_from(value.file_asset_index())
        .ok()
        .and_then(|index| runtime.file_asset(index))
        .filter(|asset| asset.type_name == "FontAsset")
}

/// Exact TextStyle branch gate from pinned `apply`: a resolved file
/// FontAsset always replaces the retained style asset, even while its decoded
/// Font is null. The private live Font is considered only when file lookup
/// failed, and an entirely empty source preserves the authored style.
pub(crate) fn applies_to_text_style(runtime: &RuntimeFile, value: &RuntimeFontAssetValue) -> bool {
    file_asset(runtime, value).is_some() || value.live_font_bytes().is_some()
}
