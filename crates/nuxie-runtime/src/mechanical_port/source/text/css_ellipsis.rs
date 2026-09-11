//! CSS ellipsis planning for the checked single-line occurrence policy.
//!
//! Preparation extracts grapheme boundaries from original LTR shaping.
//! Retained glyphs keep their original shapes and advances, including a
//! ligature intersected by the scalar cut. The caller supplies marker styling;
//! bidi ordering remains unsupported. Truncation never slices a glyph.

use super::text_engine::{GlyphRun, TextRun};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidShape {
    InvalidScalar,
    UnsupportedDirection,
    InvalidClusters,
    NonFiniteAdvance,
    MissingFont,
    UnexpectedShaping,
    InvalidMeasurement,
}

/// Painted runs with a separately measured marker. The final prefix advance
/// retains original glyph advances, including a partially retained cluster.
pub struct Prepared {
    pub runs: Vec<GlyphRun>,
    pub source_end: usize,
    pub marker_start: f32,
}

/// Prepare a single-run LTR ellipsis. `None` means the original
/// text fits. The caller supplies the block's marker style independently from
/// the text run's style. This does not mutate the original shape or source.
pub fn prepare(
    text: &[u32],
    original: &GlyphRun,
    marker_style: &TextRun,
    width: f32,
) -> Result<Option<Prepared>, InvalidShape> {
    prepare_runs(text, std::slice::from_ref(original), marker_style, width)
}

/// Prepare a complete LTR line whose source indices share the same scalar
/// coordinate space. Retained glyphs are copied without reshaping, matching
/// Chrome ShapeResultView truncation. Marker styling remains independent.
pub fn prepare_runs(
    text: &[u32],
    originals: &[GlyphRun],
    marker_style: &TextRun,
    width: f32,
) -> Result<Option<Prepared>, InvalidShape> {
    let mut merged = GlyphRun::new(0);
    for original in originals {
        if original.level & 1 != 0 {
            return Err(InvalidShape::UnsupportedDirection);
        }
        if original.glyphs.len() != original.advances.len()
            || original.glyphs.len() != original.text_indices.len()
            || original.glyphs.is_empty()
        {
            return Err(InvalidShape::InvalidClusters);
        }
        if merged
            .text_indices
            .last()
            .is_some_and(|&last| last >= original.text_indices[0])
        {
            return Err(InvalidShape::InvalidClusters);
        }
        merged.glyphs.extend_from_slice(&original.glyphs);
        merged
            .text_indices
            .extend_from_slice(&original.text_indices);
        merged.advances.extend_from_slice(&original.advances);
    }
    let boundaries = boundaries_for_run(text, &merged)?;
    let marker_font = marker_style
        .font
        .as_ref()
        .ok_or(InvalidShape::MissingFont)?;
    let marker_text: &[u32] = if marker_font.has_glyph(0x2026) {
        &[0x2026]
    } else {
        &[46, 46, 46]
    };
    let mut marker_style = marker_style.clone();
    marker_style.unichar_count = marker_text.len() as u32;
    // Chrome shapes the synthetic marker without source letter spacing.
    marker_style.letter_spacing = 0.0;
    let mut marker = marker_font.shape_text(marker_text, &[marker_style], 0);
    if marker.len() != 1 || marker[0].runs.len() != 1 {
        return Err(InvalidShape::UnexpectedShaping);
    }
    let marker = marker.remove(0).runs.remove(0);
    let marker_width: f32 = marker.advances.iter().sum();
    let Plan::Ellipsis {
        source_end,
        marker_start: _,
    } = plan(&boundaries, width, marker_width).map_err(|_| InvalidShape::InvalidMeasurement)?
    else {
        return Ok(None);
    };
    let mut runs = Vec::new();
    let mut pen = 0.0;
    for (index, original) in originals.iter().enumerate() {
        let start = original.text_indices[0] as usize;
        let end = originals
            .get(index + 1)
            .map_or(text.len(), |next| next.text_indices[0] as usize);
        if source_end <= start {
            break;
        }
        if source_end >= end {
            pen += original.advances.iter().sum::<f32>();
            runs.push(original.clone());
            continue;
        }
        // ShapeResultView keeps a glyph whose cluster starts before the cut,
        // even when its source cluster extends past it. Reshaping the prefix
        // would replace an optional ffi ligature with f/ff and move the marker.
        let retained = original.text_indices.partition_point(|&offset| (offset as usize) < source_end);
        let mut prefix = original.clone();
        prefix.glyphs.truncate(retained);
        prefix.advances.truncate(retained);
        prefix.text_indices.truncate(retained);
        prefix.offsets.truncate(retained);
        prefix.xpos.truncate(retained + 1);
        for offset in &mut prefix.breaks {
            *offset = (*offset).min(retained as u32);
        }
        prefix.joiners.retain(|&offset| (offset as usize) < source_end);
        pen += prefix.advances.iter().sum::<f32>();
        runs.push(prefix);
        break;
    }
    runs.push(marker);
    Ok(Some(Prepared {
        runs,
        source_end,
        marker_start: pen,
    }))
}

/// Extract grapheme-end advances from one complete LTR shaped run.
/// Source indices are Unicode scalar offsets relative to `text`, not bytes.
/// A ligature's advance is divided among its graphemes; combining sequences
/// remain indivisible. `prepare_runs` assembles LTR runs before extraction;
/// bidi ordering is not supported.
pub fn boundaries_for_run(text: &[u32], run: &GlyphRun) -> Result<Vec<Boundary>, InvalidShape> {
    if run.level & 1 != 0 {
        return Err(InvalidShape::UnsupportedDirection);
    }
    let source: String = text
        .iter()
        .map(|&c| char::from_u32(c).ok_or(InvalidShape::InvalidScalar))
        .collect::<Result<_, _>>()?;
    if run.glyphs.len() != run.text_indices.len() || run.glyphs.len() != run.advances.len() {
        return Err(InvalidShape::InvalidClusters);
    }
    if text.is_empty() {
        return if run.glyphs.is_empty() {
            Ok(Vec::new())
        } else {
            Err(InvalidShape::InvalidClusters)
        };
    }
    if run.text_indices.first() != Some(&0)
        || run.text_indices.windows(2).any(|w| w[0] > w[1])
        || run.text_indices.iter().any(|&i| i as usize >= text.len())
    {
        return Err(InvalidShape::InvalidClusters);
    }
    let mut scalar_end = 0;
    let ends: Vec<usize> = source
        .graphemes(true)
        .map(|g| {
            scalar_end += g.chars().count();
            scalar_end
        })
        .collect();
    let mut result = Vec::with_capacity(ends.len());
    let (mut glyph, mut boundary, mut pen) = (0, 0, 0.0_f32);
    while glyph < run.glyphs.len() {
        let start = run.text_indices[glyph];
        let mut advance = 0.0;
        while glyph < run.glyphs.len() && run.text_indices[glyph] == start {
            advance += run.advances[glyph];
            glyph += 1;
        }
        let end = run
            .text_indices
            .get(glyph)
            .map_or(text.len(), |&i| i as usize);
        let first = boundary;
        while boundary < ends.len() && ends[boundary] <= end {
            boundary += 1;
        }
        let count = boundary - first;
        for (offset, &source_end) in ends[first..boundary].iter().enumerate() {
            result.push(Boundary {
                source_end,
                advance: pen + advance * (offset + 1) as f32 / count as f32,
            });
        }
        pen += advance;
        if !pen.is_finite() || result.last().is_some_and(|b| !b.advance.is_finite()) {
            return Err(InvalidShape::NonFiniteAdvance);
        }
    }
    Ok(result)
}

/// A legal truncation boundary and its original line-relative advance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Boundary {
    pub source_end: usize,
    pub advance: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Plan {
    Unchanged,
    Ellipsis {
        source_end: usize,
        /// Preserve this position even if repainting changes prefix metrics.
        marker_start: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidMeasurement {
    NonFinite,
    NegativeMarkerWidth,
    UnorderedSourceBoundary,
}

/// Select a retained prefix for a single visual LTR text sequence.
///
/// Boundaries must include every grapheme end, including the full text end,
/// with strictly increasing nonzero source offsets. Advances may decrease
/// (for example with negative letter spacing). Empty text needs no marker.
/// The marker width is measured independently in the block's ellipsis style.
/// A negative available width is treated as zero; clipping remains a paint
/// operation. The first grapheme is retained even if it or the marker cannot
/// fit. The full original text fits without reserving space for a marker.
pub fn plan(
    boundaries: &[Boundary],
    available_width: f32,
    marker_width: f32,
) -> Result<Plan, InvalidMeasurement> {
    if !available_width.is_finite() || !marker_width.is_finite() {
        return Err(InvalidMeasurement::NonFinite);
    }
    if marker_width < 0.0 {
        return Err(InvalidMeasurement::NegativeMarkerWidth);
    }
    let mut previous = 0;
    for boundary in boundaries {
        if !boundary.advance.is_finite() {
            return Err(InvalidMeasurement::NonFinite);
        }
        if boundary.source_end <= previous {
            return Err(InvalidMeasurement::UnorderedSourceBoundary);
        }
        previous = boundary.source_end;
    }
    let Some(last) = boundaries.last() else {
        return Ok(Plan::Unchanged);
    };
    let available_width = available_width.max(0.0);
    if last.advance <= available_width {
        return Ok(Plan::Unchanged);
    }
    let mut retained = boundaries[0];
    for boundary in &boundaries[..boundaries.len() - 1] {
        if boundary.advance <= available_width - marker_width {
            retained = *boundary;
        }
    }
    Ok(Plan::Ellipsis {
        source_end: retained.source_end,
        marker_start: retained.advance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires NUXIE_ELLIPSIS_LIGATURE_FONT pointing to OpenSans-Regular.ttf"]
    fn truncation_retains_original_optional_ligature_glyph() {
        use super::super::font_hb::{HbFont, ShapingPrecision};
        let bytes = std::fs::read(std::env::var("NUXIE_ELLIPSIS_LIGATURE_FONT").unwrap()).unwrap();
        let font = HbFont::decode(&bytes).unwrap();
        let font = font.as_any().downcast_ref::<HbFont>().unwrap()
            .with_shaping_precision(ShapingPrecision::CssExperimental);
        let text: Vec<u32> = "ffi long title".chars().map(u32::from).collect();
        let style = TextRun { font: Some(font.clone()), size: 24.0,
            line_height: 40.0, letter_spacing: 0.0,
            unichar_count: text.len() as u32, script: 0, style_id: 0, level: 0 };
        let original = font.shape_text(&text, &[style.clone()], 0);
        let run = &original[0].runs[0];
        assert_eq!(&run.text_indices[..2], &[0, 3], "fixture must form ffi ligature");
        for width in [24.0, 40.0] {
            let prepared = prepare_runs(&text, &original[0].runs, &style, width).unwrap().unwrap();
            assert_eq!(prepared.runs[0].glyphs, vec![run.glyphs[0]]);
            assert_eq!(prepared.runs[0].advances, vec![run.advances[0]]);
            assert_eq!(prepared.marker_start, run.advances[0]);
        }
    }

    #[test]
    #[ignore = "requires NUXIE_TEXT_TEST_FONT pointing to the Inter reference fixture"]
    fn marker_ignores_letter_spacing_while_source_keeps_it() {
        use super::super::font_hb::{HbFont, LetterSpacingMode, ShapingPrecision};
        let bytes = std::fs::read(std::env::var("NUXIE_TEXT_TEST_FONT").unwrap()).unwrap();
        let decoded = HbFont::decode(&bytes).unwrap();
        let precise = decoded.as_any().downcast_ref::<HbFont>().unwrap()
            .with_shaping_precision(ShapingPrecision::CssExperimental);
        let font = precise.as_any().downcast_ref::<HbFont>().unwrap()
            .with_letter_spacing_mode(LetterSpacingMode::CssLetterSpacingExperimental);
        let text: Vec<u32> = "ffi long title".chars().map(u32::from).collect();
        let mut marker_advance = None;
        for (spacing, expected_end) in [(0.0, 2), (2.0, 2), (-1.0, 3)] {
            let style = TextRun { font: Some(font.clone()), size: 24.0,
                line_height: 40.0, letter_spacing: spacing,
                unichar_count: text.len() as u32, script: 0, style_id: 0, level: 0 };
            let original = font.shape_text(&text, &[style.clone()], 0);
            let prepared = prepare_runs(&text, &original[0].runs, &style, 40.0).unwrap().unwrap();
            assert_eq!(prepared.source_end, expected_end, "spacing {spacing}");
            assert_eq!(prepared.runs[0].letter_spacing, spacing);
            let marker = prepared.runs.last().unwrap();
            assert_eq!(marker.letter_spacing, 0.0);
            let advance: f32 = marker.advances.iter().sum();
            if let Some(expected) = marker_advance { assert_eq!(advance, expected); }
            marker_advance = Some(advance);
        }
    }

    #[test]
    #[ignore = "requires NUXIE_TEXT_TEST_FONT pointing to the Inter reference fixture"]
    fn mixed_runs_keep_styles_source_offsets_and_block_marker_style() {
        use super::super::font_hb::HbFont;
        let bytes = std::fs::read(std::env::var("NUXIE_TEXT_TEST_FONT").unwrap()).unwrap();
        let font = HbFont::decode(&bytes).unwrap();
        let style = |size, count, style_id| TextRun {
            font: Some(font.clone()),
            size,
            line_height: 48.0,
            letter_spacing: 0.0,
            unichar_count: count,
            script: 0,
            style_id,
            level: 0,
        };
        let text: Vec<u32> = "ABCDEFGHIJK".chars().map(u32::from).collect();
        let shaped = font.shape_text(&text, &[style(24.0, 2, 7), style(32.0, 9, 8)], 0);
        let runs = &shaped[0].runs;
        assert_eq!(runs.len(), 2);
        let marker_style = style(20.0, 1, 9);
        let marker = font.shape_text(&[0x2026], &[marker_style.clone()], 0);
        let marker_width: f32 = marker[0].runs[0].advances.iter().sum();
        let first_width: f32 = runs[0].advances.iter().sum();
        let width = first_width + runs[1].advances[0] + marker_width + 0.001;
        let prepared = prepare_runs(&text, runs, &marker_style, width)
            .unwrap()
            .unwrap();
        assert_eq!(prepared.source_end, 3);
        assert_eq!(prepared.runs.len(), 3);
        assert_eq!(prepared.runs[0].glyphs, runs[0].glyphs);
        assert_eq!(prepared.runs[0].advances, runs[0].advances);
        assert_eq!(prepared.runs[0].style_id, 7);
        assert_eq!(prepared.runs[1].style_id, 8);
        assert_eq!(prepared.runs[1].size, 32.0);
        assert_eq!(prepared.runs[1].text_indices, vec![2]);
        assert_eq!(prepared.runs[2].style_id, 9);
        assert_eq!(prepared.runs[2].size, 20.0);
        assert!((prepared.marker_start - first_width - runs[1].advances[0]).abs() < 0.001);
    }

    #[test]
    #[ignore = "requires NUXIE_TEXT_TEST_FONT pointing to the Inter reference fixture"]
    fn real_inter_shaping_matches_chrome_first_grapheme_ranges() {
        use super::super::font_hb::{HbFont, ShapingPrecision};
        use super::super::text_engine::TextRun;
        let bytes =
            std::fs::read(std::env::var("NUXIE_TEXT_TEST_FONT").expect("Inter fixture path"))
                .unwrap();
        let decoded = HbFont::decode(&bytes).unwrap();
        let font = decoded
            .as_any()
            .downcast_ref::<HbFont>()
            .unwrap()
            .with_shaping_precision(ShapingPrecision::CssExperimental);
        for (source, first_end, chrome_width) in [
            ("ffi long title", 1, 7.65625_f32),
            ("A long title", 1, 16.359375),
            ("A\u{301} long title", 2, 16.359375),
        ] {
            let text: Vec<u32> = source.chars().map(u32::from).collect();
            let shaped = font.shape_text(
                &text,
                &[TextRun {
                    font: Some(font.clone()),
                    size: 24.0,
                    line_height: 40.0,
                    letter_spacing: 0.0,
                    unichar_count: text.len() as u32,
                    script: 0,
                    style_id: 0,
                    level: 0,
                }],
                0,
            );
            assert_eq!(shaped.len(), 1);
            assert_eq!(shaped[0].runs.len(), 1);
            let boundaries = boundaries_for_run(&text, &shaped[0].runs[0]).unwrap();
            let original = &shaped[0].runs[0];
            let original_advances = original.advances.clone();
            let marker_style = TextRun {
                font: Some(font.clone()),
                size: 24.0,
                line_height: 40.0,
                letter_spacing: 0.0,
                unichar_count: 1,
                script: 0,
                style_id: 0,
                level: 0,
            };
            let prepared = prepare(&text, original, &marker_style, 8.0)
                .unwrap()
                .unwrap();
            assert_eq!(prepared.source_end, first_end);
            assert_eq!(prepared.runs.len(), 2);
            assert!(
                (prepared.runs[0].advances.iter().sum::<f32>() - prepared.marker_start).abs()
                    < 0.0001
            );
            assert_eq!(
                prepared.runs[1].glyphs.len(),
                1,
                "Inter uses U+2026, not three periods"
            );
            assert_eq!(original.advances, original_advances);
            assert!(
                prepare(&text, original, &marker_style, 1000.0)
                    .unwrap()
                    .is_none()
            );
            assert_eq!(boundaries[0].source_end, first_end);
            // DOM Range widths are quantized to 1/64 CSS pixel.
            assert!(
                (boundaries[0].advance - chrome_width).abs() <= 1.0 / 64.0,
                "{source}: {} versus {chrome_width}",
                boundaries[0].advance
            );
            assert_eq!(
                plan(&boundaries, 8.0, 24.0).unwrap(),
                Plan::Ellipsis {
                    source_end: first_end,
                    marker_start: boundaries[0].advance
                }
            );
        }
    }

    #[test]
    fn extracts_ligature_and_combining_boundaries() {
        let mut run = GlyphRun::new(3);
        run.text_indices = vec![0, 2, 3];
        run.advances = vec![15.3125, 7.6875, 12.0];
        let text: Vec<u32> = "ffiX".chars().map(u32::from).collect();
        assert_eq!(
            boundaries_for_run(&text, &run).unwrap(),
            boundaries(&[(1, 7.65625), (2, 15.3125), (3, 23.0), (4, 35.0)])
        );
        run.text_indices = vec![0, 0, 2];
        run.advances = vec![16.0, 0.0, 12.0];
        let text: Vec<u32> = "A\u{301}X".chars().map(u32::from).collect();
        assert_eq!(
            boundaries_for_run(&text, &run).unwrap(),
            boundaries(&[(2, 16.0), (3, 28.0)])
        );
    }

    #[test]
    fn emoji_sequences_use_scalar_offsets_without_internal_boundaries() {
        let text: Vec<u32> = "👩‍💻🇺🇸X".chars().map(u32::from).collect();
        let mut run = GlyphRun::new(3);
        run.text_indices = vec![0, 3, 5];
        run.advances = vec![24.0, 24.0, 12.0];
        assert_eq!(
            boundaries_for_run(&text, &run).unwrap(),
            boundaries(&[(3, 24.0), (5, 48.0), (6, 60.0)])
        );
        run.level = 1;
        assert_eq!(
            boundaries_for_run(&text, &run),
            Err(InvalidShape::UnsupportedDirection)
        );
        run.level = 0;
        run.text_indices = vec![0, 5, 3];
        assert_eq!(
            boundaries_for_run(&text, &run),
            Err(InvalidShape::InvalidClusters)
        );
    }

    fn boundaries(values: &[(usize, f32)]) -> Vec<Boundary> {
        values
            .iter()
            .map(|&(source_end, advance)| Boundary {
                source_end,
                advance,
            })
            .collect()
    }

    #[test]
    fn narrow_box_preserves_original_prefix_advance() {
        // Measured first-f range in Chrome's Inter ffi reproducer. Painting
        // this prefix independently must not move the marker to its new width.
        let text = boundaries(&[(1, 7.65625), (2, 15.3125), (3, 23.0), (4, 80.0)]);
        for width in [0.0, 8.0, 16.0, 20.0, 24.0, 30.0] {
            assert_eq!(
                plan(&text, width, 24.0),
                Ok(Plan::Ellipsis {
                    source_end: 1,
                    marker_start: 7.65625
                })
            );
        }
        assert_eq!(
            plan(&text, 40.0, 24.0),
            Ok(Plan::Ellipsis {
                source_end: 2,
                marker_start: 15.3125
            })
        );
        assert_eq!(
            plan(&text, 48.0, 24.0),
            Ok(Plan::Ellipsis {
                source_end: 3,
                marker_start: 23.0
            })
        );
    }

    #[test]
    fn resize_recomputes_from_original_boundaries() {
        let text = boundaries(&[(1, 16.0), (3, 32.0), (4, 48.0), (8, 80.0)]);
        assert_eq!(plan(&text, 80.0, 24.0), Ok(Plan::Unchanged));
        assert_eq!(
            plan(&text, 56.0, 24.0),
            Ok(Plan::Ellipsis {
                source_end: 3,
                marker_start: 32.0
            })
        );
        assert_eq!(
            plan(&text, 8.0, 24.0),
            Ok(Plan::Ellipsis {
                source_end: 1,
                marker_start: 16.0
            })
        );
        assert_eq!(plan(&text, 80.0, 24.0), Ok(Plan::Unchanged));
    }

    #[test]
    fn never_splits_a_supplied_grapheme_boundary() {
        let text = boundaries(&[(2, 16.359375), (5, 40.0)]);
        assert_eq!(
            plan(&text, 8.0, 24.0),
            Ok(Plan::Ellipsis {
                source_end: 2,
                marker_start: 16.359375
            })
        );
        assert_eq!(plan(&[], 0.0, 24.0), Ok(Plan::Unchanged));
        assert_eq!(plan(&text[..1], 17.0, 24.0), Ok(Plan::Unchanged));
    }

    #[test]
    fn scans_past_decreasing_advances() {
        let text = boundaries(&[(1, 10.0), (2, 30.0), (3, 20.0), (4, 60.0)]);
        assert_eq!(
            plan(&text, 44.0, 24.0),
            Ok(Plan::Ellipsis {
                source_end: 3,
                marker_start: 20.0
            })
        );
    }

    #[test]
    fn rejects_invalid_measurements() {
        assert_eq!(
            plan(&[], f32::NAN, 24.0),
            Err(InvalidMeasurement::NonFinite)
        );
        assert_eq!(
            plan(&[], 30.0, -1.0),
            Err(InvalidMeasurement::NegativeMarkerWidth)
        );
        assert_eq!(
            plan(&boundaries(&[(1, f32::INFINITY)]), 30.0, 24.0),
            Err(InvalidMeasurement::NonFinite)
        );
        assert_eq!(
            plan(&boundaries(&[(1, 10.0), (1, 20.0)]), 30.0, 24.0),
            Err(InvalidMeasurement::UnorderedSourceBoundary)
        );
    }
}
