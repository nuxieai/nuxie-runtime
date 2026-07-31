fn glyph_character_len(text: &str, glyphs: &[TextGlyph], glyph_index: usize) -> usize {
    let char_index = character_index_for_cluster(text, glyphs[glyph_index].cluster);
    let next_char_index = glyphs
        .iter()
        .skip(glyph_index + 1)
        .find_map(|glyph| {
            (glyph.cluster != glyphs[glyph_index].cluster)
                .then_some(character_index_for_cluster(text, glyph.cluster))
        })
        .unwrap_or_else(|| text.chars().count());
    next_char_index.saturating_sub(char_index).max(1)
}
fn glyph_coverage(coverage: &[f32], char_index: usize, char_len: usize) -> f32 {
    let end = (char_index + char_len).min(coverage.len());
    if char_index >= end {
        return 0.0;
    }
    coverage[char_index..end].iter().copied().sum::<f32>() / (end - char_index) as f32
}
