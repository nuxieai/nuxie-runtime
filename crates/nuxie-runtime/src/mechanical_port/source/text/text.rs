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
    core::Core,
    dirtyable::Dirtyable,
    generated::text::text_base::TextBase,
    hit_info::HitInfo,
    layout::{
        LayoutDirection, LayoutMeasureMode, LayoutScaleType, layout_participant::LayoutParticipant,
    },
    math::{
        aabb::Aabb,
        mat2d::Mat2D,
        raw_path::{PathDirection, RawPath},
        transform_components::TransformComponents,
        vec2d::Vec2D,
    },
    refcnt::RiveRc,
    renderer::{BlendMode, ImageSampler, RenderImage, RenderPaintStyle, Renderer},
    shapes::{
        paint::color::{ColorInt, color_modulate_opacity},
        shape_paint_path::ShapePaintPath,
    },
    text_engine::{
        Font, GlyphLine, GlyphRun, OrderedLine, Paragraph, TextAlign, TextOrigin, TextOverflow,
        TextRun, TextSizing, TextTrimBottom, TextTrimTop, TextWrap, VerticalTextAlign,
    },
    viewmodel::{
        symbol_type::SymbolType, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_value::ViewModelInstanceValue,
        viewmodel_value_dependent::ViewModelValueDependent,
    },
};
use std::ptr::NonNull;
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
        font: RiveRc<Font>,
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
            font,
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

#[cfg(feature = "with_rive_text")]
pub struct TextValueRunProperty {
    text_value_run: NonNull<TextValueRun>,
    text_value_run_listener: NonNull<TextValueRunListener>,
    instance_value: NonNull<ViewModelInstanceValue>,
    property_key: u16,
    symbol_type: SymbolType,
}

#[cfg(feature = "with_rive_text")]
impl TextValueRunProperty {
    fn new(
        text_value_run: NonNull<TextValueRun>,
        text_value_run_listener: NonNull<TextValueRunListener>,
        instance_value: NonNull<ViewModelInstanceValue>,
        property_key: u16,
        symbol_type: SymbolType,
    ) -> Self {
        Self {
            text_value_run,
            text_value_run_listener,
            instance_value,
            property_key,
            symbol_type,
        }
    }

    fn write_value(&mut self) {
        // The symbol lookup above guarantees the same concrete string value
        // that the upstream as<ViewModelInstanceString>() cast requires.
        let instance_value = unsafe {
            self.instance_value
                .cast::<ViewModelInstanceString>()
                .as_ref()
        };
        let value = instance_value.base.property_value().to_owned();
        match self.symbol_type {
            SymbolType::TextContent => {
                unsafe { self.text_value_run.as_mut() }.set_bound_text(value);
            }
            SymbolType::TextStyle => {
                let style_paints = unsafe {
                    self.text_value_run_listener
                        .as_ref()
                        .text()
                        .as_ref()
                        .text_style_paints()
                };
                for (index, style_paint) in style_paints.iter().copied().enumerate() {
                    let style_paint_ref = unsafe { style_paint.as_ref() };
                    if style_paint_ref.base.base.base.base.base.base.base.name() == value {
                        unsafe { self.text_value_run.as_mut() }.set_style(style_paint);
                        break;
                    } else if index == 0 {
                        unsafe { self.text_value_run.as_mut() }.set_style(style_paint);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(feature = "with_rive_text")]
impl Dirtyable for TextValueRunProperty {
    fn add_dirt(&mut self, _value: ComponentDirt, _recurse: bool) {
        self.write_value();
        unsafe { self.text_value_run_listener.as_mut() }.mark_dirty();
    }
}

#[cfg(feature = "with_rive_text")]
impl ViewModelValueDependent for TextValueRunProperty {
    fn relink_data_bind(&mut self) {}
}

#[cfg(feature = "with_rive_text")]
impl Drop for TextValueRunProperty {
    fn drop(&mut self) {
        let dependent: &mut dyn ViewModelValueDependent = self;
        unsafe { self.instance_value.as_mut() }.remove_dependent(NonNull::from(dependent));
    }
}

#[cfg(feature = "with_rive_text")]
pub struct TextValueRunListener {
    text_value_run: Box<TextValueRun>,
    instance: RiveRc<ViewModelInstance>,
    text: NonNull<Text>,
    properties: Vec<Box<TextValueRunProperty>>,
}

#[cfg(feature = "with_rive_text")]
impl TextValueRunListener {
    fn new(
        mut text_value_run: Box<TextValueRun>,
        instance: RiveRc<ViewModelInstance>,
        text: NonNull<Text>,
    ) -> Box<Self> {
        text_value_run.set_text_component(text);
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
        unsafe { self.text.as_mut() }.mark_shape_dirty();
    }

    fn text(&self) -> NonNull<Text> {
        self.text
    }

    fn text_value_run(&mut self) -> NonNull<TextValueRun> {
        NonNull::from(self.text_value_run.as_mut())
    }

    fn remap(&mut self, instance: RiveRc<ViewModelInstance>) {
        if !RiveRc::ptr_eq(&self.instance, &instance) {
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
    ) -> Option<Box<TextValueRunProperty>> {
        let property_key = match symbol_type {
            SymbolType::TextStyle => {
                crate::mechanical_port::source::generated::text::text_value_run_base::TextValueRunBase::STYLE_ID_PROPERTY_KEY
            }
            SymbolType::TextContent => {
                crate::mechanical_port::source::generated::text::text_value_run_base::TextValueRunBase::TEXT_PROPERTY_KEY
            }
            _ => 0,
        };
        let instance_value = self.instance.property_value_for_symbol(symbol_type)?;
        Some(Box::new(TextValueRunProperty::new(
            NonNull::from(self.text_value_run.as_mut()),
            NonNull::from(&mut *self),
            instance_value,
            property_key,
            symbol_type,
        )))
    }

    fn create_property_listener(&mut self, symbol_type: SymbolType) {
        let Some(mut listener) = self.create_single_property_listener(symbol_type) else {
            return;
        };
        listener.write_value();
        let dependent: &mut dyn ViewModelValueDependent = listener.as_mut();
        unsafe { listener.instance_value.as_mut() }.add_dependent(NonNull::from(dependent));
        self.properties.push(listener);
    }
}

enum TextDrawCommand {
    Style(NonNull<TextStylePaint>),
    ColorGlyph {
        font: RiveRc<Font>,
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
    runs: Vec<NonNull<TextValueRun>>,
    all_runs: Vec<NonNull<TextValueRun>>,
    render_styles: Vec<NonNull<TextStylePaint>>,
    shape: Vec<Paragraph>,
    modifier_shape: Vec<Paragraph>,
    lines: Vec<Vec<GlyphLine>>,
    modifier_lines: Vec<Vec<GlyphLine>>,
    ordered_lines: Vec<OrderedLine>,
    ellipsis_run: GlyphRun,
    clip_rect: RawPath,
    clip_path: ShapePaintPath,
    bounds: Aabb,
    modifier_groups: Vec<NonNull<TextModifierGroup>>,
    styled_text: StyledText,
    modifier_styled_text: StyledText,
    glyph_lookup: GlyphLookup,
    text_style_paints: Vec<NonNull<TextStylePaint>>,
    layout_width: f32,
    layout_height: f32,
    layout_width_scale_type: u8,
    layout_height_scale_type: u8,
    layout_direction: LayoutDirection,
    emoji_image_cache: Vec<((usize, u16), RiveRc<RenderImage>)>,
    draw_commands: Vec<TextDrawCommand>,
    #[cfg(feature = "with_rive_text")]
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
            #[cfg(feature = "with_rive_text")]
            value_run_listeners: Vec::new(),
        }
    }
}

impl Text {
    pub fn mark_shape_dirty(&mut self) {
        self.mark_shape_dirty_layout(true);
    }
    pub fn mark_shape_dirty_layout(&mut self, send_to_layout: bool) {
        #[cfg(not(feature = "with_rive_text"))]
        {
            let _ = send_to_layout;
            return;
        }
        self.base.add_dirt(ComponentDirt::PATH);
        for group in &mut self.modifier_groups {
            unsafe { group.as_mut() }.clear_range_maps();
        }
        self.base.mark_world_transform_dirty();
        #[cfg(feature = "with_rive_layout")]
        if send_to_layout {
            self.base.mark_layout_dirty();
        }
    }
    pub fn mark_paint_dirty(&mut self) {
        #[cfg(not(feature = "with_rive_text"))]
        return;
        self.base.add_dirt(ComponentDirt::PAINT);
    }
    pub fn modifier_shape_dirty(&mut self) {
        #[cfg(not(feature = "with_rive_text"))]
        return;
        self.base.add_dirt(ComponentDirt::PATH);
    }
    pub fn add_run(&mut self, run: &mut TextValueRun) {
        #[cfg(not(feature = "with_rive_text"))]
        {
            let _ = run;
            return;
        }
        self.runs.push(NonNull::from(run));
        self.all_runs.push(NonNull::from(run));
    }
    pub fn add_modifier_group(&mut self, group: &mut TextModifierGroup) {
        #[cfg(not(feature = "with_rive_text"))]
        {
            let _ = group;
            return;
        }
        self.modifier_groups.push(NonNull::from(group));
    }
    pub fn sizing(&self) -> TextSizing {
        unsafe { std::mem::transmute(self.base.sizing_value() as u8) }
    }
    pub fn effective_sizing(&self) -> TextSizing {
        #[cfg(feature = "with_rive_layout")]
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
        unsafe { std::mem::transmute(self.base.overflow_value() as u8) }
    }
    pub fn overflow_visible(&self) -> bool {
        self.overflow() == TextOverflow::Visible
    }
    pub fn text_origin(&self) -> TextOrigin {
        unsafe { std::mem::transmute(self.base.origin_value() as u8) }
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
        unsafe { std::mem::transmute(self.base.wrap_value() as u8) }
    }
    pub fn vertical_align(&self) -> VerticalTextAlign {
        unsafe { std::mem::transmute(self.base.vertical_align_value() as u8) }
    }
    pub fn align(&self) -> TextAlign {
        let value = unsafe { std::mem::transmute(self.base.align_value() as u8) };
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
    pub fn add_style_paint(&mut self, paint: &mut TextStylePaint) {
        self.text_style_paints.push(NonNull::from(paint));
    }
    pub fn style_from_shaper_id(&self, id: u16) -> Option<NonNull<TextStylePaint>> {
        #[cfg(not(feature = "with_rive_text"))]
        {
            let _ = id;
            return None;
        }
        self.runs
            .get(id as usize)
            .and_then(|run| unsafe { run.as_ref() }.style())
    }
    pub fn runs(&self) -> &[NonNull<TextValueRun>] {
        &self.all_runs
    }
    pub fn have_modifiers(&self) -> bool {
        #[cfg(feature = "with_rive_text")]
        {
            !self.modifier_groups.is_empty()
        }
        #[cfg(not(feature = "with_rive_text"))]
        {
            false
        }
    }
    #[cfg(feature = "with_rive_text")]
    pub fn text_style_paints(&self) -> &[NonNull<TextStylePaint>] {
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
        for (run_index, run) in self.all_runs.iter().copied().enumerate() {
            let run = unsafe { run.as_ref() };
            let Some(style) = run.style() else {
                continue;
            };
            let style = unsafe { style.as_ptr().as_mut().unwrap() };
            let Some(font) = style.base.base.font() else {
                continue;
            };
            if run.base.text().is_empty() {
                continue;
            }
            styled.append(
                font,
                style.base.base.base.font_size() * font_scale,
                style.base.base.base.line_height(),
                style.base.base.base.letter_spacing(),
                run.base.text(),
                run_index as u16,
            );
        }
        if with_modifiers {
            let this = self as *const Text;
            for group in &mut self.modifier_groups {
                unsafe { group.as_mut() }.apply_shape_modifiers(unsafe { &*this }, styled);
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
        #[cfg(not(feature = "with_rive_text"))]
        return false;
        self.modifier_groups
            .iter()
            .any(|g| unsafe { g.as_ref() }.needs_shape())
    }
    pub fn update(&mut self, value: ComponentDirt) {
        #[cfg(not(feature = "with_rive_text"))]
        {
            let _ = value;
            return;
        }
        self.base.update(value);
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
                    for group in &mut self.modifier_groups {
                        let group = unsafe { group.as_mut() };
                        group.compute_range_map(
                            styled.unichars(),
                            &self.modifier_shape,
                            &self.modifier_lines,
                            &self.glyph_lookup,
                        );
                        group.compute_coverage(text_size);
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
                    for group in &mut self.modifier_groups {
                        let group = unsafe { group.as_mut() };
                        group.compute_range_map(
                            styled.unichars(),
                            &self.shape,
                            &self.lines,
                            &self.glyph_lookup,
                        );
                        group.compute_coverage(text_size);
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
                unsafe { style.as_mut() }
                    .paints
                    .propagate_opacity(self.base.render_opacity());
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
            unsafe { style.as_mut() }.rewind_path();
        }
        self.render_styles.clear();
        self.draw_commands.clear();
        for run in &mut self.all_runs {
            unsafe { run.as_mut() }.reset_hit_test();
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
        for value_run in self.all_runs.iter().copied() {
            let value_run = unsafe { value_run.as_ref() };
            let Some(mut style) = value_run.style() else {
                continue;
            };
            let style = unsafe { style.as_mut() };
            if style.base.base.font().is_some() && !value_run.base.text().is_empty() {
                max_size = max_size.max(style.base.base.base.font_size());
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
            for group in &mut self.modifier_groups {
                let group = unsafe { group.as_mut() };
                group.compute_coverage(text_size);
                group.reset_text_follow_path();
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
                        for group in &mut self.modifier_groups {
                            let group = unsafe { group.as_mut() };
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
                    }
                    path_transform = Mat2D::from_translate(
                        current_position.x + center_x + offset.x,
                        current_position.y + offset.y,
                    ) * path_transform;

                    let mut value_run = self.all_runs[run.style_id as usize];
                    let value_run_ref = unsafe { value_run.as_mut() };
                    let mut style = value_run_ref.style().expect("TextValueRun style");
                    if run.font.is_color_glyph(glyph_id) {
                        self.draw_commands.push(TextDrawCommand::ColorGlyph {
                            font: run.font.clone(),
                            glyph_id,
                            transform: path_transform,
                            foreground_color: unsafe { style.as_ref() }.foreground_color(),
                            opacity,
                        });
                    } else {
                        let path = run.font.get_path(glyph_id).transform(path_transform);
                        if unsafe { style.as_mut() }.add_path(&path, opacity) {
                            self.render_styles.push(style);
                            unsafe { style.as_mut() }
                                .paints
                                .propagate_opacity(self.base.render_opacity());
                            self.draw_commands.push(TextDrawCommand::Style(style));
                        }
                    }
                    if value_run_ref.is_hit_target() {
                        value_run_ref.add_hit_rect(Aabb::new(
                            current_x,
                            current_y + line.top,
                            current_x + advance,
                            current_y + line.bottom,
                        ));
                    }
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
        #[cfg(feature = "with_rive_layout")]
        self.base.mark_layout_dirty();
        for run in &mut self.all_runs {
            let run = unsafe { run.as_mut() };
            if run.is_hit_target() {
                run.compute_hit_contours();
            }
        }
    }
    pub fn draw(&mut self, renderer: &mut Renderer) {
        #[cfg(not(feature = "with_rive_text"))]
        {
            let _ = renderer;
            return;
        }
        if self.base.needs_save_operation() {
            renderer.save();
        }
        if self.overflow() == TextOverflow::Clipped
            && (!self.clip_path.empty() || self.clip_path.has_render_path())
        {
            let factory = self.base.artboard().factory();
            renderer.clip_path(self.clip_path.render_path_for_factory(factory));
        }
        let world_transform = self.shape_world_transform;
        for index in 0..self.draw_commands.len() {
            match &self.draw_commands[index] {
                TextDrawCommand::Style(mut style) => {
                    unsafe { style.as_mut() }.draw(renderer, &world_transform);
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
        font: RiveRc<Font>,
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
        let factory = self.base.artboard().factory();
        renderer.save();
        renderer.transform(&(world_transform * transform));
        for mut layer in layers {
            if layer.paint_type
                == crate::mechanical_port::source::text_engine::ColorGlyphPaintType::Image
            {
                let key = (font.as_ptr() as *const () as usize, glyph_id);
                let image = if let Some((_, image)) = self
                    .emoji_image_cache
                    .iter()
                    .find(|(cached_key, _)| *cached_key == key)
                {
                    image.clone()
                } else {
                    let image = factory.decode_image(&layer.image_bytes);
                    self.emoji_image_cache.push((key, image.clone()));
                    image
                };
                renderer.save();
                renderer.transform(&Mat2D::new(
                    layer.image_extent_x / layer.image_width as f32,
                    0.0,
                    0.0,
                    layer.image_extent_y / layer.image_height as f32,
                    layer.image_bearing_x,
                    layer.image_bearing_y,
                ));
                renderer.draw_image(
                    &image,
                    ImageSampler::linear_clamp(),
                    BlendMode::SrcOver,
                    opacity,
                );
                renderer.restore();
            } else {
                let mut path = factory.make_render_path(
                    &mut layer.path,
                    crate::mechanical_port::source::renderer::FillRule::NonZero,
                );
                let mut paint = factory.make_render_paint();
                paint.style(RenderPaintStyle::Fill);
                paint.color(color_modulate_opacity(layer.color, opacity));
                renderer.draw_path(&mut path, &mut paint);
            }
        }
        renderer.restore();
    }
    pub fn local_bounds(&self) -> Aabb {
        #[cfg(not(feature = "with_rive_text"))]
        return Aabb::default();
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
        #[cfg(not(feature = "with_rive_text"))]
        {
            let _ = value;
            return;
        }
        if value.intersects(ComponentDirt::WORLD_TRANSFORM) {
            for group in &mut self.modifier_groups {
                unsafe { group.as_mut() }.on_text_world_transform_dirty();
            }
        }
        if value.intersects(ComponentDirt::PATH | ComponentDirt::PAINT) {
            for style in &mut self.render_styles {
                unsafe { style.as_mut() }.paints.invalidate_stroke_effects();
            }
        }
    }
    pub fn compose_world_transform(&mut self) {
        #[cfg(feature = "with_rive_layout")]
        if let (Some(participant), Some(parent)) = (
            self.layout_participant(),
            self.base.parent_transform_component(),
        ) {
            let base = Mat2D::from_translation(Vec2D::new(
                participant.resolved_left() + self.base.origin_x() * participant.resolved_width(),
                participant.resolved_top() + self.base.origin_y() * participant.resolved_height(),
            ));
            self.base
                .set_world_transform(parent.world_transform() * base * self.base.transform());
            return;
        }
        self.base.compose_world_transform();
        self.shape_world_transform = *self.base.world_transform() * self.internal_transform;
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
        #[cfg(not(feature = "with_rive_text"))]
        {
            let _ = (width, width_mode, height, height_mode);
            return Vec2D::default();
        }
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
        #[cfg(not(feature = "with_rive_text"))]
        {
            let _ = (size, w, h, d);
            return;
        }
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
        #[cfg(not(feature = "with_rive_text"))]
        return;
        self.mark_paint_dirty();
        self.base.mark_world_transform_dirty();
    }
    pub fn origin_x_changed(&mut self) {
        #[cfg(not(feature = "with_rive_text"))]
        return;
        self.mark_paint_dirty();
        self.base.mark_world_transform_dirty();
    }
    pub fn origin_y_changed(&mut self) {
        #[cfg(not(feature = "with_rive_text"))]
        return;
        self.mark_paint_dirty();
        self.base.mark_world_transform_dirty();
    }
    pub fn vertical_trim_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn update_list(&mut self, list: Option<&[RiveRc<ViewModelInstanceListItem>]>) {
        let Some(list) = list else {
            return;
        };
        #[cfg(feature = "with_rive_text")]
        {
            self.build_text_style_paints();
            self.all_runs.clear();
            self.all_runs.extend(self.runs.iter().copied());
            let current_size = self.value_run_listeners.len();
            let mut index = 0usize;
            let text = NonNull::from(&mut *self);
            for item in list {
                let Some(instance) = item.view_model_instance() else {
                    continue;
                };
                let text_run = if index < current_size {
                    let listener = &mut self.value_run_listeners[index];
                    listener.remap(instance);
                    listener.text_value_run()
                } else {
                    let listener = TextValueRunListener::new(
                        Box::new(TextValueRun::default()),
                        instance,
                        text,
                    );
                    self.value_run_listeners.push(listener);
                    self.value_run_listeners[index].text_value_run()
                };
                self.all_runs.push(text_run);
                index += 1;
            }
            self.value_run_listeners.truncate(index);
            self.mark_shape_dirty();
        }
        #[cfg(not(feature = "with_rive_text"))]
        let _ = list;
    }
    pub fn build_text_style_paints(&mut self) {
        if self.text_style_paints.is_empty() {
            for child in self.base.children() {
                if let Some(style) = child.as_text_style_paint() {
                    self.text_style_paints.push(style);
                }
            }
        }
    }
}
