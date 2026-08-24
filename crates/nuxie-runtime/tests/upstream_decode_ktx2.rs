//! One-for-one expected-red ports of all eleven cases in pinned
//! `tests/unit_tests/runtime/decode_ktx2_test.cpp`.
//!
//! Rust currently has no KTX2/BC7 decoder owner. The complete synthetic byte
//! streams and every rejection/happy-path assertion remain explicit here.

const KTX2_IDENTIFIER: [u8; 12] = [
    0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a,
];
const VK_FORMAT_BC7_SRGB_BLOCK: u32 = 146;

#[derive(Debug, Default, PartialEq, Eq)]
enum TextureFormat {
    #[default]
    Unknown,
    Bc7,
}

#[derive(Debug, Default)]
struct Ktx2DecodeResult {
    format: TextureFormat,
    pixel_width: u32,
    pixel_height: u32,
    level_count: u32,
    blocks: Vec<u8>,
}

fn append_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn append_u64(buffer: &mut Vec<u8>, value: u64) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn build_skeleton_ktx2(
    vk_format: u32,
    pixel_width: u32,
    pixel_height: u32,
    level_count: u32,
    supercompression_scheme: u32,
    face_count: u32,
    layer_count: u32,
) -> Vec<u8> {
    let mut buffer = KTX2_IDENTIFIER.to_vec();
    append_u32(&mut buffer, vk_format);
    append_u32(&mut buffer, 1);
    append_u32(&mut buffer, pixel_width);
    append_u32(&mut buffer, pixel_height);
    append_u32(&mut buffer, 0);
    append_u32(&mut buffer, layer_count);
    append_u32(&mut buffer, face_count);
    append_u32(&mut buffer, level_count);
    append_u32(&mut buffer, supercompression_scheme);
    append_u32(&mut buffer, 0);
    append_u32(&mut buffer, 0);
    append_u32(&mut buffer, 0);
    append_u32(&mut buffer, 0);
    append_u64(&mut buffer, 0);
    append_u64(&mut buffer, 0);
    buffer
}

fn decode_ktx2(_: &[u8], _: &mut Ktx2DecodeResult) -> bool {
    panic!("Rust has no production KTX2/BC7 decoder owner")
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_rejects_buffer_smaller_than_identifier_and_header() {
    let buffer = vec![0; 40];
    let mut output = Ktx2DecodeResult::default();
    assert!(!decode_ktx2(&buffer, &mut output));
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_rejects_bad_magic() {
    let mut buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 1, 0);
    buffer[0] = b'X';
    let mut output = Ktx2DecodeResult::default();
    assert!(!decode_ktx2(&buffer, &mut output));
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_rejects_unsupported_vk_format() {
    let buffer = build_skeleton_ktx2(37, 4, 4, 1, 0, 1, 0);
    let mut output = Ktx2DecodeResult::default();
    assert!(!decode_ktx2(&buffer, &mut output));
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_rejects_supercompressed_payload() {
    let buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 2, 1, 0);
    let mut output = Ktx2DecodeResult::default();
    assert!(!decode_ktx2(&buffer, &mut output));
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_rejects_cubemaps_and_array_layers() {
    {
        let buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 6, 0);
        let mut output = Ktx2DecodeResult::default();
        assert!(!decode_ktx2(&buffer, &mut output));
    }
    {
        let buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 1, 4);
        let mut output = Ktx2DecodeResult::default();
        assert!(!decode_ktx2(&buffer, &mut output));
    }
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_rejects_out_of_range_dimensions() {
    {
        let buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 0, 4, 1, 0, 1, 0);
        let mut output = Ktx2DecodeResult::default();
        assert!(!decode_ktx2(&buffer, &mut output));
    }
    {
        let buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 999_999, 1, 0, 1, 0);
        let mut output = Ktx2DecodeResult::default();
        assert!(!decode_ktx2(&buffer, &mut output));
    }
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_rejects_truncated_level_index() {
    let buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 1, 0);
    let mut output = Ktx2DecodeResult::default();
    assert!(!decode_ktx2(&buffer, &mut output));
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_rejects_level_pointer_outside_buffer() {
    let mut buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 1, 0);
    append_u64(&mut buffer, 1_u64 << 32);
    append_u64(&mut buffer, 16);
    append_u64(&mut buffer, 16);
    let mut output = Ktx2DecodeResult::default();
    assert!(!decode_ktx2(&buffer, &mut output));
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_rejects_byte_length_inconsistent_with_logical_block_grid() {
    let mut buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 1, 0);
    let level_offset = buffer.len() as u64 + 24;
    append_u64(&mut buffer, level_offset);
    append_u64(&mut buffer, 32);
    append_u64(&mut buffer, 32);
    buffer.resize(buffer.len() + 32, 0);
    let mut output = Ktx2DecodeResult::default();
    assert!(!decode_ktx2(&buffer, &mut output));
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_happy_path_single_four_by_four_bc7_mip_zero() {
    let mut buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 1, 0);
    let level_offset = buffer.len() as u64 + 24;
    append_u64(&mut buffer, level_offset);
    append_u64(&mut buffer, 16);
    append_u64(&mut buffer, 16);
    let expected = [
        0xde, 0xad, 0xbe, 0xef, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0xca, 0xfe, 0xba,
        0xbe,
    ];
    buffer.extend_from_slice(&expected);

    let mut output = Ktx2DecodeResult::default();
    assert!(decode_ktx2(&buffer, &mut output));
    assert_eq!(output.format, TextureFormat::Bc7);
    assert_eq!(output.pixel_width, 4);
    assert_eq!(output.pixel_height, 4);
    assert_eq!(output.level_count, 1);
    assert_eq!(output.blocks.len(), 16);
    assert_eq!(&output.blocks[..], &expected);
}

#[test]
#[ignore = "expected-red: Rust has no KTX2/BC7 decoder owner"]
fn ktx2_happy_path_eight_by_eight_with_two_mip_levels_concatenated() {
    let mut buffer = build_skeleton_ktx2(VK_FORMAT_BC7_SRGB_BLOCK, 8, 8, 2, 0, 1, 0);
    let header_end = buffer.len() as u64;
    let level_index_bytes = 24_u64 * 2;
    let mip0_offset = header_end + level_index_bytes;
    let mip0_bytes = 64_u64;
    let mip1_offset = mip0_offset + mip0_bytes;
    let mip1_bytes = 16_u64;
    append_u64(&mut buffer, mip0_offset);
    append_u64(&mut buffer, mip0_bytes);
    append_u64(&mut buffer, mip0_bytes);
    append_u64(&mut buffer, mip1_offset);
    append_u64(&mut buffer, mip1_bytes);
    append_u64(&mut buffer, mip1_bytes);
    buffer.resize(buffer.len() + mip0_bytes as usize, 0xaa);
    buffer.resize(buffer.len() + mip1_bytes as usize, 0xbb);

    let mut output = Ktx2DecodeResult::default();
    assert!(decode_ktx2(&buffer, &mut output));
    assert_eq!(output.pixel_width, 8);
    assert_eq!(output.pixel_height, 8);
    assert_eq!(output.level_count, 2);
    assert_eq!(output.blocks.len(), (mip0_bytes + mip1_bytes) as usize);
    assert_eq!(output.blocks[0], 0xaa);
    assert_eq!(output.blocks[mip0_bytes as usize - 1], 0xaa);
    assert_eq!(output.blocks[mip0_bytes as usize], 0xbb);
    assert_eq!(output.blocks[(mip0_bytes + mip1_bytes) as usize - 1], 0xbb);
}
