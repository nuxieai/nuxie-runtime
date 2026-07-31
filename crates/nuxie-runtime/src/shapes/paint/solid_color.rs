//! SolidColor initializes and updates its owning RenderPaint immediately from
//! the generated color callback, preserving import-order paint allocation.

pub(crate) fn authored_color_is_visible(color: u32) -> bool {
    color >> 24 != 0
}
