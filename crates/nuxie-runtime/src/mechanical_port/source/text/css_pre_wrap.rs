//! Opt-in CSS wrapping. The raw Rive line breaker stays unchanged.
use super::super::text_engine::{GlyphLine, GlyphRun, Paragraph, TextAlign, TextWrap};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CssWrapPolicy {
    PreWrap,
    PreLine,
    Normal,
}

#[derive(Clone, Copy)]
struct Glyph {
    run: u32,
    index: u32,
    source: usize,
    start: f32,
    end: f32,
    advance: f32,
    tab: bool,
    content_end: usize,
    forced: bool,
}

struct Line {
    glyphs: GlyphLine,
    content_end: (u32, u32),
    full_width: f32,
    content_width: f32,
    soft: bool,
}

fn make_line(glyphs: &[Glyph], start: usize, end: usize, soft: bool) -> Line {
    let first = glyphs[start];
    let mut line = GlyphLine::at(first.run, first.index);
    // Cached prefix boundary avoids rescanning long runs of tabs at every
    // allowed break. Every fit candidate now needs constant-time width lookup.
    let content_end = if end > start {
        glyphs[end - 1].content_end.max(start)
    } else {
        start
    };
    if end > start {
        let last = glyphs[end - 1];
        line.end_run_index = last.run;
        line.end_glyph_index = last.index + 1;
    }
    Line {
        glyphs: line,
        content_end: if content_end > start {
            let last = glyphs[content_end - 1];
            (last.run, last.index + 1)
        } else {
            (first.run, first.index)
        },
        full_width: if end > start {
            glyphs[end - 1].end - first.start
        } else {
            0.0
        },
        content_width: if content_end > start {
            glyphs[content_end - 1].end - first.start
        } else {
            0.0
        },
        soft,
    }
}

#[derive(Clone, Copy)]
struct Layout {
    width: f32,
    wrap: bool,
    relative_tabs: bool,
}

fn position_glyphs(glyphs: &mut [Glyph], runs: &[GlyphRun], mut pen: f32, origin: f32) -> f32 {
    for glyph in glyphs {
        let run = &runs[glyph.run as usize];
        if glyph.tab {
            glyph.advance = run
                .font
                .as_ref()
                .and_then(|font| {
                    font.experimental_css_tab_advance(pen - origin, run.size, run.letter_spacing)
                })
                .unwrap_or(glyph.advance);
        }
        glyph.start = pen;
        pen += glyph.advance;
        glyph.end = pen;
    }
    pen
}

fn segment_lines(
    out: &mut Vec<Line>,
    glyphs: &mut [Glyph],
    runs: &[GlyphRun],
    breaks: &[bool],
    range: std::ops::Range<usize>,
    layout: Layout,
) -> f32 {
    let (start, end) = (range.start, range.end);
    let mut line_start = start;
    let mut previous = start;
    let mut pen = glyphs[start].start;
    let mut origin = pen;
    for boundary in start + 1..=end {
        if boundary != end
            && !(layout.wrap
                && glyphs[boundary].source != glyphs[boundary - 1].source
                && breaks[glyphs[boundary].source])
        {
            continue;
        }
        pen = if layout.relative_tabs {
            position_glyphs(&mut glyphs[previous..boundary], runs, pen, origin)
        } else {
            glyphs[boundary - 1].end
        };
        let candidate = make_line(glyphs, line_start, boundary, false);
        if layout.wrap && candidate.content_width > layout.width && previous > line_start {
            out.push(make_line(glyphs, line_start, previous, true));
            line_start = previous;
            origin = glyphs[previous].start;
            if layout.relative_tabs {
                // Only the pending token is retried at the new origin. Earlier
                // lines keep their positions; long documents stay linear.
                pen = position_glyphs(&mut glyphs[previous..boundary], runs, origin, origin);
            }
        }
        previous = boundary;
    }
    out.push(make_line(glyphs, line_start, end, false));
    pen
}

pub(super) fn break_lines(
    paragraphs: &mut [Paragraph],
    source: &[u32],
    width: f32,
    align: TextAlign,
    wrap: TextWrap,
    policy: CssWrapPolicy,
) -> Vec<Vec<GlyphLine>> {
    // UAX #14 uses byte indices; Rive glyph/source identities use scalar indices.
    // Only breaks that coincide with an actual shaped cluster boundary are used.
    let mut utf8 = String::new();
    let mut offsets = Vec::with_capacity(source.len() + 1);
    for &point in source {
        offsets.push(utf8.len());
        utf8.push(char::from_u32(point).unwrap_or(char::REPLACEMENT_CHARACTER));
    }
    offsets.push(utf8.len());
    let mut breaks = vec![false; source.len() + 1];
    for (byte, _) in unicode_linebreak::linebreaks(&utf8) {
        if let Ok(scalar) = offsets.binary_search(&byte) {
            breaks[scalar] = true;
        }
    }
    let auto_width = width < 0.0;
    let soft_wrap = !auto_width && wrap == TextWrap::Wrap;
    let mut layouts = Vec::with_capacity(paragraphs.len());
    let mut paragraph_width = if auto_width { 0.0 } else { width };
    for paragraph in paragraphs.iter_mut() {
        let mut glyphs = Vec::new();
        let mut content_end = 0;
        for (run_index, run) in paragraph.runs.iter().enumerate() {
            for (index, &source_index) in run.text_indices.iter().enumerate() {
                let point = source[source_index as usize];
                // Normal collapses ASCII whitespace in the compiler, but
                // retained Unicode separators contribute to fitting and
                // alignment, including at a line end. Pre-line and pre-wrap
                // instead hang separators; Chromium retains Ogham in pre-line.
                let hanging = match policy {
                    CssWrapPolicy::Normal => matches!(point, 9 | 32),
                    CssWrapPolicy::PreLine | CssWrapPolicy::PreWrap => {
                        (point == 9
                            || char::from_u32(point).is_some_and(|ch| {
                                ch.general_category() == GeneralCategory::SpaceSeparator
                            }))
                            && !(policy == CssWrapPolicy::PreLine && point == 0x1680)
                    }
                };
                if !hanging {
                    content_end = glyphs.len() + 1;
                }
                glyphs.push(Glyph {
                    run: run_index as u32,
                    index: index as u32,
                    source: source_index as usize,
                    start: run.xpos[index],
                    end: run.xpos[index + 1],
                    advance: run.advances[index],
                    tab: point == 9,
                    content_end,
                    forced: matches!(point, 10 | 0x85 | 0x2028 | 0x2029),
                });
            }
        }
        let layout = Layout {
            width,
            wrap: soft_wrap,
            relative_tabs: glyphs.iter().any(|g| g.tab),
        };
        let mut lines = Vec::new();
        let mut start = 0;
        for index in 0..glyphs.len() {
            if glyphs[index].forced {
                let pen = segment_lines(
                    &mut lines,
                    &mut glyphs,
                    &paragraph.runs,
                    &breaks,
                    start..index,
                    layout,
                );
                if layout.relative_tabs {
                    glyphs[index].start = pen;
                    glyphs[index].end = pen + glyphs[index].advance;
                    if index + 1 < glyphs.len() {
                        glyphs[index + 1].start = glyphs[index].end;
                    }
                }
                start = index + 1;
            }
        }
        if start < glyphs.len() {
            let end = glyphs.len();
            segment_lines(
                &mut lines,
                &mut glyphs,
                &paragraph.runs,
                &breaks,
                start..end,
                layout,
            );
        }
        if layout.relative_tabs {
            for glyph in &glyphs {
                let run = &mut paragraph.runs[glyph.run as usize];
                run.xpos[glyph.index as usize] = glyph.start;
                run.xpos[glyph.index as usize + 1] = glyph.end;
                run.advances[glyph.index as usize] = glyph.advance;
            }
        }
        if auto_width {
            for line in &lines {
                paragraph_width = paragraph_width.max(if policy != CssWrapPolicy::PreWrap {
                    line.content_width
                } else {
                    line.full_width
                });
            }
        }
        layouts.push(lines);
    }
    layouts
        .into_iter()
        .zip(paragraphs)
        .enumerate()
        .map(|(index, (layout, paragraph))| {
            let mut lines: Vec<_> = layout
                .iter()
                .map(|line| {
                    let mut glyphs = line.glyphs.clone();
                    if policy == CssWrapPolicy::Normal {
                        // Collapsed trailing ASCII spaces neither paint nor
                        // extend decorations. Retained Unicode spaces remain
                        // part of content_end under the normal policy.
                        (glyphs.end_run_index, glyphs.end_glyph_index) = line.content_end;
                    }
                    glyphs
                })
                .collect();
            GlyphLine::compute_line_spacing(
                index == 0,
                &mut lines,
                &paragraph.runs,
                paragraph_width,
                align,
            );
            for (line, metrics) in lines.iter_mut().zip(layout) {
                let alignment_width = if metrics.soft || policy != CssWrapPolicy::PreWrap {
                    metrics.content_width
                } else {
                    metrics
                        .content_width
                        .max(metrics.full_width.min(paragraph_width))
                };
                let remaining = (paragraph_width - alignment_width).max(0.0);
                line.start_x = match align {
                    TextAlign::Center => remaining / 2.0,
                    TextAlign::Right => remaining,
                    _ => 0.0,
                };
            }
            lines
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixed advances isolate line breaking and alignment from font rasterization.
    fn lines(text: &str, width: f32, policy: CssWrapPolicy) -> Vec<GlyphLine> {
        let source: Vec<_> = text.chars().map(u32::from).collect();
        let mut run = GlyphRun::new(source.len());
        // Line spacing requires real font metrics even though horizontal
        // advances are fixed here. Use the runtime-owned pinned fixture.
        run.font = super::super::font_hb::HbFont::decode(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/fonts/Inter-Regular.ttf"
        )));
        run.text_indices = (0..source.len() as u32).collect();
        run.advances.fill(10.0);
        run.xpos = (0..=source.len()).map(|index| index as f32 * 10.0).collect();
        let mut paragraphs = [Paragraph { runs: vec![run], level: 0 }];
        break_lines(&mut paragraphs, &source, width, TextAlign::Center, TextWrap::Wrap, policy)
            .remove(0)
    }

    #[test]
    fn normal_retains_unicode_space_in_alignment_but_pre_line_hangs_it() {
        let normal = lines("a\u{2003}", 100.0, CssWrapPolicy::Normal);
        let pre_line = lines("a\u{2003}", 100.0, CssWrapPolicy::PreLine);
        assert_eq!(normal.len(), 1);
        assert_eq!(pre_line.len(), 1);
        assert_eq!(normal[0].start_x, 40.0);
        assert_eq!(pre_line[0].start_x, 45.0);
    }

    #[test]
    fn normal_retains_unicode_space_during_fitting_not_only_alignment() {
        let normal = lines("a b\u{2003}ccc", 35.0, CssWrapPolicy::Normal);
        let pre_line = lines("a b\u{2003}ccc", 35.0, CssWrapPolicy::PreLine);
        assert_eq!(normal.len(), 3);
        assert_eq!(pre_line.len(), 2);
        assert_eq!(normal[0].end_glyph_index, 1);
        assert_eq!(normal[1].end_glyph_index, 4);
    }

    #[test]
    fn normal_does_not_emergency_split_an_overlong_word() {
        let normal = lines("abcdef", 25.0, CssWrapPolicy::Normal);
        assert_eq!(normal.len(), 1);
        assert_eq!(normal[0].end_glyph_index, 6);
        assert_eq!(normal[0].start_x, 0.0);
    }
}
