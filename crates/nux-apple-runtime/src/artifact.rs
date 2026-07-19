//! Authentication and declaration validation for the current Nuxie flow
//! artifact adapter.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ed25519_dalek::{Signature, VerifyingKey};
use nuxie::File;
use serde::Deserialize;
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const SIGNATURE_FILE_NAME: &str = "nuxie-manifest.json";
const SIGNATURE_ALGORITHM: &str = "ed25519";
pub(crate) const MAX_EXTERNAL_ASSET_COUNT: usize = 1_024;
const MAX_IDENTITY_BYTE_LENGTH: usize = 4_096;
const MAX_ASSET_UNIQUE_NAME_BYTE_LENGTH: usize = 4_096;
const MAX_ASSET_SOURCE_KEY_BYTE_LENGTH: usize = 4_194_304;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedArtifactSigningKey {
    pub(crate) key_id: String,
    pub(crate) public_key: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExternalAssetKind {
    Image,
    Font,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExternalAssetInput {
    Supplied {
        kind: ExternalAssetKind,
        asset_id: u32,
        unique_name: String,
        source_key: String,
        expected_sha256: String,
        required: bool,
        bytes: Vec<u8>,
    },
    Omitted {
        kind: ExternalAssetKind,
        asset_id: u32,
        unique_name: String,
        source_key: String,
        expected_sha256: String,
        required: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowArtifactImportInput {
    pub(crate) expected_flow_id: String,
    pub(crate) expected_build_id: String,
    pub(crate) artifact_bytes: Vec<u8>,
    pub(crate) manifest_bytes: Vec<u8>,
    pub(crate) signature_envelope_bytes: Option<Vec<u8>>,
    pub(crate) selected_key: Option<SelectedArtifactSigningKey>,
    pub(crate) external_assets: Vec<ExternalAssetInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VisualOnlyReason {
    MissingSignature,
    MalformedSignature,
    UnsupportedSignature,
    MissingSelectedKey,
    KeyMismatch,
    InvalidSelectedKey,
    InvalidSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArtifactAuthorization {
    Authenticated { key_id: String },
    VisualOnly { reason: VisualOnlyReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArtifactDiagnosticSeverity {
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArtifactDiagnostic {
    pub(crate) severity: ArtifactDiagnosticSeverity,
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedExternalAsset {
    pub(crate) kind: ExternalAssetKind,
    pub(crate) asset_id: u32,
    pub(crate) unique_name: String,
    pub(crate) source_key: String,
    pub(crate) expected_sha256: String,
    pub(crate) required: bool,
    pub(crate) bytes: Option<Vec<u8>>,
}

pub(crate) struct ValidatedFlowArtifactImport {
    pub(crate) file: File,
    pub(crate) authorization: ArtifactAuthorization,
    pub(crate) external_assets: Vec<ValidatedExternalAsset>,
    pub(crate) diagnostics: Vec<ArtifactDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArtifactImportError {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

impl std::fmt::Display for ArtifactImportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ArtifactImportError {}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlowArtifactManifest {
    version: u32,
    flow_id: String,
    build_id: String,
    renderer: String,
    riv: FlowArtifactRivDeclaration,
    #[serde(default)]
    assets: FlowArtifactAssetDeclarations,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlowArtifactRivDeclaration {
    #[allow(dead_code)]
    path: String,
    sha256: String,
    size_bytes: u64,
}

#[derive(Default, Deserialize)]
struct FlowArtifactAssetDeclarations {
    #[serde(default)]
    images: Vec<FlowArtifactImageDeclaration>,
    #[serde(default)]
    fonts: Vec<FlowArtifactFontDeclaration>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlowArtifactImageDeclaration {
    rive_asset_id: u64,
    rive_unique_name: String,
    source_asset_key: String,
    sha256: String,
    required: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlowArtifactFontDeclaration {
    rive_asset_id: u64,
    rive_unique_name: String,
    request_key: String,
    sha256: String,
    size_bytes: u64,
    required: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlowManifestSignatureEnvelope {
    version: u32,
    signs: String,
    algorithm: String,
    key_id: String,
    signature_base64: String,
}

struct ExternalAssetDeclaration {
    kind: ExternalAssetKind,
    asset_id: u32,
    unique_name: String,
    source_key: String,
    sha256: String,
    size_bytes: Option<u64>,
    required: bool,
}

pub(crate) fn validate_flow_artifact_import(
    input: FlowArtifactImportInput,
) -> Result<ValidatedFlowArtifactImport, ArtifactImportError> {
    let manifest: FlowArtifactManifest = serde_json::from_slice(&input.manifest_bytes)
        .map_err(|error| import_error("artifact.manifest.invalid", error.to_string()))?;
    if manifest.version != 1 {
        return Err(import_error(
            "artifact.manifest.unsupported_version",
            format!("manifest version {} is not supported", manifest.version),
        ));
    }
    if manifest.renderer != "rive" {
        return Err(import_error(
            "artifact.manifest.unsupported_renderer",
            format!("manifest renderer '{}' is not supported", manifest.renderer),
        ));
    }
    validate_manifest_identity(
        &manifest.flow_id,
        MAX_IDENTITY_BYTE_LENGTH,
        "artifact.identity.invalid_flow_id",
        "flow ID",
    )?;
    validate_manifest_identity(
        &manifest.build_id,
        MAX_IDENTITY_BYTE_LENGTH,
        "artifact.identity.invalid_build_id",
        "build ID",
    )?;
    if manifest.flow_id != input.expected_flow_id {
        return Err(import_error(
            "artifact.identity.flow_mismatch",
            format!(
                "manifest flow '{}' does not match requested flow '{}'",
                manifest.flow_id, input.expected_flow_id
            ),
        ));
    }
    if manifest.build_id != input.expected_build_id {
        return Err(import_error(
            "artifact.identity.build_mismatch",
            format!(
                "manifest build '{}' does not match requested build '{}'",
                manifest.build_id, input.expected_build_id
            ),
        ));
    }

    validate_sha256(&manifest.riv.sha256, "artifact.riv.invalid_hash", "RIV")?;
    let actual_size = u64::try_from(input.artifact_bytes.len()).map_err(|_| {
        import_error(
            "artifact.riv.size_mismatch",
            "RIV byte length does not fit the manifest size domain",
        )
    })?;
    if manifest.riv.size_bytes != actual_size {
        return Err(import_error(
            "artifact.riv.size_mismatch",
            format!(
                "manifest declares {} RIV bytes, received {actual_size}",
                manifest.riv.size_bytes
            ),
        ));
    }
    let actual_hash = sha256_hex(&input.artifact_bytes);
    if !manifest.riv.sha256.eq_ignore_ascii_case(&actual_hash) {
        return Err(import_error(
            "artifact.riv.hash_mismatch",
            format!(
                "manifest RIV hash '{}' does not match '{actual_hash}'",
                manifest.riv.sha256
            ),
        ));
    }

    let declarations = external_asset_declarations(manifest.assets)?;
    let (authorization, authentication_diagnostic) = authenticate_manifest(
        &input.manifest_bytes,
        input.signature_envelope_bytes.as_deref(),
        input.selected_key.as_ref(),
    );
    let file = File::import(&input.artifact_bytes)
        .map_err(|error| import_error("artifact.riv.import_failed", error.to_string()))?;
    let mut diagnostics = Vec::new();
    if let Some(diagnostic) = authentication_diagnostic {
        diagnostics.push(diagnostic);
    }
    let external_assets =
        validate_external_assets(&file, declarations, input.external_assets, &mut diagnostics)?;
    Ok(ValidatedFlowArtifactImport {
        file,
        authorization,
        external_assets,
        diagnostics,
    })
}

fn external_asset_declarations(
    assets: FlowArtifactAssetDeclarations,
) -> Result<Vec<ExternalAssetDeclaration>, ArtifactImportError> {
    let declaration_count = assets
        .images
        .len()
        .checked_add(assets.fonts.len())
        .ok_or_else(|| {
            import_error(
                "artifact.asset.invalid_declaration",
                "asset declaration count overflowed",
            )
        })?;
    if declaration_count > MAX_EXTERNAL_ASSET_COUNT {
        return Err(import_error(
            "artifact.asset.too_many_declarations",
            format!(
                "manifest declares {declaration_count} assets; the limit is {MAX_EXTERNAL_ASSET_COUNT}"
            ),
        ));
    }
    let mut declarations = Vec::with_capacity(declaration_count);
    for image in assets.images {
        validate_asset_declaration_string(
            &image.rive_unique_name,
            MAX_ASSET_UNIQUE_NAME_BYTE_LENGTH,
            "unique name",
        )?;
        validate_asset_declaration_string(
            &image.source_asset_key,
            MAX_ASSET_SOURCE_KEY_BYTE_LENGTH,
            "source key",
        )?;
        declarations.push(ExternalAssetDeclaration {
            kind: ExternalAssetKind::Image,
            asset_id: manifest_asset_id(image.rive_asset_id, &image.rive_unique_name)?,
            unique_name: image.rive_unique_name,
            source_key: image.source_asset_key,
            sha256: validated_sha256(image.sha256)?,
            size_bytes: None,
            required: image.required,
        });
    }
    for font in assets.fonts {
        validate_asset_declaration_string(
            &font.rive_unique_name,
            MAX_ASSET_UNIQUE_NAME_BYTE_LENGTH,
            "unique name",
        )?;
        validate_asset_declaration_string(
            &font.request_key,
            MAX_ASSET_SOURCE_KEY_BYTE_LENGTH,
            "request key",
        )?;
        declarations.push(ExternalAssetDeclaration {
            kind: ExternalAssetKind::Font,
            asset_id: manifest_asset_id(font.rive_asset_id, &font.rive_unique_name)?,
            unique_name: font.rive_unique_name,
            source_key: font.request_key,
            sha256: validated_sha256(font.sha256)?,
            size_bytes: Some(font.size_bytes),
            required: font.required,
        });
    }

    let mut ids = BTreeSet::new();
    let mut unique_names = BTreeSet::new();
    for declaration in &declarations {
        if !ids.insert(declaration.asset_id) {
            return Err(import_error(
                "artifact.asset.duplicate_id",
                format!(
                    "manifest declares asset id {} more than once",
                    declaration.asset_id
                ),
            ));
        }
        if declaration.unique_name.is_empty() {
            return Err(import_error(
                "artifact.asset.invalid_declaration",
                format!(
                    "manifest asset {} has an empty unique name",
                    declaration.asset_id
                ),
            ));
        }
        if !unique_names.insert(declaration.unique_name.as_str()) {
            return Err(import_error(
                "artifact.asset.duplicate_unique_name",
                format!(
                    "manifest declares asset unique name '{}' more than once",
                    declaration.unique_name
                ),
            ));
        }
    }
    Ok(declarations)
}

fn validate_manifest_identity(
    value: &str,
    maximum_length: usize,
    code: &'static str,
    label: &str,
) -> Result<(), ArtifactImportError> {
    if !value.is_empty() && value.len() <= maximum_length {
        return Ok(());
    }
    Err(import_error(
        code,
        format!("manifest {label} must contain 1 through {maximum_length} UTF-8 bytes"),
    ))
}

fn validate_asset_declaration_string(
    value: &str,
    maximum_length: usize,
    label: &str,
) -> Result<(), ArtifactImportError> {
    if value.len() <= maximum_length {
        return Ok(());
    }
    Err(import_error(
        "artifact.asset.invalid_declaration",
        format!("manifest asset {label} exceeds {maximum_length} UTF-8 bytes"),
    ))
}

fn manifest_asset_id(value: u64, unique_name: &str) -> Result<u32, ArtifactImportError> {
    u32::try_from(value).map_err(|_| {
        import_error(
            "artifact.asset.invalid_declaration",
            format!("manifest asset '{unique_name}' id {value} does not fit in UInt32"),
        )
    })
}

fn validated_sha256(value: String) -> Result<String, ArtifactImportError> {
    validate_sha256(&value, "artifact.asset.invalid_declaration", "asset")?;
    Ok(value)
}

fn validate_sha256(
    value: &str,
    code: &'static str,
    label: &str,
) -> Result<(), ArtifactImportError> {
    if value.len() == 64 && value.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Ok(());
    }
    Err(import_error(
        code,
        format!("{label} SHA-256 '{value}' is not 64 hexadecimal characters"),
    ))
}

fn validate_external_assets(
    file: &File,
    declarations: Vec<ExternalAssetDeclaration>,
    inputs: Vec<ExternalAssetInput>,
    diagnostics: &mut Vec<ArtifactDiagnostic>,
) -> Result<Vec<ValidatedExternalAsset>, ArtifactImportError> {
    validate_riv_asset_catalog(file, &declarations)?;

    let mut inputs_by_id = BTreeMap::new();
    let mut input_unique_names = BTreeSet::new();
    for input in inputs {
        let (asset_id, unique_name) = external_asset_input_identity(&input);
        let unique_name = unique_name.to_owned();
        if inputs_by_id.insert(asset_id, input).is_some() {
            return Err(import_error(
                "artifact.asset.duplicate_input",
                format!("asset id {asset_id} was supplied or omitted more than once"),
            ));
        }
        if !input_unique_names.insert(unique_name.clone()) {
            return Err(import_error(
                "artifact.asset.duplicate_input",
                format!("asset unique name '{unique_name}' appears more than once"),
            ));
        }
    }

    let mut validated = Vec::with_capacity(declarations.len());
    for declaration in declarations {
        let Some(input) = inputs_by_id.remove(&declaration.asset_id) else {
            return Err(import_error(
                "artifact.asset.input_missing",
                format!(
                    "manifest asset {} '{}' was neither supplied nor explicitly omitted",
                    declaration.asset_id, declaration.unique_name
                ),
            ));
        };
        let (kind, asset_id, unique_name, source_key, expected_sha256, required, bytes) =
            match input {
                ExternalAssetInput::Supplied {
                    kind,
                    asset_id,
                    unique_name,
                    source_key,
                    expected_sha256,
                    required,
                    bytes,
                } => (
                    kind,
                    asset_id,
                    unique_name,
                    source_key,
                    expected_sha256,
                    required,
                    Some(bytes),
                ),
                ExternalAssetInput::Omitted {
                    kind,
                    asset_id,
                    unique_name,
                    source_key,
                    expected_sha256,
                    required,
                } => (
                    kind,
                    asset_id,
                    unique_name,
                    source_key,
                    expected_sha256,
                    required,
                    None,
                ),
            };
        if kind != declaration.kind {
            return Err(import_error(
                "artifact.asset.kind_mismatch",
                format!(
                    "asset {asset_id} '{}' is {:?}, manifest declares {:?}",
                    unique_name, kind, declaration.kind
                ),
            ));
        }
        if unique_name != declaration.unique_name {
            return Err(import_error(
                "artifact.asset.unique_name_mismatch",
                format!(
                    "asset {asset_id} unique name '{}' does not match manifest '{}'",
                    unique_name, declaration.unique_name
                ),
            ));
        }
        if source_key != declaration.source_key {
            return Err(import_error(
                "artifact.asset.source_key_mismatch",
                format!(
                    "asset {asset_id} '{}' source key does not match the signed manifest",
                    declaration.unique_name
                ),
            ));
        }
        if expected_sha256 != declaration.sha256 {
            return Err(import_error(
                "artifact.asset.expected_hash_mismatch",
                format!(
                    "asset {asset_id} '{}' expected hash does not match the signed manifest",
                    declaration.unique_name
                ),
            ));
        }
        if required != declaration.required {
            return Err(import_error(
                "artifact.asset.required_mismatch",
                format!(
                    "asset {asset_id} '{}' required policy does not match the signed manifest",
                    declaration.unique_name
                ),
            ));
        }
        if let Some(bytes) = bytes.as_ref() {
            let actual_size = u64::try_from(bytes.len()).map_err(|_| {
                import_error(
                    "artifact.asset.size_mismatch",
                    format!("asset {asset_id} byte length does not fit in UInt64"),
                )
            })?;
            if let Some(expected_size) = declaration.size_bytes
                && actual_size != expected_size
            {
                return Err(import_error(
                    "artifact.asset.size_mismatch",
                    format!(
                        "asset {asset_id} '{}' declares {expected_size} bytes, received {actual_size}",
                        unique_name
                    ),
                ));
            }
            let actual_hash = sha256_hex(bytes);
            if !declaration.sha256.eq_ignore_ascii_case(&actual_hash) {
                return Err(import_error(
                    "artifact.asset.hash_mismatch",
                    format!(
                        "asset {asset_id} '{}' hash does not match its manifest declaration",
                        unique_name
                    ),
                ));
            }
        } else if declaration.required {
            return Err(import_error(
                "artifact.asset.required_missing",
                format!(
                    "required asset {asset_id} '{}' was omitted",
                    declaration.unique_name
                ),
            ));
        } else {
            diagnostics.push(ArtifactDiagnostic {
                severity: ArtifactDiagnosticSeverity::Warning,
                code: "artifact.asset.optional_missing",
                message: format!(
                    "optional asset {asset_id} '{}' was omitted",
                    declaration.unique_name
                ),
            });
        }
        validated.push(ValidatedExternalAsset {
            kind,
            asset_id,
            unique_name,
            source_key: declaration.source_key,
            expected_sha256: declaration.sha256,
            required: declaration.required,
            bytes,
        });
    }

    if let Some((asset_id, input)) = inputs_by_id.into_iter().next() {
        let (_, unique_name) = external_asset_input_identity(&input);
        return Err(import_error(
            "artifact.asset.undeclared",
            format!("asset {asset_id} '{unique_name}' is not declared by the manifest"),
        ));
    }
    Ok(validated)
}

fn external_asset_input_identity(input: &ExternalAssetInput) -> (u32, &str) {
    match input {
        ExternalAssetInput::Supplied {
            asset_id,
            unique_name,
            ..
        }
        | ExternalAssetInput::Omitted {
            asset_id,
            unique_name,
            ..
        } => (*asset_id, unique_name),
    }
}

fn validate_riv_asset_catalog(
    file: &File,
    declarations: &[ExternalAssetDeclaration],
) -> Result<(), ArtifactImportError> {
    let mut assets_by_id = BTreeMap::new();
    for asset in file.runtime().file_assets() {
        let Some(raw_asset_id) = asset.uint_property("assetId") else {
            continue;
        };
        let Ok(asset_id) = u32::try_from(raw_asset_id) else {
            continue;
        };
        if assets_by_id.insert(asset_id, asset).is_some() {
            return Err(import_error(
                "artifact.asset.catalog_duplicate_id",
                format!("imported RIV catalog contains asset id {asset_id} more than once"),
            ));
        }
    }

    for declaration in declarations {
        let Some(asset) = assets_by_id.get(&declaration.asset_id) else {
            return Err(import_error(
                "artifact.asset.catalog_missing",
                format!(
                    "manifest asset {} '{}' is absent from the imported RIV catalog",
                    declaration.asset_id, declaration.unique_name
                ),
            ));
        };
        let expected_type = match declaration.kind {
            ExternalAssetKind::Image => "ImageAsset",
            ExternalAssetKind::Font => "FontAsset",
        };
        if asset.type_name != expected_type {
            return Err(import_error(
                "artifact.asset.catalog_kind_mismatch",
                format!(
                    "manifest asset {} '{}' expects {expected_type}, imported RIV contains {}",
                    declaration.asset_id, declaration.unique_name, asset.type_name
                ),
            ));
        }
        let actual_unique_name = asset.file_asset_unique_name().ok_or_else(|| {
            import_error(
                "artifact.asset.catalog_unique_name_mismatch",
                format!(
                    "imported RIV asset {} has no valid unique name",
                    declaration.asset_id
                ),
            )
        })?;
        if actual_unique_name != declaration.unique_name {
            return Err(import_error(
                "artifact.asset.catalog_unique_name_mismatch",
                format!(
                    "manifest asset {} unique name '{}' does not match imported RIV '{}'",
                    declaration.asset_id, declaration.unique_name, actual_unique_name
                ),
            ));
        }
    }
    Ok(())
}

fn authenticate_manifest(
    manifest_bytes: &[u8],
    signature_envelope_bytes: Option<&[u8]>,
    selected_key: Option<&SelectedArtifactSigningKey>,
) -> (ArtifactAuthorization, Option<ArtifactDiagnostic>) {
    let Some(signature_envelope_bytes) = signature_envelope_bytes else {
        return visual_only(
            VisualOnlyReason::MissingSignature,
            "artifact.authentication.missing",
            "artifact has no detached manifest signature",
        );
    };
    let envelope: FlowManifestSignatureEnvelope =
        match serde_json::from_slice(signature_envelope_bytes) {
            Ok(envelope) => envelope,
            Err(error) => {
                return visual_only(
                    VisualOnlyReason::MalformedSignature,
                    "artifact.authentication.malformed",
                    format!("detached manifest signature is malformed: {error}"),
                );
            }
        };
    if envelope.version != 1
        || envelope.signs != SIGNATURE_FILE_NAME
        || envelope.algorithm != SIGNATURE_ALGORITHM
    {
        return visual_only(
            VisualOnlyReason::UnsupportedSignature,
            "artifact.authentication.unsupported",
            "detached manifest signature uses an unsupported envelope",
        );
    }
    let Some(selected_key) = selected_key else {
        return visual_only(
            VisualOnlyReason::MissingSelectedKey,
            "artifact.authentication.missing_key",
            format!(
                "no validation key was selected for signature key '{}'",
                envelope.key_id
            ),
        );
    };
    if envelope.key_id != selected_key.key_id {
        return visual_only(
            VisualOnlyReason::KeyMismatch,
            "artifact.authentication.key_mismatch",
            format!(
                "signature key '{}' does not match selected key '{}'",
                envelope.key_id, selected_key.key_id
            ),
        );
    }
    let verifying_key = match VerifyingKey::from_bytes(&selected_key.public_key) {
        Ok(key) => key,
        Err(error) => {
            return visual_only(
                VisualOnlyReason::InvalidSelectedKey,
                "artifact.authentication.invalid_key",
                format!("selected Ed25519 key is invalid: {error}"),
            );
        }
    };
    let signature_bytes = match BASE64.decode(envelope.signature_base64.as_bytes()) {
        Ok(bytes) => bytes,
        Err(error) => {
            return visual_only(
                VisualOnlyReason::MalformedSignature,
                "artifact.authentication.malformed",
                format!("detached Ed25519 signature is not valid base64: {error}"),
            );
        }
    };
    let signature = match Signature::from_slice(&signature_bytes) {
        Ok(signature) => signature,
        Err(error) => {
            return visual_only(
                VisualOnlyReason::MalformedSignature,
                "artifact.authentication.malformed",
                format!("detached Ed25519 signature has an invalid length: {error}"),
            );
        }
    };
    if verifying_key
        .verify_strict(manifest_bytes, &signature)
        .is_err()
    {
        return visual_only(
            VisualOnlyReason::InvalidSignature,
            "artifact.authentication.invalid_signature",
            "detached signature does not authenticate the exact manifest bytes",
        );
    }
    (
        ArtifactAuthorization::Authenticated {
            key_id: envelope.key_id,
        },
        None,
    )
}

fn visual_only(
    reason: VisualOnlyReason,
    code: &'static str,
    message: impl Into<String>,
) -> (ArtifactAuthorization, Option<ArtifactDiagnostic>) {
    (
        ArtifactAuthorization::VisualOnly { reason },
        Some(ArtifactDiagnostic {
            severity: ArtifactDiagnosticSeverity::Warning,
            code,
            message: message.into(),
        }),
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn import_error(code: &'static str, message: impl Into<String>) -> ArtifactImportError {
    ArtifactImportError {
        code,
        message: message.into(),
    }
}
