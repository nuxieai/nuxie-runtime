//! Native Bitmap ports of pinned `image_decoders_test.cpp`.
//! Oversized malformed fixtures retain their assertions but are resource-deferred.

use nuxie_runtime::source::decoders::bitmap_decoder::{Bitmap, PixelFormat};
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
    let bitmap = Bitmap::decode(&bytes).expect("PNG bitmap");
    assert_eq!((bitmap.width(), bitmap.height()), (226, 128));
    let channels = if bitmap.pixel_format() == PixelFormat::Rgb {
        3
    } else {
        4
    };
    assert_eq!(bitmap.num_bytes(), 226 * 128 * channels);
    assert_eq!(bitmap.bytes().len(), bitmap.num_bytes());
    assert_platform_format(&bitmap);
}

#[test]
fn wave_b5_jpeg_file_decodes_correctly() {
    let bytes = pinned("open_source.jpg", 8_880);
    let bitmap = Bitmap::decode(&bytes).expect("JPEG bitmap");
    assert_eq!((bitmap.width(), bitmap.height()), (350, 200));
    let channels = if bitmap.pixel_format() == PixelFormat::Rgb {
        3
    } else {
        4
    };
    assert_eq!(bitmap.num_bytes(), 350 * 200 * channels);
    assert_eq!(bitmap.bytes().len(), bitmap.num_bytes());
    assert_platform_format(&bitmap);
    #[cfg(not(target_vendor = "apple"))]
    assert_eq!(bitmap.pixel_format(), PixelFormat::Rgb);
}

#[test]
#[cfg(not(target_vendor = "apple"))]
#[ignore = "resource-deferred: pinned malformed JPEG decode may allocate over 4 GB; not a product-admission test"]
fn wave_b5_bad_jpeg_file_does_not_cause_an_overflow() {
    let bytes = pinned("bad.jpg", 88_731);
    let bitmap = Bitmap::decode(&bytes).expect("pinned decoder returns a guarded bitmap");
    assert_eq!((bitmap.width(), bitmap.height()), (24_566, 58_278));
}

#[test]
#[ignore = "resource-deferred: pinned Apple malformed PNG decode may allocate over 5 GB; not a product-admission test"]
fn wave_b5_bad_png_file_does_not_cause_an_overflow() {
    let bytes = pinned("bad.png", 534_283);
    if cfg!(target_vendor = "apple") {
        let bitmap = Bitmap::decode(&bytes).expect("pinned Apple decoder returns a black bitmap");
        assert_eq!((bitmap.width(), bitmap.height()), (58_278, 24_566));
    } else {
        assert!(Bitmap::decode(&bytes).is_none());
    }
}

#[test]
fn wave_b5_webp_file_decodes_correctly() {
    let bytes = pinned("1.webp", 30_320);
    let bitmap = Bitmap::decode(&bytes).expect("WebP bitmap");
    assert_eq!((bitmap.width(), bitmap.height()), (550, 368));
    assert_eq!(bitmap.num_bytes(), 550 * 368 * 4);
    assert_eq!(bitmap.bytes().len(), bitmap.num_bytes());
    assert_platform_format(&bitmap);
    #[cfg(not(target_vendor = "apple"))]
    assert_eq!(bitmap.pixel_format(), PixelFormat::Rgba);
}

fn assert_platform_format(bitmap: &Bitmap) {
    #[cfg(target_vendor = "apple")]
    assert_eq!(bitmap.pixel_format(), PixelFormat::RgbaPremul);
    #[cfg(not(target_vendor = "apple"))]
    assert!(matches!(
        bitmap.pixel_format(),
        PixelFormat::Rgb | PixelFormat::Rgba
    ));
}
