#![cfg(feature = "rive_text")]
use super::fully_shaped_text::FullyShapedText;
use crate::mechanical_port::source::{
    math::{aabb::Aabb, vec2d::Vec2D},
    shapes::shape_paint_path::ShapePaintPath,
    text_engine::TextDirection,
};
#[derive(Clone, Copy)]
pub struct CursorVisualPosition {
    found: bool,
    x: f32,
    top: f32,
    bottom: f32,
}
impl CursorVisualPosition {
    pub fn missing() -> Self {
        Self {
            found: false,
            x: 0.0,
            top: 0.0,
            bottom: 0.0,
        }
    }
    pub fn new(x: f32, top: f32, bottom: f32) -> Self {
        Self {
            found: true,
            x,
            top,
            bottom,
        }
    }
    pub fn found(&self) -> bool {
        self.found
    }
    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn top(&self) -> f32 {
        self.top
    }
    pub fn bottom(&self) -> f32 {
        self.bottom
    }
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CursorPosition {
    line_index: u32,
    code_point_index: u32,
}
impl CursorPosition {
    pub fn new(line: u32, index: u32) -> Self {
        Self {
            line_index: line,
            code_point_index: index,
        }
    }
    pub fn unresolved(index: u32) -> Self {
        Self {
            line_index: u32::MAX,
            code_point_index: index,
        }
    }
    pub fn zero() -> Self {
        Self::new(0, 0)
    }
    pub fn line_index(&self) -> u32 {
        self.line_index
    }
    pub fn line_index_offset(&self, inc: i32) -> u32 {
        if inc < 0 && (-inc) as u32 > self.line_index {
            0
        } else {
            self.line_index.wrapping_add(inc as u32)
        }
    }
    pub fn code_point_index(&self) -> u32 {
        self.code_point_index
    }
    pub fn code_point_index_offset(&self, inc: i32) -> u32 {
        if inc < 0 && (-inc) as u32 > self.code_point_index {
            0
        } else {
            self.code_point_index.wrapping_add(inc as u32)
        }
    }
    pub fn has_line_index(&self) -> bool {
        self.line_index != u32::MAX
    }
    pub fn visual_position(&self, shape: &FullyShapedText) -> CursorVisualPosition {
        let lookup = shape.glyph_lookup();
        let lines = shape.ordered_lines();
        let target = lookup.get(self.code_point_index);
        let Some(line) = lines.get(self.line_index as usize) else {
            return CursorVisualPosition::missing();
        };
        let glyph_line = line.glyph_line();
        let mut x = glyph_line.start_x;
        let (mut first, mut last, mut have) = (0, 0, false);
        for glyph in line.iter() {
            let run = glyph.run;
            let i = glyph.glyph_index as usize;
            let advance = run.advances[i];
            if advance != 0.0 && target == lookup.get(run.text_indices[i]) {
                x += advance
                    * lookup.advance_factor(
                        self.code_point_index as i32,
                        run.direction() == TextDirection::Rtl,
                    );
                let font = &run.font;
                return CursorVisualPosition::new(
                    x,
                    line.y() + font.ascent(run.size),
                    line.y() + font.descent(run.size),
                );
            }
            if !have {
                first = run.text_indices[i];
                last = first;
                have = true;
            } else {
                last = run.text_indices[i];
            }
            x += advance;
        }
        let run = line.last_run();
        let final_x =
            if self.code_point_index.abs_diff(first) < self.code_point_index.abs_diff(last) {
                glyph_line.start_x
            } else {
                x
            };
        CursorVisualPosition::new(
            final_x,
            line.y() + run.font.ascent(run.size),
            line.y() + run.font.descent(run.size),
        )
    }
    pub fn from_translation(p: Vec2D, shape: &FullyShapedText) -> Self {
        let lines = shape.ordered_lines();
        if lines.is_empty() {
            return Self::zero();
        }
        let max = lines.len() - 1;
        for (line_index, line) in lines.iter().enumerate() {
            if line.bottom() < p.y && line_index != max {
                continue;
            }
            return Self::from_ordered_line(line, line_index as u32, p.x, shape);
        }
        Self::zero()
    }
    fn from_ordered_line(
        line: &crate::mechanical_port::source::text_engine::OrderedLine,
        line_index: u32,
        x_target: f32,
        shape: &FullyShapedText,
    ) -> Self {
        let lookup = shape.glyph_lookup();
        let mut x = line.glyph_line().start_x;
        let mut last = None;
        for glyph in line.iter() {
            last = Some(glyph);
            let run = glyph.run;
            let i = glyph.glyph_index as usize;
            let advance = run.advances[i];
            if x_target <= x + advance {
                let ratio = if advance == 0.0 {
                    1.0
                } else {
                    ((x_target - x) / advance).clamp(0.0, 1.0)
                };
                let text = run.text_indices[i];
                let absolute = lookup.get(text);
                let mut next = text;
                while next != lookup.size().saturating_sub(1) as u32 && lookup.get(next) == absolute
                {
                    next += 1;
                }
                let part = (ratio * (next - text) as f32).round() as u32;
                return Self::new(
                    line_index,
                    if run.direction() == TextDirection::Ltr {
                        text + part
                    } else {
                        next.saturating_sub(part)
                    },
                )
                .clamped(shape);
            }
            x += advance;
        }
        let glyph = last.expect("OrderedLine has a glyph");
        let run = glyph.run;
        let text = run.text_indices[glyph.glyph_index as usize];
        let absolute = lookup.get(text);
        let mut next = text;
        while next != lookup.size().saturating_sub(1) as u32 && lookup.get(next) == absolute {
            next += 1;
        }
        Self::new(
            line_index,
            if run.direction() == TextDirection::Ltr {
                next
            } else {
                text
            },
        )
        .clamped(shape)
    }
    pub fn from_line_x(line: u32, x: f32, shape: &FullyShapedText) -> Self {
        shape
            .ordered_lines()
            .get(line as usize)
            .map_or(Self::zero(), |ordered| {
                Self::from_ordered_line(ordered, line, x, shape)
            })
    }
    pub fn clamped(&self, shape: &FullyShapedText) -> Self {
        Self::new(
            self.line_index
                .min(shape.ordered_lines().len().saturating_sub(1) as u32),
            self.code_point_index.min(
                shape
                    .glyph_lookup()
                    .last_code_point_index()
                    .saturating_sub(1),
            ),
        )
    }
    pub fn at_index(index: u32, shape: &FullyShapedText) -> Self {
        if index
            >= shape
                .glyph_lookup()
                .last_code_point_index()
                .saturating_sub(1)
        {
            return Self::new(
                shape.ordered_lines().len().saturating_sub(1) as u32,
                shape
                    .glyph_lookup()
                    .last_code_point_index()
                    .saturating_sub(1),
            );
        }
        let mut line_index = 0;
        for (paragraph, lines) in shape.paragraphs().iter().zip(shape.paragraph_lines()) {
            for line in lines {
                let run = &paragraph.runs[line.start_run_index as usize];
                if run.text_indices[line.start_glyph_index as usize] <= index {
                    line_index += 1;
                    continue;
                }
                return Self::new(line_index - 1, index).clamped(shape);
            }
        }
        Self::new(line_index - 1, index).clamped(shape)
    }
    pub fn resolve_line(&mut self, shape: &FullyShapedText) {
        self.line_index = shape
            .ordered_lines()
            .iter()
            .position(|line| {
                line.contains_code_point_index(shape.glyph_lookup(), self.code_point_index)
            })
            .unwrap_or(shape.ordered_lines().len()) as u32;
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    start: CursorPosition,
    end: CursorPosition,
}
impl Cursor {
    pub fn new(start: CursorPosition, end: CursorPosition) -> Self {
        Self { start, end }
    }
    pub fn collapsed(p: CursorPosition) -> Self {
        Self::new(p, p)
    }
    pub fn zero() -> Self {
        Self::collapsed(CursorPosition::zero())
    }
    pub fn at_start() -> Self {
        Self::zero()
    }
    pub fn start(&self) -> CursorPosition {
        self.start
    }
    pub fn end(&self) -> CursorPosition {
        self.end
    }
    pub fn first(&self) -> CursorPosition {
        self.start.min(self.end)
    }
    pub fn last(&self) -> CursorPosition {
        self.start.max(self.end)
    }
    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }
    pub fn has_selection(&self) -> bool {
        self.start != self.end
    }
    pub fn resolve_line_positions(&mut self, shape: &FullyShapedText) -> bool {
        let mut resolved = false;
        if !self.start.has_line_index() {
            self.start.resolve_line(shape);
            resolved = true;
        }
        if !self.end.has_line_index() {
            self.end.resolve_line(shape);
            resolved = true;
        }
        resolved
    }
    pub fn contains(&self, index: u32) -> bool {
        index >= self.first().code_point_index && index < self.last().code_point_index
    }
    pub fn update_selection_path(
        &self,
        _path: &mut ShapePaintPath,
        _rects: &[Aabb],
        _shape: &FullyShapedText,
    ) {
    }
    pub fn selection_rects(&self, rects: &mut Vec<Aabb>, shape: &FullyShapedText) {
        let first = self.first().clamped(shape);
        let last = self.last().clamped(shape);
        let lookup = shape.glyph_lookup();
        for line_index in first.line_index..=last.line_index {
            let line = &shape.ordered_lines()[line_index as usize];
            let mut x = line.glyph_line().start_x;
            for glyph in line.iter() {
                let run = glyph.run;
                let i = glyph.glyph_index as usize;
                let advance = run.advances[i];
                let index = run.text_indices[i];
                let count = lookup.count(index);
                let end = index + count;
                if last.code_point_index > index && end > first.code_point_index {
                    let after = first.code_point_index.saturating_sub(index);
                    let before = end.saturating_sub(last.code_point_index);
                    let (mut sf, mut ef) = (
                        after as f32 / count as f32,
                        (count - before) as f32 / count as f32,
                    );
                    if run.direction() == TextDirection::Rtl {
                        sf = 1.0 - sf;
                        ef = 1.0 - ef;
                    }
                    let (mut left, mut right) = (x + advance * sf, x + advance * ef);
                    if left > right {
                        std::mem::swap(&mut left, &mut right);
                    }
                    rects.push(Aabb::new(
                        left,
                        line.y() + run.font.ascent(run.size),
                        right,
                        line.y() + run.font.descent(run.size),
                    ));
                }
                x += advance;
            }
        }
    }
}
