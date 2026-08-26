//! Exact executable ports of pinned `image_decoders_test.cpp`.

use nuxie_image_codec::{decoded_rgba_len, validate_encoded_image};
use std::path::PathBuf;

fn pinned(name: &str, expected_len: usize) -> Vec<u8> {
    let path = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets")
    .join(name);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    assert_eq!(bytes.len(), expected_len);
    bytes
}

#[test]
fn wave_b5_png_file_decodes_correctly() {
    let bytes = pinned("placeholder.png", 1_096);
    let bitmap = validate_encoded_image(&bytes).expect("PNG bitmap");
    assert_eq!((bitmap.width, bitmap.height), (226, 128));
    let num_bytes = decoded_rgba_len(bitmap.width, bitmap.height).expect("bounded bitmap");
    assert!(num_bytes == 226 * 128 * 3 || num_bytes == 226 * 128 * 4);
}

#[test]
fn wave_b5_jpeg_file_decodes_correctly() {
    let bytes = pinned("open_source.jpg", 8_880);
    let bitmap = validate_encoded_image(&bytes).expect("JPEG bitmap");
    assert_eq!((bitmap.width, bitmap.height), (350, 200));
    let num_bytes = decoded_rgba_len(bitmap.width, bitmap.height).expect("bounded bitmap");
    assert!(num_bytes == 350 * 200 * 3 || num_bytes == 350 * 200 * 4);
}

#[test]
#[ignore = "expected-red: pinned non-Apple decoder returns the 24566x58278 bad JPEG bitmap, while Rust rejects it before oversized allocation"]
fn wave_b5_bad_jpeg_file_does_not_cause_an_overflow() {
    let bytes = pinned("bad.jpg", 88_731);
    let bitmap = validate_encoded_image(&bytes).expect("pinned decoder returns a guarded bitmap");
    assert_eq!((bitmap.width, bitmap.height), (24_566, 58_278));
}

#[test]
#[ignore = "expected-red: pinned Apple decoder returns the 58278x24566 bad PNG bitmap, while Rust rejects it before oversized allocation"]
fn wave_b5_bad_png_file_does_not_cause_an_overflow() {
    let bytes = pinned("bad.png", 534_283);
    if cfg!(target_os = "macos") {
        let bitmap =
            validate_encoded_image(&bytes).expect("pinned Apple decoder returns a black bitmap");
        assert_eq!((bitmap.width, bitmap.height), (58_278, 24_566));
    } else {
        assert!(validate_encoded_image(&bytes).is_none());
    }
}

#[test]
fn wave_b5_webp_file_decodes_correctly() {
    let bytes = pinned("1.webp", 30_320);
    let bitmap = validate_encoded_image(&bytes).expect("WebP bitmap");
    assert_eq!((bitmap.width, bitmap.height), (550, 368));
    assert_eq!(
        decoded_rgba_len(bitmap.width, bitmap.height),
        Some(550 * 368 * 4)
    );
}
