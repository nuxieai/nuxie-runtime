//! A top-level frame placed by a CSS slot (docs/CSS_LAYOUT.md, 2.1 and 4.4).
//!
//! A CSS slot is a `NestedArtboardLayout` with its own `LayoutCssStyle`. It
//! gives the frame it mounts a hosted item record that is not the frame's own
//! record, which makes the frame a CSS box of the slot's flow. Only such a
//! frame:
//! - draws its own x and y as a CSS translate, outside its turn and scale;
//! - gives its slot a stacking context through its effects.
//!
//! Every other top-level frame keeps upstream's rules. A list row, a Screen
//! included, hosts its own record; a native host or a plain slot hosts none.
use crate::source::{
    artboard::{Artboard, RuntimeArtboardDirtyHandle},
    core::{CoreHandle, CoreObject},
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

/// Whether a frame whose hosted item record is `hosted`, and whose own CSS
/// record is `own`, is placed by a CSS slot.
pub(crate) fn placed_by_css_slot(hosted: Option<&CoreHandle>, own: Option<&CoreHandle>) -> bool {
    hosted.is_some_and(|hosted| own != Some(hosted))
}

/// Whether [`self_transform`] differs from identity.
pub(crate) fn has_self_transform(
    offset: Option<Vec2D>,
    rotation: f32,
    scale: Vec2D,
    stretch: Vec2D,
) -> bool {
    rotation != 0.0
        || scale.x != 1.0
        || scale.y != 1.0
        || stretch.x != 1.0
        || stretch.y != 1.0
        || offset.is_some_and(|offset| offset.x != 0.0 || offset.y != 0.0)
}

/// A frame's own transform. A frame placed by a CSS slot draws CSS's
/// `translate(x, y) rotate scale`, as an inline box does; `offset` is its x
/// and y. Any other frame draws upstream's `rotate scale`, since load zeroes
/// an artboard's x and y. A flight's stretch is the innermost factor.
///
/// A frame whose origin is a pivot only (docs/CSS_LAYOUT.md, 2.1) turns and
/// scales about its `pivot`, with its content in its top-left frame:
/// `translate(pivot) * rotate * scale * translate(-pivot)`, as
/// `LayoutComponent` turns about its origin, skipped when nothing turns or
/// scales. The stretch scales the pivot with the frame. Without a pivot it
/// turns about its local zero, which is upstream's origin.
pub(crate) fn self_transform(
    offset: Option<Vec2D>,
    rotation: f32,
    scale: Vec2D,
    stretch: Vec2D,
    pivot: Option<Vec2D>,
) -> Mat2D {
    let mut local = Mat2D::from_rotation(rotation);
    local.scale_by_values(scale.x, scale.y);
    if let Some(pivot) = pivot
        && (rotation != 0.0 || scale.x != 1.0 || scale.y != 1.0)
    {
        let pivot = Vec2D::new(pivot.x * stretch.x, pivot.y * stretch.y);
        if pivot.x != 0.0 || pivot.y != 0.0 {
            local = Mat2D::from_translate(pivot.x, pivot.y)
                * local
                * Mat2D::from_translate(-pivot.x, -pivot.y);
        }
    }
    let mut transform = match offset {
        Some(offset) => Mat2D::from_translate(offset.x, offset.y) * local,
        None => local,
    };
    transform.scale_by_values(stretch.x, stretch.y);
    transform
}

/// Queues a CSS paint-order sort of the Screen that hosts this frame: the
/// parent artboard of its host attachment, never its own artboard. It reads
/// cells only, so it borrows neither the host nor the Screen.
pub(crate) fn mark_host_screen(frame: &RuntimeArtboardDirtyHandle) {
    if let Some(screen) = frame
        .host_parent_artboard()
        .and_then(|screen| screen.artboard_dirty_handle())
    {
        screen.mark_css_draw_order_dirty();
    }
}

/// Whether a slot with a CSS item record is a stacking context: its own
/// opacity is below 1 (CSS counts an element's own opacity, not what it
/// inherits), or the frame it places has a CSS effect.
pub(crate) fn slot_makes_context(slot: &dyn CoreObject) -> bool {
    let Some(slot) = slot.as_nested_artboard() else {
        return false;
    };
    slot.opacity() < 1.0
        || slot.source_artboard().is_some_and(|frame| {
            frame
                .with_downcast::<Artboard, _>(Artboard::css_slot_frame_makes_context)
                .unwrap_or(false)
        })
}

/// A slot's own opacity crossed 1. A slot with a CSS item record re-sorts its
/// Screen; any other slot keeps upstream's order.
pub(crate) fn slot_fade_changed(slot: &CoreHandle) {
    let screen = slot
        .with(|object| {
            let record = object
                .as_container_component()
                .and_then(|container| super::record::find_in(container.children()));
            record.and_then(|_| object.as_nested_artboard()?.parent_artboard_handle())
        })
        .flatten();
    if let Some(screen) = screen.and_then(|screen| screen.artboard_dirty_handle()) {
        screen.mark_css_draw_order_dirty();
    }
}
