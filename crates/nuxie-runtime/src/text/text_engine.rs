pub(crate) fn static_text_constraint_bounds(
    _runtime: &RuntimeFile,
    _graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
) -> Option<(f32, f32, f32, f32)> {
    instance
        .component(text_local)
        .and_then(|component| component.concrete.text.as_ref())
        .and_then(|text| text.bounds())
}
/// Construct the bounds retained by `Text::buildRenderStyles` during the
/// mutable component update. Ordinary bounds readers use
/// `static_text_constraint_bounds` and never repeat this shaping work.
pub(crate) fn build_static_text_constraint_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
) -> Option<(f32, f32, f32, f32)> {
    if let Ok(slice) = StaticTextSlice::from_graph(runtime, graph, text_local) {
        let controlled = layout_constraint.and_then(|constraint| {
            slice
                .local_bounds_with_layout_constraint(runtime, instance, constraint)
                .ok()
                .flatten()
        });
        let unconstrained = slice.local_bounds(runtime, instance).ok().flatten();
        if let Some(bounds) = controlled.or(unconstrained) {
            return Some(bounds);
        }
    }
    static_fixed_text_constraint_bounds(runtime, graph, instance, text_local, None)
}
pub(crate) fn static_text_layout_measure_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: RuntimeTextLayoutConstraint,
) -> Option<(f32, f32, f32, f32)> {
    if let Ok(slice) = StaticTextSlice::from_graph(runtime, graph, text_local)
        && let Ok(Some(bounds)) =
            slice.measure_bounds_with_layout_constraint(runtime, instance, layout_constraint)
    {
        return Some(bounds);
    }
    static_fixed_text_constraint_bounds(runtime, graph, instance, text_local, None).map(
        |(_x, _y, width, height)| {
            (
                0.0,
                0.0,
                width.min(layout_constraint.width),
                height.min(layout_constraint.height),
            )
        },
    )
}
pub(crate) fn static_text_controlled_layout_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    _layout_constraint: RuntimeTextLayoutConstraint,
) -> Option<(f32, f32, f32, f32)> {
    static_text_constraint_bounds(runtime, graph, instance, text_local)
}

fn byte_index_for_glyph_end(text: &str, glyphs: &[TextGlyph], glyph_end: usize) -> usize {
    if glyph_end >= glyphs.len() {
        return text.len();
    }
    let target = (glyphs[glyph_end].cluster as usize).min(text.len());
    if text.is_char_boundary(target) {
        return target;
    }
    text.char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= target)
        .last()
        .unwrap_or(0)
}

fn first_fitting_glyph_end(
    glyphs: &[TextGlyph],
    max_width: f32,
    scale: f32,
    letter_spacing: f32,
) -> usize {
    let mut width = 0.0;
    for (index, glyph) in glyphs.iter().enumerate() {
        let advance = glyph.advance * scale + letter_spacing;
        if width + advance > max_width {
            return index.max(1);
        }
        width += advance;
    }
    glyphs.len()
}

fn apply_static_ellipsis(
    glyphs: &mut Vec<StyledTextGlyph>,
    ellipsis: Vec<StyledTextGlyph>,
    max_width: f32,
    force: bool,
) {
    // Exact `OrderedLine::buildEllipsisRuns`: the final visual line first
    // measures authored advances without reserving ellipsis room
    // (`src/text/text_engine.cpp:165-302`).
    if !force {
        let mut authored_width = 0.0f32;
        let mut fits = true;
        for glyph in glyphs.iter() {
            authored_width += glyph.advance;
            if authored_width > max_width {
                fits = false;
                break;
            }
        }
        if fits {
            return;
        }
    }

    let ellipsis_width = ellipsis.iter().map(|glyph| glyph.advance).sum::<f32>();
    let mut width = 0.0;
    let mut keep = glyphs.len();
    for (index, glyph) in glyphs.iter().enumerate() {
        if width + glyph.advance + ellipsis_width > max_width {
            keep = index;
            break;
        }
        width += glyph.advance;
    }
    if keep < glyphs.len() {
        glyphs.truncate(keep);
        glyphs.extend(ellipsis);
    } else if force {
        glyphs.extend(ellipsis);
    }
}

// ---------------------------------------------------------------------------
// Shared color-glyph classification and extraction.
//
// This lives at the text-engine seam deliberately: authored Text and the
// plain RawText owner consume the same font classification. Their replay
// policies remain different (RawText accepts solid paths only, while Text may
// decode retained bitmap images), matching the pinned C++ owners.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeColorGlyphClassification {
    Monochrome,
    Colr,
    Raster,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeColorGlyphGradientStop {
    pub offset: f32,
    pub color: u32,
    pub uses_foreground: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeColorGlyphPaint {
    Solid { color: u32 },
    LinearGradient {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        stops: Vec<RuntimeColorGlyphGradientStop>,
    },
    RadialGradient {
        x0: f32,
        y0: f32,
        r0: f32,
        x1: f32,
        y1: f32,
        r1: f32,
        stops: Vec<RuntimeColorGlyphGradientStop>,
    },
    SweepGradient {
        x0: f32,
        y0: f32,
        start_angle: f32,
        end_angle: f32,
        stops: Vec<RuntimeColorGlyphGradientStop>,
    },
    Image {
        bytes: std::sync::Arc<[u8]>,
        width: u32,
        height: u32,
        bearing_x: f32,
        bearing_y: f32,
        extent_x: f32,
        extent_y: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeColorGlyphLayer {
    pub path: nuxie_render_api::RawPath,
    pub paint: RuntimeColorGlyphPaint,
    pub uses_foreground: bool,
}

fn runtime_color_record(color: skrifa::color::Color, alpha: f32) -> u32 {
    let source_alpha = f32::from(color.alpha()) / 255.0;
    let alpha = (source_alpha * alpha).clamp(0.0, 1.0);
    let alpha = (alpha * 255.0).round() as u32;
    (alpha << 24)
        | (u32::from(color.red()) << 16)
        | (u32::from(color.green()) << 8)
        | u32::from(color.blue())
}

fn runtime_color_glyph_path(
    font: &SkrifaFontRef<'_>,
    glyph_id: GlyphId,
    transform: Option<skrifa::color::Transform>,
) -> nuxie_render_api::RawPath {
    let Some(outline) = font.outline_glyphs().get(glyph_id) else {
        return nuxie_render_api::RawPath::new();
    };
    let mut pen = TextOutlinePen::new(
        0.0,
        0.0,
        1.0 / TEXT_SHAPE_SCALE_F32,
        0.0,
        0.0,
        0.0,
        0.0,
        Mat2D::IDENTITY,
    );
    let settings = DrawSettings::unhinted(Size::new(TEXT_SHAPE_SCALE_F32), LocationRef::default())
        .with_path_style(PathStyle::FreeType);
    if outline.draw(settings, &mut pen).is_err() {
        return nuxie_render_api::RawPath::new();
    }
    if let Some(transform) = transform {
        // Skrifa's COLR matrix is y-up/font-unit based; TextOutlinePen has
        // already normalized to em units and flipped y for Rive's canvas.
        let mapped = Mat2D([
            transform.xx,
            -transform.yx,
            -transform.xy,
            transform.yy,
            transform.dx / TEXT_SHAPE_SCALE_F32,
            -transform.dy / TEXT_SHAPE_SCALE_F32,
        ]);
        crate::draw::transform_path_commands(&mut pen.commands, mapped);
    }
    crate::math::raw_path::runtime_raw_path_from_commands(&pen.commands)
}

struct RuntimeColorLayerCollector<'font> {
    font: &'font SkrifaFontRef<'font>,
    foreground: u32,
    palette: Vec<skrifa::color::Color>,
    transforms: Vec<skrifa::color::Transform>,
    layers: Vec<RuntimeColorGlyphLayer>,
    clip_glyph: Option<GlyphId>,
}

impl RuntimeColorLayerCollector<'_> {
    fn current_transform(&self) -> Option<skrifa::color::Transform> {
        self.transforms.last().copied()
    }

    fn solid_color(&self, palette_index: u16, alpha: f32) -> (u32, bool) {
        if palette_index == 0xffff {
            let source_alpha = ((self.foreground >> 24) & 0xff) as f32 / 255.0;
            let alpha = (source_alpha * alpha).clamp(0.0, 1.0);
            return (
                (self.foreground & 0x00ff_ffff) | (((alpha * 255.0).round() as u32) << 24),
                true,
            );
        }
        self.palette
            .get(usize::from(palette_index))
            .copied()
            .map(|color| (runtime_color_record(color, alpha), false))
            .unwrap_or((0xff00_0000, false))
    }

    fn gradient_stops(
        &self,
        stops: &[skrifa::color::ColorStop],
    ) -> (Vec<RuntimeColorGlyphGradientStop>, bool) {
        let mut any_foreground = false;
        let stops = stops
            .iter()
            .map(|stop| {
                let (color, uses_foreground) = self.solid_color(stop.palette_index, stop.alpha);
                any_foreground |= uses_foreground;
                RuntimeColorGlyphGradientStop {
                    offset: stop.offset,
                    color,
                    uses_foreground,
                }
            })
            .collect();
        (stops, any_foreground)
    }

    fn push_brush_layer(
        &mut self,
        glyph_id: GlyphId,
        brush_transform: Option<skrifa::color::Transform>,
        brush: skrifa::color::Brush<'_>,
    ) {
        let transform = brush_transform.or_else(|| self.current_transform());
        let path = runtime_color_glyph_path(self.font, glyph_id, transform);
        let (paint, uses_foreground) = match brush {
            skrifa::color::Brush::Solid {
                palette_index,
                alpha,
            } => {
                let (color, uses_foreground) = self.solid_color(palette_index, alpha);
                (RuntimeColorGlyphPaint::Solid { color }, uses_foreground)
            }
            skrifa::color::Brush::LinearGradient {
                p0,
                p1,
                color_stops,
                ..
            } => {
                let (stops, foreground) = self.gradient_stops(color_stops);
                (
                    RuntimeColorGlyphPaint::LinearGradient {
                        x0: p0.x / TEXT_SHAPE_SCALE_F32,
                        y0: -p0.y / TEXT_SHAPE_SCALE_F32,
                        x1: p1.x / TEXT_SHAPE_SCALE_F32,
                        y1: -p1.y / TEXT_SHAPE_SCALE_F32,
                        stops,
                    },
                    foreground,
                )
            }
            skrifa::color::Brush::RadialGradient {
                c0,
                r0,
                c1,
                r1,
                color_stops,
                ..
            } => {
                let (stops, foreground) = self.gradient_stops(color_stops);
                (
                    RuntimeColorGlyphPaint::RadialGradient {
                        x0: c0.x / TEXT_SHAPE_SCALE_F32,
                        y0: -c0.y / TEXT_SHAPE_SCALE_F32,
                        r0: r0 / TEXT_SHAPE_SCALE_F32,
                        x1: c1.x / TEXT_SHAPE_SCALE_F32,
                        y1: -c1.y / TEXT_SHAPE_SCALE_F32,
                        r1: r1 / TEXT_SHAPE_SCALE_F32,
                        stops,
                    },
                    foreground,
                )
            }
            skrifa::color::Brush::SweepGradient {
                c0,
                start_angle,
                end_angle,
                color_stops,
                ..
            } => {
                let (stops, foreground) = self.gradient_stops(color_stops);
                (
                    RuntimeColorGlyphPaint::SweepGradient {
                        x0: c0.x / TEXT_SHAPE_SCALE_F32,
                        y0: -c0.y / TEXT_SHAPE_SCALE_F32,
                        start_angle,
                        end_angle,
                        stops,
                    },
                    foreground,
                )
            }
        };
        self.layers.push(RuntimeColorGlyphLayer {
            path,
            paint,
            uses_foreground,
        });
    }
}

impl skrifa::color::ColorPainter for RuntimeColorLayerCollector<'_> {
    fn push_transform(&mut self, transform: skrifa::color::Transform) {
        self.transforms.push(transform);
    }

    fn pop_transform(&mut self) {
        self.transforms.pop();
    }

    fn push_clip_glyph(&mut self, glyph_id: GlyphId) {
        self.clip_glyph = Some(glyph_id);
    }

    fn push_clip_box(&mut self, _clip_box: skrifa::raw::types::BoundingBox<f32>) {}

    fn pop_clip(&mut self) {
        self.clip_glyph = None;
    }

    fn fill(&mut self, brush: skrifa::color::Brush<'_>) {
        if let Some(glyph_id) = self.clip_glyph {
            self.push_brush_layer(glyph_id, None, brush);
        }
    }

    fn fill_glyph(
        &mut self,
        glyph_id: GlyphId,
        brush_transform: Option<skrifa::color::Transform>,
        brush: skrifa::color::Brush<'_>,
    ) {
        self.push_brush_layer(glyph_id, brush_transform, brush);
    }

    fn push_layer(&mut self, _composite_mode: skrifa::color::CompositeMode) {}
}

#[doc(hidden)]
pub fn runtime_classify_color_glyph(
    font_bytes: &[u8],
    glyph_id: u32,
) -> RuntimeColorGlyphClassification {
    runtime_classify_color_glyph_face(font_bytes, 0, glyph_id)
}

#[doc(hidden)]
pub fn runtime_classify_color_glyph_face(
    font_bytes: &[u8],
    face_index: u32,
    glyph_id: u32,
) -> RuntimeColorGlyphClassification {
    let Ok(font) = SkrifaFontRef::from_index(font_bytes, face_index) else {
        return RuntimeColorGlyphClassification::Monochrome;
    };
    let glyph_id = GlyphId::new(glyph_id);
    if font.color_glyphs().get(glyph_id).is_some() {
        RuntimeColorGlyphClassification::Colr
    } else if font
        .bitmap_strikes()
        .glyph_for_size(Size::unscaled(), glyph_id)
        .is_some()
    {
        RuntimeColorGlyphClassification::Raster
    } else {
        RuntimeColorGlyphClassification::Monochrome
    }
}

#[doc(hidden)]
pub fn runtime_extract_color_glyph_layers(
    font_bytes: &[u8],
    glyph_id: u32,
    foreground: u32,
) -> Vec<RuntimeColorGlyphLayer> {
    runtime_extract_color_glyph_layers_face(font_bytes, 0, glyph_id, foreground)
}

#[doc(hidden)]
pub fn runtime_extract_color_glyph_layers_face(
    font_bytes: &[u8],
    face_index: u32,
    glyph_id: u32,
    foreground: u32,
) -> Vec<RuntimeColorGlyphLayer> {
    let Ok(font) = SkrifaFontRef::from_index(font_bytes, face_index) else {
        return Vec::new();
    };
    let glyph_id = GlyphId::new(glyph_id);
    if let Some(color_glyph) = font.color_glyphs().get(glyph_id) {
        let palette = font
            .color_palettes()
            .get(0)
            .map(|palette| palette.colors().to_vec())
            .unwrap_or_default();
        let mut collector = RuntimeColorLayerCollector {
            font: &font,
            foreground,
            palette,
            transforms: Vec::new(),
            layers: Vec::new(),
            clip_glyph: None,
        };
        let _ = color_glyph.paint(LocationRef::default(), &mut collector);
        return collector.layers;
    }

    let Some(bitmap) = font
        .bitmap_strikes()
        .glyph_for_size(Size::unscaled(), glyph_id)
    else {
        return Vec::new();
    };
    let skrifa::bitmap::BitmapData::Png(bytes) = bitmap.data else {
        return Vec::new();
    };
    let extent_x = bitmap.width as f32 / bitmap.ppem_x;
    let extent_y = bitmap.height as f32 / bitmap.ppem_y;
    let bearing_x = bitmap.bearing_x / TEXT_SHAPE_SCALE_F32
        + bitmap.inner_bearing_x / bitmap.ppem_x;
    let bearing_y = -bitmap.bearing_y / TEXT_SHAPE_SCALE_F32
        - bitmap.inner_bearing_y / bitmap.ppem_y;
    vec![RuntimeColorGlyphLayer {
        path: nuxie_render_api::RawPath::new(),
        paint: RuntimeColorGlyphPaint::Image {
            bytes: std::sync::Arc::from(bytes),
            width: bitmap.width,
            height: bitmap.height,
            bearing_x,
            bearing_y,
            extent_x,
            extent_y,
        },
        uses_foreground: false,
    }]
}
