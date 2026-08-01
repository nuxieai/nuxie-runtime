#[derive(Clone)]
struct StyledTextGlyph {
    glyph_id: u32,
    char_index: usize,
    char_len: usize,
    style_index: usize,
    advance: f32,
    scale: f32,
    rtl: bool,
}
fn shape_text_glyphs_for_style(
    font_bytes: &[u8],
    style: &StaticTextStyle<'_>,
    instance: &ArtboardInstance,
    text: &str,
) -> Result<Vec<TextGlyph>> {
    let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
    let harf_variations = style.harf_variations(instance);
    let shaper_instance = if harf_variations.is_empty() {
        None
    } else {
        Some(ShaperInstance::from_variations(
            &harf_font,
            harf_variations.iter().copied(),
        ))
    };
    let shaper_data = ShaperData::new(&harf_font);
    let shaper = shaper_data
        .shaper(&harf_font)
        .instance(shaper_instance.as_ref())
        .build();
    let skrifa_font = SkrifaFontRef::new(font_bytes).context("failed to parse font for shaping")?;
    Ok(shape_text_glyphs(
        &shaper,
        text,
        disable_legacy_kern_for_advances(&skrifa_font),
    ))
}

/// Direct update for the embedded `TextVariationHelper` dependency owner.
/// Axis dirt rebuilds the variation-bearing font by invalidating precisely the
/// retained Text occurrence (`text_variation_helper.cpp:14-17`).
pub(crate) fn update_text_variation_helper(
    instance: &mut ArtboardInstance,
    text: crate::components::ComponentHandle,
    dirt: crate::components::ComponentDirt,
) {
    if !dirt.contains(crate::components::ComponentDirt::TEXT_SHAPE) {
        return;
    }
    if let Some(text_local) = instance.component_local_id(text) {
        instance.add_dirt(
            text_local,
            crate::components::ComponentDirt::TEXT_SHAPE,
            false,
        );
    }
}
