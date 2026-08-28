use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ptr;
use std::rc::Rc;

use harfbuzz_sys::*;
use sheenbidi_sys::*;

use crate::mechanical_port::source::math::mat2d::Mat2D;
use crate::mechanical_port::source::math::raw_path::RawPath;
use crate::mechanical_port::source::math::vec2d::Vec2D;
use crate::mechanical_port::source::shapes::paint::color::{ColorInt, color_argb};
use crate::mechanical_port::source::text_engine::{
    Axis, ColorGlyphLayer, ColorGlyphPaintType, Coord, Feature, Font, FontBase, FontRef, GlyphId,
    GlyphRun, GradientStop, LineMetrics, Paragraph, TextDirection, TextRun, Unichar, fallback_proc,
    fallback_proc_enabled,
};

const STANDARD_SCALE: i32 = 2048;
const INVERSE_SCALE: f32 = 1.0 / STANDARD_SCALE as f32;

pub struct HbFont {
    base: FontBase,
    pub font: *mut hb_font_t,
    pub features: Vec<hb_feature_t>,
    draw_funcs: *mut hb_draw_funcs_t,
    paint_funcs: *mut hb_paint_funcs_t,
    feature_values: HashMap<u32, u32>,
    axis_values: HashMap<u32, f32>,
    has_color_layers: bool,
    has_color_paint: bool,
    has_png: bool,
    palette_colors: Vec<hb_color_t>,
    color_layer_cache: RefCell<HashMap<GlyphId, Vec<ColorGlyphLayer>>>,
}

impl HbFont {
    pub fn decode(bytes: &[u8]) -> Option<FontRef> {
        Self::decode_face(bytes, 0)
    }

    pub fn decode_face(bytes: &[u8], face_index: u32) -> Option<FontRef> {
        // SAFETY: HarfBuzz duplicates `bytes` before this call returns. Every
        // successfully created blob/face is destroyed after transferring its
        // reference to the next HarfBuzz owner, and the returned font becomes
        // the singular owned handle destroyed by `HbFont::drop`.
        unsafe {
            let blob = hb_blob_create_or_fail(
                bytes.as_ptr().cast(),
                bytes.len() as u32,
                HB_MEMORY_MODE_DUPLICATE,
                ptr::null_mut(),
                None,
            );
            if !blob.is_null() {
                let face = hb_face_create_or_fail(blob, face_index);
                hb_blob_destroy(blob);
                if !face.is_null() {
                    let font = hb_font_create(face);
                    hb_face_destroy(face);
                    if !font.is_null() {
                        return Some(Rc::new(Self::new(font)));
                    }
                }
            }
        }
        None
    }

    /// Rebuild a server-local HarfBuzz font from the Send-safe font payload
    /// used at the command-queue boundary.
    pub fn from_raw_text(font: &crate::text::RawTextFont) -> Option<FontRef> {
        let decoded = Self::decode_face(font.source_bytes().as_ref(), font.face_index())?;
        let variable_axes = (0..font.axis_count())
            .map(|index| {
                let axis = font.axis(index);
                Coord {
                    axis: axis.tag,
                    value: font.axis_value(axis.tag),
                }
            })
            .collect::<Vec<_>>();
        let features = font
            .features()
            .into_iter()
            .filter_map(|tag| {
                let value = font.feature_value(tag);
                (value != u32::MAX).then_some(Feature { tag, value })
            })
            .collect::<Vec<_>>();
        Some(decoded.with_options(&variable_axes, &features))
    }

    #[cfg(not(target_os = "macos"))]
    pub fn from_system(
        _system_font: *mut c_void,
        _use_system_shaper: bool,
        _weight: u16,
        _width: u8,
    ) -> Option<FontRef> {
        None
    }

    pub fn get_style(font: *mut hb_font_t, style_tag: u32) -> f32 {
        // SAFETY: callers supply the live font owned by an `HbFont`; HarfBuzz
        // reads it only for this call and does not retain it.
        unsafe { hb_style_get_value(font, style_tag as hb_style_tag_t) }
    }

    pub fn font(&self) -> *mut hb_font_t {
        self.font
    }

    pub fn new(font: *mut hb_font_t) -> Self {
        Self::with_stored_options(font, HashMap::new(), HashMap::new(), Vec::new())
    }

    fn with_stored_options(
        font: *mut hb_font_t,
        axis_values: HashMap<u32, f32>,
        feature_values: HashMap<u32, u32>,
        features: Vec<hb_feature_t>,
    ) -> Self {
        // SAFETY: `font` is a newly owned/live HarfBuzz reference. Callback
        // function tables are created here, retained only by this `HbFont`,
        // and destroyed exactly once in `Drop`; callback userdata is null or
        // points to same-call stack values supplied by the invoking methods.
        unsafe {
            let draw_funcs = hb_draw_funcs_create();
            hb_draw_funcs_set_move_to_func(
                draw_funcs,
                Some(raw_path_move_to),
                ptr::null_mut(),
                None,
            );
            hb_draw_funcs_set_line_to_func(
                draw_funcs,
                Some(raw_path_line_to),
                ptr::null_mut(),
                None,
            );
            hb_draw_funcs_set_quadratic_to_func(
                draw_funcs,
                Some(raw_path_quadratic_to),
                ptr::null_mut(),
                None,
            );
            hb_draw_funcs_set_cubic_to_func(
                draw_funcs,
                Some(raw_path_cubic_to),
                ptr::null_mut(),
                None,
            );
            hb_draw_funcs_set_close_path_func(
                draw_funcs,
                Some(raw_path_close),
                ptr::null_mut(),
                None,
            );
            hb_draw_funcs_make_immutable(draw_funcs);

            let paint_funcs = hb_paint_funcs_create();
            hb_paint_funcs_set_push_transform_func(
                paint_funcs,
                Some(paint_push_transform),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_pop_transform_func(
                paint_funcs,
                Some(paint_pop_transform),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_push_clip_glyph_func(
                paint_funcs,
                Some(paint_push_clip_glyph),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_push_clip_rectangle_func(
                paint_funcs,
                Some(paint_push_clip_rectangle),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_pop_clip_func(
                paint_funcs,
                Some(paint_pop_clip),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_color_func(paint_funcs, Some(paint_solid), ptr::null_mut(), None);
            hb_paint_funcs_set_push_group_func(
                paint_funcs,
                Some(paint_push_group),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_pop_group_func(
                paint_funcs,
                Some(paint_pop_group),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_color_glyph_func(
                paint_funcs,
                Some(paint_color_glyph),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_linear_gradient_func(
                paint_funcs,
                Some(paint_linear_gradient),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_radial_gradient_func(
                paint_funcs,
                Some(paint_radial_gradient),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_sweep_gradient_func(
                paint_funcs,
                Some(paint_sweep_gradient),
                ptr::null_mut(),
                None,
            );
            hb_paint_funcs_set_image_func(paint_funcs, Some(paint_image), ptr::null_mut(), None);
            hb_paint_funcs_make_immutable(paint_funcs);

            let face = hb_font_get_face(font);
            let has_color_layers = hb_ot_color_has_layers(face) != 0;
            let has_color_paint = hb_ot_color_has_paint(face) != 0;
            let has_png = hb_ot_color_has_png(face) != 0;
            let mut palette_colors = Vec::new();
            if has_color_layers || has_color_paint {
                let mut color_count = 0;
                hb_ot_color_palette_get_colors(face, 0, 0, &mut color_count, ptr::null_mut());
                if color_count > 0 {
                    palette_colors.resize(color_count as usize, 0);
                    hb_ot_color_palette_get_colors(
                        face,
                        0,
                        0,
                        &mut color_count,
                        palette_colors.as_mut_ptr(),
                    );
                }
            }

            Self {
                base: FontBase::new(make_line_metrics(font)),
                font,
                features,
                draw_funcs,
                paint_funcs,
                feature_values,
                axis_values,
                has_color_layers,
                has_color_paint,
                has_png,
                palette_colors,
                color_layer_cache: RefCell::new(HashMap::new()),
            }
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

impl Drop for HbFont {
    fn drop(&mut self) {
        // SAFETY: these are the three owned HarfBuzz references established
        // by `with_stored_options`; no other `HbFont` destroys them.
        unsafe {
            hb_draw_funcs_destroy(self.draw_funcs);
            hb_paint_funcs_destroy(self.paint_funcs);
            hb_font_destroy(self.font);
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
        // SAFETY: `self.font` owns a live face for this whole call; `index` is
        // checked before the single bounded POD out-parameter is read.
        unsafe {
            let face = hb_font_get_face(self.font);
            assert!((index as u32) < hb_ot_var_get_axis_count(face));
            let mut count = 1;
            // SAFETY: HarfBuzz declares this record as C POD; zero is a valid
            // out-parameter seed. `count == 1` bounds the single live slot,
            // and `face` remains owned by `self.font` for this call.
            let mut info = std::mem::zeroed::<hb_ot_var_axis_info_t>();
            hb_ot_var_get_axis_infos(face, index as u32, &mut count, &mut info);
            assert_eq!(count, 1);
            Axis {
                tag: info.tag,
                min: info.min_value,
                def: info.default_value,
                max: info.max_value,
            }
        }
    }

    fn get_axis_count(&self) -> u16 {
        // SAFETY: `self.font` remains live and HarfBuzz retains no returned
        // face pointer beyond this expression.
        unsafe { hb_ot_var_get_axis_count(hb_font_get_face(self.font)) as u16 }
    }

    fn get_axis_value(&self, axis_tag: u32) -> f32 {
        if let Some(value) = self.axis_values.get(&axis_tag) {
            return *value;
        }
        // SAFETY: `self.font` keeps the face live, and every requested axis
        // record uses one bounded POD out slot initialized before inspection.
        unsafe {
            let face = hb_font_get_face(self.font);
            let count = hb_ot_var_get_axis_count(face);
            for index in 0..count {
                // SAFETY: HarfBuzz's axis-info record is C POD and the API
                // initializes one requested slot before it is read. The face
                // pointer remains valid through the owning `self.font`.
                let mut info = std::mem::zeroed::<hb_ot_var_axis_info_t>();
                let mut one = 1;
                hb_ot_var_get_axis_infos(face, index, &mut one, &mut info);
                if info.tag == axis_tag {
                    return info.default_value;
                }
            }
        }
        0.0
    }

    fn get_feature_value(&self, feature_tag: u32) -> u32 {
        self.feature_values
            .get(&feature_tag)
            .copied()
            .unwrap_or(u32::MAX)
    }

    fn get_weight(&self) -> u16 {
        Self::get_style(self.font, tag(b'w', b'g', b'h', b't')) as u16
    }

    fn is_italic(&self) -> bool {
        Self::get_style(self.font, tag(b'i', b't', b'a', b'l')) != 0.0
    }

    fn features(&self) -> Vec<u32> {
        // SAFETY: the face borrowed from the owned font remains live while the
        // helpers perform count-then-fill calls into correctly sized vectors.
        unsafe {
            let mut features = HashSet::new();
            let face = hb_font_get_face(self.font);
            fill_features(face, HB_OT_TAG_GSUB, &mut features);
            fill_features(face, HB_OT_TAG_GPOS, &mut features);
            features.into_iter().collect()
        }
    }

    fn has_glyph(&self, missing: Unichar) -> bool {
        // SAFETY: HarfBuzz receives the owned live font and one same-call
        // scalar out-parameter, which is not read after the call returns false.
        unsafe {
            let mut glyph = 0;
            hb_font_get_nominal_glyph(self.font, missing, &mut glyph) != 0
        }
    }

    fn with_options(&self, coords: &[Coord], features: &[Feature]) -> FontRef {
        // SAFETY: HarfBuzz creates a new owned sub-font reference. Variation
        // slices are live for the call and copied into the sub-font; ownership
        // then transfers to the returned `HbFont`.
        unsafe {
            let mut axis_values = self.axis_values.clone();
            for coord in coords {
                axis_values.insert(coord.axis, coord.value);
            }
            let variations: Vec<hb_variation_t> = axis_values
                .iter()
                .map(|(tag, value)| hb_variation_t {
                    tag: *tag,
                    value: *value,
                })
                .collect();
            let font = hb_font_create_sub_font(self.font);
            hb_font_set_variations(font, variations.as_ptr(), variations.len() as u32);

            let mut feature_values = self.feature_values.clone();
            for feature in features {
                feature_values.insert(feature.tag, feature.value);
            }
            let hb_features = feature_values
                .iter()
                .map(|(tag, value)| hb_feature_t {
                    tag: *tag,
                    value: *value,
                    start: HB_FEATURE_GLOBAL_START,
                    end: HB_FEATURE_GLOBAL_END,
                })
                .collect();
            Rc::new(Self::with_stored_options(
                font,
                axis_values,
                feature_values,
                hb_features,
            ))
        }
    }

    fn get_path(&self, glyph: GlyphId) -> RawPath {
        let mut path = RawPath::default();
        // SAFETY: callback userdata points to this stack-local `RawPath` only
        // for the synchronous draw call; the callback table is owned by self.
        unsafe {
            hb_font_draw_glyph(
                self.font,
                glyph as u32,
                self.draw_funcs,
                (&mut path as *mut RawPath).cast(),
            );
        }
        path
    }

    fn has_color_glyphs(&self) -> bool {
        self.has_color_layers || self.has_color_paint || self.has_png
    }

    fn is_color_glyph(&self, glyph: GlyphId) -> bool {
        if !self.has_color_layers && !self.has_color_paint && !self.has_png {
            return false;
        }
        // SAFETY: every HarfBuzz handle is borrowed from `self.font` for this
        // call. Any returned PNG blob is destroyed exactly once after its
        // length is inspected.
        unsafe {
            let face = hb_font_get_face(self.font);
            if self.has_color_layers
                && hb_ot_color_glyph_get_layers(
                    face,
                    glyph as u32,
                    0,
                    ptr::null_mut(),
                    ptr::null_mut(),
                ) > 0
            {
                return true;
            }
            if self.has_color_paint && hb_ot_color_glyph_has_paint(face, glyph as u32) != 0 {
                return true;
            }
            if self.has_png {
                let blob = hb_ot_color_glyph_reference_png(self.font, glyph as u32);
                if !blob.is_null() {
                    let has_data = hb_blob_get_length(blob) > 0;
                    hb_blob_destroy(blob);
                    return has_data;
                }
            }
        }
        false
    }

    fn get_color_layers(
        &self,
        glyph: GlyphId,
        out: &mut Vec<ColorGlyphLayer>,
        foreground: ColorInt,
    ) -> usize {
        if !self.has_color_layers && !self.has_color_paint && !self.has_png {
            return 0;
        }
        if let Some(cached_layers) = self.color_layer_cache.borrow().get(&glyph).cloned() {
            for cached in &cached_layers {
                let mut layer = cached.clone();
                layer.color = if cached.use_foreground {
                    foreground
                } else {
                    cached.color
                };
                out.push(layer);
            }
            return cached_layers.len();
        }

        let mut layers = Vec::new();
        // SAFETY: all callback userdata points to stack-local `RawPath` or
        // `PaintState` values for synchronous HarfBuzz calls only. Count/fill
        // queries bound every POD vector, and owned font/function handles stay
        // live throughout callback execution.
        unsafe {
            let face = hb_font_get_face(self.font);
            if self.has_color_layers {
                let mut layer_count = hb_ot_color_glyph_get_layers(
                    face,
                    glyph as u32,
                    0,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                if layer_count > 0 {
                    // SAFETY: `hb_ot_color_layer_t` is C POD. The first query
                    // supplies the allocation bound; the second initializes at
                    // most `layer_count` elements while `face` stays alive.
                    let mut hb_layers =
                        vec![std::mem::zeroed::<hb_ot_color_layer_t>(); layer_count as usize];
                    hb_ot_color_glyph_get_layers(
                        face,
                        glyph as u32,
                        0,
                        &mut layer_count,
                        hb_layers.as_mut_ptr(),
                    );
                    layers.reserve(layer_count as usize);
                    for hb_layer in hb_layers.into_iter().take(layer_count as usize) {
                        let mut layer = ColorGlyphLayer::default();
                        hb_font_draw_glyph(
                            self.font,
                            hb_layer.glyph,
                            self.draw_funcs,
                            (&mut layer.path as *mut RawPath).cast(),
                        );
                        if hb_layer.color_index == 0xffff {
                            layer.use_foreground = true;
                            layer.color = foreground;
                        } else {
                            layer.use_foreground = false;
                            layer.color =
                                if (hb_layer.color_index as usize) < self.palette_colors.len() {
                                    hb_color_to_color_int(
                                        self.palette_colors[hb_layer.color_index as usize],
                                    )
                                } else {
                                    0xff000000
                                };
                        }
                        layers.push(layer);
                    }
                }
            }

            if layers.is_empty() && (self.has_color_paint || self.has_png) {
                let mut state = PaintState {
                    font: self.font,
                    draw_funcs: self.draw_funcs,
                    layers: &mut layers,
                    clip_glyph: 0,
                    has_clip: false,
                    foreground,
                    transform_stack: Vec::new(),
                };
                hb_font_paint_glyph(
                    self.font,
                    glyph as u32,
                    self.paint_funcs,
                    (&mut state as *mut PaintState).cast(),
                    0,
                    hb_color(0, 0, 0, 255),
                );
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

    fn on_shape_text(
        &self,
        text: &[Unichar],
        text_runs: &[TextRun],
        text_direction_flag: i32,
    ) -> Vec<Paragraph> {
        // SAFETY: the implementation passes only call-scoped slice buffers to
        // SheenBidi/HarfBuzz and releases every created algorithm/paragraph/
        // buffer handle before returning.
        unsafe { self.on_shape_text_ffi(text, text_runs, text_direction_flag) }
    }
}

impl HbFont {
    unsafe fn on_shape_text_ffi(
        &self,
        text: &[Unichar],
        text_runs: &[TextRun],
        text_direction_flag: i32,
    ) -> Vec<Paragraph> {
        let mut paragraphs = Vec::new();
        let sequence = SBCodepointSequence {
            stringEncoding: SBStringEncodingUTF32,
            stringBuffer: text.as_ptr() as *mut c_void,
            stringLength: text.len(),
        };
        let unicode_funcs = hb_unicode_funcs_get_default();
        let mut text_index = 0usize;
        let mut run_index = 0usize;
        let mut run_start_text_index = 0usize;
        let mut paragraph_start = 0usize;
        let algorithm = SBAlgorithmCreate(&sequence);
        let mut unichar_index = 0usize;
        let mut run_text_index = 0u32;
        let default_level = match text_direction_flag {
            0 => 0,
            1 => 1,
            _ => SBLevelDefaultLTR,
        };

        while paragraph_start < text.len() {
            let paragraph = SBAlgorithmCreateParagraph(
                algorithm,
                paragraph_start,
                i32::MAX as usize,
                default_level,
            );
            let paragraph_length = SBParagraphGetLength(paragraph);
            paragraph_start += paragraph_length;
            let bidi_levels =
                std::slice::from_raw_parts(SBParagraphGetLevelsPtr(paragraph), paragraph_length);
            let paragraph_level = SBParagraphGetBaseLevel(paragraph);
            let mut paragraph_text_index = 0usize;
            let mut bidi_runs = Vec::with_capacity(text_runs.len());

            while run_index < text_runs.len() {
                let text_run = &text_runs[run_index];
                assert_ne!(text_run.unichar_count, 0);
                let mut last_level = bidi_levels[paragraph_text_index];
                let point = text[text_index];
                let mut last_script = hb_unicode_script(unicode_funcs, point);
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
                    let mut script = if hb_unicode_general_category(unicode_funcs, point)
                        == HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK
                    {
                        HB_SCRIPT_INHERITED
                    } else {
                        hb_unicode_script(unicode_funcs, point)
                    };
                    if script == HB_SCRIPT_COMMON || script == HB_SCRIPT_INHERITED {
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
            SBParagraphRelease(paragraph);
        }
        SBAlgorithmRelease(algorithm);
        paragraphs
    }
}

unsafe fn make_line_metrics(font: *mut hb_font_t) -> LineMetrics {
    hb_ot_font_set_funcs(font);
    hb_font_set_scale(font, STANDARD_SCALE, STANDARD_SCALE);
    // SAFETY: both HarfBuzz extent records are C POD, zero is a valid
    // out-parameter seed, and `font` is required by this function's contract
    // to remain live for every same-call HarfBuzz access below.
    let mut extents = std::mem::zeroed::<hb_font_extents_t>();
    hb_font_get_h_extents(font, &mut extents);
    let ascent = -extents.ascender as f32 * INVERSE_SCALE;
    let measure_glyph_top = |unicode: u32| {
        let mut glyph = 0;
        // SAFETY: this HarfBuzz C POD is a same-call out parameter; it is read
        // only when the API reports successful initialization.
        let mut glyph_extents = std::mem::zeroed::<hb_glyph_extents_t>();
        if hb_font_get_nominal_glyph(font, unicode, &mut glyph) != 0
            && hb_font_get_glyph_extents(font, glyph, &mut glyph_extents) != 0
        {
            -glyph_extents.y_bearing as f32 * INVERSE_SCALE
        } else {
            ascent
        }
    };
    LineMetrics {
        ascent,
        descent: -extents.descender as f32 * INVERSE_SCALE,
        cap_height: measure_glyph_top(b'H' as u32),
        x_height: measure_glyph_top(b'x' as u32),
    }
}

unsafe fn fill_language_features(
    face: *mut hb_face_t,
    table_tag: u32,
    script_index: u32,
    language_index: u32,
    features: &mut HashSet<u32>,
) {
    let mut count = hb_ot_layout_language_get_feature_tags(
        face,
        table_tag,
        script_index,
        language_index,
        0,
        ptr::null_mut(),
        ptr::null_mut(),
    );
    let mut tags = vec![0; count as usize];
    hb_ot_layout_language_get_feature_tags(
        face,
        table_tag,
        script_index,
        language_index,
        0,
        &mut count,
        tags.as_mut_ptr(),
    );
    for feature_tag in tags.into_iter().take(count as usize) {
        features.insert(feature_tag);
    }
}

unsafe fn fill_features(face: *mut hb_face_t, table_tag: u32, features: &mut HashSet<u32>) {
    let mut script_count =
        hb_ot_layout_table_get_script_tags(face, table_tag, 0, ptr::null_mut(), ptr::null_mut());
    let mut scripts = vec![0; script_count as usize];
    hb_ot_layout_table_get_script_tags(face, table_tag, 0, &mut script_count, scripts.as_mut_ptr());
    for script_index in 0..script_count {
        let mut language_count = hb_ot_layout_script_get_language_tags(
            face,
            table_tag,
            script_index,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        );
        if language_count > 0 {
            let mut languages = vec![0; language_count as usize];
            hb_ot_layout_script_get_language_tags(
                face,
                table_tag,
                script_index,
                0,
                &mut language_count,
                languages.as_mut_ptr(),
            );
            for language_index in 0..language_count {
                fill_language_features(face, table_tag, script_index, language_index, features);
            }
        } else {
            fill_language_features(
                face,
                table_tag,
                script_index,
                HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX,
                features,
            );
        }
    }
}

fn shape_run(text: &[Unichar], text_run: &TextRun, text_offset: u32) -> GlyphRun {
    // SAFETY: the HarfBuzz buffer is owned and destroyed in this call. Input
    // slices outlive shaping, and glyph info/position slices are bounded by
    // the count HarfBuzz returns while the buffer remains live.
    unsafe {
        let buffer = hb_buffer_create();
        hb_buffer_add_utf32(
            buffer,
            text.as_ptr(),
            text_run.unichar_count as i32,
            0,
            text_run.unichar_count as i32,
        );
        hb_buffer_set_direction(
            buffer,
            if text_run.level & 1 != 0 {
                HB_DIRECTION_RTL
            } else {
                HB_DIRECTION_LTR
            },
        );
        hb_buffer_set_script(buffer, text_run.script as hb_script_t);
        hb_buffer_set_language(buffer, hb_language_get_default());
        let hb_font = text_run
            .font
            .as_ref()
            .unwrap()
            .as_any()
            .downcast_ref::<HbFont>()
            .expect("text font must be an HBFont");
        hb_shape(
            hb_font.font,
            buffer,
            hb_font.features.as_ptr(),
            hb_font.features.len() as u32,
        );
        let mut glyph_count = 0;
        let glyph_info = hb_buffer_get_glyph_infos(buffer, &mut glyph_count);
        let glyph_positions = hb_buffer_get_glyph_positions(buffer, &mut glyph_count);
        let infos = std::slice::from_raw_parts(glyph_info, glyph_count as usize);
        let positions = std::slice::from_raw_parts(glyph_positions, glyph_count as usize);
        let mut glyph_run = GlyphRun::new(glyph_count as usize);
        glyph_run.font = text_run.font.clone();
        glyph_run.size = text_run.size;
        glyph_run.line_height = text_run.line_height;
        glyph_run.letter_spacing = text_run.letter_spacing;
        glyph_run.style_id = text_run.style_id;
        glyph_run.level = text_run.level;
        let scale = text_run.size / STANDARD_SCALE as f32;
        for index in 0..glyph_count as usize {
            let source_index = if text_run.level & 1 != 0 {
                glyph_count as usize - 1 - index
            } else {
                index
            };
            glyph_run.glyphs[index] = infos[source_index].codepoint as u16;
            glyph_run.text_indices[index] = text_offset + infos[source_index].cluster;
            let advance =
                positions[source_index].x_advance as f32 * scale + text_run.letter_spacing;
            glyph_run.advances[index] = advance;
            glyph_run.xpos[index] = advance;
            glyph_run.offsets[index] = Vec2D::new(
                positions[source_index].x_offset as f32 * scale,
                -positions[source_index].y_offset as f32 * scale,
            );
        }
        glyph_run.xpos[glyph_count as usize] = 0.0;
        hb_buffer_destroy(buffer);
        glyph_run
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
    font: *mut hb_font_t,
    draw_funcs: *mut hb_draw_funcs_t,
    layers: &'a mut Vec<ColorGlyphLayer>,
    clip_glyph: GlyphId,
    has_clip: bool,
    foreground: ColorInt,
    transform_stack: Vec<Mat2D>,
}

impl PaintState<'_> {
    fn push_transform(&mut self, xx: f32, yx: f32, xy: f32, yy: f32, dx: f32, dy: f32) {
        let matrix = Mat2D::new(xx, yx, xy, yy, dx * INVERSE_SCALE, -dy * INVERSE_SCALE);
        if self.transform_stack.is_empty() {
            self.transform_stack.push(matrix);
        } else {
            self.transform_stack
                .push(self.transform_stack.last().unwrap().multiply(&matrix));
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

    unsafe fn extract_stops(
        color_line: *mut hb_color_line_t,
        foreground: ColorInt,
    ) -> Vec<GradientStop> {
        let mut count = 0;
        hb_color_line_get_color_stops(color_line, 0, &mut count, ptr::null_mut());
        // SAFETY: `hb_color_stop_t` is C POD. The count-only query determines
        // the allocation bound, the fill query initializes at most that many
        // slots, and `color_line` remains live for both adjacent calls.
        let mut hb_stops = vec![std::mem::zeroed::<hb_color_stop_t>(); count as usize];
        hb_color_line_get_color_stops(color_line, 0, &mut count, hb_stops.as_mut_ptr());
        hb_stops
            .into_iter()
            .take(count as usize)
            .map(|stop| GradientStop {
                offset: stop.offset,
                color: if stop.is_foreground != 0 {
                    foreground
                } else {
                    hb_color_to_color_int(stop.color)
                },
            })
            .collect()
    }

    unsafe fn make_clip_layer(&self) -> ColorGlyphLayer {
        let mut layer = ColorGlyphLayer::default();
        hb_font_draw_glyph(
            self.font,
            self.clip_glyph as u32,
            self.draw_funcs,
            (&mut layer.path as *mut RawPath).cast(),
        );
        layer
    }
}

fn hb_color_to_color_int(color: hb_color_t) -> ColorInt {
    // SAFETY: HarfBuzz's color accessors are pure scalar extraction macros/
    // functions and do not dereference or retain external storage.
    unsafe {
        color_argb(
            hb_color_get_alpha(color),
            hb_color_get_red(color),
            hb_color_get_green(color),
            hb_color_get_blue(color),
        )
    }
}

const fn tag(a: u8, b: u8, c: u8, d: u8) -> u32 {
    ((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | d as u32
}

const fn hb_color(red: u8, green: u8, blue: u8, alpha: u8) -> u32 {
    (blue as u32) | ((green as u32) << 8) | ((red as u32) << 16) | ((alpha as u32) << 24)
}

// SAFETY CONTRACT FOR THE HARFBUZZ CALLBACKS BELOW: HarfBuzz invokes each
// callback synchronously from `hb_font_draw_glyph`/`hb_font_paint_glyph`.
// `path` is the unique stack-local `RawPath` supplied by `get_path` or layer
// construction; `data` is the unique stack-local `PaintState` supplied by
// `get_color_layers`; HarfBuzz color-line/blob/extent handles remain live for
// the callback and are never retained. Every callback returns before those
// stack locals or HarfBuzz handles are released.
unsafe extern "C" fn raw_path_move_to(
    _: *mut hb_draw_funcs_t,
    path: *mut c_void,
    _: *mut hb_draw_state_t,
    x: f32,
    y: f32,
    _: *mut c_void,
) {
    (*(path as *mut RawPath)).move_to(x * INVERSE_SCALE, -y * INVERSE_SCALE);
}

unsafe extern "C" fn raw_path_line_to(
    _: *mut hb_draw_funcs_t,
    path: *mut c_void,
    _: *mut hb_draw_state_t,
    x: f32,
    y: f32,
    _: *mut c_void,
) {
    (*(path as *mut RawPath)).line_to(x * INVERSE_SCALE, -y * INVERSE_SCALE);
}

unsafe extern "C" fn raw_path_quadratic_to(
    _: *mut hb_draw_funcs_t,
    path: *mut c_void,
    _: *mut hb_draw_state_t,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    _: *mut c_void,
) {
    (*(path as *mut RawPath)).quad_to_cubic(
        x1 * INVERSE_SCALE,
        -y1 * INVERSE_SCALE,
        x2 * INVERSE_SCALE,
        -y2 * INVERSE_SCALE,
    );
}

unsafe extern "C" fn raw_path_cubic_to(
    _: *mut hb_draw_funcs_t,
    path: *mut c_void,
    _: *mut hb_draw_state_t,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    _: *mut c_void,
) {
    (*(path as *mut RawPath)).cubic_to(
        x1 * INVERSE_SCALE,
        -y1 * INVERSE_SCALE,
        x2 * INVERSE_SCALE,
        -y2 * INVERSE_SCALE,
        x3 * INVERSE_SCALE,
        -y3 * INVERSE_SCALE,
    );
}

unsafe extern "C" fn raw_path_close(
    _: *mut hb_draw_funcs_t,
    path: *mut c_void,
    _: *mut hb_draw_state_t,
    _: *mut c_void,
) {
    (*(path as *mut RawPath)).close();
}

unsafe fn paint_state<'a>(data: *mut c_void) -> &'a mut PaintState<'a> {
    // SAFETY: established by the callback contract immediately above.
    &mut *(data as *mut PaintState<'a>)
}

unsafe extern "C" fn paint_push_transform(
    _: *mut hb_paint_funcs_t,
    data: *mut c_void,
    xx: f32,
    yx: f32,
    xy: f32,
    yy: f32,
    dx: f32,
    dy: f32,
    _: *mut c_void,
) {
    paint_state(data).push_transform(xx, yx, xy, yy, dx, dy);
}

unsafe extern "C" fn paint_pop_transform(
    _: *mut hb_paint_funcs_t,
    data: *mut c_void,
    _: *mut c_void,
) {
    paint_state(data).pop_transform();
}

unsafe extern "C" fn paint_push_clip_glyph(
    _: *mut hb_paint_funcs_t,
    data: *mut c_void,
    glyph: u32,
    _: *mut hb_font_t,
    _: *mut c_void,
) {
    let state = paint_state(data);
    state.clip_glyph = glyph as GlyphId;
    state.has_clip = true;
}

unsafe extern "C" fn paint_push_clip_rectangle(
    _: *mut hb_paint_funcs_t,
    _: *mut c_void,
    _: f32,
    _: f32,
    _: f32,
    _: f32,
    _: *mut c_void,
) {
}

unsafe extern "C" fn paint_pop_clip(_: *mut hb_paint_funcs_t, data: *mut c_void, _: *mut c_void) {
    paint_state(data).has_clip = false;
}

unsafe extern "C" fn paint_solid(
    _: *mut hb_paint_funcs_t,
    data: *mut c_void,
    is_foreground: i32,
    color: hb_color_t,
    _: *mut c_void,
) {
    let state = paint_state(data);
    if !state.has_clip {
        return;
    }
    let mut layer = state.make_clip_layer();
    layer.paint_type = ColorGlyphPaintType::Solid;
    if is_foreground != 0 {
        layer.use_foreground = true;
        layer.color = state.foreground;
    } else {
        layer.color = hb_color_to_color_int(color);
    }
    state.layers.push(layer);
}

unsafe extern "C" fn paint_linear_gradient(
    _: *mut hb_paint_funcs_t,
    data: *mut c_void,
    color_line: *mut hb_color_line_t,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    _: f32,
    _: f32,
    _: *mut c_void,
) {
    let state = paint_state(data);
    if !state.has_clip {
        return;
    }
    let mut layer = state.make_clip_layer();
    layer.paint_type = ColorGlyphPaintType::LinearGradient;
    layer.stops = PaintState::extract_stops(color_line, state.foreground);
    let point0 = state.map_point(x0, y0);
    let point1 = state.map_point(x1, y1);
    layer.x0 = point0.x;
    layer.y0 = point0.y;
    layer.x1 = point1.x;
    layer.y1 = point1.y;
    state.layers.push(layer);
}

unsafe extern "C" fn paint_radial_gradient(
    _: *mut hb_paint_funcs_t,
    data: *mut c_void,
    color_line: *mut hb_color_line_t,
    x0: f32,
    y0: f32,
    radius0: f32,
    x1: f32,
    y1: f32,
    radius1: f32,
    _: *mut c_void,
) {
    let state = paint_state(data);
    if !state.has_clip {
        return;
    }
    let mut layer = state.make_clip_layer();
    layer.paint_type = ColorGlyphPaintType::RadialGradient;
    layer.stops = PaintState::extract_stops(color_line, state.foreground);
    let point0 = state.map_point(x0, y0);
    let point1 = state.map_point(x1, y1);
    layer.x0 = point0.x;
    layer.y0 = point0.y;
    layer.x1 = point1.x;
    layer.y1 = point1.y;
    layer.r0 = state.map_radius(radius0);
    layer.r1 = state.map_radius(radius1);
    state.layers.push(layer);
}

unsafe extern "C" fn paint_sweep_gradient(
    _: *mut hb_paint_funcs_t,
    data: *mut c_void,
    color_line: *mut hb_color_line_t,
    center_x: f32,
    center_y: f32,
    start_angle: f32,
    end_angle: f32,
    _: *mut c_void,
) {
    let state = paint_state(data);
    if !state.has_clip {
        return;
    }
    let mut layer = state.make_clip_layer();
    layer.paint_type = ColorGlyphPaintType::SweepGradient;
    layer.stops = PaintState::extract_stops(color_line, state.foreground);
    let center = state.map_point(center_x, center_y);
    layer.x0 = center.x;
    layer.y0 = center.y;
    layer.start_angle = start_angle;
    layer.end_angle = end_angle;
    state.layers.push(layer);
}

unsafe extern "C" fn paint_push_group(_: *mut hb_paint_funcs_t, _: *mut c_void, _: *mut c_void) {}

unsafe extern "C" fn paint_pop_group(
    _: *mut hb_paint_funcs_t,
    _: *mut c_void,
    _: hb_paint_composite_mode_t,
    _: *mut c_void,
) {
}

unsafe extern "C" fn paint_color_glyph(
    _: *mut hb_paint_funcs_t,
    _: *mut c_void,
    _: u32,
    _: *mut hb_font_t,
    _: *mut c_void,
) -> i32 {
    0
}

unsafe extern "C" fn paint_image(
    _: *mut hb_paint_funcs_t,
    data: *mut c_void,
    blob: *mut hb_blob_t,
    width: u32,
    height: u32,
    format: u32,
    _: f32,
    extents: *mut hb_glyph_extents_t,
    _: *mut c_void,
) -> i32 {
    if format != tag(b'p', b'n', b'g', b' ') {
        return 0;
    }
    let mut length = 0;
    let bytes = hb_blob_get_data(blob, &mut length);
    if bytes.is_null() || length == 0 {
        return 0;
    }
    let state = paint_state(data);
    let mut layer = ColorGlyphLayer::default();
    layer.paint_type = ColorGlyphPaintType::Image;
    // SAFETY: HarfBuzz returned `length` readable bytes owned by the live
    // callback-scoped blob. They are copied before the callback returns.
    layer.image_bytes = std::slice::from_raw_parts(bytes.cast::<u8>(), length as usize).to_vec();
    layer.image_width = width;
    layer.image_height = height;
    if !extents.is_null() {
        // SAFETY: HarfBuzz supplied a non-null callback-scoped extent record;
        // it is read only during this callback and never retained.
        layer.image_bearing_x = (*extents).x_bearing as f32 * INVERSE_SCALE;
        layer.image_bearing_y = -(*extents).y_bearing as f32 * INVERSE_SCALE;
        layer.image_extent_x = (*extents).width as f32 * INVERSE_SCALE;
        layer.image_extent_y = -(*extents).height as f32 * INVERSE_SCALE;
    }
    state.layers.push(layer);
    1
}
