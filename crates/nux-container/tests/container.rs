#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]

mod common;

use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use common::{
    EMBEDDED_BYTES, GoldenSource, encode_raw, golden_bytes, sha256_hex, toc_field_positions,
};
use nux_container::test_support::{TEST_ONLY_DEV_KEY_ID, TEST_ONLY_DEV_KEYPAIR};
use nux_container::{
    AssetLocation, EmbeddedMember, MemberInventoryEntry, MemberRole, NUX_MAX_JOURNEY_BYTES,
    NUX_MAX_MANIFEST_BYTES, NUX_MAX_MEMBERS, NUX_MAX_PACKAGE_BYTES, NUX_MAX_SIGNATURE_BYTES,
    NuxContainerError, NuxPackageModel, SignatureSource, SignatureVerification, read_package,
    validate_nux_roundtrip, verify_signature, write_package,
};

#[test]
fn writer_reader_signature_and_roundtrip_form_a_complete_tracer_path() {
    let bytes = golden_bytes();
    let package = read_package(&bytes).expect("golden package reads");

    assert_eq!(
        package.manifest().identity.experience_id,
        "experience-fixture"
    );
    assert!(package.member("scene").expect("scene").starts_with(b"RIVE"));
    let SignatureVerification::Verified { key_id, scene } = verify_signature(
        &package,
        [(
            TEST_ONLY_DEV_KEY_ID,
            LazyLock::force(&TEST_ONLY_DEV_KEYPAIR).public_key(),
        )],
    ) else {
        panic!("golden package signature must verify");
    };
    assert_eq!(key_id, TEST_ONLY_DEV_KEY_ID);
    assert_eq!(scene.bytes(), package.member("scene").expect("scene"));
    validate_nux_roundtrip(&bytes).expect("writer output is canonical");
}

#[test]
fn golden_fixture_matches_the_deterministic_generator() {
    // Regenerate intentionally with:
    // NUX_REGENERATE_FIXTURES=1 cargo test -p nux-container golden_fixture_matches
    let generated = golden_bytes();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden.nux");
    if std::env::var_os("NUX_REGENERATE_FIXTURES").is_some() {
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture dir");
        fs::write(&path, &generated).expect("write regenerated golden fixture");
    }
    assert_eq!(fs::read(path).expect("committed golden fixture"), generated);
}

#[test]
fn writer_is_deterministic_and_uses_canonical_order_and_alignment() {
    let first = golden_bytes();
    let second = golden_bytes();
    assert_eq!(first, second);

    let package = read_package(&first).expect("valid package");
    let names: Vec<&str> = package.toc().iter().map(|entry| entry.name).collect();
    assert_eq!(
        names,
        vec![
            "manifest",
            "signature",
            "scene",
            "journey",
            GoldenSource::new().embedded_name.as_str(),
        ]
    );
    assert!(package.toc().iter().all(|entry| entry.offset % 16 == 0));
}

#[test]
fn rejects_bad_magic_wrong_version_and_truncated_toc() {
    let mut bad_magic = golden_bytes();
    bad_magic[0] ^= 0xff;
    assert!(matches!(
        read_package(&bad_magic),
        Err(NuxContainerError::BadMagic)
    ));

    let mut wrong_version = golden_bytes();
    wrong_version[8..12].copy_from_slice(&2u32.to_le_bytes());
    assert!(matches!(
        read_package(&wrong_version),
        Err(NuxContainerError::UnsupportedVersion(2))
    ));

    let mut truncated = b"\x89NUX\r\n\x1a\n".to_vec();
    truncated.extend_from_slice(&1u32.to_le_bytes());
    truncated.extend_from_slice(&1u32.to_le_bytes());
    assert!(matches!(
        read_package(&truncated),
        Err(NuxContainerError::Truncated(_))
    ));
}

#[test]
fn rejects_excessive_member_count_and_malformed_utf8_name() {
    let mut too_many = b"\x89NUX\r\n\x1a\n".to_vec();
    too_many.extend_from_slice(&1u32.to_le_bytes());
    too_many.extend_from_slice(&(NUX_MAX_MEMBERS + 1).to_le_bytes());
    assert!(matches!(
        read_package(&too_many),
        Err(NuxContainerError::TooManyMembers { .. })
    ));

    let mut malformed_name = b"\x89NUX\r\n\x1a\n".to_vec();
    malformed_name.extend_from_slice(&1u32.to_le_bytes());
    malformed_name.extend_from_slice(&1u32.to_le_bytes());
    malformed_name.extend_from_slice(&1u16.to_le_bytes());
    malformed_name.push(0xff);
    malformed_name.extend_from_slice(&0u64.to_le_bytes());
    malformed_name.extend_from_slice(&0u64.to_le_bytes());
    assert!(matches!(
        read_package(&malformed_name),
        Err(NuxContainerError::InvalidMemberNameUtf8)
    ));
}

#[test]
fn rejects_overlapping_out_of_bounds_and_duplicate_members() {
    let mut overlapping = golden_bytes();
    let positions = toc_field_positions(&overlapping);
    let scene_offset_position = positions
        .iter()
        .find(|(name, _, _)| name == "scene")
        .expect("scene position")
        .1;
    let journey_offset_position = positions
        .iter()
        .find(|(name, _, _)| name == "journey")
        .expect("journey position")
        .1;
    let scene_offset = overlapping[scene_offset_position..scene_offset_position + 8].to_vec();
    overlapping[journey_offset_position..journey_offset_position + 8]
        .copy_from_slice(&scene_offset);
    assert!(matches!(
        read_package(&overlapping),
        Err(NuxContainerError::OverlappingMembers { .. })
    ));

    let mut out_of_bounds = golden_bytes();
    let scene_length_position = toc_field_positions(&out_of_bounds)
        .into_iter()
        .find(|(name, _, _)| name == "scene")
        .expect("scene position")
        .2;
    out_of_bounds[scene_length_position..scene_length_position + 8]
        .copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(matches!(
        read_package(&out_of_bounds),
        Err(NuxContainerError::MemberOutOfBounds { .. })
    ));

    let duplicate = encode_raw(&[("manifest", b""), ("manifest", b"")]);
    assert!(matches!(
        read_package(&duplicate),
        Err(NuxContainerError::DuplicateMemberName(_))
    ));
}

#[test]
fn rejects_missing_manifest_and_signature() {
    let no_manifest = encode_raw(&[("signature", b"{}")]);
    assert!(matches!(
        read_package(&no_manifest),
        Err(NuxContainerError::MissingMember("manifest"))
    ));
    let no_signature = encode_raw(&[("manifest", b"{}")]);
    assert!(matches!(
        read_package(&no_signature),
        Err(NuxContainerError::MissingMember("signature"))
    ));

    let empty_signature = encode_raw(&[("manifest", b"{}"), ("signature", b"")]);
    assert!(matches!(
        read_package(&empty_signature),
        Err(NuxContainerError::ZeroLengthMember(name)) if name == "signature"
    ));
}

#[test]
fn rejects_digest_mismatch_for_scene_journey_and_embedded_asset() {
    for member_name in [
        "scene",
        "journey",
        GoldenSource::new().embedded_name.as_str(),
    ] {
        let mut bytes = golden_bytes();
        let package = read_package(&bytes).expect("valid before mutation");
        let entry = package
            .toc()
            .iter()
            .find(|entry| entry.name == member_name)
            .copied()
            .expect("member entry");
        let position = usize::try_from(entry.offset).expect("fixture offset");
        drop(package);
        bytes[position] ^= 1;
        assert!(
            matches!(
                read_package(&bytes),
                Err(NuxContainerError::DigestMismatch(name)) if name == member_name
            ),
            "member {member_name}"
        );
    }
}

#[test]
fn rejects_named_member_over_its_cap() {
    let oversized_len = usize::try_from(NUX_MAX_MANIFEST_BYTES + 1).expect("limit fits usize");
    let oversized = vec![0u8; oversized_len];
    let bytes = encode_raw(&[("manifest", &oversized), ("signature", b"")]);
    assert!(matches!(
        read_package(&bytes),
        Err(NuxContainerError::MemberTooLarge { name, .. }) if name == "manifest"
    ));

    let oversized_len = usize::try_from(NUX_MAX_JOURNEY_BYTES + 1).expect("limit fits usize");
    let oversized = vec![0u8; oversized_len];
    let bytes = encode_raw(&[
        ("manifest", b""),
        ("signature", b"{}"),
        ("journey", &oversized),
    ]);
    assert!(matches!(
        read_package(&bytes),
        Err(NuxContainerError::MemberTooLarge { name, .. }) if name == "journey"
    ));

    let oversized_len = usize::try_from(NUX_MAX_SIGNATURE_BYTES + 1).expect("limit fits usize");
    let oversized = vec![0u8; oversized_len];
    let bytes = encode_raw(&[("manifest", b""), ("signature", &oversized)]);
    assert!(matches!(
        read_package(&bytes),
        Err(NuxContainerError::MemberTooLarge { name, .. }) if name == "signature"
    ));
}

#[test]
fn rejects_package_over_its_cap_before_parsing() {
    let oversized_len = usize::try_from(NUX_MAX_PACKAGE_BYTES + 1).expect("limit fits usize");
    let bytes = vec![0u8; oversized_len];
    assert!(matches!(
        read_package(&bytes),
        Err(NuxContainerError::PackageTooLarge { .. })
    ));
}

#[test]
fn rejects_unknown_capability_and_invalid_entry_screen() {
    let mut capability = GoldenSource::new();
    capability.manifest.required_capabilities = vec!["future.scene.extension".to_owned()];
    assert!(matches!(
        read_package(&write_package(&capability.model()).expect("write")),
        Err(NuxContainerError::UnknownCapability(name)) if name == "future.scene.extension"
    ));

    let mut entry = GoldenSource::new();
    entry.manifest.entry.screen_id = "missing-screen".to_owned();
    assert!(matches!(
        read_package(&write_package(&entry.model()).expect("write")),
        Err(NuxContainerError::InvalidManifest(message))
            if message.contains("entry.screenId")
    ));
}

#[test]
fn rejects_bad_scene_header_and_journey_envelope() {
    let bad_scene = b"NOPE synthetic scene";
    let source = GoldenSource::new();
    let bad_scene_model = NuxPackageModel {
        manifest: source.manifest.clone(),
        scene: bad_scene,
        journey: common::JOURNEY_BYTES,
        embedded_assets: vec![EmbeddedMember {
            name: &source.embedded_name,
            bytes: EMBEDDED_BYTES,
        }],
        signature: SignatureSource::Signer(LazyLock::force(&TEST_ONLY_DEV_KEYPAIR)),
    };
    assert!(matches!(
        read_package(&write_package(&bad_scene_model).expect("write")),
        Err(NuxContainerError::InvalidSceneHeader)
    ));

    let bad_journey = br#"{"schemaVersion":2}"#;
    let bad_journey_model = NuxPackageModel {
        manifest: source.manifest.clone(),
        scene: common::SCENE_BYTES,
        journey: bad_journey,
        embedded_assets: vec![EmbeddedMember {
            name: &source.embedded_name,
            bytes: EMBEDDED_BYTES,
        }],
        signature: SignatureSource::Signer(LazyLock::force(&TEST_ONLY_DEV_KEYPAIR)),
    };
    assert!(matches!(
        read_package(&write_package(&bad_journey_model).expect("write")),
        Err(NuxContainerError::JourneySchemaVersionMismatch)
    ));
}

#[test]
fn manifest_and_signature_envelopes_deny_unknown_fields() {
    let bytes = golden_bytes();
    let package = read_package(&bytes).expect("golden package");
    let manifest_text = std::str::from_utf8(package.manifest_bytes()).expect("manifest UTF-8");
    let changed_manifest = manifest_text.replacen(
        r#"{"version":1,"identity":"#,
        r#"{"version":1,"unknown":true,"identity":"#,
        1,
    );
    let source = GoldenSource::new();
    let strict_manifest = encode_raw(&[
        ("manifest", changed_manifest.as_bytes()),
        ("signature", package.signature_bytes()),
        ("scene", package.member("scene").expect("scene")),
        ("journey", package.member("journey").expect("journey")),
        (
            &source.embedded_name,
            package.member(&source.embedded_name).expect("asset"),
        ),
    ]);
    assert!(matches!(
        read_package(&strict_manifest),
        Err(NuxContainerError::ManifestJson(_))
    ));

    let signature_text = std::str::from_utf8(package.signature_bytes()).expect("signature UTF-8");
    let changed_signature = signature_text.replacen(
        r#"{"version":1,"signs":"#,
        r#"{"version":1,"unknown":true,"signs":"#,
        1,
    );
    let strict_signature = encode_raw(&[
        ("manifest", package.manifest_bytes()),
        ("signature", changed_signature.as_bytes()),
        ("scene", package.member("scene").expect("scene")),
        ("journey", package.member("journey").expect("journey")),
        (
            &source.embedded_name,
            package.member(&source.embedded_name).expect("asset"),
        ),
    ]);
    let parsed = read_package(&strict_signature).expect("signature parsing remains separate");
    assert_eq!(
        verify_signature(
            &parsed,
            [(
                TEST_ONLY_DEV_KEY_ID,
                LazyLock::force(&TEST_ONLY_DEV_KEYPAIR).public_key(),
            )],
        ),
        SignatureVerification::MalformedEnvelope
    );
}

#[test]
fn rejects_inventory_set_mismatch_and_missing_embedded_member() {
    let bytes = golden_bytes();
    let package = read_package(&bytes).expect("golden package");
    let mut manifest = package.manifest().clone();
    manifest.members.push(MemberInventoryEntry {
        name: "future".to_owned(),
        role: MemberRole::Asset,
        sha256: sha256_hex(b"future"),
        size_bytes: 6,
        content_type: "application/octet-stream".to_owned(),
    });
    let manifest_bytes = serde_json::to_vec(&manifest).expect("serialize");
    let mismatched = encode_raw(&[
        ("manifest", &manifest_bytes),
        ("signature", package.signature_bytes()),
        ("scene", package.member("scene").expect("scene")),
        ("journey", package.member("journey").expect("journey")),
        (
            GoldenSource::new().embedded_name.as_str(),
            package
                .member(&GoldenSource::new().embedded_name)
                .expect("asset"),
        ),
    ]);
    assert!(matches!(
        read_package(&mismatched),
        Err(NuxContainerError::MemberSetMismatch)
    ));

    let mut missing_asset = GoldenSource::new();
    let nonexistent_hash = sha256_hex(b"nonexistent");
    missing_asset.manifest.assets.images[1].location = AssetLocation::Embedded {
        member: format!("assets/sha256/{nonexistent_hash}.png"),
    };
    missing_asset.manifest.assets.images[1].sha256 = nonexistent_hash;
    missing_asset.manifest.assets.images[1].size_bytes = 11;
    assert!(matches!(
        read_package(&write_package(&missing_asset.model()).expect("write")),
        Err(NuxContainerError::InvalidAsset(message))
            if message.contains("does not exist")
    ));
}

#[test]
fn accepts_an_inventoried_unknown_member() {
    let bytes = golden_bytes();
    let package = read_package(&bytes).expect("golden package");
    let mut manifest = package.manifest().clone();
    manifest.members.push(MemberInventoryEntry {
        name: "future-data".to_owned(),
        role: MemberRole::Asset,
        sha256: sha256_hex(b"future"),
        size_bytes: 6,
        content_type: "application/octet-stream".to_owned(),
    });
    let manifest_bytes = serde_json::to_vec(&manifest).expect("serialize");
    let source = GoldenSource::new();
    let accepted = encode_raw(&[
        ("future-data", b"future"),
        ("journey", package.member("journey").expect("journey")),
        ("manifest", &manifest_bytes),
        ("signature", package.signature_bytes()),
        (
            &source.embedded_name,
            package.member(&source.embedded_name).expect("asset"),
        ),
        ("scene", package.member("scene").expect("scene")),
    ]);

    let parsed = read_package(&accepted).expect("unknown member is tolerated");
    assert_eq!(parsed.member("future-data"), Some(b"future".as_slice()));
}

#[test]
fn reports_all_signature_outcomes() {
    let bytes = golden_bytes();
    let package = read_package(&bytes).expect("golden package");
    assert_eq!(
        verify_signature(&package, [("other-key", [7u8; 32])]),
        SignatureVerification::UnknownKey
    );

    let mut bad_signature =
        serde_json::from_slice::<nux_container::SignatureEnvelopeV1>(package.signature_bytes())
            .expect("signature envelope");
    bad_signature.signature_base64.replace_range(..1, "A");
    let bad_signature_bytes = serde_json::to_vec(&bad_signature).expect("serialize");
    let source = GoldenSource::new();
    let bad_package_bytes = encode_raw(&[
        ("manifest", package.manifest_bytes()),
        ("signature", &bad_signature_bytes),
        ("scene", package.member("scene").expect("scene")),
        ("journey", package.member("journey").expect("journey")),
        (
            &source.embedded_name,
            package.member(&source.embedded_name).expect("asset"),
        ),
    ]);
    let bad_package = read_package(&bad_package_bytes).expect("structurally valid");
    assert_eq!(
        verify_signature(
            &bad_package,
            [(
                TEST_ONLY_DEV_KEY_ID,
                LazyLock::force(&TEST_ONLY_DEV_KEYPAIR).public_key(),
            )],
        ),
        SignatureVerification::BadSignature
    );

    let malformed_bytes = encode_raw(&[
        ("manifest", package.manifest_bytes()),
        ("signature", b"not json"),
        ("scene", package.member("scene").expect("scene")),
        ("journey", package.member("journey").expect("journey")),
        (
            &source.embedded_name,
            package.member(&source.embedded_name).expect("asset"),
        ),
    ]);
    let malformed = read_package(&malformed_bytes).expect("signature parsing is separate");
    assert_eq!(
        verify_signature(
            &malformed,
            [(
                TEST_ONLY_DEV_KEY_ID,
                LazyLock::force(&TEST_ONLY_DEV_KEYPAIR).public_key(),
            )],
        ),
        SignatureVerification::MalformedEnvelope
    );
}

#[test]
fn malformed_signature_takes_precedence_over_unknown_key() {
    let bytes = golden_bytes();
    let package = read_package(&bytes).expect("golden package");
    let mut envelope =
        serde_json::from_slice::<nux_container::SignatureEnvelopeV1>(package.signature_bytes())
            .expect("signature envelope");
    envelope.key_id = "absent-key".to_owned();
    envelope.signature_base64 = "not base64!".to_owned();
    let signature_bytes = serde_json::to_vec(&envelope).expect("serialize");
    let source = GoldenSource::new();
    let malformed_bytes = encode_raw(&[
        ("manifest", package.manifest_bytes()),
        ("signature", &signature_bytes),
        ("scene", package.member("scene").expect("scene")),
        ("journey", package.member("journey").expect("journey")),
        (
            &source.embedded_name,
            package.member(&source.embedded_name).expect("asset"),
        ),
    ]);
    let malformed = read_package(&malformed_bytes).expect("signature parsing is separate");
    assert_eq!(
        verify_signature(&malformed, std::iter::empty::<(&str, [u8; 32])>()),
        SignatureVerification::MalformedEnvelope
    );
}

#[test]
fn writer_computes_inventory_for_embedded_members() {
    let source = GoldenSource::new();
    let extra_name = format!("assets/sha256/{}.png", sha256_hex(EMBEDDED_BYTES));
    let model = NuxPackageModel {
        manifest: source.manifest.clone(),
        scene: common::SCENE_BYTES,
        journey: common::JOURNEY_BYTES,
        embedded_assets: vec![EmbeddedMember {
            name: &extra_name,
            bytes: EMBEDDED_BYTES,
        }],
        signature: SignatureSource::Signer(LazyLock::force(&TEST_ONLY_DEV_KEYPAIR)),
    };
    let bytes = write_package(&model).expect("write");
    let package = read_package(&bytes).expect("valid writer output");
    let inventory = package
        .manifest()
        .members
        .iter()
        .find(|entry| entry.name == extra_name)
        .expect("embedded inventory");
    assert_eq!(inventory.sha256, sha256_hex(EMBEDDED_BYTES));
    assert_eq!(inventory.size_bytes, EMBEDDED_BYTES.len() as u64);
}

#[test]
fn roundtrip_deduplicates_a_shared_embedded_asset_member() {
    let mut source = GoldenSource::new();
    let mut shared = source
        .manifest
        .assets
        .images
        .get(1)
        .expect("embedded image")
        .clone();
    shared.rive_asset_id = 99;
    shared.rive_unique_name = "shared-badge".to_owned();
    source.manifest.assets.images.push(shared);

    let bytes = write_package(&source.model()).expect("write");
    read_package(&bytes).expect("shared embedded member package reads");
    validate_nux_roundtrip(&bytes).expect("shared embedded member roundtrips");
}
