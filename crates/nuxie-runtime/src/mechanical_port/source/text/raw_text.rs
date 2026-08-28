use super::text::Text;
use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    color::ColorInt,
    factory::RuntimeFactoryHandle,
    math::raw_path::PathDirection,
    math::{aabb::Aabb, mat2d::Mat2D},
    shapes::shape_paint_path::ShapePaintPath,
    text_engine::{
        FontRef, GlyphLine, GlyphRun, OrderedLine, Paragraph, StyledText, TextAlign, TextOrigin,
        TextOverflow, TextSizing, TextWrap,
    },
};
use nuxie_render_api::{FillRule, RenderPaint, RenderPaintStyle, Renderer};

type RuntimeRenderPaintHandle = Rc<RefCell<Box<dyn RenderPaint>>>;

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
    paragraph_spacing: f32,
    origin: TextOrigin,
    sizing: TextSizing,
    overflow: TextOverflow,
    align: TextAlign,
    wrap: TextWrap,
    max_width: f32,
    max_height: f32,
    ordered_lines: Vec<OrderedLine>,
    ellipsis_run: GlyphRun,
    bounds: Aabb,
    clip_render_path: Option<ShapePaintPath>,
    draw_commands: Vec<DrawCommand>,
}
impl RawText {
    pub fn new(factory: RuntimeFactoryHandle) -> Self {
        Self {
            shape: Vec::new(),
            lines: Vec::new(),
            styled: StyledText::default(),
            factory,
            styles: Vec::new(),
            render_styles: Vec::new(),
            dirty: false,
            paragraph_spacing: 0.0,
            origin: TextOrigin::Top,
            sizing: TextSizing::AutoWidth,
            overflow: TextOverflow::Visible,
            align: TextAlign::Left,
            wrap: TextWrap::Wrap,
            max_width: 0.0,
            max_height: 0.0,
            ordered_lines: Vec::new(),
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
        text: &str,
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
            .position(|s| match (&s.paint, &paint) {
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
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
        self.dirty = true;
    }
    pub fn clear(&mut self) {
        self.styled.clear();
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
    pub fn set_sizing(&mut self, v: TextSizing) {
        if self.sizing != v {
            self.sizing = v;
            self.dirty = true;
        }
    }
    pub fn set_overflow(&mut self, v: TextOverflow) {
        if self.overflow != v {
            self.overflow = v;
            self.dirty = true;
        }
    }
    pub fn set_align(&mut self, v: TextAlign) {
        if self.align != v {
            self.align = v;
            self.dirty = true;
        }
    }
    pub fn set_max_width(&mut self, v: f32) {
        if self.max_width != v {
            self.max_width = v;
            self.dirty = true;
        }
    }
    pub fn set_max_height(&mut self, v: f32) {
        if self.max_height != v {
            self.max_height = v;
            self.dirty = true;
        }
    }
    pub fn set_paragraph_spacing(&mut self, v: f32) {
        if self.paragraph_spacing != v {
            self.paragraph_spacing = v;
            self.dirty = true;
        }
    }
    fn update(&mut self) {
        for style in &mut self.styles {
            style.path.rewind();
            style.is_empty = true;
        }
        self.render_styles.clear();
        self.draw_commands.clear();
        if self.styled.empty() {
            return;
        }
        let runs = self.styled.runs();
        self.shape = runs[0].font.shape_text(self.styled.unichars(), runs);
        self.lines = Text::break_lines(
            &self.shape,
            if self.sizing == TextSizing::AutoWidth {
                -1.0
            } else {
                self.max_width
            },
            self.align,
            self.wrap,
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
                let mut x = line.start_x;
                for glyph in self.ordered_lines.last().unwrap().iter() {
                    let run = glyph.run;
                    let i = glyph.glyph_index as usize;
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
                    if run.font.is_color_glyph(run.glyphs[i]) {
                        self.draw_commands.push(DrawCommand::Color {
                            font: run.font.clone(),
                            glyph_id: run.glyphs[i],
                            transform,
                            foreground_color: self.styles[style].foreground_color,
                        });
                    } else {
                        let path = run.font.get_path(run.glyphs[i]);
                        self.styles[style]
                            .path
                            .add_path_clockwise_with_transform(&path, &transform);
                        if self.styles[style].is_empty {
                            self.styles[style].is_empty = false;
                            self.render_styles.push(style);
                            self.draw_commands.push(DrawCommand::Style(style));
                        }
                    }
                }
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
                    let mut layers = Vec::new();
                    if font.get_color_layers(*glyph_id, &mut layers, *foreground_color) > 0 {
                        renderer.save();
                        renderer.transform(nuxie_render_api::Mat2D(*transform.values()));
                        for layer in layers {
                            let mut path = ShapePaintPath::with_fill_rule(true, FillRule::NonZero);
                            path.add_path(&layer.path, None);
                            let mut paint = self
                                .factory
                                .with_factory_mut(|factory| factory.make_render_paint());
                            paint.set_style(RenderPaintStyle::Fill);
                            paint.set_color(layer.color);
                            renderer.draw_path(path.render_path(&self.factory), paint.as_ref());
                        }
                        renderer.restore();
                    }
                }
            }
        }
        if self.overflow == TextOverflow::Clipped && self.clip_render_path.is_some() {
            renderer.restore();
        }
    }
}
