#[derive(Debug, Clone)]
struct StyledTextGlyph {
    glyph_id: u32,
    char_index: usize,
    char_len: usize,
    style_index: usize,
    advance: f32,
    offset_x: f32,
    offset_y: f32,
    scale: f32,
    rtl: bool,
    variations: Vec<(u32, f32)>,
}
fn shape_text_glyphs_for_style(
    font_bytes: &[u8],
    style: &StaticTextStyle,
    instance: &ArtboardInstance,
    text: &str,
) -> Result<Vec<TextGlyph>> {
    shape_text_glyphs_for_style_with_variations(
        font_bytes,
        style,
        instance,
        text,
        &BTreeMap::new(),
    )
}
fn shape_text_glyphs_for_style_with_variations(
    font_bytes: &[u8],
    style: &StaticTextStyle,
    instance: &ArtboardInstance,
    text: &str,
    localized: &BTreeMap<u32, f32>,
) -> Result<Vec<TextGlyph>> {
    let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
    let mut harf_variations = style.harf_variations(instance);
    for (tag, value) in localized {
        if let Some(existing) = harf_variations
            .iter_mut()
            .find(|(existing, _)| u32::from_be_bytes(existing.to_be_bytes()) == *tag)
        {
            existing.1 = *value;
        } else {
            harf_variations.push((HarfTag::from_u32(*tag), *value));
        }
    }
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
    let features = style.harf_features(instance);
    Ok(shape_text_glyphs_with_features(
        &shaper,
        text,
        disable_legacy_kern_for_advances(&skrifa_font),
        &features,
    ))
}

/// Direct update for the embedded `TextVariationHelper` dependency owner.
/// C++ refreshes the style's cached variable font on every helper update and
/// never dirties the owning Text from here (`text_variation_helper.cpp:14-17`,
/// `text_style.cpp:98-126`); the reshape request is `TextStyle::onDirty`
/// placing TextShape dirt on both the Text and this helper
/// (`text_style.cpp:27-43`). Rust shapes against live axis values instead of a
/// cached font object, so the refresh only has observable work for that
/// TextShape request. A transform- or opacity-only wave reaching this helper
/// through the artboard dependency must leave the retained Text occurrence
/// alone: escalating it re-runs `Text::buildRenderStyles`, whose
/// `clearRenderStyles` replaces the opacity-bucket render paths that pinned
/// C++ keeps retained across such waves.
pub(crate) fn update_text_variation_helper(
    instance: &mut ArtboardInstance,
    text: crate::components::ComponentHandle,
    dirt: crate::components::ComponentDirt,
) {
    if (dirt & crate::components::ComponentDirt::TEXT_SHAPE).is_empty() {
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
