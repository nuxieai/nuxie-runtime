use super::{glyph_lookup::GlyphLookup, text::Text};
use crate::mechanical_port::source::{
    math::aabb::Aabb,
    text_engine::{
        GlyphLine, GlyphRun, OrderedLine, Paragraph, TextAlign, TextOrigin, TextOverflow, TextRun,
        TextSizing, TextWrap, VerticalTextAlign,
    },
};
#[derive(Default)]
pub struct FullyShapedText {
    paragraphs: Vec<Paragraph>,
    paragraph_lines: Vec<Vec<GlyphLine>>,
    ordered_lines: Vec<OrderedLine>,
    glyph_lookup: GlyphLookup,
    ellipsis_run: GlyphRun,
    bounds: Aabb,
    vertical_offset: f32,
}
impl FullyShapedText {
    pub fn vertical_offset(&self) -> f32 {
        self.vertical_offset
    }
    pub fn paragraphs(&self) -> &[Paragraph] {
        &self.paragraphs
    }
    pub fn paragraph_lines(&self) -> &[Vec<GlyphLine>] {
        &self.paragraph_lines
    }
    pub fn ordered_lines(&self) -> &[OrderedLine] {
        &self.ordered_lines
    }
    pub fn glyph_lookup(&self) -> &GlyphLookup {
        &self.glyph_lookup
    }
    pub fn bounds(&self) -> Aabb {
        self.bounds
    }
    pub fn has_valid_bounds(&self) -> bool {
        !self.bounds.is_empty_or_nan()
    }
    pub fn line_count(&self) -> u32 {
        self.ordered_lines.len() as u32
    }
    #[allow(clippy::too_many_arguments)]
    pub fn shape(
        &mut self,
        text: &mut [u32],
        runs: &mut [TextRun],
        sizing: TextSizing,
        max_width: f32,
        max_height: f32,
        alignment: TextAlign,
        wrap: TextWrap,
        origin: TextOrigin,
        overflow: TextOverflow,
        paragraph_spacing: f32,
    ) {
        self.shape_aligned(
            text,
            runs,
            sizing,
            max_width,
            max_height,
            alignment,
            wrap,
            origin,
            overflow,
            paragraph_spacing,
            0.0,
            VerticalTextAlign::Top,
            0.0,
        );
    }
    #[allow(clippy::too_many_arguments)]
    pub fn shape_aligned(
        &mut self,
        text: &mut [u32],
        runs: &mut [TextRun],
        sizing: TextSizing,
        max_width: f32,
        max_height: f32,
        alignment: TextAlign,
        wrap: TextWrap,
        origin: TextOrigin,
        overflow: TextOverflow,
        paragraph_spacing: f32,
        align_width: f32,
        vertical_alignment: VerticalTextAlign,
        align_height: f32,
    ) {
        self.vertical_offset = 0.0;
        self.paragraphs = runs[0]
            .font
            .as_ref()
            .expect("shaped text retains its font")
            .shape_text(text, runs, -1);
        self.glyph_lookup.compute(text, &self.paragraphs);
        self.paragraph_lines = Text::break_lines_aligned(
            &self.paragraphs,
            if sizing == TextSizing::AutoWidth {
                -1.0
            } else {
                max_width
            },
            alignment,
            wrap,
            align_width,
        );
        self.ordered_lines.clear();
        self.ellipsis_run = GlyphRun::default();
        if self.paragraphs.is_empty() {
            self.bounds = Aabb::new(0.0, 0.0, 0.0, 0.0);
            return;
        }
        let mut y = 0.0;
        let mut min_y = 0.0;
        let mut min_x = f32::MAX;
        let mut max_x = -f32::MAX;
        if origin == TextOrigin::Baseline
            && !self.paragraph_lines.is_empty()
            && !self.paragraph_lines[0].is_empty()
        {
            y -= self.paragraph_lines[0][0].baseline;
            min_y = y;
        }
        let want_ellipsis = overflow == TextOverflow::Ellipsis && sizing == TextSizing::Fixed;
        let mut ellipsis_line = -1;
        let mut last_line_index = -1;
        for (paragraph, lines) in self.paragraphs.iter().zip(&self.paragraph_lines) {
            for line in lines {
                let end = &paragraph.runs[line.end_run_index as usize];
                let start = &paragraph.runs[line.start_run_index as usize];
                let width = end.xpos[line.end_glyph_index as usize]
                    - start.xpos[line.start_glyph_index as usize];
                min_x = min_x.min(line.start_x);
                max_x = max_x.max(line.start_x + width);
                last_line_index += 1;
                if want_ellipsis && y + line.bottom <= max_height {
                    ellipsis_line += 1;
                }
            }
            if let Some(last) = lines.last() {
                y += last.bottom;
            }
            y += paragraph_spacing;
        }
        if want_ellipsis && ellipsis_line == -1 {
            ellipsis_line = 0;
        }
        let ellipsis_last = last_line_index == ellipsis_line;
        if min_x > max_x {
            min_x = 0.0;
            max_x = 0.0;
        }
        let max_y = min_y.max(y - paragraph_spacing);
        let slack = align_height - (max_y - min_y);
        if slack > 0.0 {
            self.vertical_offset = match vertical_alignment {
                VerticalTextAlign::Middle => slack / 2.0,
                VerticalTextAlign::Bottom => slack,
                _ => 0.0,
            };
        }
        self.bounds = Aabb::new(
            min_x,
            min_y + self.vertical_offset,
            max_x,
            max_y + self.vertical_offset,
        );
        y = 0.0;
        if origin == TextOrigin::Baseline
            && !self.paragraph_lines.is_empty()
            && !self.paragraph_lines[0].is_empty()
        {
            y -= self.paragraph_lines[0][0].baseline;
        }
        let mut line_index = 0;
        for (paragraph, lines) in self.paragraphs.iter().zip(&self.paragraph_lines) {
            for line in lines {
                match overflow {
                    TextOverflow::Hidden
                        if sizing == TextSizing::Fixed && y + line.bottom > max_height =>
                    {
                        return;
                    }
                    TextOverflow::Clipped
                        if sizing == TextSizing::Fixed && y + line.top > max_height =>
                    {
                        return;
                    }
                    _ => {}
                }
                self.ordered_lines.push(OrderedLine::new(
                    paragraph,
                    line,
                    max_width,
                    ellipsis_line == line_index,
                    ellipsis_last,
                    &mut self.ellipsis_run,
                    y + line.baseline + self.vertical_offset,
                ));
                if line_index == ellipsis_line {
                    return;
                }
                line_index += 1;
            }
            if let Some(last) = lines.last() {
                y += last.bottom;
            }
            y += paragraph_spacing;
        }
    }
}
