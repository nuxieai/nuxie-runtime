use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use harfrust::{
    Direction, Feature as ShapingFeature, FontRef as ShapingFont, ShapeOptions, ShaperData,
    ShaperInstance, Tag, UnicodeBuffer,
};
use skrifa::instance::{Location, LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::setting::VariationSetting;
use skrifa::{FontRef as OutlineFont, GlyphId as OutlineGlyphId, MetadataProvider};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
use unicode_script::UnicodeScript;

use crate::mechanical_port::source::math::{mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D};
use crate::mechanical_port::source::shapes::paint::color::ColorInt;
use crate::mechanical_port::source::text_engine::{
    Axis, ColorGlyphLayer, ColorGlyphPaintType, Coord, Feature, Font, FontBase, FontRef, GlyphId,
    GlyphRun, GradientStop, LineMetrics, Paragraph, TextRun, Unichar, fallback_proc,
    fallback_proc_enabled,
};

const STANDARD_SCALE: i32 = 2048;
const INVERSE_SCALE: f32 = 1.0 / STANDARD_SCALE as f32;

/// The HBFont owner retains Rive's run/fallback/color semantics. Only its
/// backend is adapted: harfrust shapes, skrifa reads outlines and color tables,
/// and unicode-bidi supplies paragraph levels. No C font or bidi pointer lives
/// in this owner, and no legacy packed text implementation is consulted.
pub struct HbFont {
    base: FontBase,
    bytes: Arc<[u8]>,
    face_index: u32,
    shaper_data: ShaperData,
    shaper_instance: ShaperInstance,
    features: Vec<ShapingFeature>,
    feature_values: HashMap<u32, u32>,
    axis_values: HashMap<u32, f32>,
    has_color_layers: bool,
    has_color_paint: bool,
    has_png: bool,
    palette_colors: Vec<skrifa::color::Color>,
    color_layer_cache: RefCell<HashMap<GlyphId, Vec<ColorGlyphLayer>>>,
}

impl HbFont {
    pub fn source_bytes(&self) -> Arc<[u8]> {
        self.bytes.clone()
    }
    pub fn face_index(&self) -> u32 {
        self.face_index
    }
    pub fn decode(bytes: &[u8]) -> Option<FontRef> {
        Self::decode_face(bytes, 0)
    }

    pub fn decode_face(bytes: &[u8], face_index: u32) -> Option<FontRef> {
        let shaping = ShapingFont::from_index(bytes, face_index).ok()?;
        let outline = OutlineFont::from_index(bytes, face_index).ok()?;
        let _ = (shaping, outline);
        Some(Rc::new(Self::with_stored_options(
            Arc::from(bytes),
            face_index,
            HashMap::new(),
            HashMap::new(),
        )))
    }

    /// Reconstruct a server-local font from the public Send-safe byte payload.
    /// RawTextFont is a host DTO here, never a shaping or fallback owner.
    pub fn from_raw_text(font: &crate::text::RawTextFont) -> Option<FontRef> {
        let decoded = Self::decode_face(font.source_bytes().as_ref(), font.face_index())?;
        let coords: Vec<_> = (0..font.axis_count())
            .map(|index| {
                let axis = font.axis(index);
                Coord {
                    axis: axis.tag,
                    value: font.axis_value(axis.tag),
                }
            })
            .collect();
        let features: Vec<_> = font
            .features()
            .into_iter()
            .filter_map(|tag| {
                let value = font.feature_value(tag);
                (value != u32::MAX).then_some(Feature { tag, value })
            })
            .collect();
        Some(decoded.with_options(&coords, &features))
    }

    /// System fonts cross the approved Rust host boundary as owned bytes plus
    /// a face index; the runtime does not take a platform font pointer.
    pub fn from_system_bytes(bytes: &[u8], face_index: u32) -> Option<FontRef> {
        Self::decode_face(bytes, face_index)
    }

    fn outline_font(&self) -> OutlineFont<'_> {
        OutlineFont::from_index(&self.bytes, self.face_index)
            .expect("HBFont retains the face validated at decode")
    }

    fn location(&self, font: &OutlineFont<'_>) -> Location {
        font.axes().location(
            self.axis_values
                .iter()
                .map(|(&tag, &value)| VariationSetting::new(skrifa::Tag::from_u32(tag), value)),
        )
    }

    fn with_stored_options(
        bytes: Arc<[u8]>,
        face_index: u32,
        axis_values: HashMap<u32, f32>,
        feature_values: HashMap<u32, u32>,
    ) -> Self {
        let shaping = ShapingFont::from_index(&bytes, face_index).expect("validated shaping face");
        let shaper_data = ShaperData::new(&shaping);
        let shaper_instance = ShaperInstance::from_variations(
            &shaping,
            axis_values
                .iter()
                .map(|(&tag, &value)| harfrust::Variation {
                    tag: Tag::from_u32(tag),
                    value,
                }),
        );
        let outline = OutlineFont::from_index(&bytes, face_index).expect("validated outline face");
        let location = outline.axes().location(
            axis_values
                .iter()
                .map(|(&tag, &value)| VariationSetting::new(skrifa::Tag::from_u32(tag), value)),
        );
        let line_metrics = make_line_metrics(&outline, LocationRef::from(&location));
        let has_color_layers = outline.colr().is_ok();
        let has_color_paint = outline.colr().is_ok_and(|colr| colr.version() >= 1);
        let has_png = outline.sbix().is_ok() || outline.cbdt().is_ok();
        let palette_colors = if has_color_layers || has_color_paint {
            outline
                .color_palettes()
                .get(0)
                .map(|palette| palette.colors().to_vec())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let features = feature_values
            .iter()
            .map(|(&tag, &value)| ShapingFeature::new(Tag::from_u32(tag), value, ..))
            .collect();
        Self {
            base: FontBase::new(line_metrics),
            bytes,
            face_index,
            shaper_data,
            shaper_instance,
            features,
            feature_values,
            axis_values,
            has_color_layers,
            has_color_paint,
            has_png,
            palette_colors,
            color_layer_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn shape_fallback_run(
        &self,
        glyph_runs: &mut Vec<GlyphRun>,
        text: &[Unichar],
        text_start: u32,
        text_run: &TextRun,
        original_text_run: &TextRun,
        fallback_index: u32,
    ) {
        let glyph_run = shape_run(&text[text_start as usize..], text_run, text_start);
        if let Some(index) = glyph_run.glyphs.iter().position(|glyph| *glyph == 0) {
            let missing = text[glyph_run.text_indices[index] as usize];
            let fallback =
                fallback_proc().and_then(|callback| callback(missing, fallback_index, self));
            if let Some(fallback_font) = fallback {
                let fallback_hb = fallback_font
                    .as_any()
                    .downcast_ref::<HbFont>()
                    .expect("fallback font must be an HBFont");
                if !std::ptr::eq(fallback_hb, self) {
                    perform_fallback(
                        fallback_font,
                        glyph_runs,
                        text,
                        &glyph_run,
                        original_text_run,
                        fallback_index + 1,
                    );
                } else if !glyph_run.glyphs.is_empty() {
                    glyph_runs.push(glyph_run);
                }
            } else if !glyph_run.glyphs.is_empty() {
                glyph_runs.push(glyph_run);
            }
        } else if !glyph_run.glyphs.is_empty() {
            glyph_runs.push(glyph_run);
        }
    }
}

impl Font for HbFont {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn base(&self) -> &FontBase {
        &self.base
    }
    fn get_axis(&self, index: u16) -> Axis {
        let font = self.outline_font();
        let axis = font
            .axes()
            .get(index as usize)
            .expect("font axis index in range");
        Axis {
            tag: u32::from_be_bytes(axis.tag().to_be_bytes()),
            min: axis.min_value(),
            def: axis.default_value(),
            max: axis.max_value(),
        }
    }
    fn get_axis_count(&self) -> u16 {
        self.outline_font().axes().len() as u16
    }
    fn get_axis_value(&self, axis_tag: u32) -> f32 {
        self.axis_values.get(&axis_tag).copied().unwrap_or_else(|| {
            self.outline_font()
                .axes()
                .get_by_tag(skrifa::Tag::from_u32(axis_tag))
                .map_or(0.0, |axis| axis.default_value())
        })
    }
    fn get_feature_value(&self, feature_tag: u32) -> u32 {
        self.feature_values
            .get(&feature_tag)
            .copied()
            .unwrap_or(u32::MAX)
    }
    fn get_weight(&self) -> u16 {
        let font = self.outline_font();
        let tag = u32::from_be_bytes(*b"wght");
        if font.axes().get_by_tag(skrifa::Tag::from_u32(tag)).is_some() {
            self.get_axis_value(tag) as u16
        } else {
            font.attributes().weight.value() as u16
        }
    }
    fn is_italic(&self) -> bool {
        let font = self.outline_font();
        let tag = u32::from_be_bytes(*b"ital");
        if font.axes().get_by_tag(skrifa::Tag::from_u32(tag)).is_some() {
            self.get_axis_value(tag) != 0.0
        } else {
            !matches!(font.attributes().style, skrifa::attribute::Style::Normal)
        }
    }
    fn features(&self) -> Vec<u32> {
        let font = self.outline_font();
        let mut features = HashSet::new();
        if let Ok(table) = font.gsub() {
            if let (Ok(scripts), Ok(list)) = (table.script_list(), table.feature_list()) {
                fill_features(scripts, list, &mut features);
            }
        }
        if let Ok(table) = font.gpos() {
            if let (Ok(scripts), Ok(list)) = (table.script_list(), table.feature_list()) {
                fill_features(scripts, list, &mut features);
            }
        }
        features.into_iter().collect()
    }
    fn has_glyph(&self, value: Unichar) -> bool {
        self.outline_font()
            .charmap()
            .map(value)
            .is_some_and(|glyph| glyph.to_u32() != 0)
    }
    fn with_options(&self, coords: &[Coord], features: &[Feature]) -> FontRef {
        let mut axes = self.axis_values.clone();
        for coord in coords {
            axes.insert(coord.axis, coord.value);
        }
        let mut values = self.feature_values.clone();
        for feature in features {
            values.insert(feature.tag, feature.value);
        }
        Rc::new(Self::with_stored_options(
            Arc::clone(&self.bytes),
            self.face_index,
            axes,
            values,
        ))
    }
    fn get_path(&self, glyph: GlyphId) -> RawPath {
        let font = self.outline_font();
        let location = self.location(&font);
        glyph_path(&font, LocationRef::from(&location), glyph)
    }
    fn has_color_glyphs(&self) -> bool {
        self.has_color_layers || self.has_color_paint || self.has_png
    }
    fn is_color_glyph(&self, glyph: GlyphId) -> bool {
        if !self.has_color_glyphs() {
            return false;
        }
        let font = self.outline_font();
        let id = OutlineGlyphId::new(glyph as u32);
        font.color_glyphs().get(id).is_some() || font.bitmap_strikes()
            .glyph_for_size(Size::unscaled(), id)
            .is_some_and(|bitmap| matches!(bitmap.data, skrifa::bitmap::BitmapData::Png(bytes) if !bytes.is_empty()))
    }
    fn get_color_layers(
        &self,
        glyph: GlyphId,
        out: &mut Vec<ColorGlyphLayer>,
        foreground: ColorInt,
    ) -> usize {
        if !self.has_color_glyphs() {
            return 0;
        }
        if let Some(cached) = self.color_layer_cache.borrow().get(&glyph).cloned() {
            let count = cached.len();
            out.extend(cached.into_iter().map(|mut layer| {
                if layer.use_foreground {
                    layer.color = foreground;
                }
                layer
            }));
            return count;
        }
        let font = self.outline_font();
        let location = self.location(&font);
        let id = OutlineGlyphId::new(glyph as u32);
        let mut layers = Vec::new();
        // Pinned order is COLRv0 layers, then COLRv1/PNG only if no v0 layers.
        if self.has_color_layers {
            if let Some(color) = font
                .color_glyphs()
                .get_with_format(id, skrifa::color::ColorGlyphFormat::ColrV0)
            {
                let mut collector = PaintState::new(self, &mut layers, foreground);
                let _ = color.paint(LocationRef::from(&location), &mut collector);
            }
        }
        if layers.is_empty() && self.has_color_paint {
            if let Some(color) = font
                .color_glyphs()
                .get_with_format(id, skrifa::color::ColorGlyphFormat::ColrV1)
            {
                let mut collector = PaintState::new(self, &mut layers, foreground);
                let _ = color.paint(LocationRef::from(&location), &mut collector);
            }
        }
        if layers.is_empty() && self.has_png {
            if let Some(bitmap) = font.bitmap_strikes().glyph_for_size(Size::unscaled(), id) {
                if let skrifa::bitmap::BitmapData::Png(bytes) = bitmap.data {
                    if !bytes.is_empty() {
                        let units = font
                            .metrics(Size::unscaled(), LocationRef::from(&location))
                            .units_per_em as f32;
                        let mut layer = ColorGlyphLayer::default();
                        layer.paint_type = ColorGlyphPaintType::Image;
                        layer.image_bytes = bytes.to_vec();
                        layer.image_width = bitmap.width;
                        layer.image_height = bitmap.height;
                        layer.image_bearing_x =
                            bitmap.bearing_x / units + bitmap.inner_bearing_x / bitmap.ppem_x;
                        layer.image_bearing_y =
                            -bitmap.bearing_y / units - bitmap.inner_bearing_y / bitmap.ppem_y;
                        layer.image_extent_x = bitmap.width as f32 / bitmap.ppem_x;
                        layer.image_extent_y = bitmap.height as f32 / bitmap.ppem_y;
                        layers.push(layer);
                    }
                }
            }
        }
        if layers.is_empty() {
            return 0;
        }
        let count = layers.len();
        self.color_layer_cache
            .borrow_mut()
            .insert(glyph, layers.clone());
        out.extend(layers);
        count
    }
    fn on_shape_text(&self, text: &[Unichar], runs: &[TextRun], direction: i32) -> Vec<Paragraph> {
        self.on_shape_text_native(text, runs, direction)
    }
}

fn make_line_metrics(font: &OutlineFont<'_>, location: LocationRef<'_>) -> LineMetrics {
    let metrics = font.metrics(Size::new(STANDARD_SCALE as f32), location);
    let ascent = -metrics.ascent.round() * INVERSE_SCALE;
    let glyph_scale = STANDARD_SCALE as f32 / metrics.units_per_em as f32;
    let top = |value| {
        font.charmap()
            .map(value)
            .and_then(|glyph| font.glyph_metrics(Size::unscaled(), location).bounds(glyph))
            .map(|bounds| {
                // The pinned glyf extents round the varied font-unit bound,
                // then hb_font_t::scale_glyph_extents rounds at the 2048 scale.
                -(bounds.y_max.round() * glyph_scale).round() * INVERSE_SCALE
            })
            .unwrap_or(ascent)
    };
    LineMetrics {
        ascent,
        descent: -metrics.descent.round() * INVERSE_SCALE,
        cap_height: top('H'),
        x_height: top('x'),
    }
}

fn fill_features(
    scripts: skrifa::raw::tables::layout::ScriptList<'_>,
    features: skrifa::raw::tables::layout::FeatureList<'_>,
    out: &mut HashSet<u32>,
) {
    let mut language_features = |language: skrifa::raw::tables::layout::LangSys<'_>| {
        // hb_ot_layout_language_get_feature_tags enumerates feature_indices,
        // not the separately exposed required_feature_index.
        for index in language.feature_indices() {
            if let Ok(record) = features.get(index.get()) {
                out.insert(u32::from_be_bytes(record.tag.to_be_bytes()));
            }
        }
    };
    for index in 0..scripts.script_count() {
        let Ok(script) = scripts.get(index) else {
            continue;
        };
        if script.lang_sys_count() == 0 {
            if let Some(Ok(language)) = script.default_lang_sys() {
                language_features(language);
            }
        } else {
            for index in 0..script.lang_sys_count() {
                if let Ok(language) = script.lang_sys(index) {
                    language_features(language.element);
                }
            }
        }
    }
}

fn unicode_script(point: u32) -> u32 {
    char::from_u32(point)
        .unwrap_or(char::REPLACEMENT_CHARACTER)
        .script()
        .as_iso15924_tag()
}

fn shape_run(text: &[Unichar], text_run: &TextRun, text_offset: u32) -> GlyphRun {
    let font = text_run
        .font
        .as_ref()
        .expect("text run font")
        .as_any()
        .downcast_ref::<HbFont>()
        .expect("text font must be an HBFont");
    let face =
        ShapingFont::from_index(&font.bytes, font.face_index).expect("validated shaping face");
    let shaper = font
        .shaper_data
        .shaper(&face)
        .instance(Some(&font.shaper_instance))
        .build();
    let mut buffer = UnicodeBuffer::new();
    for (index, &point) in text[..text_run.unichar_count as usize].iter().enumerate() {
        buffer.add(
            char::from_u32(point).unwrap_or(char::REPLACEMENT_CHARACTER),
            index as u32,
        );
    }
    buffer.set_direction(if text_run.level & 1 != 0 {
        Direction::RightToLeft
    } else {
        Direction::LeftToRight
    });
    buffer.set_script(
        harfrust::Script::from_iso15924_tag(Tag::from_u32(text_run.script))
            .unwrap_or(harfrust::script::UNKNOWN),
    );
    buffer.guess_segment_properties();
    let shaped = shaper.shape(
        buffer,
        ShapeOptions::new()
            .scale(Some(STANDARD_SCALE))
            .features(&font.features),
    );
    let infos = shaped.glyph_infos();
    let positions = shaped.glyph_positions();
    let count = infos.len();
    let mut run = GlyphRun::new(count);
    run.font = text_run.font.clone();
    run.size = text_run.size;
    run.line_height = text_run.line_height;
    run.letter_spacing = text_run.letter_spacing;
    run.style_id = text_run.style_id;
    run.level = text_run.level;
    let scale = text_run.size / STANDARD_SCALE as f32;
    for index in 0..count {
        let source = if text_run.level & 1 != 0 {
            count - 1 - index
        } else {
            index
        };
        run.glyphs[index] = infos[source].glyph_id as GlyphId;
        run.text_indices[index] = text_offset + infos[source].cluster;
        let advance = positions[source].x_advance as f32 * scale + text_run.letter_spacing;
        run.advances[index] = advance;
        run.xpos[index] = advance;
        run.offsets[index] = Vec2D::new(
            positions[source].x_offset as f32 * scale,
            -positions[source].y_offset as f32 * scale,
        );
    }
    run.xpos[count] = 0.0;
    run
}

fn glyph_path(font: &OutlineFont<'_>, location: LocationRef<'_>, glyph: GlyphId) -> RawPath {
    let mut path = RawPath::default();
    if let Some(outline) = font.outline_glyphs().get(OutlineGlyphId::new(glyph as u32)) {
        let settings = DrawSettings::unhinted(Size::new(STANDARD_SCALE as f32), location)
            .with_path_style(skrifa::outline::pen::PathStyle::FreeType);
        let _ = outline.draw(settings, &mut RawPathPen(&mut path));
    }
    path
}
struct RawPathPen<'a>(&'a mut RawPath);
impl OutlinePen for RawPathPen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x * INVERSE_SCALE, -y * INVERSE_SCALE);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x * INVERSE_SCALE, -y * INVERSE_SCALE);
    }
    fn quad_to(&mut self, x: f32, y: f32, end_x: f32, end_y: f32) {
        self.0.quad_to_cubic(
            x * INVERSE_SCALE,
            -y * INVERSE_SCALE,
            end_x * INVERSE_SCALE,
            -end_y * INVERSE_SCALE,
        );
    }
    fn curve_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.0.cubic_to(
            x0 * INVERSE_SCALE,
            -y0 * INVERSE_SCALE,
            x1 * INVERSE_SCALE,
            -y1 * INVERSE_SCALE,
            x2 * INVERSE_SCALE,
            -y2 * INVERSE_SCALE,
        );
    }
    fn close(&mut self) {
        self.0.close();
    }
}

impl HbFont {
    fn on_shape_text_native(
        &self,
        text: &[Unichar],
        text_runs: &[TextRun],
        text_direction_flag: i32,
    ) -> Vec<Paragraph> {
        let mut paragraphs = Vec::new();
        let mut text_index = 0usize;
        let mut run_index = 0usize;
        let mut run_start_text_index = 0usize;
        let mut unichar_index = 0usize;
        let mut run_text_index = 0u32;
        let default_level = match text_direction_flag {
            0 => Some(unicode_bidi::Level::ltr()),
            1 => Some(unicode_bidi::Level::rtl()),
            _ => None,
        };
        // Bidi's UTF-8 offsets never enter Rive's UTF-32 index domain.
        let mut utf8 = String::new();
        let mut byte_offsets = Vec::with_capacity(text.len() + 1);
        for &point in text {
            byte_offsets.push(utf8.len());
            utf8.push(char::from_u32(point).unwrap_or(char::REPLACEMENT_CHARACTER));
        }
        byte_offsets.push(utf8.len());
        let bidi = unicode_bidi::BidiInfo::new(&utf8, default_level);
        for paragraph in &bidi.paragraphs {
            let start = byte_offsets
                .binary_search(&paragraph.range.start)
                .expect("paragraph scalar boundary");
            let end = byte_offsets
                .binary_search(&paragraph.range.end)
                .expect("paragraph scalar boundary");
            let paragraph_length = end - start;
            let bidi_levels: Vec<u8> = byte_offsets[start..end]
                .iter()
                .map(|&offset| bidi.levels[offset].number())
                .collect();
            let paragraph_level = paragraph.level.number();
            let mut paragraph_text_index = 0usize;
            let mut bidi_runs = Vec::with_capacity(text_runs.len());

            while run_index < text_runs.len() {
                let text_run = &text_runs[run_index];
                assert_ne!(text_run.unichar_count, 0);
                let mut last_level = bidi_levels[paragraph_text_index];
                let point = text[text_index];
                let mut last_script = unicode_script(point);
                let split_run = TextRun {
                    font: text_run.font.clone(),
                    size: text_run.size,
                    line_height: text_run.line_height,
                    letter_spacing: text_run.letter_spacing,
                    unichar_count: text_run.unichar_count - run_text_index,
                    script: last_script as u32,
                    style_id: text_run.style_id,
                    level: last_level,
                };
                run_start_text_index = text_index;
                run_text_index += 1;
                text_index += 1;
                paragraph_text_index += 1;
                bidi_runs.push(split_run);

                while run_text_index < text_run.unichar_count
                    && paragraph_text_index < paragraph_length
                {
                    let point = text[text_index];
                    let mut script = if char::from_u32(point).is_some_and(|point| {
                        point.general_category() == GeneralCategory::NonspacingMark
                    }) {
                        u32::from_be_bytes(*b"Zinh")
                    } else {
                        unicode_script(point)
                    };
                    if script == u32::from_be_bytes(*b"Zyyy")
                        || script == u32::from_be_bytes(*b"Zinh")
                    {
                        script = last_script;
                    }
                    if bidi_levels[paragraph_text_index] != last_level || script != last_script {
                        last_script = script;
                        let back = bidi_runs.last_mut().unwrap();
                        back.unichar_count = (text_index - run_start_text_index) as u32;
                        last_level = bidi_levels[paragraph_text_index];
                        bidi_runs.push(TextRun {
                            font: back.font.clone(),
                            size: back.size,
                            line_height: back.line_height,
                            letter_spacing: back.letter_spacing,
                            unichar_count: text_run.unichar_count - run_text_index,
                            script: script as u32,
                            style_id: back.style_id,
                            level: last_level,
                        });
                        run_start_text_index = text_index;
                    }
                    run_text_index += 1;
                    text_index += 1;
                    paragraph_text_index += 1;
                }
                if run_text_index == text_run.unichar_count {
                    run_index += 1;
                    run_text_index = 0;
                }
                if paragraph_text_index == paragraph_length {
                    bidi_runs.last_mut().unwrap().unichar_count =
                        (text_index - run_start_text_index) as u32;
                    break;
                }
            }

            let mut glyph_runs = Vec::with_capacity(bidi_runs.len());
            for text_run in &bidi_runs {
                let mut glyph_run =
                    shape_run(&text[unichar_index..], text_run, unichar_index as u32);
                unichar_index += text_run.unichar_count as usize;

                if fallback_proc().is_some() && fallback_proc_enabled() && !self.has_color_glyphs()
                {
                    for glyph_index in 0..glyph_run.glyphs.len() {
                        if glyph_run.glyphs[glyph_index] != 0 {
                            let text_position = glyph_run.text_indices[glyph_index] as usize;
                            if text_position + 1 < text.len() && text[text_position + 1] == 0xfe0f {
                                let emoji_font = fallback_proc()
                                    .and_then(|callback| callback(text[text_position], 0, self));
                                if emoji_font
                                    .as_ref()
                                    .is_some_and(|font| font.has_color_glyphs())
                                {
                                    glyph_run.glyphs[glyph_index] = 0;
                                }
                            }
                        }
                    }
                }

                let missing_index = glyph_run.glyphs.iter().position(|glyph| *glyph == 0);
                if fallback_proc().is_none() || missing_index.is_none() || !fallback_proc_enabled()
                {
                    if !glyph_run.glyphs.is_empty() {
                        glyph_runs.push(glyph_run);
                    }
                } else {
                    let index = missing_index.unwrap();
                    let missing = text[glyph_run.text_indices[index] as usize];
                    let fallback = fallback_proc().and_then(|callback| callback(missing, 0, self));
                    if let Some(fallback) = fallback {
                        perform_fallback(fallback, &mut glyph_runs, text, &glyph_run, text_run, 1);
                    } else if !glyph_run.glyphs.is_empty() {
                        glyph_runs.push(glyph_run);
                    }
                }
            }

            let mut position = 0.0;
            for glyph_run in &mut glyph_runs {
                for x_position in &mut glyph_run.xpos {
                    let advance = *x_position;
                    *x_position = position;
                    position += advance;
                }
            }
            paragraphs.push(Paragraph {
                runs: glyph_runs,
                level: paragraph_level,
            });
        }
        paragraphs
    }
}

fn extract_subset(original: &GlyphRun, start: usize, end: usize) -> GlyphRun {
    let mut subset = GlyphRun::from_arrays(
        original.glyphs[start..end].to_vec(),
        original.text_indices[start..end].to_vec(),
        original.advances[start..end].to_vec(),
        original.xpos[start..=end].to_vec(),
        original.offsets[start..end].to_vec(),
    );
    subset.font = original.font.clone();
    subset.size = original.size;
    subset.line_height = original.line_height;
    subset.letter_spacing = original.letter_spacing;
    subset.level = original.level;
    *subset.xpos.last_mut().unwrap() = 0.0;
    subset.style_id = original.style_id;
    subset
}

fn perform_fallback(
    fallback_font: FontRef,
    glyph_runs: &mut Vec<GlyphRun>,
    text: &[Unichar],
    original: &GlyphRun,
    original_text_run: &TextRun,
    fallback_index: u32,
) {
    assert!(!original.glyphs.is_empty());
    let count = original.glyphs.len();
    let mut start = 0;
    while start < count {
        let mut end = start + 1;
        if original.glyphs[start] == 0 {
            while end < count && original.glyphs[end] == 0 {
                end += 1;
            }
            let text_start = original.text_indices[start];
            let text_count = if end == count {
                original_text_run.unichar_count
                    - (original.text_indices[start] - original.text_indices[0])
            } else {
                original.text_indices[end] - text_start
            };
            let text_run = TextRun {
                font: Some(fallback_font.clone()),
                size: original.size,
                line_height: original.line_height,
                letter_spacing: original_text_run.letter_spacing,
                unichar_count: text_count,
                script: original_text_run.script,
                style_id: original.style_id,
                level: original.level,
            };
            fallback_font
                .as_any()
                .downcast_ref::<HbFont>()
                .expect("fallback font must be an HBFont")
                .shape_fallback_run(
                    glyph_runs,
                    text,
                    text_start,
                    &text_run,
                    original_text_run,
                    fallback_index,
                );
        } else {
            while end < count && original.glyphs[end] != 0 {
                end += 1;
            }
            glyph_runs.push(extract_subset(original, start, end));
        }
        start = end;
    }
}

struct PaintState<'a> {
    font: &'a HbFont,
    layers: &'a mut Vec<ColorGlyphLayer>,
    clip_glyph: GlyphId,
    has_clip: bool,
    foreground: ColorInt,
    transform_stack: Vec<Mat2D>,
}

impl<'a> PaintState<'a> {
    fn new(font: &'a HbFont, layers: &'a mut Vec<ColorGlyphLayer>, foreground: ColorInt) -> Self {
        Self {
            font,
            layers,
            clip_glyph: 0,
            has_clip: false,
            foreground,
            transform_stack: Vec::new(),
        }
    }
    fn push_transform(&mut self, xx: f32, yx: f32, xy: f32, yy: f32, dx: f32, dy: f32) {
        let matrix = Mat2D::new(xx, yx, xy, yy, dx * INVERSE_SCALE, -dy * INVERSE_SCALE);
        if self.transform_stack.is_empty() {
            self.transform_stack.push(matrix);
        } else {
            self.transform_stack
                .push(*self.transform_stack.last().unwrap() * matrix);
        }
    }

    fn pop_transform(&mut self) {
        if !self.transform_stack.is_empty() {
            self.transform_stack.pop();
        }
    }

    fn map_point(&self, x: f32, y: f32) -> Vec2D {
        let scaled_x = x * INVERSE_SCALE;
        let scaled_y = -y * INVERSE_SCALE;
        if let Some(matrix) = self.transform_stack.last() {
            Vec2D::new(
                matrix[0] * x + matrix[2] * y + matrix[4],
                matrix[1] * x + matrix[3] * y + matrix[5],
            )
        } else {
            Vec2D::new(scaled_x, scaled_y)
        }
    }

    fn map_radius(&self, radius: f32) -> f32 {
        let scaled = radius * INVERSE_SCALE;
        if let Some(matrix) = self.transform_stack.last() {
            let scale_x = (matrix[0] * matrix[0] + matrix[1] * matrix[1]).sqrt();
            radius * scale_x
        } else {
            scaled
        }
    }

    fn color(&self, palette: u16, alpha: f32) -> (ColorInt, bool) {
        if palette == 0xffff {
            return (self.foreground, true);
        }
        let Some(color) = self.font.palette_colors.get(palette as usize) else {
            return (0xff000000, false);
        };
        let alpha = (color.alpha() as f32 * alpha).round().clamp(0.0, 255.0) as u32;
        (
            (alpha << 24)
                | ((color.red() as u32) << 16)
                | ((color.green() as u32) << 8)
                | color.blue() as u32,
            false,
        )
    }
    fn extract_stops(&self, stops: &[skrifa::color::ColorStop]) -> Vec<GradientStop> {
        stops
            .iter()
            .map(|stop| GradientStop {
                offset: stop.offset,
                color: self.color(stop.palette_index, stop.alpha).0,
            })
            .collect()
    }
    fn make_clip_layer(&self) -> ColorGlyphLayer {
        ColorGlyphLayer {
            path: self.font.get_path(self.clip_glyph),
            ..ColorGlyphLayer::default()
        }
    }
    fn font_scale(&self) -> f32 {
        STANDARD_SCALE as f32
            / self
                .font
                .outline_font()
                .metrics(Size::unscaled(), LocationRef::default())
                .units_per_em as f32
    }
}
impl skrifa::color::ColorPainter for PaintState<'_> {
    fn push_transform(&mut self, transform: skrifa::color::Transform) {
        let scale = self.font_scale();
        PaintState::push_transform(
            self,
            transform.xx,
            transform.yx,
            transform.xy,
            transform.yy,
            transform.dx * scale,
            transform.dy * scale,
        );
    }
    fn pop_transform(&mut self) {
        PaintState::pop_transform(self);
    }
    fn push_clip_glyph(&mut self, glyph: OutlineGlyphId) {
        self.clip_glyph = glyph.to_u32() as GlyphId;
        self.has_clip = true;
    }
    fn push_clip_box(&mut self, _: skrifa::raw::types::BoundingBox<f32>) {}
    fn pop_clip(&mut self) {
        self.has_clip = false;
    }
    fn fill(&mut self, brush: skrifa::color::Brush<'_>) {
        if !self.has_clip {
            return;
        }
        let mut layer = self.make_clip_layer();
        // Skrifa's callbacks are font-unit based; normalize them to the
        // standard 2048-unit callback inputs before the pinned Rive mapping.
        let scale = self.font_scale();
        match brush {
            skrifa::color::Brush::Solid {
                palette_index,
                alpha,
            } => {
                let (color, foreground) = self.color(palette_index, alpha);
                layer.color = color;
                layer.use_foreground = foreground;
            }
            skrifa::color::Brush::LinearGradient {
                p0,
                p1,
                color_stops,
                ..
            } => {
                layer.paint_type = ColorGlyphPaintType::LinearGradient;
                layer.stops = self.extract_stops(color_stops);
                let a = self.map_point(p0.x * scale, p0.y * scale);
                let b = self.map_point(p1.x * scale, p1.y * scale);
                layer.x0 = a.x;
                layer.y0 = a.y;
                layer.x1 = b.x;
                layer.y1 = b.y;
            }
            skrifa::color::Brush::RadialGradient {
                c0,
                r0,
                c1,
                r1,
                color_stops,
                ..
            } => {
                layer.paint_type = ColorGlyphPaintType::RadialGradient;
                layer.stops = self.extract_stops(color_stops);
                let a = self.map_point(c0.x * scale, c0.y * scale);
                let b = self.map_point(c1.x * scale, c1.y * scale);
                layer.x0 = a.x;
                layer.y0 = a.y;
                layer.x1 = b.x;
                layer.y1 = b.y;
                layer.r0 = self.map_radius(r0 * scale);
                layer.r1 = self.map_radius(r1 * scale);
            }
            skrifa::color::Brush::SweepGradient {
                c0,
                start_angle,
                end_angle,
                color_stops,
                ..
            } => {
                layer.paint_type = ColorGlyphPaintType::SweepGradient;
                layer.stops = self.extract_stops(color_stops);
                let center = self.map_point(c0.x * scale, c0.y * scale);
                layer.x0 = center.x;
                layer.y0 = center.y;
                layer.start_angle = start_angle.to_radians();
                layer.end_angle = end_angle.to_radians();
            }
        }
        self.layers.push(layer);
    }
    // Pinned HB callbacks deliberately do not implement group compositing.
    fn push_layer(&mut self, _: skrifa::color::CompositeMode) {}
    fn pop_layer(&mut self) {}
}
