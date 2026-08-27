use super::{
    glyph_lookup::GlyphLookup, text_interface::TextInterface,
    text_modifier_group::TextModifierGroup, text_style_paint::TextStylePaint,
    text_value_run::TextValueRun, utf::Utf,
};
use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    generated::text::text_base::TextBase,
    layout::{LayoutDirection, LayoutMeasureMode, LayoutScaleType},
    math::{aabb::Aabb, mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
    refcnt::RiveRc,
    renderer::{RenderImage, Renderer},
    text_engine::{
        Font, GlyphLine, GlyphRun, OrderedLine, Paragraph, TextAlign, TextOrigin, TextOverflow,
        TextRun, TextSizing, TextTrimBottom, TextTrimTop, TextWrap, VerticalTextAlign,
    },
    viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem,
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
}
impl Text {
    pub fn mark_shape_dirty(&mut self) {
        self.mark_shape_dirty_layout(true);
    }
    pub fn mark_shape_dirty_layout(&mut self, send_to_layout: bool) {
        self.base
            .add_dirt(ComponentDirt::TEXT_SHAPE | ComponentDirt::PAINT);
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
    pub fn add_run(&mut self, run: &mut TextValueRun) {
        self.runs.push(NonNull::from(run));
        self.all_runs.push(NonNull::from(run));
    }
    pub fn add_modifier_group(&mut self, group: &mut TextModifierGroup) {
        self.modifier_groups.push(NonNull::from(group));
    }
    pub fn sizing(&self) -> TextSizing {
        unsafe { std::mem::transmute(self.base.sizing_value() as u8) }
    }
    pub fn effective_sizing(&self) -> TextSizing {
        if self.layout_width.is_nan() {
            self.sizing()
        } else if self.layout_height.is_nan() {
            TextSizing::AutoHeight
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
    pub fn wrap(&self) -> TextWrap {
        unsafe { std::mem::transmute(self.base.wrap_value() as u8) }
    }
    pub fn align(&self) -> TextAlign {
        let align = unsafe { std::mem::transmute(self.base.align_value() as u8) };
        if self.layout_direction == LayoutDirection::Rtl {
            match align {
                TextAlign::Left => TextAlign::Right,
                TextAlign::Right => TextAlign::Left,
                _ => align,
            }
        } else {
            align
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
        self.text_style_paints.get(id as usize).copied()
    }
    pub fn runs(&self) -> &[NonNull<TextValueRun>] {
        &self.all_runs
    }
    pub fn ordered_lines(&self) -> &[OrderedLine] {
        &self.ordered_lines
    }
    pub fn make_styled(
        &self,
        styled: &mut StyledText,
        with_modifiers: bool,
        font_scale: f32,
    ) -> bool {
        styled.clear();
        for run in &self.runs {
            let run = unsafe { run.as_ref() };
            let Some(style) = run.style() else {
                continue;
            };
            let style = unsafe { style.as_ref() };
            let Some(font) = style.base.font() else {
                return false;
            };
            styled.append(
                font,
                style.base.font_size() * font_scale,
                style.base.line_height(),
                style.base.letter_spacing(),
                run.base.text(),
                self.text_style_paints
                    .iter()
                    .position(|p| p == &style.into())
                    .unwrap_or(0) as u16,
            );
        }
        if with_modifiers {
            for group in &self.modifier_groups {
                unsafe { group.as_ref() }.apply_shape_modifiers_readonly(self, styled);
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
        paragraphs
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut lines = p.break_lines(width, align, wrap);
                GlyphLine::compute_line_spacing(i == 0, &mut lines, &p.runs, width, align);
                lines
            })
            .collect()
    }
    pub fn modifier_ranges_need_shape(&self) -> bool {
        self.modifier_groups
            .iter()
            .any(|g| unsafe { g.as_ref() }.needs_shape())
    }
    pub fn update(&mut self, value: ComponentDirt) {
        if value.intersects(ComponentDirt::TEXT_SHAPE | ComponentDirt::PATH) {
            self.build_text_style_paints();
            if !self.make_styled(&mut self.styled_text, false, 1.0) {
                self.shape.clear();
                self.lines.clear();
                self.ordered_lines.clear();
                self.bounds = Aabb::default();
                return;
            }
            self.shape = self.styled_text.runs()[0]
                .font
                .shape_text(self.styled_text.unichars(), self.styled_text.runs());
            self.lines = Self::break_lines(
                &self.shape,
                if self.effective_sizing() == TextSizing::AutoWidth {
                    -1.0
                } else {
                    self.effective_width()
                },
                self.align(),
                self.wrap(),
            );
            self.glyph_lookup
                .compute(self.styled_text.unichars(), &self.shape);
            for group in &mut self.modifier_groups {
                let group = unsafe { group.as_mut() };
                group.compute_range_map(
                    self.styled_text.unichars(),
                    &self.shape,
                    &self.lines,
                    &self.glyph_lookup,
                );
                group.compute_coverage(self.styled_text.unichars().len() as u32);
                group.reset_text_follow_path();
            }
            self.rebuild_ordered_lines();
        }
        if value.intersects(ComponentDirt::PAINT | ComponentDirt::TEXT_SHAPE | ComponentDirt::PATH)
        {
            self.build_render_styles();
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
    pub fn build_render_styles(&mut self) {
        for style in &mut self.render_styles {
            unsafe { style.as_mut() }.rewind_path();
        }
        self.render_styles.clear();
        for line in &self.ordered_lines {
            let mut x = line.glyph_line().start_x;
            for glyph in line.iter() {
                let run = glyph.run;
                let i = glyph.glyph_index as usize;
                let style = self.text_style_paints[run.style_id as usize];
                let transform = Mat2D::new(
                    run.size,
                    0.0,
                    0.0,
                    run.size,
                    x + run.offsets[i].x,
                    line.y() + run.offsets[i].y,
                );
                x += run.advances[i];
                if !run.font.is_color_glyph(run.glyphs[i]) {
                    let path = run.font.get_path(run.glyphs[i]);
                    let first = unsafe { style.as_ptr().as_mut().unwrap() }
                        .add_path(&path.transformed(&transform), 1.0);
                    if first {
                        self.render_styles.push(style);
                    }
                }
            }
        }
    }
    pub fn draw(&mut self, renderer: &mut Renderer) {
        for style in &mut self.render_styles {
            unsafe { style.as_mut() }.draw(renderer, &self.shape_world_transform);
        }
    }
    pub fn local_bounds(&self) -> Aabb {
        self.bounds
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
                unsafe { group.as_mut() }.on_text_world_transform_dirty();
            }
        }
    }
    pub fn compose_world_transform(&mut self) {
        self.base.compose_world_transform();
        self.shape_world_transform = *self.base.world_transform() * self.internal_transform;
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
                f32::INFINITY
            } else {
                width
            },
            if height_mode == LayoutMeasureMode::Undefined {
                f32::INFINITY
            } else {
                height
            },
        );
        let measured = self.measure(max);
        Vec2D::new(
            if width_mode == LayoutMeasureMode::Exactly {
                width
            } else {
                measured.x
            },
            if height_mode == LayoutMeasureMode::Exactly {
                height
            } else {
                measured.y
            },
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
    fn measure(&mut self, max: Vec2D) -> Vec2D {
        let old = (self.layout_width, self.layout_height);
        self.layout_width = max.x;
        self.layout_height = max.y;
        self.update(ComponentDirt::TEXT_SHAPE);
        let result = Vec2D::new(self.bounds.width(), self.bounds.height());
        (self.layout_width, self.layout_height) = old;
        result
    }
    pub fn align_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn sizing_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn overflow_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn width_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn height_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn paragraph_spacing_changed(&mut self) {
        self.mark_paint_dirty();
    }
    pub fn origin_value_changed(&mut self) {
        self.mark_paint_dirty();
    }
    pub fn origin_x_changed(&mut self) {
        self.mark_paint_dirty();
    }
    pub fn origin_y_changed(&mut self) {
        self.mark_paint_dirty();
    }
    pub fn vertical_trim_value_changed(&mut self) {
        self.mark_shape_dirty();
    }
    pub fn update_list(&mut self, list: Option<&[RiveRc<ViewModelInstanceListItem>]>) {
        let Some(list) = list else {
            return;
        };
        self.runs.clear();
        for item in list {
            if let Some(mut instance) = item.view_model_instance() {
                if let Some(mut value) = instance.property_value_for_symbol(
                    crate::mechanical_port::source::viewmodel::symbol_type::SymbolType::TextContent,
                ) {
                    if let Some(run) = unsafe { value.as_mut() }.base.as_text_value_run() {
                        self.runs.push(run);
                    }
                }
            }
        }
        self.mark_shape_dirty();
    }
    pub fn build_text_style_paints(&mut self) {
        self.text_style_paints.clear();
        for run in &self.all_runs {
            if let Some(style) = unsafe { run.as_ref() }.style() {
                if !self.text_style_paints.contains(&style) {
                    self.text_style_paints.push(style);
                }
            }
        }
    }
}
