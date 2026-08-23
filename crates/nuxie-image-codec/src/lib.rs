//! Canonical encoded-image decoding shared by every runtime consumer.
//!
//! This crate deliberately excludes renderer backends. A consumer that only
//! needs to prove PNG, JPEG, or WebP bytes can be decoded must not pull WebGPU,
//! WebGL, or their JavaScript host ABI into a raw `wasm32-unknown-unknown`
//! module. Platform decoder mechanics that are part of Rive parity still live
//! here: on macOS, PNG, JPEG, and WebP use ImageIO/CoreGraphics just like the
//! C++ runtime.
//!
//! The baseline entry points enforce the resource ceilings below. Product and
//! platform admission may add stricter limits with [`ImageAdmissionPolicy`],
//! but cannot relax those baseline ceilings. The explicitly named
//! [`decode_image_rgba_unbounded`] entry point exists only for pinned low-level
//! runtime compatibility and must not be used for untrusted product admission.

#[cfg(target_os = "macos")]
use std::ffi::c_void;
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

/// Additional caller-owned limits applied after the baseline decoder accepts
/// an image.
///
/// These limits are admission policy, not decoder configuration: they can
/// reject an otherwise baseline-safe image, but cannot make the canonical
/// decoder accept an image above its built-in safety ceilings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageAdmissionPolicy {
    pub max_encoded_bytes: usize,
    pub max_dimension: u32,
    pub max_decoded_bytes: usize,
}

impl ImageAdmissionPolicy {
    /// Admission policy that adds no restriction beyond baseline safety.
    pub const BASELINE: Self = Self {
        max_encoded_bytes: MAX_ENCODED_IMAGE_BYTES,
        max_dimension: MAX_IMAGE_DIMENSION,
        max_decoded_bytes: MAX_DECODED_IMAGE_BYTES,
    };

    fn admits_dimensions(self, dimensions: DecodedImageDimensions) -> bool {
        dimensions.width <= self.max_dimension
            && dimensions.height <= self.max_dimension
            && decoded_sample_len(dimensions.width, dimensions.height, 4)
                .is_some_and(|len| len <= self.max_decoded_bytes)
    }
}

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

/// Fully decode using baseline safety ceilings, then apply stricter
/// caller-owned admission policy.
///
/// The policy is checked against baseline-safe header dimensions before the
/// pixel allocation, then acceptance is proven by the same full canonical
/// decode used by scripting and rendering.
#[must_use]
pub fn validate_encoded_image_with_policy(
    data: &[u8],
    policy: ImageAdmissionPolicy,
) -> Option<DecodedImageDimensions> {
    if data.len() > policy.max_encoded_bytes {
        return None;
    }
    let expected = preflight_encoded_image(data)?;
    if !policy.admits_dimensions(expected) {
        return None;
    }
    let decoded = decode_image_rgba(data)?;
    ((decoded.width, decoded.height) == (expected.width, expected.height)
        && decoded.pixels.len() == decoded_sample_len(decoded.width, decoded.height, 4)?)
    .then_some(expected)
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
    if width == 0
        || height == 0
        || max_decoded_bytes
            .is_some_and(|_| width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION)
    {
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
    validate_encoded_image_with_policy(data, ImageAdmissionPolicy::BASELINE)
}

#[cfg(target_os = "macos")]
fn decode_png_rgba(data: &[u8], max_decoded_bytes: Option<usize>) -> Option<(u32, u32, Vec<u8>)> {
    decode_macos_image_rgba(data, max_decoded_bytes)
}

#[cfg(not(target_os = "macos"))]
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

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CoreGraphicsPoint {
    x: f64,
    y: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CoreGraphicsSize {
    width: f64,
    height: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CoreGraphicsRect {
    origin: CoreGraphicsPoint,
    size: CoreGraphicsSize,
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "CoreGraphics", kind = "framework")]
#[link(name = "ImageIO", kind = "framework")]
unsafe extern "C" {
    fn CFDataCreate(allocator: *const c_void, bytes: *const u8, length: isize) -> *const c_void;
    fn CFRelease(cf: *const c_void);
    fn CGImageSourceCreateWithData(data: *const c_void, options: *const c_void) -> *const c_void;
    fn CGImageSourceCreateImageAtIndex(
        source: *const c_void,
        index: usize,
        options: *const c_void,
    ) -> *const c_void;
    fn CGImageGetAlphaInfo(image: *const c_void) -> u32;
    fn CGImageGetWidth(image: *const c_void) -> usize;
    fn CGImageGetHeight(image: *const c_void) -> usize;
    fn CGColorSpaceCreateDeviceRGB() -> *const c_void;
    fn CGColorSpaceRelease(space: *const c_void);
    fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bits_per_component: usize,
        bytes_per_row: usize,
        space: *const c_void,
        bitmap_info: u32,
    ) -> *mut c_void;
    fn CGContextSetBlendMode(context: *mut c_void, mode: i32);
    fn CGContextDrawImage(context: *mut c_void, rect: CoreGraphicsRect, image: *const c_void);
    fn CGContextRelease(context: *mut c_void);
}

#[cfg(target_os = "macos")]
fn decode_macos_image_rgba(
    data: &[u8],
    max_decoded_bytes: Option<usize>,
) -> Option<(u32, u32, Vec<u8>)> {
    const ALPHA_PREMULTIPLIED_LAST: u32 = 1;
    const ALPHA_NONE: u32 = 0;
    const ALPHA_NONE_SKIP_LAST: u32 = 5;
    const ALPHA_NONE_SKIP_FIRST: u32 = 6;
    const BYTE_ORDER_32_BIG: u32 = 4 << 12;
    const BLEND_MODE_COPY: i32 = 17;

    let data_length = isize::try_from(data.len()).ok()?;
    let encoded = unsafe { CFDataCreate(std::ptr::null(), data.as_ptr(), data_length) };
    if encoded.is_null() {
        return None;
    }
    let source = unsafe { CGImageSourceCreateWithData(encoded, std::ptr::null()) };
    unsafe { CFRelease(encoded) };
    if source.is_null() {
        return None;
    }
    let image = unsafe { CGImageSourceCreateImageAtIndex(source, 0, std::ptr::null()) };
    unsafe { CFRelease(source) };
    if image.is_null() {
        return None;
    }

    let image_width = unsafe { CGImageGetWidth(image) };
    let image_height = unsafe { CGImageGetHeight(image) };
    let Some(row_bytes) = image_width.checked_mul(4) else {
        unsafe { CFRelease(image) };
        return None;
    };
    let Some(byte_count) = row_bytes.checked_mul(image_height) else {
        unsafe { CFRelease(image) };
        return None;
    };
    let (Ok(width), Ok(height)) = (u32::try_from(image_width), u32::try_from(image_height)) else {
        unsafe { CFRelease(image) };
        return None;
    };
    let Some(expected_byte_count) = decoded_rgba_len_with_limit(width, height, max_decoded_bytes)
    else {
        unsafe { CFRelease(image) };
        return None;
    };
    if byte_count != expected_byte_count {
        unsafe { CFRelease(image) };
        return None;
    }

    let alpha_info = unsafe { CGImageGetAlphaInfo(image) };
    let opaque = matches!(
        alpha_info,
        ALPHA_NONE | ALPHA_NONE_SKIP_LAST | ALPHA_NONE_SKIP_FIRST
    );
    let color_space = unsafe { CGColorSpaceCreateDeviceRGB() };
    if color_space.is_null() {
        unsafe { CFRelease(image) };
        return None;
    }
    let Some(mut pixels) = zeroed_buffer(byte_count) else {
        unsafe {
            CGColorSpaceRelease(color_space);
            CFRelease(image);
        }
        return None;
    };
    let bitmap_info = BYTE_ORDER_32_BIG
        | if opaque {
            ALPHA_NONE_SKIP_LAST
        } else {
            ALPHA_PREMULTIPLIED_LAST
        };
    let context = unsafe {
        CGBitmapContextCreate(
            pixels.as_mut_ptr().cast(),
            image_width,
            image_height,
            8,
            row_bytes,
            color_space,
            bitmap_info,
        )
    };
    unsafe { CGColorSpaceRelease(color_space) };
    if context.is_null() {
        unsafe { CFRelease(image) };
        return None;
    }
    unsafe {
        CGContextSetBlendMode(context, BLEND_MODE_COPY);
        CGContextDrawImage(
            context,
            CoreGraphicsRect {
                origin: CoreGraphicsPoint { x: 0.0, y: 0.0 },
                size: CoreGraphicsSize {
                    width: f64::from(width),
                    height: f64::from(height),
                },
            },
            image,
        );
        CGContextRelease(context);
        CFRelease(image);
    }
    Some((width, height, pixels))
}

#[cfg(target_os = "macos")]
fn decode_jpeg_rgba(data: &[u8], max_decoded_bytes: Option<usize>) -> Option<(u32, u32, Vec<u8>)> {
    decode_macos_image_rgba(data, max_decoded_bytes)
}

#[cfg(not(target_os = "macos"))]
fn decode_jpeg_rgba(data: &[u8], max_decoded_bytes: Option<usize>) -> Option<(u32, u32, Vec<u8>)> {
    decode_portable_jpeg_rgba(data, max_decoded_bytes)
}

#[cfg(any(not(target_os = "macos"), test))]
fn decode_portable_jpeg_rgba(
    data: &[u8],
    max_decoded_bytes: Option<usize>,
) -> Option<(u32, u32, Vec<u8>)> {
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

#[cfg(target_os = "macos")]
fn decode_webp_rgba(data: &[u8], max_decoded_bytes: Option<usize>) -> Option<(u32, u32, Vec<u8>)> {
    decode_macos_image_rgba(data, max_decoded_bytes)
}

#[cfg(not(target_os = "macos"))]
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

#[cfg(any(not(target_os = "macos"), test))]
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

#[cfg(any(not(target_os = "macos"), test))]
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
        DecodedImageDimensions, ImageAdmissionPolicy, MAX_IMAGE_DIMENSION, decode_image_rgba,
        decode_image_rgba_unbounded, decoded_rgba_len, preflight_encoded_image,
        validate_encoded_image, validate_encoded_image_with_policy,
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

    #[test]
    fn decodes_png_to_premultiplied_rgba() {
        let mut encoded = Vec::new();
        let mut encoder = png::Encoder::new(&mut encoded, 2, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .expect("PNG header")
            .write_image_data(&[240, 120, 60, 128, 10, 20, 30, 255])
            .expect("PNG data");

        let decoded = decode_image_rgba(&encoded).expect("PNG decode");
        assert_eq!((decoded.width, decoded.height), (2, 1));
        assert_eq!(decoded.pixels, [120, 60, 30, 128, 10, 20, 30, 255]);
    }

    #[test]
    fn decodes_profiled_corpus_jpeg_to_opaque_rgba() {
        let stream = include_str!(
            "../../../fixtures/renderer/streams/riv/clipping_and_draw_order.rive-stream"
        );
        let encoded = stream
            .lines()
            .find_map(|line| line.strip_prefix("decodeImage "))
            .and_then(|line| line.split_once("data="))
            .map(|(_, hex)| {
                hex.as_bytes()
                    .chunks_exact(2)
                    .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                    .collect::<Vec<_>>()
            })
            .expect("fixture JPEG");
        let mut decoder = jpeg_decoder::Decoder::new(std::io::Cursor::new(&encoded));
        decoder.read_info().expect("JPEG header");
        assert!(decoder.icc_profile().is_some());

        let decoded = decode_image_rgba(&encoded).expect("JPEG decode");
        assert_eq!((decoded.width, decoded.height), (278, 278));
        assert_eq!(decoded.pixels.len(), 278 * 278 * 4);
        assert!(decoded.pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));

        let (portable_width, portable_height, portable_rgba) =
            super::decode_portable_jpeg_rgba(&encoded, Some(super::MAX_DECODED_IMAGE_BYTES))
                .expect("portable JPEG decode");
        assert_eq!((portable_width, portable_height), (278, 278));
        assert_eq!(portable_rgba.len(), decoded.pixels.len());
    }

    #[test]
    fn decodes_lossy_and_lossless_webp() {
        let mut lossless = Vec::new();
        image_webp::WebPEncoder::new(&mut lossless)
            .encode(
                &[240, 120, 60, 128, 10, 20, 30, 255],
                2,
                1,
                image_webp::ColorType::Rgba8,
            )
            .expect("lossless WebP");
        let decoded = decode_image_rgba(&lossless).expect("lossless WebP decode");
        assert_eq!(decoded.pixels, [120, 60, 30, 128, 10, 20, 30, 255]);

        let lossy = [
            0x52, 0x49, 0x46, 0x46, 0x3c, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50,
            0x38, 0x20, 0x30, 0x00, 0x00, 0x00, 0xd0, 0x01, 0x00, 0x9d, 0x01, 0x2a, 0x02, 0x00,
            0x02, 0x00, 0x02, 0x00, 0x34, 0x25, 0xa0, 0x02, 0x74, 0xba, 0x01, 0xf8, 0x00, 0x03,
            0xb0, 0x00, 0xfe, 0xf0, 0xc4, 0x0b, 0xff, 0x20, 0xb9, 0x61, 0x75, 0xc8, 0xd7, 0xff,
            0x20, 0x3f, 0xe4, 0x07, 0xfc, 0x80, 0xff, 0xf8, 0xf2, 0x00, 0x00, 0x00,
        ];
        let decoded = decode_image_rgba(&lossy).expect("lossy WebP decode");
        assert_eq!((decoded.width, decoded.height), (2, 2));
        assert!(decoded.pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }

    #[test]
    fn corrupt_payloads_fail_every_public_bounded_mode() {
        let mut truncated_webp = Vec::new();
        image_webp::WebPEncoder::new(&mut truncated_webp)
            .encode(&[240, 120, 60, 128], 1, 1, image_webp::ColorType::Rgba8)
            .expect("WebP fixture");
        truncated_webp.truncate(truncated_webp.len() / 2);

        for encoded in [&truncated_webp[..], b"not an image"] {
            assert_eq!(preflight_encoded_image(encoded), None);
            assert_eq!(decode_image_rgba(encoded), None);
            assert_eq!(decode_image_rgba_unbounded(encoded), None);
            assert_eq!(validate_encoded_image(encoded), None);
            assert_eq!(
                validate_encoded_image_with_policy(encoded, ImageAdmissionPolicy::BASELINE),
                None
            );
        }
    }

    #[test]
    fn admission_policy_can_only_tighten_baseline_safety() {
        let mut encoded = Vec::new();
        let mut encoder = png::Encoder::new(&mut encoded, 2, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .expect("PNG header")
            .write_image_data(&[255; 8])
            .expect("PNG data");

        assert!(validate_encoded_image(&encoded).is_some());
        assert_eq!(
            validate_encoded_image_with_policy(
                &encoded,
                ImageAdmissionPolicy {
                    max_encoded_bytes: usize::MAX,
                    max_dimension: 1,
                    max_decoded_bytes: usize::MAX,
                },
            ),
            None
        );
        assert_eq!(
            validate_encoded_image_with_policy(
                &encoded,
                ImageAdmissionPolicy {
                    max_encoded_bytes: encoded.len() - 1,
                    max_dimension: u32::MAX,
                    max_decoded_bytes: usize::MAX,
                },
            ),
            None
        );
        assert_eq!(
            validate_encoded_image_with_policy(
                &encoded,
                ImageAdmissionPolicy {
                    max_encoded_bytes: usize::MAX,
                    max_dimension: u32::MAX,
                    max_decoded_bytes: 7,
                },
            ),
            None
        );
    }

    #[test]
    fn unbounded_compatibility_mode_is_distinct_from_baseline_admission() {
        let width = MAX_IMAGE_DIMENSION + 1;
        let mut encoded = Vec::new();
        let mut encoder = png::Encoder::new(&mut encoded, width, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .expect("PNG header")
            .write_image_data(&vec![255; width as usize * 4])
            .expect("PNG data");

        assert_eq!(decode_image_rgba(&encoded), None);
        assert_eq!(
            validate_encoded_image_with_policy(
                &encoded,
                ImageAdmissionPolicy {
                    max_encoded_bytes: usize::MAX,
                    max_dimension: u32::MAX,
                    max_decoded_bytes: usize::MAX,
                },
            ),
            None,
            "caller policy cannot relax baseline safety ceilings"
        );
        let decoded = decode_image_rgba_unbounded(&encoded).expect("compatibility decode");
        assert_eq!((decoded.width, decoded.height), (width, 1));
    }

    #[test]
    fn embedded_png_profile_changes_rgb_and_preserves_premultiplied_alpha() {
        let stream = nuxie_render_stream::RenderStream::parse(include_str!(
            "../../../fixtures/renderer/streams/gm/image_aa_border.rive-stream"
        ))
        .expect("ICC stream");
        let encoded = stream
            .resources
            .iter()
            .find_map(|resource| match resource {
                nuxie_render_stream::Resource::Image { data, .. } => Some(data.as_slice()),
                _ => None,
            })
            .expect("ICC PNG");
        let reader = png::Decoder::new(std::io::Cursor::new(encoded))
            .read_info()
            .expect("PNG header");
        let profile = reader.info().icc_profile.as_ref().expect("ICC profile");
        let mut pixel = [64, 128, 192, 77];
        let original = pixel;
        super::convert_icc_rgba_to_srgb(&mut pixel, 1, profile);
        assert_ne!(pixel[..3], original[..3]);
        assert_eq!(pixel[3], original[3]);

        let decoded = decode_image_rgba(encoded).expect("ICC PNG decode");
        assert_eq!((decoded.width, decoded.height), (319, 320));
        assert_eq!(decoded.pixels.len(), 319 * 320 * 4);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_jpeg_decode_fails_closed_when_imageio_rejects_input() {
        let encoded = [
            0xff, 0xd8, // SOI
            0xff, 0xc4, 0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // DHT
            0xff, 0xc3, 0x00, 0x0b, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x13,
            0x00, // SOF3, invalid 1x3 sampling for a single grayscale component
            0xff, 0xda, 0x00, 0x08, 0x01, 0x01, 0x00, 0x01, 0x00, 0x00, // SOS
            0x7f, 0xff, 0xd9, // zero difference and EOI
        ];
        let mut portable = jpeg_decoder::Decoder::new(std::io::Cursor::new(encoded));
        assert!(portable.decode().is_ok());
        assert!(super::decode_macos_image_rgba(&encoded, None).is_none());
        assert!(decode_image_rgba(&encoded).is_none());
    }
}
