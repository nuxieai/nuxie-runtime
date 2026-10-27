use super::{
    glyph_lookup::GlyphLookup,
    text_interface::TextInterface,
    text_modifier_group::{TextModifierGroup, TransformGlyphArg},
    text_style_background::TextStyleBackground,
    text_style_geometry::TextStyleGeometry,
    text_style_on_demand::TextStyleOnDemand,
    text_style_paint::TextStylePaint,
    text_style_reference::TextStyleReferenceHandle,
    text_value_run::TextValueRun,
    utf::Utf,
};
use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::{Core, CoreHandle},
    dirtyable::Dirtyable,
    generated::{core_registry::CoreCapabilities, text::text_base::TextBase},
    hit_info::HitInfo,
    importers::import_stack::ImportStack,
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
        layout_participant::LayoutParticipant,
    },
    math::{
        aabb::Aabb, mat2d::Mat2D, path_types::PathDirection, raw_path::RawPath,
        transform_components::TransformComponents, vec2d::Vec2D,
    },
    renderer::{BlendMode, ImageSampler, RenderPaintStyle, Renderer, to_render_raw_path},
    semantic::{semantic_provider::ResolvedSemanticData, semantic_role::SemanticRole},
    shapes::{
        paint::color::{ColorInt, color_modulate_opacity},
        paint::shape_paint_path::ShapePaintPath,
    },
    status_code::StatusCode,
    text_engine::{
        FontRef, GlyphLine, GlyphRun, OrderedLine, Paragraph, TextAlign, TextOrigin, TextOverflow,
        TextRun, TextSizing, TextTrimBottom, TextTrimTop, TextWordBreak, TextWrap,
        VerticalTextAlign, fallback_calls, fallback_state,
    },
    viewmodel::{
        symbol_type::SymbolType,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_value::{ValueDependentHandle, ViewModelInstanceValue},
        viewmodel_value_dependent::ViewModelValueDependent,
    },
};
use crate::source::viewmodel::viewmodel_value_dependent::TextStyleSourceValue;
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    rc::{Rc, Weak},
};

// Sweep gradients have no Rive shader and fall back to their first stop.
pub fn draw_color_glyph_layer(
    renderer: &mut Renderer,
    factory: &crate::mechanical_port::source::factory::RuntimeFactoryHandle,
    layer: &mut crate::mechanical_port::source::text_engine::ColorGlyphLayer,
    opacity: f32,
) {
    use crate::mechanical_port::source::text_engine::ColorGlyphPaintType as PaintType;
    let mut paint = factory.with_factory_mut(|factory| factory.make_render_paint());
    paint.style(RenderPaintStyle::Fill);
    let mut to_unit_circle = Mat2D::default();
    let is_linear = layer.paint_type == PaintType::LinearGradient;
    let is_radial = layer.paint_type == PaintType::RadialGradient
        && layer.radial_transform.invert(&mut to_unit_circle);
    if layer.stops.len() < 2 || (!is_linear && !is_radial) {
        let color = if layer.paint_type == PaintType::Solid || layer.stops.is_empty() {
            layer.color
        } else {
            layer.stops[0].color
        };
        paint.color(color_modulate_opacity(color, opacity));
        let path = factory.with_factory_mut(|factory| {
            factory.make_render_path(
                to_render_raw_path(&layer.path),
                nuxie_render_api::FillRule::NonZero,
            )
        });
        renderer.draw_path(path.as_ref(), paint.as_ref());
        return;
    }
    let colors: Vec<_> = layer
        .stops
        .iter()
        .map(|stop| color_modulate_opacity(stop.color, opacity))
        .collect();
    let stops: Vec<_> = layer.stops.iter().map(|stop| stop.offset).collect();
    if is_linear {
        let shader = factory.with_factory_mut(|factory| {
            factory.make_linear_gradient(layer.x0, layer.y0, layer.x1, layer.y1, &colors, &stops)
        });
        paint.shader(Some(shader.as_ref()));
        let path = factory.with_factory_mut(|factory| {
            factory.make_render_path(
                to_render_raw_path(&layer.path),
                nuxie_render_api::FillRule::NonZero,
            )
        });
        renderer.draw_path(path.as_ref(), paint.as_ref());
        return;
    }
    let shader = factory
        .with_factory_mut(|factory| factory.make_radial_gradient(0.0, 0.0, 1.0, &colors, &stops));
    paint.shader(Some(shader.as_ref()));
    layer.path.transform_in_place(to_unit_circle);
    renderer.save();
    renderer.transform(nuxie_render_api::Mat2D(*layer.radial_transform.values()));
    let path = factory.with_factory_mut(|factory| {
        factory.make_render_path(
            to_render_raw_path(&layer.path),
            nuxie_render_api::FillRule::NonZero,
        )
    });
    renderer.draw_path(path.as_ref(), paint.as_ref());
    renderer.restore();
}

pub(crate) struct ColorGlyphImageCacheEntry {
    pub(crate) font: FontRef,
    pub(crate) glyph_id: u16,
    pub(crate) cluster_text: Vec<u32>,
    images: Vec<(Option<Rc<dyn nuxie_render_api::RenderImage>>, Mat2D)>,
}
impl ColorGlyphImageCacheEntry {
    pub(crate) fn matches(&self, font: &FontRef, glyph_id: u16, cluster: &[u32]) -> bool {
        color_glyph_cache_key(&self.font, self.glyph_id, &self.cluster_text)
            == color_glyph_cache_key(font, glyph_id, cluster)
    }
}
pub(crate) type ColorGlyphImageCache = Vec<ColorGlyphImageCacheEntry>;

// Glyph-drawing hosts receive text only as context, so their decoded picture
// remains the same across clusters. Cluster-drawing hosts retain the whole key.
pub(crate) fn color_glyph_cache_key<'a>(
    font: &FontRef,
    glyph_id: u16,
    cluster: &'a [u32],
) -> (usize, u16, &'a [u32]) {
    let cluster = if font.host_picture_scope()
        == Some(crate::mechanical_port::source::text_engine::HostPictureScope::Glyph)
    {
        &[]
    } else {
        cluster
    };
    (
        std::sync::Arc::as_ptr(font) as *const () as usize,
        glyph_id,
        cluster,
    )
}

fn draw_cached_images(
    renderer: &mut Renderer,
    images: &[(Option<Rc<dyn nuxie_render_api::RenderImage>>, Mat2D)],
    opacity: f32,
) {
    for (image, placement) in images {
        let Some(image) = image else { continue };
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(*placement.values()));
        renderer.draw_image(
            Some(image.as_ref()),
            ImageSampler::LINEAR_CLAMP,
            BlendMode::SrcOver,
            opacity,
        );
        renderer.restore();
    }
}

// Shared by Text, RawText and the additive text-field color-glyph route.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_color_glyph(
    renderer: &mut Renderer,
    factory: &crate::mechanical_port::source::factory::RuntimeFactoryHandle,
    image_cache: &mut ColorGlyphImageCache,
    font: FontRef,
    glyph_id: u16,
    cluster: &[u32],
    transform: Mat2D,
    foreground_color: ColorInt,
    opacity: f32,
    world_transform: Mat2D,
) {
    draw_color_glyph_with_factory(
        renderer,
        || factory.clone(),
        image_cache,
        font,
        glyph_id,
        cluster,
        transform,
        foreground_color,
        opacity,
        world_transform,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_color_glyph_with_factory(
    renderer: &mut Renderer,
    factory: impl FnOnce() -> crate::mechanical_port::source::factory::RuntimeFactoryHandle,
    image_cache: &mut ColorGlyphImageCache,
    font: FontRef,
    glyph_id: u16,
    cluster: &[u32],
    transform: Mat2D,
    foreground_color: ColorInt,
    opacity: f32,
    world_transform: Mat2D,
) {
    if let Some(cached) = image_cache
        .iter()
        .find(|entry| entry.matches(&font, glyph_id, cluster))
    {
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(
            *(world_transform * transform).values(),
        ));
        draw_cached_images(renderer, &cached.images, opacity);
        renderer.restore();
        return;
    }
    let mut layers = Vec::new();
    if font.get_color_layers_with_text(glyph_id, cluster, &mut layers, foreground_color) == 0 {
        return;
    }
    // A glyph with no drawable layers needs no factory (for example CBDT format 19).
    let factory = factory();
    renderer.save();
    renderer.transform(nuxie_render_api::Mat2D(
        *(world_transform * transform).values(),
    ));
    let only_images = layers.iter().all(|layer| {
        layer.paint_type == crate::mechanical_port::source::text_engine::ColorGlyphPaintType::Image
    });
    let mut images = Vec::new();
    for mut layer in layers {
        if layer.paint_type
            == crate::mechanical_port::source::text_engine::ColorGlyphPaintType::Image
        {
            let image = factory
                .with_factory_mut(|factory| factory.decode_image(&layer.image_bytes))
                .ok()
                .map(Rc::<dyn nuxie_render_api::RenderImage>::from);
            let placement = Mat2D::new(
                layer.image_extent_x / layer.image_width as f32,
                0.0,
                0.0,
                layer.image_extent_y / layer.image_height as f32,
                layer.image_bearing_x,
                layer.image_bearing_y,
            );
            images.push((image, placement));
            draw_cached_images(renderer, &images[images.len() - 1..], opacity);
        } else {
            draw_color_glyph_layer(renderer, &factory, &mut layer, opacity);
        }
    }
    if only_images {
        let cluster_text = color_glyph_cache_key(&font, glyph_id, cluster).2.to_vec();
        image_cache.push(ColorGlyphImageCacheEntry {
            font,
            glyph_id,
            cluster_text,
            images,
        });
    }
    renderer.restore();
}

#[derive(Default)]
pub struct StyledText {
    value: Vec<u32>,
    runs: Vec<TextRun>,
    css_strut: Option<TextRun>,
    css_indices: Option<crate::css_text::TextIndexMap>,
}
impl StyledText {
    pub fn empty(&self) -> bool {
        self.runs.is_empty()
    }
    pub fn clear(&mut self) {
        self.value.clear();
        self.runs.clear();
        self.css_strut = None;
        self.css_indices = None;
    }
    pub fn append(
        &mut self,
        font: FontRef,
        size: f32,
        line_height: f32,
        letter_spacing: f32,
        text: impl AsRef<[u8]>,
        style_id: u16,
    ) {
        let start = self.value.len();
        let mut bytes = text.as_ref();
        // The slice bounds decoding just as upstream's data/end pointers do;
        // a NUL lead byte still terminates the authored text.
        while bytes.first().is_some_and(|byte| *byte != 0) {
            self.value.push(Utf::next_utf8(&mut bytes));
        }
        // Shaping takes no empty runs.
        if self.value.len() == start {
            return;
        }
        self.runs.push(TextRun {
            font: Some(font),
            size,
            line_height,
            letter_spacing,
            unichar_count: (self.value.len() - start) as u32,
            script: 0,
            style_id,
            level: 0,
        });
    }
    pub fn unichars(&self) -> &[u32] {
        &self.value
    }
    pub fn source_unichars(&self) -> &[u32] {
        self.css_indices
            .as_ref()
            .map_or(&self.value, |map| map.source())
    }
    pub fn source_to_display(&self, index: u32) -> u32 {
        self.css_indices
            .as_ref()
            .map_or(index, |map| map.to_display(index))
    }
    pub fn display_to_source(&self, index: u32) -> u32 {
        self.css_indices
            .as_ref()
            .map_or(index, |map| map.to_source(index))
    }
    pub(crate) fn css_indices(&self) -> Option<&crate::css_text::TextIndexMap> {
        self.css_indices.as_ref()
    }
    pub fn runs(&self) -> &[TextRun] {
        &self.runs
    }
    pub fn swap_runs(&mut self, other: &mut Vec<TextRun>) {
        std::mem::swap(&mut self.runs, other);
    }

    fn preserved_tabs(&self) -> &[u32] {
        self.css_indices
            .as_ref()
            .map_or(&[], |map| map.preserved_tabs())
    }
    fn shape(&self) -> Vec<Paragraph> {
        let font = self.runs[0]
            .font
            .as_ref()
            .expect("shaped text retains its font");
        if self.preserved_tabs().is_empty() {
            return font.shape_text(self.unichars(), self.runs(), 0);
        }
        let mut shaping_text = self.value.clone();
        for &tab in self.preserved_tabs() {
            shaping_text[tab as usize] = 32;
        }
        // A tab is an isolated gap, so font features and kerning cannot join
        // it to either adjacent cluster when it is shaped as a space.
        let mut shaping_runs = Vec::new();
        let mut offset = 0;
        for run in &self.runs {
            let end = offset + run.unichar_count;
            let first = self.preserved_tabs().partition_point(|&tab| tab < offset);
            let last = self.preserved_tabs().partition_point(|&tab| tab < end);
            for &tab in &self.preserved_tabs()[first..last] {
                if tab > offset {
                    shaping_runs.push(TextRun {
                        unichar_count: tab - offset,
                        ..run.clone()
                    });
                }
                shaping_runs.push(TextRun {
                    unichar_count: 1,
                    ..run.clone()
                });
                offset = tab + 1;
            }
            if offset < end {
                shaping_runs.push(TextRun {
                    unichar_count: end - offset,
                    ..run.clone()
                });
            }
            offset = end;
        }
        let mut paragraphs = font.shape_text(&shaping_text, &shaping_runs, 0);
        crate::css_text::prepare_tabs(&mut paragraphs, self.css_indices.as_ref().unwrap());
        paragraphs
    }

    /// Whether shaping this reads exactly what shaping `unichars` over `runs`
    /// reads: the same code points and runs, fonts by identity and sizes by bits.
    fn shapes_like(&self, unichars: &[u32], runs: &[TextRun], tabs: &[u32]) -> bool {
        if self.preserved_tabs() != tabs {
            return false;
        }
        self.value == unichars
            && self.runs.len() == runs.len()
            && self.runs.iter().zip(runs).all(|(a, b)| {
                let same_font = match (&a.font, &b.font) {
                    (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
                    (None, None) => true,
                    _ => false,
                };
                same_font
                    && a.size.to_bits() == b.size.to_bits()
                    && a.line_height.to_bits() == b.line_height.to_bits()
                    && a.letter_spacing.to_bits() == b.letter_spacing.to_bits()
                    && a.unichar_count == b.unichar_count
                    && a.script == b.script
                    && a.style_id == b.style_id
                    && a.level == b.level
            })
    }
}

/// The paragraphs Text::measure last shaped and what it shaped them from.
/// Layout measures a text several times per solve with the same styled text;
/// reshaping it would produce these same paragraphs.
struct MeasuredShape {
    unichars: Vec<u32>,
    // Preserved tabs affect fixed advances and glyph suppression.
    css_tabs: Vec<u32>,
    runs: Vec<TextRun>,
    // fallback_state() when shaped. A shaping that called the fallback is not
    // kept: what the callback returns may change.
    fallback: (bool, bool),
    shape: Vec<Paragraph>,
}
pub struct TextBoundsInfo {
    pub min_y: f32,
    pub max_width: f32,
    pub total_height: f32,
    pub ellipsis_line: i32,
    pub is_ellipsis_line_last: bool,
    pub top_trim: f32,
    pub bottom_trim: f32,
}
#[repr(u8)]
pub enum LineIter {
    DrawLine,
    SkipThisLine,
    YOutOfBounds,
}

#[derive(Clone)]
pub enum TextValueRunHandle {
    Core(CoreHandle),
    Runtime(Rc<RefCell<TextValueRun>>),
}

impl TextValueRunHandle {
    pub(crate) fn with<R>(&self, use_run: impl FnOnce(&TextValueRun) -> R) -> Option<R> {
        match self {
            Self::Core(run) => run
                .with(|run| run.as_text_value_run().map(use_run))
                .flatten(),
            Self::Runtime(run) => Some(use_run(&run.borrow())),
        }
    }

    pub(super) fn with_mut<R>(&self, use_run: impl FnOnce(&mut TextValueRun) -> R) -> Option<R> {
        match self {
            Self::Core(run) => run
                .with_mut(|run| run.as_text_value_run_mut().map(use_run))
                .flatten(),
            Self::Runtime(run) => Some(use_run(&mut run.borrow_mut())),
        }
    }
}

pub struct TextValueRunProperty {
    text_value_run: TextValueRunHandle,
    text: CoreHandle,
    instance_value: CoreHandle,
    instance: CoreHandle,
    property_key: u16,
    symbol_type: SymbolType,
}

impl TextValueRunProperty {
    fn new(
        text_value_run: TextValueRunHandle,
        text: CoreHandle,
        instance_value: CoreHandle,
        instance: CoreHandle,
        property_key: u16,
        symbol_type: SymbolType,
    ) -> Self {
        Self {
            text_value_run,
            text,
            instance_value,
            instance,
            property_key,
            symbol_type,
        }
    }

    fn write_value(&mut self) {
        self.write_value_with_text(None);
    }

    fn write_value_with_text(&mut self, mut text: Option<&mut Text>) {
        match self.symbol_type {
            SymbolType::TextContent => {
                let Some(value) = self
                    .instance_value
                    .with_downcast::<ViewModelInstanceString, _>(|value| {
                        value.base.property_value().to_owned()
                    })
                else {
                    return;
                };
                if let Some(text) = text.as_deref_mut() {
                    self.text_value_run
                        .with_mut(|run| run.set_bound_text_with_borrowed_text(value, text));
                } else {
                    self.text_value_run
                        .with_mut(|run| run.set_bound_text(value));
                }
            }
            SymbolType::TextStyle | SymbolType::GlobalTextStyle => {
                // Initial list construction holds Text; its occurrence dispatcher
                // resolves both symbols immediately after releasing that borrow.
                if text.is_none() {
                    let style = Text::resolve_list_run_style_occurrence(&self.text, &self.instance);
                    self.text_value_run
                        .with_mut(|run| run.set_style_reference(style));
                }
            }
            _ => {}
        }
    }
}

impl Dirtyable for TextValueRunProperty {
    fn add_dirt(&mut self, _value: ComponentDirt, _recurse: bool) {
        self.write_value();
        self.text.with_mut(|text| {
            if let Some(text) = text.as_text_mut() {
                text.mark_shape_dirty();
            }
        });
    }
}

impl ViewModelValueDependent for TextValueRunProperty {
    fn relink_data_bind(&mut self) {}
    fn add_dirt_from_text_style_source(
        &mut self,
        _value: ComponentDirt,
        _recurse: bool,
        source: &CoreHandle,
        current: &TextStyleSourceValue,
    ) {
        match self.symbol_type {
            SymbolType::TextContent if source == &self.instance_value => {
                if let TextStyleSourceValue::String(value) = current {
                    self.text_value_run
                        .with_mut(|run| run.set_bound_text(value.clone()));
                }
            }
            SymbolType::TextStyle | SymbolType::GlobalTextStyle => {
                let style = Text::resolve_list_run_style_with_source(
                    &self.text,
                    &self.instance,
                    Some((source, current)),
                );
                self.text_value_run
                    .with_mut(|run| run.set_style_reference(style));
            }
            _ => self.write_value(),
        }
        self.text.with_mut(|text| {
            if let Some(text) = text.as_text_mut() {
                text.mark_shape_dirty();
            }
        });
    }
}

pub struct TextValueRunListener {
    text_value_run: Rc<RefCell<TextValueRun>>,
    instance: CoreHandle,
    text: CoreHandle,
    properties: Vec<Rc<RefCell<dyn ViewModelValueDependent>>>,
}

impl TextValueRunListener {
    fn new(
        text_value_run: TextValueRun,
        instance: CoreHandle,
        text: CoreHandle,
        text_owner: &mut Text,
    ) -> Box<Self> {
        let text_value_run = Rc::new(RefCell::new(text_value_run));
        text_value_run.borrow_mut().set_text_component(text.clone());
        let mut listener = Box::new(Self {
            text_value_run,
            instance,
            text,
            properties: Vec::new(),
        });
        listener.create_properties_with_text(Some(text_owner));
        listener
    }

    fn mark_dirty(&mut self) {
        self.text.with_mut(|text| {
            if let Some(text) = text.as_text_mut() {
                text.mark_shape_dirty();
            }
        });
    }

    fn text_value_run(&self) -> TextValueRunHandle {
        TextValueRunHandle::Runtime(Rc::clone(&self.text_value_run))
    }

    fn remap(&mut self, instance: CoreHandle, text: &mut Text) {
        if self.instance != instance {
            self.properties.clear();
            self.instance = instance;
            self.create_properties_with_text(Some(text));
        }
    }

    fn create_properties_with_text(&mut self, mut text: Option<&mut Text>) {
        self.properties.clear();
        self.create_property_listener(SymbolType::TextStyle, text.as_deref_mut());
        self.create_property_listener(SymbolType::GlobalTextStyle, text.as_deref_mut());
        self.create_property_listener(SymbolType::TextContent, text.as_deref_mut());
    }

    fn create_single_property_listener(
        &mut self,
        symbol_type: SymbolType,
    ) -> Option<TextValueRunProperty> {
        let property_key = match symbol_type {
            SymbolType::TextStyle | SymbolType::GlobalTextStyle => {
                crate::mechanical_port::source::generated::text::text_value_run_base::TextValueRunBase::STYLE_ID_PROPERTY_KEY
            }
            SymbolType::TextContent => {
                crate::mechanical_port::source::generated::text::text_value_run_base::TextValueRunBase::TEXT_PROPERTY_KEY
            }
            _ => 0,
        };
        let instance_value = self
            .instance
            .with(|instance| {
                instance
                    .as_view_model_instance()?
                    .property_value_for_symbol(symbol_type)
            })
            .flatten()?;
        let correct_type = instance_value.with(|value| match symbol_type {
            SymbolType::TextContent | SymbolType::TextStyle => value.as_any().downcast_ref::<ViewModelInstanceString>().is_some(),
            SymbolType::GlobalTextStyle => value.as_any().downcast_ref::<crate::source::viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard>().is_some(),
            _ => false,
        }).unwrap_or(false);
        if !correct_type {
            return None;
        }
        Some(TextValueRunProperty::new(
            self.text_value_run(),
            self.text.clone(),
            instance_value,
            self.instance.clone(),
            property_key,
            symbol_type,
        ))
    }

    fn create_property_listener(&mut self, symbol_type: SymbolType, text: Option<&mut Text>) {
        let Some(listener) = self.create_single_property_listener(symbol_type) else {
            return;
        };
        let instance_value = listener.instance_value.clone();
        let listener = Rc::new(RefCell::new(listener));
        listener.borrow_mut().write_value_with_text(text);
        let dependent: Rc<RefCell<dyn ViewModelValueDependent>> = listener;
        instance_value.with_mut(|instance_value| {
            if let Some(instance_value) = instance_value.as_view_model_instance_value_mut() {
                instance_value.add_dependent(ValueDependentHandle::runtime(&dependent));
            }
        });
        self.properties.push(dependent);
    }
}

enum TextDrawCommand {
    Style(CoreHandle),
    SharedStyle(CoreHandle, Rc<RefCell<TextStyleGeometry>>),
    ColorGlyph {
        font: FontRef,
        glyph_id: u16,
        cluster_text: std::sync::Arc<[u32]>,
        transform: Mat2D,
        foreground_color: ColorInt,
        opacity: f32,
    },
}

/// Nuxie addition: adds decoration lines to a style's glyph path, registering
/// the style for drawing when it has no glyph path yet.
fn add_decoration(
    render_styles: &mut Vec<CoreHandle>,
    draw_commands: &mut Vec<TextDrawCommand>,
    shared_geometry: &mut Vec<(CoreHandle, Rc<RefCell<TextStyleGeometry>>)>,
    render_opacity: f32,
    style: &CoreHandle,
    path: &RawPath,
    opacity: f32,
) {
    if path.empty() {
        return;
    }
    if style
        .with_downcast::<TextStylePaint, _>(TextStylePaint::is_shared)
        .unwrap_or(false)
    {
        let geometry =
            if let Some((_, geometry)) = shared_geometry.iter().find(|(paint, _)| paint == style) {
                geometry.clone()
            } else {
                let geometry = Rc::new(RefCell::new(TextStyleGeometry::default()));
                shared_geometry.push((style.clone(), geometry.clone()));
                geometry
            };
        let first = geometry.borrow_mut().add_path(path, opacity);
        if first {
            draw_commands.push(TextDrawCommand::SharedStyle(style.clone(), geometry));
        }
        return;
    }
    let first_path = style
        .with_downcast_mut::<TextStylePaint, _>(|paint| paint.add_path(path, opacity))
        .unwrap_or(false);
    if first_path {
        render_styles.push(style.clone());
        style.with_downcast_mut::<TextStylePaint, _>(|paint| {
            paint.paints.propagate_opacity(render_opacity);
        });
        draw_commands.push(TextDrawCommand::Style(style.clone()));
    }
}

fn compute_vertical_trim(
    lines: &[Vec<GlyphLine>],
    shape: &[Paragraph],
    trim_top: TextTrimTop,
    trim_bottom: TextTrimBottom,
) -> (f32, f32) {
    let mut top_trim = 0.0f32;
    let mut bottom_trim = 0.0f32;
    if lines.is_empty() || (trim_top == TextTrimTop::None && trim_bottom == TextTrimBottom::None) {
        return (top_trim, bottom_trim);
    }

    if trim_top != TextTrimTop::None {
        if let Some(first_line) = lines.first().and_then(|paragraph| paragraph.first()) {
            let first_paragraph = &shape[0];
            let mut edge_px = 0.0f32;
            for run_index in first_line.start_run_index..=first_line.end_run_index {
                let run = &first_paragraph.runs[run_index as usize];
                let metrics = run
                    .font
                    .as_ref()
                    .expect("shaped text retains its font")
                    .line_metrics();
                let edge = if trim_top == TextTrimTop::Cap {
                    metrics.cap_height
                } else {
                    metrics.x_height
                };
                edge_px = edge_px.max(-edge * run.size);
            }
            top_trim = ((first_line.baseline - edge_px) - first_line.top).max(0.0);
        }
    }

    if trim_bottom != TextTrimBottom::None {
        for paragraph_index in (0..lines.len()).rev() {
            let Some(last_line) = lines[paragraph_index].last() else {
                continue;
            };
            let descent_band = last_line.bottom - last_line.baseline;
            if trim_bottom == TextTrimBottom::Alphabetic {
                bottom_trim = descent_band.max(0.0);
            } else {
                let paragraph = &shape[paragraph_index];
                let mut descent_px = 0.0f32;
                for run_index in last_line.start_run_index..=last_line.end_run_index {
                    let run = &paragraph.runs[run_index as usize];
                    descent_px = descent_px.max(
                        run.font
                            .as_ref()
                            .expect("shaped text retains its font")
                            .line_metrics()
                            .descent
                            * run.size,
                    );
                }
                bottom_trim = (descent_band - descent_px).max(0.0);
            }
            break;
        }
    }
    (top_trim, bottom_trim)
}

impl std::ops::Deref for Text {
    type Target = TextBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Text {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

crate::mechanical_port::source::transform_component::impl_transform_update!(
    Text,
    |owner, dirt| {
        crate::mechanical_port::source::transform_component::update_transform_super::<Self>(
            owner, dirt,
        );
        owner.with_downcast_mut::<Self, _>(|object| object.mark_listened_runs());
        Self::sync_style_list_items_occurrence(owner);
        owner.with_downcast_mut::<Self, _>(|object| object.update_after_transform_super(dirt));
    },
    crate::mechanical_port::source::transform_component::update_local_transform::<Self>,
    crate::mechanical_port::source::node::Node::update_world_transform_occurrence::<Self>,
    |object: &mut Self| {
        if !object.try_compose_world_transform_override() {
            crate::mechanical_port::source::transform_component::compose_world_transform(object);
        }
    },
    crate::mechanical_port::source::transform_component::update_participant_constraints::<Self>
);

impl Text {
    pub fn painted_world_bounds(
        &mut self,
        out: &mut crate::mechanical_port::source::math::aabb::Aabb,
    ) -> crate::mechanical_port::source::drawable::BoundsFidelity {
        let fidelity =
            crate::mechanical_port::source::drawable::Drawable::painted_bounds_from_local(
                &self.local_bounds(),
                &self.base.world_transform(),
                None,
                out,
            );
        if fidelity == crate::mechanical_port::source::drawable::BoundsFidelity::None {
            fidelity
        } else {
            crate::mechanical_port::source::drawable::BoundsFidelity::Approximate
        }
    }
    pub const TYPE_KEY: u16 = TextBase::TYPE_KEY;
}

// File-version opt-ins travel through import and clone. In-memory text is current.
const FILE_FEATURE_LAYOUT_SIZES_BOX: u8 = 1 << 0;
const FILE_FEATURE_FIT_FONT_SIZE_RESIZES_BOX: u8 = 1 << 1;
const FILE_FEATURES_ALL: u8 =
    FILE_FEATURE_LAYOUT_SIZES_BOX | FILE_FEATURE_FIT_FONT_SIZE_RESIZES_BOX;

pub struct Text {
    pub base: TextBase,
    pub internal_transform: Mat2D,
    // The actual C++ m_shapeWorldTransform field is read by paint callbacks
    // while Text's other state is borrowed. There is no second matrix copy.
    shape_world_transform: Rc<Cell<Mat2D>>,
    runs: Vec<CoreHandle>,
    all_runs: Vec<TextValueRunHandle>,
    render_styles: Vec<CoreHandle>,
    shared_geometry: Vec<(CoreHandle, Rc<RefCell<TextStyleGeometry>>)>,
    global_backgrounds: Vec<CoreHandle>,
    on_demand_styles: Vec<Rc<RefCell<TextStyleOnDemand>>>,
    listened_runs_dirty: bool,
    list_items_dirty: bool,
    file: Option<crate::source::file::RuntimeFileWeakHandle>,
    pub(super) shape: Vec<Paragraph>,
    modifier_shape: Vec<Paragraph>,
    pub(super) lines: Vec<Vec<GlyphLine>>,
    modifier_lines: Vec<Vec<GlyphLine>>,
    pub(super) ordered_lines: Vec<OrderedLine>,
    ellipsis_run: GlyphRun,
    clip_rect: RawPath,
    clip_path: ShapePaintPath,
    bounds: Aabb,
    /// Nuxie addition: the logical line boxes of the lines the last
    /// `build_render_styles` drew, for live moves (docs/live-moves.md).
    lines_bounds: Option<Aabb>,
    // Last fitFontSize multiplier. Read paragraph spacing through the guarded
    // helper so switching overflow modes cannot reuse a stale fitted gap.
    fitted_font_scale: f32,
    modifier_groups: Vec<CoreHandle>,
    pub(super) styled_text: StyledText,
    pub(super) selection_controller:
        std::rc::Weak<RefCell<super::text_selection_controller::SelectionState>>,
    modifier_styled_text: StyledText,
    glyph_lookup: GlyphLookup,
    text_style_paints: Vec<CoreHandle>,
    layout_width: f32,
    layout_height: f32,
    layout_width_scale_type: u8,
    layout_height_scale_type: u8,
    layout_direction: LayoutDirection,
    /// Our layer: a CSS box lays this text out (docs/CSS_LAYOUT.md, 5), so
    /// layout owns its width and uses CSS line boxes.
    css_owned: bool,
    file_features: u8,
    emoji_image_cache: ColorGlyphImageCache,
    draw_commands: Vec<TextDrawCommand>,
    value_run_listeners: Vec<Box<TextValueRunListener>>,
    measured_shape: Option<Box<MeasuredShape>>,
    // Last layout measurement and last update shape, using the same source
    // inputs. Initially unequal so never-measured text still notifies layout.
    measured_key: u64,
    shaped_key: u64,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            base: TextBase::default(),
            internal_transform: Mat2D::default(),
            shape_world_transform: Rc::new(Cell::new(Mat2D::default())),
            runs: Vec::new(),
            all_runs: Vec::new(),
            render_styles: Vec::new(),
            shared_geometry: Vec::new(),
            global_backgrounds: Vec::new(),
            on_demand_styles: Vec::new(),
            listened_runs_dirty: true,
            list_items_dirty: true,
            file: None,
            shape: Vec::new(),
            modifier_shape: Vec::new(),
            lines: Vec::new(),
            modifier_lines: Vec::new(),
            ordered_lines: Vec::new(),
            ellipsis_run: GlyphRun::default(),
            clip_rect: RawPath::default(),
            clip_path: ShapePaintPath::default(),
            bounds: Aabb::default(),
            lines_bounds: None,
            fitted_font_scale: 1.0,
            modifier_groups: Vec::new(),
            styled_text: StyledText::default(),
            selection_controller: std::rc::Weak::new(),
            modifier_styled_text: StyledText::default(),
            glyph_lookup: GlyphLookup::default(),
            text_style_paints: Vec::new(),
            layout_width: f32::NAN,
            layout_height: f32::NAN,
            layout_width_scale_type: u8::MAX,
            layout_height_scale_type: u8::MAX,
            layout_direction: LayoutDirection::Inherit,
            css_owned: false,
            file_features: FILE_FEATURES_ALL,
            emoji_image_cache: Vec::new(),
            draw_commands: Vec::new(),
            value_run_listeners: Vec::new(),
            measured_shape: None,
            measured_key: 0,
            shaped_key: 1,
        }
    }
}

impl Drop for Text {
    fn drop(&mut self) {
        if let Some(controller) = self.selection_controller.upgrade() {
            if let Some(handle) = self.base.handle() {
                controller.borrow_mut().remove_dropping_text(&handle);
            }
        }
        // Listeners retain their on-demand style references; release runs first.
        self.value_run_listeners.clear();
        self.all_runs.clear();
        self.on_demand_styles.clear();
    }
}

impl Text {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let major = import_stack.major_version();
        let minor = import_stack.minor_version();
        let at_least = |want_minor| major > 7 || (major == 7 && minor >= want_minor);
        self.file_features = 0;
        if at_least(3) {
            self.file_features |= FILE_FEATURE_LAYOUT_SIZES_BOX;
        }
        if at_least(4) {
            self.file_features |= FILE_FEATURE_FIT_FONT_SIZE_RESIZES_BOX;
        }
        self.base.import(import_stack)
    }

    pub fn clone(&self) -> Self {
        let mut callbacks = Self::default();
        let mut twin = self.base.clone_into(&mut callbacks);
        twin.file_features = self.file_features;
        twin.file = self.file.clone();
        twin
    }

    pub fn internal_transform(&self) -> Mat2D {
        self.internal_transform
    }

    pub fn shape_world_transform(&self) -> Mat2D {
        self.shape_world_transform.get()
    }
    // C++ exposes m_shapeWorldTransform publicly. Keep explicit writes to the
    // same field rather than publishing a mirrored callback value.
    pub fn set_shape_world_transform(&mut self, value: Mat2D) {
        self.shape_world_transform.set(value);
    }
    pub(crate) fn shape_world_transform_weak(&self) -> Weak<Cell<Mat2D>> {
        Rc::downgrade(&self.shape_world_transform)
    }
    pub(crate) fn styled_text(&self) -> &StyledText {
        &self.styled_text
    }
    pub(crate) fn css_selection_shape(&self) -> Option<super::fully_shaped_text::FullyShapedText> {
        self.styled_text.css_indices.as_ref()?;
        Some(super::fully_shaped_text::FullyShapedText::from_text(self))
    }
    pub fn unichars(&self) -> &[u32] {
        self.styled_text.unichars()
    }

    pub fn mark_shape_dirty(&mut self) {
        self.mark_shape_dirty_layout(true);
    }
    pub fn mark_shape_dirty_layout(&mut self, send_to_layout: bool) {
        self.listened_runs_dirty = true;
        self.list_items_dirty = true;
        CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
        for group in &mut self.modifier_groups {
            group.with_mut(|group| {
                if let Some(group) = group.as_text_modifier_group_mut() {
                    group.clear_range_maps();
                }
            });
        }
        CoreCapabilities::world_transform_mark_dirty(self);
        if send_to_layout {
            self.base.mark_layout_node_dirty();
        }
    }
    pub(crate) fn mark_shape_dirty_occurrence(owner: &CoreHandle, send_to_layout: bool) {
        owner.with_downcast_mut::<Self, _>(|text| {
            text.listened_runs_dirty = true;
            text.list_items_dirty = true;
        });
        crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
            owner.clone(),
        )
        .add_dirt(ComponentDirt::PATH, false);
        let groups = owner
            .with_downcast::<Text, _>(|text| text.modifier_groups.clone())
            .unwrap_or_default();
        for group in groups {
            group.with_mut(|object| {
                object
                    .as_text_modifier_group_mut()
                    .expect("TextModifierGroup")
                    .clear_range_maps();
            });
        }
        crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
            owner.clone(),
        )
        .add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
        if send_to_layout {
            owner.with_downcast_mut::<Text, _>(|text| text.base.mark_layout_node_dirty());
        }
    }
    pub fn mark_paint_dirty(&mut self) {
        CoreCapabilities::component_add_dirt(self, ComponentDirt::PAINT, false);
    }
    pub fn modifier_shape_dirty(&mut self) {
        CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
    }
    pub(crate) fn add_dirt_from_modifier_group(
        &mut self,
        value: ComponentDirt,
        active_group: &mut TextModifierGroup,
    ) -> bool {
        let Some(dirt) = self
            .as_component_mut()
            .and_then(|component| component.add_dirt_state(value))
        else {
            return false;
        };
        // Group callbacks use addDirt's default nonrecursive lane. Its onDirty
        // still receives all accumulated bits and may revisit the active group.
        self.on_dirty_with_modifier_group(dirt, Some(active_group));
        self.as_component()
            .expect("Text component")
            .notify_artboard();
        true
    }
    pub fn add_run(&mut self, run: CoreHandle) {
        self.runs.push(run.clone());
        self.all_runs.push(TextValueRunHandle::Core(run));
    }
    pub fn add_modifier_group(&mut self, group: CoreHandle) {
        self.modifier_groups.push(group);
    }
    pub fn sizing(&self) -> TextSizing {
        match self.base.sizing_value() {
            1 => TextSizing::AutoHeight,
            2 => TextSizing::Fixed,
            value @ 3.. => TextSizing::Unknown(value.into()),
            0 => TextSizing::AutoWidth,
        }
    }
    pub fn effective_sizing(&self) -> TextSizing {
        if let Some(participant) = self.layout_participant() {
            let (width_scale, height_scale) = participant
                .with_downcast::<LayoutParticipant, _>(|participant| {
                    (
                        participant.layout_width_scale_type(),
                        participant.layout_height_scale_type(),
                    )
                })
                .expect("live layout participant");
            let width_is_box = width_scale == LayoutScaleType::Fixed as u8
                || width_scale == LayoutScaleType::Fill as u8;
            let height_is_box = height_scale == LayoutScaleType::Fixed as u8
                || height_scale == LayoutScaleType::Fill as u8;
            if !width_is_box && !height_is_box {
                return self.sizing();
            }
            return if width_is_box && !height_is_box {
                TextSizing::AutoHeight
            } else {
                TextSizing::Fixed
            };
        }
        if self.layout_width_scale_type == u8::MAX
            || self.layout_width_scale_type == LayoutScaleType::Hug as u8
            || self.layout_height_scale_type == LayoutScaleType::Hug as u8
        {
            self.sizing()
        } else {
            TextSizing::Fixed
        }
    }
    pub fn overflow(&self) -> TextOverflow {
        match self.base.overflow_value() {
            1 => TextOverflow::Hidden,
            2 => TextOverflow::Clipped,
            3 => TextOverflow::Ellipsis,
            4 => TextOverflow::Fit,
            5 => TextOverflow::FitFontSize,
            value @ 6.. => TextOverflow::Unknown(value.into()),
            0 => TextOverflow::Visible,
        }
    }
    pub fn overflow_visible(&self) -> bool {
        self.overflow() == TextOverflow::Visible
    }
    pub fn text_origin(&self) -> TextOrigin {
        match self.base.origin_value() {
            1 => TextOrigin::Baseline,
            value @ 2.. => TextOrigin::Unknown(value.into()),
            0 => TextOrigin::Top,
        }
    }
    pub fn vertical_trim_top(&self) -> TextTrimTop {
        crate::mechanical_port::source::text_engine::text_trim_top(
            self.base.vertical_trim_value().into(),
        )
    }
    pub fn vertical_trim_bottom(&self) -> TextTrimBottom {
        crate::mechanical_port::source::text_engine::text_trim_bottom(
            self.base.vertical_trim_value().into(),
        )
    }
    pub fn wrap(&self) -> TextWrap {
        match self.base.wrap_value() {
            1 => TextWrap::NoWrap,
            value @ 2.. => TextWrap::Unknown(value.into()),
            0 => TextWrap::Wrap,
        }
    }
    pub fn word_break(&self) -> TextWordBreak {
        match self.base.word_break_value() {
            0 => TextWordBreak::BreakWord,
            1 => TextWordBreak::Normal,
            2 => TextWordBreak::BreakAll,
            value => TextWordBreak::Unknown(value.into()),
        }
    }
    pub fn vertical_align(&self) -> VerticalTextAlign {
        match self.base.vertical_align_value() {
            1 => VerticalTextAlign::Bottom,
            2 => VerticalTextAlign::Middle,
            value @ 3.. => VerticalTextAlign::Unknown(value.into()),
            0 => VerticalTextAlign::Top,
        }
    }
    pub fn align(&self) -> TextAlign {
        let value = match self.base.align_value() {
            1 => TextAlign::Right,
            2 => TextAlign::Center,
            value @ 3.. => TextAlign::Unknown(value.into()),
            0 => TextAlign::Left,
        };
        if self.layout_direction == LayoutDirection::Inherit || value == TextAlign::Center {
            return value;
        }
        if self.layout_direction == LayoutDirection::Ltr {
            TextAlign::Left
        } else {
            TextAlign::Right
        }
    }
    pub fn effective_width(&self) -> f32 {
        if self.layout_width.is_nan() {
            self.base.width()
        } else {
            self.layout_width
        }
    }
    pub fn effective_height(&self) -> f32 {
        if self.layout_height.is_nan() {
            self.base.height()
        } else {
            self.layout_height
        }
    }
    pub fn overflow_as_fixed(&self) -> bool {
        self.effective_sizing() == TextSizing::Fixed || !self.layout_box_width().is_nan()
    }
    fn has_file_feature(&self, feature: u8) -> bool {
        self.file_features & feature == feature
    }
    pub fn fit_font_size_resizes_box_active(&self) -> bool {
        self.base.fit_font_size_resizes_box()
            && self.has_file_feature(FILE_FEATURE_FIT_FONT_SIZE_RESIZES_BOX)
    }
    /// Marks this text as content a CSS box lays out, or not.
    pub(crate) fn set_owned_by_css_layout(&mut self, owned: bool) {
        if self.css_owned != owned {
            self.css_owned = owned;
            self.mark_shape_dirty_layout(false);
        }
    }
    pub fn layout_owns_width(&self) -> bool {
        self.css_owned
            || self.layout_participant().is_some_and(|participant| {
                participant
                    .with_downcast::<LayoutParticipant, _>(|participant| {
                        // A participant with a CSS record is sized by it.
                        let scale = participant.layout_width_scale_type();
                        participant.css_record().is_some()
                            || scale == LayoutScaleType::Fixed as u8
                            || scale == LayoutScaleType::Fill as u8
                    })
                    .expect("live layout participant")
            })
    }
    pub fn layout_box_width(&self) -> f32 {
        if self.has_file_feature(FILE_FEATURE_LAYOUT_SIZES_BOX) {
            self.layout_width
        } else {
            f32::NAN
        }
    }
    pub fn layout_box_height(&self) -> f32 {
        if self.has_file_feature(FILE_FEATURE_LAYOUT_SIZES_BOX) {
            self.layout_height
        } else {
            f32::NAN
        }
    }
    pub fn add_style_paint(&mut self, paint: CoreHandle) {
        self.text_style_paints.push(paint);
    }
    pub fn style_from_shaper_id(&self, id: u16) -> Option<CoreHandle> {
        self.all_runs
            .get(id as usize)
            .and_then(|run| run.with(TextValueRun::style).flatten())
    }
    pub fn runs(&self) -> &[TextValueRunHandle] {
        &self.all_runs
    }
    /// Return the exact settled value represented by this Text's resolved runs.
    ///
    /// Unlike inferred accessibility semantics, an empty value is still a
    /// valid Text value. Host catalogues use this projection to preserve the
    /// same empty-string observation as the upstream runtime.
    pub(crate) fn settled_text_value(&self) -> String {
        self.all_runs
            .iter()
            .filter_map(|run| run.with(|run| run.base.text().to_owned()))
            .collect()
    }
    pub(crate) fn inferred_semantic_data(&self) -> Option<ResolvedSemanticData> {
        let label = self.settled_text_value();
        (!label.is_empty()).then_some(ResolvedSemanticData {
            has_semantics: true,
            role: SemanticRole::Text as u32,
            label,
        })
    }
    pub fn have_modifiers(&self) -> bool {
        !self.modifier_groups.is_empty()
    }
    pub fn text_style_paints(&self) -> &[CoreHandle] {
        &self.text_style_paints
    }
    pub fn ordered_lines(&self) -> &[OrderedLine] {
        &self.ordered_lines
    }
    // Upstream TESTING accessor, shared with external translated test crates.
    pub fn shape(&self) -> &[Paragraph] {
        &self.shape
    }
    pub fn make_styled(
        &mut self,
        styled: &mut StyledText,
        with_modifiers: bool,
        font_scale: f32,
    ) -> bool {
        styled.clear();
        let mut css_options = Vec::new();
        for (run_index, run) in self.all_runs.iter().enumerate() {
            let Some((style, text)) = run.with(|run| (run.style(), run.base.text().to_owned()))
            else {
                continue;
            };
            let Some(style) = style else {
                continue;
            };
            let Some((font, font_size, line_height, letter_spacing)) = style
                .with_mut(|style| {
                    let style = style.as_text_style_mut()?;
                    Some((
                        style.font()?,
                        style.base.font_size(),
                        style.base.line_height(),
                        style.base.letter_spacing(),
                    ))
                })
                .flatten()
            else {
                continue;
            };
            // Preserve the negative font-metric sentinel; authored absolute
            // line height and letter spacing scale uniformly with the font.
            let line_height = if line_height >= 0.0 {
                line_height * font_scale
            } else {
                line_height
            };
            // The first authored style is the element's strut, even when
            // it has no characters or shaping splits its font into fallbacks.
            if self.css_owned && styled.css_strut.is_none() {
                styled.css_strut = Some(TextRun {
                    font: Some(font.clone()),
                    size: font_size * font_scale,
                    line_height,
                    letter_spacing: 0.0,
                    unichar_count: 0,
                    script: 0,
                    style_id: run_index as u16,
                    level: 0,
                });
            }
            if text.is_empty() {
                continue;
            }
            styled.append(
                font,
                font_size * font_scale,
                line_height,
                letter_spacing * font_scale,
                &text,
                run_index as u16,
            );
            if self.css_owned {
                css_options.resize(styled.value.len(), crate::css_text::style_options(&style));
            }
        }
        let has_css = css_options.iter().any(Option::is_some);
        if has_css {
            let (value, map) = crate::css_text::display_text(&styled.value, &css_options);
            let mut source_offset = 0;
            for run in &mut styled.runs {
                let end = source_offset + run.unichar_count;
                run.unichar_count = map.to_display(end) - map.to_display(source_offset);
                source_offset = end;
            }
            styled.runs.retain(|run| run.unichar_count != 0);
            styled.value = value;
            styled.css_indices = Some(map);
        }
        let display_lengths = has_css.then(|| {
            let mut lengths = vec![0; self.all_runs.len()];
            for run in &styled.runs {
                lengths[run.style_id as usize] += run.unichar_count;
            }
            lengths
        });
        for (i, run) in self.all_runs.iter().enumerate() {
            run.with_mut(|run| {
                run.set_display_length(display_lengths.as_ref().map(|lengths| lengths[i]))
            });
        }
        if with_modifiers {
            for group in self.modifier_groups.clone() {
                group.with_mut(|group| {
                    if let Some(group) = group.as_text_modifier_group_mut() {
                        group.apply_shape_modifiers(self, styled);
                    }
                });
            }
        }
        !styled.empty()
    }
    pub fn break_lines(
        paragraphs: &[Paragraph],
        width: f32,
        align: TextAlign,
        wrap: TextWrap,
        word_break: TextWordBreak,
    ) -> Vec<Vec<GlyphLine>> {
        Self::break_lines_aligned(paragraphs, width, align, wrap, word_break, 0.0)
    }
    pub fn break_lines_aligned(
        paragraphs: &[Paragraph],
        width: f32,
        align: TextAlign,
        wrap: TextWrap,
        word_break: TextWordBreak,
        min_align_width: f32,
    ) -> Vec<Vec<GlyphLine>> {
        Self::break_lines_with_strut(
            paragraphs,
            width,
            align,
            wrap,
            word_break,
            min_align_width,
            None,
            None,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn break_lines_with_strut(
        paragraphs: &[Paragraph],
        width: f32,
        align: TextAlign,
        wrap: TextWrap,
        word_break: TextWordBreak,
        min_align_width: f32,
        css_strut: Option<&TextRun>,
        css_indices: Option<&crate::css_text::TextIndexMap>,
    ) -> Vec<Vec<GlyphLine>> {
        let mut lines = Vec::with_capacity(paragraphs.len());
        for paragraph in paragraphs {
            let mut paragraph_lines = GlyphLine::break_lines_with_word_break(
                &paragraph.runs,
                if width == -1.0 || wrap == TextWrap::NoWrap {
                    -1.0
                } else {
                    width
                },
                word_break,
            );
            if let Some(map) = css_indices {
                for line in &mut paragraph_lines {
                    crate::css_text::trim_line(line, &paragraph.runs, map);
                }
            }
            lines.push(paragraph_lines);
        }
        Self::space_lines(paragraphs, lines, width, align, min_align_width, css_strut)
    }
    fn break_styled_lines(
        width: f32,
        align: TextAlign,
        wrap: TextWrap,
        word_break: TextWordBreak,
        styled: &StyledText,
        paragraphs: &[Paragraph],
    ) -> Vec<Vec<GlyphLine>> {
        Self::break_lines_with_strut(
            paragraphs,
            width,
            align,
            wrap,
            word_break,
            0.0,
            styled.css_strut.as_ref(),
            styled.css_indices.as_ref(),
        )
    }
    fn space_lines(
        paragraphs: &[Paragraph],
        mut lines: Vec<Vec<GlyphLine>>,
        width: f32,
        align: TextAlign,
        min_align_width: f32,
        css_strut: Option<&TextRun>,
    ) -> Vec<Vec<GlyphLine>> {
        let mut paragraph_width = width;
        if width == -1.0 {
            for (paragraph, lines) in paragraphs.iter().zip(&lines) {
                paragraph_width =
                    paragraph_width.max(GlyphLine::compute_max_width(lines, &paragraph.runs));
            }
        }
        paragraph_width = paragraph_width.max(min_align_width);
        for (index, paragraph) in paragraphs.iter().enumerate() {
            if let Some(strut) = css_strut {
                GlyphLine::compute_css_line_spacing(
                    &mut lines[index],
                    &paragraph.runs,
                    paragraph_width,
                    align,
                    strut,
                );
            } else {
                GlyphLine::compute_line_spacing(
                    index == 0,
                    &mut lines[index],
                    &paragraph.runs,
                    paragraph_width,
                    align,
                );
            }
        }
        lines
    }
    pub fn modifier_ranges_need_shape(&self) -> bool {
        self.modifier_groups.iter().any(|group| {
            group
                .with(|group| {
                    group
                        .as_text_modifier_group()
                        .is_some_and(TextModifierGroup::needs_shape)
                })
                .unwrap_or(false)
        })
    }
    pub(crate) fn update_after_transform_super(&mut self, value: ComponentDirt) {
        if value.intersects(ComponentDirt::PATH) {
            let precompute_modifier_coverage = self.modifier_ranges_need_shape();
            let parent_is_layout_not_artboard = self.base.parent_handle().is_some_and(|parent| {
                parent.is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY)
                    && parent.core_type()
                        != Some(crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY)
            });
            let font_scale = if self.overflow() == TextOverflow::FitFontSize {
                self.fit_font_scale()
            } else {
                1.0
            };
            self.fitted_font_scale = font_scale;

            if precompute_modifier_coverage {
                let mut styled = std::mem::take(&mut self.modifier_styled_text);
                if self.make_styled(&mut styled, false, font_scale) {
                    self.modifier_shape = styled.shape();
                    self.modifier_lines = Self::break_styled_lines(
                        if self.effective_sizing() == TextSizing::AutoWidth
                            && !parent_is_layout_not_artboard
                        {
                            -1.0
                        } else {
                            self.effective_width()
                        },
                        self.align(),
                        self.wrap(),
                        self.word_break(),
                        &styled,
                        &self.modifier_shape,
                    );
                    self.glyph_lookup
                        .compute(styled.unichars(), &self.modifier_shape);
                    let text_size = styled.unichars().len() as u32;
                    for group in self.modifier_groups.clone() {
                        group.with_mut(|group| {
                            if let Some(group) = group.as_text_modifier_group_mut() {
                                group.compute_range_map(
                                    self,
                                    styled.unichars(),
                                    &self.modifier_shape,
                                    &self.modifier_lines,
                                    &self.glyph_lookup,
                                );
                                group.compute_coverage(text_size);
                            }
                        });
                    }
                }
                self.modifier_styled_text = styled;
            }

            let mut styled = std::mem::take(&mut self.styled_text);
            let has_styled = self.make_styled(&mut styled, true, font_scale);
            self.shaped_key = self.measure_key(&styled, font_scale);
            if has_styled {
                self.shape = match self.take_measured_shape(&styled) {
                    Some(shape) => shape,
                    None => styled.shape(),
                };
                self.lines = Self::break_styled_lines(
                    if self.effective_sizing() == TextSizing::AutoWidth
                        && !parent_is_layout_not_artboard
                    {
                        -1.0
                    } else {
                        self.effective_width()
                    },
                    self.align(),
                    self.wrap(),
                    self.word_break(),
                    &styled,
                    &self.shape,
                );
                if !precompute_modifier_coverage && !self.modifier_groups.is_empty() {
                    self.glyph_lookup.compute(styled.unichars(), &self.shape);
                    let text_size = styled.unichars().len() as u32;
                    for group in self.modifier_groups.clone() {
                        group.with_mut(|group| {
                            if let Some(group) = group.as_text_modifier_group_mut() {
                                group.compute_range_map(
                                    self,
                                    styled.unichars(),
                                    &self.shape,
                                    &self.lines,
                                    &self.glyph_lookup,
                                );
                                group.compute_coverage(text_size);
                            }
                        });
                    }
                }
            } else {
                self.shape.clear();
                self.lines.clear();
                self.glyph_lookup.clear();
            }
            self.styled_text = styled;
            self.ordered_lines.clear();
            self.ellipsis_run = GlyphRun::default();
            self.build_render_styles();
        } else if value.intersects(ComponentDirt::PAINT) {
            self.build_render_styles();
        } else if value.intersects(ComponentDirt::RENDER_OPACITY) {
            for style in &mut self.render_styles {
                style.with_downcast_mut::<TextStylePaint, _>(|style| {
                    style.paints.propagate_opacity(self.base.render_opacity());
                });
            }
            for style in &self.text_style_paints {
                if let Some(background) = style
                    .with_downcast::<TextStylePaint, _>(TextStylePaint::background)
                    .flatten()
                {
                    background.with_downcast_mut::<TextStyleBackground, _>(|background| {
                        background.propagate_opacity(self.base.render_opacity());
                    });
                }
            }
        }

        if value
            .intersects(ComponentDirt::WORLD_TRANSFORM | ComponentDirt::PATH | ComponentDirt::PAINT)
        {
            self.clip_path.rewind();
            self.set_shape_world_transform(self.base.world_transform() * self.internal_transform);
            self.clip_path
                .add_path(&self.clip_rect, Some(&self.shape_world_transform()));
        }
        if value.intersects(ComponentDirt::PATH | ComponentDirt::PAINT) {
            if let Some(controller) = self.selection_controller.upgrade() {
                controller
                    .borrow_mut()
                    .text_updated(self, value.intersects(ComponentDirt::PATH));
            }
        }
    }
    fn rebuild_ordered_lines(&mut self) {
        self.ordered_lines.clear();
        self.ellipsis_run = GlyphRun::default();
        let (mut y, mut width) = (0.0f32, 0.0f32);
        for (p, lines) in self.shape.iter().zip(&self.lines) {
            for line in lines {
                let end = &p.runs[line.end_run_index as usize];
                let start = &p.runs[line.start_run_index as usize];
                width = width.max(
                    end.xpos[line.end_glyph_index as usize]
                        - start.xpos[line.start_glyph_index as usize],
                );
                self.ordered_lines.push(
                    OrderedLine::new(
                        p,
                        line,
                        self.effective_width(),
                        false,
                        false,
                        &mut self.ellipsis_run,
                        y + line.baseline,
                    )
                    .with_css_line_box(self.css_owned),
                );
            }
            if let Some(last) = lines.last() {
                y += last.bottom;
            }
            y += self.base.paragraph_spacing();
        }
        self.bounds = Aabb::new(
            0.0,
            0.0,
            if self.effective_sizing() == TextSizing::AutoWidth {
                width
            } else {
                self.effective_width()
            },
            if self.effective_sizing() == TextSizing::Fixed {
                self.effective_height()
            } else {
                y - self.base.paragraph_spacing()
            },
        );
    }

    fn clear_render_styles(&mut self) {
        for style in &mut self.render_styles {
            style.with_downcast_mut::<TextStylePaint, _>(TextStylePaint::rewind_path);
        }
        self.render_styles.clear();
        self.draw_commands.clear();
        for (_, geometry) in &self.shared_geometry {
            geometry.borrow_mut().rewind();
        }
        for background in &self.global_backgrounds {
            background.with_downcast_mut::<TextStyleBackground, _>(TextStyleBackground::reset_path);
        }
        self.global_backgrounds.clear();

        // Reset off the child list: emoji-only styles never enter render_styles.
        self.build_text_style_paints();
        for style in &self.text_style_paints {
            if let Some(background) = style
                .with_downcast::<TextStylePaint, _>(TextStylePaint::background)
                .flatten()
            {
                background
                    .with_downcast_mut::<TextStyleBackground, _>(TextStyleBackground::reset_path);
            }
        }

        for run in &mut self.all_runs {
            run.with_mut(TextValueRun::reset_hit_test);
        }
    }

    fn fit_paragraph_spacing(&self) -> f32 {
        if self.overflow() == TextOverflow::FitFontSize {
            self.base.paragraph_spacing() * self.fitted_font_scale
        } else {
            self.base.paragraph_spacing()
        }
    }

    fn compute_bounds_info(&self) -> TextBoundsInfo {
        let paragraph_space = self.fit_paragraph_spacing();
        let mut paragraph_index = 0usize;
        let mut y = 0.0f32;
        let mut min_y = 0.0f32;
        let mut max_width = 0.0f32;
        let mut ellipsed_height = 0.0f32;
        if self.text_origin() == TextOrigin::Baseline
            && !self.lines.is_empty()
            && !self.lines[0].is_empty()
        {
            y -= self.lines[0][0].baseline;
            min_y = y;
        }

        let mut ellipsis_line = -1i32;
        let want_ellipsis = self.overflow() == TextOverflow::Ellipsis && self.overflow_as_fixed();
        let mut last_line_index = -1i32;
        for paragraph_lines in &self.lines {
            let paragraph = &self.shape[paragraph_index];
            paragraph_index += 1;
            for line in paragraph_lines {
                let end_run = &paragraph.runs[line.end_run_index as usize];
                let start_run = &paragraph.runs[line.start_run_index as usize];
                let width = end_run.xpos[line.end_glyph_index as usize]
                    - start_run.xpos[line.start_glyph_index as usize];
                max_width = max_width.max(width);
                last_line_index += 1;
                if want_ellipsis && y + line.bottom <= self.effective_height() {
                    ellipsed_height = y + line.bottom;
                    ellipsis_line += 1;
                }
            }
            if let Some(last) = paragraph_lines.last() {
                y += last.bottom;
            }
            y += paragraph_space;
        }
        if want_ellipsis && ellipsis_line == -1 {
            ellipsis_line = 0;
        }
        let total_height = if ellipsis_line > 0 {
            ellipsed_height
        } else {
            y
        };
        let is_ellipsis_line_last = last_line_index == ellipsis_line;
        let (top_trim, bottom_trim) = if self.effective_sizing() != TextSizing::Fixed {
            compute_vertical_trim(
                &self.lines,
                &self.shape,
                self.vertical_trim_top(),
                self.vertical_trim_bottom(),
            )
        } else {
            (0.0, 0.0)
        };
        TextBoundsInfo {
            min_y,
            max_width,
            total_height,
            ellipsis_line,
            is_ellipsis_line_last,
            top_trim,
            bottom_trim,
        }
    }

    fn fit_font_scale(&mut self) -> f32 {
        if self.effective_sizing() == TextSizing::AutoWidth && !self.overflow_as_fixed() {
            return 1.0;
        }
        self.fit_font_scale_in_box(
            self.effective_width(),
            if self.overflow_as_fixed() {
                self.effective_height()
            } else {
                f32::MAX
            },
        )
    }

    fn fit_font_scale_in_box(&mut self, box_width: f32, box_height: f32) -> f32 {
        // A non-fixed layout offer already excludes the vertical trim band.
        let box_is_trimmed = self.effective_sizing() != TextSizing::Fixed;
        let mut max_size = 0.0f32;
        for value_run in &self.all_runs {
            let Some((style, has_text)) =
                value_run.with(|value_run| (value_run.style(), !value_run.base.text().is_empty()))
            else {
                continue;
            };
            let Some(style) = style else {
                continue;
            };
            let Some((has_font, font_size)) = style
                .with_mut(|style| {
                    style
                        .as_text_style_mut()
                        .map(|style| (style.font().is_some(), style.base.font_size()))
                })
                .flatten()
            else {
                continue;
            };
            if has_font && has_text {
                // No scale turns infinity finite. Like std::max, ignore NaN.
                if font_size.is_infinite() {
                    return 1.0;
                }
                if max_size < font_size {
                    max_size = font_size;
                }
            }
        }
        if max_size <= 1.0 {
            return 1.0;
        }

        let mut styled = StyledText::default();
        let mut fits = |this: &mut Text, top_size: i32| -> bool {
            let scale = top_size as f32 / max_size;
            if !this.make_styled(&mut styled, true, scale) {
                return true;
            }
            let shape = styled.shape();
            let lines = Text::break_styled_lines(
                box_width,
                this.align(),
                this.wrap(),
                this.word_break(),
                &styled,
                &shape,
            );
            let mut measured_width = 0.0f32;
            let mut y = 0.0f32;
            for (paragraph, paragraph_lines) in shape.iter().zip(&lines) {
                for line in paragraph_lines {
                    let end_run = &paragraph.runs[line.end_run_index as usize];
                    let start_run = &paragraph.runs[line.start_run_index as usize];
                    measured_width = measured_width.max(
                        end_run.xpos[line.end_glyph_index as usize]
                            - start_run.xpos[line.start_glyph_index as usize],
                    );
                }
                if let Some(last) = paragraph_lines.last() {
                    y += last.bottom;
                }
                y += this.base.paragraph_spacing() * scale;
            }
            let (top_trim, bottom_trim) = if box_is_trimmed {
                compute_vertical_trim(
                    &lines,
                    &shape,
                    this.vertical_trim_top(),
                    this.vertical_trim_bottom(),
                )
            } else {
                (0.0, 0.0)
            };
            measured_width <= box_width && y - top_trim - bottom_trim <= box_height
        };
        // Above 2^24 floats skip integers, and high + 1 must not overflow i32.
        const MAX_SEARCH_SIZE: f32 = (1 << 24) as f32;
        let mut low = 1i32;
        let mut high = if MAX_SEARCH_SIZE < max_size {
            MAX_SEARCH_SIZE
        } else {
            max_size
        } as i32;
        if fits(self, high) {
            return high as f32 / max_size;
        }
        high -= 1;
        let mut best = 1i32;
        while low <= high {
            let middle = low + (high - low) / 2;
            if fits(self, middle) {
                best = middle;
                low = middle + 1;
            } else {
                high = middle - 1;
            }
        }
        best as f32 / max_size
    }

    fn should_draw_line(&self, current_y: f32, total_height: f32, line: &GlyphLine) -> LineIter {
        match self.overflow() {
            TextOverflow::Hidden if self.overflow_as_fixed() => match self.vertical_align() {
                VerticalTextAlign::Top if current_y + line.bottom > self.effective_height() => {
                    return LineIter::YOutOfBounds;
                }
                VerticalTextAlign::Middle => {
                    if current_y + line.top < total_height / 2.0 - self.effective_height() / 2.0 {
                        return LineIter::SkipThisLine;
                    }
                    if current_y + line.bottom > total_height / 2.0 + self.effective_height() / 2.0
                    {
                        return LineIter::YOutOfBounds;
                    }
                }
                VerticalTextAlign::Bottom
                    if current_y + line.top < total_height - self.effective_height() =>
                {
                    return LineIter::SkipThisLine;
                }
                _ => {}
            },
            TextOverflow::Clipped if self.overflow_as_fixed() => match self.vertical_align() {
                VerticalTextAlign::Top if current_y + line.top > self.effective_height() => {
                    return LineIter::YOutOfBounds;
                }
                VerticalTextAlign::Middle => {
                    if current_y + line.bottom < total_height / 2.0 - self.effective_height() / 2.0
                    {
                        return LineIter::SkipThisLine;
                    }
                    if current_y + line.top > total_height / 2.0 + self.effective_height() / 2.0 {
                        return LineIter::YOutOfBounds;
                    }
                }
                VerticalTextAlign::Bottom
                    if current_y + line.bottom < total_height - self.effective_height() =>
                {
                    return LineIter::SkipThisLine;
                }
                _ => {}
            },
            _ => {}
        }
        LineIter::DrawLine
    }

    pub fn build_render_styles(&mut self) {
        self.clear_render_styles();
        // Nuxie addition: every rebuild records its drawn lines afresh.
        self.lines_bounds = None;
        if self.shape.is_empty() {
            self.bounds = Aabb::new(0.0, 0.0, 0.0, 0.0);
            self.emoji_image_cache.clear();
            return;
        }

        let info = self.compute_bounds_info();
        let has_modifiers = !self.modifier_groups.is_empty();
        if has_modifiers {
            let text_size = self.styled_text.unichars().len() as u32;
            for group in self.modifier_groups.clone() {
                group.with_mut(|group| {
                    if let Some(group) = group.as_text_modifier_group_mut() {
                        group.compute_coverage(text_size);
                        group.reset_text_follow_path(self);
                    }
                });
            }
        }

        let paragraph_space = self.fit_paragraph_spacing();
        let box_width = self.layout_box_width();
        let box_height = self.layout_box_height();
        let auto_size_max_y = if box_height.is_nan() {
            info.min_y
                .max(info.total_height - paragraph_space - info.top_trim - info.bottom_trim)
        } else {
            info.min_y + box_height
        };
        self.bounds = match self.effective_sizing() {
            TextSizing::AutoWidth => Aabb::new(
                0.0,
                info.min_y,
                if box_width.is_nan() {
                    info.max_width
                } else {
                    box_width
                },
                auto_size_max_y,
            ),
            TextSizing::AutoHeight => {
                Aabb::new(0.0, info.min_y, self.effective_width(), auto_size_max_y)
            }
            TextSizing::Fixed => Aabb::new(
                0.0,
                info.min_y,
                self.effective_width(),
                info.min_y + self.effective_height(),
            ),
            TextSizing::Unknown(_) => self.bounds,
        };

        let vertical_align_offset = match self.vertical_align() {
            VerticalTextAlign::Middle => (info.total_height - self.bounds.height()) / 2.0,
            VerticalTextAlign::Bottom => info.total_height - self.bounds.height(),
            _ => 0.0,
        };
        if self.overflow() == TextOverflow::Clipped {
            self.clip_rect.rewind();
            let bounds = self.local_bounds();
            let min_x = bounds.min_x + bounds.width() * self.base.origin_x();
            let min_y =
                bounds.min_y + bounds.height() * self.base.origin_y() + vertical_align_offset;
            self.clip_rect.add_rect(
                Aabb::new(
                    min_x,
                    min_y,
                    min_x + bounds.width(),
                    min_y + bounds.height(),
                ),
                PathDirection::Clockwise,
            );
        }

        let mut current_y = info.min_y - info.top_trim;
        let mut line_index = 0i32;
        let mut minimum_x = f32::MAX;
        // Nuxie addition: the drawn lines' extent, recorded during this walk.
        let mut lines_bounds: Option<Aabb> = None;
        'paragraphs: for (paragraph, paragraph_lines) in self.shape.iter().zip(&self.lines) {
            let mut line_index_in_paragraph = 0i32;
            for line in paragraph_lines {
                match self.should_draw_line(current_y, info.total_height, line) {
                    LineIter::YOutOfBounds => break 'paragraphs,
                    LineIter::SkipThisLine => {
                        line_index_in_paragraph += 1;
                        line_index += 1;
                        continue;
                    }
                    LineIter::DrawLine => {}
                }
                let render_y = current_y + line.baseline;
                self.ordered_lines.push(
                    OrderedLine::new(
                        paragraph,
                        line,
                        self.effective_width(),
                        info.ellipsis_line == line_index,
                        info.is_ellipsis_line_last,
                        &mut self.ellipsis_run,
                        render_y,
                    )
                    .with_css_line_box(self.css_owned),
                );
                let ordered_line = self.ordered_lines.last().unwrap();
                let mut current_x = line.start_x;
                minimum_x = minimum_x.min(current_x);
                // Nuxie addition: decorated runs on this line (none upstream).
                let mut decorations = crate::text_decoration::LineDecorations::default();
                for (run, glyph_index) in ordered_line {
                    let index = glyph_index as usize;
                    let offset = run.offsets[index];
                    let glyph_id = run.glyphs[index];
                    let advance = run.advances[index];
                    let current_position = Vec2D::new(current_x, render_y);
                    let center_x = advance / 2.0;
                    let mut components = TransformComponents::default();
                    components.set_scale_x(run.size);
                    components.set_scale_y(run.size);
                    components.set_x(-center_x);
                    let mut path_transform = Mat2D::compose(&components);
                    let glyph_layout_transform = path_transform;
                    let mut opacity = 1.0f32;
                    if has_modifiers {
                        let text_index = run.text_indices[index];
                        let glyph_count = self.glyph_lookup.count(text_index);
                        for group in self.modifier_groups.clone() {
                            group.with_mut(|group| {
                                if let Some(group) = group.as_text_modifier_group_mut() {
                                    let coverage = group.glyph_coverage(text_index, glyph_count);
                                    let mut argument = TransformGlyphArg::new(
                                        current_position,
                                        center_x,
                                        line_index_in_paragraph,
                                        paragraph_lines,
                                    );
                                    group.transform(coverage, &mut path_transform, &mut argument);
                                    if group.modifies_opacity() {
                                        opacity = group.compute_opacity(opacity, coverage);
                                    }
                                }
                            });
                        }
                    }
                    // Nuxie addition: what the modifiers did to this glyph,
                    // for its decoration to follow.
                    let modifier_transform = has_modifiers
                        .then(|| path_transform * glyph_layout_transform.invert_or_identity());
                    path_transform = Mat2D::from_translate(
                        current_position.x + center_x + offset.x,
                        current_position.y + offset.y,
                    ) * path_transform;

                    let value_run = self.all_runs[run.style_id as usize].clone();
                    let style = value_run
                        .with(TextValueRun::style)
                        .flatten()
                        .expect("TextValueRun style");
                    let mut inked = false;
                    match run.glyph_render_kind(index) {
                        crate::mechanical_port::source::text_engine::GlyphRenderKind::Color => {
                        let foreground_color = style
                            .with_downcast::<TextStylePaint, _>(TextStylePaint::foreground_color)
                            .unwrap_or(0xff000000);
                        self.draw_commands.push(TextDrawCommand::ColorGlyph {
                            font: run
                                .font
                                .as_ref()
                                .expect("shaped text retains its font")
                                .clone(),
                            glyph_id,
                            cluster_text: run.cluster_text_arc(index),
                            transform: path_transform,
                            foreground_color,
                            opacity,
                        });
                        inked = true;
                        }
                        crate::mechanical_port::source::text_engine::GlyphRenderKind::Outline => {
                        let path = run
                            .font
                            .as_ref()
                            .expect("shaped text retains its font")
                            .get_path(glyph_id)
                            .transform(path_transform);
                        inked = !path.empty();
                        if style.with_downcast::<TextStylePaint, _>(TextStylePaint::is_shared).unwrap_or(false) {
                            let geometry = if let Some((_, geometry)) = self.shared_geometry.iter().find(|(paint, _)| paint == &style) {
                                geometry.clone()
                            } else {
                                let geometry = Rc::new(RefCell::new(TextStyleGeometry::default()));
                                self.shared_geometry.push((style.clone(), geometry.clone()));
                                geometry
                            };
                            let first = geometry.borrow_mut().add_path(&path, opacity);
                            if first { self.draw_commands.push(TextDrawCommand::SharedStyle(style.clone(), geometry)); }
                        } else if style.with_downcast_mut::<TextStylePaint, _>(|style| style.add_path(&path, opacity)).unwrap_or(false) {
                            self.render_styles.push(style.clone());
                            style.with_downcast_mut::<TextStylePaint, _>(|style| {
                                style.paints.propagate_opacity(self.base.render_opacity());
                            });
                            self.draw_commands
                                .push(TextDrawCommand::Style(style.clone()));
                        }
                        }
                        crate::mechanical_port::source::text_engine::GlyphRenderKind::Suppressed => {
                            // Cluster pictures carry ink; CSS tabs carry only a gap.
                            inked = !run.is_css_tab(index);
                        }
                    }
                    // Nuxie addition: the run style's underline/line-through.
                    let decoration_line = crate::text_decoration::style_line(&style);
                    let font = run.font.as_deref().expect("shaped text retains its font");
                    if let Some(modifier_transform) = modifier_transform {
                        decorations.modified_glyph(
                            &style,
                            decoration_line,
                            current_x,
                            advance,
                            inked,
                            Mat2D::from_translate(current_x + center_x, render_y)
                                * modifier_transform,
                            opacity,
                            run.size,
                            font,
                        );
                    } else {
                        decorations.glyph(
                            &style,
                            decoration_line,
                            current_x,
                            advance,
                            inked,
                            render_y,
                            run.size,
                            font,
                        );
                    }
                    let background = style
                        .with_downcast::<TextStylePaint, _>(TextStylePaint::background)
                        .flatten();
                    let is_hit_target =
                        value_run.with(TextValueRun::is_hit_target).unwrap_or(false);
                    if is_hit_target || background.is_some() {
                        let glyph_bounds = Aabb::new(
                            current_x,
                            current_y + line.top,
                            current_x + advance,
                            current_y + line.bottom,
                        );
                        if is_hit_target {
                            value_run.with_mut(|value_run| value_run.add_hit_rect(glyph_bounds));
                        }
                        if let Some(background) = background {
                            if style
                                .with_downcast::<TextStylePaint, _>(|paint| {
                                    paint.base.parent_handle()
                                })
                                .flatten()
                                != self.base.handle()
                                && !self.global_backgrounds.contains(&background)
                            {
                                self.global_backgrounds.push(background.clone());
                            }
                            background.with_downcast_mut::<TextStyleBackground, _>(|background| {
                                background.add_rect(glyph_bounds);
                            });
                        }
                    }
                    current_x += advance;
                }
                // Nuxie addition: this drawn line's logical box. It starts at
                // the line's start, ends after its last glyph (an ellipsis
                // included) and spans the line's top to its bottom.
                let line_top = ordered_line.y() - line.baseline + line.top;
                let line_bottom = ordered_line.y() - line.baseline + line.bottom;
                lines_bounds = Some(match lines_bounds {
                    None => Aabb::new(line.start_x, line_top, current_x, line_bottom),
                    Some(drawn) => Aabb::new(
                        drawn.min_x.min(line.start_x),
                        drawn.min_y,
                        drawn.max_x.max(current_x),
                        line_bottom,
                    ),
                });
                for decoration in decorations.finish() {
                    add_decoration(
                        &mut self.render_styles,
                        &mut self.draw_commands,
                        &mut self.shared_geometry,
                        self.base.render_opacity(),
                        &decoration.style,
                        &decoration.path,
                        decoration.opacity,
                    );
                }
                if line_index == info.ellipsis_line {
                    break 'paragraphs;
                }
                line_index_in_paragraph += 1;
                line_index += 1;
            }
            if let Some(last) = paragraph_lines.last() {
                current_y += last.bottom;
            }
            current_y += self.fit_paragraph_spacing();
        }
        self.lines_bounds = lines_bounds;

        let mut scale = 1.0f32;
        let mut x_offset = -self.bounds.width() * self.base.origin_x();
        let mut y_offset = -self.bounds.height() * self.base.origin_y();
        if self.overflow() == TextOverflow::Fit {
            let x_scale = if (self.effective_sizing() != TextSizing::AutoWidth
                || self.overflow_as_fixed())
                && info.max_width > self.bounds.width()
            {
                self.bounds.width() / info.max_width
            } else {
                1.0
            };
            let baseline = if self.base.fit_from_baseline() {
                self.lines[0][0].baseline
            } else {
                0.0
            };
            let y_scale = if self.overflow_as_fixed() && info.total_height > self.bounds.height() {
                (self.bounds.height() - baseline) / (info.total_height - baseline)
            } else {
                1.0
            };
            if x_scale != 1.0 || y_scale != 1.0 {
                scale = x_scale.min(y_scale).max(0.0);
                y_offset += baseline * (1.0 - scale);
                match self.align() {
                    TextAlign::Center => {
                        x_offset += (self.bounds.width() - info.max_width * scale) / 2.0
                            - minimum_x * scale;
                    }
                    TextAlign::Right => {
                        x_offset +=
                            self.bounds.width() - info.max_width * scale - minimum_x * scale;
                    }
                    _ => {}
                }
            }
        }
        if self.vertical_align() != VerticalTextAlign::Top && self.overflow_as_fixed() {
            y_offset = -self.bounds.height() * self.base.origin_y();
            if self.vertical_align() == VerticalTextAlign::Middle {
                y_offset += (self.bounds.height() - info.total_height * scale) / 2.0;
            } else if self.vertical_align() == VerticalTextAlign::Bottom {
                y_offset += self.bounds.height() - info.total_height * scale;
            }
        }
        let active_color_keys: HashSet<_> = self
            .draw_commands
            .iter()
            .filter_map(|command| match command {
                TextDrawCommand::ColorGlyph {
                    font,
                    glyph_id,
                    cluster_text,
                    ..
                } => Some(color_glyph_cache_key(font, *glyph_id, cluster_text)),
                TextDrawCommand::Style(_) | TextDrawCommand::SharedStyle(_, _) => None,
            })
            .collect();
        self.emoji_image_cache.retain(|entry| {
            active_color_keys.contains(&color_glyph_cache_key(
                &entry.font,
                entry.glyph_id,
                &entry.cluster_text,
            ))
        });
        self.internal_transform =
            Mat2D::from_scale_and_translation(scale, scale, x_offset, y_offset);
        // A solve that already measured this shape needs no second solve.
        // Ask once when shaping disagrees with a measurement. A style-only
        // solve leaves zero; a solve that measures records a fresh key.
        if self.measured_key != 0 && self.shaped_key != self.measured_key {
            self.measured_key = 0;
            self.base.mark_layout_node_dirty();
        }
        for run in &mut self.all_runs {
            run.with_mut(|run| {
                if run.is_hit_target() {
                    run.compute_hit_contours();
                }
            });
        }
        for style in &self.text_style_paints {
            if let Some(background) = style
                .with_downcast::<TextStylePaint, _>(TextStylePaint::background)
                .flatten()
            {
                background.with_downcast_mut::<TextStyleBackground, _>(|background| {
                    background.update_path();
                    background.propagate_opacity(self.base.render_opacity());
                });
            }
        }
        for background in &self.global_backgrounds {
            background.with_downcast_mut::<TextStyleBackground, _>(|background| {
                background.update_path();
                background.propagate_opacity(self.base.render_opacity());
            });
        }
        self.shared_geometry
            .retain(|(_, geometry)| geometry.borrow().has_contents());
    }
    pub fn draw(&mut self, renderer: &mut Renderer) {
        if self.base.needs_save_operation() {
            renderer.save();
        }
        if self.overflow() == TextOverflow::Clipped
            && (!self.clip_path.empty() || self.clip_path.has_render_path())
        {
            let factory = self
                .base
                .with_artboard(|artboard| artboard.factory())
                .flatten()
                .expect("Text requires its Artboard renderer factory");
            renderer.clip_path(self.clip_path.render_path(&factory));
        }
        let world_transform = self.shape_world_transform();
        let blend_mode = self.base.blend_mode().into();
        let additive_amount = self.base.additive_amount();
        // Backgrounds precede every glyph, in style child order.
        for style in &self.text_style_paints {
            if let Some(background) = style
                .with_downcast::<TextStylePaint, _>(TextStylePaint::background)
                .flatten()
            {
                background.with_downcast_mut::<TextStyleBackground, _>(|background| {
                    background.draw(renderer, &world_transform, blend_mode, additive_amount);
                });
            }
        }
        for background in &self.global_backgrounds {
            background.with_downcast_mut::<TextStyleBackground, _>(|background| {
                background.draw(renderer, &world_transform, blend_mode, additive_amount);
            });
        }
        if let Some(controller) = self.selection_controller.upgrade() {
            controller.borrow_mut().draw(self, renderer);
        }
        for index in 0..self.draw_commands.len() {
            match &self.draw_commands[index] {
                TextDrawCommand::SharedStyle(style, geometry) => {
                    style.with_downcast_mut::<TextStylePaint, _>(|paint| {
                        paint.draw_geometry(
                            renderer,
                            &world_transform,
                            &mut geometry.borrow_mut(),
                            self.base.render_opacity(),
                            blend_mode,
                            additive_amount,
                        );
                    });
                }
                TextDrawCommand::Style(style) => {
                    style.with_downcast_mut::<TextStylePaint, _>(|style| {
                        style.draw(renderer, &world_transform, blend_mode, additive_amount)
                    });
                }
                TextDrawCommand::ColorGlyph {
                    font,
                    glyph_id,
                    cluster_text,
                    transform,
                    foreground_color,
                    opacity,
                } => self.draw_color_glyph(
                    renderer,
                    font.clone(),
                    *glyph_id,
                    &cluster_text.clone(),
                    *transform,
                    *foreground_color,
                    *opacity,
                    world_transform,
                ),
            }
        }
        if self.base.needs_save_operation() {
            renderer.restore();
        }
    }

    fn draw_color_glyph(
        &mut self,
        renderer: &mut Renderer,
        font: FontRef,
        glyph_id: u16,
        cluster: &[u32],
        transform: Mat2D,
        foreground_color: ColorInt,
        opacity: f32,
        world_transform: Mat2D,
    ) {
        let base = &self.base;
        draw_color_glyph_with_factory(
            renderer,
            || {
                base.with_artboard(|artboard| artboard.factory())
                    .flatten()
                    .expect("Text requires its Artboard renderer factory")
            },
            &mut self.emoji_image_cache,
            font,
            glyph_id,
            cluster,
            transform,
            foreground_color,
            opacity,
            world_transform,
        );
    }
    pub fn local_bounds(&self) -> Aabb {
        let width = self.bounds.width();
        let height = self.bounds.height();
        Aabb::from_ltwh(
            self.bounds.min_x - width * self.base.origin_x(),
            self.bounds.min_y - height * self.base.origin_y(),
            width,
            height,
        )
    }
    /// Nuxie addition, read-only: the rectangle of the logical line boxes of
    /// the lines this text draws, not glyph ink. Left is the smallest line
    /// start, right the furthest point after a line's last glyph (an ellipsis
    /// included), top the first drawn line's top and bottom the last drawn
    /// line's bottom. Lines the overflow skips are not in it. It is in raw
    /// line coordinates, before `internal_transform`, and is recorded by the
    /// drawing walk `build_render_styles` already runs. Empty or unshaped text,
    /// or text that draws no line, has none.
    pub fn lines_bounds(&self) -> Option<Aabb> {
        self.lines_bounds
    }
    /// First logical line's baseline in shaped-content coordinates, before
    /// `internal_transform`. Unlike visible ordered lines this remains stable
    /// when overflow clips the first line. Empty/unshaped text has no baseline.
    pub fn first_baseline(&self) -> Option<f32> {
        let line = self.lines.first()?.first()?;
        let info = self.compute_bounds_info();
        Some(info.min_y - info.top_trim + line.baseline)
    }
    pub fn hit_test<'a>(&'a self, _info: &HitInfo, _transform: &Mat2D) -> Option<&'a Core> {
        if self.base.render_opacity() == 0.0 {
            return None;
        }
        None
    }
    pub fn constraint_bounds(&self) -> Aabb {
        self.local_bounds()
    }
    pub fn computed_width(&self) -> f32 {
        self.local_bounds().width()
    }
    pub fn computed_height(&self) -> f32 {
        self.local_bounds().height()
    }
    pub fn on_dirty(&mut self, value: ComponentDirt) {
        self.on_dirty_with_modifier_group(value, None);
    }
    fn on_dirty_with_modifier_group(
        &mut self,
        value: ComponentDirt,
        mut active_group: Option<&mut TextModifierGroup>,
    ) {
        if value.intersects(ComponentDirt::WORLD_TRANSFORM) {
            for group in self.modifier_groups.clone() {
                if let Some(active_group) = active_group.as_deref_mut() {
                    TextModifierGroup::on_text_world_transform_dirty_from_group(
                        &group,
                        self,
                        active_group,
                    );
                } else {
                    TextModifierGroup::on_text_world_transform_dirty(&group, self);
                }
            }
        }
        if value.intersects(ComponentDirt::PATH | ComponentDirt::PAINT) {
            for style in &mut self.render_styles {
                style.with_downcast_mut::<TextStylePaint, _>(|style| {
                    style.paints.invalidate_stroke_effects()
                });
            }
        }
    }
    pub(crate) fn on_dirty_occurrence(owner: &CoreHandle, value: ComponentDirt) {
        if value.intersects(ComponentDirt::WORLD_TRANSFORM) {
            let mut index = 0;
            while let Some(group) = owner
                .with_downcast::<Self, _>(|text| text.modifier_groups.get(index).cloned())
                .flatten()
            {
                TextModifierGroup::on_text_world_transform_dirty_occurrence(&group);
                index += 1;
            }
        }
        if value.intersects(ComponentDirt::PATH | ComponentDirt::PAINT) {
            // Read render styles after the modifier callbacks, at the same
            // point as C++. No Text or style guard crosses renderer callbacks.
            let mut index = 0;
            while let Some(style) = owner
                .with_downcast::<Self, _>(|text| text.render_styles.get(index).cloned())
                .flatten()
            {
                crate::mechanical_port::source::shapes::shape_paint_container::ShapePaintContainer::invalidate_stroke_effects_occurrence(&style);
                index += 1;
            }
        }
    }
    pub fn layout_base_translation(&self, participant: &LayoutParticipant) -> Vec2D {
        Vec2D::new(
            participant.resolved_left() + self.base.origin_x() * participant.resolved_width(),
            participant.resolved_top() + self.base.origin_y() * participant.resolved_height(),
        )
    }

    pub(crate) fn try_compose_world_transform_override(&mut self) -> bool {
        let participant = self.base.children().iter().find_map(|child| {
            child
                .with(|child| {
                    child
                        .as_any()
                        .downcast_ref::<LayoutParticipant>()
                        .map(|participant| self.layout_base_translation(participant))
                })
                .flatten()
        });
        let parent_world = self.base.parent_transform_component().and_then(|parent| {
            parent
                .with(|parent| {
                    parent
                        .as_world_transform_component()
                        .map(|parent| parent.world_transform())
                })
                .flatten()
        });
        if let (Some(translation), Some(parent_world)) = (participant, parent_world) {
            let base = Mat2D::from_translation(translation);
            let transform = self.base.transform();
            self.base
                .set_world_transform(parent_world * base * transform);
            return true;
        }
        false
    }

    pub fn layout_participant(&self) -> Option<CoreHandle> {
        self.base
            .children()
            .iter()
            .find(|child| {
                child.is_type_of(crate::mechanical_port::source::generated::layout::layout_participant_base::LayoutParticipantBase::TYPE_KEY)
            })
            .cloned()
    }
    pub fn is_participating_in_layout(&self) -> bool {
        self.layout_participant().is_some()
    }
    pub fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        let max = Vec2D::new(
            if width_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                width
            },
            if height_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                height
            },
        );
        self.measure(
            max,
            (width_mode == LayoutMeasureMode::Exactly).then_some(width),
        )
    }
    pub fn control_size(
        &mut self,
        size: Vec2D,
        w: LayoutScaleType,
        h: LayoutScaleType,
        d: LayoutDirection,
    ) {
        if self.layout_width != size.x
            || self.layout_height != size.y
            || self.layout_width_scale_type != w as u8
            || self.layout_height_scale_type != h as u8
            || self.layout_direction != d
        {
            self.layout_width = size.x;
            self.layout_height = size.y;
            self.layout_width_scale_type = w as u8;
            self.layout_height_scale_type = h as u8;
            self.layout_direction = d;
            self.mark_shape_dirty_layout(false);
        }
    }
    /// The paragraphs the last measure shaped from exactly what `styled`
    /// shapes, taken so an update shaping the same text needn't shape it again.
    fn take_measured_shape(&mut self, styled: &StyledText) -> Option<Vec<Paragraph>> {
        let measured = self.measured_shape.take()?;
        if measured.fallback == fallback_state()
            && styled.shapes_like(&measured.unichars, &measured.runs, &measured.css_tabs)
        {
            return Some(measured.shape);
        }
        self.measured_shape = Some(measured);
        None
    }

    /// Layout adapter query after an unbounded measure. Reuse its cached shape
    /// when present. If measurement could not retain a shape, notably after a
    /// fallback-font callback, shape a temporary copy for this minimum query.
    /// The direct Rive measurement protocol and its cache policy stay unchanged.
    pub(crate) fn layout_min_content_width(&mut self) -> Option<f32> {
        if self.styled_text.empty() {
            return Some(0.0);
        }
        let temporary = if self.measured_shape.is_none() {
            let mut styled = StyledText::default();
            if !self.make_styled(&mut styled, true, 1.0) {
                return Some(0.0);
            }
            Some(styled.shape())
        } else {
            None
        };
        // CSS break-word emergency breaks do not reduce min-content.
        let word_break = if self.word_break() == TextWordBreak::BreakAll {
            TextWordBreak::BreakAll
        } else {
            TextWordBreak::Normal
        };
        let wrap = self.wrap();
        let shape = self
            .measured_shape
            .as_ref()
            .map(|measured| &measured.shape)
            .or(temporary.as_ref())?;
        let lines = Self::break_styled_lines(
            0.0,
            TextAlign::Left,
            wrap,
            word_break,
            &self.styled_text,
            shape,
        );
        Some(
            shape
                .iter()
                .zip(&lines)
                .fold(0.0f32, |minimum, (paragraph, lines)| {
                    minimum.max(GlyphLine::compute_max_width(lines, &paragraph.runs))
                }),
        )
    }

    fn measure_key(&self, styled: &StyledText, font_scale: f32) -> u64 {
        let mut hash = 0x9e3779b97f4a7c15u64;
        let mut mix = |word: u64| {
            hash ^= word;
            hash = hash.wrapping_mul(0xff51afd7ed558ccd);
            hash ^= hash >> 32;
        };
        let bits = |value: f32| u64::from(value.to_bits());
        let sizing = match self.effective_sizing() {
            TextSizing::AutoWidth => 0,
            TextSizing::AutoHeight => 1,
            TextSizing::Fixed => 2,
            TextSizing::Unknown(value) => value as u8,
        };
        let align = match self.align() {
            TextAlign::Left => 0,
            TextAlign::Right => 1,
            TextAlign::Center => 2,
            TextAlign::Unknown(value) => value as u8,
        };
        mix(bits(font_scale) | (u64::from(sizing) << 32) | (u64::from(align) << 40));
        if self.css_owned {
            mix(1);
            if let Some(strut) = &styled.css_strut {
                mix(strut.font.as_ref().map_or(0, |font| {
                    std::sync::Arc::as_ptr(font) as *const () as usize as u64
                }));
                mix(bits(strut.size) | (bits(strut.line_height) << 32));
            }
        }
        if let Some(map) = &styled.css_indices {
            mix(2);
            for i in 0..styled.value.len() {
                mix(u64::from(map.is_collapsible(i as u32))
                    | (u64::from(map.is_tab(i as u32)) << 1));
            }
        }
        for run in styled.runs() {
            // Arc's data pointer is the identity corresponding to Font*. The
            // trait-object vtable is not part of the C++ pointer word.
            mix(run.font.as_ref().map_or(0, |font| {
                std::sync::Arc::as_ptr(font) as *const () as usize as u64
            }));
            mix(bits(run.size) | (bits(run.line_height) << 32));
            mix(bits(run.letter_spacing) | (u64::from(run.unichar_count) << 32));
            mix(u64::from(run.script)
                | (u64::from(run.style_id) << 32)
                | (u64::from(run.level) << 48));
        }
        let unichars = styled.unichars();
        mix(unichars.len() as u64);
        let mut pairs = unichars.chunks_exact(2);
        for pair in &mut pairs {
            mix(u64::from(pair[0]) | (u64::from(pair[1]) << 32));
        }
        if let Some(&last) = pairs.remainder().first() {
            mix(u64::from(last));
        }
        hash
    }

    fn measure(&mut self, max: Vec2D, exact_width: Option<f32>) -> Vec2D {
        // During the solve, the participant's authored axis types are already
        // available, but control_size has not supplied the resolved slot yet.
        let measuring_width = match self.effective_sizing() {
            TextSizing::AutoHeight | TextSizing::Fixed => {
                if self.layout_owns_width() {
                    f32::MAX
                } else {
                    self.base.width()
                }
            }
            _ => f32::MAX,
        };
        let fit_width = max.x.min(measuring_width);
        let font_scale = if self.overflow() == TextOverflow::FitFontSize
            && self.fit_font_size_resizes_box_active()
            && self.has_file_feature(FILE_FEATURE_LAYOUT_SIZES_BOX)
            && fit_width != f32::MAX
        {
            self.fit_font_scale_in_box(fit_width, max.y)
        } else {
            1.0
        };
        let mut styled = std::mem::take(&mut self.styled_text);
        let has_styled = self.make_styled(&mut styled, true, font_scale);
        self.measured_key = self.measure_key(&styled, font_scale);
        if !has_styled {
            self.styled_text = styled;
            return Vec2D::default();
        }
        let paragraph_space = self.base.paragraph_spacing() * font_scale;
        // Shape only what the last measure didn't already shape.
        let fallback = fallback_state();
        let (measured, keep) = match self.measured_shape.take() {
            Some(measured)
                if measured.fallback == fallback
                    && styled.shapes_like(
                        &measured.unichars,
                        &measured.runs,
                        &measured.css_tabs,
                    ) =>
            {
                (measured, true)
            }
            previous => {
                let calls = fallback_calls();
                let runs = styled.runs();
                let shape = styled.shape();
                // Refill the previous record's storage rather than allocating.
                let mut measured = previous.unwrap_or_else(|| {
                    Box::new(MeasuredShape {
                        unichars: Vec::new(),
                        css_tabs: Vec::new(),
                        runs: Vec::new(),
                        fallback,
                        shape: Vec::new(),
                    })
                });
                measured.unichars.clear();
                measured.unichars.extend_from_slice(styled.unichars());
                measured.css_tabs.clear();
                measured.css_tabs.extend_from_slice(styled.preserved_tabs());
                measured.runs.clear();
                measured.runs.extend_from_slice(runs);
                measured.fallback = fallback;
                measured.shape = shape;
                (measured, fallback_calls() == calls)
            }
        };
        let shape = &measured.shape;
        let measuring_wrap =
            if max.x == f32::MAX && self.effective_sizing() != TextSizing::AutoHeight {
                TextWrap::NoWrap
            } else {
                self.wrap()
            };
        let lines = Self::break_styled_lines(
            fit_width,
            self.align(),
            measuring_wrap,
            self.word_break(),
            &styled,
            shape,
        );
        let mut y = 0.0f32;
        let mut computed_height = 0.0f32;
        let mut min_y = 0.0f32;
        let mut max_width = 0.0f32;
        if self.text_origin() == TextOrigin::Baseline && !lines.is_empty() && !lines[0].is_empty() {
            y -= lines[0][0].baseline;
            min_y = y;
        }
        let mut ellipsis_line = -1i32;
        let want_ellipsis =
            self.overflow() == TextOverflow::Ellipsis && self.sizing() == TextSizing::Fixed;
        'paragraphs: for (paragraph, paragraph_lines) in shape.iter().zip(&lines) {
            for line in paragraph_lines {
                let end_run = &paragraph.runs[line.end_run_index as usize];
                let start_run = &paragraph.runs[line.start_run_index as usize];
                max_width = max_width.max(
                    end_run.xpos[line.end_glyph_index as usize]
                        - start_run.xpos[line.start_glyph_index as usize],
                );
                if want_ellipsis && y + line.bottom > max.y {
                    if ellipsis_line == -1 {
                        computed_height = y + line.bottom;
                    }
                    break 'paragraphs;
                }
                ellipsis_line += 1;
                computed_height = y + line.bottom;
            }
            if let Some(last) = paragraph_lines.last() {
                y += last.bottom;
            }
            y += paragraph_space;
        }
        let (top_trim, bottom_trim) = compute_vertical_trim(
            &lines,
            shape,
            self.vertical_trim_top(),
            self.vertical_trim_bottom(),
        );
        let bounds = match self.sizing() {
            TextSizing::AutoWidth => Vec2D::new(
                max_width,
                min_y.max(computed_height - top_trim - bottom_trim),
            ),
            TextSizing::AutoHeight => Vec2D::new(
                // Taffy's known-width boundary reports its exact slot width;
                // shaping above still follows upstream participant ownership.
                exact_width.unwrap_or(self.base.width()),
                min_y.max(computed_height - top_trim - bottom_trim),
            ),
            TextSizing::Fixed => Vec2D::new(self.base.width(), min_y + self.base.height()),
            TextSizing::Unknown(_) => Vec2D::default(),
        };
        self.styled_text = styled;
        if keep {
            self.measured_shape = Some(measured);
        }
        Vec2D::new(max.x.min(bounds.x), max.y.min(bounds.y))
    }
    pub fn align_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn word_break_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn sizing_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn fit_font_size_resizes_box_changed(&mut self) {
        if self.overflow() == TextOverflow::FitFontSize {
            self.mark_shape_dirty();
        }
    }
    pub fn overflow_value_changed(&mut self) {
        if self.effective_sizing() != TextSizing::AutoWidth || self.overflow_as_fixed() {
            self.mark_shape_dirty();
        } else {
            // AutoWidth glyph shaping is unchanged, but font fitting changes
            // the layout adapter's intrinsic minimum contract.
            self.base.mark_layout_node_dirty();
        }
    }
    pub fn width_changed(&mut self) {
        if self.effective_sizing() != TextSizing::AutoWidth {
            self.mark_shape_dirty();
        }
    }
    pub fn height_changed(&mut self) {
        if self.effective_sizing() == TextSizing::Fixed {
            self.mark_shape_dirty();
        }
    }
    pub fn paragraph_spacing_changed(&mut self) {
        if self.overflow() == TextOverflow::FitFontSize {
            self.mark_shape_dirty();
        } else {
            self.mark_paint_dirty();
            self.base.mark_layout_node_dirty();
        }
    }
    pub fn origin_value_changed(&mut self) {
        self.mark_paint_dirty();
        CoreCapabilities::world_transform_mark_dirty(self);
        self.base.mark_layout_node_dirty();
    }
    pub fn origin_x_changed(&mut self) {
        self.mark_paint_dirty();
        CoreCapabilities::world_transform_mark_dirty(self);
    }
    pub fn origin_y_changed(&mut self) {
        self.mark_paint_dirty();
        CoreCapabilities::world_transform_mark_dirty(self);
    }
    pub fn vertical_trim_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn update_list(&mut self, list: Option<&[CoreHandle]>) {
        let Some(list) = list else {
            return;
        };
        {
            self.build_text_style_paints();
            self.all_runs.clear();
            self.all_runs
                .extend(self.runs.iter().cloned().map(TextValueRunHandle::Core));
            let mut value_run_listeners = std::mem::take(&mut self.value_run_listeners);
            let current_size = value_run_listeners.len();
            let mut index = 0usize;
            let Some(text) = self.base.handle() else {
                return;
            };
            for item in list {
                let Some(instance) = item
                    .with(|item| {
                        item.as_view_model_instance_list_item()?
                            .view_model_instance()
                    })
                    .flatten()
                else {
                    continue;
                };
                let text_run = if index < current_size {
                    let listener = &mut value_run_listeners[index];
                    listener.remap(instance, self);
                    listener.text_value_run()
                } else {
                    let listener = TextValueRunListener::new(
                        TextValueRun::default(),
                        instance,
                        text.clone(),
                        self,
                    );
                    value_run_listeners.push(listener);
                    value_run_listeners[index].text_value_run()
                };
                self.all_runs.push(text_run);
                index += 1;
            }
            value_run_listeners.truncate(index);
            self.value_run_listeners = value_run_listeners;
            self.mark_shape_dirty();
        }
    }
    pub fn resolve_list_run_style_occurrence(
        owner: &CoreHandle,
        item: &CoreHandle,
    ) -> Option<TextStyleReferenceHandle> {
        Self::resolve_list_run_style_with_source(owner, item, None)
    }
    fn resolve_list_run_style_with_source(
        owner: &CoreHandle,
        item: &CoreHandle,
        current: Option<(&CoreHandle, &TextStyleSourceValue)>,
    ) -> Option<TextStyleReferenceHandle> {
        use crate::source::viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard;
        let (global, name) = item
            .with(|item| {
                let vm = item.as_view_model_instance()?;
                Some((
                    vm.property_value_for_symbol(SymbolType::GlobalTextStyle),
                    vm.property_value_for_symbol(SymbolType::TextStyle),
                ))
            })
            .flatten()?;
        if let Some(value) = global.as_ref().filter(|value| {
            value.is_type_of(crate::source::generated::viewmodel::viewmodel_instance_artboard_base::ViewModelInstanceArtboardBase::TYPE_KEY)
        }) {
            if let Some(style) = Self::global_style_reference_occurrence(owner, value, item, current) {
                return Some(style);
            }
        }
        if global.is_none() && name.is_none() {
            return None;
        }
        if let Some(name) = name.and_then(|name| {
            if let Some((source, TextStyleSourceValue::String(value))) = current {
                if source == &name {
                    return Some(value.clone());
                }
            }
            name.with_downcast::<ViewModelInstanceString, _>(|name| {
                name.base.property_value().to_owned()
            })
        }) {
            if let Some(style) = owner
                .with_downcast::<Self, _>(|text| text.style_reference_named(&name))
                .flatten()
            {
                return Some(style);
            }
        }
        owner
            .with_downcast_mut::<Self, _>(Self::first_style_reference)
            .flatten()
    }
    fn global_style_reference_occurrence(
        owner: &CoreHandle,
        value: &CoreHandle,
        item: &CoreHandle,
        current: Option<(&CoreHandle, &TextStyleSourceValue)>,
    ) -> Option<TextStyleReferenceHandle> {
        use super::{
            text_style_global::TextStyleGlobal, text_style_share_cache::TextStyleShareCache,
        };
        use crate::source::viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard;
        let snapshot = current
            .filter(|(source, _)| *source == value)
            .and_then(|(_, current)| match current {
                TextStyleSourceValue::Artboard {
                    asset,
                    index,
                    bound,
                } => Some((asset.clone(), *index, bound.clone())),
                _ => None,
            });
        let (asset, index, bound) = snapshot.or_else(|| {
            value.with_downcast::<ViewModelInstanceArtboard, _>(|value| {
                (
                    value.asset(),
                    value.base.property_value(),
                    value.bound_view_model_instance(),
                )
            })
        })?;
        let (file, children, existing, host) = owner.with_downcast::<Self, _>(|text| {
            (
                text.file.as_ref().and_then(|file| file.upgrade()),
                text.base.children().to_vec(),
                text.on_demand_styles.clone(),
                text.base
                    .with_artboard(|host| (host.factory(), host.data_context())),
            )
        })?;
        let mut cross_file = None;
        let source = if let Some(asset) = asset {
            if let Some(asset_file) = asset.file().and_then(|file| file.upgrade()) {
                if !file.as_ref().is_some_and(|file| file.ptr_eq(&asset_file)) {
                    cross_file = Some(asset_file);
                }
            }
            asset.source_artboard_handle()
        } else if index != u32::MAX {
            file.as_ref()
                .and_then(|file| file.with_file(|file| file.artboard_at_source(index as usize)))
        } else {
            None
        }?;
        TextStyleGlobal::first_text_style_paint(Some(&source))?;
        let info = if cross_file.is_none() && file.is_some() {
            file.as_ref()
                .unwrap()
                .with_file(|file| file.text_style_shares())
                .borrow_mut()
                .info(&source)
        } else {
            TextStyleShareCache::analyze(&source, cross_file.as_ref())
        };
        let per_item = info.has_state_machine.then(|| item.clone());
        let mut values_source = None;
        if bound.is_none() {
            for child in children {
                if let Some((instance_source, values)) =
                    child.with_downcast::<TextStyleGlobal, _>(|style| {
                        (style.instance_source(), style.style_view_model_instance())
                    })
                {
                    if instance_source.as_ref() == Some(&source) {
                        if per_item.is_none() {
                            return Some(TextStyleReferenceHandle::Authored(child));
                        }
                        values_source = values;
                        break;
                    }
                }
            }
        }
        for style in existing {
            let same = {
                let style = style.borrow();
                style.source() == source
                    && style.bound_view_model_instance() == bound
                    && style.item() == per_item
            };
            if same {
                return Some(TextStyleReferenceHandle::OnDemand(style));
            }
        }
        let (factory, parent) = host?;
        let style = TextStyleOnDemand::new(
            source,
            bound,
            file,
            cross_file,
            factory?,
            parent,
            Some(owner.clone()),
            per_item,
            values_source,
        );
        owner.with_downcast_mut::<Self, _>(|text| text.on_demand_styles.push(style.clone()));
        Some(TextStyleReferenceHandle::OnDemand(style))
    }
    pub fn resolve_list_run_styles_occurrence(owner: &CoreHandle) {
        let listeners = owner
            .with_downcast::<Self, _>(|text| {
                text.value_run_listeners
                    .iter()
                    .map(|listener| (listener.instance.clone(), listener.text_value_run.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for (item, run) in listeners {
            let style = Self::resolve_list_run_style_occurrence(owner, &item);
            run.borrow_mut().set_style_reference(style);
        }
        owner.with_downcast_mut::<Self, _>(|text| text.prune_on_demand_styles());
    }
    pub fn set_file(&mut self, file: crate::source::file::RuntimeFileWeakHandle) {
        self.file = Some(file);
    }
    pub fn file(&self) -> Option<crate::source::file::RuntimeFileWeakHandle> {
        self.file.clone()
    }
    pub fn on_demand_style_count(&self) -> usize {
        self.on_demand_styles.len()
    }
    pub fn run_style_references(&self) -> Vec<TextStyleReferenceHandle> {
        let mut styles: Vec<_> = self.base.children().iter().filter(|child| child.is_type_of(crate::source::generated::text::text_style_reference_base::TextStyleReferenceBase::TYPE_KEY)).cloned().map(TextStyleReferenceHandle::Authored).collect();
        styles.extend(
            self.on_demand_styles
                .iter()
                .cloned()
                .map(TextStyleReferenceHandle::OnDemand),
        );
        styles
    }
    pub fn run_state_machines(
        &self,
        out: &mut Vec<
            crate::source::animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
        >,
    ) {
        out.extend(
            self.run_style_references()
                .iter()
                .filter_map(|style| style.run_state_machine()),
        );
    }
    pub fn may_have_listening_runs(&self) -> bool {
        self.run_style_references().iter().any(|style| {
            matches!(style, TextStyleReferenceHandle::Authored(_)) && style.listens_to_runs()
        }) || self
            .file
            .as_ref()
            .and_then(|file| file.upgrade())
            .is_some_and(|file| {
                file.with_file(|file| file.text_style_shares())
                    .borrow_mut()
                    .any_listens_to_runs()
            })
    }
    pub fn style_reference_named(&self, name: &str) -> Option<TextStyleReferenceHandle> {
        self.base.children().iter().find(|child| child.is_type_of(crate::source::generated::text::text_style_reference_base::TextStyleReferenceBase::TYPE_KEY) && child.with(|child| child.as_component().is_some_and(|child| child.name() == name)).unwrap_or(false)).cloned().map(TextStyleReferenceHandle::Authored)
    }
    pub fn first_style_reference(&mut self) -> Option<TextStyleReferenceHandle> {
        self.build_text_style_paints();
        self.text_style_paints
            .first()
            .cloned()
            .map(TextStyleReferenceHandle::Authored)
            .or_else(|| {
                self.run_style_references()
                    .into_iter()
                    .find(|style| matches!(style, TextStyleReferenceHandle::Authored(_)))
            })
    }
    pub fn style_reference_will_change(&mut self) {
        self.clear_render_styles();
        self.shared_geometry.clear();
        self.mark_shape_dirty();
    }
    pub fn style_reference_did_change_occurrence(owner: &CoreHandle) {
        Self::resolve_list_run_styles_occurrence(owner);
        owner.with_downcast_mut::<Self, _>(|text| {
            text.prune_on_demand_styles();
            text.mark_shape_dirty();
        });
    }
    fn prune_on_demand_styles(&mut self) {
        let used: Vec<_> = self
            .value_run_listeners
            .iter()
            .filter_map(|listener| listener.text_value_run.borrow().style_reference())
            .collect();
        if self
            .on_demand_styles
            .iter()
            .all(|style| used.contains(&TextStyleReferenceHandle::OnDemand(style.clone())))
        {
            return;
        }
        self.clear_render_styles();
        self.shared_geometry.clear();
        self.on_demand_styles
            .retain(|style| used.contains(&TextStyleReferenceHandle::OnDemand(style.clone())));
        self.mark_shape_dirty();
    }
    pub fn mark_listened_runs(&mut self) {
        if !self.listened_runs_dirty {
            return;
        }
        self.listened_runs_dirty = false;
        if !self
            .base
            .with_artboard(|artboard| artboard.is_instance())
            .unwrap_or(false)
        {
            return;
        }
        for run in &self.all_runs {
            if run
                .with(|run| {
                    run.style_reference()
                        .is_some_and(|style| style.listens_to_runs())
                })
                .unwrap_or(false)
            {
                run.with_mut(|run| run.set_is_hit_target(true));
            }
        }
    }
    pub fn sync_style_list_items_occurrence(owner: &CoreHandle) {
        let bindings = owner
            .with_downcast_mut::<Self, _>(|text| {
                if !text.list_items_dirty {
                    return Vec::new();
                }
                text.list_items_dirty = false;
                if !text
                    .base
                    .with_artboard(|artboard| artboard.is_instance())
                    .unwrap_or(false)
                {
                    return Vec::new();
                }
                text.run_style_references()
                    .into_iter()
                    .map(|style| {
                        let runs: Vec<_> = text
                            .all_runs
                            .iter()
                            .filter(|run| {
                                run.with(|run| run.style_reference().as_ref() == Some(&style))
                                    .unwrap_or(false)
                            })
                            .collect();
                        let item = if runs.len() == 1 {
                            match runs[0] {
                                TextValueRunHandle::Runtime(run) => text
                                    .value_run_listeners
                                    .iter()
                                    .find(|listener| Rc::ptr_eq(&listener.text_value_run, run))
                                    .map(|listener| listener.instance.clone()),
                                _ => None,
                            }
                        } else {
                            None
                        };
                        (style, item)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for (style, item) in bindings {
            style.list_item(item);
        }
    }
    fn on_demand_occurrences(owner: &CoreHandle) -> Vec<Rc<RefCell<TextStyleOnDemand>>> {
        owner
            .with_downcast::<Self, _>(|text| text.on_demand_styles.clone())
            .unwrap_or_default()
    }
    pub fn host_data_context_occurrence(
        owner: &CoreHandle,
        value: Option<crate::source::data_bind::data_context::RuntimeDataContextHandle>,
    ) {
        for style in Self::on_demand_occurrences(owner) {
            style.borrow_mut().host_data_context(value.clone());
        }
    }
    pub fn relink_host_data_context_occurrence(owner: &CoreHandle) {
        for style in Self::on_demand_occurrences(owner) {
            style.borrow_mut().relink_host_data_context();
        }
    }
    pub fn update_hosted_data_binds_occurrence(owner: &CoreHandle) {
        for style in Self::on_demand_occurrences(owner) {
            style.borrow_mut().update_hosted_data_binds();
        }
    }
    pub fn advance_component_occurrence(
        owner: &CoreHandle,
        elapsed: f32,
        flags: crate::source::advance_flags::AdvanceFlags,
    ) -> bool {
        Self::sync_style_list_items_occurrence(owner);
        let mut changed = false;
        let mut keep_going = false;
        for style in Self::on_demand_occurrences(owner) {
            keep_going |= style.borrow_mut().advance(elapsed, flags, &mut changed);
        }
        if changed {
            Self::mark_shape_dirty_occurrence(owner, true);
        }
        keep_going
    }
    pub fn quiet_state(&self) -> crate::source::advancing_component::QuietState {
        use crate::source::advancing_component::QuietState;
        let mut state = QuietState::Quiet;
        for style in &self.on_demand_styles {
            match style.borrow().quiet_state() {
                QuietState::Never => return QuietState::Never,
                QuietState::Busy => state = QuietState::Busy,
                QuietState::Quiet => {}
            }
        }
        state
    }
    pub fn build_text_style_paints(&mut self) {
        if self.text_style_paints.is_empty() {
            for child in self.base.children() {
                if child.core_type() == Some(crate::mechanical_port::source::generated::text::text_style_paint_base::TextStylePaintBase::TYPE_KEY) {
                    self.text_style_paints.push(child.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod settled_text_value_tests {
    use super::*;

    fn runtime_run(value: &str) -> TextValueRunHandle {
        let mut run = TextValueRun::default();
        run.base.set_text_value(value.to_owned());
        TextValueRunHandle::Runtime(Rc::new(RefCell::new(run)))
    }

    #[test]
    fn paint_style_rebuild_appends_ordered_lines_until_the_next_path_update() {
        // Pinned text.cpp clears m_orderedLines in the Path update, not in
        // buildRenderStyles. Preserve this even on a paint-only rebuild.
        let root = std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
        let bytes = std::fs::read(
            std::path::PathBuf::from(root).join("tests/unit_tests/assets/modifier_test.riv"),
        )
        .expect("pinned text modifier fixture");
        let mut factory =
            nuxie_render_api::PersistentFactory::new(nuxie_render_api::RecordingFactory::default());
        let file = crate::File::import(
            &bytes,
            crate::RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("text modifier fixture imports");
        let artboard = file.with_file(crate::File::artboard_default).unwrap();
        artboard.advance_default(0.0);
        let text = artboard
            .with_artboard(|root| root.find_all_handles::<Text>())
            .into_iter()
            .find(|text| {
                text.with_downcast::<Text, _>(|text| !text.ordered_lines.is_empty())
                    .unwrap()
            })
            .expect("fixture has shaped text lines");
        text.with_downcast_mut::<Text, _>(|text| {
            text.update_after_transform_super(ComponentDirt::PATH);
            let before = text.ordered_lines.len();
            assert!(before > 0);
            text.update_after_transform_super(ComponentDirt::PAINT);
            assert_eq!(text.ordered_lines.len(), before * 2);
            text.update_after_transform_super(ComponentDirt::PATH);
            assert_eq!(text.ordered_lines.len(), before);
        })
        .unwrap();
    }

    #[test]
    fn exact_layout_width_measures_lines_at_the_controlled_width() {
        use crate::mechanical_port::source::{
            assets::font_asset::FontAsset,
            core::CoreArena,
            text::{font_hb::HbFont, text_style::TextStyle},
        };
        let root = std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
        let bytes = std::fs::read(
            std::path::PathBuf::from(root)
                .join("tests/unit_tests/assets/fonts/Inter_18pt-Regular.ttf"),
        )
        .expect("pinned text fixture font");
        let arena = CoreArena::default();
        let asset = arena.insert(FontAsset::default());
        FontAsset::set_font_occurrence(&asset, Some(HbFont::decode(&bytes).unwrap()));
        let style = arena.insert(TextStylePaint::default());
        TextStyle::set_asset_occurrence(&style, Some(asset));
        style
            .with_mut(|object| {
                let style = object.as_text_style_mut().unwrap();
                style.base.set_font_size_value(40.0);
                style.base.set_line_height_value(44.0);
            })
            .unwrap();
        let mut run = TextValueRun::default();
        run.base
            .set_text_value("Choose what deserves your attention.".into());
        run.set_style(style);
        let mut text = Text::default();
        text.base.set_sizing_value_value(1);
        text.base.set_width_value(120.0);
        text.all_runs
            .push(TextValueRunHandle::Runtime(Rc::new(RefCell::new(run))));
        let measure = |text: &mut Text, width, mode| {
            text.measure_layout(width, mode, f32::NAN, LayoutMeasureMode::Undefined)
        };
        let narrow = measure(&mut text, 354.0, LayoutMeasureMode::AtMost);
        text.base.set_width_value(354.0);
        let authored = measure(&mut text, 354.0, LayoutMeasureMode::AtMost);
        assert!(narrow.y > authored.y, "fixture must wrap more at 120px");
        text.base.set_width_value(120.0);
        // Width ownership belongs to the participant before the first solve,
        // not to measure mode or the slot control_size supplies afterward.
        let mut participant = LayoutParticipant::default();
        participant
            .base
            .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
        participant
            .base
            .set_layout_height_scale_type_value(LayoutScaleType::Hug as u8);
        text.base.add_child(arena.insert(participant));
        let stretched = measure(&mut text, 354.0, LayoutMeasureMode::Exactly);
        assert_eq!(
            stretched, authored,
            "exact layout and authored widths must agree"
        );
        text.control_size(
            stretched,
            LayoutScaleType::Fill,
            LayoutScaleType::Hug,
            LayoutDirection::Ltr,
        );
        assert_eq!(
            measure(&mut text, 354.0, LayoutMeasureMode::Exactly),
            authored
        );
        // A later resize must use its new constraint, not the last drawn width.
        assert_eq!(
            measure(&mut text, 120.0, LayoutMeasureMode::Exactly),
            narrow
        );
    }

    #[test]
    fn settled_text_value_preserves_empty_resolved_runs_without_creating_accessibility_semantics() {
        let mut text = Text::default();
        text.all_runs.push(runtime_run(""));

        assert_eq!(text.settled_text_value(), "");
        assert!(text.inferred_semantic_data().is_none());
    }

    #[test]
    fn settled_text_value_concatenates_every_resolved_run_in_order() {
        let mut text = Text::default();
        text.all_runs
            .extend([runtime_run("first"), runtime_run(""), runtime_run("second")]);

        assert_eq!(text.settled_text_value(), "firstsecond");
    }
}

#[cfg(test)]
mod color_image_cache_tests {
    use super::*;
    use crate::mechanical_port::source::text_engine::{
        Axis, ColorGlyphLayer, ColorGlyphPaintType, Coord, Feature, Font, FontBase, LineMetrics,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct LayerCountingFont {
        base: FontBase,
        extractions: AtomicUsize,
        image: Option<ColorGlyphLayer>,
        scope: Option<crate::mechanical_port::source::text_engine::HostPictureScope>,
    }
    impl LayerCountingFont {
        fn new(image: Option<ColorGlyphLayer>) -> Arc<Self> {
            Self::with_scope(image, None)
        }
        fn with_scope(
            image: Option<ColorGlyphLayer>,
            scope: Option<crate::mechanical_port::source::text_engine::HostPictureScope>,
        ) -> Arc<Self> {
            Arc::new(Self {
                base: FontBase::new(LineMetrics {
                    ascent: -1.0,
                    descent: 0.0,
                    cap_height: 1.0,
                    x_height: 1.0,
                }),
                extractions: AtomicUsize::new(0),
                image,
                scope,
            })
        }
    }
    impl Font for LayerCountingFont {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn base(&self) -> &FontBase {
            &self.base
        }
        fn get_axis_count(&self) -> u16 {
            0
        }
        fn get_axis(&self, _: u16) -> Axis {
            unreachable!()
        }
        fn get_axis_value(&self, _: u32) -> f32 {
            0.0
        }
        fn get_weight(&self) -> u16 {
            400
        }
        fn is_italic(&self) -> bool {
            false
        }
        fn features(&self) -> Vec<u32> {
            Vec::new()
        }
        fn has_glyph(&self, _: u32) -> bool {
            true
        }
        fn codepoints(&self) -> Vec<u32> {
            vec![65]
        }
        fn get_feature_value(&self, _: u32) -> u32 {
            0
        }
        fn with_options(&self, _: &[Coord], _: &[Feature]) -> FontRef {
            unreachable!()
        }
        fn get_path(&self, _: u16) -> RawPath {
            RawPath::default()
        }
        fn on_shape_text(&self, _: &[u32], _: &[TextRun], _: i32) -> Vec<Paragraph> {
            unreachable!()
        }
        fn host_picture_scope(
            &self,
        ) -> Option<crate::mechanical_port::source::text_engine::HostPictureScope> {
            self.scope
        }
        fn get_color_layers(&self, _: u16, out: &mut Vec<ColorGlyphLayer>, _: ColorInt) -> usize {
            self.extractions.fetch_add(1, Ordering::SeqCst);
            if let Some(image) = &self.image {
                out.push(image.clone());
                1
            } else {
                0
            }
        }
    }

    fn image_layer() -> ColorGlyphLayer {
        let png = [
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
            8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4,
            0, 9, 251, 3, 253, 251, 94, 107, 43, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
        ];
        ColorGlyphLayer {
            paint_type: ColorGlyphPaintType::Image,
            image_bytes: Arc::from(png),
            image_width: 1,
            image_height: 1,
            image_bearing_x: 0.125,
            image_bearing_y: -0.875,
            image_extent_x: 1.0,
            image_extent_y: 1.0,
            ..Default::default()
        }
    }

    #[test]
    fn cached_image_draw_skips_layer_extraction_and_factory_lookup() {
        let font = LayerCountingFont::new(Some(image_layer()));
        let recording = nuxie_render_api::RecordingFactory::new();
        let mut renderer = recording.make_renderer();
        let mut factory = nuxie_render_api::PersistentFactory::new(recording);
        let handle = crate::RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let mut cache = Vec::new();
        draw_color_glyph(
            &mut renderer,
            &handle,
            &mut cache,
            font.clone(),
            1,
            &[65],
            Mat2D::default(),
            0xff000000,
            1.0,
            Mat2D::default(),
        );
        assert_eq!(font.extractions.load(Ordering::SeqCst), 1);
        draw_color_glyph_with_factory(
            &mut renderer,
            || panic!("cached image needs no factory"),
            &mut cache,
            font.clone(),
            1,
            &[65],
            Mat2D::default(),
            0xff000000,
            1.0,
            Mat2D::default(),
        );
        assert_eq!(font.extractions.load(Ordering::SeqCst), 1);
        // A different cluster sharing this glyph needs its own decoded answer.
        draw_color_glyph(
            &mut renderer,
            &handle,
            &mut cache,
            font.clone(),
            1,
            &[66],
            Mat2D::default(),
            0xff000000,
            1.0,
            Mat2D::default(),
        );
        assert_eq!(font.extractions.load(Ordering::SeqCst), 2);
        let stream = factory.borrow().stream();
        assert_eq!(
            stream
                .lines()
                .filter(|line| line.starts_with("decodeImage "))
                .count(),
            2
        );
        assert_eq!(
            stream
                .lines()
                .filter(|line| line.starts_with("drawImage "))
                .count(),
            3
        );
    }

    #[test]
    fn decoded_image_cache_uses_declared_host_picture_scope() {
        use crate::mechanical_port::source::text_engine::HostPictureScope;
        for (scope, expected_decodes) in
            [(HostPictureScope::Glyph, 1), (HostPictureScope::Cluster, 2)]
        {
            let font = LayerCountingFont::with_scope(Some(image_layer()), Some(scope));
            let recording = nuxie_render_api::RecordingFactory::new();
            let mut renderer = recording.make_renderer();
            let mut factory = nuxie_render_api::PersistentFactory::new(recording);
            let handle = crate::RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
            let mut cache = Vec::new();
            for cluster in [&[65][..], &[65, 0xfe0f][..], &[65][..]] {
                draw_color_glyph(
                    &mut renderer,
                    &handle,
                    &mut cache,
                    font.clone(),
                    1,
                    cluster,
                    Mat2D::default(),
                    0xff000000,
                    1.0,
                    Mat2D::default(),
                );
            }
            assert_eq!(font.extractions.load(Ordering::SeqCst), expected_decodes);
            assert_eq!(cache.len(), expected_decodes);
            let stream = factory.borrow().stream();
            assert_eq!(
                stream
                    .lines()
                    .filter(|line| line.starts_with("decodeImage "))
                    .count(),
                expected_decodes,
            );
            assert_eq!(
                stream
                    .lines()
                    .filter(|line| line.starts_with("drawImage "))
                    .count(),
                3,
            );
        }
    }

    #[test]
    fn text_empty_color_layers_return_before_artboard_factory_lookup() {
        let font = LayerCountingFont::new(None);
        let mut text = Text::default(); // Deliberately unattached: no renderer factory.
        let mut renderer = nuxie_render_api::NullRenderer::new();
        text.draw_color_glyph(
            &mut renderer,
            font.clone(),
            1,
            &[65],
            Mat2D::default(),
            0xff000000,
            1.0,
            Mat2D::default(),
        );
        assert_eq!(font.extractions.load(Ordering::SeqCst), 1);
    }
}

#[cfg(test)]
mod lines_bounds_tests {
    use super::*;
    use crate::mechanical_port::source::{
        assets::font_asset::FontAsset,
        core::CoreArena,
        text::{font_hb::HbFont, text_style::TextStyle},
    };

    const FONT: &[u8] = include_bytes!("../../../../../../fixtures/fonts/roboto-input.ttf");
    // Calmly's title block: as wide as the page.
    const WIDE: f32 = 362.0;

    fn close(got: f32, want: f32) {
        assert!((got - want).abs() < 0.01, "expected {want}, got {got}");
    }
    fn close_box(got: Aabb, want: Aabb) {
        close(got.min_x, want.min_x);
        close(got.min_y, want.min_y);
        close(got.max_x, want.max_x);
        close(got.max_y, want.max_y);
    }
    /// One 32-point style with a 40-point line.
    fn style(arena: &CoreArena) -> CoreHandle {
        let asset = arena.insert(FontAsset::default());
        FontAsset::set_font_occurrence(&asset, Some(HbFont::decode(FONT).unwrap()));
        let style = arena.insert(TextStylePaint::default());
        TextStyle::set_asset_occurrence(&style, Some(asset));
        style
            .with_mut(|object| {
                let style = object.as_text_style_mut().unwrap();
                style.base.set_font_size_value(32.0);
                style.base.set_line_height_value(40.0);
            })
            .unwrap();
        style
    }
    /// A standalone text, shaped and drawn once through its ordinary path update.
    fn shaped(style: &CoreHandle, value: &str, configure: impl FnOnce(&mut Text)) -> Text {
        let mut run = TextValueRun::default();
        run.base.set_text_value(value.into());
        run.set_style(style.clone());
        let mut text = Text::default();
        configure(&mut text);
        text.all_runs
            .push(TextValueRunHandle::Runtime(Rc::new(RefCell::new(run))));
        text.update_after_transform_super(ComponentDirt::PATH);
        text
    }
    /// The auto-width layout box, which hugs a single line's extent.
    fn natural(style: &CoreHandle, value: &str) -> Aabb {
        shaped(style, value, |_| {}).local_bounds()
    }
    fn wide(text: &mut Text, align: u8) {
        text.base.set_sizing_value_value(1);
        text.base.set_width_value(WIDE);
        text.base.set_align_value_value(align);
    }
    fn fixed(text: &mut Text, width: f32, height: f32, overflow: u8) {
        text.base.set_sizing_value_value(2);
        text.base.set_width_value(width);
        text.base.set_height_value(height);
        text.base.set_overflow_value_value(overflow);
    }

    #[test]
    fn lines_bounds_follow_left_center_and_right_alignment_in_a_wide_box() {
        let arena = CoreArena::default();
        let style = style(&arena);
        let line = natural(&style, "Calmly Plus");
        assert!(line.width() > 100.0 && line.width() < WIDE);
        for (align, left) in [
            (0, 0.0),
            (2, (WIDE - line.width()) / 2.0),
            (1, WIDE - line.width()),
        ] {
            let text = shaped(&style, "Calmly Plus", |text| wide(text, align));
            close(text.local_bounds().width(), WIDE);
            // The run of letters, not the box.
            close_box(
                text.lines_bounds().unwrap(),
                Aabb::new(left, line.min_y, left + line.width(), line.max_y),
            );
        }
    }

    #[test]
    fn lines_bounds_spans_every_wrapped_line() {
        let arena = CoreArena::default();
        let style = style(&arena);
        for (align, share) in [(0, 0.0), (2, 0.5)] {
            let text = shaped(&style, "Calmly Plus Annual", |text| {
                text.base.set_sizing_value_value(1);
                text.base.set_width_value(140.0);
                text.base.set_align_value_value(align);
            });
            assert!(text.ordered_lines().len() >= 2, "the fixture must wrap");
            let widest = GlyphLine::compute_max_width(&text.lines[0], &text.shape[0].runs);
            let left = (140.0 - widest) * share;
            close_box(
                text.lines_bounds().unwrap(),
                Aabb::new(
                    left,
                    0.0,
                    left + widest,
                    text.lines[0].last().unwrap().bottom,
                ),
            );
            close(
                text.lines_bounds().unwrap().max_y,
                text.local_bounds().max_y,
            );
        }
    }

    #[test]
    fn lines_bounds_from_the_baseline_origin_is_the_auto_width_box() {
        let arena = CoreArena::default();
        let style = style(&arena);
        let text = shaped(&style, "Calmly Plus", |text| {
            text.base.set_origin_value_value(1);
        });
        let lines = text.lines_bounds().unwrap();
        close(lines.min_y, -text.lines[0][0].baseline);
        assert!(lines.min_y < 0.0);
        close_box(lines, text.local_bounds());
    }

    #[test]
    fn lines_bounds_stays_in_raw_line_coordinates_for_a_nonzero_origin() {
        let arena = CoreArena::default();
        let style = style(&arena);
        let at_zero = shaped(&style, "Calmly Plus", |_| {});
        let text = shaped(&style, "Calmly Plus", |text| {
            text.base.set_origin_x_value(0.5);
            text.base.set_origin_y_value(0.5);
        });
        let lines = text.lines_bounds().unwrap();
        close_box(lines, at_zero.lines_bounds().unwrap());
        // internal_transform takes the raw lines to the text's local box.
        close_box(
            text.internal_transform().map_bounding_box(lines),
            text.local_bounds(),
        );
        assert!(text.local_bounds().min_x < 0.0);
    }

    #[test]
    fn lines_bounds_ignores_middle_and_bottom_vertical_alignment() {
        let arena = CoreArena::default();
        let style = style(&arena);
        let top = shaped(&style, "Calmly Plus", |text| fixed(text, WIDE, 200.0, 0));
        for (vertical, check) in [(2u8, 0usize), (1, 1)] {
            let text = shaped(&style, "Calmly Plus", |text| {
                fixed(text, WIDE, 200.0, 0);
                text.base.set_vertical_align_value_value(vertical);
            });
            let lines = text.lines_bounds().unwrap();
            close_box(lines, top.lines_bounds().unwrap());
            let placed = text.internal_transform().map_bounding_box(lines);
            if check == 0 {
                close(placed.center().y, 100.0);
            } else {
                close(placed.max_y, 200.0);
            }
        }
    }

    #[test]
    fn lines_bounds_is_the_unfitted_line_under_overflow_fit() {
        let arena = CoreArena::default();
        let style = style(&arena);
        let line = natural(&style, "Calmly Plus");
        assert!(line.width() > 100.0, "the fixture must overflow");
        let text = shaped(&style, "Calmly Plus", |text| {
            fixed(text, 100.0, 200.0, 4);
            text.base.set_wrap_value_value(1);
        });
        let lines = text.lines_bounds().unwrap();
        close_box(lines, line);
        // Fit scales the whole layout evenly; the raw lines are unscaled.
        let placed = text.internal_transform().map_bounding_box(lines);
        close(placed.min_x, 0.0);
        close(placed.width(), 100.0);
    }

    #[test]
    fn lines_bounds_ends_after_the_ellipsis() {
        let arena = CoreArena::default();
        let style = style(&arena);
        let line = natural(&style, "Calmly Plus");
        let text = shaped(&style, "Calmly Plus", |text| {
            fixed(text, 100.0, 200.0, 3);
            text.base.set_wrap_value_value(1);
        });
        let lines = text.lines_bounds().unwrap();
        let drawn = &text.ordered_lines()[0];
        assert_eq!(text.ordered_lines().len(), 1);
        assert_eq!(drawn.runs().len(), 2, "truncated letters, then the ellipsis");
        let right = drawn.glyph_line().start_x
            + drawn
                .into_iter()
                .map(|(run, glyph)| run.advances[glyph as usize])
                .sum::<f32>();
        let ellipsis: f32 = drawn.runs().last().unwrap().advances.iter().sum();
        assert!(ellipsis > 0.0);
        close(lines.max_x, right);
        assert!(lines.max_x <= 100.01 && lines.max_x < line.width());
        close(lines.min_x, 0.0);
        close(lines.height(), line.height());
    }

    #[test]
    fn empty_or_unshaped_text_has_no_lines_bounds() {
        let arena = CoreArena::default();
        let style = style(&arena);
        assert!(shaped(&style, "", |_| {}).lines_bounds().is_none());
        // A style without a font shapes nothing.
        let fontless = arena.insert(TextStylePaint::default());
        assert!(shaped(&fontless, "Calmly Plus", |_| {}).lines_bounds().is_none());
        assert!(Text::default().lines_bounds().is_none());
    }

    #[test]
    fn lines_bounds_ends_at_the_last_paragraphs_bottom_without_trailing_spacing() {
        let arena = CoreArena::default();
        let style = style(&arena);
        let text = shaped(&style, "Calmly\nPlus", |text| {
            text.base.set_paragraph_spacing_value(10.0);
        });
        assert_eq!(text.lines.len(), 2);
        let bottom = text.lines[0].last().unwrap().bottom + 10.0 + text.lines[1].last().unwrap().bottom;
        let widest = GlyphLine::compute_max_width(&text.lines[0], &text.shape[0].runs)
            .max(GlyphLine::compute_max_width(&text.lines[1], &text.shape[1].runs));
        close_box(
            text.lines_bounds().unwrap(),
            Aabb::new(0.0, 0.0, widest, bottom),
        );
        close(text.lines_bounds().unwrap().max_y, text.local_bounds().max_y);
    }

    #[test]
    fn lines_bounds_leaves_out_lines_the_overflow_skips() {
        let arena = CoreArena::default();
        let style = style(&arena);
        let value = "Calmly\nPlus\nAnnual";
        // Top alignment stops before the first line that leaves the box.
        let text = shaped(&style, value, |text| fixed(text, WIDE, 60.0, 1));
        assert_eq!(text.ordered_lines().len(), 1);
        close_box(
            text.lines_bounds().unwrap(),
            Aabb::new(
                0.0,
                0.0,
                GlyphLine::compute_max_width(&text.lines[0], &text.shape[0].runs),
                text.lines[0][0].bottom,
            ),
        );
        // Bottom alignment skips the leading lines that leave the box.
        let text = shaped(&style, value, |text| {
            fixed(text, WIDE, 60.0, 1);
            text.base.set_vertical_align_value_value(1);
        });
        assert_eq!(text.ordered_lines().len(), 1);
        let top = text.lines[0][0].bottom + text.lines[1][0].bottom;
        close_box(
            text.lines_bounds().unwrap(),
            Aabb::new(
                0.0,
                top,
                GlyphLine::compute_max_width(&text.lines[2], &text.shape[2].runs),
                top + text.lines[2][0].bottom,
            ),
        );
    }

    #[test]
    fn lines_bounds_is_recorded_afresh_when_the_text_changes() {
        let arena = CoreArena::default();
        let style = style(&arena);
        let mut text = shaped(&style, "Calmly Plus", |_| {});
        let first = text.lines_bounds().unwrap();
        // A paint-only rebuild walks the same lines again, not on top of them.
        text.update_after_transform_super(ComponentDirt::PAINT);
        close_box(text.lines_bounds().unwrap(), first);
        let set = |text: &mut Text, value: &str| {
            text.all_runs[0]
                .with_mut(|run| run.base.set_text_value(value.into()))
                .unwrap();
            text.update_after_transform_super(ComponentDirt::PATH);
        };
        set(&mut text, "");
        assert!(text.lines_bounds().is_none());
        set(&mut text, "Calmly");
        close_box(text.lines_bounds().unwrap(), natural(&style, "Calmly"));
        assert!(text.lines_bounds().unwrap().width() < first.width());
    }
}
