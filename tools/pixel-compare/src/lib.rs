use png::{BitDepth, ColorType, Decoder, Encoder, Transformations};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tolerance {
    pub max_channel_delta: u8,
    pub max_different_pixels: u64,
}

impl Tolerance {
    pub const EXACT: Self = Self {
        max_channel_delta: 0,
        max_different_pixels: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffReport {
    pub width: u32,
    pub height: u32,
    pub different_pixels: u64,
    pub max_channel_delta: u8,
    pub within_tolerance: bool,
}

#[derive(Clone, Copy)]
pub struct ReferenceIdentity<'a> {
    pub id: &'a str,
    pub stream: &'a Path,
    pub frame: usize,
    pub mode: &'a str,
    pub reference: &'a Path,
}

pub fn validate_reference_identities<'a>(
    base: &Path,
    entries: impl IntoIterator<Item = ReferenceIdentity<'a>>,
) -> Result<(), String> {
    let mut owners = HashMap::<PathBuf, (PathBuf, usize, &str, &str)>::new();
    for entry in entries {
        let reference = normalize_path(base, entry.reference);
        let identity = (
            normalize_path(base, entry.stream),
            entry.frame,
            entry.mode,
            entry.id,
        );
        if let Some((stream, frame, mode, id)) = owners.insert(reference, identity.clone()) {
            if (stream.as_path(), frame, mode) != (identity.0.as_path(), identity.1, identity.2) {
                return Err(format!(
                    "reference {} is shared by incompatible entries {id} ({}, frame {frame}, {mode}) and {} ({}, frame {}, {}); references must be keyed by stream, frame, and mode",
                    entry.reference.display(),
                    stream.display(),
                    entry.id,
                    entry.stream.display(),
                    entry.frame,
                    entry.mode,
                ));
            }
        }
    }
    Ok(())
}

fn normalize_path(base: &Path, path: &Path) -> PathBuf {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new(std::path::MAIN_SEPARATOR_STR)),
            Component::CurDir => {}
            Component::ParentDir => match normalized.components().next_back() {
                Some(Component::Normal(_)) => {
                    normalized.pop();
                }
                Some(Component::ParentDir) | None if !normalized.has_root() => {
                    normalized.push("..");
                }
                _ => {}
            },
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug)]
pub enum PixelError {
    DimensionMismatch {
        expected: (u32, u32),
        actual: (u32, u32),
    },
    InvalidPixelLength {
        expected: usize,
        actual: usize,
    },
    UnsupportedPng {
        color_type: ColorType,
        bit_depth: BitDepth,
    },
    Io(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for PixelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionMismatch { expected, actual } => {
                write!(
                    f,
                    "image dimensions differ: expected {expected:?}, actual {actual:?}"
                )
            }
            Self::InvalidPixelLength { expected, actual } => {
                write!(f, "RGBA buffer has {actual} bytes, expected {expected}")
            }
            Self::UnsupportedPng {
                color_type,
                bit_depth,
            } => write!(
                f,
                "unsupported decoded PNG format {color_type:?}/{bit_depth:?}"
            ),
            Self::Io(error) => error.fmt(f),
        }
    }
}

impl Error for PixelError {}

impl RgbaImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, PixelError> {
        let expected = rgba_len(width, height)?;
        if pixels.len() != expected {
            return Err(PixelError::InvalidPixelLength {
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

    pub fn read_png(path: impl AsRef<Path>) -> Result<Self, PixelError> {
        let file = File::open(path).map_err(io_error)?;
        let mut decoder = Decoder::new(BufReader::new(file));
        decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
        let mut reader = decoder.read_info().map_err(io_error)?;
        let output_size = reader
            .output_buffer_size()
            .ok_or_else(|| message_error("decoded PNG is too large"))?;
        let mut decoded = vec![0; output_size];
        let info = reader.next_frame(&mut decoded).map_err(io_error)?;
        decoded.truncate(info.buffer_size());
        let pixels = match (info.color_type, info.bit_depth) {
            (ColorType::Rgba, BitDepth::Eight) => decoded,
            (ColorType::Rgb, BitDepth::Eight) => decoded
                .chunks_exact(3)
                .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                .collect(),
            (ColorType::Grayscale, BitDepth::Eight) => decoded
                .into_iter()
                .flat_map(|value| [value, value, value, 255])
                .collect(),
            (ColorType::GrayscaleAlpha, BitDepth::Eight) => decoded
                .chunks_exact(2)
                .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
                .collect(),
            (color_type, bit_depth) => {
                return Err(PixelError::UnsupportedPng {
                    color_type,
                    bit_depth,
                });
            }
        };
        Self::new(info.width, info.height, pixels)
    }

    pub fn write_png(&self, path: impl AsRef<Path>) -> Result<(), PixelError> {
        let file = File::create(path).map_err(io_error)?;
        let mut encoder = Encoder::new(BufWriter::new(file), self.width, self.height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(io_error)?;
        writer.write_image_data(&self.pixels).map_err(io_error)
    }
}

pub fn compare(
    expected: &RgbaImage,
    actual: &RgbaImage,
    tolerance: Tolerance,
) -> Result<DiffReport, PixelError> {
    require_same_dimensions(expected, actual)?;
    let mut different_pixels = 0u64;
    let mut max_channel_delta = 0u8;
    for (expected, actual) in expected
        .pixels
        .chunks_exact(4)
        .zip(actual.pixels.chunks_exact(4))
    {
        let pixel_delta = expected
            .iter()
            .zip(actual)
            .map(|(expected, actual)| expected.abs_diff(*actual))
            .max()
            .unwrap_or(0);
        max_channel_delta = max_channel_delta.max(pixel_delta);
        if pixel_delta > tolerance.max_channel_delta {
            different_pixels += 1;
        }
    }
    Ok(DiffReport {
        width: expected.width,
        height: expected.height,
        different_pixels,
        max_channel_delta,
        within_tolerance: different_pixels <= tolerance.max_different_pixels,
    })
}

pub fn artifact(expected: &RgbaImage, actual: &RgbaImage) -> Result<RgbaImage, PixelError> {
    require_same_dimensions(expected, actual)?;
    let width = expected
        .width
        .checked_mul(3)
        .ok_or_else(|| message_error("artifact width overflow"))?;
    let mut pixels = Vec::with_capacity(rgba_len(width, expected.height)?);
    let row_bytes = expected.width as usize * 4;
    for row in 0..expected.height as usize {
        let start = row * row_bytes;
        let end = start + row_bytes;
        let expected_row = &expected.pixels[start..end];
        let actual_row = &actual.pixels[start..end];
        pixels.extend_from_slice(expected_row);
        pixels.extend_from_slice(actual_row);
        for (expected, actual) in expected_row.chunks_exact(4).zip(actual_row.chunks_exact(4)) {
            let delta = expected
                .iter()
                .zip(actual)
                .map(|(expected, actual)| expected.abs_diff(*actual))
                .max()
                .unwrap_or(0);
            pixels.extend_from_slice(&[delta, 0, 255u8.saturating_sub(delta), 255]);
        }
    }
    RgbaImage::new(width, expected.height, pixels)
}

fn require_same_dimensions(expected: &RgbaImage, actual: &RgbaImage) -> Result<(), PixelError> {
    if (expected.width, expected.height) != (actual.width, actual.height) {
        return Err(PixelError::DimensionMismatch {
            expected: (expected.width, expected.height),
            actual: (actual.width, actual.height),
        });
    }
    Ok(())
}

fn rgba_len(width: u32, height: u32) -> Result<usize, PixelError> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| message_error("RGBA dimensions overflow"))
}

fn io_error(error: impl Error + Send + Sync + 'static) -> PixelError {
    PixelError::Io(Box::new(error))
}

fn message_error(message: &'static str) -> PixelError {
    io_error(std::io::Error::other(message))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(pixels: &[[u8; 4]]) -> RgbaImage {
        RgbaImage::new(2, 1, pixels.iter().flatten().copied().collect()).unwrap()
    }

    #[test]
    fn applies_channel_and_pixel_budgets() {
        let expected = image(&[[10, 20, 30, 255], [1, 2, 3, 255]]);
        let actual = image(&[[12, 20, 30, 255], [1, 8, 3, 255]]);
        let report = compare(
            &expected,
            &actual,
            Tolerance {
                max_channel_delta: 2,
                max_different_pixels: 1,
            },
        )
        .unwrap();
        assert_eq!(report.different_pixels, 1);
        assert_eq!(report.max_channel_delta, 6);
        assert!(report.within_tolerance);
    }

    #[test]
    fn normalizes_reference_aliases_without_collapsing_leading_parents() {
        let base = Path::new("/repo/project");
        assert_eq!(
            normalize_path(base, Path::new("a/sub/../shared.png")),
            Path::new("/repo/project/a/shared.png")
        );
        assert_eq!(
            normalize_path(base, Path::new("../../shared.png")),
            Path::new("/shared.png")
        );
        assert_eq!(
            normalize_path(base, Path::new("fixtures/shared.png")),
            normalize_path(base, Path::new("/repo/project/fixtures/shared.png"))
        );
    }

    #[test]
    fn artifact_places_expected_actual_and_heatmap_side_by_side() {
        let expected = image(&[[10, 20, 30, 255], [1, 2, 3, 255]]);
        let actual = image(&[[12, 20, 30, 255], [1, 8, 3, 255]]);
        let artifact = artifact(&expected, &actual).unwrap();
        assert_eq!((artifact.width, artifact.height), (6, 1));
        assert_eq!(&artifact.pixels[..8], &expected.pixels);
        assert_eq!(&artifact.pixels[8..16], &actual.pixels);
        assert_eq!(&artifact.pixels[16..20], &[2, 0, 253, 255]);
        assert_eq!(&artifact.pixels[20..24], &[6, 0, 249, 255]);
    }

    #[test]
    fn png_round_trip_is_rgba_exact() {
        let path = std::env::temp_dir().join(format!("pixel-compare-{}.png", std::process::id()));
        let expected = image(&[[10, 20, 30, 255], [1, 2, 3, 4]]);
        expected.write_png(&path).unwrap();
        let actual = RgbaImage::read_png(&path).unwrap();
        let _ = std::fs::remove_file(path);
        assert_eq!(actual, expected);
    }
}
