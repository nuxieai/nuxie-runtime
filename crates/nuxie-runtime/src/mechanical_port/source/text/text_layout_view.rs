//! Borrowed selection layout, corresponding to `text_layout_view.hpp`.
use super::glyph_lookup::GlyphLookup;
use crate::mechanical_port::source::text_engine::{GlyphLine, OrderedLine, Paragraph};

/// The owner must keep the shaped layout alive and unchanged during a cursor
/// operation. Inputs exclude their trailing sentinel from `text_length`, while
/// regular Text can select through the end of its actual source.
#[derive(Clone, Copy)]
pub struct TextLayoutView<'a> {
    paragraphs: &'a [Paragraph],
    paragraph_lines: &'a [Vec<GlyphLine>],
    ordered_lines: &'a [OrderedLine],
    glyph_lookup: &'a GlyphLookup,
    text_length: u32,
}

impl<'a> TextLayoutView<'a> {
    pub fn new(
        paragraphs: &'a [Paragraph],
        paragraph_lines: &'a [Vec<GlyphLine>],
        ordered_lines: &'a [OrderedLine],
        glyph_lookup: &'a GlyphLookup,
        text_length: u32,
    ) -> Self {
        Self {
            paragraphs,
            paragraph_lines,
            ordered_lines,
            glyph_lookup,
            text_length,
        }
    }
    pub fn paragraphs(&self) -> &'a [Paragraph] {
        self.paragraphs
    }
    pub fn paragraph_lines(&self) -> &'a [Vec<GlyphLine>] {
        self.paragraph_lines
    }
    pub fn ordered_lines(&self) -> &'a [OrderedLine] {
        self.ordered_lines
    }
    pub fn glyph_lookup(&self) -> &'a GlyphLookup {
        self.glyph_lookup
    }
    pub fn text_length(&self) -> u32 {
        self.text_length
    }
}
