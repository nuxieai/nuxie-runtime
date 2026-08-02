//! Self-contained encoded-image validation shared by headless artifact builds.
//!
//! This crate deliberately excludes renderer backends. A consumer that only
//! needs to prove PNG, JPEG, or WebP bytes can be decoded must not pull WebGPU,
//! WebGL, or their JavaScript host ABI into a raw `wasm32-unknown-unknown`
//! module.

use std::io::Cursor;

/// Largest encoded payload accepted by the built-in image decoders.
///
/// The caller already owns the input slice, but bounding it also bounds codec
/// metadata tables and profile copies whose size is controlled by the input.
pub const MAX_ENCODED_IMAGE_BYTES: usize = 64 * 1024 * 1024;

/// Largest supported width or height for an imported image.
pub const MAX_IMAGE_DIMENSION: u32 = 8_192;

/// Largest canonical RGBA allocation accepted for one imported image.
pub const MAX_DECODED_IMAGE_BYTES: usize = 64 * 1024 * 1024;

/// Dimensions read from a supported encoded image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// Canonical tightly-packed premultiplied RGBA8 pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImageRgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Fully decode a supported encoded image into the same canonical pixel
/// representation used by Rive's `Bitmap::decode` scripting path.
#[must_use]
pub fn decode_image_rgba(data: &[u8]) -> Option<DecodedImageRgba> {
    decode_image_rgba_with_limit(data, Some(MAX_DECODED_IMAGE_BYTES))
}

/// Decode for pinned low-level compatibility surfaces, which do not impose
/// the bounded high-level import policy. Integer overflow and allocation
/// failure remain checked, but there is no product support ceiling.
#[must_use]
pub fn decode_image_rgba_unbounded(data: &[u8]) -> Option<DecodedImageRgba> {
    decode_image_rgba_with_limit(data, None)
}

fn decode_image_rgba_with_limit(
    data: &[u8],
    max_decoded_bytes: Option<usize>,
) -> Option<DecodedImageRgba> {
    if max_decoded_bytes.is_some() && data.len() > MAX_ENCODED_IMAGE_BYTES {
        return None;
    }
    let expected = preflight_encoded_image_with_limit(data, max_decoded_bytes)?;
    let (width, height, pixels) = if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        decode_png_rgba(data, max_decoded_bytes)?
    } else if data.starts_with(&[0xff, 0xd8]) {
        decode_jpeg_rgba(data, max_decoded_bytes)?
    } else if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        decode_webp_rgba(data, max_decoded_bytes)?
    } else {
        return None;
    };
    if (width, height) != (expected.width, expected.height)
        || pixels.len() != decoded_rgba_len_with_limit(width, height, max_decoded_bytes)?
    {
        return None;
    }
    Some(DecodedImageRgba {
        width,
        height,
        pixels,
    })
}

/// Inspect the header of a supported image and enforce the import resource
/// policy without allocating a decoded pixel buffer.
///
/// This does not prove that the complete payload is valid. Call
/// [`validate_encoded_image`] when complete decoding is required.
#[must_use]
pub fn preflight_encoded_image(data: &[u8]) -> Option<DecodedImageDimensions> {
    if data.len() > MAX_ENCODED_IMAGE_BYTES {
        return None;
    }

    let (width, height) = if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        preflight_png(data)?
    } else if data.starts_with(&[0xff, 0xd8]) {
        preflight_jpeg(data)?
    } else if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        preflight_webp(data)?
    } else {
        return None;
    };
    decoded_rgba_len(width, height)?;
    Some(DecodedImageDimensions { width, height })
}

fn preflight_encoded_image_with_limit(
    data: &[u8],
    max_decoded_bytes: Option<usize>,
) -> Option<DecodedImageDimensions> {
    let (width, height) = if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        preflight_png_with_limit(data, max_decoded_bytes)?
    } else if data.starts_with(&[0xff, 0xd8]) {
        preflight_jpeg_with_limit(data, max_decoded_bytes)?
    } else if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        preflight_webp_with_limit(data, max_decoded_bytes)?
    } else {
        return None;
    };
    Some(DecodedImageDimensions { width, height })
}

/// Return the canonical RGBA byte length when dimensions satisfy the image
/// import resource policy.
#[must_use]
pub fn decoded_rgba_len(width: u32, height: u32) -> Option<usize> {
    if width == 0 || height == 0 || width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        return None;
    }
    let decoded_len = decoded_sample_len(width, height, 4)?;
    (decoded_len <= MAX_DECODED_IMAGE_BYTES).then_some(decoded_len)
}

fn decoded_rgba_len_with_limit(
    width: u32,
    height: u32,
    max_decoded_bytes: Option<usize>,
) -> Option<usize> {
    if width == 0 || height == 0 {
        return None;
    }
    let decoded_len = decoded_sample_len(width, height, 4)?;
    max_decoded_bytes
        .is_none_or(|limit| decoded_len <= limit)
        .then_some(decoded_len)
}

/// Fully decode a PNG, JPEG, or WebP and validate that its decoded sample
/// buffer is structurally complete before returning its dimensions.
///
/// Header-only inspection is intentionally insufficient here: truncated and
/// corrupt payloads must fail before a publisher emits an artifact.
#[must_use]
pub fn validate_encoded_image(data: &[u8]) -> Option<DecodedImageDimensions> {
    let expected = preflight_encoded_image(data)?;
    let (width, height, decoded_len, samples_per_pixel) = if data.starts_with(b"\x89PNG\r\n\x1a\n")
    {
        decode_png(data)?
    } else if data.starts_with(&[0xff, 0xd8]) {
        decode_jpeg(data)?
    } else {
        decode_webp(data)?
    };

    if (width, height) != (expected.width, expected.height) {
        return None;
    }
    let expected_decoded_len = decoded_sample_len(width, height, samples_per_pixel)?;
    if decoded_len != expected_decoded_len {
        return None;
    }
    decoded_rgba_len(width, height)?;

    Some(DecodedImageDimensions { width, height })
}

fn decode_png_rgba(data: &[u8], max_decoded_bytes: Option<usize>) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = png_decoder_with_limit(data, max_decoded_bytes);
    let info = decoder.read_header_info().ok()?;
    decoded_rgba_len_with_limit(info.width, info.height, max_decoded_bytes)?;
    let mut reader = decoder.read_info().ok()?;
    let icc_profile = reader
        .info()
        .icc_profile
        .as_ref()
        .map(|profile| profile.as_ref().to_vec());
    let output_buffer_size = reader.output_buffer_size()?;
    if max_decoded_bytes.is_some_and(|limit| output_buffer_size > limit) {
        return None;
    }
    let mut decoded = zeroed_buffer(output_buffer_size)?;
    let info = reader.next_frame(&mut decoded).ok()?;
    decoded.truncate(info.buffer_size());
    let mut pixels = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => decoded,
        (png::ColorType::Rgb, png::BitDepth::Eight) => decoded
            .chunks_exact(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect(),
        (png::ColorType::Grayscale, png::BitDepth::Eight) => decoded
            .into_iter()
            .flat_map(|value| [value, value, value, 255])
            .collect(),
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => decoded
            .chunks_exact(2)
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),
        _ => return None,
    };
    if let Some(profile) = icc_profile {
        convert_icc_rgba_to_srgb(&mut pixels, info.width, &profile);
    }
    premultiply_rgba(&mut pixels);
    Some((info.width, info.height, pixels))
}

fn decode_jpeg_rgba(data: &[u8], max_decoded_bytes: Option<usize>) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(data));
    decoder.set_max_decoding_buffer_size(max_decoded_bytes.unwrap_or(usize::MAX));
    decoder.read_info().ok()?;
    let info = decoder.info()?;
    decoded_rgba_len_with_limit(
        u32::from(info.width),
        u32::from(info.height),
        max_decoded_bytes,
    )?;
    let decoded = decoder.decode().ok()?;
    let info = decoder.info()?;
    let icc_profile = decoder.icc_profile();
    let mut pixels: Vec<u8> = match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => decoded
            .into_iter()
            .flat_map(|value| [value, value, value, 255])
            .collect(),
        jpeg_decoder::PixelFormat::L16 => decoded
            .chunks_exact(2)
            .flat_map(|value| [value[0], value[0], value[0], 255])
            .collect(),
        jpeg_decoder::PixelFormat::RGB24 => decoded
            .chunks_exact(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect(),
        jpeg_decoder::PixelFormat::CMYK32 => decoded
            .chunks_exact(4)
            .flat_map(|cmyk| {
                let key = u16::from(cmyk[3]);
                [
                    ((u16::from(cmyk[0]) * key + 127) / 255) as u8,
                    ((u16::from(cmyk[1]) * key + 127) / 255) as u8,
                    ((u16::from(cmyk[2]) * key + 127) / 255) as u8,
                    255,
                ]
            })
            .collect(),
    };
    if let Some(profile) = icc_profile {
        convert_icc_rgba_to_srgb(&mut pixels, u32::from(info.width), &profile);
    }
    premultiply_rgba(&mut pixels);
    Some((u32::from(info.width), u32::from(info.height), pixels))
}

fn decode_webp_rgba(data: &[u8], max_decoded_bytes: Option<usize>) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = image_webp::WebPDecoder::new(Cursor::new(data)).ok()?;
    decoder.set_memory_limit(max_decoded_bytes.unwrap_or(usize::MAX));
    let (width, height) = decoder.dimensions();
    decoded_rgba_len_with_limit(width, height, max_decoded_bytes)?;
    let has_alpha = decoder.has_alpha();
    let icc_profile = decoder.icc_profile().ok()?;
    let output_buffer_size = decoder.output_buffer_size()?;
    if max_decoded_bytes.is_some_and(|limit| output_buffer_size > limit) {
        return None;
    }
    let mut decoded = zeroed_buffer(output_buffer_size)?;
    decoder.read_image(&mut decoded).ok()?;
    let mut pixels = if has_alpha {
        decoded
    } else {
        decoded
            .chunks_exact(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect()
    };
    if let Some(profile) = icc_profile {
        convert_icc_rgba_to_srgb(&mut pixels, width, &profile);
    }
    premultiply_rgba(&mut pixels);
    Some((width, height, pixels))
}

fn convert_icc_rgba_to_srgb(pixels: &mut [u8], width: u32, icc_profile: &[u8]) {
    let Ok(source) = moxcms::ColorProfile::new_from_slice(icc_profile) else {
        return;
    };
    let destination = moxcms::ColorProfile::new_srgb();
    let Ok(transform) = source.create_transform_8bit(
        moxcms::Layout::Rgba,
        &destination,
        moxcms::Layout::Rgba,
        moxcms::TransformOptions::default(),
    ) else {
        return;
    };
    let Some(row_bytes) = usize::try_from(width)
        .ok()
        .and_then(|width| width.checked_mul(4))
    else {
        return;
    };
    if row_bytes == 0 || !pixels.len().is_multiple_of(row_bytes) {
        return;
    }
    let mut converted = vec![0; pixels.len()];
    for (source, destination) in pixels
        .chunks_exact(row_bytes)
        .zip(converted.chunks_exact_mut(row_bytes))
    {
        if transform.transform(source, destination).is_err() {
            return;
        }
    }
    pixels.copy_from_slice(&converted);
}

fn premultiply_rgba(pixels: &mut [u8]) {
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = u16::from(pixel[3]);
        for channel in &mut pixel[..3] {
            *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
        }
    }
}

fn png_decoder(data: &[u8]) -> png::Decoder<Cursor<&[u8]>> {
    png_decoder_with_limit(data, Some(MAX_DECODED_IMAGE_BYTES))
}

fn png_decoder_with_limit(
    data: &[u8],
    max_decoded_bytes: Option<usize>,
) -> png::Decoder<Cursor<&[u8]>> {
    let mut decoder = png::Decoder::new_with_limits(
        Cursor::new(data),
        png::Limits {
            bytes: max_decoded_bytes.unwrap_or(usize::MAX),
        },
    );
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    decoder
}

fn preflight_png_with_limit(data: &[u8], max_decoded_bytes: Option<usize>) -> Option<(u32, u32)> {
    let mut decoder = png_decoder_with_limit(data, max_decoded_bytes);
    let info = decoder.read_header_info().ok()?;
    decoded_rgba_len_with_limit(info.width, info.height, max_decoded_bytes)?;
    Some((info.width, info.height))
}

fn preflight_jpeg_with_limit(data: &[u8], max_decoded_bytes: Option<usize>) -> Option<(u32, u32)> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(data));
    decoder.set_max_decoding_buffer_size(max_decoded_bytes.unwrap_or(usize::MAX));
    decoder.read_info().ok()?;
    let info = decoder.info()?;
    let dimensions = (u32::from(info.width), u32::from(info.height));
    decoded_rgba_len_with_limit(dimensions.0, dimensions.1, max_decoded_bytes)?;
    Some(dimensions)
}

fn preflight_webp_with_limit(data: &[u8], max_decoded_bytes: Option<usize>) -> Option<(u32, u32)> {
    let mut decoder = image_webp::WebPDecoder::new(Cursor::new(data)).ok()?;
    decoder.set_memory_limit(max_decoded_bytes.unwrap_or(usize::MAX));
    let dimensions = decoder.dimensions();
    decoded_rgba_len_with_limit(dimensions.0, dimensions.1, max_decoded_bytes)?;
    Some(dimensions)
}

fn preflight_png(data: &[u8]) -> Option<(u32, u32)> {
    let mut decoder = png_decoder(data);
    let info = decoder.read_header_info().ok()?;
    decoded_rgba_len(info.width, info.height)?;
    Some((info.width, info.height))
}

fn preflight_jpeg(data: &[u8]) -> Option<(u32, u32)> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(data));
    decoder.set_max_decoding_buffer_size(MAX_DECODED_IMAGE_BYTES);
    decoder.read_info().ok()?;
    let info = decoder.info()?;
    let dimensions = (u32::from(info.width), u32::from(info.height));
    decoded_rgba_len(dimensions.0, dimensions.1)?;
    Some(dimensions)
}

fn preflight_webp(data: &[u8]) -> Option<(u32, u32)> {
    let mut decoder = image_webp::WebPDecoder::new(Cursor::new(data)).ok()?;
    decoder.set_memory_limit(MAX_DECODED_IMAGE_BYTES);
    let dimensions = decoder.dimensions();
    decoded_rgba_len(dimensions.0, dimensions.1)?;
    Some(dimensions)
}

fn decode_png(data: &[u8]) -> Option<(u32, u32, usize, usize)> {
    let mut decoder = png_decoder(data);
    let info = decoder.read_header_info().ok()?;
    decoded_rgba_len(info.width, info.height)?;
    let mut reader = decoder.read_info().ok()?;
    let output_buffer_size = reader.output_buffer_size()?;
    if output_buffer_size > MAX_DECODED_IMAGE_BYTES {
        return None;
    }
    let mut decoded = zeroed_buffer(output_buffer_size)?;
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
    decoder.set_max_decoding_buffer_size(MAX_DECODED_IMAGE_BYTES);
    decoder.read_info().ok()?;
    let info = decoder.info()?;
    decoded_rgba_len(u32::from(info.width), u32::from(info.height))?;
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
    decoder.set_memory_limit(MAX_DECODED_IMAGE_BYTES);
    let (width, height) = decoder.dimensions();
    decoded_rgba_len(width, height)?;
    let samples_per_pixel = if decoder.has_alpha() { 4 } else { 3 };
    let output_buffer_size = decoder.output_buffer_size()?;
    if output_buffer_size > MAX_DECODED_IMAGE_BYTES {
        return None;
    }
    let mut decoded = zeroed_buffer(output_buffer_size)?;
    decoder.read_image(&mut decoded).ok()?;
    Some((width, height, decoded.len(), samples_per_pixel))
}

fn zeroed_buffer(len: usize) -> Option<Vec<u8>> {
    let mut buffer = Vec::new();
    buffer.try_reserve_exact(len).ok()?;
    buffer.resize(len, 0);
    Some(buffer)
}

fn decoded_sample_len(width: u32, height: u32, samples_per_pixel: usize) -> Option<usize> {
    usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?
        .checked_mul(samples_per_pixel)
}

#[cfg(test)]
mod tests {
    use super::{
        DecodedImageDimensions, MAX_IMAGE_DIMENSION, decode_image_rgba, decoded_rgba_len,
        preflight_encoded_image, validate_encoded_image,
    };

    const PIXEL_BOMB_DIMENSION: u32 = 4_097;

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
        let decoded = decode_image_rgba(&encoded).expect("RGBA decode");
        assert_eq!((decoded.width, decoded.height), (2, 1));
        assert_eq!(decoded.pixels, [120, 60, 30, 128, 10, 20, 30, 255]);
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

    #[test]
    fn rejects_oversized_png_during_header_preflight() {
        assert!(PIXEL_BOMB_DIMENSION <= MAX_IMAGE_DIMENSION);
        assert_eq!(
            decoded_rgba_len(PIXEL_BOMB_DIMENSION, PIXEL_BOMB_DIMENSION),
            None
        );
        let mut encoded = Vec::new();
        let writer = png::Encoder::new(&mut encoded, PIXEL_BOMB_DIMENSION, PIXEL_BOMB_DIMENSION)
            .write_header()
            .expect("PNG header encodes");
        drop(writer);
        let mut decoder = png::Decoder::new(std::io::Cursor::new(&encoded));
        let info = decoder.read_header_info().expect("PNG header parses");
        assert_eq!(
            (info.width, info.height),
            (PIXEL_BOMB_DIMENSION, PIXEL_BOMB_DIMENSION)
        );

        assert_eq!(preflight_encoded_image(&encoded), None);
        assert_eq!(validate_encoded_image(&encoded), None);
    }

    #[test]
    fn rejects_oversized_jpeg_during_header_preflight() {
        let dimension = u16::try_from(PIXEL_BOMB_DIMENSION).unwrap();
        let [height_hi, height_lo] = dimension.to_be_bytes();
        let [width_hi, width_lo] = dimension.to_be_bytes();
        let encoded = [
            0xff, 0xd8, // SOI
            0xff, 0xc0, // baseline SOF
            0x00, 0x11, // segment length
            0x08, // precision
            height_hi, height_lo, // height
            width_hi, width_lo, // width
            0x03,     // components
            0x01, 0x11, 0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00,
        ];
        let mut decoder = jpeg_decoder::Decoder::new(std::io::Cursor::new(encoded));
        decoder.read_info().expect("JPEG header parses");
        let info = decoder.info().expect("JPEG dimensions");
        assert_eq!(
            (u32::from(info.width), u32::from(info.height)),
            (PIXEL_BOMB_DIMENSION, PIXEL_BOMB_DIMENSION)
        );

        assert_eq!(preflight_encoded_image(&encoded), None);
        assert_eq!(validate_encoded_image(&encoded), None);
    }

    #[test]
    fn rejects_oversized_webp_during_header_preflight() {
        let mut encoded = Vec::new();
        image_webp::WebPEncoder::new(&mut encoded)
            .encode(&[1, 2, 3, 255], 1, 1, image_webp::ColorType::Rgba8)
            .expect("WebP fixture encodes");
        let header_offset = encoded
            .windows(4)
            .position(|window| window == b"VP8L")
            .expect("lossless WebP chunk")
            + 9;
        let encoded_dimension = PIXEL_BOMB_DIMENSION - 1;
        let dimension_bits = (encoded_dimension << 14) | encoded_dimension;
        encoded[header_offset..header_offset + 4].copy_from_slice(&dimension_bits.to_le_bytes());
        let decoder = image_webp::WebPDecoder::new(std::io::Cursor::new(&encoded))
            .expect("mutated WebP header parses");
        assert_eq!(
            decoder.dimensions(),
            (PIXEL_BOMB_DIMENSION, PIXEL_BOMB_DIMENSION)
        );

        assert_eq!(preflight_encoded_image(&encoded), None);
        assert_eq!(validate_encoded_image(&encoded), None);
    }
}
