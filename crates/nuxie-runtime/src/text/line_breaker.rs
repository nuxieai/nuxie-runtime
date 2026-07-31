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
