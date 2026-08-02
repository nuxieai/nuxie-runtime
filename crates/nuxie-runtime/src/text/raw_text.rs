// Standalone RawText owner ---------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextSizing {
    #[default]
    AutoWidth = 0,
    AutoHeight = 1,
    Fixed = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextOverflow {
    #[default]
    Visible = 0,
    Hidden = 1,
    Clipped = 2,
    Ellipsis = 3,
    Fit = 4,
    FitFontSize = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextAlign {
    #[default]
    Left = 0,
    Right = 1,
    Center = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTextFontError {
    message: &'static str,
}

impl std::fmt::Display for RawTextFontError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for RawTextFontError {}

#[derive(Clone)]
pub struct RawTextFont {
    bytes: std::sync::Arc<[u8]>,
    face_index: u32,
    fallbacks: std::sync::Arc<[RawTextFont]>,
}

impl std::fmt::Debug for RawTextFont {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RawTextFont")
            .field("byte_len", &self.bytes.len())
            .field("face_index", &self.face_index)
            .field("fallback_count", &self.fallbacks.len())
            .finish()
    }
}

impl RawTextFont {
    pub fn decode(
        bytes: impl Into<std::sync::Arc<[u8]>>,
    ) -> Result<Self, RawTextFontError> {
        Self::decode_face(bytes, 0)
    }

    pub fn decode_face(
        bytes: impl Into<std::sync::Arc<[u8]>>,
        face_index: u32,
    ) -> Result<Self, RawTextFontError> {
        let bytes = bytes.into();
        if HarfFontRef::from_index(bytes.as_ref(), face_index).is_err()
            || SkrifaFontRef::from_index(bytes.as_ref(), face_index).is_err()
        {
            return Err(RawTextFontError {
                message: "font bytes or face index are invalid",
            });
        }
        Ok(Self {
            bytes,
            face_index,
            fallbacks: std::sync::Arc::from([]),
        })
    }

    /// Return the same primary font with an ordered fallback chain.
    ///
    /// This is the safe Rust counterpart of C++'s process-global fallback
    /// callback and is intentionally occurrence-local.
    pub fn with_fallbacks(mut self, fallbacks: impl IntoIterator<Item = RawTextFont>) -> Self {
        self.fallbacks = fallbacks.into_iter().collect::<Vec<_>>().into();
        self
    }

    pub fn face_index(&self) -> u32 {
        self.face_index
    }

    fn bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

struct RawTextPaintInner {
    paint: std::cell::RefCell<Box<dyn nuxie_render_api::RenderPaint>>,
}

#[derive(Clone)]
pub struct RawTextPaint {
    inner: std::rc::Rc<RawTextPaintInner>,
}

impl std::fmt::Debug for RawTextPaint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RawTextPaint")
            .field("identity", &std::rc::Rc::as_ptr(&self.inner))
            .finish()
    }
}

impl RawTextPaint {
    pub fn new(factory: &mut dyn nuxie_render_api::Factory) -> Self {
        Self {
            inner: std::rc::Rc::new(RawTextPaintInner {
                paint: std::cell::RefCell::new(factory.make_render_paint()),
            }),
        }
    }

    pub fn with_paint<R>(
        &self,
        edit: impl FnOnce(&mut dyn nuxie_render_api::RenderPaint) -> R,
    ) -> R {
        edit(self.inner.paint.borrow_mut().as_mut())
    }

    fn same_identity(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.inner, &other.inner)
    }
}

#[derive(Clone)]
struct StandaloneTextRun {
    text: String,
    font: RawTextFont,
    size: f32,
    line_height: f32,
    letter_spacing: f32,
    style_index: usize,
    char_start: usize,
}

struct StandaloneRenderStyle {
    paint: Option<RawTextPaint>,
    foreground: u32,
    is_empty: bool,
    raw_path: nuxie_render_api::RawPath,
    render_path: Option<Box<dyn nuxie_render_api::RenderPath>>,
}

impl std::fmt::Debug for StandaloneRenderStyle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StandaloneRenderStyle")
            .field("paint", &self.paint)
            .field("foreground", &format_args!("0x{:08x}", self.foreground))
            .field("is_empty", &self.is_empty)
            .field("raw_path", &self.raw_path)
            .field("has_render_path", &self.render_path.is_some())
            .finish()
    }
}

#[derive(Debug, Clone)]
struct StandaloneGlyph {
    font: RawTextFont,
    glyph_id: u32,
    char_index: usize,
    char_len: usize,
    style_index: usize,
    advance: f32,
    offset_x: f32,
    offset_y: f32,
    size: f32,
}

#[derive(Debug, Clone)]
struct StandaloneLine {
    paragraph: usize,
    char_start: usize,
    char_end: usize,
    glyphs: Vec<StandaloneGlyph>,
    width: f32,
    top: f32,
    baseline: f32,
    bottom: f32,
    start_x: f32,
}

#[derive(Debug, Clone)]
struct StandaloneColorCommand {
    font: RawTextFont,
    glyph_id: u32,
    transform: nuxie_render_api::Mat2D,
    foreground: u32,
}

#[derive(Debug, Clone)]
enum StandaloneDrawCommand {
    Style(usize),
    Color(StandaloneColorCommand),
}

pub struct RawText<'factory> {
    factory: &'factory mut dyn nuxie_render_api::Factory,
    runs: Vec<StandaloneTextRun>,
    styles: Vec<StandaloneRenderStyle>,
    render_styles: Vec<usize>,
    lines: Vec<StandaloneLine>,
    draw_commands: Vec<StandaloneDrawCommand>,
    dirty: bool,
    paragraph_spacing: f32,
    sizing: TextSizing,
    overflow: TextOverflow,
    align: TextAlign,
    max_width: f32,
    max_height: f32,
    bounds: nuxie_render_api::Aabb,
    clip_path: Option<Box<dyn nuxie_render_api::RenderPath>>,
    update_count: u64,
}

impl std::fmt::Debug for RawText<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RawText")
            .field("runs", &self.runs.len())
            .field("styles", &self.styles)
            .field("render_styles", &self.render_styles)
            .field("lines", &self.lines)
            .field("draw_commands", &self.draw_commands)
            .field("dirty", &self.dirty)
            .field("bounds", &self.bounds)
            .finish_non_exhaustive()
    }
}

impl<'factory> RawText<'factory> {
    pub fn new(factory: &'factory mut dyn nuxie_render_api::Factory) -> Self {
        Self {
            factory,
            runs: Vec::new(),
            styles: Vec::new(),
            render_styles: Vec::new(),
            lines: Vec::new(),
            draw_commands: Vec::new(),
            dirty: false,
            paragraph_spacing: 0.0,
            sizing: TextSizing::AutoWidth,
            overflow: TextOverflow::Visible,
            align: TextAlign::Left,
            max_width: 0.0,
            max_height: 0.0,
            bounds: nuxie_render_api::Aabb::new(0.0, 0.0, 0.0, 0.0),
            clip_path: None,
            update_count: 0,
        }
    }

    pub fn make_paint(&mut self) -> RawTextPaint {
        RawTextPaint::new(self.factory)
    }

    pub fn empty(&self) -> bool {
        self.runs.is_empty()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &mut self,
        text: &str,
        paint: Option<RawTextPaint>,
        font: &RawTextFont,
        size: f32,
        line_height: f32,
        letter_spacing: f32,
        foreground_color: u32,
    ) {
        let style_index = self
            .styles
            .iter()
            .position(|style| match (&style.paint, &paint) {
                (Some(left), Some(right)) => left.same_identity(right),
                (None, None) => true,
                _ => false,
            })
            .unwrap_or_else(|| {
                self.styles.push(StandaloneRenderStyle {
                    paint: paint.clone(),
                    foreground: foreground_color,
                    is_empty: true,
                    raw_path: nuxie_render_api::RawPath::new(),
                    render_path: None,
                });
                self.styles.len() - 1
            });
        let text = text.split('\0').next().unwrap_or_default().to_owned();
        let char_start = self
            .runs
            .last()
            .map(|run| run.char_start + run.text.chars().count())
            .unwrap_or(0);
        self.runs.push(StandaloneTextRun {
            text,
            font: font.clone(),
            size,
            line_height,
            letter_spacing,
            style_index,
            char_start,
        });
        self.dirty = true;
    }

    pub fn append_default(
        &mut self,
        text: &str,
        paint: Option<RawTextPaint>,
        font: &RawTextFont,
    ) {
        self.append(text, paint, font, 16.0, -1.0, 0.0, 0xff00_0000);
    }

    pub fn clear(&mut self) {
        self.runs.clear();
        self.dirty = true;
    }

    pub fn sizing(&self) -> TextSizing {
        self.sizing
    }

    pub fn overflow(&self) -> TextOverflow {
        self.overflow
    }

    pub fn align(&self) -> TextAlign {
        self.align
    }

    pub fn max_width(&self) -> f32 {
        self.max_width
    }

    pub fn max_height(&self) -> f32 {
        self.max_height
    }

    pub fn paragraph_spacing(&self) -> f32 {
        self.paragraph_spacing
    }

    pub fn set_sizing(&mut self, value: TextSizing) {
        if self.sizing != value {
            self.sizing = value;
            self.dirty = true;
        }
    }

    pub fn set_overflow(&mut self, value: TextOverflow) {
        if self.overflow != value {
            self.overflow = value;
            self.dirty = true;
        }
    }

    pub fn set_align(&mut self, value: TextAlign) {
        if self.align != value {
            self.align = value;
            self.dirty = true;
        }
    }

    pub fn set_max_width(&mut self, value: f32) {
        if self.max_width != value {
            self.max_width = value;
            self.dirty = true;
        }
    }

    pub fn set_max_height(&mut self, value: f32) {
        if self.max_height != value {
            self.max_height = value;
            self.dirty = true;
        }
    }

    pub fn set_paragraph_spacing(&mut self, value: f32) {
        if self.paragraph_spacing != value {
            self.paragraph_spacing = value;
            self.dirty = true;
        }
    }

    pub fn bounds(&mut self) -> nuxie_render_api::Aabb {
        self.update_if_dirty();
        self.bounds
    }

    pub fn render(
        &mut self,
        renderer: &mut dyn nuxie_render_api::Renderer,
        override_paint: Option<&RawTextPaint>,
    ) {
        self.update_if_dirty();
        let clipped = self.overflow == TextOverflow::Clipped && self.clip_path.is_some();
        if clipped {
            renderer.save();
            if let Some(path) = self.clip_path.as_deref() {
                renderer.clip_path(path);
            }
        }
        for command in &self.draw_commands {
            match command {
                StandaloneDrawCommand::Style(style_index) => {
                    let Some(style) = self.styles.get(*style_index) else {
                        continue;
                    };
                    let Some(path) = style.render_path.as_deref() else {
                        continue;
                    };
                    if let Some(paint) = override_paint.or(style.paint.as_ref()) {
                        let paint = paint.inner.paint.borrow();
                        renderer.draw_path(path, paint.as_ref());
                    }
                }
                StandaloneDrawCommand::Color(command) => {
                    let layers = runtime_extract_color_glyph_layers_face(
                        command.font.bytes(),
                        command.font.face_index,
                        command.glyph_id,
                        command.foreground,
                    );
                    if layers.is_empty() {
                        continue;
                    }
                    renderer.save();
                    renderer.transform(command.transform);
                    for layer in &layers {
                        let color = match layer.paint {
                            RuntimeColorGlyphPaint::Solid { color } => color,
                            RuntimeColorGlyphPaint::Image { .. } => {
                                // Binding ruling 1: standalone RawText never
                                // decodes or draws bitmap glyph layers.
                                continue;
                            }
                            RuntimeColorGlyphPaint::LinearGradient { .. }
                            | RuntimeColorGlyphPaint::RadialGradient { .. }
                            | RuntimeColorGlyphPaint::SweepGradient { .. } => {
                                // C++ RawText consumes only the flat layer
                                // color field, whose unsupported-metadata
                                // fallback is opaque black.
                                0xff00_0000
                            }
                        };
                        let path = self
                            .factory
                            .make_render_path(layer.path.clone(), nuxie_render_api::FillRule::NonZero);
                        let mut paint = self.factory.make_render_paint();
                        paint.style(nuxie_render_api::RenderPaintStyle::Fill);
                        paint.color(color);
                        renderer.draw_path(path.as_ref(), paint.as_ref());
                    }
                    renderer.restore();
                }
            }
        }
        if clipped {
            renderer.restore();
        }
    }

    #[doc(hidden)]
    pub fn debug_update_count(&self) -> u64 {
        self.update_count
    }

    #[doc(hidden)]
    pub fn debug_dirty(&self) -> bool {
        self.dirty
    }

    #[doc(hidden)]
    pub fn debug_style_count(&self) -> usize {
        self.styles.len()
    }

    #[doc(hidden)]
    pub fn debug_style_foreground(&self, index: usize) -> Option<u32> {
        self.styles.get(index).map(|style| style.foreground)
    }

    #[doc(hidden)]
    pub fn debug_command_kinds(&self) -> Vec<&'static str> {
        self.draw_commands
            .iter()
            .map(|command| match command {
                StandaloneDrawCommand::Style(_) => "style",
                StandaloneDrawCommand::Color(_) => "color",
            })
            .collect()
    }

    #[doc(hidden)]
    pub fn debug_has_clip(&self) -> bool {
        self.clip_path.is_some()
    }

    #[doc(hidden)]
    pub fn debug_style_path_bounds(&self) -> Vec<nuxie_render_api::Aabb> {
        self.styles
            .iter()
            .filter(|style| !style.is_empty)
            .filter_map(|style| style.raw_path.bounds())
            .collect()
    }

    fn update_if_dirty(&mut self) {
        if self.dirty {
            self.update();
            self.dirty = false;
        }
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn update(&mut self) {
        self.update_count = self.update_count.wrapping_add(1);
        for style in &mut self.styles {
            style.raw_path = nuxie_render_api::RawPath::new();
            style.is_empty = true;
        }
        self.render_styles.clear();
        self.draw_commands.clear();
        if self.runs.is_empty() {
            // C++ deliberately retains stale lines, bounds, and clip here.
            return;
        }

        let text = self
            .runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>();
        let mut glyphs = Vec::new();
        for run in &self.runs {
            glyphs.extend(shape_standalone_run(run));
        }
        if glyphs.is_empty() && text.is_empty() {
            self.bounds = nuxie_render_api::Aabb::new(0.0, 0.0, 0.0, 0.0);
            return;
        }

        let mut lines = standalone_break_lines(&text, &glyphs, self.sizing, self.max_width);
        standalone_measure_lines(&mut lines, &self.runs, self.paragraph_spacing);
        standalone_reorder_bidi(&text, &mut lines);

        let measured_width = lines.iter().map(|line| line.width).fold(0.0f32, f32::max);
        let min_y = 0.0;
        let content_height = lines
            .last()
            .map(|line| line.bottom)
            .unwrap_or(0.0);
        self.bounds = match self.sizing {
            TextSizing::AutoWidth => nuxie_render_api::Aabb::new(
                0.0,
                min_y,
                measured_width,
                min_y.max(content_height),
            ),
            TextSizing::AutoHeight => nuxie_render_api::Aabb::new(
                0.0,
                min_y,
                self.max_width,
                min_y.max(content_height),
            ),
            TextSizing::Fixed => nuxie_render_api::Aabb::new(
                0.0,
                min_y,
                self.max_width,
                min_y + self.max_height,
            ),
        };

        if self.overflow == TextOverflow::Clipped {
            let path = self
                .clip_path
                .get_or_insert_with(|| self.factory.make_empty_render_path());
            path.rewind();
            path.move_to(self.bounds.min_x, self.bounds.min_y);
            path.line_to(self.bounds.max_x, self.bounds.min_y);
            path.line_to(self.bounds.max_x, self.bounds.max_y);
            path.line_to(self.bounds.min_x, self.bounds.max_y);
            path.close();
        } else {
            self.clip_path = None;
        }

        if self.sizing == TextSizing::Fixed && self.overflow == TextOverflow::Ellipsis {
            standalone_apply_ellipsis(&mut lines, &self.runs, self.max_width, self.max_height);
        }
        self.lines = lines;
        let lines = self.lines.clone();
        for line in &lines {
            if self.sizing == TextSizing::Fixed {
                if self.overflow == TextOverflow::Hidden && line.bottom > self.max_height {
                    break;
                }
                if self.overflow == TextOverflow::Clipped && line.top > self.max_height {
                    break;
                }
            }
            let mut x = match self.align {
                TextAlign::Left => 0.0,
                TextAlign::Right => self.max_width - line.width,
                TextAlign::Center => (self.max_width - line.width) / 2.0,
            };
            for glyph in &line.glyphs {
                let style_index = glyph.style_index;
                let Some(style) = self.styles.get(style_index) else {
                    x += glyph.advance;
                    continue;
                };
                let classification = runtime_classify_color_glyph_face(
                    glyph.font.bytes(),
                    glyph.font.face_index,
                    glyph.glyph_id,
                );
                if classification != RuntimeColorGlyphClassification::Monochrome {
                    self.draw_commands.push(StandaloneDrawCommand::Color(
                        StandaloneColorCommand {
                            font: glyph.font.clone(),
                            glyph_id: glyph.glyph_id,
                            transform: nuxie_render_api::Mat2D([
                                glyph.size,
                                0.0,
                                0.0,
                                glyph.size,
                                x + glyph.offset_x,
                                line.baseline + glyph.offset_y,
                            ]),
                            foreground: style.foreground,
                        },
                    ));
                } else {
                    self.append_monochrome_glyph(style_index, glyph, x, line.baseline);
                }
                x += glyph.advance;
            }
        }

        for style in &mut self.styles {
            if style.is_empty {
                continue;
            }
            let render_path = style.render_path.get_or_insert_with(|| {
                self.factory.make_empty_render_path()
            });
            render_path.rewind();
            render_path.fill_rule(nuxie_render_api::FillRule::Clockwise);
            render_path.add_raw_path(&style.raw_path);
        }
    }

    fn append_monochrome_glyph(
        &mut self,
        style_index: usize,
        glyph: &StandaloneGlyph,
        x: f32,
        baseline: f32,
    ) {
        let Ok(font) = SkrifaFontRef::from_index(glyph.font.bytes(), glyph.font.face_index) else {
            return;
        };
        let Some(outline) = font.outline_glyphs().get(GlyphId::new(glyph.glyph_id)) else {
            // Even an empty outline establishes first style occurrence.
            if let Some(style) = self.styles.get_mut(style_index)
                && style.is_empty
            {
                style.is_empty = false;
                self.render_styles.push(style_index);
                self.draw_commands.push(StandaloneDrawCommand::Style(style_index));
            }
            return;
        };
        let mut pen = TextOutlinePen::new(
            x + glyph.offset_x,
            baseline + glyph.offset_y,
            glyph.size / TEXT_SHAPE_SCALE_F32,
            x + glyph.offset_x + glyph.advance * 0.5,
            baseline + glyph.offset_y,
            Mat2D::IDENTITY,
        );
        let settings = DrawSettings::unhinted(Size::new(TEXT_SHAPE_SCALE_F32), LocationRef::default())
            .with_path_style(PathStyle::FreeType);
        let _ = outline.draw(settings, &mut pen);
        if let Some(style) = self.styles.get_mut(style_index) {
            let mut commands = crate::draw::runtime_path_commands_from_raw_path(&style.raw_path);
            commands.extend(pen.commands);
            style.raw_path = crate::math::raw_path::runtime_raw_path_from_commands(&commands);
            if style.is_empty {
                style.is_empty = false;
                self.render_styles.push(style_index);
                self.draw_commands.push(StandaloneDrawCommand::Style(style_index));
            }
        }
    }
}

fn shape_standalone_run(run: &StandaloneTextRun) -> Vec<StandaloneGlyph> {
    let Ok(harf_font) = HarfFontRef::from_index(run.font.bytes(), run.font.face_index) else {
        return Vec::new();
    };
    let shaper_data = ShaperData::new(&harf_font);
    let shaper = shaper_data.shaper(&harf_font).build();
    let Ok(skrifa_font) = SkrifaFontRef::from_index(run.font.bytes(), run.font.face_index) else {
        return Vec::new();
    };
    let raw = shape_text_glyphs(
        &shaper,
        &run.text,
        disable_legacy_kern_for_advances(&skrifa_font),
    );
    raw.iter()
        .enumerate()
        .map(|(index, glyph)| {
            let fallback = standalone_fallback_glyph(run, glyph);
            let (font, shaped) = fallback
                .as_ref()
                .map(|(font, glyph)| (font.clone(), glyph))
                .unwrap_or_else(|| (run.font.clone(), glyph));
            StandaloneGlyph {
                font,
                glyph_id: shaped.glyph_id,
                char_index: run.char_start + character_index_for_cluster(&run.text, glyph.cluster),
                char_len: glyph_character_len(&run.text, &raw, index),
                style_index: run.style_index,
                advance: shaped.advance * run.size / TEXT_SHAPE_SCALE_F32 + run.letter_spacing,
                offset_x: shaped.offset_x * run.size / TEXT_SHAPE_SCALE_F32,
                offset_y: shaped.offset_y * run.size / TEXT_SHAPE_SCALE_F32,
                size: run.size,
            }
        })
        .collect()
}

fn standalone_fallback_glyph(
    run: &StandaloneTextRun,
    glyph: &TextGlyph,
) -> Option<(RawTextFont, TextGlyph)> {
    if glyph.glyph_id != 0 {
        return None;
    }
    let byte = (glyph.cluster as usize).min(run.text.len());
    let character = run.text.get(byte..)?.chars().next()?;
    for fallback in run.font.fallbacks.iter() {
        let harf_font = HarfFontRef::from_index(fallback.bytes(), fallback.face_index).ok()?;
        let skrifa_font = SkrifaFontRef::from_index(fallback.bytes(), fallback.face_index).ok()?;
        let shaper_data = ShaperData::new(&harf_font);
        let shaper = shaper_data.shaper(&harf_font).build();
        let mut character_buffer = [0; 4];
        let character_text = character.encode_utf8(&mut character_buffer);
        let shaped = shape_text_glyphs(
            &shaper,
            character_text,
            disable_legacy_kern_for_advances(&skrifa_font),
        );
        if let Some(glyph) = shaped.into_iter().find(|glyph| glyph.glyph_id != 0) {
            return Some((fallback.clone(), glyph));
        }
    }
    None
}

#[allow(clippy::arithmetic_side_effects)]
fn standalone_break_lines(
    text: &str,
    glyphs: &[StandaloneGlyph],
    sizing: TextSizing,
    max_width: f32,
) -> Vec<StandaloneLine> {
    let mut lines = Vec::new();
    let chars = text.chars().collect::<Vec<_>>();
    let mut paragraph_start = 0usize;
    let mut paragraph = 0usize;
    for paragraph_end in chars
        .iter()
        .enumerate()
        .filter_map(|(index, character)| (*character == '\n').then_some(index))
        .chain(std::iter::once(chars.len()))
    {
        let paragraph_glyphs = glyphs
            .iter()
            .filter(|glyph| glyph.char_index >= paragraph_start && glyph.char_index < paragraph_end)
            .cloned()
            .collect::<Vec<_>>();
        if paragraph_glyphs.is_empty() {
            lines.push(StandaloneLine {
                paragraph,
                char_start: paragraph_start,
                char_end: paragraph_end,
                glyphs: Vec::new(),
                width: 0.0,
                top: 0.0,
                baseline: 0.0,
                bottom: 0.0,
                start_x: 0.0,
            });
        } else if sizing == TextSizing::AutoWidth || max_width <= 0.0 {
            let width = paragraph_glyphs.iter().map(|glyph| glyph.advance).sum();
            lines.push(StandaloneLine {
                paragraph,
                char_start: paragraph_start,
                char_end: paragraph_end,
                glyphs: paragraph_glyphs,
                width,
                top: 0.0,
                baseline: 0.0,
                bottom: 0.0,
                start_x: 0.0,
            });
        } else {
            let mut start = 0usize;
            while start < paragraph_glyphs.len() {
                let mut width = 0.0;
                let mut end = start;
                let mut last_space = None;
                while let Some(glyph) = paragraph_glyphs.get(end) {
                    if width + glyph.advance > max_width && end > start {
                        break;
                    }
                    width += glyph.advance;
                    let char_index = glyph.char_index.min(chars.len().saturating_sub(1));
                    if chars.get(char_index).is_some_and(|character| character.is_whitespace()) {
                        last_space = Some(end + 1);
                    }
                    end += 1;
                }
                if end < paragraph_glyphs.len()
                    && let Some(space) = last_space.filter(|space| *space > start)
                {
                    end = space;
                }
                end = end.max(start + 1).min(paragraph_glyphs.len());
                let line_glyphs = paragraph_glyphs[start..end].to_vec();
                let width = line_glyphs.iter().map(|glyph| glyph.advance).sum();
                let char_start = line_glyphs.first().map_or(paragraph_start, |glyph| glyph.char_index);
                let char_end = line_glyphs.last().map_or(char_start, |glyph| {
                    glyph.char_index + glyph.char_len
                });
                lines.push(StandaloneLine {
                    paragraph,
                    char_start,
                    char_end,
                    glyphs: line_glyphs,
                    width,
                    top: 0.0,
                    baseline: 0.0,
                    bottom: 0.0,
                    start_x: 0.0,
                });
                start = end;
            }
        }
        paragraph_start = paragraph_end.saturating_add(1);
        paragraph = paragraph.saturating_add(1);
    }
    lines
}

#[allow(clippy::arithmetic_side_effects)]
fn standalone_measure_lines(
    lines: &mut [StandaloneLine],
    runs: &[StandaloneTextRun],
    paragraph_spacing: f32,
) {
    let mut cursor = 0.0f32;
    let mut previous_paragraph = 0usize;
    for (line_index, line) in lines.iter_mut().enumerate() {
        if line_index > 0 && line.paragraph != previous_paragraph {
            cursor += paragraph_spacing;
        }
        previous_paragraph = line.paragraph;
        let relevant = runs.iter().filter(|run| {
            let end = run.char_start + run.text.chars().count();
            line.char_start <= end && run.char_start <= line.char_end
        });
        let mut natural_ascent = 0.0f32;
        let mut adjusted_ascent = 0.0f32;
        let mut adjusted_descent = 0.0f32;
        for run in relevant {
            let Ok(font) = SkrifaFontRef::from_index(run.font.bytes(), run.font.face_index) else {
                continue;
            };
            let (ascent, descent) = harfbuzz_line_metrics(&font, LocationRef::default());
            let natural_ascent_px = ascent * run.size / TEXT_SHAPE_SCALE_F32;
            let natural_descent_px = -descent * run.size / TEXT_SHAPE_SCALE_F32;
            natural_ascent = natural_ascent.max(natural_ascent_px);
            if run.line_height < 0.0 {
                adjusted_ascent = adjusted_ascent.max(natural_ascent_px);
                adjusted_descent = adjusted_descent.max(natural_descent_px);
            } else {
                let factor = natural_ascent_px / (natural_ascent_px + natural_descent_px);
                let ascent = factor * run.line_height;
                adjusted_ascent = adjusted_ascent.max(ascent);
                adjusted_descent = adjusted_descent.max(run.line_height - ascent);
            }
        }
        line.top = cursor;
        line.baseline = if line_index == 0 {
            natural_ascent
        } else {
            cursor + adjusted_ascent
        };
        line.bottom = line.baseline + adjusted_descent;
        cursor = line.bottom;
    }
}

#[allow(clippy::arithmetic_side_effects)]
fn standalone_reorder_bidi(text: &str, lines: &mut [StandaloneLine]) {
    let bidi = unicode_bidi::BidiInfo::new(text, None);
    for line in lines {
        let byte_start = char_byte_index(text, line.char_start);
        let byte_end = char_byte_index(text, line.char_end);
        let Some(paragraph) = bidi.paragraphs.iter().find(|paragraph| {
            paragraph.range.start <= byte_start && byte_end <= paragraph.range.end
        }) else {
            continue;
        };
        let (levels, visual_runs) = bidi.visual_runs(paragraph, byte_start..byte_end);
        let original = line.glyphs.clone();
        let mut visual = Vec::with_capacity(original.len());
        for run in visual_runs {
            let mut matching = original
                .iter()
                .filter(|glyph| run.contains(&char_byte_index(text, glyph.char_index)))
                .cloned()
                .collect::<Vec<_>>();
            if levels.get(run.start).is_some_and(|level| level.is_rtl()) {
                matching.reverse();
            }
            visual.extend(matching);
        }
        if visual.len() == original.len() {
            line.glyphs = visual;
        }
    }
}

#[allow(clippy::arithmetic_side_effects)]
fn standalone_apply_ellipsis(
    lines: &mut Vec<StandaloneLine>,
    runs: &[StandaloneTextRun],
    max_width: f32,
    max_height: f32,
) {
    if lines.is_empty() {
        return;
    }
    let selected = lines
        .iter()
        .rposition(|line| line.bottom <= max_height)
        .unwrap_or(0);
    lines.truncate(selected + 1);
    let Some(line) = lines.last_mut() else {
        return;
    };
    let text = runs.iter().map(|run| run.text.as_str()).collect::<String>();
    let Some(style) = line
        .glyphs
        .last()
        .and_then(|glyph| runs.iter().find(|run| run.style_index == glyph.style_index))
        .or_else(|| runs.first())
    else {
        return;
    };
    let mut ellipsis_run = style.clone();
    ellipsis_run.text = "...".to_owned();
    ellipsis_run.char_start = line.char_end;
    let ellipsis = shape_standalone_run(&ellipsis_run);
    let ellipsis_width = ellipsis.iter().map(|glyph| glyph.advance).sum::<f32>();
    while line.glyphs.last().is_some_and(|glyph| {
        text.chars()
            .nth(glyph.char_index)
            .is_some_and(char::is_whitespace)
    }) {
        if let Some(removed) = line.glyphs.pop() {
            line.width -= removed.advance;
        }
    }
    while line.width + ellipsis_width > max_width && !line.glyphs.is_empty() {
        if let Some(removed) = line.glyphs.pop() {
            line.width -= removed.advance;
        }
    }
    line.width += ellipsis_width;
    line.glyphs.extend(ellipsis);
}

// Integrated static-text semantic helper ------------------------------------

/// Return the exact settled text rendered by one Text object.
///
/// This follows the same resolved-run path as shaping, including live string
/// property writes and dynamically projected list runs. Callers opt into this
/// allocation only for semantic text observation; ordinary geometry queries
/// do not materialize text values.
pub(crate) fn static_text_value(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
) -> Option<String> {
    let slice = StaticTextSlice::from_graph(runtime, graph, text_local).ok()?;
    let runs = slice.resolved_runs(runtime, instance).ok()?;
    Some(runs.into_iter().map(|run| run.text).collect())
}
