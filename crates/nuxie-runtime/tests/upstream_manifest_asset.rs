//! Section framing cases from tests/unit_tests/runtime/manifest_asset_test.cpp
//! at d8727299. Upstream's section 2 decoder is intentionally not ported, so
//! that ID must be skipped like any other unknown section.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        assets::manifest_asset::ManifestAsset,
        manifest_sections::{ManifestSections, manifest_detail::append_var_uint},
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

fn names_payload(id: u64, name: &[u8]) -> Vec<u8> {
    let mut payload = Vec::new();
    write_var_uint(&mut payload, 1);
    write_var_uint(&mut payload, id);
    write_var_uint(&mut payload, name.len() as u64);
    payload.extend_from_slice(name);
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
fn a_manifest_with_no_sections_decodes() {
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &[]));
    assert_eq!(asset.resolve_name(4), "");
}

#[test]
fn a_legacy_section_two_is_skipped_and_later_sections_still_decode() {
    let mut bytes = Vec::new();
    // Version, flags, artboard index, then an appended field.
    write_section(&mut bytes, 2, &[1, 1, 7, 0x2a]);
    write_section(
        &mut bytes,
        ManifestSections::Names as u64,
        &names_payload(4, b"hi"),
    );
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &bytes));
    assert_eq!(asset.resolve_name(4), "hi");

    // Contents the retired decoder rejected are opaque bytes now.
    let mut out_of_range = Vec::new();
    append_var_uint(&mut out_of_range, 1);
    append_var_uint(&mut out_of_range, 1);
    append_var_uint(&mut out_of_range, 1u64 << 32);
    let mut bytes = Vec::new();
    write_section(&mut bytes, 2, &out_of_range);
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &bytes));
}

#[test]
fn an_unknown_section_is_skipped_and_later_sections_still_decode() {
    let mut bytes = Vec::new();
    write_section(&mut bytes, 99, &[1, 2, 3]);
    write_section(
        &mut bytes,
        ManifestSections::Names as u64,
        &names_payload(4, b"hi"),
    );
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &bytes));
    assert_eq!(asset.resolve_name(4), "hi");
}

#[test]
fn a_section_id_that_aliases_a_known_one_is_skipped() {
    let payload = names_payload(4, b"hi");
    for aliased in [256, 257] {
        let mut bytes = Vec::new();
        append_var_uint(&mut bytes, aliased);
        append_var_uint(&mut bytes, payload.len() as u64);
        bytes.extend_from_slice(&payload);
        let mut asset = ManifestAsset::default();
        assert!(decode(&mut asset, &bytes), "section id {aliased}");
        assert_eq!(asset.resolve_name(4), "", "section id {aliased}");
    }
    let mut real = Vec::new();
    write_section(&mut real, ManifestSections::Names as u64, &payload);
    let mut asset = ManifestAsset::default();
    assert!(decode(&mut asset, &real));
    assert_eq!(asset.resolve_name(4), "hi");
}
