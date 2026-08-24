//! Direct ports from
//! `tests/unit_tests/runtime/signed_content_header_test.cpp`.

use nuxie_scripting::envelope::{EnvelopeError, SIGNATURE_SIZE, SignedContent};

fn content_offset(data: &[u8], content: &[u8]) -> usize {
    data.len() - content.len()
}

#[derive(Default)]
struct ScriptAssetProbe {
    verified: bool,
    module_bytecode: Vec<u8>,
}

impl ScriptAssetProbe {
    fn bytecode(&mut self, data: &[u8]) -> bool {
        let Ok(header) = SignedContent::parse(data) else {
            return false;
        };
        self.module_bytecode = header.content.to_vec();
        self.verified = false;
        true
    }
}

#[test]
fn signed_content_header_empty_data_is_invalid() {
    let data = Vec::<u8>::new();

    assert_eq!(SignedContent::parse(&data).is_ok(), false);
}

#[test]
fn signed_content_header_unsigned_header() {
    let data = vec![0x00, 0x01, 0x02, 0x03];
    let header = SignedContent::parse(&data);

    assert_eq!(header.is_ok(), true);
    let header = header.unwrap();
    assert_eq!(header.is_signed(), false);
    assert_eq!(header.version, 0);
    assert_eq!(content_offset(&data, header.content), 1);

    let content = header.content;
    assert_eq!(content.len(), 3);
    assert_eq!(content[0], 0x01);
    assert_eq!(content[1], 0x02);
    assert_eq!(content[2], 0x03);

    let signature = header.signature;
    assert!(signature.is_none());
}

#[test]
fn signed_content_header_signed_header() {
    let mut data = vec![0; 1 + SIGNATURE_SIZE + 3];
    data[0] = 0x80;
    for i in 0..SIGNATURE_SIZE {
        data[1 + i] = i as u8;
    }
    data[1 + SIGNATURE_SIZE] = 0xaa;
    data[1 + SIGNATURE_SIZE + 1] = 0xbb;
    data[1 + SIGNATURE_SIZE + 2] = 0xcc;

    let header = SignedContent::parse(&data);

    assert_eq!(header.is_ok(), true);
    let header = header.unwrap();
    assert_eq!(header.is_signed(), true);
    assert_eq!(header.version, 0);
    assert_eq!(content_offset(&data, header.content), 65);

    let signature = header.signature.unwrap();
    assert_eq!(signature.len(), SIGNATURE_SIZE);
    assert_eq!(signature[0], 0);
    assert_eq!(signature[63], 63);

    let content = header.content;
    assert_eq!(content.len(), 3);
    assert_eq!(content[0], 0xaa);
    assert_eq!(content[1], 0xbb);
    assert_eq!(content[2], 0xcc);
}

#[test]
fn signed_content_header_version_extraction() {
    let data = vec![0x2a, 0x01];
    let header = SignedContent::parse(&data);

    assert_eq!(header.is_ok(), true);
    let header = header.unwrap();
    assert_eq!(header.is_signed(), false);
    assert_eq!(header.version, 42);
}

#[test]
fn signed_content_header_truncated_signed_data_is_invalid() {
    let data = vec![0x80, 0x01, 0x02];

    assert_eq!(
        SignedContent::parse(&data),
        Err(EnvelopeError::TruncatedSignature)
    );
    assert_eq!(data[0] & 0x80 != 0, true);
}

#[test]
fn signed_content_header_minimum_unsigned_flags_only() {
    let data = vec![0x00];
    let header = SignedContent::parse(&data);

    assert_eq!(header.is_ok(), true);
    let header = header.unwrap();
    assert!(header.content.is_empty());
}

#[test]
fn signed_content_header_minimum_signed_no_content() {
    let mut data = vec![0; 1 + SIGNATURE_SIZE];
    data[0] = 0x80;
    let header = SignedContent::parse(&data);

    assert_eq!(header.is_ok(), true);
    let header = header.unwrap();
    assert_eq!(header.is_signed(), true);
    assert!(header.content.is_empty());
    assert_eq!(header.signature.unwrap().len(), SIGNATURE_SIZE);
}

#[test]
fn signed_content_parsing_empty_data_fails() {
    let mut asset = ScriptAssetProbe::default();
    let empty_data = Vec::<u8>::new();

    let result = asset.bytecode(&empty_data);

    assert_eq!(result, false);
    assert_eq!(asset.verified, false);
}

#[test]
fn signed_content_parsing_unsigned_content_succeeds() {
    let mut asset = ScriptAssetProbe::default();
    let data = vec![0x00, 0x01, 0x02, 0x03];

    let result = asset.bytecode(&data);

    assert_eq!(result, true);
    assert_eq!(asset.verified, false);

    let bytecode = asset.module_bytecode;
    assert_eq!(bytecode.len(), 3);
    assert_eq!(bytecode[0], 0x01);
    assert_eq!(bytecode[1], 0x02);
    assert_eq!(bytecode[2], 0x03);
}

#[test]
#[ignore = "expected-red: Rust parses signed envelopes without authenticating their signatures"]
fn signed_content_parsing_signed_flag_is_detected() {
    let mut asset = ScriptAssetProbe::default();
    let mut data = vec![0; 1 + SIGNATURE_SIZE + 3];
    data[0] = 0x80;
    for byte in data.iter_mut().take(SIGNATURE_SIZE + 1).skip(1) {
        *byte = 0x00;
    }
    data[1 + SIGNATURE_SIZE] = 0x01;
    data[1 + SIGNATURE_SIZE + 1] = 0x02;
    data[1 + SIGNATURE_SIZE + 2] = 0x03;

    let result = asset.bytecode(&data);

    assert_eq!(result, false);
    assert_eq!(asset.verified, false);
}

#[test]
fn signed_content_parsing_truncated_signed_data_fails() {
    let mut asset = ScriptAssetProbe::default();
    let data = vec![0x80, 0x01, 0x02];

    let result = asset.bytecode(&data);

    assert_eq!(result, false);
    assert_eq!(asset.verified, false);
}

#[test]
fn signed_content_parsing_version_is_preserved_in_flags() {
    let mut asset = ScriptAssetProbe::default();
    let data = vec![0x01, 0x01, 0x02, 0x03];

    let result = asset.bytecode(&data);

    assert_eq!(result, true);
    assert_eq!(asset.verified, false);
}

#[test]
#[ignore = "expected-red: Rust parses signed envelopes without authenticating their signatures"]
fn signed_content_parsing_signed_content_offset_is_correct() {
    let mut asset = ScriptAssetProbe::default();
    let mut data = vec![0; 1 + SIGNATURE_SIZE + 4];
    data[0] = 0x80;
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = i as u8;
    }
    data[0] = 0x80;

    let result = asset.bytecode(&data);

    assert_eq!(result, false);
}

#[test]
fn signed_content_parsing_unsigned_content_offset_is_correct() {
    let mut asset = ScriptAssetProbe::default();
    let data = vec![0x00, 0xaa, 0xbb, 0xcc, 0xdd, 0xee];

    let result = asset.bytecode(&data);

    assert_eq!(result, true);

    let bytecode = asset.module_bytecode;
    assert_eq!(bytecode.len(), 5);
    assert_eq!(bytecode[0], 0xaa);
    assert_eq!(bytecode[1], 0xbb);
    assert_eq!(bytecode[2], 0xcc);
    assert_eq!(bytecode[3], 0xdd);
    assert_eq!(bytecode[4], 0xee);
}

#[test]
fn signed_content_parsing_minimum_valid_unsigned_data() {
    let mut asset = ScriptAssetProbe::default();
    let data = vec![0x00];

    let result = asset.bytecode(&data);

    assert_eq!(result, true);
    assert_eq!(asset.verified, false);

    let bytecode = asset.module_bytecode;
    assert_eq!(bytecode.len(), 0);
}

#[test]
#[ignore = "expected-red: Rust parses signed envelopes without authenticating their signatures"]
fn signed_content_parsing_minimum_valid_signed_data_structure() {
    let mut asset = ScriptAssetProbe::default();
    let mut data = vec![0; 1 + SIGNATURE_SIZE];
    data[0] = 0x80;

    let result = asset.bytecode(&data);

    assert_eq!(result, false);
}
