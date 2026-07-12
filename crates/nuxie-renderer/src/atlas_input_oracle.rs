//! Canonical interchange format for C++/Rust atlas tessellation inputs.

use std::error::Error;
use std::fmt;

pub(crate) const MAGIC: [u8; 8] = *b"RIVEATI\0";
pub(crate) const VERSION: u32 = 1;
const HEADER_SIZE: usize = 40;
const CONTOUR_STRIDE: u32 = 16;
const TEXEL_STRIDE: u32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContourRecord {
    pub(crate) midpoint_x_bits: u32,
    pub(crate) midpoint_y_bits: u32,
    pub(crate) path_id: u32,
    pub(crate) vertex_index0: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AtlasInputs {
    pub(crate) base_patch: u32,
    pub(crate) patch_count: u32,
    pub(crate) contours: Vec<ContourRecord>,
    pub(crate) tess_width: u32,
    pub(crate) tess_height: u32,
    pub(crate) texels: Vec<[u32; 4]>,
}

impl AtlasInputs {
    pub(crate) fn new(
        base_patch: u32,
        patch_count: u32,
        contours: Vec<ContourRecord>,
        tess_width: u32,
        tess_height: u32,
        texels: Vec<[u32; 4]>,
    ) -> Result<Self, AtlasInputError> {
        validate_batch_range(base_patch, patch_count)?;
        let contour_count =
            u32::try_from(contours.len()).map_err(|_| AtlasInputError::ContourCountOverflow {
                actual: contours.len(),
            })?;
        expected_byte_count(contour_count, tess_width, tess_height)?;
        let expected_texels = expected_texel_count(tess_width, tess_height)?;
        if texels.len() != expected_texels {
            return Err(AtlasInputError::TexelCount {
                width: tess_width,
                height: tess_height,
                expected: expected_texels,
                actual: texels.len(),
            });
        }
        Ok(Self {
            base_patch,
            patch_count,
            contours,
            tess_width,
            tess_height,
            texels,
        })
    }

    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, AtlasInputError> {
        if bytes.len() < HEADER_SIZE {
            return Err(AtlasInputError::TruncatedHeader {
                actual: bytes.len(),
            });
        }
        if bytes[..MAGIC.len()] != MAGIC {
            return Err(AtlasInputError::InvalidMagic);
        }
        let version = read_u32(bytes, 8);
        if version != VERSION {
            return Err(AtlasInputError::UnsupportedVersion(version));
        }
        let base_patch = read_u32(bytes, 12);
        let patch_count = read_u32(bytes, 16);
        validate_batch_range(base_patch, patch_count)?;
        let contour_count = read_u32(bytes, 20);
        let tess_width = read_u32(bytes, 24);
        let tess_height = read_u32(bytes, 28);
        validate_stride("contour", read_u32(bytes, 32), CONTOUR_STRIDE)?;
        validate_stride("texel", read_u32(bytes, 36), TEXEL_STRIDE)?;
        let expected_bytes = expected_byte_count(contour_count, tess_width, tess_height)?;
        if bytes.len() < expected_bytes {
            return Err(AtlasInputError::TruncatedData {
                expected: expected_bytes,
                actual: bytes.len(),
            });
        }
        if bytes.len() > expected_bytes {
            return Err(AtlasInputError::TrailingData {
                expected: expected_bytes,
                actual: bytes.len(),
            });
        }

        let contour_count =
            usize::try_from(contour_count).map_err(|_| AtlasInputError::LayoutOverflow {
                contour_count,
                tess_width,
                tess_height,
            })?;
        let mut offset = HEADER_SIZE;
        let mut contours = Vec::with_capacity(contour_count);
        for _ in 0..contour_count {
            contours.push(ContourRecord {
                midpoint_x_bits: read_u32(bytes, offset),
                midpoint_y_bits: read_u32(bytes, offset + 4),
                path_id: read_u32(bytes, offset + 8),
                vertex_index0: read_u32(bytes, offset + 12),
            });
            offset += CONTOUR_STRIDE as usize;
        }
        let texel_count = expected_texel_count(tess_width, tess_height)?;
        let mut texels = Vec::with_capacity(texel_count);
        for _ in 0..texel_count {
            texels.push([
                read_u32(bytes, offset),
                read_u32(bytes, offset + 4),
                read_u32(bytes, offset + 8),
                read_u32(bytes, offset + 12),
            ]);
            offset += TEXEL_STRIDE as usize;
        }
        Self::new(
            base_patch,
            patch_count,
            contours,
            tess_width,
            tess_height,
            texels,
        )
    }

    pub(crate) fn serialize(&self) -> Vec<u8> {
        let contour_count = u32::try_from(self.contours.len())
            .expect("validated atlas inputs have a u32 contour count");
        let capacity = expected_byte_count(contour_count, self.tess_width, self.tess_height)
            .expect("validated atlas inputs have a serializable layout");
        let mut bytes = Vec::with_capacity(capacity);
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        bytes.extend_from_slice(&self.base_patch.to_le_bytes());
        bytes.extend_from_slice(&self.patch_count.to_le_bytes());
        bytes.extend_from_slice(&contour_count.to_le_bytes());
        bytes.extend_from_slice(&self.tess_width.to_le_bytes());
        bytes.extend_from_slice(&self.tess_height.to_le_bytes());
        bytes.extend_from_slice(&CONTOUR_STRIDE.to_le_bytes());
        bytes.extend_from_slice(&TEXEL_STRIDE.to_le_bytes());
        for contour in &self.contours {
            bytes.extend_from_slice(&contour.midpoint_x_bits.to_le_bytes());
            bytes.extend_from_slice(&contour.midpoint_y_bits.to_le_bytes());
            bytes.extend_from_slice(&contour.path_id.to_le_bytes());
            bytes.extend_from_slice(&contour.vertex_index0.to_le_bytes());
        }
        for texel in &self.texels {
            for channel in texel {
                bytes.extend_from_slice(&channel.to_le_bytes());
            }
        }
        bytes
    }
}

pub(crate) fn compare_cpp_to_rust(
    cpp: &AtlasInputs,
    rust: &AtlasInputs,
) -> Result<(), AtlasInputComparisonError> {
    compare_field("base_patch", cpp.base_patch, rust.base_patch)?;
    compare_field("patch_count", cpp.patch_count, rust.patch_count)?;
    if cpp.contours.len() != rust.contours.len() {
        return Err(AtlasInputComparisonError::ContourCount {
            cpp: cpp.contours.len(),
            rust: rust.contours.len(),
        });
    }
    compare_field("tess_width", cpp.tess_width, rust.tess_width)?;
    compare_field("tess_height", cpp.tess_height, rust.tess_height)?;
    for (index, (cpp_contour, rust_contour)) in cpp.contours.iter().zip(&rust.contours).enumerate()
    {
        compare_contour_field(
            index,
            "midpoint_x_bits",
            cpp_contour.midpoint_x_bits,
            rust_contour.midpoint_x_bits,
        )?;
        compare_contour_field(
            index,
            "midpoint_y_bits",
            cpp_contour.midpoint_y_bits,
            rust_contour.midpoint_y_bits,
        )?;
        compare_contour_field(index, "path_id", cpp_contour.path_id, rust_contour.path_id)?;
        compare_contour_field(
            index,
            "vertex_index0",
            cpp_contour.vertex_index0,
            rust_contour.vertex_index0,
        )?;
    }
    for (index, (cpp_texel, rust_texel)) in cpp.texels.iter().zip(&rust.texels).enumerate() {
        let x = (index % cpp.tess_width as usize) as u32;
        let y = (index / cpp.tess_width as usize) as u32;
        for channel in 0..4 {
            if cpp_texel[channel] != rust_texel[channel] {
                return Err(AtlasInputComparisonError::Texel {
                    x,
                    y,
                    channel: channel as u8,
                    cpp: cpp_texel[channel],
                    rust: rust_texel[channel],
                });
            }
        }
    }
    Ok(())
}

fn compare_field(
    field: &'static str,
    cpp: u32,
    rust: u32,
) -> Result<(), AtlasInputComparisonError> {
    if cpp != rust {
        return Err(AtlasInputComparisonError::BatchOrDimensionField { field, cpp, rust });
    }
    Ok(())
}

fn compare_contour_field(
    index: usize,
    field: &'static str,
    cpp: u32,
    rust: u32,
) -> Result<(), AtlasInputComparisonError> {
    if cpp != rust {
        return Err(AtlasInputComparisonError::ContourField {
            index,
            field,
            cpp,
            rust,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AtlasInputError {
    TruncatedHeader {
        actual: usize,
    },
    InvalidMagic,
    UnsupportedVersion(u32),
    BatchRangeOverflow {
        base_patch: u32,
        patch_count: u32,
    },
    InvalidStride {
        name: &'static str,
        expected: u32,
        actual: u32,
    },
    ZeroDimension {
        width: u32,
        height: u32,
    },
    ContourCountOverflow {
        actual: usize,
    },
    LayoutOverflow {
        contour_count: u32,
        tess_width: u32,
        tess_height: u32,
    },
    TruncatedData {
        expected: usize,
        actual: usize,
    },
    TrailingData {
        expected: usize,
        actual: usize,
    },
    TexelCount {
        width: u32,
        height: u32,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for AtlasInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader { actual } => write!(f, "truncated atlas-input header: expected at least {HEADER_SIZE} bytes, got {actual}"),
            Self::InvalidMagic => write!(f, "invalid atlas-input magic; expected RIVEATI\\0"),
            Self::UnsupportedVersion(version) => write!(f, "unsupported atlas-input version {version}; expected {VERSION}"),
            Self::BatchRangeOverflow { base_patch, patch_count } => write!(f, "atlas-input batch range overflows u32: base_patch={base_patch}, patch_count={patch_count}"),
            Self::InvalidStride { name, expected, actual } => write!(f, "atlas-input {name} stride must be {expected} bytes, got {actual}"),
            Self::ZeroDimension { width, height } => write!(f, "atlas-input tessellation dimensions must be nonzero, got {width}x{height}"),
            Self::ContourCountOverflow { actual } => write!(f, "atlas-input has {actual} contours, which does not fit in u32"),
            Self::LayoutOverflow { contour_count, tess_width, tess_height } => write!(f, "atlas-input layout overflows host size: {contour_count} contours and {tess_width}x{tess_height} texels"),
            Self::TruncatedData { expected, actual } => write!(f, "truncated atlas-input data: expected {expected} bytes, got {actual}"),
            Self::TrailingData { expected, actual } => write!(f, "atlas-input has trailing data: expected {expected} bytes, got {actual}"),
            Self::TexelCount { width, height, expected, actual } => write!(f, "atlas-input {width}x{height} needs {expected} texels, got {actual}"),
        }
    }
}

impl Error for AtlasInputError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AtlasInputComparisonError {
    BatchOrDimensionField {
        field: &'static str,
        cpp: u32,
        rust: u32,
    },
    ContourCount {
        cpp: usize,
        rust: usize,
    },
    ContourField {
        index: usize,
        field: &'static str,
        cpp: u32,
        rust: u32,
    },
    Texel {
        x: u32,
        y: u32,
        channel: u8,
        cpp: u32,
        rust: u32,
    },
}

impl fmt::Display for AtlasInputComparisonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BatchOrDimensionField { field, cpp, rust } => write!(f, "{field} differs: C++={cpp}, Rust={rust}"),
            Self::ContourCount { cpp, rust } => write!(f, "contour count differs: C++={cpp}, Rust={rust}"),
            Self::ContourField { index, field, cpp, rust } => write!(f, "contour {index} field {field} differs: C++={cpp:#010x}, Rust={rust:#010x}"),
            Self::Texel { x, y, channel, cpp, rust } => write!(f, "tessellation texel ({x}, {y}) channel {channel} differs: C++={cpp:#010x}, Rust={rust:#010x}"),
        }
    }
}

impl Error for AtlasInputComparisonError {}

fn validate_batch_range(base_patch: u32, patch_count: u32) -> Result<(), AtlasInputError> {
    base_patch
        .checked_add(patch_count)
        .ok_or(AtlasInputError::BatchRangeOverflow {
            base_patch,
            patch_count,
        })?;
    Ok(())
}

fn validate_stride(name: &'static str, actual: u32, expected: u32) -> Result<(), AtlasInputError> {
    if actual != expected {
        return Err(AtlasInputError::InvalidStride {
            name,
            expected,
            actual,
        });
    }
    Ok(())
}

fn expected_texel_count(width: u32, height: u32) -> Result<usize, AtlasInputError> {
    if width == 0 || height == 0 {
        return Err(AtlasInputError::ZeroDimension { width, height });
    }
    let count =
        u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or(AtlasInputError::LayoutOverflow {
                contour_count: 0,
                tess_width: width,
                tess_height: height,
            })?;
    usize::try_from(count).map_err(|_| AtlasInputError::LayoutOverflow {
        contour_count: 0,
        tess_width: width,
        tess_height: height,
    })
}

fn expected_byte_count(
    contour_count: u32,
    tess_width: u32,
    tess_height: u32,
) -> Result<usize, AtlasInputError> {
    let contours = usize::try_from(contour_count).map_err(|_| AtlasInputError::LayoutOverflow {
        contour_count,
        tess_width,
        tess_height,
    })?;
    let texels = match expected_texel_count(tess_width, tess_height) {
        Ok(texels) => texels,
        Err(AtlasInputError::ZeroDimension { width, height }) => {
            return Err(AtlasInputError::ZeroDimension { width, height });
        }
        Err(AtlasInputError::LayoutOverflow { .. }) => {
            return Err(AtlasInputError::LayoutOverflow {
                contour_count,
                tess_width,
                tess_height,
            });
        }
        Err(error) => return Err(error),
    };
    HEADER_SIZE
        .checked_add(contours.checked_mul(CONTOUR_STRIDE as usize).ok_or(
            AtlasInputError::LayoutOverflow {
                contour_count,
                tess_width,
                tess_height,
            },
        )?)
        .and_then(|bytes| bytes.checked_add(texels.checked_mul(TEXEL_STRIDE as usize)?))
        .ok_or(AtlasInputError::LayoutOverflow {
            contour_count,
            tess_width,
            tess_height,
        })
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> AtlasInputs {
        AtlasInputs::new(
            3,
            5,
            vec![ContourRecord {
                midpoint_x_bits: 0x3fa0_0000,
                midpoint_y_bits: 0xc020_0000,
                path_id: 7,
                vertex_index0: 11,
            }],
            2,
            1,
            vec![[1, 2, 3, 4], [5, 6, 7, 8]],
        )
        .unwrap()
    }

    #[test]
    fn serializes_and_parses_exact_little_endian_layout() {
        let inputs = inputs();
        let bytes = inputs.serialize();

        assert_eq!(&bytes[..8], b"RIVEATI\0");
        assert_eq!(&bytes[8..12], &VERSION.to_le_bytes());
        assert_eq!(&bytes[12..16], &3u32.to_le_bytes());
        assert_eq!(&bytes[16..20], &5u32.to_le_bytes());
        assert_eq!(&bytes[20..24], &1u32.to_le_bytes());
        assert_eq!(&bytes[24..28], &2u32.to_le_bytes());
        assert_eq!(&bytes[28..32], &1u32.to_le_bytes());
        assert_eq!(&bytes[32..36], &16u32.to_le_bytes());
        assert_eq!(&bytes[36..40], &16u32.to_le_bytes());
        assert_eq!(AtlasInputs::parse(&bytes).unwrap(), inputs);
    }

    #[test]
    fn parser_rejects_malformed_format() {
        assert!(matches!(
            AtlasInputs::parse(&[0; HEADER_SIZE - 1]),
            Err(AtlasInputError::TruncatedHeader { .. })
        ));
        let mut bytes = inputs().serialize();
        bytes[0] = b'X';
        assert_eq!(
            AtlasInputs::parse(&bytes),
            Err(AtlasInputError::InvalidMagic)
        );

        let mut bytes = inputs().serialize();
        bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
        assert_eq!(
            AtlasInputs::parse(&bytes),
            Err(AtlasInputError::UnsupportedVersion(2))
        );

        let mut bytes = inputs().serialize();
        bytes[32..36].copy_from_slice(&8u32.to_le_bytes());
        assert!(matches!(
            AtlasInputs::parse(&bytes),
            Err(AtlasInputError::InvalidStride {
                name: "contour",
                ..
            })
        ));

        let mut bytes = inputs().serialize();
        bytes[36..40].copy_from_slice(&8u32.to_le_bytes());
        assert!(matches!(
            AtlasInputs::parse(&bytes),
            Err(AtlasInputError::InvalidStride { name: "texel", .. })
        ));

        let mut bytes = inputs().serialize();
        bytes[24..28].copy_from_slice(&0u32.to_le_bytes());
        assert!(matches!(
            AtlasInputs::parse(&bytes),
            Err(AtlasInputError::ZeroDimension { .. })
        ));

        let mut bytes = inputs().serialize();
        bytes[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            AtlasInputs::parse(&bytes),
            Err(AtlasInputError::BatchRangeOverflow { .. })
        ));

        let bytes = inputs().serialize();
        assert!(matches!(
            AtlasInputs::parse(&bytes[..bytes.len() - 1]),
            Err(AtlasInputError::TruncatedData { .. })
        ));
        let mut bytes = inputs().serialize();
        bytes.push(0);
        assert!(matches!(
            AtlasInputs::parse(&bytes),
            Err(AtlasInputError::TrailingData { .. })
        ));

        let mut bytes = inputs().serialize();
        bytes[24..28].copy_from_slice(&u32::MAX.to_le_bytes());
        bytes[28..32].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            AtlasInputs::parse(&bytes),
            Err(AtlasInputError::LayoutOverflow { .. })
        ));

        assert!(matches!(
            AtlasInputs::new(0, 0, vec![], u32::MAX, u32::MAX, vec![]),
            Err(AtlasInputError::LayoutOverflow { .. })
        ));
    }

    #[test]
    fn comparator_reports_exact_batch_contour_and_texel_fields() {
        let original = inputs();

        let mut batch = original.clone();
        batch.patch_count += 1;
        assert_eq!(
            compare_cpp_to_rust(&original, &batch),
            Err(AtlasInputComparisonError::BatchOrDimensionField {
                field: "patch_count",
                cpp: 5,
                rust: 6,
            })
        );

        let mut contour = original.clone();
        contour.contours[0].path_id ^= 1;
        assert_eq!(
            compare_cpp_to_rust(&original, &contour),
            Err(AtlasInputComparisonError::ContourField {
                index: 0,
                field: "path_id",
                cpp: 7,
                rust: 6,
            })
        );

        let mut texel = original.clone();
        texel.texels[1][2] ^= 1;
        assert_eq!(
            compare_cpp_to_rust(&original, &texel),
            Err(AtlasInputComparisonError::Texel {
                x: 1,
                y: 0,
                channel: 2,
                cpp: 7,
                rust: 6,
            })
        );
    }
}
