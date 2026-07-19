#[path = "../src/artifact.rs"]
mod artifact;

use artifact::{
    ArtifactAuthorization, ExternalAssetInput, ExternalAssetKind, FlowArtifactImportInput,
    SelectedArtifactSigningKey, VisualOnlyReason, validate_flow_artifact_import,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ed25519_dalek::{Signer as _, SigningKey};
use serde_json::json;
use sha2::{Digest as _, Sha256};

fn fixture_bytes() -> Vec<u8> {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/minimal/two_artboards.riv");
    std::fs::read(fixture).expect("fixture must be readable")
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn manifest_bytes(artifact: &[u8]) -> Vec<u8> {
    manifest_bytes_with_assets(artifact, json!({ "images": [], "fonts": [] }))
}

fn manifest_bytes_with_assets(artifact: &[u8], assets: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "version": 1,
        "flowId": "flow-1",
        "buildId": "build-1",
        "renderer": "rive",
        "riv": {
            "path": "flow.riv",
            "sha256": sha256_hex(artifact),
            "sizeBytes": artifact.len(),
        },
        "assets": assets,
    }))
    .expect("manifest must encode")
}

fn signed_input_for(
    artifact_bytes: Vec<u8>,
    manifest_bytes: Vec<u8>,
    external_assets: Vec<ExternalAssetInput>,
) -> FlowArtifactImportInput {
    let signing_key = SigningKey::from_bytes(&[7; 32]);
    let signature = signing_key.sign(&manifest_bytes);
    let signature_envelope_bytes = serde_json::to_vec(&json!({
        "version": 1,
        "signs": "nuxie-manifest.json",
        "algorithm": "ed25519",
        "keyId": "test-key",
        "signatureBase64": BASE64.encode(signature.to_bytes()),
    }))
    .expect("signature envelope must encode");
    FlowArtifactImportInput {
        expected_flow_id: "flow-1".to_owned(),
        expected_build_id: "build-1".to_owned(),
        artifact_bytes,
        manifest_bytes,
        signature_envelope_bytes: Some(signature_envelope_bytes),
        selected_key: Some(SelectedArtifactSigningKey {
            key_id: "test-key".to_owned(),
            public_key: signing_key.verifying_key().to_bytes(),
        }),
        external_assets,
    }
}

fn signed_input() -> FlowArtifactImportInput {
    let artifact_bytes = fixture_bytes();
    let manifest_bytes = manifest_bytes(&artifact_bytes);
    signed_input_for(artifact_bytes, manifest_bytes, Vec::new())
}

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("fixture type exists");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("fixture ancestor exists")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .expect("fixture property exists")
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            nuxie_schema::definition_by_name(type_name)
                .expect("fixture type exists")
                .type_key
                .int,
        ),
    );
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value);
}

fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_blob(bytes, type_name, name, value.as_bytes());
}

fn external_asset_riv() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 992);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "FontAsset", |bytes| {
        push_uint(bytes, "FontAsset", "assetId", 0);
        push_string(bytes, "FontAsset", "name", "font.ttf");
    });
    push_object(&mut bytes, "ImageAsset", |bytes| {
        push_uint(bytes, "ImageAsset", "assetId", 1);
        push_string(bytes, "ImageAsset", "name", "image.png");
    });
    push_object(&mut bytes, "Artboard", |_| {});
    bytes
}

#[test]
fn exact_signed_manifest_authenticates_the_imported_riv() {
    let validated = validate_flow_artifact_import(signed_input()).expect("valid signed artifact");

    assert_eq!(
        validated.authorization,
        ArtifactAuthorization::Authenticated {
            key_id: "test-key".to_owned(),
        }
    );
    assert!(validated.diagnostics.is_empty());
    assert!(validated.external_assets.is_empty());
    assert_eq!(validated.file.artboard_count(), 2);
}

#[test]
fn supplied_assets_match_signed_declarations_and_the_imported_riv_catalog() {
    let artifact_bytes = external_asset_riv();
    let image_bytes = b"encoded image".to_vec();
    let font_bytes = b"encoded font".to_vec();
    let manifest_bytes = manifest_bytes_with_assets(
        &artifact_bytes,
        json!({
            "images": [{
                "riveAssetId": 1,
                "riveUniqueName": "image-1",
                "sourceAssetKey": "image-source",
                "sha256": sha256_hex(&image_bytes),
                "required": true
            }],
            "fonts": [{
                "riveAssetId": 0,
                "riveUniqueName": "font-0",
                "requestKey": "Inter::400::normal",
                "sha256": sha256_hex(&font_bytes),
                "sizeBytes": font_bytes.len(),
                "required": true
            }]
        }),
    );
    let input = signed_input_for(
        artifact_bytes,
        manifest_bytes,
        vec![
            ExternalAssetInput::Supplied {
                kind: ExternalAssetKind::Image,
                asset_id: 1,
                unique_name: "image-1".to_owned(),
                source_key: "image-source".to_owned(),
                expected_sha256: sha256_hex(&image_bytes),
                required: true,
                bytes: image_bytes.clone(),
            },
            ExternalAssetInput::Supplied {
                kind: ExternalAssetKind::Font,
                asset_id: 0,
                unique_name: "font-0".to_owned(),
                source_key: "Inter::400::normal".to_owned(),
                expected_sha256: sha256_hex(&font_bytes),
                required: true,
                bytes: font_bytes.clone(),
            },
        ],
    );

    let validated = validate_flow_artifact_import(input).expect("declared assets validate");

    assert_eq!(
        validated.external_assets,
        vec![
            artifact::ValidatedExternalAsset {
                kind: ExternalAssetKind::Image,
                asset_id: 1,
                unique_name: "image-1".to_owned(),
                source_key: "image-source".to_owned(),
                expected_sha256: sha256_hex(&image_bytes),
                required: true,
                bytes: Some(image_bytes),
            },
            artifact::ValidatedExternalAsset {
                kind: ExternalAssetKind::Font,
                asset_id: 0,
                unique_name: "font-0".to_owned(),
                source_key: "Inter::400::normal".to_owned(),
                expected_sha256: sha256_hex(&font_bytes),
                required: true,
                bytes: Some(font_bytes),
            },
        ]
    );
}

#[test]
fn missing_signature_keeps_the_artifact_visual_only_with_a_stable_warning() {
    let mut input = signed_input();
    input.signature_envelope_bytes = None;

    let validated = validate_flow_artifact_import(input).expect("unsigned visuals still import");

    assert_eq!(
        validated.authorization,
        ArtifactAuthorization::VisualOnly {
            reason: VisualOnlyReason::MissingSignature,
        }
    );
    assert_eq!(validated.diagnostics.len(), 1);
    assert_eq!(
        validated.diagnostics.first().map(|item| item.code),
        Some("artifact.authentication.missing")
    );
}

#[test]
fn malformed_riv_digest_is_an_integrity_error_not_an_authentication_downgrade() {
    let artifact_bytes = fixture_bytes();
    let manifest_bytes = serde_json::to_vec(&json!({
        "version": 1,
        "flowId": "flow-1",
        "buildId": "build-1",
        "renderer": "rive",
        "riv": {
            "path": "flow.riv",
            "sha256": "not-a-sha256",
            "sizeBytes": artifact_bytes.len(),
        },
        "assets": { "images": [], "fonts": [] },
    }))
    .expect("manifest must encode");
    let error =
        validate_flow_artifact_import(signed_input_for(artifact_bytes, manifest_bytes, Vec::new()))
            .err()
            .expect("malformed integrity declaration must fail");

    assert_eq!(error.code, "artifact.riv.invalid_hash");
}

#[test]
fn changing_only_the_exact_manifest_bytes_invalidates_authentication() {
    let mut input = signed_input();
    input.manifest_bytes.push(b'\n');

    let validated = validate_flow_artifact_import(input).expect("tampered visuals still import");

    assert_eq!(
        validated.authorization,
        ArtifactAuthorization::VisualOnly {
            reason: VisualOnlyReason::InvalidSignature,
        }
    );
    assert_eq!(
        validated.diagnostics.first().map(|item| item.code),
        Some("artifact.authentication.invalid_signature")
    );
}

#[test]
fn requested_flow_and_build_identity_prevent_manifest_replay() {
    let mut flow_replay = signed_input();
    flow_replay.expected_flow_id = "another-flow".to_owned();
    let flow_error = validate_flow_artifact_import(flow_replay)
        .err()
        .expect("flow replay must fail");
    assert_eq!(flow_error.code, "artifact.identity.flow_mismatch");

    let mut build_replay = signed_input();
    build_replay.expected_build_id = "another-build".to_owned();
    let build_error = validate_flow_artifact_import(build_replay)
        .err()
        .expect("build replay must fail");
    assert_eq!(build_error.code, "artifact.identity.build_mismatch");
}

#[test]
fn malformed_or_unusable_signature_evidence_always_downgrades_to_visual_only() {
    let mut malformed = signed_input();
    malformed.signature_envelope_bytes = Some(b"not json".to_vec());
    let malformed = validate_flow_artifact_import(malformed).expect("visual import");
    assert_eq!(
        malformed.authorization,
        ArtifactAuthorization::VisualOnly {
            reason: VisualOnlyReason::MalformedSignature,
        }
    );

    let mut missing_key = signed_input();
    missing_key.selected_key = None;
    let missing_key = validate_flow_artifact_import(missing_key).expect("visual import");
    assert_eq!(
        missing_key.authorization,
        ArtifactAuthorization::VisualOnly {
            reason: VisualOnlyReason::MissingSelectedKey,
        }
    );

    let mut key_mismatch = signed_input();
    key_mismatch
        .selected_key
        .as_mut()
        .expect("selected key")
        .key_id = "another-key".to_owned();
    let key_mismatch = validate_flow_artifact_import(key_mismatch).expect("visual import");
    assert_eq!(
        key_mismatch.authorization,
        ArtifactAuthorization::VisualOnly {
            reason: VisualOnlyReason::KeyMismatch,
        }
    );
}

fn image_declaration(
    artifact_bytes: &[u8],
    image_bytes: &[u8],
    required: bool,
    unique_name: &str,
) -> Vec<u8> {
    manifest_bytes_with_assets(
        artifact_bytes,
        json!({
            "images": [{
                "riveAssetId": 1,
                "riveUniqueName": unique_name,
                "sourceAssetKey": "image-source",
                "sha256": sha256_hex(image_bytes),
                "required": required
            }],
            "fonts": []
        }),
    )
}

#[test]
fn required_asset_omission_is_a_hard_declaration_failure() {
    let artifact_bytes = external_asset_riv();
    let manifest_bytes = image_declaration(&artifact_bytes, b"image", true, "image-1");
    let input = signed_input_for(
        artifact_bytes,
        manifest_bytes,
        vec![ExternalAssetInput::Omitted {
            kind: ExternalAssetKind::Image,
            asset_id: 1,
            unique_name: "image-1".to_owned(),
            source_key: "image-source".to_owned(),
            expected_sha256: sha256_hex(b"image"),
            required: true,
        }],
    );

    let error = validate_flow_artifact_import(input)
        .err()
        .expect("required omission must fail");

    assert_eq!(error.code, "artifact.asset.required_missing");
}

#[test]
fn optional_asset_omission_is_retained_with_a_stable_warning() {
    let artifact_bytes = external_asset_riv();
    let manifest_bytes = image_declaration(&artifact_bytes, b"image", false, "image-1");
    let input = signed_input_for(
        artifact_bytes,
        manifest_bytes,
        vec![ExternalAssetInput::Omitted {
            kind: ExternalAssetKind::Image,
            asset_id: 1,
            unique_name: "image-1".to_owned(),
            source_key: "image-source".to_owned(),
            expected_sha256: sha256_hex(b"image"),
            required: false,
        }],
    );

    let validated = validate_flow_artifact_import(input).expect("optional omission is valid");

    assert_eq!(validated.external_assets.len(), 1);
    assert_eq!(
        validated
            .external_assets
            .first()
            .and_then(|asset| asset.bytes.as_ref()),
        None
    );
    assert_eq!(
        validated.diagnostics.first().map(|item| item.code),
        Some("artifact.asset.optional_missing")
    );
}

#[test]
fn supplied_asset_hash_and_font_size_are_hard_integrity_requirements() {
    let artifact_bytes = external_asset_riv();
    let expected_image = b"expected image";
    let image_manifest = image_declaration(&artifact_bytes, expected_image, true, "image-1");
    let image_error = validate_flow_artifact_import(signed_input_for(
        artifact_bytes.clone(),
        image_manifest,
        vec![ExternalAssetInput::Supplied {
            kind: ExternalAssetKind::Image,
            asset_id: 1,
            unique_name: "image-1".to_owned(),
            source_key: "image-source".to_owned(),
            expected_sha256: sha256_hex(expected_image),
            required: true,
            bytes: b"tampered image".to_vec(),
        }],
    ))
    .err()
    .expect("image hash mismatch must fail");
    assert_eq!(image_error.code, "artifact.asset.hash_mismatch");

    let font_bytes = b"font".to_vec();
    let font_manifest = manifest_bytes_with_assets(
        &artifact_bytes,
        json!({
            "images": [],
            "fonts": [{
                "riveAssetId": 0,
                "riveUniqueName": "font-0",
                "requestKey": "Inter::400::normal",
                "sha256": sha256_hex(&font_bytes),
                "sizeBytes": font_bytes.len() + 1,
                "required": true
            }]
        }),
    );
    let font_error = validate_flow_artifact_import(signed_input_for(
        artifact_bytes,
        font_manifest,
        vec![ExternalAssetInput::Supplied {
            kind: ExternalAssetKind::Font,
            asset_id: 0,
            unique_name: "font-0".to_owned(),
            source_key: "Inter::400::normal".to_owned(),
            expected_sha256: sha256_hex(&font_bytes),
            required: true,
            bytes: font_bytes,
        }],
    ))
    .err()
    .expect("font size mismatch must fail");
    assert_eq!(font_error.code, "artifact.asset.size_mismatch");
}

#[test]
fn declarations_must_be_unique_and_match_the_riv_kind_and_unique_name() {
    let artifact_bytes = external_asset_riv();
    let duplicate_manifest = manifest_bytes_with_assets(
        &artifact_bytes,
        json!({
            "images": [{
                "riveAssetId": 1,
                "riveUniqueName": "image-1",
                "sourceAssetKey": "image-source",
                "sha256": sha256_hex(b"image"),
                "required": true
            }],
            "fonts": [{
                "riveAssetId": 1,
                "riveUniqueName": "font-1",
                "requestKey": "Inter::400::normal",
                "sha256": sha256_hex(b"font"),
                "sizeBytes": 4,
                "required": true
            }]
        }),
    );
    let duplicate_error = validate_flow_artifact_import(signed_input_for(
        artifact_bytes.clone(),
        duplicate_manifest,
        Vec::new(),
    ))
    .err()
    .expect("duplicate id must fail");
    assert_eq!(duplicate_error.code, "artifact.asset.duplicate_id");

    let wrong_unique_manifest = image_declaration(&artifact_bytes, b"image", true, "wrong-1");
    let wrong_unique_error = validate_flow_artifact_import(signed_input_for(
        artifact_bytes,
        wrong_unique_manifest,
        vec![ExternalAssetInput::Supplied {
            kind: ExternalAssetKind::Image,
            asset_id: 1,
            unique_name: "wrong-1".to_owned(),
            source_key: "image-source".to_owned(),
            expected_sha256: sha256_hex(b"image"),
            required: true,
            bytes: b"image".to_vec(),
        }],
    ))
    .err()
    .expect("RIV unique name mismatch must fail");
    assert_eq!(
        wrong_unique_error.code,
        "artifact.asset.catalog_unique_name_mismatch"
    );
}

#[test]
fn every_manifest_asset_requires_one_typed_supplied_or_omitted_input() {
    let artifact_bytes = external_asset_riv();
    let manifest_bytes = image_declaration(&artifact_bytes, b"image", false, "image-1");
    let error =
        validate_flow_artifact_import(signed_input_for(artifact_bytes, manifest_bytes, Vec::new()))
            .err()
            .expect("missing asset state must fail");

    assert_eq!(error.code, "artifact.asset.input_missing");
}

#[test]
fn manifest_asset_declarations_share_the_public_1024_item_limit() {
    let artifact_bytes = fixture_bytes();
    let images = (0..1_025u32)
        .map(|asset_id| {
            json!({
                "riveAssetId": asset_id,
                "riveUniqueName": format!("image-{asset_id}"),
                "sourceAssetKey": format!("source-{asset_id}"),
                "sha256": sha256_hex(b"image"),
                "required": false,
            })
        })
        .collect::<Vec<_>>();
    let manifest_bytes = manifest_bytes_with_assets(
        &artifact_bytes,
        json!({ "images": images, "fonts": [] }),
    );

    let error = validate_flow_artifact_import(signed_input_for(
        artifact_bytes,
        manifest_bytes,
        Vec::new(),
    ))
    .err()
    .expect("oversized declaration catalog must fail before RIV matching");

    assert_eq!(error.code, "artifact.asset.too_many_declarations");
}

#[test]
fn manifest_controlled_identity_and_asset_names_are_bounded() {
    let artifact_bytes = fixture_bytes();
    let oversized_flow_id = "f".repeat(4_097);
    let manifest_bytes = serde_json::to_vec(&json!({
        "version": 1,
        "flowId": oversized_flow_id,
        "buildId": "build-1",
        "renderer": "rive",
        "riv": {
            "path": "flow.riv",
            "sha256": sha256_hex(&artifact_bytes),
            "sizeBytes": artifact_bytes.len(),
        },
        "assets": { "images": [], "fonts": [] },
    }))
    .expect("manifest encodes");
    let identity_error = validate_flow_artifact_import(signed_input_for(
        artifact_bytes,
        manifest_bytes,
        Vec::new(),
    ))
    .err()
    .expect("oversized identity must fail before interpolation");
    assert_eq!(identity_error.code, "artifact.identity.invalid_flow_id");

    let artifact_bytes = external_asset_riv();
    let oversized_unique_name = "n".repeat(4_097);
    let manifest_bytes = image_declaration(
        &artifact_bytes,
        b"image",
        false,
        &oversized_unique_name,
    );
    let name_error = validate_flow_artifact_import(signed_input_for(
        artifact_bytes,
        manifest_bytes,
        Vec::new(),
    ))
    .err()
    .expect("oversized unique name must fail before diagnostics");
    assert_eq!(name_error.code, "artifact.asset.invalid_declaration");
}

#[test]
fn adapter_asset_metadata_must_match_the_signed_manifest() {
    let artifact_bytes = external_asset_riv();
    let image_bytes = b"image";
    let manifest_bytes = image_declaration(&artifact_bytes, image_bytes, false, "image-1");

    let source_key_error = validate_flow_artifact_import(signed_input_for(
        artifact_bytes.clone(),
        manifest_bytes.clone(),
        vec![ExternalAssetInput::Supplied {
            kind: ExternalAssetKind::Image,
            asset_id: 1,
            unique_name: "image-1".to_owned(),
            source_key: "drifted-source".to_owned(),
            expected_sha256: sha256_hex(image_bytes),
            required: false,
            bytes: image_bytes.to_vec(),
        }],
    ))
    .err()
    .expect("source-key drift must fail");
    assert_eq!(source_key_error.code, "artifact.asset.source_key_mismatch");

    let expected_hash_error = validate_flow_artifact_import(signed_input_for(
        artifact_bytes.clone(),
        manifest_bytes.clone(),
        vec![ExternalAssetInput::Supplied {
            kind: ExternalAssetKind::Image,
            asset_id: 1,
            unique_name: "image-1".to_owned(),
            source_key: "image-source".to_owned(),
            expected_sha256: sha256_hex(b"another image"),
            required: false,
            bytes: image_bytes.to_vec(),
        }],
    ))
    .err()
    .expect("expected-hash drift must fail");
    assert_eq!(
        expected_hash_error.code,
        "artifact.asset.expected_hash_mismatch"
    );

    let required_error = validate_flow_artifact_import(signed_input_for(
        artifact_bytes,
        manifest_bytes,
        vec![ExternalAssetInput::Omitted {
            kind: ExternalAssetKind::Image,
            asset_id: 1,
            unique_name: "image-1".to_owned(),
            source_key: "image-source".to_owned(),
            expected_sha256: sha256_hex(image_bytes),
            required: true,
        }],
    ))
    .err()
    .expect("required-policy drift must fail");
    assert_eq!(required_error.code, "artifact.asset.required_mismatch");
}
