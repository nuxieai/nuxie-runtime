fn font_parser_panic_boundary<F>(parse: F) -> bool
where
    F: FnOnce() -> bool + std::panic::UnwindSafe,
{
    std::panic::catch_unwind(parse).unwrap_or(false)
}
/// Whether embedded font bytes can be parsed by both runtime text backends.
///
/// Dynamic authoring calls this before publishing a structural edit so a
/// committed scene cannot defer malformed-font failure until layout or draw.
#[must_use]
pub fn embedded_font_is_parseable(font_bytes: &[u8]) -> bool {
    if HarfFontRef::new(font_bytes).is_err() {
        return false;
    }

    font_parser_panic_boundary(|| {
        let Ok(font) = SkrifaFontRef::new(font_bytes) else {
            return false;
        };
        let Ok(maxp) = font.maxp() else {
            return false;
        };
        let outlines = font.outline_glyphs();
        for glyph_index in 0..u32::from(maxp.num_glyphs()) {
            let Some(outline) = outlines.get(GlyphId::new(glyph_index)) else {
                continue;
            };
            if outline
                .draw(
                    DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
                    &mut NullPen,
                )
                .is_err()
            {
                return false;
            }
        }
        true
    })
}
/// Whether every in-band `FontAsset` can be safely consumed by the text
/// backends. Hosted font assets without in-band contents remain valid and are
/// checked when their bytes are attached.
#[must_use]
pub fn embedded_fonts_are_parseable(runtime: &RuntimeFile) -> bool {
    runtime
        .file_assets()
        .into_iter()
        .filter(|asset| asset.type_name == "FontAsset")
        .all(|asset| {
            embedded_file_asset_bytes(runtime, asset.id).is_none_or(embedded_font_is_parseable)
        })
}
#[derive(Clone)]
struct TextGlyph {
    glyph_id: u32,
    cluster: u32,
    advance: f32,
    offset_x: f32,
    offset_y: f32,
}
fn harfrust_script_for_unicode_script(script: UnicodeScript) -> HarfScript {
    HarfScript::from_iso15924_tag(HarfTag::from_u32(script.as_iso15924_tag()))
        .unwrap_or(harfrust::script::UNKNOWN)
}
fn harfbuzz_line_metrics(font: &SkrifaFontRef<'_>, location_ref: LocationRef<'_>) -> (f32, f32) {
    // Mirrors src/text/font_hb.cpp::make_lmx: HarfBuzz scales extents to
    // kStdScale and rounds them before Rive applies the authored font size. Its
    // OpenType funcs use the platform line-metric policy: OS/2 typo metrics
    // only when USE_TYPO_METRICS is set, hhea otherwise, with font fallbacks.
    let metrics = font.metrics(Size::new(TEXT_SHAPE_SCALE_F32), location_ref);
    (metrics.ascent.round(), metrics.descent.round())
}
fn harfbuzz_scaled_glyph_top(raw_edge: f32) -> f32 {
    // C++ asks HarfBuzz for integer glyph extents after scaling to 2048.
    // Skrifa returns exact varied bounds as floats, so keep integral bounds
    // stable and snap fractional varied tops to HarfBuzz's lower integer step.
    let rounded = raw_edge.round();
    if (raw_edge - rounded).abs() <= 1e-4 {
        rounded
    } else {
        raw_edge.trunc() - 1.0
    }
}
fn disable_legacy_kern_for_advances(font: &SkrifaFontRef<'_>) -> bool {
    font.kern().is_ok() && font.gpos().is_err()
}
fn shape_text_glyphs(
    shaper: &harfrust::Shaper<'_>,
    text: &str,
    disable_legacy_kern: bool,
) -> Vec<TextGlyph> {
    shape_text_glyphs_with_features(shaper, text, disable_legacy_kern, &[])
}
fn shape_text_glyphs_with_features(
    shaper: &harfrust::Shaper<'_>,
    text: &str,
    disable_legacy_kern: bool,
    features: &[Feature],
) -> Vec<TextGlyph> {
    let mut glyphs = Vec::new();
    for run in cxx_script_runs(text) {
        let cluster_offset = u32::try_from(run.byte_start).unwrap_or(u32::MAX);
        let mut run_glyphs = shape_cxx_script_run_glyphs_with_features(
            shaper,
            run.text,
            run.script,
            disable_legacy_kern,
            features,
        );
        for glyph in &mut run_glyphs {
            glyph.cluster = glyph.cluster.saturating_add(cluster_offset);
        }
        glyphs.extend(run_glyphs);
    }
    glyphs
}
fn shape_bidi_text_glyphs(
    shaper: &harfrust::Shaper<'_>,
    text: &str,
    disable_legacy_kern: bool,
) -> Vec<TextGlyph> {
    shape_bidi_text_glyphs_with_features(shaper, text, disable_legacy_kern, &[])
}
fn shape_bidi_text_glyphs_with_features(
    shaper: &harfrust::Shaper<'_>,
    text: &str,
    disable_legacy_kern: bool,
    features: &[Feature],
) -> Vec<TextGlyph> {
    let bidi = unicode_bidi::BidiInfo::new(text, None);
    let mut glyphs = Vec::new();
    for paragraph in &bidi.paragraphs {
        let (_, runs) = bidi.visual_runs(paragraph, paragraph.range.clone());
        for run in runs {
            let direction = if bidi.levels[run.start].is_rtl() {
                Direction::RightToLeft
            } else {
                Direction::LeftToRight
            };
            for script_run in cxx_script_runs(&text[run.clone()]) {
                let cluster_offset =
                    u32::try_from(run.start + script_run.byte_start).unwrap_or(u32::MAX);
                let mut run_glyphs = shape_cxx_script_run_glyphs_in_direction(
                    shaper,
                    script_run.text,
                    script_run.script,
                    direction,
                    disable_legacy_kern,
                    features,
                );
                for glyph in &mut run_glyphs {
                    glyph.cluster = glyph.cluster.saturating_add(cluster_offset);
                }
                glyphs.extend(run_glyphs);
            }
        }
    }
    // Line breaking and style lookup consume logical order. Equal clusters
    // retain HarfBuzz's within-cluster visual order.
    glyphs.sort_by_key(|glyph| glyph.cluster);
    glyphs
}
fn shape_cxx_script_run_glyphs(
    shaper: &harfrust::Shaper<'_>,
    text: &str,
    script: HarfScript,
    disable_legacy_kern: bool,
) -> Vec<TextGlyph> {
    shape_cxx_script_run_glyphs_with_features(shaper, text, script, disable_legacy_kern, &[])
}
fn shape_cxx_script_run_glyphs_with_features(
    shaper: &harfrust::Shaper<'_>,
    text: &str,
    script: HarfScript,
    disable_legacy_kern: bool,
    features: &[Feature],
) -> Vec<TextGlyph> {
    shape_cxx_script_run_glyphs_in_direction(
        shaper,
        text,
        script,
        Direction::LeftToRight,
        disable_legacy_kern,
        features,
    )
}
fn shape_cxx_script_run_glyphs_in_direction(
    shaper: &harfrust::Shaper<'_>,
    text: &str,
    script: HarfScript,
    direction: Direction,
    disable_legacy_kern: bool,
    features: &[Feature],
) -> Vec<TextGlyph> {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(direction);
    buffer.set_script(script);
    buffer.guess_segment_properties();
    let mut shape_features = features.to_vec();
    if disable_legacy_kern {
        shape_features.push(Feature::new(HarfTag::new(b"kern"), 0, ..));
    }
    let shape_options = ShapeOptions::new().scale(Some(TEXT_SHAPE_SCALE));
    let shape_options = if shape_features.is_empty() {
        shape_options
    } else {
        shape_options.features(&shape_features)
    };
    let glyphs = shaper.shape(buffer, shape_options);
    glyphs
        .glyph_infos()
        .iter()
        .zip(glyphs.glyph_positions())
        .map(|(info, position)| TextGlyph {
            glyph_id: info.glyph_id,
            cluster: info.cluster,
            advance: position.x_advance as f32,
            offset_x: position.x_offset as f32,
            offset_y: -position.y_offset as f32,
        })
        .collect()
}
struct TextOutlinePen {
    commands: Vec<RuntimePathCommand>,
    x: f32,
    y: f32,
    scale: f32,
    center_x: f32,
    center_y: f32,
    transform: Mat2D,
    current: Option<(f32, f32)>,
    contour_start: Option<(f32, f32)>,
    current_outline: Option<(f32, f32)>,
    contour_start_outline: Option<(f32, f32)>,
}
impl TextOutlinePen {
    fn new(x: f32, y: f32, scale: f32, center_x: f32, center_y: f32, transform: Mat2D) -> Self {
        Self {
            commands: Vec::new(),
            x,
            y,
            scale,
            center_x,
            center_y,
            transform,
            current: None,
            contour_start: None,
            current_outline: None,
            contour_start_outline: None,
        }
    }

    fn normalize_outline_point(x: f32, y: f32) -> (f32, f32) {
        let inverse_shape_scale = 1.0 / TEXT_SHAPE_SCALE_F32;
        (x * inverse_shape_scale, -y * inverse_shape_scale)
    }

    fn map_normalized(&self, x: f32, y: f32) -> (f32, f32) {
        let font_size = self.scale * TEXT_SHAPE_SCALE_F32;
        if self.transform == Mat2D::IDENTITY {
            // C++ first records HarfBuzz outlines in em units, then maps the
            // normalized path with the font-size matrix. Preserve its scale-
            // and-translate operation order here.
            let glyph_center = self.center_x - self.x;
            let translation_x = -glyph_center + (self.x + glyph_center);
            return (font_size * x + translation_x, font_size * y + self.y);
        }
        let point = (self.x + x * font_size, self.y + y * font_size);
        let transformed = self
            .transform
            .transform_point(point.0 - self.center_x, point.1 - self.center_y);
        (self.center_x + transformed.0, self.center_y + transformed.1)
    }

    fn map(&self, x: f32, y: f32) -> ((f32, f32), (f32, f32)) {
        let outline = Self::normalize_outline_point(x, y);
        (self.map_normalized(outline.0, outline.1), outline)
    }
}
impl OutlinePen for TextOutlinePen {
    fn move_to(&mut self, x: f32, y: f32) {
        let (point, outline) = self.map(x, y);
        self.commands.push(RuntimePathCommand::Move {
            x: point.0,
            y: point.1,
        });
        self.current = Some(point);
        self.contour_start = Some(point);
        self.current_outline = Some(outline);
        self.contour_start_outline = Some(outline);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        if self.scale == 0.0 {
            // C++ RawPath collapses zero-size glyph contours to move/close pairs.
            return;
        }
        let (point, outline) = self.map(x, y);
        self.commands.push(RuntimePathCommand::Line {
            x: point.0,
            y: point.1,
        });
        self.current = Some(point);
        self.current_outline = Some(outline);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        if self.scale == 0.0 {
            return;
        }
        let Some(current_outline) = self.current_outline else {
            self.move_to(x, y);
            return;
        };
        let control_outline = Self::normalize_outline_point(cx0, cy0);
        let end_outline = Self::normalize_outline_point(x, y);
        // C++ converts HarfBuzz quadratic contours to cubics before applying
        // the glyph matrix. Doing the lerps after mapping is algebraically
        // equivalent but rounds hundreds of text control points differently.
        let t = 2.0 / 3.0;
        let control1_outline = (
            current_outline.0 + (control_outline.0 - current_outline.0) * t,
            current_outline.1 + (control_outline.1 - current_outline.1) * t,
        );
        let control2_outline = (
            end_outline.0 + (control_outline.0 - end_outline.0) * t,
            end_outline.1 + (control_outline.1 - end_outline.1) * t,
        );
        let control1 = self.map_normalized(control1_outline.0, control1_outline.1);
        let control2 = self.map_normalized(control2_outline.0, control2_outline.1);
        let end = self.map_normalized(end_outline.0, end_outline.1);
        self.commands.push(RuntimePathCommand::Cubic {
            x1: control1.0,
            y1: control1.1,
            x2: control2.0,
            y2: control2.1,
            x3: end.0,
            y3: end.1,
        });
        self.current = Some(end);
        self.current_outline = Some(end_outline);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        if self.scale == 0.0 {
            return;
        }
        let (control0, _) = self.map(cx0, cy0);
        let (control1, _) = self.map(cx1, cy1);
        let (end, end_outline) = self.map(x, y);
        self.commands.push(RuntimePathCommand::Cubic {
            x1: control0.0,
            y1: control0.1,
            x2: control1.0,
            y2: control1.1,
            x3: end.0,
            y3: end.1,
        });
        self.current = Some(end);
        self.current_outline = Some(end_outline);
    }

    fn close(&mut self) {
        if let (Some(current), Some(start)) = (self.current, self.contour_start)
            && ((current.0 - start.0).abs() > f32::EPSILON
                || (current.1 - start.1).abs() > f32::EPSILON)
        {
            self.commands.push(RuntimePathCommand::Line {
                x: start.0,
                y: start.1,
            });
        }
        self.commands.push(RuntimePathCommand::Close);
        self.current = self.contour_start;
        self.current_outline = self.contour_start_outline;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CxxScriptRun<'a> {
    text: &'a str,
    byte_start: usize,
    script: HarfScript,
}

fn cxx_script_runs(text: &str) -> Vec<CxxScriptRun<'_>> {
    let Some((_, first_character)) = text.char_indices().next() else {
        return Vec::new();
    };

    // Exact port of the script-boundary half of `HBFont::onShapeText`. The
    // first code point starts a run with its raw Unicode Script value. Common,
    // inherited, and non-spacing characters after that inherit the preceding
    // run's script (`src/text/font_hb.cpp:1158-1230`).
    let mut runs = Vec::with_capacity(1);
    let mut run_byte_start = 0;
    let mut last_script = harfrust_script_for_unicode_script(first_character.script());
    for (byte_index, character) in text.char_indices().skip(1) {
        let unicode_script = if character.general_category() == GeneralCategory::NonspacingMark {
            UnicodeScript::Inherited
        } else {
            character.script()
        };
        let mut script = harfrust_script_for_unicode_script(unicode_script);
        if script == harfrust::script::COMMON || script == harfrust::script::INHERITED {
            script = last_script;
        }
        if script != last_script {
            runs.push(CxxScriptRun {
                text: &text[run_byte_start..byte_index],
                byte_start: run_byte_start,
                script: last_script,
            });
            run_byte_start = byte_index;
            last_script = script;
        }
    }
    runs.push(CxxScriptRun {
        text: &text[run_byte_start..],
        byte_start: run_byte_start,
        script: last_script,
    });
    runs
}
