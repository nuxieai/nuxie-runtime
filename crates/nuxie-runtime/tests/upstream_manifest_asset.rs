//! All eight cases from tests/unit_tests/runtime/manifest_asset_test.cpp at d8727299.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        assets::manifest_asset::ManifestAsset,
        manifest_sections::{
            ManifestSections, WATERMARK_FLAG_ENABLED, WATERMARK_SECTION_VERSION,
            manifest_detail::append_var_uint,
        },
    },
};

// As upstream, the small-value helper deliberately keeps expected bytes legible.
fn write_var_uint(out: &mut Vec<u8>, value: u64) {
    assert!(value < 128);
    out.push(value as u8);
}

fn write_section(out: &mut Vec<u8>, section: u64, payload: &[u8]) {
    write_var_uint(out, section);
    write_var_uint(out, payload.len() as u64);
    out.extend_from_slice(payload);
}

fn watermark_payload(version: u64, flags: u64, artboard_index: u64) -> Vec<u8> {
    let mut payload = Vec::new();
    write_var_uint(&mut payload, version);
    write_var_uint(&mut payload, flags);
    write_var_uint(&mut payload, artboard_index);
    payload
}

fn decode(asset: &mut ManifestAsset, bytes: &[u8]) -> bool {
    // Upstream supplies null: decode does not use the factory. This is the
    // existing Rust factory-handle boundary, with no allocation during decode.
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    asset.decode(bytes, &factory)
}

#[test]
fn a_manifest_with_no_sections_decodes_with_no_watermark() {
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &[]));
    assert!(!asset.has_watermark());
}

#[test]
fn the_watermark_section_decodes() {
    let mut bytes = Vec::new();
    write_section(
        &mut bytes,
        ManifestSections::Watermark as u64,
        &watermark_payload(1, 1, 7),
    );
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &bytes));
    assert!(asset.has_watermark());
    assert_eq!(asset.watermark_artboard_index(), 7);
}

#[test]
fn a_cleared_enabled_flag_leaves_the_watermark_off() {
    let mut bytes = Vec::new();
    write_section(
        &mut bytes,
        ManifestSections::Watermark as u64,
        &watermark_payload(1, 0, 7),
    );
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &bytes));
    assert!(!asset.has_watermark());
}

#[test]
fn trailing_bytes_in_the_watermark_section_are_skipped() {
    let mut payload = watermark_payload(1, 1, 3);
    payload.extend_from_slice(&[0x2a, 0x2b]);
    let mut bytes = Vec::new();
    write_section(&mut bytes, ManifestSections::Watermark as u64, &payload);
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &bytes));
    assert!(asset.has_watermark());
    assert_eq!(asset.watermark_artboard_index(), 3);
}

#[test]
fn an_unrecognized_watermark_version_is_ignored_not_fatal() {
    let mut bytes = Vec::new();
    write_section(
        &mut bytes,
        ManifestSections::Watermark as u64,
        &watermark_payload(99, 1, 3),
    );
    let mut names = Vec::new();
    write_var_uint(&mut names, 1);
    write_var_uint(&mut names, 4);
    write_var_uint(&mut names, 2);
    names.extend_from_slice(b"hi");
    write_section(&mut bytes, ManifestSections::Names as u64, &names);
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &bytes));
    assert!(!asset.has_watermark());
    assert_eq!(asset.resolve_name(4), "hi");
}

#[test]
fn an_unknown_section_is_skipped_and_later_sections_still_decode() {
    let mut bytes = Vec::new();
    write_section(&mut bytes, 99, &[1, 2, 3]);
    write_section(
        &mut bytes,
        ManifestSections::Watermark as u64,
        &watermark_payload(1, 1, 5),
    );
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &bytes));
    assert!(asset.has_watermark());
    assert_eq!(asset.watermark_artboard_index(), 5);
}

#[test]
fn an_out_of_range_watermark_artboard_index_is_rejected() {
    let mut payload = Vec::new();
    append_var_uint(&mut payload, WATERMARK_SECTION_VERSION);
    append_var_uint(&mut payload, WATERMARK_FLAG_ENABLED);
    append_var_uint(&mut payload, 1u64 << 32);
    let mut bytes = Vec::new();
    write_section(&mut bytes, ManifestSections::Watermark as u64, &payload);
    let mut asset = ManifestAsset::default();
    assert!(!decode(&mut asset, &bytes));
    assert!(!asset.has_watermark());

    let mut ok_payload = Vec::new();
    append_var_uint(&mut ok_payload, WATERMARK_SECTION_VERSION);
    append_var_uint(&mut ok_payload, WATERMARK_FLAG_ENABLED);
    append_var_uint(&mut ok_payload, u32::MAX as u64);
    let mut ok_bytes = Vec::new();
    write_section(
        &mut ok_bytes,
        ManifestSections::Watermark as u64,
        &ok_payload,
    );
    let mut ok = ManifestAsset::default();
    assert!(decode(&mut ok, &ok_bytes));
    assert!(ok.has_watermark());
    assert_eq!(ok.watermark_artboard_index(), u32::MAX);
}

#[test]
fn a_section_id_that_aliases_a_known_one_is_skipped() {
    let payload = watermark_payload(WATERMARK_SECTION_VERSION, WATERMARK_FLAG_ENABLED, 7);
    for aliased in [256, 257, 258] {
        let mut bytes = Vec::new();
        append_var_uint(&mut bytes, aliased);
        append_var_uint(&mut bytes, payload.len() as u64);
        bytes.extend_from_slice(&payload);
        let mut asset = ManifestAsset::default();
        assert!(decode(&mut asset, &bytes), "section id {aliased}");
        assert!(!asset.has_watermark(), "section id {aliased}");
    }
    let mut real = Vec::new();
    write_section(&mut real, ManifestSections::Watermark as u64, &payload);
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &real));
    assert!(asset.has_watermark());
    assert_eq!(asset.watermark_artboard_index(), 7);
}
