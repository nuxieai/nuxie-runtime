use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::manifest::{AssetLocation, MemberInventoryEntry, MemberRole};
use crate::signature::SignatureEnvelopeV1;
use crate::{
    FORMAT_VERSION, MAGIC, ManifestSigner, NUX_ACQUISITION_CONTRACT_VERSION,
    NUX_MAX_ASSET_SOURCE_KEY_BYTES, NUX_MAX_ASSET_UNIQUE_NAME_BYTES, NUX_MAX_EXTERNAL_ASSET_BYTES,
    NUX_MAX_EXTERNAL_ASSET_TOTAL_BYTES, NUX_MAX_EXTERNAL_ASSETS, NUX_MAX_JOURNEY_BYTES,
    NUX_MAX_MANIFEST_BYTES, NUX_MAX_MEMBERS, NUX_MAX_PACKAGE_BYTES, NUX_MAX_SIGNATURE_BYTES,
    NuxAcquisitionAssetKind, NuxAcquisitionExternalAsset, NuxAcquisitionIdentity,
    NuxAcquisitionMetadataV1, NuxContainerError, NuxPackageManifestV1, Result,
};

const ALIGNMENT: u64 = 16;
const FIXED_HEADER_BYTES: usize = 16;
const SCENE_MEMBER: &str = "scene";
const JOURNEY_MEMBER: &str = "journey";
const MANIFEST_MEMBER: &str = "manifest";
const SIGNATURE_MEMBER: &str = "signature";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TocEntry<'a> {
    pub name: &'a str,
    pub offset: u64,
    pub length: u64,
}

#[derive(Debug)]
pub struct NuxPackage<'a> {
    bytes: &'a [u8],
    toc: Vec<TocEntry<'a>>,
    manifest: NuxPackageManifestV1,
    manifest_index: usize,
    signature_index: usize,
}

impl<'a> NuxPackage<'a> {
    pub fn manifest(&self) -> &NuxPackageManifestV1 {
        &self.manifest
    }

    pub fn toc(&self) -> &[TocEntry<'a>] {
        &self.toc
    }

    pub fn member(&self, name: &str) -> Option<&'a [u8]> {
        let entry = self.toc.iter().find(|entry| entry.name == name)?;
        member_slice(self.bytes, entry).ok()
    }

    pub fn manifest_bytes(&self) -> &'a [u8] {
        self.toc
            .get(self.manifest_index)
            .and_then(|entry| member_slice(self.bytes, entry).ok())
            .unwrap_or_default()
    }

    pub fn signature_bytes(&self) -> &'a [u8] {
        self.toc
            .get(self.signature_index)
            .and_then(|entry| member_slice(self.bytes, entry).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EmbeddedMember<'a> {
    pub name: &'a str,
    pub bytes: &'a [u8],
}

pub enum SignatureSource<'a> {
    Signer(&'a dyn ManifestSigner),
    Precomputed(&'a SignatureEnvelopeV1),
}

pub struct NuxPackageModel<'a> {
    pub manifest: NuxPackageManifestV1,
    pub scene: &'a [u8],
    pub journey: &'a [u8],
    pub embedded_assets: Vec<EmbeddedMember<'a>>,
    pub signature: SignatureSource<'a>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcquisitionManifestV1 {
    version: u32,
    identity: AcquisitionIdentity,
    assets: AcquisitionAssets,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcquisitionIdentity {
    experience_id: String,
    build_id: String,
}

#[derive(Deserialize)]
struct AcquisitionAssets {
    images: Vec<AcquisitionAsset>,
    fonts: Vec<AcquisitionAsset>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcquisitionAsset {
    location: AcquisitionLocation,
    rive_asset_id: u64,
    rive_unique_name: String,
    sha256: String,
    size_bytes: u64,
    required: bool,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum AcquisitionLocation {
    External {
        key: String,
    },
    Embedded {
        #[serde(rename = "member")]
        _member: String,
    },
}

/// Reads only the untrusted metadata required to acquire external assets.
///
/// This validates the bounded container envelope and required member identity,
/// but deliberately does not decode the journey or expose product/runtime
/// metadata. [`read_package`] and [`crate::verify_signature`] remain the
/// authoritative post-acquisition path.
pub fn read_acquisition_metadata(bytes: &[u8]) -> Result<NuxAcquisitionMetadataV1> {
    enforce_package_limit(bytes)?;
    let (toc, toc_end) = parse_toc(bytes)?;
    let manifest_index = required_member_index(&toc, MANIFEST_MEMBER)?;
    required_member_index(&toc, SIGNATURE_MEMBER)?;
    required_member_index(&toc, SCENE_MEMBER)?;
    required_member_index(&toc, JOURNEY_MEMBER)?;
    validate_ranges_and_padding(bytes, &toc, toc_end)?;
    enforce_named_member_limits(&toc)?;

    let manifest_bytes = member_slice(
        bytes,
        toc.get(manifest_index)
            .ok_or(NuxContainerError::MissingMember(MANIFEST_MEMBER))?,
    )?;
    let manifest: AcquisitionManifestV1 =
        serde_json::from_slice(manifest_bytes).map_err(NuxContainerError::ManifestJson)?;
    if manifest.version != FORMAT_VERSION {
        return Err(NuxContainerError::InvalidManifest(format!(
            "manifest version {} is not {FORMAT_VERSION}",
            manifest.version
        )));
    }
    if manifest.identity.experience_id.is_empty() || manifest.identity.build_id.is_empty() {
        return Err(NuxContainerError::InvalidManifest(
            "acquisition identity must not be empty".to_owned(),
        ));
    }

    let mut external_assets = Vec::new();
    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    let mut total_bytes = 0u64;
    for (kind, asset) in manifest
        .assets
        .images
        .into_iter()
        .map(|asset| (NuxAcquisitionAssetKind::Image, asset))
        .chain(
            manifest
                .assets
                .fonts
                .into_iter()
                .map(|asset| (NuxAcquisitionAssetKind::Font, asset)),
        )
    {
        let key = match &asset.location {
            AcquisitionLocation::External { key } => key.clone(),
            AcquisitionLocation::Embedded { .. } => continue,
        };
        let asset_id = u32::try_from(asset.rive_asset_id).map_err(|_| {
            NuxContainerError::InvalidAsset("external asset id does not fit in u32".to_owned())
        })?;
        if !ids.insert(asset_id) || !names.insert(asset.rive_unique_name.clone()) {
            return Err(NuxContainerError::InvalidAsset(
                "external asset ids and unique names must be unique".to_owned(),
            ));
        }
        validate_acquisition_asset(&key, &asset)?;
        total_bytes = total_bytes
            .checked_add(asset.size_bytes)
            .ok_or(NuxContainerError::SizeOverflow)?;
        external_assets.push(NuxAcquisitionExternalAsset {
            kind,
            asset_id,
            unique_name: asset.rive_unique_name,
            key,
            sha256: asset.sha256,
            size_bytes: asset.size_bytes,
            required: asset.required,
        });
    }
    let asset_count =
        u32::try_from(external_assets.len()).map_err(|_| NuxContainerError::SizeOverflow)?;
    if asset_count > NUX_MAX_EXTERNAL_ASSETS {
        return Err(NuxContainerError::MemberTooLarge {
            name: "external asset count".to_owned(),
            actual: u64::from(asset_count),
            max: u64::from(NUX_MAX_EXTERNAL_ASSETS),
        });
    }
    if total_bytes > NUX_MAX_EXTERNAL_ASSET_TOTAL_BYTES {
        return Err(NuxContainerError::MemberTooLarge {
            name: "aggregate external assets".to_owned(),
            actual: total_bytes,
            max: NUX_MAX_EXTERNAL_ASSET_TOTAL_BYTES,
        });
    }

    Ok(NuxAcquisitionMetadataV1 {
        contract_version: NUX_ACQUISITION_CONTRACT_VERSION,
        package_version: manifest.version,
        identity: NuxAcquisitionIdentity {
            experience_id: manifest.identity.experience_id,
            build_id: manifest.identity.build_id,
        },
        external_assets,
    })
}

fn validate_acquisition_asset(key: &str, asset: &AcquisitionAsset) -> Result<()> {
    let name_bytes =
        u64::try_from(asset.rive_unique_name.len()).map_err(|_| NuxContainerError::SizeOverflow)?;
    let key_bytes = u64::try_from(key.len()).map_err(|_| NuxContainerError::SizeOverflow)?;
    if asset.rive_unique_name.is_empty() || name_bytes > NUX_MAX_ASSET_UNIQUE_NAME_BYTES {
        return Err(NuxContainerError::InvalidAsset(
            "external asset unique name is invalid".to_owned(),
        ));
    }
    if key.is_empty() || key_bytes > NUX_MAX_ASSET_SOURCE_KEY_BYTES {
        return Err(NuxContainerError::InvalidAsset(
            "external asset source key is invalid".to_owned(),
        ));
    }
    if asset.size_bytes > NUX_MAX_EXTERNAL_ASSET_BYTES {
        return Err(NuxContainerError::MemberTooLarge {
            name: key.to_owned(),
            actual: asset.size_bytes,
            max: NUX_MAX_EXTERNAL_ASSET_BYTES,
        });
    }
    if asset.sha256.len() != 64
        || !asset
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(NuxContainerError::InvalidAsset(asset.sha256.clone()));
    }
    let Some(rest) = key.strip_prefix("assets/sha256/") else {
        return Err(NuxContainerError::InvalidAsset(key.to_owned()));
    };
    let Some((hash, extension)) = rest.rsplit_once('.') else {
        return Err(NuxContainerError::InvalidAsset(key.to_owned()));
    };
    if hash != asset.sha256 || !matches!(extension, "png" | "jpg" | "jpeg" | "webp" | "ttf" | "otf")
    {
        return Err(NuxContainerError::InvalidAsset(key.to_owned()));
    }
    Ok(())
}

pub fn read_package(bytes: &[u8]) -> Result<NuxPackage<'_>> {
    enforce_package_limit(bytes)?;
    let (toc, toc_end) = parse_toc(bytes)?;
    let manifest_index = required_member_index(&toc, MANIFEST_MEMBER)?;
    let signature_index = required_member_index(&toc, SIGNATURE_MEMBER)?;
    validate_ranges_and_padding(bytes, &toc, toc_end)?;
    enforce_named_member_limits(&toc)?;

    let manifest_bytes = member_slice(
        bytes,
        toc.get(manifest_index)
            .ok_or(NuxContainerError::MissingMember(MANIFEST_MEMBER))?,
    )?;
    let manifest: NuxPackageManifestV1 =
        serde_json::from_slice(manifest_bytes).map_err(NuxContainerError::ManifestJson)?;
    manifest.validate_structure()?;
    validate_inventory(bytes, &toc, &manifest)?;
    validate_scene_and_journey(bytes, &toc, &manifest)?;
    validate_embedded_assets(&toc, &manifest)?;

    Ok(NuxPackage {
        bytes,
        toc,
        manifest,
        manifest_index,
        signature_index,
    })
}

pub fn write_package(model: &NuxPackageModel<'_>) -> Result<Vec<u8>> {
    let journey_value: Value =
        serde_json::from_slice(model.journey).map_err(NuxContainerError::JourneyJson)?;
    let journey_bytes =
        serde_json::to_vec(&journey_value).map_err(|_| NuxContainerError::JourneySerialization)?;

    let mut manifest = model.manifest.clone();
    manifest.scene.member = SCENE_MEMBER.to_owned();
    manifest.scene.sha256 = sha256_hex(model.scene);
    manifest.scene.size_bytes = usize_to_u64(model.scene.len())?;
    manifest.journey.member = JOURNEY_MEMBER.to_owned();
    manifest.journey.sha256 = sha256_hex(&journey_bytes);
    manifest.journey.size_bytes = usize_to_u64(journey_bytes.len())?;

    let mut embedded = model.embedded_assets.clone();
    embedded.sort_by(|left, right| left.name.cmp(right.name));
    update_embedded_asset_declarations(&mut manifest, &embedded)?;
    manifest.members = build_inventory(&manifest, model.scene, &journey_bytes, &embedded)?;

    let manifest_bytes =
        serde_json::to_vec(&manifest).map_err(|_| NuxContainerError::ManifestSerialization)?;
    let envelope = match model.signature {
        SignatureSource::Signer(signer) => signer.sign_manifest(&manifest_bytes),
        SignatureSource::Precomputed(envelope) => envelope.clone(),
    };
    let signature_bytes =
        serde_json::to_vec(&envelope).map_err(|_| NuxContainerError::SignatureSerialization)?;

    let mut members = Vec::with_capacity(embedded.len().saturating_add(4));
    members.push((MANIFEST_MEMBER, manifest_bytes.as_slice()));
    members.push((SIGNATURE_MEMBER, signature_bytes.as_slice()));
    members.push((SCENE_MEMBER, model.scene));
    members.push((JOURNEY_MEMBER, journey_bytes.as_slice()));
    members.extend(embedded.iter().map(|member| (member.name, member.bytes)));

    encode_container(&members)
}

pub fn validate_nux_roundtrip(bytes: &[u8]) -> Result<()> {
    let package = read_package(bytes)?;
    let envelope: SignatureEnvelopeV1 = serde_json::from_slice(package.signature_bytes())
        .map_err(|_| NuxContainerError::NonCanonicalPackage)?;
    let scene = package
        .member(SCENE_MEMBER)
        .ok_or(NuxContainerError::MissingMember(SCENE_MEMBER))?;
    let journey = package
        .member(JOURNEY_MEMBER)
        .ok_or(NuxContainerError::MissingMember(JOURNEY_MEMBER))?;
    let mut embedded_names = HashSet::new();
    let embedded_assets =
        package
            .manifest()
            .assets
            .images
            .iter()
            .filter_map(|asset| match &asset.location {
                AssetLocation::Embedded { member } => Some(member.as_str()),
                AssetLocation::External { .. } => None,
            })
            .chain(package.manifest().assets.fonts.iter().filter_map(
                |asset| match &asset.location {
                    AssetLocation::Embedded { member } => Some(member.as_str()),
                    AssetLocation::External { .. } => None,
                },
            ))
            .filter(|name| embedded_names.insert(*name))
            .map(|name| {
                package
                    .member(name)
                    .map(|member_bytes| EmbeddedMember {
                        name,
                        bytes: member_bytes,
                    })
                    .ok_or(NuxContainerError::MemberSetMismatch)
            })
            .collect::<Result<Vec<_>>>()?;
    let model = NuxPackageModel {
        manifest: package.manifest().clone(),
        scene,
        journey,
        embedded_assets,
        signature: SignatureSource::Precomputed(&envelope),
    };
    let rewritten = write_package(&model)?;
    if rewritten == bytes {
        Ok(())
    } else {
        Err(NuxContainerError::NonCanonicalPackage)
    }
}

fn enforce_package_limit(bytes: &[u8]) -> Result<()> {
    let actual = usize_to_u64(bytes.len())?;
    if actual > NUX_MAX_PACKAGE_BYTES {
        return Err(NuxContainerError::PackageTooLarge {
            actual,
            max: NUX_MAX_PACKAGE_BYTES,
        });
    }
    Ok(())
}

fn parse_toc(bytes: &[u8]) -> Result<(Vec<TocEntry<'_>>, usize)> {
    let magic = bytes
        .get(..MAGIC.len())
        .ok_or(NuxContainerError::Truncated("magic"))?;
    if magic != MAGIC {
        return Err(NuxContainerError::BadMagic);
    }
    let version = read_u32(bytes, 8, "version")?;
    if version != FORMAT_VERSION {
        return Err(NuxContainerError::UnsupportedVersion(version));
    }
    let count = read_u32(bytes, 12, "member count")?;
    if count > NUX_MAX_MEMBERS {
        return Err(NuxContainerError::TooManyMembers {
            actual: count,
            max: NUX_MAX_MEMBERS,
        });
    }

    let capacity = usize::try_from(count).map_err(|_| NuxContainerError::SizeOverflow)?;
    let mut toc = Vec::with_capacity(capacity);
    let mut names = HashSet::with_capacity(capacity);
    let mut cursor = FIXED_HEADER_BYTES;
    for _ in 0..count {
        let name_len = usize::from(read_u16(bytes, cursor, "member name length")?);
        cursor = cursor
            .checked_add(2)
            .ok_or(NuxContainerError::SizeOverflow)?;
        let name_end = cursor
            .checked_add(name_len)
            .ok_or(NuxContainerError::SizeOverflow)?;
        let name_bytes = bytes
            .get(cursor..name_end)
            .ok_or(NuxContainerError::Truncated("member name"))?;
        let name = std::str::from_utf8(name_bytes)
            .map_err(|_| NuxContainerError::InvalidMemberNameUtf8)?;
        if !names.insert(name) {
            return Err(NuxContainerError::DuplicateMemberName(name.to_owned()));
        }
        cursor = name_end;
        let offset = read_u64(bytes, cursor, "member offset")?;
        cursor = cursor
            .checked_add(8)
            .ok_or(NuxContainerError::SizeOverflow)?;
        let length = read_u64(bytes, cursor, "member length")?;
        cursor = cursor
            .checked_add(8)
            .ok_or(NuxContainerError::SizeOverflow)?;
        toc.push(TocEntry {
            name,
            offset,
            length,
        });
    }
    Ok((toc, cursor))
}

fn validate_ranges_and_padding(bytes: &[u8], toc: &[TocEntry<'_>], toc_end: usize) -> Result<()> {
    let toc_end_u64 = usize_to_u64(toc_end)?;
    let package_len = usize_to_u64(bytes.len())?;
    let mut ordered = toc.to_vec();
    ordered.sort_by_key(|entry| entry.offset);

    let mut previous_name: Option<&str> = None;
    let mut previous_end = toc_end_u64;
    for entry in &ordered {
        if entry.offset % ALIGNMENT != 0 {
            return Err(NuxContainerError::UnalignedMember {
                name: entry.name.to_owned(),
                offset: entry.offset,
            });
        }
        if entry.offset < toc_end_u64 {
            return Err(NuxContainerError::MemberBeforePayloads {
                name: entry.name.to_owned(),
            });
        }
        let end = entry.offset.checked_add(entry.length).ok_or_else(|| {
            NuxContainerError::MemberOutOfBounds {
                name: entry.name.to_owned(),
            }
        })?;
        if end > package_len {
            return Err(NuxContainerError::MemberOutOfBounds {
                name: entry.name.to_owned(),
            });
        }
        if entry.offset < previous_end {
            return Err(NuxContainerError::OverlappingMembers {
                first: previous_name.unwrap_or("table of contents").to_owned(),
                second: entry.name.to_owned(),
            });
        }
        validate_zero_range(bytes, previous_end, entry.offset)?;
        previous_name = Some(entry.name);
        previous_end = end;
    }
    validate_zero_range(bytes, previous_end, package_len)
}

fn validate_zero_range(bytes: &[u8], start: u64, end: u64) -> Result<()> {
    let start = usize::try_from(start).map_err(|_| NuxContainerError::SizeOverflow)?;
    let end = usize::try_from(end).map_err(|_| NuxContainerError::SizeOverflow)?;
    let padding = bytes
        .get(start..end)
        .ok_or(NuxContainerError::Truncated("padding"))?;
    if padding.iter().any(|byte| *byte != 0) {
        return Err(NuxContainerError::NonZeroPadding);
    }
    Ok(())
}

fn enforce_named_member_limits(toc: &[TocEntry<'_>]) -> Result<()> {
    for entry in toc {
        if entry.name == SIGNATURE_MEMBER && entry.length == 0 {
            return Err(NuxContainerError::ZeroLengthMember(
                SIGNATURE_MEMBER.to_owned(),
            ));
        }
        let max = match entry.name {
            MANIFEST_MEMBER => Some(NUX_MAX_MANIFEST_BYTES),
            JOURNEY_MEMBER => Some(NUX_MAX_JOURNEY_BYTES),
            SIGNATURE_MEMBER => Some(NUX_MAX_SIGNATURE_BYTES),
            _ => None,
        };
        if let Some(max) = max
            && entry.length > max
        {
            return Err(NuxContainerError::MemberTooLarge {
                name: entry.name.to_owned(),
                actual: entry.length,
                max,
            });
        }
    }
    Ok(())
}

fn validate_inventory(
    bytes: &[u8],
    toc: &[TocEntry<'_>],
    manifest: &NuxPackageManifestV1,
) -> Result<()> {
    let toc_names: HashSet<&str> = toc
        .iter()
        .filter(|entry| entry.name != SIGNATURE_MEMBER)
        .map(|entry| entry.name)
        .collect();
    let inventory_names: HashSet<&str> = manifest
        .members
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    if toc_names != inventory_names {
        return Err(NuxContainerError::MemberSetMismatch);
    }

    let inventory: HashMap<&str, &MemberInventoryEntry> = manifest
        .members
        .iter()
        .map(|entry| (entry.name.as_str(), entry))
        .collect();
    for entry in toc.iter().filter(|entry| entry.name != SIGNATURE_MEMBER) {
        let declared = inventory
            .get(entry.name)
            .ok_or(NuxContainerError::MemberSetMismatch)?;
        if entry.name == MANIFEST_MEMBER {
            continue;
        }
        if entry.length != declared.size_bytes {
            return Err(NuxContainerError::SizeMismatch {
                name: entry.name.to_owned(),
                expected: declared.size_bytes,
                actual: entry.length,
            });
        }
        let payload = member_slice(bytes, entry)?;
        if sha256_hex(payload) != declared.sha256 {
            return Err(NuxContainerError::DigestMismatch(entry.name.to_owned()));
        }
    }
    Ok(())
}

fn validate_scene_and_journey(
    bytes: &[u8],
    toc: &[TocEntry<'_>],
    manifest: &NuxPackageManifestV1,
) -> Result<()> {
    if manifest.scene.member != SCENE_MEMBER || manifest.journey.member != JOURNEY_MEMBER {
        return Err(NuxContainerError::InvalidManifest(
            "v1 scene and journey member names must be literal `scene` and `journey`".to_owned(),
        ));
    }
    let scene_entry = toc
        .iter()
        .find(|entry| entry.name == manifest.scene.member)
        .ok_or(NuxContainerError::MissingMember(SCENE_MEMBER))?;
    let scene = member_slice(bytes, scene_entry)?;
    require_inventory_role(manifest, scene_entry.name, MemberRole::Scene)?;
    validate_descriptor(
        scene_entry,
        &manifest.scene.sha256,
        manifest.scene.size_bytes,
        scene,
    )?;
    if !scene.starts_with(b"RIVE") {
        return Err(NuxContainerError::InvalidSceneHeader);
    }

    let journey_entry = toc
        .iter()
        .find(|entry| entry.name == manifest.journey.member)
        .ok_or(NuxContainerError::MissingMember(JOURNEY_MEMBER))?;
    let journey = member_slice(bytes, journey_entry)?;
    require_inventory_role(manifest, journey_entry.name, MemberRole::Journey)?;
    validate_descriptor(
        journey_entry,
        &manifest.journey.sha256,
        manifest.journey.size_bytes,
        journey,
    )?;
    let value: Value = serde_json::from_slice(journey).map_err(NuxContainerError::JourneyJson)?;
    let object = value
        .as_object()
        .ok_or(NuxContainerError::JourneyNotObject)?;
    let schema_version = object.get("schemaVersion").and_then(Value::as_u64);
    if schema_version != Some(u64::from(manifest.journey.schema_version)) {
        return Err(NuxContainerError::JourneySchemaVersionMismatch);
    }
    Ok(())
}

fn validate_descriptor(
    entry: &TocEntry<'_>,
    expected_hash: &str,
    expected_size: u64,
    bytes: &[u8],
) -> Result<()> {
    if entry.length != expected_size {
        return Err(NuxContainerError::SizeMismatch {
            name: entry.name.to_owned(),
            expected: expected_size,
            actual: entry.length,
        });
    }
    if sha256_hex(bytes) != expected_hash {
        return Err(NuxContainerError::DigestMismatch(entry.name.to_owned()));
    }
    Ok(())
}

fn validate_embedded_assets(toc: &[TocEntry<'_>], manifest: &NuxPackageManifestV1) -> Result<()> {
    let inventory: HashMap<&str, &MemberInventoryEntry> = manifest
        .members
        .iter()
        .map(|entry| (entry.name.as_str(), entry))
        .collect();
    for (location, sha256, size_bytes) in manifest
        .assets
        .images
        .iter()
        .map(|asset| (&asset.location, asset.sha256.as_str(), asset.size_bytes))
        .chain(
            manifest
                .assets
                .fonts
                .iter()
                .map(|asset| (&asset.location, asset.sha256.as_str(), asset.size_bytes)),
        )
    {
        let AssetLocation::Embedded { member } = location else {
            continue;
        };
        if !toc.iter().any(|entry| entry.name == member) {
            return Err(NuxContainerError::InvalidAsset(format!(
                "embedded member `{member}` does not exist"
            )));
        }
        let Some(declared) = inventory.get(member.as_str()) else {
            return Err(NuxContainerError::InvalidAsset(format!(
                "embedded member `{member}` is absent from inventory"
            )));
        };
        if declared.role != MemberRole::Asset
            || declared.sha256 != sha256
            || declared.size_bytes != size_bytes
        {
            return Err(NuxContainerError::InvalidAsset(format!(
                "embedded member `{member}` disagrees with its inventory entry"
            )));
        }
    }
    Ok(())
}

fn require_inventory_role(
    manifest: &NuxPackageManifestV1,
    name: &str,
    expected: MemberRole,
) -> Result<()> {
    let role = manifest
        .members
        .iter()
        .find(|member| member.name == name)
        .map(|member| member.role);
    if role != Some(expected) {
        return Err(NuxContainerError::InvalidManifest(format!(
            "member `{name}` has the wrong role"
        )));
    }
    Ok(())
}

fn update_embedded_asset_declarations(
    manifest: &mut NuxPackageManifestV1,
    embedded: &[EmbeddedMember<'_>],
) -> Result<()> {
    for asset in &mut manifest.assets.images {
        if let AssetLocation::Embedded { member } = &asset.location
            && let Some(payload) = embedded.iter().find(|payload| payload.name == member)
        {
            asset.sha256 = sha256_hex(payload.bytes);
            asset.size_bytes = usize_to_u64(payload.bytes.len())?;
        }
    }
    for asset in &mut manifest.assets.fonts {
        if let AssetLocation::Embedded { member } = &asset.location
            && let Some(payload) = embedded.iter().find(|payload| payload.name == member)
        {
            asset.sha256 = sha256_hex(payload.bytes);
            asset.size_bytes = usize_to_u64(payload.bytes.len())?;
        }
    }
    Ok(())
}

fn build_inventory(
    manifest: &NuxPackageManifestV1,
    scene: &[u8],
    journey: &[u8],
    embedded: &[EmbeddedMember<'_>],
) -> Result<Vec<MemberInventoryEntry>> {
    let mut members = vec![
        MemberInventoryEntry {
            name: MANIFEST_MEMBER.to_owned(),
            role: MemberRole::Manifest,
            sha256: "0".repeat(64),
            size_bytes: 0,
            content_type: "application/json".to_owned(),
        },
        inventory_entry(
            SCENE_MEMBER,
            MemberRole::Scene,
            scene,
            "application/vnd.nuxie.scene",
        )?,
        inventory_entry(
            JOURNEY_MEMBER,
            MemberRole::Journey,
            journey,
            "application/json",
        )?,
    ];
    for payload in embedded {
        let content_type = embedded_content_type(manifest, payload.name);
        members.push(inventory_entry(
            payload.name,
            MemberRole::Asset,
            payload.bytes,
            content_type,
        )?);
    }
    Ok(members)
}

fn embedded_content_type<'a>(manifest: &'a NuxPackageManifestV1, name: &str) -> &'a str {
    if let Some(asset) = manifest.assets.images.iter().find(
        |asset| matches!(&asset.location, AssetLocation::Embedded { member } if member == name),
    ) {
        return asset.content_type.as_str();
    }
    if let Some(asset) = manifest.assets.fonts.iter().find(
        |asset| matches!(&asset.location, AssetLocation::Embedded { member } if member == name),
    ) {
        return asset.content_type.as_str();
    }
    "application/octet-stream"
}

fn inventory_entry(
    name: &str,
    role: MemberRole,
    bytes: &[u8],
    content_type: &str,
) -> Result<MemberInventoryEntry> {
    Ok(MemberInventoryEntry {
        name: name.to_owned(),
        role,
        sha256: sha256_hex(bytes),
        size_bytes: usize_to_u64(bytes.len())?,
        content_type: content_type.to_owned(),
    })
}

fn encode_container(members: &[(&str, &[u8])]) -> Result<Vec<u8>> {
    let count = u32::try_from(members.len()).map_err(|_| NuxContainerError::SizeOverflow)?;
    if count > NUX_MAX_MEMBERS {
        return Err(NuxContainerError::TooManyMembers {
            actual: count,
            max: NUX_MAX_MEMBERS,
        });
    }
    let toc_bytes = members.iter().try_fold(0usize, |total, (name, _)| {
        let name_bytes = name.as_bytes();
        let _ = u16::try_from(name_bytes.len()).map_err(|_| NuxContainerError::SizeOverflow)?;
        total
            .checked_add(18)
            .and_then(|value| value.checked_add(name_bytes.len()))
            .ok_or(NuxContainerError::SizeOverflow)
    })?;
    let header_end = FIXED_HEADER_BYTES
        .checked_add(toc_bytes)
        .ok_or(NuxContainerError::SizeOverflow)?;
    let mut next_offset = align_up(usize_to_u64(header_end)?)?;
    let mut offsets = Vec::with_capacity(members.len());
    for (_, payload) in members {
        offsets.push(next_offset);
        next_offset = next_offset
            .checked_add(usize_to_u64(payload.len())?)
            .ok_or(NuxContainerError::SizeOverflow)?;
        next_offset = align_up(next_offset)?;
    }
    let final_size = members
        .last()
        .zip(offsets.last())
        .map(|((_, payload), offset)| {
            offset
                .checked_add(usize_to_u64(payload.len())?)
                .ok_or(NuxContainerError::SizeOverflow)
        })
        .transpose()?
        .unwrap_or_else(|| usize_to_u64(header_end).unwrap_or_default());
    if final_size > NUX_MAX_PACKAGE_BYTES {
        return Err(NuxContainerError::PackageTooLarge {
            actual: final_size,
            max: NUX_MAX_PACKAGE_BYTES,
        });
    }
    let final_size_usize =
        usize::try_from(final_size).map_err(|_| NuxContainerError::SizeOverflow)?;
    let mut output = Vec::with_capacity(final_size_usize);
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    output.extend_from_slice(&count.to_le_bytes());
    for ((name, payload), offset) in members.iter().zip(&offsets) {
        let name_len = u16::try_from(name.len()).map_err(|_| NuxContainerError::SizeOverflow)?;
        output.extend_from_slice(&name_len.to_le_bytes());
        output.extend_from_slice(name.as_bytes());
        output.extend_from_slice(&offset.to_le_bytes());
        output.extend_from_slice(&usize_to_u64(payload.len())?.to_le_bytes());
    }
    for ((_, payload), offset) in members.iter().zip(&offsets) {
        let offset = usize::try_from(*offset).map_err(|_| NuxContainerError::SizeOverflow)?;
        output.resize(offset, 0);
        output.extend_from_slice(payload);
    }
    Ok(output)
}

fn required_member_index(toc: &[TocEntry<'_>], name: &'static str) -> Result<usize> {
    toc.iter()
        .position(|entry| entry.name == name)
        .ok_or(NuxContainerError::MissingMember(name))
}

fn member_slice<'a>(bytes: &'a [u8], entry: &TocEntry<'_>) -> Result<&'a [u8]> {
    let start =
        usize::try_from(entry.offset).map_err(|_| NuxContainerError::MemberOutOfBounds {
            name: entry.name.to_owned(),
        })?;
    let end_u64 = entry.offset.checked_add(entry.length).ok_or_else(|| {
        NuxContainerError::MemberOutOfBounds {
            name: entry.name.to_owned(),
        }
    })?;
    let end = usize::try_from(end_u64).map_err(|_| NuxContainerError::MemberOutOfBounds {
        name: entry.name.to_owned(),
    })?;
    bytes
        .get(start..end)
        .ok_or_else(|| NuxContainerError::MemberOutOfBounds {
            name: entry.name.to_owned(),
        })
}

fn read_u16(bytes: &[u8], offset: usize, field: &'static str) -> Result<u16> {
    let end = offset
        .checked_add(2)
        .ok_or(NuxContainerError::SizeOverflow)?;
    let encoded = bytes
        .get(offset..end)
        .ok_or(NuxContainerError::Truncated(field))?;
    let array: [u8; 2] = encoded
        .try_into()
        .map_err(|_| NuxContainerError::Truncated(field))?;
    Ok(u16::from_le_bytes(array))
}

fn read_u32(bytes: &[u8], offset: usize, field: &'static str) -> Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or(NuxContainerError::SizeOverflow)?;
    let encoded = bytes
        .get(offset..end)
        .ok_or(NuxContainerError::Truncated(field))?;
    let array: [u8; 4] = encoded
        .try_into()
        .map_err(|_| NuxContainerError::Truncated(field))?;
    Ok(u32::from_le_bytes(array))
}

fn read_u64(bytes: &[u8], offset: usize, field: &'static str) -> Result<u64> {
    let end = offset
        .checked_add(8)
        .ok_or(NuxContainerError::SizeOverflow)?;
    let encoded = bytes
        .get(offset..end)
        .ok_or(NuxContainerError::Truncated(field))?;
    let array: [u8; 8] = encoded
        .try_into()
        .map_err(|_| NuxContainerError::Truncated(field))?;
    Ok(u64::from_le_bytes(array))
}

fn align_up(value: u64) -> Result<u64> {
    value
        .checked_add(ALIGNMENT - 1)
        .and_then(|value| (value / ALIGNMENT).checked_mul(ALIGNMENT))
        .ok_or(NuxContainerError::SizeOverflow)
}

fn usize_to_u64(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| NuxContainerError::SizeOverflow)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
