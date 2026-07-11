//! Canonical interchange format for the C++/Rust WebGPU atlas-mask oracle.
//!
//! The file layout is a 20-byte little-endian header followed by rows of f16
//! coverage samples with no padding:
//!
//! ```text
//! bytes  0.. 8  magic: "RIVEMSK\\0"
//! bytes  8..12  format version: u32 little-endian (1)
//! bytes 12..16  width: u32 little-endian
//! bytes 16..20  height: u32 little-endian
//! bytes 20..    width * height f16 bit patterns, row-major and little-endian
//! ```

use std::error::Error;
use std::fmt;

pub(crate) const MAGIC: [u8; 8] = *b"RIVEMSK\0";
pub(crate) const VERSION: u32 = 1;
const HEADER_SIZE: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AtlasMask {
    width: u32,
    height: u32,
    samples: Vec<u16>,
}

impl AtlasMask {
    pub(crate) fn new(width: u32, height: u32, samples: Vec<u16>) -> Result<Self, AtlasMaskError> {
        let expected_samples = expected_sample_count(width, height)?;
        if samples.len() != expected_samples {
            return Err(AtlasMaskError::SampleCount {
                width,
                height,
                expected: expected_samples,
                actual: samples.len(),
            });
        }
        Ok(Self {
            width,
            height,
            samples,
        })
    }

    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, AtlasMaskError> {
        if bytes.len() < HEADER_SIZE {
            return Err(AtlasMaskError::TruncatedHeader {
                actual: bytes.len(),
            });
        }
        if bytes[..MAGIC.len()] != MAGIC {
            return Err(AtlasMaskError::InvalidMagic);
        }
        let version = read_u32(bytes, 8);
        if version != VERSION {
            return Err(AtlasMaskError::UnsupportedVersion(version));
        }
        let width = read_u32(bytes, 12);
        let height = read_u32(bytes, 16);
        let expected_samples = expected_sample_count(width, height)?;
        let expected_bytes = expected_samples
            .checked_mul(2)
            .and_then(|sample_bytes| HEADER_SIZE.checked_add(sample_bytes))
            .ok_or(AtlasMaskError::DimensionsOverflow { width, height })?;
        if bytes.len() != expected_bytes {
            return Err(AtlasMaskError::DataLength {
                width,
                height,
                expected: expected_bytes,
                actual: bytes.len(),
            });
        }
        let samples = bytes[HEADER_SIZE..]
            .chunks_exact(2)
            .map(|sample| u16::from_le_bytes(sample.try_into().unwrap()))
            .collect();
        Self::new(width, height, samples)
    }

    pub(crate) fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_SIZE + self.samples.len() * 2);
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        bytes.extend_from_slice(&self.width.to_le_bytes());
        bytes.extend_from_slice(&self.height.to_le_bytes());
        for sample in &self.samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        bytes
    }

    pub(crate) fn sample_bits(&self, x: usize, y: usize) -> u16 {
        self.samples[y * self.width as usize + x]
    }

    pub(crate) fn sample_value(&self, x: usize, y: usize) -> f32 {
        f16_bits_to_f32(self.sample_bits(x, y))
    }

    pub(crate) fn set_sample_bits(&mut self, x: usize, y: usize, bits: u16) {
        self.samples[y * self.width as usize + x] = bits;
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MaskComparisonTolerances {
    /// Samples at or below this coverage are treated as outside the mask.
    pub(crate) support: f32,
    /// Absolute coverage difference allowed once both samples have support.
    pub(crate) value: f32,
}

pub(crate) fn compare_cpp_to_rust(
    cpp: &AtlasMask,
    rust: &AtlasMask,
    tolerances: MaskComparisonTolerances,
) -> Result<(), AtlasMaskComparisonError> {
    if !tolerances.support.is_finite() || tolerances.support < 0.0 {
        return Err(AtlasMaskComparisonError::InvalidTolerance {
            name: "support",
            value: tolerances.support,
        });
    }
    if !tolerances.value.is_finite() || tolerances.value < 0.0 {
        return Err(AtlasMaskComparisonError::InvalidTolerance {
            name: "value",
            value: tolerances.value,
        });
    }
    if (cpp.width, cpp.height) != (rust.width, rust.height) {
        return Err(AtlasMaskComparisonError::Dimensions {
            cpp: (cpp.width, cpp.height),
            rust: (rust.width, rust.height),
        });
    }
    for (index, (&cpp_bits, &rust_bits)) in cpp.samples.iter().zip(&rust.samples).enumerate() {
        let cpp_value = f16_bits_to_f32(cpp_bits);
        let rust_value = f16_bits_to_f32(rust_bits);
        let x = index % cpp.width as usize;
        let y = index / cpp.width as usize;
        if !cpp_value.is_finite() || !rust_value.is_finite() {
            return Err(AtlasMaskComparisonError::NonFiniteValue {
                x,
                y,
                cpp: cpp_value,
                rust: rust_value,
            });
        }
        let cpp_supported = cpp_value > tolerances.support;
        let rust_supported = rust_value > tolerances.support;
        if cpp_supported != rust_supported {
            return Err(AtlasMaskComparisonError::Support {
                x,
                y,
                cpp: cpp_value,
                rust: rust_value,
                tolerance: tolerances.support,
            });
        }
        if cpp_supported && (cpp_value - rust_value).abs() > tolerances.value {
            return Err(AtlasMaskComparisonError::Value {
                x,
                y,
                cpp: cpp_value,
                rust: rust_value,
                tolerance: tolerances.value,
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AtlasMaskError {
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
    DataLength {
        width: u32,
        height: u32,
        expected: usize,
        actual: usize,
    },
    SampleCount {
        width: u32,
        height: u32,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for AtlasMaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader { actual } => write!(
                f,
                "truncated atlas-mask header: expected at least {HEADER_SIZE} bytes, got {actual}"
            ),
            Self::InvalidMagic => write!(f, "invalid atlas-mask magic; expected RIVEMSK\\0"),
            Self::UnsupportedVersion(version) => {
                write!(
                    f,
                    "unsupported atlas-mask version {version}; expected {VERSION}"
                )
            }
            Self::ZeroDimension { width, height } => {
                write!(
                    f,
                    "atlas-mask dimensions must be nonzero, got {width}x{height}"
                )
            }
            Self::DimensionsOverflow { width, height } => {
                write!(
                    f,
                    "atlas-mask dimensions overflow host size: {width}x{height}"
                )
            }
            Self::DataLength {
                width,
                height,
                expected,
                actual,
            } => write!(
                f,
                "atlas-mask data length for {width}x{height} must be {expected} bytes, got {actual}"
            ),
            Self::SampleCount {
                width,
                height,
                expected,
                actual,
            } => write!(
                f,
                "atlas-mask {width}x{height} needs {expected} samples, got {actual}"
            ),
        }
    }
}

impl Error for AtlasMaskError {}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AtlasMaskComparisonError {
    InvalidTolerance {
        name: &'static str,
        value: f32,
    },
    Dimensions {
        cpp: (u32, u32),
        rust: (u32, u32),
    },
    NonFiniteValue {
        x: usize,
        y: usize,
        cpp: f32,
        rust: f32,
    },
    Support {
        x: usize,
        y: usize,
        cpp: f32,
        rust: f32,
        tolerance: f32,
    },
    Value {
        x: usize,
        y: usize,
        cpp: f32,
        rust: f32,
        tolerance: f32,
    },
}

impl fmt::Display for AtlasMaskComparisonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTolerance { name, value } => {
                write!(f, "{name} tolerance must be finite and nonnegative, got {value}")
            }
            Self::Dimensions { cpp, rust } => write!(
                f,
                "dimensions differ: C++ is {}x{}, Rust is {}x{}",
                cpp.0, cpp.1, rust.0, rust.1
            ),
            Self::NonFiniteValue { x, y, cpp, rust } => write!(
                f,
                "non-finite coverage at ({x}, {y}): C++={cpp}, Rust={rust}"
            ),
            Self::Support {
                x,
                y,
                cpp,
                rust,
                tolerance,
            } => write!(
                f,
                "support differs at ({x}, {y}) with support tolerance {tolerance}: C++={cpp}, Rust={rust}"
            ),
            Self::Value {
                x,
                y,
                cpp,
                rust,
                tolerance,
            } => write!(
                f,
                "coverage differs at ({x}, {y}) by more than value tolerance {tolerance}: C++={cpp}, Rust={rust}"
            ),
        }
    }
}

impl Error for AtlasMaskComparisonError {}

fn expected_sample_count(width: u32, height: u32) -> Result<usize, AtlasMaskError> {
    if width == 0 || height == 0 {
        return Err(AtlasMaskError::ZeroDimension { width, height });
    }
    let count = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or(AtlasMaskError::DimensionsOverflow { width, height })?;
    usize::try_from(count).map_err(|_| AtlasMaskError::DimensionsOverflow { width, height })
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn f16_bits_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits & 0x8000) << 16;
    let exponent = (bits >> 10) & 0x1f;
    let fraction = u32::from(bits & 0x03ff);
    match exponent {
        0 if fraction == 0 => f32::from_bits(sign),
        0 => {
            let value = (fraction as f32) * 2.0f32.powi(-24);
            if sign == 0 {
                value
            } else {
                -value
            }
        }
        0x1f => f32::from_bits(sign | 0x7f80_0000 | (fraction << 13)),
        _ => f32::from_bits(sign | (u32::from(exponent) + 112) << 23 | (fraction << 13)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCES: MaskComparisonTolerances = MaskComparisonTolerances {
        support: 1.0 / 1024.0,
        value: 1.0 / 512.0,
    };

    #[test]
    fn serializes_and_parses_row_packed_little_endian_f16() {
        let mask = AtlasMask::new(2, 2, vec![0x0000, 0x3c00, 0x3800, 0xbc00]).unwrap();

        let bytes = mask.serialize();

        assert_eq!(&bytes[..8], b"RIVEMSK\0");
        assert_eq!(&bytes[8..12], &1u32.to_le_bytes());
        assert_eq!(&bytes[12..16], &2u32.to_le_bytes());
        assert_eq!(&bytes[16..20], &2u32.to_le_bytes());
        assert_eq!(&bytes[20..], &[0, 0, 0, 60, 0, 56, 0, 188]);
        assert_eq!(AtlasMask::parse(&bytes).unwrap(), mask);
    }

    #[test]
    fn parser_rejects_malformed_headers_and_data() {
        assert!(matches!(
            AtlasMask::parse(&[0; HEADER_SIZE - 1]),
            Err(AtlasMaskError::TruncatedHeader { .. })
        ));
        let mut bytes = AtlasMask::new(1, 1, vec![0]).unwrap().serialize();
        bytes[0] = b'X';
        assert_eq!(AtlasMask::parse(&bytes), Err(AtlasMaskError::InvalidMagic));

        let mut bytes = AtlasMask::new(1, 1, vec![0]).unwrap().serialize();
        bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
        assert_eq!(
            AtlasMask::parse(&bytes),
            Err(AtlasMaskError::UnsupportedVersion(2))
        );

        let bytes = AtlasMask::new(1, 1, vec![0]).unwrap().serialize();
        assert!(matches!(
            AtlasMask::parse(&bytes[..bytes.len() - 1]),
            Err(AtlasMaskError::DataLength { .. })
        ));
    }

    #[test]
    fn serializer_rejects_invalid_dimensions_and_sample_counts() {
        assert!(matches!(
            AtlasMask::new(0, 1, vec![]),
            Err(AtlasMaskError::ZeroDimension { .. })
        ));
        assert!(matches!(
            AtlasMask::new(2, 2, vec![0; 3]),
            Err(AtlasMaskError::SampleCount {
                expected: 4,
                actual: 3,
                ..
            })
        ));
    }

    #[test]
    fn comparison_reports_dimension_support_and_value_mismatches() {
        let cpp = AtlasMask::new(2, 1, vec![0x0000, 0x3c00]).unwrap();
        let rust = AtlasMask::new(1, 2, vec![0x0000, 0x3c00]).unwrap();
        assert!(matches!(
            compare_cpp_to_rust(&cpp, &rust, TOLERANCES),
            Err(AtlasMaskComparisonError::Dimensions { .. })
        ));

        let rust = AtlasMask::new(2, 1, vec![0x0000, 0x0000]).unwrap();
        assert!(matches!(
            compare_cpp_to_rust(&cpp, &rust, TOLERANCES),
            Err(AtlasMaskComparisonError::Support { x: 1, y: 0, .. })
        ));

        let cpp = AtlasMask::new(1, 1, vec![0x3800]).unwrap();
        let rust = AtlasMask::new(1, 1, vec![0x3a00]).unwrap();
        assert!(matches!(
            compare_cpp_to_rust(&cpp, &rust, TOLERANCES),
            Err(AtlasMaskComparisonError::Value { x: 0, y: 0, .. })
        ));
    }

    #[test]
    fn comparison_accepts_values_within_explicit_tolerances() {
        let cpp = AtlasMask::new(1, 1, vec![0x3800]).unwrap();
        let rust = AtlasMask::new(1, 1, vec![0x3801]).unwrap();

        compare_cpp_to_rust(&cpp, &rust, TOLERANCES).unwrap();
    }
}
