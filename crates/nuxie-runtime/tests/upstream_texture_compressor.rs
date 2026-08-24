//! One-for-one expected-red ports of pinned
//! `tests/unit_tests/runtime/texture_compressor_test.cpp`.
//!
//! The upstream tests exercise the standalone `write_ktx2` test/tool helper,
//! which has no Rust production owner. The complete seven bodies remain here
//! so the source-correspondence phase cannot silently omit that boundary.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const VK_FORMAT_BC7_UNORM_BLOCK: u32 = 145;
const VK_FORMAT_BC7_SRGB_BLOCK: u32 = 146;

#[derive(Clone, Copy)]
enum ColorMode {
    Srgb,
    Linear,
}

struct Ktx2Mip {
    pixel_width: u32,
    pixel_height: u32,
    blocks: Vec<u8>,
}

fn temp_path(suffix: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "rive_ktx2_test_{}{}",
        COUNTER.fetch_add(1, Ordering::Relaxed),
        suffix
    ))
}

fn read_file(path: &PathBuf) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn read_u32(buffer: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(buffer[offset..offset + 4].try_into().expect("u32 bytes"))
}

fn read_u64(buffer: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(buffer[offset..offset + 8].try_into().expect("u64 bytes"))
}

fn dummy_mip(width: u32, height: u32) -> Ktx2Mip {
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);
    Ktx2Mip {
        pixel_width: width,
        pixel_height: height,
        blocks: vec![0xab; (blocks_x * blocks_y * 16) as usize],
    }
}

fn write_ktx2_bc7(_: &PathBuf, mips: &[Ktx2Mip], _: ColorMode) -> bool {
    let _logical_dimensions = mips.first().map(|mip| (mip.pixel_width, mip.pixel_height));
    panic!("Rust has no owner corresponding to the upstream write_ktx2 tool helper")
}

#[test]
#[ignore = "expected-red: Rust has no write_ktx2 tool-helper owner"]
fn write_ktx2_bc7_writes_ktx2_identifier() {
    let path = temp_path(".ktx2");
    let mips = vec![dummy_mip(4, 4)];
    assert!(write_ktx2_bc7(&path, &mips, ColorMode::Srgb));

    let buffer = read_file(&path);
    assert!(buffer.len() >= 12);
    let expected = [
        0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a,
    ];
    assert_eq!(&buffer[..12], expected);
    std::fs::remove_file(path).expect("remove temporary KTX2");
}

#[test]
#[ignore = "expected-red: Rust has no write_ktx2 tool-helper owner"]
fn write_ktx2_bc7_srgb_selects_bc7_srgb_block() {
    let path = temp_path(".ktx2");
    let mips = vec![dummy_mip(8, 8)];
    assert!(write_ktx2_bc7(&path, &mips, ColorMode::Srgb));

    let buffer = read_file(&path);
    assert_eq!(read_u32(&buffer, 12), VK_FORMAT_BC7_SRGB_BLOCK);
    std::fs::remove_file(path).expect("remove temporary KTX2");
}

#[test]
#[ignore = "expected-red: Rust has no write_ktx2 tool-helper owner"]
fn write_ktx2_bc7_linear_selects_bc7_unorm_block() {
    let path = temp_path(".ktx2");
    let mips = vec![dummy_mip(8, 8)];
    assert!(write_ktx2_bc7(&path, &mips, ColorMode::Linear));

    let buffer = read_file(&path);
    assert_eq!(read_u32(&buffer, 12), VK_FORMAT_BC7_UNORM_BLOCK);
    std::fs::remove_file(path).expect("remove temporary KTX2");
}

#[test]
#[ignore = "expected-red: Rust has no write_ktx2 tool-helper owner"]
fn write_ktx2_bc7_header_records_logical_mip_0_dimensions() {
    let path = temp_path(".ktx2");
    let mip = Ktx2Mip {
        pixel_width: 5,
        pixel_height: 3,
        blocks: vec![0; 2 * 16],
    };
    assert!(write_ktx2_bc7(&path, &[mip], ColorMode::Srgb));

    let buffer = read_file(&path);
    assert_eq!(read_u32(&buffer, 20), 5);
    assert_eq!(read_u32(&buffer, 24), 3);
    std::fs::remove_file(path).expect("remove temporary KTX2");
}

#[test]
#[ignore = "expected-red: Rust has no write_ktx2 tool-helper owner"]
fn write_ktx2_bc7_level_count_matches_mip_count() {
    let path = temp_path(".ktx2");
    let mips = vec![
        dummy_mip(8, 8),
        dummy_mip(4, 4),
        dummy_mip(2, 2),
        dummy_mip(1, 1),
    ];
    assert!(write_ktx2_bc7(&path, &mips, ColorMode::Srgb));

    let buffer = read_file(&path);
    assert_eq!(read_u32(&buffer, 40), 4);
    std::fs::remove_file(path).expect("remove temporary KTX2");
}

#[test]
#[ignore = "expected-red: Rust has no write_ktx2 tool-helper owner"]
fn write_ktx2_bc7_level_data_offsets_are_16_byte_aligned() {
    let path = temp_path(".ktx2");
    let mips = vec![
        dummy_mip(8, 8),
        dummy_mip(4, 4),
        dummy_mip(2, 2),
        dummy_mip(1, 1),
    ];
    assert!(write_ktx2_bc7(&path, &mips, ColorMode::Srgb));

    let buffer = read_file(&path);
    const LEVEL_INDEX: usize = 80;
    const ENTRY_BYTES: usize = 24;
    for (index, mip) in mips.iter().enumerate() {
        let byte_offset = read_u64(&buffer, LEVEL_INDEX + index * ENTRY_BYTES);
        assert_eq!(byte_offset & 0xf, 0);
        let byte_length = read_u64(&buffer, LEVEL_INDEX + index * ENTRY_BYTES + 8);
        assert_eq!(byte_length, mip.blocks.len() as u64);
    }
    std::fs::remove_file(path).expect("remove temporary KTX2");
}

#[test]
#[ignore = "expected-red: Rust has no write_ktx2 tool-helper owner"]
fn write_ktx2_bc7_rejects_empty_mip_list() {
    let path = temp_path(".ktx2");
    assert!(!write_ktx2_bc7(&path, &[], ColorMode::Srgb));
    let _ = std::fs::remove_file(path);
}
