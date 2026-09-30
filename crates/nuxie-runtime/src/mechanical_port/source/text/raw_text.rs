use super::text::{StyledText, Text};
use super::{glyph_lookup::GlyphLookup, text_layout_view::TextLayoutView};
use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    factory::RuntimeFactoryHandle,
    math::path_types::PathDirection,
    math::{aabb::Aabb, mat2d::Mat2D},
    math::{raw_path::RawPath, vec2d::Vec2D},
    shapes::paint::color::ColorInt,
    shapes::paint::shape_paint_path::ShapePaintPath,
    text_engine::{
        Font, FontRef, GlyphLine, GlyphRun, OrderedLine, Paragraph, TextAlign, TextDirection,
        TextOrigin, TextOverflow, TextSizing, TextWordBreak, TextWrap,
    },
};
use nuxie_render_api::{FillRule, RenderPaint, RenderPaintStyle, Renderer};

type RuntimeRenderPaintHandle = Rc<RefCell<Box<dyn RenderPaint>>>;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalAlign {
    None,
    Start,
    End,
}

struct RenderStyle {
    paint: Option<RuntimeRenderPaintHandle>,
    is_empty: bool,
    path: ShapePaintPath,
    foreground_color: ColorInt,
}
enum DrawCommand {
    Style(usize),
    Color {
        font: FontRef,
        glyph_id: u16,
        transform: Mat2D,
        foreground_color: ColorInt,
    },
}
pub struct RawText {
    shape: Vec<Paragraph>,
    lines: Vec<Vec<GlyphLine>>,
    styled: StyledText,
    factory: RuntimeFactoryHandle,
    styles: Vec<RenderStyle>,
    render_styles: Vec<usize>,
    dirty: bool,
    lookup_dirty: bool,
    revision: u32,
    paragraph_spacing: f32,
    origin: TextOrigin,
    sizing: TextSizing,
    overflow: TextOverflow,
    align: TextAlign,
    logical_align: LogicalAlign,
    wrap: TextWrap,
    word_break: TextWordBreak,
    direction_flag: i32,
    glyph_lookup: GlyphLookup,
    max_width: f32,
    max_height: f32,
    ordered_lines: Vec<OrderedLine>,
    line_directions: Vec<TextDirection>,
    line_glyph_counts: Vec<u32>,
    ellipsis_run: GlyphRun,
    bounds: Aabb,
    clip_render_path: Option<ShapePaintPath>,
    draw_commands: Vec<DrawCommand>,
}
impl RawText {
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
        self.styles.get(index).map(|style| style.foreground_color)
    }
    #[doc(hidden)]
    pub fn debug_command_kinds(&self) -> Vec<&'static str> {
        self.draw_commands
            .iter()
            .map(|command| match command {
                DrawCommand::Style(_) => "style",
                DrawCommand::Color { .. } => "color",
            })
            .collect()
    }
    #[doc(hidden)]
    pub fn debug_has_clip(&self) -> bool {
        self.clip_render_path.is_some()
    }
    #[doc(hidden)]
    pub fn debug_style_path_bounds(&self) -> Vec<Aabb> {
        self.styles
            .iter()
            .filter(|style| !style.is_empty)
            .map(|style| style.path.raw_path().bounds())
            .collect()
    }
    pub fn new(factory: RuntimeFactoryHandle) -> Self {
        Self {
            shape: Vec::new(),
            lines: Vec::new(),
            styled: StyledText::default(),
            factory,
            styles: Vec::new(),
            render_styles: Vec::new(),
            dirty: false,
            lookup_dirty: false,
            revision: 0,
            paragraph_spacing: 0.0,
            origin: TextOrigin::Top,
            sizing: TextSizing::AutoWidth,
            overflow: TextOverflow::Visible,
            align: TextAlign::Left,
            logical_align: LogicalAlign::None,
            wrap: TextWrap::Wrap,
            word_break: TextWordBreak::BreakWord,
            direction_flag: -1,
            glyph_lookup: GlyphLookup::default(),
            max_width: 0.0,
            max_height: 0.0,
            ordered_lines: Vec::new(),
            line_directions: Vec::new(),
            line_glyph_counts: Vec::new(),
            ellipsis_run: GlyphRun::default(),
            bounds: Aabb::default(),
            clip_render_path: None,
            draw_commands: Vec::new(),
        }
    }
    pub fn empty(&self) -> bool {
        self.styled.empty()
    }
    pub fn append(
        &mut self,
        text: impl AsRef<[u8]>,
        paint: Option<RuntimeRenderPaintHandle>,
        font: FontRef,
        size: f32,
        line_height: f32,
        letter_spacing: f32,
        foreground: ColorInt,
    ) {
        let index = self
            .styles
            .iter()
            .position(|s| {
                s.foreground_color == foreground
                    && match (&s.paint, &paint) {
                        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                        (None, None) => true,
                        _ => false,
                    }
            })
            .unwrap_or_else(|| {
                self.styles.push(RenderStyle {
                    paint: paint.clone(),
                    is_empty: true,
                    path: ShapePaintPath::default(),
                    foreground_color: foreground,
                });
                self.styles.len() - 1
            });
        self.styled
            .append(font, size, line_height, letter_spacing, text, index as u16);
        self.mark_dirty();
    }
    pub fn clear(&mut self) {
        self.styled.clear();
        self.styles.clear();
        self.mark_dirty();
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.lookup_dirty = true;
        self.revision = self.revision.wrapping_add(1);
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
    pub fn logical_align(&self) -> LogicalAlign {
        self.logical_align
    }
    pub fn wrap(&self) -> TextWrap {
        self.wrap
    }
    pub fn word_break(&self) -> TextWordBreak {
        self.word_break
    }
    pub fn origin(&self) -> TextOrigin {
        self.origin
    }
    pub fn direction_flag(&self) -> i32 {
        self.direction_flag
    }
    pub fn length(&self) -> usize {
        self.styled.unichars().len()
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn style_count(&self) -> usize {
        self.styles.len()
    }
    pub fn style_paint(&self, style_id: u16) -> Option<RuntimeRenderPaintHandle> {
        self.styles
            .get(style_id as usize)
            .and_then(|style| style.paint.clone())
    }
    pub fn style_foreground_color(&self, style_id: u16) -> ColorInt {
        self.styles
            .get(style_id as usize)
            .map_or(0xff000000, |style| style.foreground_color)
    }
    pub fn set_logical_align(&mut self, value: LogicalAlign) {
        if self.logical_align != value {
            self.logical_align = value;
            self.mark_dirty();
        }
    }
    pub fn set_wrap(&mut self, value: TextWrap) {
        if self.wrap != value {
            self.wrap = value;
            self.mark_dirty();
        }
    }
    pub fn set_word_break(&mut self, value: TextWordBreak) {
        if self.word_break != value {
            self.word_break = value;
            self.mark_dirty();
        }
    }
    pub fn set_origin(&mut self, value: TextOrigin) {
        if self.origin != value {
            self.origin = value;
            self.mark_dirty();
        }
    }
    pub fn set_direction_flag(&mut self, value: i32) {
        if self.direction_flag != value {
            self.direction_flag = value;
            self.mark_dirty();
        }
    }
    pub fn align_index(&self) -> i32 {
        match self.logical_align {
            LogicalAlign::None => match self.align {
                TextAlign::Left => 0,
                TextAlign::Right => 1,
                TextAlign::Center => 2,
                TextAlign::Unknown(value) => value as i32,
            },
            LogicalAlign::Start => 3,
            LogicalAlign::End => 4,
        }
    }
    pub fn set_align_index(&mut self, value: i32) {
        if (0..=2).contains(&value) {
            self.set_align(match value {
                0 => TextAlign::Left,
                1 => TextAlign::Right,
                _ => TextAlign::Center,
            });
            self.set_logical_align(LogicalAlign::None);
        } else if value == 3 || value == 4 {
            self.set_logical_align(if value == 3 {
                LogicalAlign::Start
            } else {
                LogicalAlign::End
            });
        }
    }
    pub fn direction_index(&self) -> i32 {
        self.direction_flag.wrapping_add(1)
    }
    pub fn set_direction_index(&mut self, value: i32) {
        if (0..=2).contains(&value) {
            self.set_direction_flag(value - 1);
        }
    }
    pub fn set_sizing_index(&mut self, value: i32) {
        match value {
            0 => self.set_sizing(TextSizing::AutoWidth),
            1 => self.set_sizing(TextSizing::AutoHeight),
            2 => self.set_sizing(TextSizing::Fixed),
            _ => {}
        }
    }
    pub fn set_overflow_index(&mut self, value: i32) {
        match value {
            0 => self.set_overflow(TextOverflow::Visible),
            1 => self.set_overflow(TextOverflow::Hidden),
            2 => self.set_overflow(TextOverflow::Clipped),
            3 => self.set_overflow(TextOverflow::Ellipsis),
            _ => {}
        }
    }
    pub fn set_wrap_index(&mut self, value: i32) {
        match value {
            0 => self.set_wrap(TextWrap::Wrap),
            1 => self.set_wrap(TextWrap::NoWrap),
            _ => {}
        }
    }
    pub fn set_word_break_index(&mut self, value: i32) {
        match value {
            0 => self.set_word_break(TextWordBreak::BreakWord),
            1 => self.set_word_break(TextWordBreak::Normal),
            2 => self.set_word_break(TextWordBreak::BreakAll),
            _ => {}
        }
    }
    pub fn set_origin_index(&mut self, value: i32) {
        match value {
            0 => self.set_origin(TextOrigin::Top),
            1 => self.set_origin(TextOrigin::Baseline),
            _ => {}
        }
    }
    pub fn ordered_lines(&mut self) -> &[OrderedLine] {
        if self.dirty {
            self.update();
            self.dirty = false;
        }
        &self.ordered_lines
    }
    pub fn line_count(&mut self) -> u32 {
        self.ordered_lines().len() as u32
    }
    pub fn line_top(&mut self, line: u32) -> f32 {
        let ordered = &self.ordered_lines()[line as usize];
        ordered.y() - ordered.glyph_line().baseline + ordered.glyph_line().top
    }
    pub fn line_direction(&mut self, line: u32) -> TextDirection {
        self.ordered_lines();
        self.line_directions[line as usize]
    }
    pub fn line_glyph_count(&mut self, line: u32) -> u32 {
        self.ordered_lines();
        self.line_glyph_counts[line as usize]
    }
    pub fn for_each_glyph(
        &mut self,
        line: u32,
        mut visit: impl FnMut(&GlyphRun, u32, Vec2D) -> bool,
    ) {
        let ordered = &self.ordered_lines()[line as usize];
        let mut x = ordered.glyph_line().start_x;
        let y = ordered.y();
        for (run, glyph_index) in ordered {
            let offset = run.offsets[glyph_index as usize];
            if !visit(run, glyph_index, Vec2D::new(x + offset.x, y + offset.y)) {
                return;
            }
            x += run.advances[glyph_index as usize];
        }
    }
    pub fn layout_view(&mut self) -> TextLayoutView<'_> {
        self.ordered_lines();
        if self.lookup_dirty {
            self.glyph_lookup
                .compute(self.styled.unichars(), &self.shape);
            self.lookup_dirty = false;
        }
        TextLayoutView::new(
            &self.shape,
            &self.lines,
            &self.ordered_lines,
            &self.glyph_lookup,
            self.styled.unichars().len() as u32,
        )
    }
    pub fn glyph_transform(run: &GlyphRun, position: Vec2D) -> Mat2D {
        Mat2D::new(run.size, 0.0, 0.0, run.size, position.x, position.y)
    }
    pub fn glyph_path(font: &dyn Font, glyph_id: u16) -> RawPath {
        let mut path = ShapePaintPath::default();
        path.add_path_clockwise(&font.get_path(glyph_id), None);
        path.raw_path().clone()
    }
    pub fn set_sizing(&mut self, v: TextSizing) {
        if self.sizing != v {
            self.sizing = v;
            self.mark_dirty();
        }
    }
    pub fn set_overflow(&mut self, v: TextOverflow) {
        if self.overflow != v {
            self.overflow = v;
            self.mark_dirty();
        }
    }
    pub fn set_align(&mut self, v: TextAlign) {
        if self.align != v {
            self.align = v;
            self.mark_dirty();
        }
    }
    pub fn set_max_width(&mut self, v: f32) {
        if self.max_width != v {
            self.max_width = v;
            self.mark_dirty();
        }
    }
    pub fn set_max_height(&mut self, v: f32) {
        if self.max_height != v {
            self.max_height = v;
            self.mark_dirty();
        }
    }
    pub fn set_paragraph_spacing(&mut self, v: f32) {
        if self.paragraph_spacing != v {
            self.paragraph_spacing = v;
            self.mark_dirty();
        }
    }
    fn update(&mut self) {
        for style in &mut self.styles {
            style.path.rewind();
            style.is_empty = true;
        }
        self.render_styles.clear();
        self.draw_commands.clear();
        self.ordered_lines.clear();
        self.line_directions.clear();
        self.line_glyph_counts.clear();
        if self.styled.empty() {
            self.shape.clear();
            self.lines.clear();
            self.bounds = Aabb::new(0.0, 0.0, 0.0, 0.0);
            return;
        }
        let runs = self.styled.runs();
        self.shape = runs[0]
            .font
            .as_ref()
            .expect("shaped text retains its font")
            .shape_text(self.styled.unichars(), runs, self.direction_flag);
        let mut align = self.align;
        if self.logical_align != LogicalAlign::None && !self.shape.is_empty() {
            let rtl = self.shape[0].base_direction() == TextDirection::Rtl;
            align = if (self.logical_align == LogicalAlign::Start) != rtl {
                TextAlign::Left
            } else {
                TextAlign::Right
            };
        }
        self.lines = Text::break_lines(
            &self.shape,
            if self.sizing == TextSizing::AutoWidth {
                -1.0
            } else {
                self.max_width
            },
            align,
            self.wrap,
            self.word_break,
        );
        self.ellipsis_run = GlyphRun::default();
        if self.shape.is_empty() {
            self.bounds = Aabb::new(0.0, 0.0, 0.0, 0.0);
            return;
        }
        let (mut y, mut min_y, mut width) = (0.0f32, 0.0f32, 0.0f32);
        if self.origin == TextOrigin::Baseline && !self.lines[0].is_empty() {
            y -= self.lines[0][0].baseline;
            min_y = y;
        }
        let want = self.overflow == TextOverflow::Ellipsis && self.sizing == TextSizing::Fixed;
        let (mut ellipse, mut last) = (-1, -1);
        for (p, lines) in self.shape.iter().zip(&self.lines) {
            for line in lines {
                width = width.max(
                    p.runs[line.end_run_index as usize].xpos[line.end_glyph_index as usize]
                        - p.runs[line.start_run_index as usize].xpos
                            [line.start_glyph_index as usize],
                );
                last += 1;
                if want && y + line.bottom <= self.max_height {
                    ellipse += 1;
                }
            }
            if let Some(line) = lines.last() {
                y += line.bottom;
            }
            y += self.paragraph_spacing;
        }
        if want && ellipse == -1 {
            ellipse = 0;
        }
        self.bounds = match self.sizing {
            TextSizing::AutoWidth => {
                Aabb::new(0.0, min_y, width, min_y.max(y - self.paragraph_spacing))
            }
            TextSizing::AutoHeight => Aabb::new(
                0.0,
                min_y,
                self.max_width,
                min_y.max(y - self.paragraph_spacing),
            ),
            TextSizing::Fixed => Aabb::new(0.0, min_y, self.max_width, min_y + self.max_height),
            TextSizing::Unknown(_) => self.bounds,
        };
        if self.overflow == TextOverflow::Clipped {
            let path = self
                .clip_render_path
                .get_or_insert_with(|| ShapePaintPath::with_fill_rule(true, FillRule::NonZero));
            path.rewind();
            path.add_rect(self.bounds, PathDirection::Clockwise);
        } else {
            self.clip_render_path = None;
        }
        y = if self.origin == TextOrigin::Baseline && !self.lines[0].is_empty() {
            -self.lines[0][0].baseline
        } else {
            0.0
        };
        let mut line_index = 0;
        for (p, lines) in self.shape.iter().zip(&self.lines) {
            for line in lines {
                if self.sizing == TextSizing::Fixed
                    && ((self.overflow == TextOverflow::Hidden
                        && y + line.bottom > self.max_height)
                        || (self.overflow == TextOverflow::Clipped
                            && y + line.top > self.max_height))
                {
                    return;
                }
                let render_y = y + line.baseline;
                self.ordered_lines.push(OrderedLine::new(
                    p,
                    line,
                    self.max_width,
                    ellipse == line_index,
                    last == ellipse,
                    &mut self.ellipsis_run,
                    render_y,
                ));
                self.line_directions.push(p.base_direction());
                let mut line_glyphs = 0u32;
                let mut x = line.start_x;
                for (run, glyph_index) in self.ordered_lines.last().unwrap() {
                    line_glyphs += 1;
                    let i = glyph_index as usize;
                    let transform = Mat2D::new(
                        run.size,
                        0.0,
                        0.0,
                        run.size,
                        x + run.offsets[i].x,
                        render_y + run.offsets[i].y,
                    );
                    x += run.advances[i];
                    let style = run.style_id as usize;
                    if run
                        .font
                        .as_ref()
                        .expect("shaped text retains its font")
                        .is_color_glyph(run.glyphs[i])
                    {
                        self.draw_commands.push(DrawCommand::Color {
                            font: run
                                .font
                                .as_ref()
                                .expect("shaped text retains its font")
                                .clone(),
                            glyph_id: run.glyphs[i],
                            transform,
                            foreground_color: self.styles[style].foreground_color,
                        });
                    } else {
                        let path = run
                            .font
                            .as_ref()
                            .expect("shaped text retains its font")
                            .get_path(run.glyphs[i]);
                        self.styles[style]
                            .path
                            .add_path_clockwise(&path, Some(&transform));
                        if self.styles[style].is_empty {
                            self.styles[style].is_empty = false;
                            self.render_styles.push(style);
                            self.draw_commands.push(DrawCommand::Style(style));
                        }
                    }
                }
                self.line_glyph_counts.push(line_glyphs);
                if line_index == ellipse {
                    return;
                }
                line_index += 1;
            }
            if let Some(line) = lines.last() {
                y += line.bottom;
            }
            y += self.paragraph_spacing;
        }
    }
    pub fn bounds(&mut self) -> Aabb {
        if self.dirty {
            self.update();
            self.dirty = false;
        }
        self.bounds
    }
    pub fn render(
        &mut self,
        renderer: &mut dyn Renderer,
        override_paint: Option<RuntimeRenderPaintHandle>,
    ) {
        if self.dirty {
            self.update();
            self.dirty = false;
        }
        if self.overflow == TextOverflow::Clipped && self.clip_render_path.is_some() {
            renderer.save();
            let clip = self
                .clip_render_path
                .as_mut()
                .expect("the clipped branch retains a clip path")
                .render_path(&self.factory);
            renderer.clip_path(clip);
        }
        for command in &self.draw_commands {
            match command {
                DrawCommand::Style(index) => {
                    if let Some(paint) = override_paint
                        .as_ref()
                        .or(self.styles[*index].paint.as_ref())
                        .cloned()
                    {
                        let paint = paint.borrow();
                        renderer.draw_path(
                            self.styles[*index].path.render_path(&self.factory),
                            paint.as_ref(),
                        );
                    }
                }
                DrawCommand::Color {
                    font,
                    glyph_id,
                    transform,
                    foreground_color,
                } => {
                    self.draw_color_glyph(
                        renderer,
                        font.as_ref(),
                        *glyph_id,
                        transform,
                        *foreground_color,
                    );
                }
            }
        }
        if self.overflow == TextOverflow::Clipped && self.clip_render_path.is_some() {
            renderer.restore();
        }
    }
    fn draw_color_glyph(
        &self,
        renderer: &mut dyn Renderer,
        font: &dyn Font,
        glyph_id: u16,
        transform: &Mat2D,
        foreground_color: ColorInt,
    ) {
        let mut layers = Vec::new();
        if font.get_color_layers(glyph_id, &mut layers, foreground_color) == 0 {
            return;
        }
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(*transform.values()));
        for layer in layers {
            let mut path = ShapePaintPath::with_fill_rule(true, FillRule::NonZero);
            path.add_path(&layer.path, None);
            let mut paint = self
                .factory
                .with_factory_mut(|factory| factory.make_render_paint());
            paint.style(RenderPaintStyle::Fill);
            paint.color(layer.color);
            renderer.draw_path(path.render_path(&self.factory), paint.as_ref());
        }
        renderer.restore();
    }
}
