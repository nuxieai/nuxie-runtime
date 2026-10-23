use super::text::{ColorGlyphImageCache, color_glyph_cache_key, draw_color_glyph};
use std::ops::{BitAnd, BitOr, BitOrAssign, Not};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::mechanical_port::source::factory::RuntimeFactoryHandle;
use crate::mechanical_port::source::math::aabb::Aabb;
use crate::mechanical_port::source::math::mat2d::Mat2D;
use crate::mechanical_port::source::math::raw_path::RawPath;
use crate::mechanical_port::source::math::vec2d::Vec2D;
use crate::mechanical_port::source::renderer::{RenderPaint, RenderPath, Renderer};
use crate::mechanical_port::source::shapes::paint::shape_paint_path::ShapePaintPath;
use crate::mechanical_port::source::text::cursor::{Cursor, CursorPosition, CursorVisualPosition};
use crate::mechanical_port::source::text::fully_shaped_text::FullyShapedText;
use crate::mechanical_port::source::text::text_selection_path::TextSelectionPath;
use crate::mechanical_port::source::text::utf::Utf;
use crate::mechanical_port::source::text_engine::{
    FontRef, GlyphLine, GlyphRenderKind, OrderedLine, TextAlign, TextOrigin, TextOverflow, TextRun,
    TextSizing, TextWrap, Unichar, VerticalTextAlign, is_white_space,
};

const ZERO_WIDTH_SPACE: Unichar = 8203;
const OBSCURING_BULLET: Unichar = 0x2022;

// src/text/raw_text_input.cpp::encodeCodePoints at ed92313a. Encode the
// complete counted span, including any embedded zero inserted as a code point.
fn encode_code_points(code_points: &[Unichar]) -> String {
    let mut buffer = vec![0; Utf::count_code_point_length(code_points) as usize];
    let mut encoded = 0;
    for code_point in code_points {
        encoded += Utf::encode(&mut buffer[encoded..], *code_point) as usize;
    }
    String::from_utf8(buffer).unwrap()
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorBoundary {
    Character,
    Word,
    SubWord,
    Line,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flags {
    None = 0,
    ShapeDirty = 1 << 0,
    SelectionDirty = 1 << 1,
    SeparateSelectionText = 1 << 2,
    MeasureDirty = 1 << 3,
    Obscured = 1 << 4,
    TextPathDirty = 1 << 5,
}

impl BitOr for Flags {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

impl BitOr<Flags> for u8 {
    type Output = u8;

    fn bitor(self, rhs: Flags) -> Self::Output {
        self | rhs as u8
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Delineator(u8);

impl Delineator {
    pub const Unknown: Self = Self(0);
    pub const Lowercase: Self = Self(1 << 0);
    pub const Uppercase: Self = Self(1 << 1);
    pub const Symbol: Self = Self(1 << 2);
    pub const Underscore: Self = Self(1 << 3);
    pub const Whitespace: Self = Self(1 << 4);
    pub const Word: Self = Self((1 << 0) | (1 << 1) | (1 << 3));
    pub const Any: Self = Self((1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4));
}

impl BitOr for Delineator {
    type Output = Delineator;

    fn bitor(self, rhs: Self) -> Self::Output {
        Delineator(self.0 | rhs.0)
    }
}

impl BitAnd for Delineator {
    type Output = Delineator;

    fn bitand(self, rhs: Self) -> Self::Output {
        Delineator(self.0 & rhs.0)
    }
}

impl BitAnd<u8> for Delineator {
    type Output = u8;

    fn bitand(self, rhs: u8) -> Self::Output {
        self.0 & rhs
    }
}

impl Not for Delineator {
    type Output = u8;

    fn not(self) -> Self::Output {
        !self.0
    }
}

// Our additive text-field route retains the exact outline transform. Selection
// only chooses which drawable owns the glyph; it never recolors the glyph.
struct InputColorGlyph {
    font: FontRef,
    glyph_id: u16,
    cluster_text: std::sync::Arc<[Unichar]>,
    transform: Mat2D,
    selected: bool,
}

#[derive(Clone)]
struct JournalEntry {
    cursor_from: Cursor,
    cursor_to: Cursor,
    text: String,
}

pub struct RawTextInput {
    #[cfg(any(test, feature = "tools"))]
    pub measure_count: u32,
    cursor: Cursor,
    text_run: TextRun,
    css_line_box: bool,
    text_path: ShapePaintPath,
    selected_text_path: ShapePaintPath,
    color_glyphs: Vec<InputColorGlyph>,
    emoji_image_cache: ColorGlyphImageCache,
    cursor_path: ShapePaintPath,
    selection_path: TextSelectionPath,
    text: Vec<Unichar>,
    obscured_text: Vec<Unichar>,
    shape: FullyShapedText,
    // Upstream measurement only. CSS uses the primary positioned layout.
    measuring_shape: Option<Box<FullyShapedText>>,
    last_measure_max_width: f32,
    last_measure_max_height: f32,
    flags: u8,
    paragraph_spacing: f32,
    origin: TextOrigin,
    sizing: TextSizing,
    overflow: TextOverflow,
    align: TextAlign,
    align_width: f32,
    vertical_align: VerticalTextAlign,
    align_height: f32,
    wrap: TextWrap,
    max_width: f32,
    max_height: f32,
    clip_render_path: Option<Rc<RefCell<Box<RenderPath>>>>,
    ideal_cursor_x: f32,
    cursor_visual_position: CursorVisualPosition,
    selection_rects: Vec<Aabb>,
    selection_corner_radius: f32,
    journal: Vec<JournalEntry>,
    journal_index: u32,
}

impl Default for RawTextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl RawTextInput {
    const CARET_WIDTH: f32 = 1.0;

    pub fn new() -> Self {
        Self {
            #[cfg(any(test, feature = "tools"))]
            measure_count: 0,
            cursor: Cursor::at_start(),
            css_line_box: false,
            text_run: TextRun {
                font: None,
                size: 16.0,
                line_height: -1.0,
                letter_spacing: 0.0,
                unichar_count: 0,
                script: 0,
                style_id: 0,
                level: 0,
            },
            text_path: ShapePaintPath::default(),
            selected_text_path: ShapePaintPath::default(),
            color_glyphs: Vec::new(),
            emoji_image_cache: Vec::new(),
            cursor_path: ShapePaintPath::default(),
            selection_path: TextSelectionPath::default(),
            text: vec![ZERO_WIDTH_SPACE],
            obscured_text: Vec::new(),
            shape: FullyShapedText::default(),
            measuring_shape: None,
            last_measure_max_width: 0.0,
            last_measure_max_height: 0.0,
            flags: Flags::None as u8,
            paragraph_spacing: 0.0,
            origin: TextOrigin::Top,
            sizing: TextSizing::AutoWidth,
            overflow: TextOverflow::Visible,
            align: TextAlign::Left,
            align_width: 0.0,
            vertical_align: VerticalTextAlign::Top,
            align_height: 0.0,
            wrap: TextWrap::Wrap,
            max_width: 0.0,
            max_height: 0.0,
            clip_render_path: None,
            ideal_cursor_x: -1.0,
            cursor_visual_position: CursorVisualPosition::missing(),
            selection_rects: Vec::new(),
            selection_corner_radius: 5.0,
            journal: vec![JournalEntry {
                cursor_from: Cursor::at_start(),
                cursor_to: Cursor::at_start(),
                text: String::new(),
            }],
            journal_index: 0,
        }
    }

    pub fn draw(
        &mut self,
        factory: &RuntimeFactoryHandle,
        renderer: &mut Renderer,
        _world_transform: &Mat2D,
        text_paint: &mut RenderPaint,
        selection_paint: &mut RenderPaint,
        cursor_paint: &mut RenderPaint,
    ) {
        if self.overflow == TextOverflow::Clipped && self.clip_render_path.is_some() {
            renderer.save();
            renderer.clip_path(self.clip_render_path.as_ref().unwrap().borrow().as_ref());
        }
        if self.cursor.has_selection() {
            let render_path = self.selection_path.path.render_path(factory);
            renderer.draw_path(render_path, selection_paint);
        }
        let render_path = self.text_path.render_path(factory);
        renderer.draw_path(render_path, text_paint);
        self.draw_color_glyphs(factory, renderer, Mat2D::default(), None, 0xff000000, 1.0);
        let cursor_render_path = self.cursor_path.render_path(factory);
        renderer.draw_path(cursor_render_path, cursor_paint);
        if self.overflow == TextOverflow::Clipped && self.clip_render_path.is_some() {
            renderer.restore();
        }
    }

    pub(crate) fn has_color_glyphs(&self, selected: Option<bool>) -> bool {
        self.color_glyphs
            .iter()
            .any(|glyph| selected.is_none_or(|selected| selected == glyph.selected))
    }

    /// Draw retained color glyphs after the field's outline paints. `selected`
    /// is None for the editor's combined route and Some for authored drawables.
    pub(crate) fn draw_color_glyphs(
        &mut self,
        factory: &RuntimeFactoryHandle,
        renderer: &mut Renderer,
        world: Mat2D,
        selected: Option<bool>,
        foreground: u32,
        opacity: f32,
    ) {
        if self.color_glyphs.is_empty() {
            return;
        }
        for glyph in &self.color_glyphs {
            if selected.is_some_and(|selected| selected != glyph.selected) {
                continue;
            }
            draw_color_glyph(
                renderer,
                factory,
                &mut self.emoji_image_cache,
                glyph.font.clone(),
                glyph.glyph_id,
                &glyph.cluster_text,
                glyph.transform,
                foreground,
                opacity,
                world,
            );
        }
    }

    // Line height and the CSS line box never move a break, but they decide
    // how many lines a fixed height box keeps, so a resolved line index can
    // outlive its line.
    pub(crate) fn set_css_line_box(&mut self, enabled: bool) {
        if self.css_line_box != enabled {
            self.css_line_box = enabled;
            self.invalidate_shape();
        }
    }

    pub(crate) fn set_line_height(&mut self, height: f32) {
        if self.text_run.line_height != height {
            self.text_run.line_height = height;
            self.invalidate_shape();
        }
    }

    pub fn font_size(&self) -> f32 {
        self.text_run.size
    }

    pub fn set_font_size(&mut self, value: f32) {
        if self.text_run.size == value {
            return;
        }
        self.text_run.size = value;
        self.invalidate_shape();
    }

    pub fn max_width(&self) -> f32 {
        self.max_width
    }

    pub fn set_max_width(&mut self, value: f32) {
        if self.max_width == value {
            return;
        }
        self.max_width = value;
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn max_height(&self) -> f32 {
        self.max_height
    }

    pub fn set_max_height(&mut self, value: f32) {
        if self.max_height == value {
            return;
        }
        self.max_height = value;
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn sizing(&self) -> TextSizing {
        self.sizing
    }

    pub fn align(&self) -> TextAlign {
        self.align
    }
    pub fn align_width(&self) -> f32 {
        self.align_width
    }
    pub fn vertical_align(&self) -> VerticalTextAlign {
        self.vertical_align
    }
    pub fn align_height(&self) -> f32 {
        self.align_height
    }

    pub fn set_align(&mut self, value: TextAlign) {
        if self.align != value {
            self.align = value;
            self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
        }
    }
    pub fn set_align_width(&mut self, value: f32) {
        if self.align_width != value {
            self.align_width = value;
            self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
        }
    }
    pub fn set_vertical_align(&mut self, value: VerticalTextAlign) {
        if self.vertical_align != value {
            self.vertical_align = value;
            self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
        }
    }
    pub fn set_align_height(&mut self, value: f32) {
        if self.align_height != value {
            self.align_height = value;
            self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
        }
    }

    pub fn set_sizing(&mut self, value: TextSizing) {
        if self.sizing == value {
            return;
        }
        self.sizing = value;
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn overflow(&self) -> TextOverflow {
        self.overflow
    }

    pub fn set_overflow(&mut self, value: TextOverflow) {
        if self.overflow == value {
            return;
        }
        self.overflow = value;
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn font(&self) -> Option<FontRef> {
        self.text_run.font.clone()
    }

    pub fn set_font(&mut self, value: Option<FontRef>) {
        let unchanged = match (&self.text_run.font, &value) {
            (Some(left), Some(right)) => std::sync::Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        };
        if unchanged {
            return;
        }
        self.text_run.font = value;
        self.invalidate_shape();
    }

    /// Invalidate shaping, including fallbacks under the same font. Only CSS
    /// discards the positioned layout's vertical navigation target.
    pub fn invalidate_shape(&mut self) {
        if self.css_line_box {
            self.ideal_cursor_x = -1.0;
        }
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn paragraph_spacing(&self) -> f32 {
        self.paragraph_spacing
    }

    pub fn set_paragraph_spacing(&mut self, value: f32) {
        if self.paragraph_spacing == value {
            return;
        }
        self.paragraph_spacing = value;
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn selection_corner_radius(&self) -> f32 {
        self.selection_corner_radius
    }

    pub fn set_selection_corner_radius(&mut self, value: f32) {
        if self.selection_corner_radius == value {
            return;
        }
        self.selection_corner_radius = value;
        self.flag(Flags::SelectionDirty as u8);
    }

    pub fn separate_selection_text(&self) -> bool {
        self.flagged(Flags::SeparateSelectionText as u8)
    }

    pub fn set_separate_selection_text(&mut self, value: bool) {
        if value {
            self.flag(Flags::SeparateSelectionText as u8);
        } else {
            self.unflag(Flags::SeparateSelectionText as u8);
        }
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn text_path(&mut self) -> &mut ShapePaintPath {
        &mut self.text_path
    }

    pub fn selected_text_path(&mut self) -> &mut ShapePaintPath {
        &mut self.selected_text_path
    }

    pub fn cursor_path(&mut self) -> &mut ShapePaintPath {
        &mut self.cursor_path
    }

    pub fn selection_path(&mut self) -> &mut TextSelectionPath {
        &mut self.selection_path
    }

    pub fn clip_render_path(&self) -> Option<Rc<RefCell<Box<RenderPath>>>> {
        self.clip_render_path.clone()
    }

    pub fn bounds(&self) -> Aabb {
        self.shape.bounds()
    }

    pub fn cursor_visual_position_at(&self, position: CursorPosition) -> CursorVisualPosition {
        position.visual_position(&self.shape.layout_view())
    }

    pub fn cursor_visual_position(&self) -> CursorVisualPosition {
        if self.css_line_box {
            let mut end = self.cursor.end();
            end.resolve_line(&self.shape.layout_view());
            return end.visual_position(&self.shape.layout_view());
        }
        self.cursor_visual_position
    }

    /// CSS overflow reads the very layout used for drawing and hit testing.
    /// It has no measurement shape or independently cached extent.
    pub(crate) fn positioned_layout_bounds(&mut self) -> Aabb {
        if self.text_run.font.is_none() {
            return Aabb::default();
        }
        self.ensure_shape();
        let mut bounds = self.shape.bounds();
        let caret = self.cursor_visual_position();
        if caret.found() {
            bounds.expand(Aabb::new(
                caret.x(),
                caret.top(),
                caret.x() + Self::CARET_WIDTH,
                caret.bottom(),
            ));
        }
        bounds
    }

    pub fn insert_code_point(&mut self, code_point: Unichar) {
        let starting_cursor = self.cursor.clone();
        self.erase();
        assert!(self.cursor.is_collapsed());
        let index = self.cursor.start().code_point_index() as usize;
        self.text.insert(index, code_point);
        let position = CursorPosition::unresolved(self.cursor.first().code_point_index_offset(1));
        self.cursor = Cursor::collapsed(position);
        self.capture_journal_entry(starting_cursor);
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn insert(&mut self, value: &str) {
        let starting_cursor = self.cursor.clone();
        self.erase();
        let mut code_point_index = self.cursor.start().code_point_index();
        let mut bytes = value.as_bytes();
        while bytes.first().is_some_and(|byte| *byte != 0) {
            let code_point = Utf::next_utf8(&mut bytes);
            self.text.insert(code_point_index as usize, code_point);
            code_point_index += 1;
        }
        self.cursor = Cursor::collapsed(CursorPosition::unresolved(code_point_index));
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
        self.capture_journal_entry(starting_cursor);
    }

    pub fn erase(&mut self) {
        self.ideal_cursor_x = -1.0;
        if self.cursor.is_collapsed() {
            return;
        }
        assert!(self.cursor.first().code_point_index() < self.length() as u32);
        assert!(self.cursor.last().code_point_index() <= self.length() as u32);
        let start = self.cursor.first().code_point_index();
        let end = self.cursor.last().code_point_index();
        self.text.drain(start as usize..end as usize);
        self.cursor = Cursor::collapsed(CursorPosition::unresolved(start));
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn backspace(&mut self, direction: i32) {
        let starting_cursor = self.cursor.clone();
        if !self.cursor.is_collapsed() {
            self.erase();
            self.capture_journal_entry(starting_cursor);
            return;
        }
        self.ideal_cursor_x = -1.0;
        self.ensure_shape();
        let glyph_lookup = self.shape.glyph_lookup();
        if direction > 0 {
            let index = self.cursor.first().code_point_index();
            if index as usize >= self.text.len() - 1 {
                return;
            }
            let cluster_count = glyph_lookup.count(index);
            self.text
                .drain(index as usize..(index + cluster_count) as usize);
            self.cursor = Cursor::collapsed(CursorPosition::unresolved(index));
        } else {
            let index = self.cursor.first().code_point_index();
            if index == 0 {
                return;
            }
            let cluster_start = glyph_lookup.glyph_start(index - 1);
            let cluster_count = glyph_lookup.count(cluster_start);
            self.text
                .drain(cluster_start as usize..(cluster_start + cluster_count) as usize);
            self.cursor = Cursor::collapsed(CursorPosition::unresolved(cluster_start));
        }
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
        self.capture_journal_entry(starting_cursor);
    }

    pub fn update(&mut self, factory: &RuntimeFactoryHandle) -> u8 {
        let mut updated = Flags::None as u8;
        if self.text_run.font.is_none() {
            return updated;
        }
        let mut update_text_path = false;
        let rebuild_text_path = if self.css_line_box {
            self.ensure_shape();
            self.unflag(Flags::TextPathDirty as u8)
        } else {
            // Upstream only rebuilds paths when shape dirt survives until
            // update. Selection may already have consumed it in ensureShape.
            self.flagged(Flags::ShapeDirty as u8)
        };
        if rebuild_text_path {
            updated |= Flags::ShapeDirty as u8;
            self.ensure_shape();
            update_text_path = true;
        }
        if self.unflag(Flags::SelectionDirty as u8) {
            updated |= Flags::SelectionDirty as u8;
            if self.flagged(Flags::SeparateSelectionText as u8) {
                update_text_path = true;
            }
            self.cursor
                .resolve_line_positions(&self.shape.layout_view());
            self.compute_visual_position_from_cursor();
            self.selection_rects.clear();
            self.cursor
                .selection_rects(&mut self.selection_rects, &self.shape.layout_view());
            self.selection_path
                .update(&self.selection_rects, self.selection_corner_radius);
            self.cursor_path.rewind();
            let caret_width = Self::CARET_WIDTH;
            let caret = self.cursor_visual_position();
            let mut rectangle = RawPath::default();
            rectangle.move_to(caret.x(), caret.top());
            rectangle.line_to(caret.x() + caret_width, caret.top());
            rectangle.line_to(caret.x() + caret_width, caret.bottom());
            rectangle.line_to(caret.x(), caret.bottom());
            rectangle.close();
            self.cursor_path.add_path_clockwise(&rectangle, None);
        }
        if update_text_path {
            self.build_text_paths(factory);
        }
        updated
    }

    fn ensure_shape(&mut self) {
        if self.unflag(Flags::ShapeDirty as u8) {
            self.text_run.unichar_count = self.text.len() as u32;
            self.prepare_obscured_text();
            let text = if self.obscured() {
                &mut self.obscured_text
            } else {
                &mut self.text
            };
            self.shape.set_css_line_box(self.css_line_box);
            self.shape.shape_aligned(
                text,
                std::slice::from_mut(&mut self.text_run),
                self.sizing,
                self.max_width,
                self.max_height,
                self.align,
                self.wrap,
                self.origin,
                self.overflow,
                self.paragraph_spacing,
                self.align_width,
                self.vertical_align,
                self.align_height,
            );
            if self.css_line_box {
                // CSS selection may build the layout before update. Rendering
                // still rebuilds paths from that same positioned layout.
                self.flag(Flags::TextPathDirty | Flags::SelectionDirty);
                self.cursor
                    .resolve_line_positions(&self.shape.layout_view());
            }
        }
    }

    fn compute_visual_position_from_cursor(&mut self) {
        if !self.css_line_box {
            self.cursor_visual_position = self.cursor_visual_position_at(self.cursor.end());
        }
    }

    fn build_text_paths(&mut self, factory: &RuntimeFactoryHandle) {
        let want_separate = self.flagged(Flags::SeparateSelectionText as u8);
        self.text_path.rewind();
        self.selected_text_path.rewind();
        self.color_glyphs.clear();
        if !self.shape.has_valid_bounds() {
            self.emoji_image_cache.clear();
            self.clip_render_path = None;
            return;
        }

        if self.overflow == TextOverflow::Clipped {
            if self.clip_render_path.is_none() {
                self.clip_render_path =
                    Some(Rc::new(RefCell::new(factory.with_factory_mut(|factory| {
                        factory.make_empty_render_path()
                    }))));
            } else {
                self.clip_render_path
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .rewind();
            }
            let bounds = self.shape.bounds();
            let mut raw = nuxie_render_api::RawPath::new();
            raw.add_rect(nuxie_render_api::Aabb::new(
                bounds.left(),
                bounds.top(),
                bounds.right(),
                bounds.bottom(),
            ));
            self.clip_render_path
                .as_ref()
                .unwrap()
                .borrow_mut()
                .add_raw_path(raw.as_ref());
        } else {
            self.clip_render_path = None;
        }

        let mut y = self.shape.vertical_offset();
        let paragraph_lines = self.shape.paragraph_lines();
        let ordered_lines = self.shape.ordered_lines();
        if self.origin == TextOrigin::Baseline
            && !paragraph_lines.is_empty()
            && !paragraph_lines[0].is_empty()
        {
            y -= paragraph_lines[0][0].baseline;
        }
        let mut line_index = 0usize;
        for lines in paragraph_lines {
            for line in lines {
                if line_index >= ordered_lines.len() {
                    break;
                }
                let ordered_line = &ordered_lines[line_index];
                let mut x = line.start_x;
                let render_y = y + line.baseline;
                for (run, glyph_index) in ordered_line {
                    let glyph_index = glyph_index as usize;
                    let font = run.font.as_ref().unwrap();
                    let offset = run.offsets[glyph_index];
                    let glyph_id = run.glyphs[glyph_index];
                    let advance = run.advances[glyph_index];
                    let transform = Mat2D::new(
                        run.size,
                        0.0,
                        0.0,
                        run.size,
                        x + offset.x,
                        render_y + offset.y,
                    );
                    x += advance;
                    let selected =
                        want_separate && self.cursor.contains(run.text_indices[glyph_index]);
                    match run.glyph_render_kind(glyph_index) {
                        GlyphRenderKind::Color => {
                            self.color_glyphs.push(InputColorGlyph {
                                font: font.clone(),
                                glyph_id,
                                cluster_text: run.cluster_text_arc(glyph_index),
                                transform,
                                selected,
                            });
                            continue;
                        }
                        GlyphRenderKind::Suppressed => continue,
                        GlyphRenderKind::Outline => {}
                    }
                    let mut raw_path = font.get_path(glyph_id);
                    raw_path.transform_in_place(transform);
                    if want_separate && self.cursor.contains(run.text_indices[glyph_index]) {
                        self.selected_text_path.add_path_clockwise(&raw_path, None);
                    } else {
                        self.text_path.add_path_clockwise(&raw_path, None);
                    }
                }
                line_index += 1;
            }
            if !lines.is_empty() {
                y += lines.last().unwrap().bottom;
            }
            y += self.paragraph_spacing;
        }
        let active_color_keys: HashSet<_> = self
            .color_glyphs
            .iter()
            .map(|active| {
                color_glyph_cache_key(&active.font, active.glyph_id, &active.cluster_text)
            })
            .collect();
        self.emoji_image_cache.retain(|entry| {
            active_color_keys.contains(&color_glyph_cache_key(
                &entry.font,
                entry.glyph_id,
                &entry.cluster_text,
            ))
        });
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor.clone()
    }

    pub fn set_cursor(&mut self, value: Cursor) {
        if self.cursor == value {
            return;
        }
        self.cursor = value;
        self.flag(Flags::SelectionDirty as u8);
    }

    pub fn select_word(&mut self) {
        let mut search_position = self.cursor.start();
        let mut classification = self.classify_position(search_position);
        if (classification & Delineator::Word) == Delineator::Unknown {
            let previous_position = search_position - 1;
            let previous_classification = self.classify_position(previous_position);
            if (previous_classification & Delineator::Word) != Delineator::Unknown {
                search_position = previous_position;
                classification = previous_classification;
            }
        }
        if (classification & Delineator::Word) != Delineator::Unknown {
            classification = Delineator::Word;
        }
        let start = self.find_position(!classification, search_position, -1);
        let mut end = self.find_position(!classification, search_position, 1);
        end = end + 1;
        self.cursor = Cursor::new(start, end);
        self.flag(Flags::SelectionDirty as u8);
    }

    pub fn select_all(&mut self) {
        self.ensure_shape();
        self.ideal_cursor_x = -1.0;
        let start = CursorPosition::at_index(0, &self.shape.layout_view());
        let end = CursorPosition::at_index(self.length() as u32, &self.shape.layout_view());
        self.cursor = Cursor::new(start, end);
        self.flag(Flags::SelectionDirty as u8);
    }

    pub fn select_line(&mut self) {
        self.ensure_shape();
        self.ideal_cursor_x = -1.0;
        let mut cursor = self.cursor.start();
        cursor.resolve_line(&self.shape.layout_view());
        let Some(line) = self.ordered_line(cursor) else {
            return;
        };
        let glyph_lookup = self.shape.glyph_lookup();
        let start = CursorPosition::new(
            cursor.line_index(),
            line.first_code_point_index(glyph_lookup),
        );
        let end = CursorPosition::new(
            cursor.line_index(),
            line.last_code_point_index(glyph_lookup),
        );
        self.cursor = Cursor::new(start, end);
        self.flag(Flags::SelectionDirty as u8);
    }

    pub fn clear_selection(&mut self) {
        self.set_cursor(Cursor::collapsed(self.cursor.end()));
    }

    fn ordered_line(&self, position: CursorPosition) -> Option<&OrderedLine> {
        let ordered_lines = self.shape.ordered_lines();
        if position.line_index() as usize >= ordered_lines.len() {
            None
        } else {
            Some(&ordered_lines[position.line_index() as usize])
        }
    }

    pub fn cursor_left(&mut self, boundary: CursorBoundary, select: bool) {
        self.cursor_horizontal(-1, boundary, select);
    }

    pub fn cursor_right(&mut self, boundary: CursorBoundary, select: bool) {
        self.cursor_horizontal(1, boundary, select);
    }

    fn cursor_horizontal(&mut self, offset: i32, boundary: CursorBoundary, select: bool) {
        self.ensure_shape();
        self.ideal_cursor_x = -1.0;
        // A non-selecting leftward move starts at the selection's leading edge.
        let base_position = if !select && offset < 0 && self.cursor.has_selection() {
            self.cursor.first()
        } else {
            self.cursor.end()
        };
        let mut position = base_position;
        match boundary {
            CursorBoundary::Character => {
                let glyph_lookup = self.shape.glyph_lookup();
                let mut next_index = base_position.code_point_index_offset(offset);
                if offset > 0 {
                    while (next_index as usize) < self.text.len() - 1
                        && !glyph_lookup.is_glyph_boundary(next_index)
                    {
                        next_index += 1;
                    }
                } else {
                    while next_index > 0 && !glyph_lookup.is_glyph_boundary(next_index) {
                        next_index -= 1;
                    }
                }
                position = CursorPosition::at_index(next_index, &self.shape.layout_view());
            }
            CursorBoundary::Line => {
                if let Some(line) = self.ordered_line(base_position) {
                    let code_point_index = if offset < 0 {
                        line.first_code_point_index(self.shape.glyph_lookup())
                    } else {
                        line.last_code_point_index(self.shape.glyph_lookup())
                    };
                    position = CursorPosition::new(base_position.line_index(), code_point_index);
                }
            }
            CursorBoundary::Word | CursorBoundary::SubWord => {
                let mut classification =
                    self.classify_position(position + if offset < 0 { -1 } else { 0 });
                if classification == Delineator::Whitespace
                    || classification == Delineator::Underscore
                {
                    classification = self.find(!classification, &mut position, offset);
                }
                match classification {
                    Delineator::Symbol => {
                        self.find(!classification, &mut position, offset);
                    }
                    Delineator::Lowercase => {
                        if boundary == CursorBoundary::SubWord {
                            let non_lowercase =
                                self.find(!Delineator::Lowercase, &mut position, offset);
                            if offset == -1 && non_lowercase == Delineator::Uppercase {
                                position = position - 1;
                            }
                        } else {
                            self.find(
                                !(Delineator::Lowercase
                                    | Delineator::Uppercase
                                    | Delineator::Underscore),
                                &mut position,
                                offset,
                            );
                        }
                    }
                    Delineator::Uppercase => {
                        if boundary == CursorBoundary::SubWord {
                            let start_position = position;
                            let non_uppercase =
                                self.find(!Delineator::Uppercase, &mut position, offset);
                            if offset == 1 && non_uppercase == Delineator::Lowercase {
                                position = position - 1;
                                if position.code_point_index() == start_position.code_point_index()
                                {
                                    self.find(!Delineator::Lowercase, &mut position, offset);
                                }
                            }
                        } else {
                            self.find(
                                !(Delineator::Lowercase
                                    | Delineator::Uppercase
                                    | Delineator::Underscore),
                                &mut position,
                                offset,
                            );
                        }
                    }
                    _ => {
                        self.find(!classification, &mut position, offset);
                    }
                }
            }
        }
        self.cursor = if select {
            Cursor::new(self.cursor.start(), position)
        } else {
            Cursor::collapsed(position)
        };
        self.flag(Flags::SelectionDirty as u8);
    }

    fn classify(code_point: Unichar) -> Delineator {
        if is_white_space(code_point) {
            return Delineator::Whitespace;
        }
        if code_point == 95 {
            return Delineator::Underscore;
        }
        if code_point < 48
            || (58..=64).contains(&code_point)
            || (91..=96).contains(&code_point)
            || (123..=127).contains(&code_point)
        {
            return Delineator::Symbol;
        }
        if (65..=90).contains(&code_point) {
            return Delineator::Uppercase;
        }
        Delineator::Lowercase
    }

    fn classify_position(&self, position: CursorPosition) -> Delineator {
        if self.empty() || position.code_point_index() as usize >= self.text.len() - 1 {
            Delineator::Whitespace
        } else {
            Self::classify(if self.obscured() {
                OBSCURING_BULLET
            } else {
                self.text[position.code_point_index() as usize]
            })
        }
    }

    fn find(
        &self,
        delineator_mask: u8,
        position: &mut CursorPosition,
        direction: i32,
    ) -> Delineator {
        let mut last_classification = Delineator::Unknown;
        loop {
            let next_position = *position + direction;
            if next_position.code_point_index() == position.code_point_index() {
                break;
            }
            *position = next_position;
            last_classification =
                self.classify_position(next_position + if direction < 0 { -1 } else { 0 });
            if last_classification & delineator_mask != 0 {
                break;
            }
        }
        last_classification
    }

    fn find_position(
        &self,
        delineator_mask: u8,
        position: CursorPosition,
        direction: i32,
    ) -> CursorPosition {
        let mut result = position;
        loop {
            let next_position = result + direction;
            if next_position.code_point_index() == result.code_point_index()
                || next_position.code_point_index() as usize >= self.length()
            {
                break;
            }
            if self.classify_position(next_position) & delineator_mask != 0 {
                break;
            }
            result = next_position;
        }
        result
    }

    pub fn cursor_up(&mut self, select: bool) {
        self.ensure_shape();
        if self.ideal_cursor_x == -1.0 {
            self.ideal_cursor_x = self.cursor_visual_position().x();
        }
        let line_index = self.cursor.end().line_index();
        let position = if line_index == 0 {
            CursorPosition::zero()
        } else {
            CursorPosition::from_line_x(
                self.cursor.end().line_index_offset(-1),
                self.ideal_cursor_x,
                &self.shape.layout_view(),
            )
        };
        self.cursor = if select {
            Cursor::new(self.cursor.start(), position)
        } else {
            Cursor::collapsed(position)
        };
        self.flag(Flags::SelectionDirty as u8);
    }

    pub fn cursor_down(&mut self, select: bool) {
        self.ensure_shape();
        if self.ideal_cursor_x == -1.0 {
            self.ideal_cursor_x = self.cursor_visual_position().x();
        }
        let next_line_index = self.cursor.end().line_index_offset(1);
        let position = if self.shape.line_count() != 0
            && self.text.len() > 1
            && next_line_index >= self.shape.line_count()
        {
            CursorPosition::new(
                self.shape.line_count() as u32 - 1,
                self.text.len() as u32 - 1,
            )
        } else {
            CursorPosition::from_line_x(
                next_line_index,
                self.ideal_cursor_x,
                &self.shape.layout_view(),
            )
        };
        self.cursor = if select {
            Cursor::new(self.cursor.start(), position)
        } else {
            Cursor::collapsed(position)
        };
        self.flag(Flags::SelectionDirty as u8);
    }

    pub fn move_cursor_to(&mut self, translation: Vec2D, select: bool) {
        self.ensure_shape();
        self.ideal_cursor_x = -1.0;
        let position = CursorPosition::from_translation(translation, &self.shape.layout_view());
        self.cursor = if select {
            Cursor::new(self.cursor.start(), position)
        } else {
            Cursor::collapsed(position)
        };
        self.flag(Flags::SelectionDirty as u8);
    }

    pub fn shape(&self) -> &FullyShapedText {
        &self.shape
    }

    pub fn text(&self) -> String {
        let size = self.text.len();
        if size == 0 {
            return String::new();
        }
        encode_code_points(&self.text[..size - 1])
    }

    pub fn selected_text(&self) -> String {
        if self.text.is_empty() {
            return String::new();
        }
        // The trailing zero-width-space sentinel is never selectable.
        let text_end = self.text.len() - 1;
        let first = (self.cursor.first().code_point_index() as usize).min(text_end);
        let last = (self.cursor.last().code_point_index() as usize).min(text_end);
        if first >= last {
            return String::new();
        }
        encode_code_points(&self.text[first..last])
    }

    fn set_text_private(&mut self, value: String) {
        let mut bytes = value.as_bytes();
        self.text.clear();
        while bytes.first().is_some_and(|byte| *byte != 0) {
            self.text.push(Utf::next_utf8(&mut bytes));
        }
        self.text.push(ZERO_WIDTH_SPACE);
    }

    pub fn set_text(&mut self, value: String) {
        self.set_text_private(value);
        self.cursor = Cursor::collapsed(CursorPosition::zero());
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
        self.journal.clear();
        self.journal.push(JournalEntry {
            cursor_from: self.cursor.clone(),
            cursor_to: self.cursor.clone(),
            text: self.text(),
        });
        self.journal_index = 0;
    }

    pub fn set_text_preserve_cursor(&mut self, value: String) {
        let starting_cursor = self.cursor.clone();
        self.ideal_cursor_x = -1.0;
        self.set_text_private(value);
        let max_index = self.length() as u32;
        let start =
            CursorPosition::unresolved(starting_cursor.start().code_point_index().min(max_index));
        let end =
            CursorPosition::unresolved(starting_cursor.end().code_point_index().min(max_index));
        self.cursor = Cursor::new(start, end);
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
        self.capture_journal_entry(starting_cursor);
    }

    pub fn length(&self) -> usize {
        if self.empty() { 0 } else { self.text.len() - 1 }
    }

    pub fn empty(&self) -> bool {
        self.text.len() <= 1
    }

    fn capture_journal_entry(&mut self, cursor: Cursor) {
        if self.journal_index as usize + 1 < self.journal.len() {
            self.journal.truncate(self.journal_index as usize + 1);
        }
        self.journal.push(JournalEntry {
            cursor_from: cursor,
            cursor_to: self.cursor.clone(),
            text: self.text(),
        });
        self.journal_index = self.journal.len() as u32 - 1;
    }

    pub fn undo(&mut self) {
        if self.journal_index == 0 {
            return;
        }
        let entry_from = self.journal[self.journal_index as usize].clone();
        let entry_to = self.journal[self.journal_index as usize - 1].clone();
        self.set_text_private(entry_to.text);
        self.cursor = entry_from.cursor_from;
        self.journal_index -= 1;
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn redo(&mut self) {
        if self.journal.is_empty() || self.journal_index as usize + 1 >= self.journal.len() {
            return;
        }
        let entry_to = self.journal[self.journal_index as usize + 1].clone();
        self.set_text_private(entry_to.text);
        self.cursor = entry_to.cursor_to;
        self.journal_index += 1;
        self.flag(Flags::ShapeDirty | Flags::MeasureDirty | Flags::SelectionDirty);
    }

    pub fn obscured(&self) -> bool {
        self.flagged(Flags::Obscured as u8)
    }

    pub fn set_obscured(&mut self, value: bool) {
        if value == self.obscured() {
            return;
        }
        if value {
            self.flag(Flags::Obscured as u8);
        } else {
            self.unflag(Flags::Obscured as u8);
        }
        // Upstream invalidates resolved lines and ideal x for masking only.
        self.forget_line_indices();
        self.ideal_cursor_x = -1.0;
        self.invalidate_shape();
    }

    // From src/text/raw_text_input.cpp::obscured. A reflow leaves resolved
    // line indices stale, and update never resolves a line index twice, so
    // keep only the code point indices. The journal holds cursor snapshots
    // too, or undo would restore one resolved against the old layout.
    // Upstream does this for masking only. CSS invalidations discard the
    // positioned layout's resolved cursor lines together.
    fn forget_line_indices(&mut self) {
        let unresolved = |cursor: Cursor| {
            Cursor::new(
                CursorPosition::unresolved(cursor.start().code_point_index()),
                CursorPosition::unresolved(cursor.end().code_point_index()),
            )
        };
        self.cursor = unresolved(self.cursor);
        for entry in &mut self.journal {
            entry.cursor_from = unresolved(entry.cursor_from);
            entry.cursor_to = unresolved(entry.cursor_to);
        }
    }

    fn prepare_obscured_text(&mut self) {
        if self.obscured() {
            self.obscured_text.clear();
            self.obscured_text.resize(self.text.len(), OBSCURING_BULLET);
            *self.obscured_text.last_mut().expect("text sentinel") = ZERO_WIDTH_SPACE;
        }
    }

    pub fn measure(&mut self, max_width: f32, max_height: f32) -> Aabb {
        if self.text_run.font.is_none() {
            return Aabb::default();
        }
        let mut force =
            self.last_measure_max_width != max_width || self.last_measure_max_height != max_height;
        if self.measuring_shape.is_none() {
            force = true;
            self.measuring_shape = Some(Box::new(FullyShapedText::default()));
        }
        if self.unflag(Flags::MeasureDirty as u8) || force {
            self.text_run.unichar_count = self.text.len() as u32;
            self.prepare_obscured_text();
            let text = if self.obscured() {
                &mut self.obscured_text
            } else {
                &mut self.text
            };
            self.measuring_shape
                .as_mut()
                .unwrap()
                .set_css_line_box(self.css_line_box);
            self.measuring_shape.as_mut().unwrap().shape_aligned(
                text,
                std::slice::from_mut(&mut self.text_run),
                self.sizing,
                max_width,
                max_height,
                self.align,
                self.wrap,
                self.origin,
                self.overflow,
                self.paragraph_spacing,
                self.align_width,
                self.vertical_align,
                self.align_height,
            );
            self.last_measure_max_width = max_width;
            self.last_measure_max_height = max_height;
            #[cfg(any(test, feature = "tools"))]
            {
                self.measure_count += 1;
            }
        }
        self.measuring_shape.as_ref().unwrap().bounds()
    }

    fn flagged(&self, mask: u8) -> bool {
        self.flags & mask != 0
    }

    fn unflag(&mut self, mask: u8) -> bool {
        if self.flags & mask != 0 {
            self.flags &= !mask;
            true
        } else {
            false
        }
    }

    fn flag(&mut self, mask: u8) {
        if self.css_line_box && mask & Flags::ShapeDirty as u8 != 0 {
            self.forget_line_indices();
        }
        self.flags |= mask;
    }
}
