//! Self-contained encoded-image validation shared by headless artifact builds.
//!
//! This crate deliberately excludes renderer backends. A consumer that only
//! needs to prove PNG, JPEG, or WebP bytes can be decoded must not pull WebGPU,
//! WebGL, or their JavaScript host ABI into a raw `wasm32-unknown-unknown`
//! module.

use std::io::Cursor;

/// Dimensions proven by a complete decode of a supported encoded image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// Fully decode a PNG, JPEG, or WebP and validate that its decoded sample
/// buffer is structurally complete before returning its dimensions.
///
/// Header-only inspection is intentionally insufficient here: truncated and
/// corrupt payloads must fail before a publisher emits an artifact.
#[must_use]
pub fn validate_encoded_image(data: &[u8]) -> Option<DecodedImageDimensions> {
    let (width, height, decoded_len, samples_per_pixel) = if data.starts_with(b"\x89PNG\r\n\x1a\n")
    {
        decode_png(data)?
    } else if data.starts_with(&[0xff, 0xd8]) {
        decode_jpeg(data)?
    } else if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        decode_webp(data)?
    } else {
        return None;
    };

    if width == 0 || height == 0 {
        return None;
    }
    let expected_decoded_len = decoded_sample_len(width, height, samples_per_pixel)?;
    if decoded_len != expected_decoded_len {
        return None;
    }
    // Prove that the canonical RGBA representation used downstream is also
    // representable without overflowing an address-sized allocation.
    decoded_sample_len(width, height, 4)?;

    Some(DecodedImageDimensions { width, height })
}

fn decode_png(data: &[u8]) -> Option<(u32, u32, usize, usize)> {
    let mut decoder = png::Decoder::new(Cursor::new(data));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut decoded = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut decoded).ok()?;
    let samples_per_pixel = match (info.color_type, info.bit_depth) {
        (png::ColorType::Grayscale, png::BitDepth::Eight) => 1,
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => 2,
        (png::ColorType::Rgb, png::BitDepth::Eight) => 3,
        (png::ColorType::Rgba, png::BitDepth::Eight) => 4,
        _ => return None,
    };
    Some((
        info.width,
        info.height,
        info.buffer_size(),
        samples_per_pixel,
    ))
}

fn decode_jpeg(data: &[u8]) -> Option<(u32, u32, usize, usize)> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(data));
    let decoded = decoder.decode().ok()?;
    let info = decoder.info()?;
    let samples_per_pixel = match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => 1,
        jpeg_decoder::PixelFormat::L16 => 2,
        jpeg_decoder::PixelFormat::RGB24 => 3,
        jpeg_decoder::PixelFormat::CMYK32 => 4,
    };
    Some((
        u32::from(info.width),
        u32::from(info.height),
        decoded.len(),
        samples_per_pixel,
    ))
}

fn decode_webp(data: &[u8]) -> Option<(u32, u32, usize, usize)> {
    let mut decoder = image_webp::WebPDecoder::new(Cursor::new(data)).ok()?;
    let (width, height) = decoder.dimensions();
    let samples_per_pixel = if decoder.has_alpha() { 4 } else { 3 };
    let mut decoded = vec![0; decoder.output_buffer_size()?];
    decoder.read_image(&mut decoded).ok()?;
    Some((width, height, decoded.len(), samples_per_pixel))
}

fn decoded_sample_len(width: u32, height: u32, samples_per_pixel: usize) -> Option<usize> {
    usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?
        .checked_mul(samples_per_pixel)
}

#[cfg(test)]
mod tests {
    use super::{DecodedImageDimensions, validate_encoded_image};

    #[test]
    fn validates_fully_decoded_webp_dimensions() {
        let mut encoded = Vec::new();
        image_webp::WebPEncoder::new(&mut encoded)
            .encode(
                &[240, 120, 60, 128, 10, 20, 30, 255],
                2,
                1,
                image_webp::ColorType::Rgba8,
            )
            .expect("fixture encodes");

        assert_eq!(
            validate_encoded_image(&encoded),
            Some(DecodedImageDimensions {
                width: 2,
                height: 1,
            })
        );
    }

    #[test]
    fn rejects_truncated_header_only_png() {
        let mut encoded = vec![0; 24];
        encoded[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        encoded[12..16].copy_from_slice(b"IHDR");
        encoded[16..20].copy_from_slice(&3_u32.to_be_bytes());
        encoded[20..24].copy_from_slice(&5_u32.to_be_bytes());

        assert_eq!(validate_encoded_image(&encoded), None);
    }

    #[test]
    fn rejects_unsupported_and_empty_payloads() {
        assert_eq!(validate_encoded_image(b"not an image"), None);
        assert_eq!(validate_encoded_image(&[]), None);
    }
}
