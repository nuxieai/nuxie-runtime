//! ShapePaint owns one RenderPaint, its mutator, optional Feather, and one
//! EffectPath per `(StrokeEffect, PathProvider)` occurrence. The renderer-facing
//! sidecars are retained in `RuntimeShapePaintOwner` and are dirtied only by
//! these owner-local callbacks.

pub(crate) fn blend_mode(paint_value: u32, parent_value: u32) -> u32 {
    if paint_value == 127 {
        parent_value
    } else {
        paint_value
    }
}
