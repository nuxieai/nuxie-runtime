//! Retained editable buffer ported from `src/text/raw_text_input.cpp`.

use super::cursor::{Cursor, CursorPosition};
use super::text_selection_path::TextSelectionPath;
use super::{
    RuntimeTextLayoutConstraint, StaticShapedTextLayout, StaticShapedTextLine, StaticTextSlice,
    char_byte_index, paragraph_base_is_rtl,
};
use crate::{ArtboardInstance, Mat2D, RuntimePathCommand};
use nuxie_binary::RuntimeFile;
use nuxie_graph::ArtboardGraph;
use nuxie_render_api::{Aabb as RenderAabb, Vec2D as RenderVec2D};
use unicode_properties::UnicodeGeneralCategory;

#[derive(Clone)]
pub(crate) struct TextInputGeometry {
    layout: StaticShapedTextLayout,
    local_bounds: Option<(f32, f32, f32, f32)>,
}

impl std::fmt::Debug for TextInputGeometry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TextInputGeometry")
            .field("text_len", &self.layout.text.len())
            .field("line_count", &self.layout.lines.len())
            .field("local_bounds", &self.local_bounds)
            .finish()
    }
}

impl TextInputGeometry {
    pub(crate) fn caret(&self, byte_offset: usize) -> Option<(RenderVec2D, RenderVec2D)> {
        self.layout.caret(byte_offset)
    }

    pub(crate) fn hit(&self, point: RenderVec2D) -> Option<usize> {
        let inverse = self.layout.shape_world.invert_or_identity();
        let (x, y) = inverse.transform_point(point.x, point.y);
        let line = self
            .layout
            .lines
            .iter()
            .find(|line| y <= line.bottom)
            .or_else(|| self.layout.lines.last())?;
        if x <= line.start_x.min(line.end_x) {
            let first = line.glyphs.first()?;
            let index = if first.glyph.rtl {
                first.glyph.char_index.saturating_add(first.glyph.char_len)
            } else {
                first.glyph.char_index
            };
            return Some(char_byte_index(&self.layout.text, index));
        }
        if x >= line.start_x.max(line.end_x) {
            let last = line.glyphs.last()?;
            let index = if last.glyph.rtl {
                last.glyph.char_index
            } else {
                last.glyph.char_index.saturating_add(last.glyph.char_len)
            };
            return Some(char_byte_index(&self.layout.text, index));
        }
        self.layout.hit(point)
    }

    pub(crate) fn selection_rects(&self, range: std::ops::Range<usize>) -> Vec<RenderAabb> {
        self.layout.selection_rects(range)
    }

    pub(crate) fn local_bounds(&self) -> Option<(f32, f32, f32, f32)> {
        self.local_bounds
    }

    pub(crate) fn line_metrics(&self) -> Vec<(usize, usize, f32, f32)> {
        self.layout
            .lines
            .iter()
            .map(|line| (line.char_start, line.char_end, line.top, line.bottom))
            .collect()
    }

    pub(crate) fn line_directions(&self) -> Vec<bool> {
        self.layout
            .lines
            .iter()
            .map(|line| self.line_is_rtl(line))
            .collect()
    }

    fn line_is_rtl(&self, line: &StaticShapedTextLine) -> bool {
        paragraph_base_is_rtl(&self.layout.text, line.char_start)
    }

    pub(crate) fn line_range(&self, codepoint_index: usize) -> Option<std::ops::Range<usize>> {
        let line =
            self.layout.lines.iter().find(|line| {
                codepoint_index >= line.char_start && codepoint_index <= line.char_end
            })?;
        Some(line.char_start..line.char_end)
    }

    pub(crate) fn vertical_cursor(
        &self,
        codepoint_index: usize,
        direction: i32,
        ideal_x: Option<f32>,
    ) -> Option<(usize, f32)> {
        let current = self.layout.lines.iter().position(|line| {
            codepoint_index >= line.char_start && codepoint_index <= line.char_end
        })?;
        let target = if direction < 0 {
            current.checked_sub(1)
        } else {
            current
                .checked_add(1)
                .filter(|index| *index < self.layout.lines.len())
        };
        let byte = char_byte_index(
            &self.layout.text,
            codepoint_index.min(self.layout.text.chars().count()),
        );
        let (top, bottom) = self.layout.caret(byte)?;
        let x = ideal_x.unwrap_or((top.x + bottom.x) * 0.5);
        let Some(target) = target else {
            return Some((
                if direction < 0 {
                    0
                } else {
                    self.layout.text.chars().count()
                },
                x,
            ));
        };
        let target_line = &self.layout.lines[target];
        let target_y = (target_line.top + target_line.bottom) * 0.5;
        let hit_byte = self.layout.hit(RenderVec2D::new(x, target_y))?;
        Some((self.layout.char_index_at_byte(hit_byte)?, x))
    }
}

pub(crate) fn build_text_input_geometry(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
) -> Option<TextInputGeometry> {
    let slice = StaticTextSlice::from_text_input_graph(runtime, graph, text_local).ok()?;
    let local_bounds = match layout_constraint {
        Some(constraint) => slice
            .local_bounds_with_layout_constraint(runtime, instance, constraint)
            .ok()
            .flatten(),
        None => slice.local_bounds(runtime, instance).ok().flatten(),
    };
    let layout = slice
        .shaped_layout(runtime, instance, layout_constraint, Mat2D::IDENTITY)
        .ok()??;
    Some(TextInputGeometry {
        layout,
        local_bounds,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorBoundary {
    Character,
    Word,
    SubWord,
    Line,
}

#[derive(Debug, Clone)]
struct JournalEntry {
    cursor_from: Cursor,
    cursor_to: Cursor,
    text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RawTextInput {
    text: Vec<char>,
    cursor: Cursor,
    journal: Vec<JournalEntry>,
    journal_index: usize,
    pub(crate) selection_corner_radius: f32,
    pub(crate) separate_selection_text: bool,
    ideal_cursor_x: Option<f32>,
    geometry: Option<TextInputGeometry>,
    geometry_dirty: bool,
    measure_cache: Option<((u32, u32), (f32, f32, f32, f32))>,
    measure_count: usize,
    selection_path: TextSelectionPath,
    selection_dirty: bool,
}

impl Default for RawTextInput {
    fn default() -> Self {
        Self {
            text: Vec::new(),
            cursor: Cursor::at_start(),
            journal: Vec::new(),
            journal_index: 0,
            selection_corner_radius: 5.0,
            separate_selection_text: false,
            ideal_cursor_x: None,
            geometry: None,
            geometry_dirty: true,
            measure_cache: None,
            measure_count: 0,
            selection_path: TextSelectionPath::default(),
            selection_dirty: true,
        }
    }
}

impl RawTextInput {
    pub(crate) fn text(&self) -> String {
        self.text.iter().collect()
    }

    pub(crate) fn length(&self) -> usize {
        self.text.len()
    }

    pub(crate) fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub(crate) fn set_cursor(&mut self, cursor: Cursor) -> bool {
        let cursor = Cursor {
            start: CursorPosition::unresolved(cursor.start.codepoint_index.min(self.length())),
            end: CursorPosition::unresolved(cursor.end.codepoint_index.min(self.length())),
        };
        if self.cursor == cursor {
            return false;
        }
        self.cursor = cursor;
        self.selection_dirty = true;
        true
    }

    pub(crate) fn set_text(&mut self, value: &str) -> bool {
        self.set_text_impl(value, false)
    }

    pub(crate) fn set_text_preserve_cursor(&mut self, value: &str) -> bool {
        self.set_text_impl(value, true)
    }

    fn set_text_impl(&mut self, value: &str, preserve_cursor: bool) -> bool {
        if self.text.iter().copied().eq(value.chars()) {
            return false;
        }
        let starting_cursor = self.cursor;
        self.text = value.chars().collect();
        self.cursor = if preserve_cursor {
            Cursor {
                start: CursorPosition::unresolved(
                    starting_cursor.start.codepoint_index.min(self.length()),
                ),
                end: CursorPosition::unresolved(
                    starting_cursor.end.codepoint_index.min(self.length()),
                ),
            }
        } else {
            Cursor::at_start()
        };
        self.ideal_cursor_x = None;
        self.geometry_dirty = true;
        self.measure_cache = None;
        self.selection_dirty = true;
        self.capture_journal_entry(starting_cursor);
        true
    }

    pub(crate) fn insert(&mut self, value: &str) -> bool {
        if value.is_empty() {
            return false;
        }
        let starting_cursor = self.cursor;
        self.erase_selection();
        let at = self.cursor.start.codepoint_index;
        let inserted = value.chars().collect::<Vec<_>>();
        let count = inserted.len();
        self.text.splice(at..at, inserted);
        self.cursor = Cursor::collapsed(CursorPosition::unresolved(at.saturating_add(count)));
        self.ideal_cursor_x = None;
        self.geometry_dirty = true;
        self.measure_cache = None;
        self.selection_dirty = true;
        self.capture_journal_entry(starting_cursor);
        true
    }

    pub(crate) fn backspace(&mut self, direction: i32) -> bool {
        let starting_cursor = self.cursor;
        if self.erase_selection() {
            self.capture_journal_entry(starting_cursor);
            return true;
        }
        let at = self.cursor.start.codepoint_index;
        let range = if direction > 0 {
            if at >= self.length() {
                return false;
            }
            at..self.cluster_end(at)
        } else {
            if at == 0 {
                return false;
            }
            self.cluster_start(at.saturating_sub(1))..at
        };
        let collapsed = range.start;
        self.text.drain(range);
        self.cursor = Cursor::collapsed(CursorPosition::unresolved(collapsed));
        self.ideal_cursor_x = None;
        self.geometry_dirty = true;
        self.measure_cache = None;
        self.selection_dirty = true;
        self.capture_journal_entry(starting_cursor);
        true
    }

    fn erase_selection(&mut self) -> bool {
        if self.cursor.is_collapsed() {
            return false;
        }
        let range = self.cursor.first().codepoint_index..self.cursor.last().codepoint_index;
        let collapsed = range.start;
        self.text.drain(range);
        self.cursor = Cursor::collapsed(CursorPosition::unresolved(collapsed));
        self.ideal_cursor_x = None;
        self.geometry_dirty = true;
        self.measure_cache = None;
        self.selection_dirty = true;
        true
    }

    fn is_combining_mark(value: char) -> bool {
        matches!(
            value.general_category(),
            unicode_properties::GeneralCategory::NonspacingMark
                | unicode_properties::GeneralCategory::SpacingMark
                | unicode_properties::GeneralCategory::EnclosingMark
        )
    }

    fn cluster_start(&self, mut index: usize) -> usize {
        while index > 0
            && self
                .text
                .get(index)
                .is_some_and(|c| Self::is_combining_mark(*c))
        {
            index -= 1;
        }
        index
    }

    fn cluster_end(&self, index: usize) -> usize {
        let mut end = index.saturating_add(1).min(self.length());
        while self
            .text
            .get(end)
            .is_some_and(|c| Self::is_combining_mark(*c))
        {
            end += 1;
        }
        end
    }

    pub(crate) fn cursor_horizontal(
        &mut self,
        direction: i32,
        boundary: CursorBoundary,
        select: bool,
        line_range: Option<std::ops::Range<usize>>,
    ) -> bool {
        self.ideal_cursor_x = None;
        let end = self.cursor.end.codepoint_index;
        let next = match boundary {
            CursorBoundary::Character if direction > 0 => self.cluster_end(end),
            CursorBoundary::Character => self.cluster_start(end.saturating_sub(1)),
            CursorBoundary::Line => {
                line_range.map_or(
                    end,
                    |line| {
                        if direction < 0 { line.start } else { line.end }
                    },
                )
            }
            CursorBoundary::Word | CursorBoundary::SubWord => {
                self.word_boundary(end, direction, boundary == CursorBoundary::SubWord)
            }
        }
        .min(self.length());
        let position = CursorPosition::unresolved(next);
        let next_cursor = if select {
            Cursor {
                start: self.cursor.start,
                end: position,
            }
        } else {
            Cursor::collapsed(position)
        };
        self.set_cursor(next_cursor)
    }

    fn word_boundary(&self, start: usize, direction: i32, subword: bool) -> usize {
        if direction > 0 {
            let mut index = start;
            if index >= self.length() {
                return self.length();
            }
            while index < self.length() && classify(self.text[index]) == Class::Whitespace {
                index += 1;
            }
            if index >= self.length() {
                return index;
            }
            let class = classify(self.text[index]);
            if !class.is_word() {
                while index < self.length() && classify(self.text[index]) == class {
                    index += 1;
                }
                return index;
            }
            if !subword {
                while index < self.length() && classify(self.text[index]).is_word() {
                    index += 1;
                }
                return index;
            }
            match class {
                Class::Lower | Class::Underscore => {
                    while index < self.length()
                        && matches!(classify(self.text[index]), Class::Lower | Class::Underscore)
                    {
                        index += 1;
                    }
                }
                Class::Upper => {
                    index += 1;
                    while index < self.length() && classify(self.text[index]) == Class::Lower {
                        index += 1;
                    }
                }
                Class::Symbol | Class::Whitespace => unreachable!(),
            }
            index
        } else {
            let mut index = start;
            if index == 0 {
                return 0;
            }
            while index > 0 && classify(self.text[index - 1]) == Class::Whitespace {
                index -= 1;
            }
            if index == 0 {
                return 0;
            }
            let class = classify(self.text[index - 1]);
            if !class.is_word() {
                while index > 0 && classify(self.text[index - 1]) == class {
                    index -= 1;
                }
                return index;
            }
            if !subword {
                while index > 0 && classify(self.text[index - 1]).is_word() {
                    index -= 1;
                }
                return index;
            }
            match class {
                Class::Lower | Class::Underscore => {
                    while index > 0
                        && matches!(
                            classify(self.text[index - 1]),
                            Class::Lower | Class::Underscore
                        )
                    {
                        index -= 1;
                    }
                    if index > 0 && classify(self.text[index - 1]) == Class::Upper {
                        index -= 1;
                    }
                }
                Class::Upper => {
                    while index > 0 && classify(self.text[index - 1]) == Class::Upper {
                        index -= 1;
                    }
                }
                Class::Symbol | Class::Whitespace => unreachable!(),
            }
            index
        }
    }

    pub(crate) fn move_cursor_to(&mut self, codepoint_index: usize, select: bool) -> bool {
        self.ideal_cursor_x = None;
        let position = CursorPosition::unresolved(codepoint_index.min(self.length()));
        self.set_cursor(if select {
            Cursor {
                start: self.cursor.start,
                end: position,
            }
        } else {
            Cursor::collapsed(position)
        })
    }

    pub(crate) fn ideal_cursor_x(&self) -> Option<f32> {
        self.ideal_cursor_x
    }

    pub(crate) fn geometry(&self) -> Option<&TextInputGeometry> {
        self.geometry.as_ref()
    }

    pub(crate) fn geometry_dirty(&self) -> bool {
        self.geometry_dirty || self.geometry.is_none()
    }

    pub(crate) fn mark_geometry_dirty(&mut self) {
        self.geometry_dirty = true;
        self.measure_cache = None;
        self.selection_dirty = true;
    }

    pub(crate) fn selection_needs_update(&self) -> bool {
        self.selection_dirty
    }

    pub(crate) fn cached_measure(
        &self,
        max_width: f32,
        max_height: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        self.measure_cache
            .filter(|(key, _)| *key == (max_width.to_bits(), max_height.to_bits()))
            .map(|(_, bounds)| bounds)
    }

    pub(crate) fn retain_measure(
        &mut self,
        max_width: f32,
        max_height: f32,
        bounds: (f32, f32, f32, f32),
    ) {
        self.measure_cache = Some(((max_width.to_bits(), max_height.to_bits()), bounds));
        self.measure_count = self.measure_count.saturating_add(1);
    }

    pub(crate) fn measure_count(&self) -> usize {
        self.measure_count
    }

    pub(crate) fn set_geometry(&mut self, geometry: Option<TextInputGeometry>) -> bool {
        self.geometry = geometry;
        self.geometry_dirty = false;
        self.selection_dirty = true;
        self.geometry.is_some()
    }

    pub(crate) fn set_selection_corner_radius(&mut self, radius: f32) -> bool {
        if self.selection_corner_radius == radius {
            return false;
        }
        self.selection_corner_radius = radius;
        self.selection_dirty = true;
        true
    }

    pub(crate) fn selection_path_commands(&mut self) -> Vec<RuntimePathCommand> {
        if self.selection_dirty {
            let cursor = self.cursor;
            let rects = if cursor.has_selection() {
                let text = self.text();
                let start = super::char_byte_index(&text, cursor.first().codepoint_index);
                let end = super::char_byte_index(&text, cursor.last().codepoint_index);
                self.geometry
                    .as_ref()
                    .map(|geometry| geometry.selection_rects(start..end))
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            self.selection_path
                .update(&rects, self.selection_corner_radius);
            self.selection_dirty = false;
        }
        self.selection_path.commands().to_vec()
    }

    pub(crate) fn move_cursor_vertical(
        &mut self,
        codepoint_index: usize,
        select: bool,
        ideal_cursor_x: f32,
    ) -> bool {
        let position = CursorPosition::unresolved(codepoint_index.min(self.length()));
        let changed = self.set_cursor(if select {
            Cursor {
                start: self.cursor.start,
                end: position,
            }
        } else {
            Cursor::collapsed(position)
        });
        self.ideal_cursor_x = Some(ideal_cursor_x);
        changed
    }

    pub(crate) fn select_word(&mut self) -> bool {
        if self.text.is_empty() {
            return false;
        }
        let mut at = self
            .cursor
            .start
            .codepoint_index
            .min(self.length().saturating_sub(1));
        if !classify(self.text[at]).is_word() && at > 0 && classify(self.text[at - 1]).is_word() {
            at -= 1;
        }
        let class = classify(self.text[at]);
        let matches = |other: Class| {
            if class.is_word() {
                other.is_word()
            } else {
                other == class
            }
        };
        let mut start = at;
        while start > 0 && matches(classify(self.text[start - 1])) {
            start -= 1;
        }
        let mut end = at.saturating_add(1);
        while end < self.length() && matches(classify(self.text[end])) {
            end += 1;
        }
        self.set_cursor(Cursor {
            start: CursorPosition::unresolved(start),
            end: CursorPosition::unresolved(end),
        })
    }

    pub(crate) fn select_all(&mut self) -> bool {
        self.ideal_cursor_x = None;
        self.set_cursor(Cursor {
            start: CursorPosition::unresolved(0),
            end: CursorPosition::unresolved(self.length()),
        })
    }

    pub(crate) fn select_line(&mut self, range: std::ops::Range<usize>) -> bool {
        self.ideal_cursor_x = None;
        self.set_cursor(Cursor {
            start: CursorPosition::unresolved(range.start.min(self.length())),
            end: CursorPosition::unresolved(range.end.min(self.length())),
        })
    }

    pub(crate) fn undo(&mut self) -> bool {
        if self.journal_index == 0 || self.journal.is_empty() {
            return false;
        }
        let from_cursor = self.journal[self.journal_index].cursor_from;
        self.journal_index -= 1;
        self.text = self.journal[self.journal_index].text.chars().collect();
        self.cursor = from_cursor;
        self.ideal_cursor_x = None;
        self.geometry_dirty = true;
        self.measure_cache = None;
        self.selection_dirty = true;
        true
    }

    pub(crate) fn redo(&mut self) -> bool {
        if self.journal_index.saturating_add(1) >= self.journal.len() {
            return false;
        }
        self.journal_index += 1;
        let entry = &self.journal[self.journal_index];
        self.text = entry.text.chars().collect();
        self.cursor = entry.cursor_to;
        self.ideal_cursor_x = None;
        self.geometry_dirty = true;
        self.measure_cache = None;
        self.selection_dirty = true;
        true
    }

    fn capture_journal_entry(&mut self, cursor_from: Cursor) {
        if self.journal_index.saturating_add(1) < self.journal.len() {
            self.journal.truncate(self.journal_index.saturating_add(1));
        }
        self.journal.push(JournalEntry {
            cursor_from,
            cursor_to: self.cursor,
            text: self.text(),
        });
        self.journal_index = self.journal.len().saturating_sub(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Lower,
    Upper,
    Symbol,
    Underscore,
    Whitespace,
}

impl Class {
    fn is_word(self) -> bool {
        matches!(self, Self::Lower | Self::Upper | Self::Underscore)
    }
}

fn classify(value: char) -> Class {
    if value.is_whitespace() {
        Class::Whitespace
    } else if value == '_' {
        Class::Underscore
    } else if value.is_ascii_uppercase() {
        Class::Upper
    } else if value.is_ascii_punctuation() || value.is_ascii_control() {
        Class::Symbol
    } else {
        Class::Lower
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cursor(raw: &RawTextInput) -> (usize, usize) {
        (
            raw.cursor().start.codepoint_index,
            raw.cursor().end.codepoint_index,
        )
    }

    #[test]
    fn upstream_insert_delete_selection_and_text_contracts_are_ported() {
        let mut raw = RawTextInput::default();
        assert!(raw.insert("hello "));
        assert_eq!(raw.text(), "hello ");
        raw.set_cursor(Cursor::at_start());
        raw.move_cursor_to(raw.length(), false);
        raw.insert("world");
        assert_eq!(raw.text(), "hello world");
        raw.set_cursor(Cursor {
            start: CursorPosition::unresolved(5),
            end: CursorPosition::unresolved(11),
        });
        raw.insert("!");
        assert_eq!(raw.text(), "hello!");
        assert_eq!(cursor(&raw), (6, 6));
    }

    #[test]
    fn upstream_combining_cluster_movement_and_deletion_are_ported() {
        let mut raw = RawTextInput::default();
        raw.insert("cafe\u{301}s");
        raw.set_cursor(Cursor::at_start());
        for expected in [1, 2, 3, 5] {
            raw.cursor_horizontal(1, CursorBoundary::Character, false, None);
            assert_eq!(raw.cursor().start.codepoint_index, expected);
        }
        raw.backspace(-1);
        assert_eq!(raw.text(), "cafs");
        assert_eq!(raw.cursor().start.codepoint_index, 3);

        raw.set_text("cafe\u{301}s");
        raw.set_cursor(Cursor::collapsed(CursorPosition::unresolved(3)));
        raw.backspace(1);
        assert_eq!(raw.text(), "cafs");
        assert_eq!(raw.cursor().start.codepoint_index, 3);
    }

    #[test]
    fn upstream_word_selection_and_journal_contracts_are_ported() {
        let mut raw = RawTextInput::default();
        raw.insert("oneTwo three == four");
        raw.set_cursor(Cursor::collapsed(CursorPosition::unresolved(9)));
        raw.select_word();
        assert_eq!(cursor(&raw), (7, 12));
        raw.insert("X");
        assert_eq!(raw.text(), "oneTwo X == four");
        assert!(raw.undo());
        assert_eq!(raw.text(), "oneTwo three == four");
        assert_eq!(cursor(&raw), (7, 12));
        assert!(raw.redo());
        assert_eq!(raw.text(), "oneTwo X == four");
    }

    #[test]
    fn upstream_multiline_select_all_and_line_contracts_are_ported() {
        let mut raw = RawTextInput::default();
        raw.insert("hello\nworld");
        raw.set_cursor(Cursor::collapsed(CursorPosition::unresolved(8)));
        raw.select_line(6..11);
        assert_eq!(cursor(&raw), (6, 11));
        raw.select_all();
        assert_eq!(cursor(&raw), (0, 11));
    }

    #[test]
    fn upstream_word_cursor_sequence_is_ported() {
        let mut raw = RawTextInput::default();
        raw.insert("one two three fo4ur five");
        raw.set_cursor(Cursor::at_start());
        for expected in [3, 7] {
            raw.cursor_horizontal(1, CursorBoundary::Word, false, None);
            assert_eq!(raw.cursor().start.codepoint_index, expected);
        }
        for expected in [4, 0] {
            raw.cursor_horizontal(-1, CursorBoundary::Word, false, None);
            assert_eq!(raw.cursor().start.codepoint_index, expected);
        }
        for _ in 0..4 {
            raw.cursor_horizontal(1, CursorBoundary::Word, false, None);
        }
        assert_eq!(raw.cursor().start.codepoint_index, 19);
        raw.cursor_horizontal(-1, CursorBoundary::Character, false, None);
        raw.cursor_horizontal(-1, CursorBoundary::Character, false, None);
        raw.cursor_horizontal(-1, CursorBoundary::Word, false, None);
        assert_eq!(raw.cursor().start.codepoint_index, 14);
        raw.cursor_horizontal(1, CursorBoundary::Character, false, None);
        raw.cursor_horizontal(1, CursorBoundary::Character, false, None);
        raw.cursor_horizontal(1, CursorBoundary::Word, false, None);
        assert_eq!(raw.cursor().start.codepoint_index, 19);
    }

    #[test]
    fn upstream_subword_cursor_sequence_is_ported() {
        let mut raw = RawTextInput::default();
        raw.insert("oneTwo threeFo+ur fi--ve");
        raw.set_cursor(Cursor::at_start());
        for expected in [3, 6, 12, 14, 15, 17] {
            raw.cursor_horizontal(1, CursorBoundary::SubWord, false, None);
            assert_eq!(raw.cursor().start.codepoint_index, expected);
        }
        for expected in [15, 14, 12, 7] {
            raw.cursor_horizontal(-1, CursorBoundary::SubWord, false, None);
            assert_eq!(raw.cursor().start.codepoint_index, expected);
        }
        for expected in [14, 15, 17] {
            raw.cursor_horizontal(1, CursorBoundary::Word, false, None);
            assert_eq!(raw.cursor().start.codepoint_index, expected);
        }
        for expected in [20, 22] {
            raw.cursor_horizontal(1, CursorBoundary::SubWord, false, None);
            assert_eq!(raw.cursor().start.codepoint_index, expected);
        }
        raw.cursor_horizontal(-1, CursorBoundary::SubWord, false, None);
        assert_eq!(raw.cursor().start.codepoint_index, 20);
    }

    #[test]
    fn upstream_word_selection_edges_are_ported() {
        let mut raw = RawTextInput::default();
        raw.insert("oneTwo three == four");
        for (at, expected) in [(0, (0, 6)), (9, (7, 12)), (12, (7, 12)), (14, (13, 15))] {
            raw.set_cursor(Cursor::collapsed(CursorPosition::unresolved(at)));
            raw.select_word();
            assert_eq!(cursor(&raw), expected);
        }
    }

    #[test]
    fn upstream_journal_branching_sequence_is_ported() {
        let mut raw = RawTextInput::default();
        raw.insert("oneTwo");
        raw.set_cursor(Cursor::at_start());
        for _ in 0..3 {
            raw.cursor_horizontal(1, CursorBoundary::Character, false, None);
        }
        for inserted in [" ", "2", " "] {
            raw.insert(inserted);
        }
        assert_eq!((raw.text(), cursor(&raw)), ("one 2 Two".to_owned(), (6, 6)));
        for (text, expected_cursor) in [
            ("one 2Two", (5, 5)),
            ("one Two", (4, 4)),
            ("oneTwo", (3, 3)),
        ] {
            assert!(raw.undo());
            assert_eq!(
                (raw.text(), cursor(&raw)),
                (text.to_owned(), expected_cursor)
            );
        }
        assert!(raw.redo());
        assert_eq!((raw.text(), cursor(&raw)), ("one Two".to_owned(), (4, 4)));
        raw.insert("X");
        assert!(!raw.redo());
        assert!(raw.undo());
        for _ in 0..3 {
            raw.cursor_horizontal(1, CursorBoundary::Character, true, None);
        }
        raw.insert("2");
        assert_eq!((raw.text(), cursor(&raw)), ("one 2".to_owned(), (5, 5)));
        assert!(raw.undo());
        assert_eq!((raw.text(), cursor(&raw)), ("one Two".to_owned(), (4, 7)));
    }
}
