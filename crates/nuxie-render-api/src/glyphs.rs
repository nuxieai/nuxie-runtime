//! Optional platform glyph rendering data. This module contains no font
//! rasterizer, layout engine, compiler, or platform dependency.
use crate::{BlendMode, ColorInt};
use std::sync::Arc;

/// One variation coordinate in the font's design space, identified by its
/// OpenType four-byte tag. Unsupported instances must be declined intact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphFontVariation {
    pub tag: u32,
    pub value: f32,
}

/// Borrowed immutable font source. An adapter may clone `bytes` to retain it;
/// it must not retain a borrowed pointer after the draw call. Face index and
/// variation coordinates are part of the font instance and its cache identity.
#[derive(Clone, Copy, Debug)]
pub struct GlyphFontRef<'a> {
    pub bytes: &'a Arc<[u8]>,
    pub face_index: u32,
    pub variations: &'a [GlyphFontVariation],
}

/// A shaped glyph at its final baseline origin in the renderer's current local
/// coordinate system. These are glyph IDs, not Unicode values. Layout, bidi,
/// shaping, kerning and line wrapping have already been performed by the caller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionedGlyph {
    pub id: u16,
    pub x: f32,
    pub y: f32,
}

/// One solid-color font run. The current renderer transform and clip apply.
/// Adapters must validate finite coordinates/size and their resource limits
/// before allocation or platform calls. Unsupported font instances, effects,
/// scales or resource requirements must be declined without drawing anything.
/// Color includes straight alpha; adapters must not apply opacity twice.
#[derive(Clone, Copy, Debug)]
pub struct RenderGlyphRun<'a> {
    pub font: GlyphFontRef<'a>,
    pub glyphs: &'a [PositionedGlyph],
    pub font_size: f32,
    pub color: ColorInt,
    pub blend_mode: BlendMode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RecordingFactory, Renderer};

    #[test]
    fn existing_renderer_declines_glyphs_without_changing_the_draw_stream() {
        let factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let before = factory.stream();
        // The default path must not parse or retain font bytes; the caller
        // still owns the vector fallback for this same shaped run.
        let bytes: Arc<[u8]> = Arc::from([0_u8]);
        let glyphs = [PositionedGlyph {
            id: 161,
            x: 0.25,
            y: 22.0,
        }];
        let run = RenderGlyphRun {
            font: GlyphFontRef {
                bytes: &bytes,
                face_index: 0,
                variations: &[],
            },
            glyphs: &glyphs,
            font_size: 20.0,
            color: 0x80336699,
            blend_mode: BlendMode::SrcOver,
        };
        assert!(!renderer.supports_glyph_runs());
        assert!(!renderer.draw_glyph_run(&run));
        assert_eq!(factory.stream(), before);
        assert_eq!(Arc::strong_count(&bytes), 1);
    }
}
