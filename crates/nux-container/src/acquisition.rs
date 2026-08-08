//! Untrusted metadata that a host may use before package authentication.
//!
//! This intentionally excludes journey, screen, product, script, text-input,
//! and other execution metadata. The complete manifest remains authoritative
//! only after [`crate::verify_signature`] succeeds.

/// Version of the host acquisition contract, independent of the container
/// format version.
pub const NUX_ACQUISITION_CONTRACT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuxAcquisitionErrorCode {
    LimitExceeded,
    InvalidContainer,
    UnsupportedVersion,
    MissingMember,
    InvalidManifest,
    InvalidExternalAsset,
}

impl NuxAcquisitionErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LimitExceeded => "acquisition.limit_exceeded",
            Self::InvalidContainer => "acquisition.invalid_container",
            Self::UnsupportedVersion => "acquisition.unsupported_version",
            Self::MissingMember => "acquisition.missing_member",
            Self::InvalidManifest => "acquisition.invalid_manifest",
            Self::InvalidExternalAsset => "acquisition.invalid_external_asset",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuxAcquisitionMetadataV1 {
    pub contract_version: u32,
    pub package_version: u32,
    pub identity: NuxAcquisitionIdentity,
    pub external_assets: Vec<NuxAcquisitionExternalAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuxAcquisitionIdentity {
    pub experience_id: String,
    pub build_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuxAcquisitionAssetKind {
    Image,
    Font,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuxAcquisitionExternalAsset {
    pub kind: NuxAcquisitionAssetKind,
    pub asset_id: u32,
    pub unique_name: String,
    pub key: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub required: bool,
}
