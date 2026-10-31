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
//!
//! Both things are read through one reader, the frame at rest in a slot
//! ([`frame_at_rest`]). It takes each value from whoever sets it: what the
//! host sets from the slot, what the element sets live from the frame. While
//! a live move's flight carries an anchor's element, the anchor's frame at
//! rest is the frame in that flight (docs/CSS_LAYOUT.md, 4.4).
use crate::source::{
    artboard::{Artboard, RuntimeArtboardDirtyHandle, RuntimeArtboardInstanceHandle},
    core::{CoreHandle, CoreObject},
    math::{mat2d::Mat2D, vec2d::Vec2D},
    nested_artboard::NestedArtboard,
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

/// The frame at rest in a slot. Each value comes from whoever sets it.
pub(crate) struct FrameAtRest {
    /// The frame whose own x, y, turn, scale, opacity and transform marks
    /// are read live: the slot's own, or for an empty anchor the frame in the
    /// flight slot carrying its element.
    pub frame: Option<CoreHandle>,
    /// The host's layout instance, which holds the layout box, origin and
    /// pivot-only flag it places its frame by: the frame when the slot is
    /// filled, the placeholder `remember` kept when it is empty.
    pub layout: Option<RuntimeArtboardInstanceHandle>,
    /// The CSS-slot gate hosting set: the frame's own when the slot is
    /// filled, the one `remember` kept when it is empty.
    pub gate: bool,
}

/// The frame at rest in `slot`. An empty slot reads the frame in the flight
/// slot its link names while that flight slot's placement names it back;
/// any other empty slot reads none.
pub(crate) fn frame_at_rest(slot: &NestedArtboard) -> FrameAtRest {
    let layout = slot.live_move_layout_instance();
    if let Some(frame) = slot.source_artboard() {
        let gate = frame
            .with_downcast::<Artboard, _>(|frame| frame.base.base.is_css_slot_frame())
            .unwrap_or(false);
        return FrameAtRest {
            frame: Some(frame),
            layout,
            gate,
        };
    }
    let this = slot.base.handle();
    let frame = slot.live_move_carried_by().and_then(|flight| {
        flight
            .with(|o| {
                let flight = o.as_nested_artboard()?;
                let (anchor, _) = flight.live_move_paint()?;
                (Some(anchor) == this)
                    .then(|| flight.source_artboard())
                    .flatten()
            })
            .flatten()
    });
    FrameAtRest {
        frame,
        layout,
        gate: slot.live_move_gate(),
    }
}

/// The transform the frame at rest draws inside its slot, at rest: its x
/// and y when the gate is set, its own turn and scale, no stretch, about the
/// pivot its slot places it by. A transform a timeline keys is motion, taken
/// at its rest, identity.
pub(crate) fn rest_transform(rest: &FrameAtRest) -> Mat2D {
    let Some(frame) = &rest.frame else {
        return Mat2D::identity();
    };
    let pivot = rest.layout.as_ref().and_then(|layout| {
        layout.with_artboard(|b| b.origin_is_pivot_only().then(|| b.origin_offset()))
    });
    frame
        .with_downcast::<Artboard, _>(|frame| {
            if frame.base.base.transform_keyed() {
                return Mat2D::identity();
            }
            self_transform(
                rest.gate.then(|| Vec2D::new(frame.x(), frame.y())),
                frame.rotation(),
                Vec2D::new(frame.scale_x(), frame.scale_y()),
                Vec2D::new(1.0, 1.0),
                pivot,
            )
        })
        .unwrap_or_else(Mat2D::identity)
}

/// Whether a slot with a CSS item record is a stacking context: its own
/// opacity is below 1 (CSS counts an element's own opacity, not what it
/// inherits), or the frame at rest in it, placed by a CSS slot, has a CSS
/// effect.
pub(crate) fn slot_makes_context(slot: &dyn CoreObject) -> bool {
    let Some(slot) = slot.as_nested_artboard() else {
        return false;
    };
    if slot.opacity() < 1.0 {
        return true;
    }
    let rest = frame_at_rest(slot);
    rest.gate
        && rest.frame.is_some_and(|frame| {
            frame
                .with_downcast::<Artboard, _>(|frame| frame.base.base.css_effect_stacking_context())
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
