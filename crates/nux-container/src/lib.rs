//! Reader, writer, and validator for the Nuxie `.nux` package format.
//!
//! This crate is the grammar authority for version 1 packages. Reading is
//! zero-copy for member payloads; the typed manifest is owned.

mod format;
mod manifest;
mod signature;

mod acquisition;

pub use acquisition::{
    NUX_ACQUISITION_CONTRACT_VERSION, NuxAcquisitionAssetKind, NuxAcquisitionErrorCode,
    NuxAcquisitionExternalAsset, NuxAcquisitionIdentity, NuxAcquisitionMetadataV1,
};

pub use format::{
    EmbeddedMember, NuxPackage, NuxPackageModel, SignatureSource, TocEntry,
    read_acquisition_metadata, read_package, validate_nux_roundtrip, write_package,
};
pub use manifest::{
    AssetLocation, Assets, Entry, FontAsset, FontContentType, FontFormat, FontStyle,
    FontStyleValue, Geometry, Identity, ImageAsset, ImageContentType, JourneyMember, LuauProducer,
    MemberInventoryEntry, MemberRole, NuxPackageManifestV1, Producer, SceneFormat, SceneMember,
    Screen, TextInput, TextInputStyle,
};
pub use signature::{
    ManifestSigner, SignatureEnvelopeV1, SignatureVerification, VerifiedScene, verify_signature,
};

pub mod test_support;

use thiserror::Error;

pub const NUX_MAX_PACKAGE_BYTES: u64 = 64 * 1024 * 1024;
pub const NUX_MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
pub const NUX_MAX_JOURNEY_BYTES: u64 = 8 * 1024 * 1024;
pub const NUX_MAX_SIGNATURE_BYTES: u64 = 64 * 1024;
pub const NUX_MAX_EXTERNAL_ASSET_BYTES: u64 = 32 * 1024 * 1024;
pub const NUX_MAX_EXTERNAL_ASSETS: u32 = 1024;
pub const NUX_MAX_EXTERNAL_ASSET_TOTAL_BYTES: u64 = 128 * 1024 * 1024;
pub const NUX_MAX_ASSET_UNIQUE_NAME_BYTES: u64 = 4 * 1024;
pub const NUX_MAX_ASSET_SOURCE_KEY_BYTES: u64 = 4 * 1024 * 1024;
pub const NUX_MAX_MEMBERS: u32 = 4096;

/// Known scene/runtime capabilities a package may require.
///
/// Version 1 intentionally defines none.
pub const KNOWN_CAPABILITIES: &[&str] = &[];

/// First key in the Nuxie-owned scene type/property key allocation band.
pub const NUXIE_SCENE_KEY_BAND_MIN: u16 = 60_000;

pub(crate) const MAGIC: &[u8; 8] = b"\x89NUX\r\n\x1a\n";
pub(crate) const FORMAT_VERSION: u32 = 1;

pub type Result<T> = std::result::Result<T, NuxContainerError>;

#[derive(Debug, Error)]
pub enum NuxContainerError {
    #[error("package is {actual} bytes, exceeding the {max}-byte limit")]
    PackageTooLarge { actual: u64, max: u64 },
    #[error("bad .nux magic")]
    BadMagic,
    #[error("unsupported .nux version {0}")]
    UnsupportedVersion(u32),
    #[error("member count {actual} exceeds the limit of {max}")]
    TooManyMembers { actual: u32, max: u32 },
    #[error("container is truncated while reading {0}")]
    Truncated(&'static str),
    #[error("member name is not valid UTF-8")]
    InvalidMemberNameUtf8,
    #[error("duplicate member name `{0}`")]
    DuplicateMemberName(String),
    #[error("member `{name}` offset {offset} is not 16-byte aligned")]
    UnalignedMember { name: String, offset: u64 },
    #[error("member `{name}` begins inside the header or table of contents")]
    MemberBeforePayloads { name: String },
    #[error("member `{name}` range is outside the package")]
    MemberOutOfBounds { name: String },
    #[error("member `{first}` overlaps member `{second}`")]
    OverlappingMembers { first: String, second: String },
    #[error("non-zero byte in container padding")]
    NonZeroPadding,
    #[error("required member `{0}` is missing")]
    MissingMember(&'static str),
    #[error("member `{name}` is {actual} bytes, exceeding the {max}-byte limit")]
    MemberTooLarge { name: String, actual: u64, max: u64 },
    #[error("member `{0}` cannot be empty")]
    ZeroLengthMember(String),
    #[error("manifest JSON is invalid: {0}")]
    ManifestJson(#[source] serde_json::Error),
    #[error("manifest invariant failed: {0}")]
    InvalidManifest(String),
    #[error("manifest member inventory does not exactly match the table of contents")]
    MemberSetMismatch,
    #[error("member `{name}` has size {actual}, expected {expected}")]
    SizeMismatch {
        name: String,
        expected: u64,
        actual: u64,
    },
    #[error("member `{0}` sha256 digest does not match the manifest")]
    DigestMismatch(String),
    #[error("scene member does not begin with `RIVE`")]
    InvalidSceneHeader,
    #[error("journey JSON is invalid: {0}")]
    JourneyJson(#[source] serde_json::Error),
    #[error("journey JSON cannot be serialized")]
    JourneySerialization,
    #[error("journey top level must be an object")]
    JourneyNotObject,
    #[error("journey schemaVersion does not equal the manifest value")]
    JourneySchemaVersionMismatch,
    #[error("unknown required capability `{0}`")]
    UnknownCapability(String),
    #[error("asset declaration is invalid: {0}")]
    InvalidAsset(String),
    #[error("signature envelope cannot be serialized")]
    SignatureSerialization,
    #[error("manifest cannot be serialized")]
    ManifestSerialization,
    #[error("container size arithmetic overflowed")]
    SizeOverflow,
    #[error("package is not in canonical writer form")]
    NonCanonicalPackage,
}

impl NuxContainerError {
    /// Stable cross-SDK classification for failures in the untrusted
    /// acquisition phase. Detailed messages remain diagnostic only.
    pub const fn acquisition_code(&self) -> NuxAcquisitionErrorCode {
        match self {
            Self::PackageTooLarge { .. }
            | Self::TooManyMembers { .. }
            | Self::MemberTooLarge { .. }
            | Self::SizeOverflow => NuxAcquisitionErrorCode::LimitExceeded,
            Self::UnsupportedVersion(_) => NuxAcquisitionErrorCode::UnsupportedVersion,
            Self::MissingMember(_) | Self::ZeroLengthMember(_) => {
                NuxAcquisitionErrorCode::MissingMember
            }
            Self::ManifestJson(_) | Self::InvalidManifest(_) => {
                NuxAcquisitionErrorCode::InvalidManifest
            }
            Self::InvalidAsset(_) => NuxAcquisitionErrorCode::InvalidExternalAsset,
            _ => NuxAcquisitionErrorCode::InvalidContainer,
        }
    }
}
