use png::{BitDepth, ColorType, Decoder, Encoder, Transformations};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Cursor, Seek};
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
        Self::decode_png_reader(BufReader::new(file))
    }

    pub fn decode_png(bytes: &[u8]) -> Result<Self, PixelError> {
        Self::decode_png_reader(Cursor::new(bytes))
    }

    fn decode_png_reader(reader: impl BufRead + Seek) -> Result<Self, PixelError> {
        let mut decoder = Decoder::new(reader);
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

    pub fn read_riveabl(path: impl AsRef<Path>) -> Result<Self, PixelError> {
        const HEADER_SIZE: usize = 20;
        let bytes = fs::read(path).map_err(io_error)?;
        if bytes.len() < HEADER_SIZE || &bytes[..8] != b"RIVEABL\0" {
            return Err(message_error("invalid or truncated RIVEABL header"));
        }
        let word = |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        if word(8) != 1 {
            return Err(message_error("unsupported RIVEABL version"));
        }
        Self::new(word(12), word(16), bytes[HEADER_SIZE..].to_vec())
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

/// How far a frame may sit from a reference drawn on a different Apple GPU
/// (UNIV-6384).
///
/// The renderer's Metal shaders are built with fast math, and Metal finishes
/// compiling each one for the GPU it runs on. An Apple M2 Max on macOS 27 and
/// an Apple M5 Max on macOS 26 therefore draw the same draws with slightly
/// different anti-aliased edges and gradient rounding. Measured on every
/// Metal GM: at most 2 levels of 255 in smooth areas; every larger
/// difference on an edge, at most 18 levels, on at most 0.56% of a frame.
///
/// A comparison passes when:
/// 1. no channel of any pixel moves by more than `max_channel_delta`; or
/// 2. every pixel that does is an edge pixel, one whose 3x3 neighborhood in
///    the expected image spans at least `edge_contrast` levels in some
///    channel; and no edge pixel moves by more than `max_edge_delta`; and at
///    most `max_edge_percent` percent of the frame's pixels are such edge
///    pixels.
///
/// Edges are read from the expected image only, so a changed pixel cannot
/// make the edge that excuses it. A recolor, a missing or extra draw, or any
/// difference over `max_channel_delta` in a smooth area of the expected
/// image fails, and so does a shape moved by a pixel or more across an edge
/// of more than `max_edge_delta` levels. What it cannot catch: a change of
/// up to `max_channel_delta` levels anywhere; a nudge of a fraction of a
/// pixel that moves its edge pixels by at most `max_edge_delta` levels; and a
/// missing or changed draw of at most `max_edge_delta` levels of contrast
/// whose pixels all lie on the expected image's edges, such as a faint
/// hairline, on at most `max_edge_percent` percent of the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuTolerance {
    pub max_channel_delta: u8,
    pub edge_contrast: u8,
    pub max_edge_delta: u8,
    pub max_edge_percent: u8,
}

impl GpuTolerance {
    /// The one rule for comparing Metal frames across Apple GPUs.
    pub const GPU_EDGES: Self = Self {
        max_channel_delta: 2,
        edge_contrast: 16,
        max_edge_delta: 24,
        max_edge_percent: 1,
    };
}

/// One pixel, where it is and its value on each side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DifferingPixel {
    pub x: u32,
    pub y: u32,
    pub expected: [u8; 4],
    pub actual: [u8; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuDiffReport {
    pub width: u32,
    pub height: u32,
    /// Pixels over `max_channel_delta` on one of the expected image's edges.
    pub edge_pixels: u64,
    /// Pixels over `max_channel_delta` off an edge. Any one fails.
    pub smooth_pixels: u64,
    pub max_edge_delta: u8,
    pub max_smooth_delta: u8,
    /// The box, in pixels, `[left, top, right, bottom)`, holding every pixel
    /// over `max_channel_delta`.
    pub differing_box: Option<[u32; 4]>,
    /// The first pixel that fails on its own: off an edge, or on one by more
    /// than `max_edge_delta`.
    pub first_failure: Option<DifferingPixel>,
    pub within_tolerance: bool,
}

impl fmt::Display for GpuDiffReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} edge pixels (at most {} levels) and {} smooth pixels (at most {} levels) differ in a {}x{} frame",
            self.edge_pixels,
            self.max_edge_delta,
            self.smooth_pixels,
            self.max_smooth_delta,
            self.width,
            self.height
        )?;
        if let Some([left, top, right, bottom]) = self.differing_box {
            write!(f, ", in pixels [{left},{top},{right},{bottom})")?;
        }
        if let Some(pixel) = self.first_failure {
            write!(
                f,
                "; first failing pixel ({},{}) expected rgba{:?}, actual rgba{:?}",
                pixel.x, pixel.y, pixel.expected, pixel.actual
            )?;
        }
        Ok(())
    }
}

/// Compares a frame with a reference by [`GpuTolerance`]'s rule.
pub fn compare_gpu(
    expected: &RgbaImage,
    actual: &RgbaImage,
    tolerance: GpuTolerance,
) -> Result<GpuDiffReport, PixelError> {
    require_same_dimensions(expected, actual)?;
    let (width, height) = (expected.width as usize, expected.height as usize);
    let pixel = |image: &RgbaImage, x: usize, y: usize| -> [u8; 4] {
        let index = (y * width + x) * 4;
        image.pixels[index..index + 4].try_into().unwrap()
    };
    let spans_an_edge = |image: &RgbaImage, x: usize, y: usize| {
        let mut low = [u8::MAX; 4];
        let mut high = [0u8; 4];
        for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
            for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                let value = pixel(image, nx, ny);
                for channel in 0..4 {
                    low[channel] = low[channel].min(value[channel]);
                    high[channel] = high[channel].max(value[channel]);
                }
            }
        }
        (0..4).any(|channel| high[channel] - low[channel] >= tolerance.edge_contrast)
    };
    let mut report = GpuDiffReport {
        width: expected.width,
        height: expected.height,
        edge_pixels: 0,
        smooth_pixels: 0,
        max_edge_delta: 0,
        max_smooth_delta: 0,
        differing_box: None,
        first_failure: None,
        within_tolerance: true,
    };
    for y in 0..height {
        for x in 0..width {
            let (e, a) = (pixel(expected, x, y), pixel(actual, x, y));
            let delta = (0..4).map(|c| e[c].abs_diff(a[c])).max().unwrap_or(0);
            if delta <= tolerance.max_channel_delta {
                continue;
            }
            let (px, py) = (x as u32, y as u32);
            report.differing_box = Some(match report.differing_box {
                None => [px, py, px + 1, py + 1],
                Some([l, t, r, b]) => [l.min(px), t.min(py), r.max(px + 1), b.max(py + 1)],
            });
            let on_edge = spans_an_edge(expected, x, y);
            let fails = if on_edge {
                report.edge_pixels += 1;
                report.max_edge_delta = report.max_edge_delta.max(delta);
                delta > tolerance.max_edge_delta
            } else {
                report.smooth_pixels += 1;
                report.max_smooth_delta = report.max_smooth_delta.max(delta);
                true
            };
            if fails && report.first_failure.is_none() {
                report.first_failure = Some(DifferingPixel {
                    x: px,
                    y: py,
                    expected: e,
                    actual: a,
                });
            }
        }
    }
    let pixels = width as u64 * height as u64;
    report.within_tolerance = report.smooth_pixels == 0
        && report.max_edge_delta <= tolerance.max_edge_delta
        && report.edge_pixels * 100 <= u64::from(tolerance.max_edge_percent) * pixels;
    Ok(report)
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

    const SIDE: u32 = 64;
    const BACKGROUND: [u8; 4] = [20, 30, 40, 255];

    /// A frame of `SIDE` square pixels: the background, then each fill as an
    /// anti-aliased box (`left, top, right, bottom` in pixels, any fraction),
    /// each pixel covered in proportion to its area inside the box.
    fn frame(fills: &[([f32; 4], [u8; 4])]) -> RgbaImage {
        let mut pixels = Vec::new();
        for y in 0..SIDE {
            for x in 0..SIDE {
                let mut color = BACKGROUND.map(f32::from);
                for ([left, top, right, bottom], fill) in fills {
                    let cover_x = ((x + 1) as f32).min(*right) - (x as f32).max(*left);
                    let cover_y = ((y + 1) as f32).min(*bottom) - (y as f32).max(*top);
                    let coverage = cover_x.max(0.0) * cover_y.max(0.0);
                    for channel in 0..3 {
                        color[channel] += (fill[channel] as f32 - color[channel]) * coverage;
                    }
                }
                pixels.extend(color.map(|value| value.round() as u8));
            }
        }
        RgbaImage::new(SIDE, SIDE, pixels).unwrap()
    }

    const CARD: [f32; 4] = [10.3, 12.6, 50.7, 44.2];
    const CARD_COLOR: [u8; 4] = [200, 180, 160, 255];

    fn gpu(expected: &RgbaImage, actual: &RgbaImage) -> GpuDiffReport {
        compare_gpu(expected, actual, GpuTolerance::GPU_EDGES).unwrap()
    }

    #[test]
    fn gpu_rule_passes_the_same_frame_and_a_drift_of_two_levels() {
        let card = frame(&[(CARD, CARD_COLOR)]);
        let same = gpu(&card, &card);
        assert!(same.within_tolerance);
        assert_eq!(
            (same.edge_pixels, same.smooth_pixels, same.differing_box),
            (0, 0, None)
        );
        let mut drift = card.clone();
        for value in drift.pixels.iter_mut() {
            *value = value.saturating_add(2);
        }
        assert!(gpu(&card, &drift).within_tolerance);
    }

    #[test]
    fn gpu_rule_passes_edge_rounding() {
        // A GPU's rounding moves partly covered pixels; the card's own pixels
        // and the background stay put.
        let card = frame(&[(CARD, CARD_COLOR)]);
        let mut rounded = card.clone();
        let mut moved = 0;
        for (index, value) in rounded.pixels.chunks_exact_mut(4).enumerate() {
            let covered = value[0] != BACKGROUND[0] && value[0] != CARD_COLOR[0];
            if covered && index % 4 == 0 {
                value[0] = value[0].saturating_add(12);
                moved += 1;
            }
        }
        let report = gpu(&card, &rounded);
        assert!(report.within_tolerance, "{report}");
        assert_eq!((report.edge_pixels, report.smooth_pixels), (moved, 0));
        assert_eq!(report.max_edge_delta, 12);
    }

    #[test]
    fn gpu_rule_fails_a_shape_moved_by_a_pixel() {
        let card = frame(&[(CARD, CARD_COLOR)]);
        let [left, top, right, bottom] = CARD;
        let moved = frame(&[([left + 1.0, top, right + 1.0, bottom], CARD_COLOR)]);
        let report = gpu(&card, &moved);
        assert!(!report.within_tolerance, "{report}");
        assert_eq!(report.smooth_pixels, 0, "a move changes only edges");
        assert!(
            report.max_edge_delta > GpuTolerance::GPU_EDGES.max_edge_delta,
            "{report}"
        );
    }

    #[test]
    fn gpu_rule_fails_a_recolor() {
        let card = frame(&[(CARD, CARD_COLOR)]);
        for levels in [3, 10] {
            let recolored = frame(&[(CARD, CARD_COLOR.map(|value| value - levels))]);
            let report = gpu(&card, &recolored);
            assert!(!report.within_tolerance, "recolor by {levels}: {report}");
            assert!(report.smooth_pixels > 0, "recolor by {levels}: {report}");
            assert_eq!(report.max_smooth_delta, levels);
        }
    }

    #[test]
    fn gpu_rule_fails_three_levels_in_a_smooth_area() {
        let flat = frame(&[]);
        let mut changed = flat.clone();
        changed.pixels[(30 * SIDE as usize + 30) * 4] += 3;
        let report = gpu(&flat, &changed);
        assert!(!report.within_tolerance, "{report}");
        assert_eq!((report.smooth_pixels, report.max_smooth_delta), (1, 3));
        let failure = report.first_failure.unwrap();
        assert_eq!((failure.x, failure.y), (30, 30));
        assert_eq!(report.differing_box, Some([30, 30, 31, 31]));
    }

    #[test]
    fn gpu_rule_fails_a_missing_draw() {
        let badge = ([20.5, 20.5, 30.0, 26.0], [240, 80, 60, 255]);
        let drawn = frame(&[(CARD, CARD_COLOR), badge]);
        let missing = frame(&[(CARD, CARD_COLOR)]);
        assert!(!gpu(&drawn, &missing).within_tolerance);
        // A missing hairline is all edge, and fails on how far it moves.
        let hairline = ([5.0, 4.0, 59.0, 5.0], [230, 230, 230, 255]);
        let report = gpu(&frame(&[hairline]), &frame(&[]));
        assert!(!report.within_tolerance, "{report}");
        assert_eq!(report.smooth_pixels, 0);
    }

    #[test]
    fn gpu_rule_fails_edge_differences_over_one_percent_of_the_frame() {
        // Stripes make every pixel an edge pixel; small moves on more than
        // 1% of them fail.
        let stripes = frame(
            &(0..SIDE)
                .step_by(2)
                .map(|x| ([x as f32, 0.0, x as f32 + 1.0, SIDE as f32], CARD_COLOR))
                .collect::<Vec<_>>(),
        );
        let pixels = (SIDE * SIDE) as usize;
        let allowed = pixels / 100;
        for (count, passes) in [(allowed, true), (allowed + 1, false)] {
            let mut changed = stripes.clone();
            for index in 0..count {
                changed.pixels[index * 4] = changed.pixels[index * 4].wrapping_add(5);
            }
            let report = gpu(&stripes, &changed);
            assert_eq!(report.edge_pixels, count as u64);
            assert_eq!(report.within_tolerance, passes, "{count} pixels: {report}");
        }
    }

    #[test]
    fn gpu_rule_fails_a_flat_pixel_that_would_make_its_own_edge() {
        // A changed pixel or patch is never its own edge: edges come from the
        // expected image only.
        let flat = frame(&[]);
        for side in [1usize, 2] {
            let mut changed = flat.clone();
            for dy in 0..side {
                for dx in 0..side {
                    changed.pixels[((30 + dy) * SIDE as usize + 30 + dx) * 4] += 20;
                }
            }
            let report = gpu(&flat, &changed);
            assert!(!report.within_tolerance, "{side}x{side}: {report}");
            assert_eq!(report.smooth_pixels, (side * side) as u64);
        }
        // So is an extra draw: its pixels sit on no edge of the expected frame.
        let extra = ([20.0, 20.0, 30.0, 30.0], [40, 50, 60, 255]);
        assert!(!gpu(&flat, &frame(&[extra])).within_tolerance);
    }

    #[test]
    fn gpu_rule_cannot_see_a_tenth_of_a_pixel_nudge() {
        // The stated limit: a nudge too small to move edge pixels by more
        // than `max_edge_delta` levels cannot be told from GPU rounding.
        let badge = [20.3, 20.6, 30.7, 26.2];
        let [left, top, right, bottom] = badge;
        let report = gpu(
            &frame(&[(badge, CARD_COLOR)]),
            &frame(&[([left + 0.1, top, right + 0.1, bottom], CARD_COLOR)]),
        );
        assert!(report.edge_pixels > 0);
        assert!(report.within_tolerance, "{report}");
    }

    #[test]
    fn gpu_rule_cannot_see_a_missing_faint_hairline() {
        // The stated limit: a draw of at most `max_edge_delta` levels of
        // contrast whose pixels all lie on the expected frame's edges.
        let faint = [
            BACKGROUND[0] + 20,
            BACKGROUND[1] + 20,
            BACKGROUND[2] + 20,
            255,
        ];
        let report = gpu(&frame(&[([5.0, 4.0, 30.0, 5.0], faint)]), &frame(&[]));
        assert_eq!((report.edge_pixels, report.smooth_pixels), (25, 0));
        assert!(report.within_tolerance, "{report}");
    }

    #[test]
    fn gpu_rule_refuses_frames_of_different_sizes() {
        let small = RgbaImage::new(1, 1, vec![0; 4]).unwrap();
        assert!(compare_gpu(&small, &frame(&[]), GpuTolerance::GPU_EDGES).is_err());
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
        let decoded = RgbaImage::decode_png(&fs::read(&path).unwrap()).unwrap();
        let _ = std::fs::remove_file(path);
        assert_eq!(actual, expected);
        assert_eq!(decoded, expected);
    }

    #[test]
    fn reads_exact_riveabl_payload_and_rejects_malformed_artifacts() {
        let root =
            std::env::temp_dir().join(format!("pixel-compare-riveabl-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("frame.rgba");
        let expected = image(&[[10, 20, 30, 255], [1, 2, 3, 4]]);
        let mut artifact = b"RIVEABL\0".to_vec();
        artifact.extend_from_slice(&1u32.to_le_bytes());
        artifact.extend_from_slice(&expected.width.to_le_bytes());
        artifact.extend_from_slice(&expected.height.to_le_bytes());
        artifact.extend_from_slice(&expected.pixels);
        fs::write(&path, &artifact).unwrap();
        assert_eq!(RgbaImage::read_riveabl(&path).unwrap(), expected);

        artifact[8..12].copy_from_slice(&2u32.to_le_bytes());
        fs::write(&path, &artifact).unwrap();
        assert!(RgbaImage::read_riveabl(&path)
            .unwrap_err()
            .to_string()
            .contains("version"));
        artifact.truncate(19);
        fs::write(&path, artifact).unwrap();
        assert!(RgbaImage::read_riveabl(&path).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
