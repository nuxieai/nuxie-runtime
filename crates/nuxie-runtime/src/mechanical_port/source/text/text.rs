use super::{
    glyph_lookup::GlyphLookup,
    text_interface::TextInterface,
    text_modifier_group::{TextModifierGroup, TransformGlyphArg},
    text_style_background::TextStyleBackground,
    text_style_paint::TextStylePaint,
    text_value_run::TextValueRun,
    utf::Utf,
};
use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::{Core, CoreHandle},
    dirtyable::Dirtyable,
    generated::{core_registry::CoreCapabilities, text::text_base::TextBase},
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
        self.write_value_with_text(None);
    }

    fn write_value_with_text(&mut self, mut text: Option<&mut Text>) {
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
                if let Some(text) = text.as_deref_mut() {
                    self.text_value_run
                        .with_mut(|run| run.set_bound_text_with_borrowed_text(value, text));
                } else {
                    self.text_value_run
                        .with_mut(|run| run.set_bound_text(value));
                }
            }
            SymbolType::TextStyle => {
                let owned_style_paints;
                let style_paints = if let Some(text) = text.as_deref() {
                    text.text_style_paints()
                } else {
                    owned_style_paints = self
                        .text
                        .with(|text| text.as_text().map(|text| text.text_style_paints().to_vec()))
                        .flatten()
                        .unwrap_or_default();
                    &owned_style_paints
                };
                for (index, style_paint) in style_paints.iter().enumerate() {
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
        self.create_property_listener(SymbolType::TextContent, text.as_deref_mut());
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

impl Text {
    pub const TYPE_KEY: u16 = TextBase::TYPE_KEY;

    /// Retain host-installed CSS semantics on a clone, but rebuild all derived
    /// shaping, measurement and drawing state for the new occurrence.
    pub fn clone_core(&self) -> Self {
        let mut twin = self.base.clone_into(&mut Self::default());
        twin.css_wrap_policy = self.css_wrap_policy;
        twin.css_nowrap_alignment = self.css_nowrap_alignment;
        twin.experimental_css_ellipsis = self.experimental_css_ellipsis;
        twin.css_underlines = self.css_underlines.clone();
        twin.css_strikethroughs = self.css_strikethroughs.clone();
        twin
    }
}

struct CssDecorationPath {
    strike: Option<super::css_decoration::StrikethroughStripe>,
    bounds: Aabb,
    snapped_bounds: Option<Aabb>,
    after_glyphs: bool,
    color: u32,
    full: ShapePaintPath,
    fallback: ShapePaintPath,
    exclusions: Vec<Aabb>,
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
    // Last fitFontSize multiplier. Read paragraph spacing through the guarded
    // helper so switching overflow modes cannot reuse a stale fitted gap.
    fitted_font_scale: f32,
    css_nowrap_alignment: bool,
    measured_css_baseline: Option<f32>,
    experimental_css_ellipsis: bool,
    css_underlines: Vec<super::css_decoration::ResolvedUnderline>,
    css_strikethroughs: Vec<super::css_decoration::ResolvedStrikethrough>,
    css_decoration_paths: Vec<CssDecorationPath>,
    css_decoration_paint: Option<Box<dyn nuxie_render_api::RenderPaint>>,
    css_wrap_policy: Option<super::css_pre_wrap::CssWrapPolicy>,
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
            clip_path: ShapePaintPath::default(),
            bounds: Aabb::default(),
            fitted_font_scale: 1.0,
            css_nowrap_alignment: false,
            measured_css_baseline: None,
            experimental_css_ellipsis: false,
            css_underlines: Vec::new(),
            css_strikethroughs: Vec::new(),
            css_decoration_paths: Vec::new(),
            css_decoration_paint: None,
            css_wrap_policy: None,
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
    /// Install resolved solid underlines on this occurrence. These are not Rive
    /// properties; reinstall on a fresh import. Empty input removes them.
    pub fn set_css_underlines(
        &mut self,
        underlines: Vec<super::css_decoration::ResolvedUnderline>,
    ) {
        if self.css_underlines != underlines {
            self.css_underlines = underlines;
            self.mark_shape_dirty();
        }
    }

    /// Install resolved strikethroughs, painted after glyphs. Reinstall on a
    /// fresh import; resizing retains them. Empty input removes them.
    pub fn set_css_strikethroughs(
        &mut self,
        lines: Vec<super::css_decoration::ResolvedStrikethrough>,
    ) {
        if self.css_strikethroughs != lines {
            self.css_strikethroughs = lines;
            self.mark_shape_dirty();
        }
    }

    /// Opt into CSS nowrap overflow alignment for this text occurrence. Hosts
    /// must reinstall this policy on a fresh import; clones retain it, and it is not a Rive
    /// property. Resizing and reshaping retain it. Ordinary wrapped text and
    /// default Rive interpretation are unchanged.
    pub fn set_css_nowrap_alignment(&mut self, enabled: bool) {
        if self.css_nowrap_alignment != enabled {
            self.css_nowrap_alignment = enabled;
            self.mark_shape_dirty();
        }
    }

    /// Validate the static LTR single-line occurrence contract before installation.
    /// Hosts must validate every target before installing any occurrence policy.
    pub fn validate_css_single_line_ellipsis(&self) -> Result<(), &'static str> {
        if self.wrap() != TextWrap::NoWrap || self.css_wrap_policy.is_some() {
            return Err("CSS single-line ellipsis requires nowrap text");
        }
        if self.have_modifiers() {
            return Err("CSS single-line ellipsis does not support text modifiers");
        }
        let mut font_metrics = None;
        for run in &self.all_runs {
            let style = run.with(TextValueRun::style).flatten()
                .ok_or("CSS single-line ellipsis requires resolved text styles")?;
            let current = style.with_downcast::<TextStylePaint, _>(|style| {
                let base = &style.base.base;
                (base.font_asset_id(), base.font_size(), base.line_height())
            }).ok_or("CSS single-line ellipsis requires paint styles")?;
            if font_metrics.is_some_and(|first| first != current) {
                return Err("CSS single-line ellipsis does not support mixed source fonts or sizes");
            }
            font_metrics = Some(current);
        }
        let source = self.settled_text_value();
        for character in source.chars() {
            if matches!(character, '\n' | '\r' | '\t' | '\u{2028}' | '\u{2029}') {
                return Err("CSS single-line ellipsis does not support breaks or tabs");
            }
            use unicode_bidi::BidiClass;
            if matches!(unicode_bidi::bidi_class(character), BidiClass::B | BidiClass::S) {
                return Err("CSS single-line ellipsis does not support breaks or tabs");
            }
            if matches!(unicode_bidi::bidi_class(character),
                BidiClass::R | BidiClass::AL | BidiClass::AN |
                BidiClass::RLE | BidiClass::RLO | BidiClass::LRE | BidiClass::LRO |
                BidiClass::PDF | BidiClass::LRI | BidiClass::RLI | BidiClass::FSI | BidiClass::PDI) {
                return Err("CSS single-line ellipsis does not support bidi text");
            }
        }
        Ok(())
    }

    /// Install the checked static single-line policy. Reinstall on fresh imports;
    /// resizing retains original source and recomputes truncation. The host must
    /// supply CSS shaping precision and an enclosing CSS overflow clip.
    pub fn install_css_single_line_ellipsis(&mut self) -> Result<(), &'static str> {
        self.validate_css_single_line_ellipsis()?;
        if self.base.set_overflow_value_value(3) {
            self.overflow_value_changed();
        }
        self.set_css_nowrap_alignment(true);
        self.set_experimental_css_ellipsis(true);
        Ok(())
    }

    /// Probe-only single-line CSS ellipsis experiment; no compiler capability yet.
    pub fn set_experimental_css_ellipsis(&mut self, enabled: bool) {
        if self.experimental_css_ellipsis != enabled {
            self.experimental_css_ellipsis = enabled;
            self.mark_shape_dirty();
        }
    }

    /// Opt into preserved CSS soft wrapping for this occurrence. Reinstall on
    /// fresh imports; resizing, reshaping and font replacement retain the policy.
    pub fn set_css_pre_wrap(&mut self, enabled: bool) {
        self.set_css_wrap_policy(enabled.then_some(super::css_pre_wrap::CssWrapPolicy::PreWrap));
    }

    /// Opt into pre-line hanging for this occurrence.
    /// The compiler supplies text with collapsed ASCII spaces and preserved LF.
    pub fn set_css_pre_line(&mut self, enabled: bool) {
        self.set_css_wrap_policy(enabled.then_some(super::css_pre_wrap::CssWrapPolicy::PreLine));
    }

    /// Opt into CSS normal wrapping: words overflow instead of emergency glyph
    /// splitting. The compiler supplies already-collapsed normal whitespace.
    pub fn set_css_normal_wrap(&mut self, enabled: bool) {
        self.set_css_wrap_policy(enabled.then_some(super::css_pre_wrap::CssWrapPolicy::Normal));
    }

    fn set_css_wrap_policy(&mut self, policy: Option<super::css_pre_wrap::CssWrapPolicy>) {
        if self.css_wrap_policy != policy {
            self.css_wrap_policy = policy;
            self.mark_shape_dirty();
        }
    }

    fn break_lines_for_layout(
        &self,
        paragraphs: &mut [Paragraph],
        source: &[u32],
        width: f32,
        align: TextAlign,
        wrap: TextWrap,
    ) -> Vec<Vec<GlyphLine>> {
        if let Some(policy) = self.css_wrap_policy {
            return super::css_pre_wrap::break_lines(
                paragraphs, source, width, align, wrap, policy,
            );
        }
        let mut lines = Self::break_lines(paragraphs, width, align, wrap);
        if self.css_nowrap_alignment && wrap == TextWrap::NoWrap {
            for line in lines.iter_mut().flatten() {
                line.start_x = line.start_x.max(0.0);
            }
        }
        lines
    }

    pub fn internal_transform(&self) -> Mat2D {
        self.internal_transform
    }

    pub fn shape_world_transform(&self) -> &Mat2D {
        &self.shape_world_transform
    }

    pub fn mark_shape_dirty(&mut self) {
        self.mark_shape_dirty_layout(true);
    }
    pub fn mark_shape_dirty_layout(&mut self, send_to_layout: bool) {
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
        owner.with_mut(|object| object.component_add_dirt(ComponentDirt::PATH, false));
        let groups = owner
            .with_downcast::<Text, _>(|text| text.modifier_groups.clone())
            .expect("live Text");
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
            if text.is_empty() {
                continue;
            }
            // Preserve the negative font-metric sentinel; authored absolute
            // line height and letter spacing scale uniformly with the font.
            let line_height = if line_height >= 0.0 {
                line_height * font_scale
            } else {
                line_height
            };
            styled.append(
                font,
                font_size * font_scale,
                line_height,
                letter_spacing * font_scale,
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
                    let runs = styled.runs();
                    let mut shape = runs[0]
                        .font
                        .as_ref()
                        .expect("shaped text retains its font")
                        .shape_text(styled.unichars(), runs, 0);
                    self.modifier_lines = self.break_lines_for_layout(
                        &mut shape,
                        styled.unichars(),
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
                    self.modifier_shape = shape;
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
            if self.make_styled(&mut styled, true, font_scale) {
                let runs = styled.runs();
                let mut shape = runs[0]
                    .font
                    .as_ref()
                    .expect("shaped text retains its font")
                    .shape_text(styled.unichars(), runs, 0);
                self.lines = self.break_lines_for_layout(
                    &mut shape,
                    styled.unichars(),
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
                self.shape = shape;
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
        self.css_decoration_paths.clear();
        for style in &mut self.render_styles {
            style.with_downcast_mut::<TextStylePaint, _>(TextStylePaint::rewind_path);
        }
        self.render_styles.clear();
        self.draw_commands.clear();

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
        let mut styled = StyledText::default();
        let mut fits = |this: &mut Text, top_size: i32| -> bool {
            let scale = top_size as f32 / max_size;
            if !this.make_styled(&mut styled, true, scale) {
                return true;
            }
            let runs = styled.runs();
            let mut shape = runs[0]
                .font
                .as_ref()
                .expect("shaped text retains its font")
                .shape_text(styled.unichars(), runs, 0);
            let lines = this.break_lines_for_layout(
                &mut shape,
                styled.unichars(),
                box_width,
                this.align(),
                this.wrap(),
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
                        group.reset_text_follow_path(self);
                    }
                });
            }
        }

        let paragraph_space = self.fit_paragraph_spacing();
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
                if self.experimental_css_ellipsis {
                    assert!(self.shape.len() == 1 && paragraph_lines.len() == 1 && !has_modifiers, "CSS ellipsis probe requires one unmodified LTR line");
                    let prepared = super::css_ellipsis::prepare_runs(
                        self.styled_text.unichars(),
                        &paragraph.runs,
                        &self.styled_text.runs()[0],
                        self.effective_width(),
                    ).expect("CSS ellipsis probe received unsupported shaping");
                    if let Some(prepared) = prepared {
                        *self.ordered_lines.last_mut().unwrap() = OrderedLine::from_css_runs(prepared.runs, line, render_y);
                    }
                }
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
                    if run
                        .font
                        .as_ref()
                        .expect("shaped text retains its font")
                        .is_color_glyph(glyph_id)
                    {
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
                            transform: path_transform,
                            foreground_color,
                            opacity,
                        });
                    } else {
                        let path = run
                            .font
                            .as_ref()
                            .expect("shaped text retains its font")
                            .get_path(glyph_id)
                            .transform(path_transform);
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
                            self.draw_commands
                                .push(TextDrawCommand::Style(style.clone()));
                        }
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
                            background.with_downcast_mut::<TextStyleBackground, _>(|background| {
                                background.add_rect(glyph_bounds);
                            });
                        }
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
            current_y += self.fit_paragraph_spacing();
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
        let stripes = self
            .css_underlines
            .iter()
            .flat_map(|line| {
                line.build_stripes(&self.ordered_lines, self.styled_text.unichars())
                    .into_iter()
                    .map(move |stripe| (false, line.color(), stripe, None))
            })
            .chain(self.css_strikethroughs.iter().flat_map(|line| {
                line.build_stripes(&self.ordered_lines)
                    .into_iter()
                    .map(move |stripe| {
                        (
                            true,
                            line.color(),
                            super::css_decoration::UnderlineStripe {
                                bounds: stripe.bounds,
                                exclusions: Vec::new(),
                            },
                            Some(stripe),
                        )
                    })
            }));
        for (after_glyphs, color, stripe, strike) in stripes {
            let mut raw = RawPath::default();
            raw.add_rect(
                stripe.bounds,
                crate::source::math::path_types::PathDirection::Clockwise,
            );
            let mut full = ShapePaintPath::default();
            full.add_path(&raw, None);
            raw.rewind();
            stripe.append_fallback(&mut raw);
            let mut fallback = ShapePaintPath::default();
            fallback.add_path(&raw, None);
            self.css_decoration_paths.push(CssDecorationPath {
                strike,
                bounds: stripe.bounds,
                snapped_bounds: None,
                after_glyphs,
                color,
                full,
                fallback,
                exclusions: stripe.exclusions,
            });
        }
        self.internal_transform =
            Mat2D::from_scale_and_translation(scale, scale, x_offset, y_offset);
        self.base.mark_layout_node_dirty();
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
        // Backgrounds precede every glyph, in style child order.
        for style in &self.text_style_paints {
            if let Some(background) = style
                .with_downcast::<TextStylePaint, _>(TextStylePaint::background)
                .flatten()
            {
                background.with_downcast_mut::<TextStyleBackground, _>(|background| {
                    background.draw(renderer, &world_transform, blend_mode);
                });
            }
        }
        self.draw_css_decorations(renderer, &world_transform, blend_mode, false);
        for index in 0..self.draw_commands.len() {
            match &self.draw_commands[index] {
                TextDrawCommand::Style(style) => {
                    if renderer.supports_glyph_runs()
                        && self.draw_solid_glyph_style(renderer, style, blend_mode)
                    {
                        continue;
                    }
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
        self.draw_css_decorations(renderer, &world_transform, blend_mode, true);
        if self.base.needs_save_operation() {
            renderer.restore();
        }
    }

    fn draw_css_decorations(
        &mut self,
        renderer: &mut Renderer,
        world: &Mat2D,
        blend: BlendMode,
        after_glyphs: bool,
    ) {
        if !self
            .css_decoration_paths
            .iter()
            .any(|line| line.after_glyphs == after_glyphs)
        {
            return;
        }
        let Some(factory) = self
            .base
            .with_artboard(|artboard| artboard.factory())
            .flatten()
        else {
            return;
        };
        let paint = self
            .css_decoration_paint
            .get_or_insert_with(|| factory.with_factory_mut(|factory| factory.make_render_paint()));
        paint.style(RenderPaintStyle::Fill);
        paint.blend_mode(blend);
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(*world.values()));
        for line in self
            .css_decoration_paths
            .iter_mut()
            .filter(|line| line.after_glyphs == after_glyphs)
        {
            let values = world.values();
            let paint_bounds = if let Some(strike) = line.strike {
                Some(strike.paint_bounds(values[5]))
            } else if values[0] == 1.0 && values[1] == 0.0 && values[2] == 0.0 && values[3] == 1.0 {
                // Snap after CSS layout translation, before the host's DPR or
                // transform. Local snapping alone leaves fractional containers
                // with an antialiased stripe spread over two pixel rows.
                let top = (line.bounds.min_y + values[5] + 0.5).floor() - values[5];
                Some(Aabb::new(line.bounds.min_x, top, line.bounds.max_x,
                    top + line.bounds.max_y - line.bounds.min_y))
            } else {
                Some(line.bounds)
            };
            if let Some(bounds) = paint_bounds {
                if line.snapped_bounds != Some(bounds) {
                    let mut raw = RawPath::default();
                    raw.add_rect(
                        bounds,
                        crate::source::math::path_types::PathDirection::Clockwise,
                    );
                    line.full = ShapePaintPath::default();
                    line.full.add_path(&raw, None);
                    raw.rewind();
                    super::css_decoration::UnderlineStripe {
                        bounds,
                        exclusions: line.exclusions.clone(),
                    }.append_fallback(&mut raw);
                    line.fallback = ShapePaintPath::default();
                    line.fallback.add_path(&raw, None);
                    line.snapped_bounds = Some(bounds);
                }
            }
            paint.color(color_modulate_opacity(
                line.color,
                self.base.render_opacity(),
            ));
            renderer.save();
            let clipped = line.exclusions.iter().all(|rect| {
                renderer.clip_out_rect(nuxie_render_api::Aabb::new(
                    rect.min_x, rect.min_y, rect.max_x, rect.max_y,
                ))
            });
            if clipped {
                renderer.draw_path(line.full.render_path(&factory), paint.as_ref());
            }
            renderer.restore();
            if !clipped && !line.fallback.empty() {
                // Declining a later exclusion must discard all earlier clips
                // before drawing the explicit geometric fallback.
                renderer.draw_path(line.fallback.render_path(&factory), paint.as_ref());
            }
        }
        renderer.restore();
    }

    fn draw_solid_glyph_style(
        &self,
        renderer: &mut Renderer,
        style: &CoreHandle,
        blend: nuxie_render_api::BlendMode,
    ) -> bool {
        use super::font_hb::HbFont;
        use nuxie_render_api::{GlyphFontRef, PositionedGlyph, RenderGlyphRun};
        if self.have_modifiers()
            || !(matches!(self.overflow(), TextOverflow::Visible | TextOverflow::Clipped)
                || (self.experimental_css_ellipsis && self.overflow() == TextOverflow::Ellipsis))
            || self
                .draw_commands
                .iter()
                .any(|command| matches!(command, TextDrawCommand::ColorGlyph { .. }))
        {
            return false;
        }
        let Some(color) = style
            .with_downcast::<TextStylePaint, _>(TextStylePaint::solid_glyph_color)
            .flatten()
        else {
            return false;
        };
        let mut selected_font: Option<FontRef> = None;
        let mut size = 0.0;
        let mut glyphs = Vec::new();
        for line in self.ordered_lines() {
            let mut x = line.glyph_line().start_x;
            for (run, index) in line {
                let i = index as usize;
                if self.style_from_shaper_id(run.style_id).as_ref() == Some(style) {
                    let Some(font) = run.font.as_ref() else {
                        return false;
                    };
                    if let Some(selected) = selected_font.as_ref() {
                        if !Rc::ptr_eq(selected, font) || size != run.size {
                            return false;
                        }
                    } else {
                        selected_font = Some(font.clone());
                        size = run.size;
                    }
                    if glyphs.len() == 16_384 {
                        return false;
                    }
                    glyphs.push(PositionedGlyph {
                        id: run.glyphs[i],
                        x: x + run.offsets[i].x,
                        y: line.y() + run.offsets[i].y,
                    });
                }
                x += run.advances[i];
            }
        }
        let Some(font) = selected_font else {
            return false;
        };
        if font.get_axis_count() != 0 {
            return false;
        }
        let Some(font) = font.as_any().downcast_ref::<HbFont>() else {
            return false;
        };
        let bytes = font.source_bytes();
        let request = RenderGlyphRun {
            font: GlyphFontRef {
                bytes: &bytes,
                face_index: font.face_index(),
                variations: &[],
            },
            glyphs: &glyphs,
            font_size: size,
            color,
            blend_mode: blend,
        };
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(
            *self.shape_world_transform.values(),
        ));
        let consumed = renderer.draw_glyph_run(&request);
        renderer.restore();
        consumed
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
            .with_artboard(|artboard| artboard.factory())
            .flatten()
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
            for group in self.modifier_groups.clone() {
                TextModifierGroup::on_text_world_transform_dirty(&group, self);
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
                        .map(|parent| *parent.world_transform())
                })
                .flatten()
        });
        if let (Some(translation), Some(parent_world)) = (participant, parent_world) {
            let base = Mat2D::from_translation(translation);
            let transform = *self.base.transform();
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
    /// First alphabetic baseline from the most recent intrinsic measurement.
    /// Read by the explicitly enabled CSS layout baseline channel only.
    pub fn measured_css_baseline(&self) -> Option<f32> {
        self.measured_css_baseline
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
    fn measure(&mut self, max: Vec2D, exact_width: Option<f32>) -> Vec2D {
        self.measured_css_baseline = None;
        let mut styled = std::mem::take(&mut self.styled_text);
        if !self.make_styled(&mut styled, true, 1.0) {
            self.styled_text = styled;
            return Vec2D::default();
        }
        let paragraph_space = self.base.paragraph_spacing();
        let runs = styled.runs();
        let mut shape = runs[0]
            .font
            .as_ref()
            .expect("shaped text retains its font")
            .shape_text(styled.unichars(), runs, 0);
        // Layout can stretch a text child beyond its authored fallback width.
        // Measure at that exact width, just as control_size will later draw it;
        // capping it at base.width reserves height for lines that never render.
        let measuring_width = exact_width.unwrap_or_else(|| match self.effective_sizing() {
            TextSizing::AutoHeight | TextSizing::Fixed => self.base.width(),
            TextSizing::AutoWidth => f32::MAX,
            TextSizing::Unknown(_) => f32::MAX,
        });
        let measuring_wrap =
            if max.x == f32::MAX && self.effective_sizing() != TextSizing::AutoHeight {
                TextWrap::NoWrap
            } else {
                self.wrap()
            };
        // CSS intrinsic width is based on unwrapped content, not the widest
        // resulting wrapped line. A fit-content box can therefore be wider
        // than every line it contains; nowrap content can exceed available space.
        let css_intrinsic_width = if self.sizing() == TextSizing::AutoWidth
            && (self.css_wrap_policy.is_some() || self.css_nowrap_alignment)
        {
            let intrinsic_advance = |available, wrap| {
                let mut intrinsic_shape = shape.clone();
                let intrinsic_lines = self.break_lines_for_layout(
                    &mut intrinsic_shape, styled.unichars(), available,
                    TextAlign::Left, wrap,
                );
                intrinsic_shape.iter().zip(&intrinsic_lines)
                    .flat_map(|(paragraph, lines)| lines.iter().map(move |line| {
                        let start = &paragraph.runs[line.start_run_index as usize];
                        let end = &paragraph.runs[line.end_run_index as usize];
                        end.xpos[line.end_glyph_index as usize] - start.xpos[line.start_glyph_index as usize]
                    })).fold(0.0f32, f32::max)
            };
            Some(exact_width.unwrap_or_else(|| {
                let maximum = intrinsic_advance(f32::MAX, TextWrap::NoWrap);
                if self.wrap() == TextWrap::NoWrap || maximum <= max.x {
                    maximum
                } else {
                    // Fit-content cannot be narrower than its longest
                    // unbreakable segment, even with limited available width.
                    let minimum = intrinsic_advance(0.0, self.wrap());
                    maximum.min(max.x.max(minimum))
                }
            }))
        } else { None };
        let lines = self.break_lines_for_layout(
            &mut shape,
            styled.unichars(),
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
        self.measured_css_baseline = lines.first().and_then(|lines| lines.first()).map(|line| {
            let origin = if self.text_origin() == TextOrigin::Baseline { -line.baseline } else { 0.0 };
            line.baseline + origin - top_trim + self.base.y()
        });
        let bounds = match self.sizing() {
            TextSizing::AutoWidth => Vec2D::new(
                css_intrinsic_width.unwrap_or(max_width),
                min_y.max(computed_height - top_trim - bottom_trim),
            ),
            TextSizing::AutoHeight => Vec2D::new(
                // CSS nowrap text still contributes its intrinsic advance to
                // an auto-sized ancestor. The authored fallback width can be
                // zero; an exact stretch constraint takes precedence later.
                exact_width.unwrap_or_else(|| {
                    if self.css_nowrap_alignment { max_width } else { self.base.width() }
                }),
                min_y.max(computed_height - top_trim - bottom_trim),
            ),
            TextSizing::Fixed => Vec2D::new(self.base.width(), min_y + self.base.height()),
            TextSizing::Unknown(_) => Vec2D::default(),
        };
        self.styled_text = styled;
        Vec2D::new(if css_intrinsic_width.is_some() { bounds.x } else { max.x.min(bounds.x) }, max.y.min(bounds.y))
    }
    pub fn align_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn sizing_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn overflow_value_changed(&mut self) {
        if self.effective_sizing() != TextSizing::AutoWidth || self.overflow_as_fixed() {
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
        if self.overflow() == TextOverflow::FitFontSize {
            self.mark_shape_dirty();
        } else {
            self.mark_paint_dirty();
        }
    }
    pub fn origin_value_changed(&mut self) {
        self.mark_paint_dirty();
        CoreCapabilities::world_transform_mark_dirty(self);
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

    fn css_test_font_bytes() -> Vec<u8> {
        let font_path = std::env::var_os("NUXIE_TEXT_TEST_FONT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                let root = std::env::var_os("RIVE_RUNTIME_DIR")
                    .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
                std::path::PathBuf::from(root)
                    .join("tests/unit_tests/assets/fonts/Inter_18pt-Regular.ttf")
            });
        std::fs::read(font_path).expect("pinned text fixture font")
    }

    #[test]
    fn css_nowrap_alignment_is_opt_in_per_line_and_survives_resizing() {
        use crate::mechanical_port::source::{text::font_hb::HbFont, text_engine::TextRun};
        let bytes = css_test_font_bytes();
        let font = HbFont::decode(&bytes).unwrap();
        let chars: Vec<u32> = "one two three four five six seven\nshort"
            .chars()
            .map(u32::from)
            .collect();
        let shape = font.shape_text(
            &chars,
            &[TextRun {
                font: Some(font.clone()),
                size: 20.0,
                line_height: 30.0,
                letter_spacing: 0.0,
                unichar_count: chars.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        );
        let mut text = Text::default();
        for align in [TextAlign::Center, TextAlign::Right] {
            let origins = |text: &Text, width, wrap| -> Vec<f32> {
                text.break_lines_for_layout(&mut shape.clone(), &chars, width, align, wrap)
                    .iter()
                    .flatten()
                    .map(|line| line.start_x)
                    .collect()
            };
            text.set_css_nowrap_alignment(false);
            let legacy = origins(&text, 224.0, TextWrap::NoWrap);
            assert_eq!(legacy.len(), 2);
            assert!(legacy[0] < 0.0);
            assert!(legacy[1] > 0.0);
            let wrapped = origins(&text, 224.0, TextWrap::Wrap);
            let wide = origins(&text, 752.0, TextWrap::NoWrap);
            text.set_css_nowrap_alignment(true);
            assert_eq!(
                origins(&text, 224.0, TextWrap::NoWrap),
                vec![0.0, legacy[1]]
            );
            assert_eq!(origins(&text, 752.0, TextWrap::NoWrap), wide);
            assert_eq!(
                origins(&text, 224.0, TextWrap::NoWrap),
                vec![0.0, legacy[1]]
            );
            assert_eq!(origins(&text, 224.0, TextWrap::Wrap), wrapped);
            assert_eq!(origins(&Text::default(), 224.0, TextWrap::NoWrap), legacy);
            text.set_css_nowrap_alignment(false);
            assert_eq!(origins(&text, 224.0, TextWrap::NoWrap), legacy);
        }
    }

    #[test]
    fn css_pre_wrap_hangs_spaces_and_preserves_words_without_changing_legacy() {
        use crate::mechanical_port::source::text::font_hb::HbFont;
        let bytes = css_test_font_bytes();
        let font = HbFont::decode(&bytes).unwrap();
        let mut text = Text::default();
        for (value, expected) in [
            (
                "SupercalifragilisticexpialidociousSupercalifragilisticexpialidocious".to_owned(),
                1,
            ),
            (" ".repeat(56), 1),
            (format!("one{}\ntwo", " ".repeat(60)), 2),
            ("  one  \n  two  ".to_owned(), 2),
            (
                "one two three four five six seven eight nine  ".to_owned(),
                2,
            ),
        ] {
            let chars: Vec<u32> = value.chars().map(u32::from).collect();
            let shape = font.shape_text(
                &chars,
                &[TextRun {
                    font: Some(font.clone()),
                    size: 20.0,
                    line_height: 30.0,
                    letter_spacing: 0.0,
                    unichar_count: chars.len() as u32,
                    script: u32::from_be_bytes(*b"Latn"),
                    style_id: 0,
                    level: 0,
                }],
                0,
            );
            let lines = |text: &Text, width| {
                text.break_lines_for_layout(
                    &mut shape.clone(),
                    &chars,
                    width,
                    TextAlign::Center,
                    TextWrap::Wrap,
                )
            };
            text.set_css_pre_wrap(false);
            let legacy = format!("{:?}", lines(&text, 224.0));
            text.set_css_pre_wrap(true);
            let narrow = lines(&text, 224.0);
            assert_eq!(
                narrow.iter().map(Vec::len).sum::<usize>(),
                expected,
                "{value:?}"
            );
            assert!(narrow.iter().flatten().all(|line| line.start_x >= 0.0));
            if value.starts_with("Super") || value.starts_with("one ") && value.contains('\n') {
                assert_eq!(
                    narrow[0][0].start_x, 0.0,
                    "overflow must not hide the first word"
                );
                assert_ne!(
                    format!("{narrow:?}"),
                    legacy,
                    "fixture must exercise the legacy defect"
                );
            }
            let _ = lines(&text, 752.0);
            assert_eq!(format!("{:?}", lines(&text, 224.0)), format!("{narrow:?}"));
            assert_eq!(format!("{:?}", lines(&Text::default(), 224.0)), legacy);
            text.set_css_pre_wrap(false);
            assert_eq!(format!("{:?}", lines(&text, 224.0)), legacy);
        }
    }

    #[test]
    fn css_pre_line_hangs_forced_end_spaces_without_changing_pre_wrap() {
        use crate::mechanical_port::source::text::font_hb::HbFont;
        let font = HbFont::decode(&css_test_font_bytes()).unwrap();
        let mut text = Text::default();
        for value in ["word\u{2003}\u{2003}", "word\u{a0}\u{a0}\nnext"] {
            let chars: Vec<u32> = value.chars().map(u32::from).collect();
            let shape = font.shape_text(
                &chars,
                &[TextRun {
                    font: Some(font.clone()),
                    size: 20.0,
                    line_height: 30.0,
                    letter_spacing: 0.0,
                    unichar_count: chars.len() as u32,
                    script: u32::from_be_bytes(*b"Latn"),
                    style_id: 0,
                    level: 0,
                }],
                0,
            );
            let lines = |text: &Text, width| {
                text.break_lines_for_layout(
                    &mut shape.clone(),
                    &chars,
                    width,
                    TextAlign::Right,
                    TextWrap::Wrap,
                )
            };
            let legacy = format!("{:?}", lines(&Text::default(), 224.0));
            text.set_css_pre_wrap(true);
            let preserved = lines(&text, 224.0);
            text.set_css_pre_line(true);
            let collapsed = lines(&text, 224.0);
            assert!(collapsed[0][0].start_x > preserved[0][0].start_x);
            assert_eq!(
                collapsed[0][0].end_glyph_index, preserved[0][0].end_glyph_index,
                "noncollapsible spaces retain source and glyph range while hanging"
            );
            assert!(
                (lines(&text, 752.0)[0][0].start_x - collapsed[0][0].start_x - 528.0).abs() < 0.001
            );
            assert_eq!(
                format!("{:?}", lines(&text, 224.0)),
                format!("{collapsed:?}")
            );
            text.set_css_pre_wrap(true);
            assert_eq!(
                format!("{:?}", lines(&text, 224.0)),
                format!("{preserved:?}")
            );
            text.set_css_pre_line(false);
            assert_eq!(format!("{:?}", lines(&text, 224.0)), legacy);
        }
    }

    #[test]
    fn css_clone_retains_policies_without_sharing_derived_state() {
        use crate::source::core::CoreObject;
        use super::super::css_decoration::{ResolvedUnderline, ResolvedStrikethrough, SkipInk};
        let mut source = Text::default();
        source.set_css_normal_wrap(true);
        source.set_css_nowrap_alignment(true);
        source.set_experimental_css_ellipsis(true);
        source.set_css_underlines(vec![ResolvedUnderline::solid(0xff123456, 1.0, 2.0, SkipInk::None).unwrap()]);
        source.set_css_strikethroughs(vec![ResolvedStrikethrough::solid(0xff654321, 1.0, 3.0, 12.0).unwrap()]);
        source.layout_width = 123.0;
        source.measured_css_baseline = Some(12.0);
        let mut cloned = source.clone_boxed().unwrap();
        let clone = cloned.as_text_mut().unwrap();
        assert!(clone.css_wrap_policy == source.css_wrap_policy);
        assert_eq!(clone.css_underlines, source.css_underlines);
        assert_eq!(clone.css_strikethroughs, source.css_strikethroughs);
        assert!(clone.css_nowrap_alignment && clone.experimental_css_ellipsis);
        assert!(clone.layout_width.is_nan());
        assert_eq!(clone.measured_css_baseline, None);
        clone.set_css_normal_wrap(false);
        clone.set_css_underlines(Vec::new());
        assert!(source.css_wrap_policy.is_some());
        assert_eq!(source.css_underlines.len(), 1);
        let default = Text::default().clone_core();
        assert!(default.css_wrap_policy.is_none());
        assert!(!default.css_nowrap_alignment && !default.experimental_css_ellipsis);
    }

    #[test]
    fn css_intrinsic_width_preserves_available_box_and_nowrap_overflow() {
        use crate::mechanical_port::source::{
            assets::font_asset::FontAsset, core::CoreArena,
            text::{font_hb::HbFont, text_style::TextStyle},
        };
        let arena = CoreArena::default();
        let asset = arena.insert(FontAsset::default());
        FontAsset::set_font_occurrence(&asset, Some(HbFont::decode(&css_test_font_bytes()).unwrap()));
        let style = arena.insert(TextStyle::default());
        TextStyle::set_asset_occurrence(&style, Some(asset));
        style.with_downcast_mut::<TextStyle, _>(|style| {
            style.base.set_font_size_value(20.0);
            style.base.set_line_height_value(30.0);
        }).unwrap();
        let mut run = TextValueRun::default();
        run.base.set_text_value("A longer paragraph with words that need to wrap in narrow panels".into());
        run.set_style(style);
        let mut text = Text::default();
        text.base.set_sizing_value_value(0);
        text.all_runs.push(TextValueRunHandle::Runtime(Rc::new(RefCell::new(run))));
        let measure = |text: &mut Text, width, mode| text.measure_layout(
            width, mode, f32::NAN, LayoutMeasureMode::Undefined,
        );
        let legacy = measure(&mut text, 224.0, LayoutMeasureMode::AtMost);
        text.set_css_normal_wrap(true);
        let narrow = measure(&mut text, 224.0, LayoutMeasureMode::AtMost);
        assert_eq!(narrow.x, 224.0);
        let wide = measure(&mut text, 1000.0, LayoutMeasureMode::AtMost);
        assert!(wide.x > 224.0 && wide.x < 1000.0);
        assert!(narrow.y > wide.y);
        assert_eq!(measure(&mut text, 224.0, LayoutMeasureMode::AtMost), narrow);
        text.set_css_normal_wrap(false);
        assert_eq!(measure(&mut text, 224.0, LayoutMeasureMode::AtMost), legacy);
        text.base.set_wrap_value_value(1);
        text.set_css_nowrap_alignment(true);
        let overflow = measure(&mut text, 224.0, LayoutMeasureMode::AtMost);
        assert!(overflow.x > 224.0);
        assert_eq!(measure(&mut text, 224.0, LayoutMeasureMode::Exactly).x, 224.0);
        text.base.set_wrap_value_value(0);
        text.set_css_nowrap_alignment(false);
        text.set_css_normal_wrap(true);
        if let TextValueRunHandle::Runtime(run) = &text.all_runs[0] {
            run.borrow_mut().base.set_text_value("SupercalifragilisticexpialidociousSupercalifragilisticexpialidocious".into());
        }
        let unbreakable = measure(&mut text, 224.0, LayoutMeasureMode::AtMost);
        assert!(unbreakable.x > 224.0);
        assert_eq!(measure(&mut text, 1000.0, LayoutMeasureMode::AtMost).x, unbreakable.x);
        assert_eq!(measure(&mut text, 224.0, LayoutMeasureMode::Exactly).x, 224.0);
    }

    #[test]
    fn css_pre_wrap_measurement_survives_resize_and_font_replacement() {
        use crate::mechanical_port::source::{
            assets::font_asset::FontAsset,
            core::CoreArena,
            text::{font_hb::HbFont, text_style::TextStyle},
        };
        let bytes = css_test_font_bytes();
        let arena = CoreArena::default();
        let asset = arena.insert(FontAsset::default());
        FontAsset::set_font_occurrence(&asset, Some(HbFont::decode(&bytes).unwrap()));
        let style = arena.insert(TextStyle::default());
        TextStyle::set_asset_occurrence(&style, Some(asset.clone()));
        style
            .with_downcast_mut::<TextStyle, _>(|style| {
                style.base.set_font_size_value(20.0);
                style.base.set_line_height_value(30.0);
            })
            .unwrap();
        let make_text = || {
            let mut run = TextValueRun::default();
            run.base.set_text_value(
                "SupercalifragilisticexpialidociousSupercalifragilisticexpialidocious".into(),
            );
            run.set_style(style.clone());
            let mut text = Text::default();
            text.base.set_sizing_value_value(1);
            text.base.set_width_value(224.0);
            text.all_runs
                .push(TextValueRunHandle::Runtime(Rc::new(RefCell::new(run))));
            text
        };
        let measure = |text: &mut Text, width| {
            text.measure_layout(
                width,
                LayoutMeasureMode::Exactly,
                f32::NAN,
                LayoutMeasureMode::Undefined,
            )
        };
        for pre_line in [false, true] {
            let mut text = make_text();
            let legacy = measure(&mut text, 224.0);
            if pre_line {
                text.set_css_pre_line(true);
            } else {
                text.set_css_pre_wrap(true);
            }
            let narrow = measure(&mut text, 224.0);
            assert!(legacy.y > narrow.y);
            assert_eq!(narrow.y, measure(&mut text, 752.0).y);
            text.control_size(
                narrow,
                LayoutScaleType::Fill,
                LayoutScaleType::Hug,
                LayoutDirection::Ltr,
            );
            assert_eq!(measure(&mut text, 224.0), narrow);
            FontAsset::set_font_occurrence(&asset, None);
            FontAsset::set_font_occurrence(&asset, Some(HbFont::decode(&bytes).unwrap()));
            assert_eq!(measure(&mut text, 224.0), narrow);
            assert_eq!(
                measure(&mut make_text(), 224.0),
                legacy,
                "another occurrence sharing the font retains legacy wrapping"
            );
            text.set_css_pre_wrap(false);
            assert_eq!(measure(&mut text, 224.0), legacy);
        }
    }

    #[test]
    fn css_pre_wrap_tabs_recompute_advances_for_each_soft_line() {
        use crate::mechanical_port::source::text::font_hb::HbFont;
        let decoded = HbFont::decode(&css_test_font_bytes()).unwrap();
        let font = decoded
            .as_any()
            .downcast_ref::<HbFont>()
            .unwrap()
            .with_experimental_css_tabs(true);
        let chars: Vec<u32> = "one two three four five six seven\tX"
            .chars()
            .map(u32::from)
            .collect();
        let original = font.shape_text(
            &chars,
            &[TextRun {
                font: Some(font.clone()),
                size: 20.0,
                line_height: 30.0,
                letter_spacing: 0.0,
                unichar_count: chars.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        );
        let identities = |shape: &[Paragraph]| {
            shape
                .iter()
                .flat_map(|p| &p.runs)
                .map(|run| (run.glyphs.clone(), run.text_indices.clone()))
                .collect::<Vec<_>>()
        };
        let origins = |text: &Text, shape: &mut [Paragraph], width| {
            let lines =
                text.break_lines_for_layout(shape, &chars, width, TextAlign::Left, TextWrap::Wrap);
            let mut result = Vec::new();
            let mut line_number = 0;
            for (paragraph, lines) in shape.iter().zip(lines) {
                for line in lines {
                    for run_index in line.start_run_index..=line.end_run_index {
                        let run = &paragraph.runs[run_index as usize];
                        let start = if run_index == line.start_run_index {
                            line.start_glyph_index
                        } else {
                            0
                        };
                        let end = if run_index == line.end_run_index {
                            line.end_glyph_index
                        } else {
                            run.glyphs.len() as u32
                        };
                        for index in start..end {
                            if chars[run.text_indices[index as usize] as usize] == u32::from('X') {
                                let origin = paragraph.runs[line.start_run_index as usize].xpos
                                    [line.start_glyph_index as usize];
                                result.push((
                                    line_number,
                                    run.xpos[index as usize] - origin + line.start_x,
                                ));
                            }
                        }
                    }
                    line_number += 1;
                }
            }
            result
        };
        let mut text = Text::default();
        let legacy = origins(&text, &mut original.clone(), 224.0);
        text.set_css_pre_wrap(true);
        let mut shape = original.clone();
        let narrow = origins(&text, &mut shape, 224.0);
        assert_eq!(
            narrow,
            vec![(1, 131.71875)],
            "pinned Chromium X origin after a soft break"
        );
        assert_ne!(
            narrow, legacy,
            "paragraph-relative tabs must expose the original defect"
        );
        let wide = origins(&text, &mut shape, 752.0);
        assert_eq!(wide, origins(&text, &mut original.clone(), 752.0));
        assert_eq!(wide[0].0, 0);
        assert_eq!(
            origins(&text, &mut shape, 224.0),
            narrow,
            "reuse the same shaped runs across sizes"
        );
        assert_eq!(identities(&shape), identities(&original));
        for run in shape.iter().flat_map(|p| &p.runs) {
            for (index, source) in run.text_indices.iter().enumerate() {
                if chars[*source as usize] == 9 {
                    assert!(font.get_path(run.glyphs[index]).empty());
                }
            }
        }
        assert_eq!(
            origins(&Text::default(), &mut original.clone(), 224.0),
            legacy
        );
    }

    #[test]
    fn css_pre_wrap_long_tab_sequence_keeps_one_hanging_line_and_following_text() {
        use crate::mechanical_port::source::text::font_hb::HbFont;
        let decoded = HbFont::decode(&css_test_font_bytes()).unwrap();
        let font = decoded
            .as_any()
            .downcast_ref::<HbFont>()
            .unwrap()
            .with_experimental_css_tabs(true);
        let chars: Vec<u32> = ("\t".repeat(16384) + "X").chars().map(u32::from).collect();
        let mut shape = font.shape_text(
            &chars,
            &[TextRun {
                font: Some(font.clone()),
                size: 20.0,
                line_height: 30.0,
                letter_spacing: 0.0,
                unichar_count: chars.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        );
        let mut text = Text::default();
        text.set_css_pre_wrap(true);
        let lines =
            text.break_lines_for_layout(&mut shape, &chars, 224.0, TextAlign::Left, TextWrap::Wrap);
        assert_eq!(lines.iter().map(Vec::len).sum::<usize>(), 2);
        let last = lines.last().unwrap().last().unwrap();
        let run = &shape.last().unwrap().runs[last.start_run_index as usize];
        assert_eq!(
            chars[run.text_indices[last.start_glyph_index as usize] as usize],
            u32::from('X')
        );
        assert_eq!(last.start_x, 0.0);
    }

    fn runtime_run(value: &str) -> TextValueRunHandle {
        let mut run = TextValueRun::default();
        run.base.set_text_value(value.to_owned());
        TextValueRunHandle::Runtime(Rc::new(RefCell::new(run)))
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
        let style = arena.insert(TextStyle::default());
        TextStyle::set_asset_occurrence(&style, Some(asset));
        style
            .with_downcast_mut::<TextStyle, _>(|style| {
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
        let stretched = measure(&mut text, 354.0, LayoutMeasureMode::Exactly);
        text.base.set_width_value(354.0);
        let authored = measure(&mut text, 354.0, LayoutMeasureMode::AtMost);
        assert!(narrow.y > authored.y, "fixture must wrap more at 120px");
        assert_eq!(
            stretched, authored,
            "exact layout and authored widths must agree"
        );
        text.base.set_width_value(120.0);
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
