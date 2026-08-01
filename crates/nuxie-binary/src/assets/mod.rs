pub(super) mod blob_asset;
pub(super) mod file_asset;
pub(super) mod file_asset_contents;
pub(super) mod file_asset_referencer;
pub(super) mod manifest_asset;
pub(super) mod shader_asset;

pub(crate) use file_asset::{cpp_file_assets_contains, normalize_file_asset_ids};
pub use file_asset_contents::RuntimeFileAssetContents;
pub use manifest_asset::RuntimeManifest;
pub(crate) use manifest_asset::validate_cpp_manifest_assets_with_budget;
#[cfg(test)]
pub(crate) use manifest_asset::{
    cpp_manifest_key, cpp_manifest_resolver_key, validate_cpp_manifest_asset_with_budget,
};
