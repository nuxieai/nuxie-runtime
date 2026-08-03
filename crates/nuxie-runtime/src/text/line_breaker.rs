fn split_static_text_lines(text: &str) -> Vec<StaticTextLine<'_>> {
    let mut lines = Vec::new();
    let mut line_start_byte = 0;
    let mut line_start_char = 0;
    let mut line_index = 0;
    let mut iter = text.char_indices().peekable();

    while let Some((byte_index, ch)) = iter.next() {
        if !matches!(ch, '\n' | '\r' | '\u{2028}') {
            continue;
        }

        lines.push(StaticTextLine {
            text: &text[line_start_byte..byte_index],
            char_start: line_start_char,
            line_index,
            soft_wrap_skipped_start: None,
            terminal_soft_wrap_skipped_end: None,
        });

        let mut next_start_byte = byte_index + ch.len_utf8();
        let mut separator_chars = 1;
        if ch == '\r'
            && let Some((next_byte_index, '\n')) = iter.peek().copied()
        {
            iter.next();
            next_start_byte = next_byte_index + '\n'.len_utf8();
            separator_chars = 2;
        }

        line_start_char += text[line_start_byte..byte_index].chars().count() + separator_chars;
        line_start_byte = next_start_byte;
        line_index += 1;
    }

    // Static Text matches Rive's line-break contract: a trailing separator
    // does not create another paragraph/GlyphLine. Editable RawTextInput adds
    // its own U+200B sentinel; that is a separate future geometry surface.
    if line_start_byte < text.len() || lines.is_empty() {
        lines.push(StaticTextLine {
            text: &text[line_start_byte..],
            char_start: line_start_char,
            line_index,
            soft_wrap_skipped_start: None,
            terminal_soft_wrap_skipped_end: None,
        });
    }
    lines
}
fn static_text_line_iteration(
    overflow: u64,
    sizing: u64,
    vertical_align: u64,
    metrics: StaticTextLineMetrics,
    current_y: f32,
    total_height: f32,
    fixed_height: f32,
) -> StaticTextLineIteration {
    if sizing != TEXT_SIZING_FIXED
        || !matches!(overflow, TEXT_OVERFLOW_HIDDEN | TEXT_OVERFLOW_CLIPPED)
    {
        return StaticTextLineIteration::Draw;
    }

    // Exact `Text::shouldDrawLine` comparisons against the authoritative
    // GlyphLine geometry. Hidden requires the full line; clipped admits a
    // partially intersecting line.
    let line_top = current_y + metrics.top;
    let line_bottom = current_y + metrics.bottom;

    if overflow == TEXT_OVERFLOW_HIDDEN {
        match vertical_align {
            1 if line_top < total_height - fixed_height => StaticTextLineIteration::Skip,
            2 if line_top < total_height / 2.0 - fixed_height / 2.0 => {
                StaticTextLineIteration::Skip
            }
            2 if line_bottom > total_height / 2.0 + fixed_height / 2.0 => {
                StaticTextLineIteration::Stop
            }
            0 if line_bottom > fixed_height => StaticTextLineIteration::Stop,
            _ => StaticTextLineIteration::Draw,
        }
    } else {
        match vertical_align {
            1 if line_bottom < total_height - fixed_height => StaticTextLineIteration::Skip,
            2 if line_bottom < total_height / 2.0 - fixed_height / 2.0 => {
                StaticTextLineIteration::Skip
            }
            2 if line_top > total_height / 2.0 + fixed_height / 2.0 => {
                StaticTextLineIteration::Stop
            }
            0 if line_top > fixed_height => StaticTextLineIteration::Stop,
            _ => StaticTextLineIteration::Draw,
        }
    }
}
fn leading_whitespace_bytes(text: &str) -> usize {
    text.char_indices()
        .find_map(|(index, ch)| (!ch.is_whitespace()).then_some(index))
        .unwrap_or(text.len())
}
fn text_glyph_width(glyphs: &[TextGlyph], scale: f32, letter_spacing: f32) -> f32 {
    glyphs
        .iter()
        .map(|glyph| glyph.advance * scale + letter_spacing)
        .sum()
}

fn materialize_renderer_glyph_run_annotations(
    text: &str,
    glyphs: &mut [TextGlyph],
) -> Result<nuxie_render_api::GlyphRunAnnotations> {
    let characters = text.chars().collect::<Vec<_>>();
    let text_indices = glyphs
        .iter()
        .map(|glyph| {
            u32::try_from(character_index_for_cluster(text, glyph.cluster))
                .context("glyph character index exceeds the renderer annotation format")
        })
        .collect::<Result<Vec<_>>>()?;
    let annotation = nuxie_render_api::annotate_glyph_runs(&characters, &[&text_indices])
        .context("glyph clusters do not address the shaped text")?
        .pop()
        .context("renderer did not return the shaped run annotation")?;

    for glyph in glyphs.iter_mut() {
        glyph.renderer_breaks_before = 0;
        glyph.renderer_breaks_after = 0;
        glyph.renderer_joiners.clear();
    }
    for glyph_index in annotation.breaks.iter().copied() {
        let glyph_index = usize::try_from(glyph_index).unwrap_or(usize::MAX);
        if let Some(glyph) = glyphs.get_mut(glyph_index) {
            glyph.renderer_breaks_before = glyph.renderer_breaks_before.saturating_add(1);
        } else if glyph_index == glyphs.len()
            && let Some(glyph) = glyphs.last_mut()
        {
            glyph.renderer_breaks_after = glyph.renderer_breaks_after.saturating_add(1);
        }
    }
    for joiner in annotation.joiners.iter().copied() {
        let glyph_index = glyphs
            .iter()
            .position(|glyph| {
                u32::try_from(character_index_for_cluster(text, glyph.cluster))
                    .is_ok_and(|index| index >= joiner)
            })
            .or_else(|| (!glyphs.is_empty()).then_some(glyphs.len() - 1));
        if let Some(glyph) = glyph_index.and_then(|index| glyphs.get_mut(index)) {
            glyph.renderer_joiners.push(joiner);
        }
    }
    Ok(annotation)
}

fn materialized_renderer_glyph_run_annotations(
    glyphs: &[TextGlyph],
) -> nuxie_render_api::GlyphRunAnnotations {
    let mut breaks = Vec::new();
    let mut joiners = Vec::new();
    for (glyph_index, glyph) in glyphs.iter().enumerate() {
        let glyph_index = u32::try_from(glyph_index).unwrap_or(u32::MAX);
        breaks.extend(std::iter::repeat_n(
            glyph_index,
            usize::from(glyph.renderer_breaks_before),
        ));
        joiners.extend(glyph.renderer_joiners.iter().copied());
    }
    if let Some(glyph) = glyphs.last() {
        let glyph_count = u32::try_from(glyphs.len()).unwrap_or(u32::MAX);
        breaks.extend(std::iter::repeat_n(
            glyph_count,
            usize::from(glyph.renderer_breaks_after),
        ));
    }
    nuxie_render_api::GlyphRunAnnotations { breaks, joiners }
}

/// Single-run equivalent of pinned `RunIterator::back`/`forward` around
/// `GlyphRun::joiners` (`src/text/line_breaker.cpp:152-309`).
fn glyph_end_avoiding_word_joiner(
    text: &str,
    glyphs: &[TextGlyph],
    glyph_end: usize,
    joiners: &[u32],
) -> usize {
    if glyph_end == 0 || glyph_end >= glyphs.len() {
        return glyph_end;
    }
    let next_text_index =
        u32::try_from(character_index_for_cluster(text, glyphs[glyph_end].cluster))
            .unwrap_or(u32::MAX);
    let adjacent_joiner = joiners.iter().position(|joiner| {
        *joiner == next_text_index || joiner.checked_add(1) == Some(next_text_index)
    });
    let Some(mut first_joiner) = adjacent_joiner else {
        return glyph_end;
    };
    let mut last_joiner = first_joiner;
    while first_joiner > 0
        && joiners[first_joiner - 1].checked_add(1) == Some(joiners[first_joiner])
    {
        first_joiner -= 1;
    }
    while last_joiner + 1 < joiners.len()
        && joiners[last_joiner].checked_add(1) == Some(joiners[last_joiner + 1])
    {
        last_joiner += 1;
    }

    let left_text_index = joiners[first_joiner].saturating_sub(1);
    let left_cluster = glyphs
        .iter()
        .filter_map(|glyph| {
            u32::try_from(character_index_for_cluster(text, glyph.cluster))
                .ok()
                .filter(|index| *index <= left_text_index)
        })
        .max();
    let left_glyph_index = glyphs
        .iter()
        .position(|glyph| {
            u32::try_from(character_index_for_cluster(text, glyph.cluster)).ok() == left_cluster
        })
        .unwrap_or(0);
    if left_glyph_index != 0 {
        return left_glyph_index;
    }

    let right_text_index = joiners[last_joiner].saturating_add(1);
    let Some(right_glyph_index) = glyphs.iter().position(|glyph| {
        u32::try_from(character_index_for_cluster(text, glyph.cluster))
            .is_ok_and(|index| index >= right_text_index)
    }) else {
        return glyphs.len();
    };
    let right_cluster = glyphs[right_glyph_index].cluster;
    right_glyph_index
        + glyphs[right_glyph_index..]
            .iter()
            .take_while(|glyph| glyph.cluster == right_cluster)
            .count()
}
