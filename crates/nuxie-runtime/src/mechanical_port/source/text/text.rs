use super::{
    glyph_lookup::GlyphLookup,
    text_interface::TextInterface,
    text_modifier_group::{TextModifierGroup, TransformGlyphArg},
    text_style_paint::TextStylePaint,
    text_value_run::TextValueRun,
    utf::Utf,
};
use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::{Core, CoreHandle},
    dirtyable::Dirtyable,
    generated::text::text_base::TextBase,
    hit_info::HitInfo,
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
    text_engine::{
        FontRef, GlyphLine, GlyphRun, OrderedLine, Paragraph, TextAlign, TextOrigin, TextOverflow,
        TextRun, TextSizing, TextTrimBottom, TextTrimTop, TextWrap, VerticalTextAlign,
    },
    viewmodel::{
        symbol_type::SymbolType,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_value::{ValueDependentHandle, ViewModelInstanceValue},
        viewmodel_value_dependent::ViewModelValueDependent,
    },
};
use std::{cell::RefCell, rc::Rc};
#[derive(Default)]
pub struct StyledText {
    value: Vec<u32>,
    runs: Vec<TextRun>,
}
impl StyledText {
    pub fn empty(&self) -> bool {
        self.runs.is_empty()
    }
    pub fn clear(&mut self) {
        self.value.clear();
        self.runs.clear();
    }
    pub fn append(
        &mut self,
        font: FontRef,
        size: f32,
        line_height: f32,
        letter_spacing: f32,
        text: &str,
        style_id: u16,
    ) {
        let start = self.value.len();
        let mut bytes = text.as_bytes();
        while !bytes.is_empty() {
            self.value.push(Utf::next_utf8(&mut bytes));
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
    pub fn runs(&self) -> &[TextRun] {
        &self.runs
    }
    pub fn swap_runs(&mut self, other: &mut Vec<TextRun>) {
        std::mem::swap(&mut self.runs, other);
    }
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
    fn with<R>(&self, use_run: impl FnOnce(&TextValueRun) -> R) -> Option<R> {
        match self {
            Self::Core(run) => run
                .with(|run| run.as_text_value_run().map(use_run))
                .flatten(),
            Self::Runtime(run) => Some(use_run(&run.borrow())),
        }
    }

    fn with_mut<R>(&self, use_run: impl FnOnce(&mut TextValueRun) -> R) -> Option<R> {
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
    property_key: u16,
    symbol_type: SymbolType,
}

impl TextValueRunProperty {
    fn new(
        text_value_run: TextValueRunHandle,
        text: CoreHandle,
        instance_value: CoreHandle,
        property_key: u16,
        symbol_type: SymbolType,
    ) -> Self {
        Self {
            text_value_run,
            text,
            instance_value,
            property_key,
            symbol_type,
        }
    }

    fn write_value(&mut self) {
        // The symbol lookup above guarantees the same concrete string value
        // that the upstream as<ViewModelInstanceString>() cast requires.
        let Some(value) = self
            .instance_value
            .with_downcast::<ViewModelInstanceString, _>(|value| {
                value.base.property_value().to_owned()
            })
        else {
            return;
        };
        match self.symbol_type {
            SymbolType::TextContent => {
                self.text_value_run
                    .with_mut(|run| run.set_bound_text(value));
            }
            SymbolType::TextStyle => {
                let style_paints = self
                    .text
                    .with(|text| text.as_text().map(|text| text.text_style_paints().to_vec()))
                    .flatten()
                    .unwrap_or_default();
                for (index, style_paint) in style_paints.into_iter().enumerate() {
                    let matches = style_paint
                        .with(|style| {
                            style
                                .as_text_style()
                                .is_some_and(|style| style.base.name() == value)
                        })
                        .unwrap_or(false);
                    if matches || index == 0 {
                        self.text_value_run
                            .with_mut(|run| run.set_style(style_paint.clone()));
                        if matches {
                            break;
                        }
                    }
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
}

pub struct TextValueRunListener {
    text_value_run: Rc<RefCell<TextValueRun>>,
    instance: CoreHandle,
    text: CoreHandle,
    properties: Vec<Rc<RefCell<dyn ViewModelValueDependent>>>,
}

impl TextValueRunListener {
    fn new(text_value_run: TextValueRun, instance: CoreHandle, text: CoreHandle) -> Box<Self> {
        let text_value_run = Rc::new(RefCell::new(text_value_run));
        text_value_run.borrow_mut().set_text_component(text.clone());
        let mut listener = Box::new(Self {
            text_value_run,
            instance,
            text,
            properties: Vec::new(),
        });
        listener.create_properties();
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

    fn remap(&mut self, instance: CoreHandle) {
        if self.instance != instance {
            self.properties.clear();
            self.instance = instance;
            self.create_properties();
        }
    }

    fn create_properties(&mut self) {
        self.properties.clear();
        self.create_property_listener(SymbolType::TextStyle);
        self.create_property_listener(SymbolType::TextContent);
    }

    fn create_single_property_listener(
        &mut self,
        symbol_type: SymbolType,
    ) -> Option<TextValueRunProperty> {
        let property_key = match symbol_type {
            SymbolType::TextStyle => {
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
        Some(TextValueRunProperty::new(
            self.text_value_run(),
            self.text.clone(),
            instance_value,
            property_key,
            symbol_type,
        ))
    }

    fn create_property_listener(&mut self, symbol_type: SymbolType) {
        let Some(listener) = self.create_single_property_listener(symbol_type) else {
            return;
        };
        let instance_value = listener.instance_value.clone();
        let listener = Rc::new(RefCell::new(listener));
        listener.borrow_mut().write_value();
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
    ColorGlyph {
        font: FontRef,
        glyph_id: u16,
        transform: Mat2D,
        foreground_color: ColorInt,
        opacity: f32,
    },
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
                let metrics = run.font.line_metrics();
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
                    descent_px = descent_px.max(run.font.line_metrics().descent * run.size);
                }
                bottom_trim = (descent_band - descent_px).max(0.0);
            }
            break;
        }
    }
    (top_trim, bottom_trim)
}

pub struct Text {
    pub base: TextBase,
    pub internal_transform: Mat2D,
    pub shape_world_transform: Mat2D,
    runs: Vec<CoreHandle>,
    all_runs: Vec<TextValueRunHandle>,
    render_styles: Vec<CoreHandle>,
    shape: Vec<Paragraph>,
    modifier_shape: Vec<Paragraph>,
    lines: Vec<Vec<GlyphLine>>,
    modifier_lines: Vec<Vec<GlyphLine>>,
    ordered_lines: Vec<OrderedLine>,
    ellipsis_run: GlyphRun,
    clip_rect: RawPath,
    clip_path: ShapePaintPath,
    bounds: Aabb,
    modifier_groups: Vec<CoreHandle>,
    styled_text: StyledText,
    modifier_styled_text: StyledText,
    glyph_lookup: GlyphLookup,
    text_style_paints: Vec<CoreHandle>,
    layout_width: f32,
    layout_height: f32,
    layout_width_scale_type: u8,
    layout_height_scale_type: u8,
    layout_direction: LayoutDirection,
    emoji_image_cache: Vec<(FontRef, u16, Option<Rc<dyn nuxie_render_api::RenderImage>>)>,
    draw_commands: Vec<TextDrawCommand>,
    value_run_listeners: Vec<Box<TextValueRunListener>>,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            base: TextBase::default(),
            internal_transform: Mat2D::default(),
            shape_world_transform: Mat2D::default(),
            runs: Vec::new(),
            all_runs: Vec::new(),
            render_styles: Vec::new(),
            shape: Vec::new(),
            modifier_shape: Vec::new(),
            lines: Vec::new(),
            modifier_lines: Vec::new(),
            ordered_lines: Vec::new(),
            ellipsis_run: GlyphRun::default(),
            clip_rect: RawPath::default(),
            clip_path: ShapePaintPath::clockwise(),
            bounds: Aabb::default(),
            modifier_groups: Vec::new(),
            styled_text: StyledText::default(),
            modifier_styled_text: StyledText::default(),
            glyph_lookup: GlyphLookup::default(),
            text_style_paints: Vec::new(),
            layout_width: f32::NAN,
            layout_height: f32::NAN,
            layout_width_scale_type: u8::MAX,
            layout_height_scale_type: u8::MAX,
            layout_direction: LayoutDirection::Inherit,
            emoji_image_cache: Vec::new(),
            draw_commands: Vec::new(),
            value_run_listeners: Vec::new(),
        }
    }
}

impl Text {
    pub fn mark_shape_dirty(&mut self) {
        self.mark_shape_dirty_layout(true);
    }
    pub fn mark_shape_dirty_layout(&mut self, send_to_layout: bool) {
        self.base.add_dirt(ComponentDirt::PATH);
        for group in &mut self.modifier_groups {
            group.with_mut(|group| {
                if let Some(group) = group.as_text_modifier_group_mut() {
                    group.clear_range_maps();
                }
            });
        }
        self.base.mark_world_transform_dirty();
        if send_to_layout {
            self.base.mark_layout_dirty();
        }
    }
    pub fn mark_paint_dirty(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT);
    }
    pub fn modifier_shape_dirty(&mut self) {
        self.base.add_dirt(ComponentDirt::PATH);
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
            value @ 3.. => TextSizing::Unknown(value),
            0 => TextSizing::AutoWidth,
        }
    }
    pub fn effective_sizing(&self) -> TextSizing {
        if self.is_participating_in_layout() {
            let width_is_box = self.layout_width_scale_type == LayoutScaleType::Fixed as u8
                || self.layout_width_scale_type == LayoutScaleType::Fill as u8;
            let height_is_box = self.layout_height_scale_type == LayoutScaleType::Fixed as u8
                || self.layout_height_scale_type == LayoutScaleType::Fill as u8;
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
            value @ 6.. => TextOverflow::Unknown(value),
            0 => TextOverflow::Visible,
        }
    }
    pub fn overflow_visible(&self) -> bool {
        self.overflow() == TextOverflow::Visible
    }
    pub fn text_origin(&self) -> TextOrigin {
        match self.base.origin_value() {
            1 => TextOrigin::Baseline,
            value @ 2.. => TextOrigin::Unknown(value),
            0 => TextOrigin::Top,
        }
    }
    pub fn vertical_trim_top(&self) -> TextTrimTop {
        crate::mechanical_port::source::text_engine::text_trim_top(self.base.vertical_trim_value())
    }
    pub fn vertical_trim_bottom(&self) -> TextTrimBottom {
        crate::mechanical_port::source::text_engine::text_trim_bottom(
            self.base.vertical_trim_value(),
        )
    }
    pub fn wrap(&self) -> TextWrap {
        match self.base.wrap_value() {
            1 => TextWrap::NoWrap,
            value @ 2.. => TextWrap::Unknown(value),
            0 => TextWrap::Wrap,
        }
    }
    pub fn vertical_align(&self) -> VerticalTextAlign {
        match self.base.vertical_align_value() {
            1 => VerticalTextAlign::Bottom,
            2 => VerticalTextAlign::Middle,
            value @ 3.. => VerticalTextAlign::Unknown(value),
            0 => VerticalTextAlign::Top,
        }
    }
    pub fn align(&self) -> TextAlign {
        let value = match self.base.align_value() {
            1 => TextAlign::Right,
            2 => TextAlign::Center,
            value @ 3.. => TextAlign::Unknown(value),
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
        self.effective_sizing() == TextSizing::Fixed || !self.layout_width.is_nan()
    }
    pub fn add_style_paint(&mut self, paint: CoreHandle) {
        self.text_style_paints.push(paint);
    }
    pub fn style_from_shaper_id(&self, id: u16) -> Option<CoreHandle> {
        self.runs.get(id as usize).and_then(|run| {
            run.with(|run| run.as_text_value_run().and_then(TextValueRun::style))
                .flatten()
        })
    }
    pub fn runs(&self) -> &[TextValueRunHandle] {
        &self.all_runs
    }
    pub(crate) fn inferred_semantic_data(&self) -> Option<ResolvedSemanticData> {
        let label = self
            .all_runs
            .iter()
            .filter_map(|run| run.with(|run| run.base.text().to_owned()))
            .collect::<String>();
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
    pub fn make_styled(
        &mut self,
        styled: &mut StyledText,
        with_modifiers: bool,
        font_scale: f32,
    ) -> bool {
        styled.clear();
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
                .flatten()
            else {
                continue;
            };
            if text.is_empty() {
                continue;
            }
            styled.append(
                font,
                font_size * font_scale,
                line_height,
                letter_spacing,
                &text,
                run_index as u16,
            );
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
    ) -> Vec<Vec<GlyphLine>> {
        let auto_width = width == -1.0;
        let mut paragraph_width = width;
        let mut lines = Vec::with_capacity(paragraphs.len());
        for paragraph in paragraphs {
            let paragraph_lines = GlyphLine::break_lines(
                &paragraph.runs,
                if auto_width || wrap == TextWrap::NoWrap {
                    -1.0
                } else {
                    width
                },
            );
            if auto_width {
                paragraph_width = paragraph_width.max(GlyphLine::compute_max_width(
                    &paragraph_lines,
                    &paragraph.runs,
                ));
            }
            lines.push(paragraph_lines);
        }
        for (paragraph_index, paragraph) in paragraphs.iter().enumerate() {
            GlyphLine::compute_line_spacing(
                paragraph_index == 0,
                &mut lines[paragraph_index],
                &paragraph.runs,
                paragraph_width,
                align,
            );
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
            let parent_is_layout_not_artboard = self.base.parent().is_some_and(|parent| {
                crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::is_type_of(parent.core_type())
                    && parent.core_type()
                        != crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY
            });
            let font_scale = if self.overflow() == TextOverflow::FitFontSize {
                self.fit_font_scale()
            } else {
                1.0
            };

            if precompute_modifier_coverage {
                let mut styled = std::mem::take(&mut self.modifier_styled_text);
                if self.make_styled(&mut styled, false, font_scale) {
                    let runs = styled.runs();
                    self.modifier_shape = runs[0].font.shape_text(styled.unichars(), runs, 0);
                    self.modifier_lines = Self::break_lines(
                        &self.modifier_shape,
                        if self.effective_sizing() == TextSizing::AutoWidth
                            && !parent_is_layout_not_artboard
                        {
                            -1.0
                        } else {
                            self.effective_width()
                        },
                        self.align(),
                        self.wrap(),
                    );
                    self.glyph_lookup
                        .compute(styled.unichars(), &self.modifier_shape);
                    let text_size = styled.unichars().len() as u32;
                    for group in self.modifier_groups.clone() {
                        group.with_mut(|group| {
                            if let Some(group) = group.as_text_modifier_group_mut() {
                                group.compute_range_map(
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
            if self.make_styled(&mut styled, true, font_scale) {
                let runs = styled.runs();
                self.shape = runs[0].font.shape_text(styled.unichars(), runs, 0);
                self.lines = Self::break_lines(
                    &self.shape,
                    if self.effective_sizing() == TextSizing::AutoWidth
                        && !parent_is_layout_not_artboard
                    {
                        -1.0
                    } else {
                        self.effective_width()
                    },
                    self.align(),
                    self.wrap(),
                );
                if !precompute_modifier_coverage && !self.modifier_groups.is_empty() {
                    self.glyph_lookup.compute(styled.unichars(), &self.shape);
                    let text_size = styled.unichars().len() as u32;
                    for group in self.modifier_groups.clone() {
                        group.with_mut(|group| {
                            if let Some(group) = group.as_text_modifier_group_mut() {
                                group.compute_range_map(
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
            self.emoji_image_cache.clear();
            self.build_render_styles();
        } else if value.intersects(ComponentDirt::PAINT) {
            self.build_render_styles();
        } else if value.intersects(ComponentDirt::RENDER_OPACITY) {
            for style in &mut self.render_styles {
                style.with_downcast_mut::<TextStylePaint, _>(|style| {
                    style.paints.propagate_opacity(self.base.render_opacity());
                });
            }
        }

        if value
            .intersects(ComponentDirt::WORLD_TRANSFORM | ComponentDirt::PATH | ComponentDirt::PAINT)
        {
            self.clip_path.rewind();
            self.shape_world_transform = *self.base.world_transform() * self.internal_transform;
            self.clip_path
                .add_path(&self.clip_rect, Some(&self.shape_world_transform));
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
                self.ordered_lines.push(OrderedLine::new(
                    p,
                    line,
                    self.effective_width(),
                    false,
                    false,
                    &mut self.ellipsis_run,
                    y + line.baseline,
                ));
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
        for run in &mut self.all_runs {
            run.with_mut(TextValueRun::reset_hit_test);
        }
    }

    fn compute_bounds_info(&self) -> TextBoundsInfo {
        let paragraph_space = self.base.paragraph_spacing();
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
                max_size = max_size.max(font_size);
            }
        }
        let sizing = self.effective_sizing();
        if max_size <= 1.0 || (sizing == TextSizing::AutoWidth && !self.overflow_as_fixed()) {
            return 1.0;
        }

        let box_width = self.effective_width();
        let box_height = self.effective_height();
        let paragraph_space = self.base.paragraph_spacing();
        let mut styled = StyledText::default();
        let mut fits = |this: &mut Text, top_size: i32| -> bool {
            let scale = top_size as f32 / max_size;
            if !this.make_styled(&mut styled, true, scale) {
                return true;
            }
            let runs = styled.runs();
            let shape = runs[0].font.shape_text(styled.unichars(), runs, 0);
            let lines = Text::break_lines(&shape, box_width, this.align(), this.wrap());
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
                y += paragraph_space;
            }
            measured_width <= box_width && (!this.overflow_as_fixed() || y <= box_height)
        };
        let mut low = 1i32;
        let mut high = (max_size as i32).max(1);
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
        if self.shape.is_empty() {
            self.bounds = Aabb::new(0.0, 0.0, 0.0, 0.0);
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
                        group.reset_text_follow_path();
                    }
                });
            }
        }

        let paragraph_space = self.base.paragraph_spacing();
        let auto_size_max_y = if self.layout_height.is_nan() {
            info.min_y
                .max(info.total_height - paragraph_space - info.top_trim - info.bottom_trim)
        } else {
            info.min_y + self.layout_height
        };
        self.bounds = match self.effective_sizing() {
            TextSizing::AutoWidth => Aabb::new(
                0.0,
                info.min_y,
                if self.layout_width.is_nan() {
                    info.max_width
                } else {
                    self.layout_width
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

        self.ordered_lines.clear();
        let mut current_y = info.min_y - info.top_trim;
        let mut line_index = 0i32;
        let mut minimum_x = f32::MAX;
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
                self.ordered_lines.push(OrderedLine::new(
                    paragraph,
                    line,
                    self.effective_width(),
                    info.ellipsis_line == line_index,
                    info.is_ellipsis_line_last,
                    &mut self.ellipsis_run,
                    render_y,
                ));
                let ordered_line = self.ordered_lines.last().unwrap();
                let mut current_x = line.start_x;
                minimum_x = minimum_x.min(current_x);
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
                    path_transform = Mat2D::from_translate(
                        current_position.x + center_x + offset.x,
                        current_position.y + offset.y,
                    ) * path_transform;

                    let value_run = self.all_runs[run.style_id as usize].clone();
                    let style = value_run
                        .with(TextValueRun::style)
                        .flatten()
                        .expect("TextValueRun style");
                    if run.font.is_color_glyph(glyph_id) {
                        let foreground_color = style
                            .with_downcast::<TextStylePaint, _>(TextStylePaint::foreground_color)
                            .unwrap_or(0xff000000);
                        self.draw_commands.push(TextDrawCommand::ColorGlyph {
                            font: run.font.clone(),
                            glyph_id,
                            transform: path_transform,
                            foreground_color,
                            opacity,
                        });
                    } else {
                        let path = run.font.get_path(glyph_id).transform(path_transform);
                        let first_path = style
                            .with_downcast_mut::<TextStylePaint, _>(|style| {
                                style.add_path(&path, opacity)
                            })
                            .unwrap_or(false);
                        if first_path {
                            self.render_styles.push(style.clone());
                            style.with_downcast_mut::<TextStylePaint, _>(|style| {
                                style.paints.propagate_opacity(self.base.render_opacity());
                            });
                            self.draw_commands.push(TextDrawCommand::Style(style));
                        }
                    }
                    value_run.with_mut(|value_run| {
                        if value_run.is_hit_target() {
                            value_run.add_hit_rect(Aabb::new(
                                current_x,
                                current_y + line.top,
                                current_x + advance,
                                current_y + line.bottom,
                            ));
                        }
                    });
                    current_x += advance;
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
            current_y += paragraph_space;
        }

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
        self.internal_transform =
            Mat2D::from_scale_and_translation(scale, scale, x_offset, y_offset);
        self.base.mark_layout_dirty();
        for run in &mut self.all_runs {
            run.with_mut(|run| {
                if run.is_hit_target() {
                    run.compute_hit_contours();
                }
            });
        }
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
        let world_transform = self.shape_world_transform;
        let blend_mode = self.base.blend_mode().into();
        for index in 0..self.draw_commands.len() {
            match &self.draw_commands[index] {
                TextDrawCommand::Style(style) => {
                    style.with_downcast_mut::<TextStylePaint, _>(|style| {
                        style.draw(renderer, &world_transform, blend_mode)
                    });
                }
                TextDrawCommand::ColorGlyph {
                    font,
                    glyph_id,
                    transform,
                    foreground_color,
                    opacity,
                } => self.draw_color_glyph(
                    renderer,
                    font.clone(),
                    *glyph_id,
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
        transform: Mat2D,
        foreground_color: ColorInt,
        opacity: f32,
        world_transform: Mat2D,
    ) {
        let mut layers = Vec::new();
        if font.get_color_layers(glyph_id, &mut layers, foreground_color) == 0 {
            return;
        }
        let factory = self
            .base
            .artboard()
            .factory()
            .expect("Text requires its Artboard renderer factory");
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(
            *(world_transform * transform).values(),
        ));
        for mut layer in layers {
            if layer.paint_type
                == crate::mechanical_port::source::text_engine::ColorGlyphPaintType::Image
            {
                let image = if let Some((_, _, image)) =
                    self.emoji_image_cache
                        .iter()
                        .find(|(cached_font, cached_glyph, _)| {
                            Rc::ptr_eq(cached_font, &font) && *cached_glyph == glyph_id
                        }) {
                    image.clone()
                } else {
                    let image = factory
                        .with_factory_mut(|factory| factory.decode_image(&layer.image_bytes))
                        .ok()
                        .map(Rc::<dyn nuxie_render_api::RenderImage>::from);
                    self.emoji_image_cache
                        .push((font.clone(), glyph_id, image.clone()));
                    image
                };
                let Some(image) = image else {
                    continue;
                };
                renderer.save();
                renderer.transform(nuxie_render_api::Mat2D(
                    *Mat2D::new(
                        layer.image_extent_x / layer.image_width as f32,
                        0.0,
                        0.0,
                        layer.image_extent_y / layer.image_height as f32,
                        layer.image_bearing_x,
                        layer.image_bearing_y,
                    )
                    .values(),
                ));
                renderer.draw_image(
                    Some(image.as_ref()),
                    ImageSampler::LINEAR_CLAMP,
                    BlendMode::SrcOver,
                    opacity,
                );
                renderer.restore();
            } else {
                let (path, mut paint) = factory.with_factory_mut(|factory| {
                    (
                        factory.make_render_path(
                            to_render_raw_path(&layer.path),
                            crate::mechanical_port::source::renderer::FillRule::NonZero,
                        ),
                        factory.make_render_paint(),
                    )
                });
                paint.style(RenderPaintStyle::Fill);
                paint.color(color_modulate_opacity(layer.color, opacity));
                renderer.draw_path(path.as_ref(), paint.as_ref());
            }
        }
        renderer.restore();
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
        self.bounds.width()
    }
    pub fn computed_height(&self) -> f32 {
        self.bounds.height()
    }
    pub fn on_dirty(&mut self, value: ComponentDirt) {
        if value.intersects(ComponentDirt::WORLD_TRANSFORM) {
            for group in &mut self.modifier_groups {
                group.with_mut(|group| {
                    if let Some(group) = group.as_text_modifier_group_mut() {
                        group.on_text_world_transform_dirty();
                    }
                });
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
    pub(crate) fn try_compose_world_transform_override(&mut self) -> bool {
        let participant = self.base.children().iter().find_map(|child| {
            child
                .with(|child| {
                    child.as_any().downcast_ref::<crate::mechanical_port::source::layout::layout_participant::LayoutParticipant>().map(|participant| {
                        (
                            participant.resolved_left(),
                            participant.resolved_top(),
                            participant.resolved_width(),
                            participant.resolved_height(),
                        )
                    })
                })
                .flatten()
        });
        let parent_world = self.base.parent_transform_component().and_then(|parent| {
            parent
                .with(|parent| {
                    parent
                        .as_world_transform_component()
                        .map(|parent| *parent.world_transform())
                })
                .flatten()
        });
        if let (Some((left, top, width, height)), Some(parent_world)) = (participant, parent_world)
        {
            let base = Mat2D::from_translation(Vec2D::new(
                left + self.base.origin_x() * width,
                top + self.base.origin_y() * height,
            ));
            self.base
                .set_world_transform(parent_world * base * *self.base.transform());
            return true;
        }
        false
    }

    pub fn layout_participant(&self) -> Option<&LayoutParticipant> {
        self.base
            .children()
            .iter()
            .find_map(crate::mechanical_port::source::core::Core::as_layout_participant)
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
        self.measure(max)
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
    fn measure(&mut self, max: Vec2D) -> Vec2D {
        let mut styled = std::mem::take(&mut self.styled_text);
        if !self.make_styled(&mut styled, true, 1.0) {
            self.styled_text = styled;
            return Vec2D::default();
        }
        let paragraph_space = self.base.paragraph_spacing();
        let runs = styled.runs();
        let shape = runs[0].font.shape_text(styled.unichars(), runs, 0);
        let measuring_width = match self.effective_sizing() {
            TextSizing::AutoHeight | TextSizing::Fixed => self.base.width(),
            TextSizing::AutoWidth => f32::MAX,
            TextSizing::Unknown(_) => f32::MAX,
        };
        let measuring_wrap =
            if max.x == f32::MAX && self.effective_sizing() != TextSizing::AutoHeight {
                TextWrap::NoWrap
            } else {
                self.wrap()
            };
        let lines = Self::break_lines(
            &shape,
            max.x.min(measuring_width),
            self.align(),
            measuring_wrap,
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
            &shape,
            self.vertical_trim_top(),
            self.vertical_trim_bottom(),
        );
        let bounds = match self.sizing() {
            TextSizing::AutoWidth => Vec2D::new(
                max_width,
                min_y.max(computed_height - top_trim - bottom_trim),
            ),
            TextSizing::AutoHeight => Vec2D::new(
                self.base.width(),
                min_y.max(computed_height - top_trim - bottom_trim),
            ),
            TextSizing::Fixed => Vec2D::new(self.base.width(), min_y + self.base.height()),
            TextSizing::Unknown(_) => Vec2D::default(),
        };
        self.styled_text = styled;
        Vec2D::new(max.x.min(bounds.x), max.y.min(bounds.y))
    }
    pub fn align_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn sizing_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn overflow_value_changed(&mut self) {
        if self.effective_sizing() != TextSizing::AutoWidth {
            self.mark_shape_dirty();
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
        self.mark_paint_dirty();
    }
    pub fn origin_value_changed(&mut self) {
        self.mark_paint_dirty();
        self.base.mark_world_transform_dirty();
    }
    pub fn origin_x_changed(&mut self) {
        self.mark_paint_dirty();
        self.base.mark_world_transform_dirty();
    }
    pub fn origin_y_changed(&mut self) {
        self.mark_paint_dirty();
        self.base.mark_world_transform_dirty();
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
            let current_size = self.value_run_listeners.len();
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
                    let listener = &mut self.value_run_listeners[index];
                    listener.remap(instance);
                    listener.text_value_run()
                } else {
                    let listener =
                        TextValueRunListener::new(TextValueRun::default(), instance, text.clone());
                    self.value_run_listeners.push(listener);
                    self.value_run_listeners[index].text_value_run()
                };
                self.all_runs.push(text_run);
                index += 1;
            }
            self.value_run_listeners.truncate(index);
            self.mark_shape_dirty();
        }
    }
    pub fn build_text_style_paints(&mut self) {
        if self.text_style_paints.is_empty() {
            for child in self.base.children() {
                if child.with_downcast::<TextStylePaint, _>(|_| ()).is_some() {
                    self.text_style_paints.push(child.clone());
                }
            }
        }
    }
}
