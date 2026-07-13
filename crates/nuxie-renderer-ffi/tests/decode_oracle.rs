use nuxie_renderer_ffi::decode_bitmap_rgba;
use sha2::{Digest, Sha256};

const JPEG_SHA256: &str = "62e087df734fa3a0f57524db98a4d5aa30a8628ede9a7d59ed67981cc71823de";
const ICC_PNG_SHA256: &str = "a72cd2314ae2cb861da62dfb9782e323337fc600e44200cd16bd150d7c15f2cb";

#[derive(Debug, PartialEq, Eq)]
struct DecodeDelta {
    differing_pixels: usize,
    pixels_over_delta_2: usize,
    differing_channels: usize,
    max_delta: u8,
    alpha_mismatches: usize,
}

fn compare_decode(encoded: &[u8], expected_dimensions: (u32, u32)) -> DecodeDelta {
    let cpp = decode_bitmap_rgba(encoded).expect("C++ image decode");
    let rust = nuxie_renderer::decode_image_rgba_for_oracle(encoded).expect("Rust image decode");
    assert_eq!((cpp.width, cpp.height), expected_dimensions);
    assert_eq!((rust.width, rust.height), expected_dimensions);
    assert_eq!(cpp.pixels.len(), rust.pixels.len());

    let mut result = DecodeDelta {
        differing_pixels: 0,
        pixels_over_delta_2: 0,
        differing_channels: 0,
        max_delta: 0,
        alpha_mismatches: 0,
    };
    for (cpp, rust) in cpp.pixels.chunks_exact(4).zip(rust.pixels.chunks_exact(4)) {
        let mut pixel_differs = false;
        let mut pixel_exceeds_threshold = false;
        for channel in 0..4 {
            let delta = cpp[channel].abs_diff(rust[channel]);
            if delta != 0 {
                result.differing_channels += 1;
                pixel_differs = true;
                result.max_delta = result.max_delta.max(delta);
                if channel == 3 {
                    result.alpha_mismatches += 1;
                }
            }
            pixel_exceeds_threshold |= delta > 2;
        }
        result.differing_pixels += usize::from(pixel_differs);
        result.pixels_over_delta_2 += usize::from(pixel_exceeds_threshold);
    }
    result
}

fn assert_sha256(data: &[u8], expected: &str) {
    assert_eq!(format!("{:x}", Sha256::digest(data)), expected);
}

fn reachable_jpeg() -> Vec<u8> {
    let encoded =
        include_str!("../../../fixtures/renderer/streams/riv/clipping_and_draw_order.rive-stream")
            .lines()
            .find_map(|line| line.strip_prefix("decodeImage "))
            .and_then(|line| line.split_once("data="))
            .map(|(_, hex)| {
                hex.as_bytes()
                    .chunks_exact(2)
                    .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                    .collect::<Vec<_>>()
            })
            .expect("reachable JPEG fixture");
    assert!(encoded.starts_with(&[0xff, 0xd8, 0xff]));
    assert_sha256(&encoded, JPEG_SHA256);
    encoded
}

fn icc_png() -> Vec<u8> {
    let stream = nuxie_render_stream::RenderStream::parse(include_str!(
        "../../../fixtures/renderer/streams/gm/image_aa_border.rive-stream"
    ))
    .expect("ICC stream");
    let encoded = stream
        .resources
        .into_iter()
        .find_map(|resource| match resource {
            nuxie_render_stream::Resource::Image { data, .. } => Some(data),
            _ => None,
        })
        .expect("ICC PNG fixture");
    assert!(encoded.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(encoded.windows(4).any(|bytes| bytes == b"iCCP"));
    assert_sha256(&encoded, ICC_PNG_SHA256);
    encoded
}

#[test]
fn production_decode_contracts() {
    let jpeg = compare_decode(&reachable_jpeg(), (278, 278));
    eprintln!("reachable JPEG decode delta: {jpeg:?}");
    assert_eq!(jpeg.alpha_mismatches, 0);
    assert!(jpeg.differing_pixels > 0);
    assert!(jpeg.differing_pixels <= 40_000);
    assert!(jpeg.pixels_over_delta_2 <= 40_000);
    assert!(jpeg.differing_channels <= 90_000);
    assert!(jpeg.max_delta <= 40);

    let icc_png = compare_decode(&icc_png(), (319, 320));
    eprintln!("ICC PNG decode delta: {icc_png:?}");
    assert_eq!(icc_png.alpha_mismatches, 0);
    assert_eq!(icc_png.pixels_over_delta_2, 0);
    assert!(icc_png.differing_pixels <= 6_000);
    assert!(icc_png.differing_channels <= 6_000);
    assert!(icc_png.max_delta <= 2);

    assert!(decode_bitmap_rgba(b"not an image").is_none());
}
