//! Canonical interchange format for the C++/Rust WebGPU atlas-blit oracle.
//!
//! The file layout is a 20-byte little-endian header followed by tightly
//! packed RGBA8 pixels in row-major order.

use std::error::Error;
use std::fmt;

pub(crate) const MAGIC: [u8; 8] = *b"RIVEABL\0";
pub(crate) const VERSION: u32 = 1;
const HEADER_SIZE: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AtlasBlit {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl AtlasBlit {
    pub(crate) fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, AtlasBlitError> {
        let expected = expected_pixel_bytes(width, height)?;
        if pixels.len() != expected {
            return Err(AtlasBlitError::PixelDataLength {
                width,
                height,
                expected,
                actual: pixels.len(),
            });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, AtlasBlitError> {
        if bytes.len() < HEADER_SIZE {
            return Err(AtlasBlitError::TruncatedHeader {
                actual: bytes.len(),
            });
        }
        if bytes[..MAGIC.len()] != MAGIC {
            return Err(AtlasBlitError::InvalidMagic);
        }
        let version = read_u32(bytes, 8);
        if version != VERSION {
            return Err(AtlasBlitError::UnsupportedVersion(version));
        }
        let width = read_u32(bytes, 12);
        let height = read_u32(bytes, 16);
        Self::new(width, height, bytes[HEADER_SIZE..].to_vec())
    }

    #[cfg(test)]
    pub(crate) fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    #[cfg(test)]
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_SIZE + self.pixels.len());
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        bytes.extend_from_slice(&self.width.to_le_bytes());
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.pixels);
        bytes
    }
}

pub(crate) fn compare_cpp_to_rust(
    cpp: &AtlasBlit,
    rust: &AtlasBlit,
) -> Result<(), AtlasBlitComparisonError> {
    compare_cpp_to_rust_with_tolerance(cpp, rust, 0, 0)
}

pub(crate) fn compare_cpp_to_rust_with_tolerance(
    cpp: &AtlasBlit,
    rust: &AtlasBlit,
    max_channel_delta_allowed: u8,
    max_different_pixels_allowed: usize,
) -> Result<(), AtlasBlitComparisonError> {
    if (cpp.width, cpp.height) != (rust.width, rust.height) {
        return Err(AtlasBlitComparisonError::Dimensions {
            cpp: (cpp.width, cpp.height),
            rust: (rust.width, rust.height),
        });
    }
    let mut first = None;
    let mut different_pixels = 0usize;
    let mut max_channel_delta = 0u8;
    for (pixel_index, (cpp_pixel, rust_pixel)) in cpp
        .pixels
        .chunks_exact(4)
        .zip(rust.pixels.chunks_exact(4))
        .enumerate()
    {
        let mut pixel_differs = false;
        for channel in 0..4 {
            let cpp_value = cpp_pixel[channel];
            let rust_value = rust_pixel[channel];
            if cpp_value != rust_value {
                pixel_differs = true;
                max_channel_delta = max_channel_delta.max(cpp_value.abs_diff(rust_value));
                first.get_or_insert((pixel_index, channel, cpp_value, rust_value));
            }
        }
        if pixel_differs {
            different_pixels += 1;
        }
    }
    match first {
        None => Ok(()),
        Some(_)
            if max_channel_delta <= max_channel_delta_allowed
                && different_pixels <= max_different_pixels_allowed =>
        {
            Ok(())
        }
        Some((pixel, channel, first_cpp, first_rust)) => Err(AtlasBlitComparisonError::Pixels {
            first_x: pixel % cpp.width as usize,
            first_y: pixel / cpp.width as usize,
            first_channel: channel,
            first_cpp,
            first_rust,
            different_pixels,
            max_channel_delta,
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AtlasBlitError {
    TruncatedHeader {
        actual: usize,
    },
    InvalidMagic,
    UnsupportedVersion(u32),
    ZeroDimension {
        width: u32,
        height: u32,
    },
    DimensionsOverflow {
        width: u32,
        height: u32,
    },
    PixelDataLength {
        width: u32,
        height: u32,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for AtlasBlitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader { actual } => write!(
                f,
                "truncated atlas-blit header: expected at least {HEADER_SIZE} bytes, got {actual}"
            ),
            Self::InvalidMagic => write!(f, "invalid atlas-blit magic; expected RIVEABL\\0"),
            Self::UnsupportedVersion(version) => write!(
                f,
                "unsupported atlas-blit version {version}; expected {VERSION}"
            ),
            Self::ZeroDimension { width, height } => {
                write!(
                    f,
                    "atlas-blit dimensions must be nonzero, got {width}x{height}"
                )
            }
            Self::DimensionsOverflow { width, height } => {
                write!(
                    f,
                    "atlas-blit dimensions overflow host size: {width}x{height}"
                )
            }
            Self::PixelDataLength {
                width,
                height,
                expected,
                actual,
            } => write!(
                f,
                "atlas-blit data length for {width}x{height} must be {expected} bytes, got {actual}"
            ),
        }
    }
}

impl Error for AtlasBlitError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AtlasBlitComparisonError {
    Dimensions {
        cpp: (u32, u32),
        rust: (u32, u32),
    },
    Pixels {
        first_x: usize,
        first_y: usize,
        first_channel: usize,
        first_cpp: u8,
        first_rust: u8,
        different_pixels: usize,
        max_channel_delta: u8,
    },
}

impl fmt::Display for AtlasBlitComparisonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dimensions { cpp, rust } => write!(
                f,
                "dimensions differ: C++ is {}x{}, Rust is {}x{}",
                cpp.0, cpp.1, rust.0, rust.1
            ),
            Self::Pixels {
                first_x,
                first_y,
                first_channel,
                first_cpp,
                first_rust,
                different_pixels,
                max_channel_delta,
            } => write!(
                f,
                "{different_pixels} pixels differ (max channel delta {max_channel_delta}); first at ({first_x}, {first_y}) channel {first_channel}: C++={first_cpp}, Rust={first_rust}"
            ),
        }
    }
}

impl Error for AtlasBlitComparisonError {}

fn expected_pixel_bytes(width: u32, height: u32) -> Result<usize, AtlasBlitError> {
    if width == 0 || height == 0 {
        return Err(AtlasBlitError::ZeroDimension { width, height });
    }
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(AtlasBlitError::DimensionsOverflow { width, height })
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_canonical_payload() {
        let blit = AtlasBlit::new(2, 1, vec![1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        assert_eq!(AtlasBlit::parse(&blit.serialize()).unwrap(), blit);
    }

    #[test]
    fn rejects_malformed_payloads() {
        assert!(matches!(
            AtlasBlit::parse(b"short"),
            Err(AtlasBlitError::TruncatedHeader { .. })
        ));
        let mut bytes = AtlasBlit::new(1, 1, vec![0; 4]).unwrap().serialize();
        bytes.pop();
        assert!(matches!(
            AtlasBlit::parse(&bytes),
            Err(AtlasBlitError::PixelDataLength { .. })
        ));
    }

    #[test]
    fn comparator_reports_first_channel_difference() {
        let cpp = AtlasBlit::new(1, 1, vec![1, 2, 3, 4]).unwrap();
        let rust = AtlasBlit::new(1, 1, vec![1, 9, 3, 4]).unwrap();
        assert_eq!(
            compare_cpp_to_rust(&cpp, &rust),
            Err(AtlasBlitComparisonError::Pixels {
                first_x: 0,
                first_y: 0,
                first_channel: 1,
                first_cpp: 2,
                first_rust: 9,
                different_pixels: 1,
                max_channel_delta: 7,
            })
        );
    }

    #[test]
    fn bounded_comparator_enforces_pixel_and_channel_caps() {
        let cpp = AtlasBlit::new(2, 1, vec![10, 20, 30, 40, 50, 60, 70, 80]).unwrap();
        let within = AtlasBlit::new(2, 1, vec![11, 20, 30, 40, 50, 59, 70, 80]).unwrap();
        assert!(compare_cpp_to_rust_with_tolerance(&cpp, &within, 1, 2).is_ok());
        assert!(compare_cpp_to_rust_with_tolerance(&cpp, &within, 0, 2).is_err());
        assert!(compare_cpp_to_rust_with_tolerance(&cpp, &within, 1, 1).is_err());
    }
}
