//! The layout hold of an element flying in place (docs/live-moves.md, Flying
//! in place).
//!
//! While a matched element flies in place in Layout mode, its parent keeps
//! seeing the element as it rests, so its neighbors don't move, and its
//! inside is laid out at the flight size each frame. The element stays its
//! parent's own Taffy child with its own style: the parent pass lays it out
//! exactly as at rest. A flight pass then lays the element out again, alone,
//! as its parent last asked but at the flight size
//! (`TaffyTree::relayout_at_size`, vendor/taffy-0.15.0-nuxie-css/NUXIE_PATCH.md).
//! That pass writes only the element's descendants; the element's own Taffy
//! layout stays the place its parent gave it, its resting slot.
//!
//! Two readers turn that into what is published, and the layout checker
//! reads the same two: `resting_slot` (the element's place and size in its
//! parent) and `flight_layout` (that place at the flight's size and
//! content). No Rive object and no Taffy node is added, and no style is
//! changed: Taffy is driven differently only in the flight pass of a held
//! element.
//!
//! The element is a layout node's owner: a box, or a participant (a text,
//! an image or a video). Its rest follows its parent through the step: a
//! percentage or stretched size follows a parent resize, and its neighbors
//! move as they would at rest when its content changes. While held, and in
//! the frame the hold ends, the element's own box does not interpolate: the
//! flight drives it.
use crate::mechanical_port::source::{
    core::CoreHandle,
    layout_component::{Layout, LayoutComponent},
};
use taffy::prelude::{NodeId, Size, Style, TaffyTree};
use taffy::style::BoxSizing;
use taffy::tree::{LayoutInput, LayoutOutput};

#[derive(Clone, Debug)]
pub(crate) struct FlightHold {
    /// The element's CSS width and height while held, under its own
    /// box-sizing, as the slot path sizes a unit's root.
    size: (f32, f32),
    /// The element's resting slot, relative to its layout parent, as the
    /// last solve published it: its own published layout, were it at rest.
    pub(crate) rest: Layout,
}

impl FlightHold {
    /// The flight size: CSS width and height under the element's box-sizing.
    pub(crate) fn size(&self) -> (f32, f32) {
        self.size
    }
}

/// The element's resting slot: its place and size among its parent's
/// children, as the parent pass laid it out. The flight pass leaves it as
/// it is.
pub(crate) fn resting_slot<T>(taffy: &TaffyTree<T>, node: NodeId) -> Option<taffy::Layout> {
    taffy.layout(node).ok().copied()
}

/// The element's flight layout: its resting slot's place, at the size and
/// with the content the flight pass gave it.
pub(crate) fn flight_layout(slot: &taffy::Layout, flight: &LayoutOutput) -> taffy::Layout {
    taffy::Layout {
        size: flight.size,
        scrollable_overflow_rect: flight.scrollable_overflow_rect,
        ..*slot
    }
}

/// The flight pass of one held element: lay its inside out at `size` (CSS
/// width and height under its box-sizing), as its parent last asked. None
/// before the element's first full layout.
pub(crate) fn fly<T, M>(
    taffy: &mut TaffyTree<T>,
    node: NodeId,
    (width, height): (f32, f32),
    measure: M,
) -> Option<LayoutOutput>
where
    M: FnMut(LayoutInput, NodeId, Option<&mut T>, &Style) -> LayoutOutput,
{
    let slot = resting_slot(taffy, node)?;
    let border_box = match taffy.style(node).ok()?.box_sizing {
        BoxSizing::BorderBox => Size { width, height },
        // Padding and border resolve against the parent, which the flight
        // pass asks as the parent pass did: the slot's are the flight's.
        BoxSizing::ContentBox => {
            let inset = slot.padding + slot.border;
            Size {
                width: width + inset.left + inset.right,
                height: height + inset.top + inset.bottom,
            }
        }
    };
    taffy.relayout_at_size(node, border_box, measure)
}

/// Starts holding `element`'s place in its parent's layout; from the next
/// solve its inside is laid out at `size`, its CSS width and height under
/// its own box-sizing. False, and nothing held, for an element that is not
/// laid out by a parent, has not been laid out, or is already held.
pub(crate) fn begin(element: &CoreHandle, size: (f32, f32)) -> bool {
    let begun = element
        .with_mut(|object| {
            LayoutComponent::layout_parent_of(object).filter(|parent| parent != element)?;
            let data = LayoutComponent::layout_data_of(object)?;
            if data.flight_hold.is_some() || data.node().is_none() {
                return None;
            }
            let rest = data.solved_layout;
            if !(rest.width().is_finite() && rest.height().is_finite()) {
                return None;
            }
            data.flight_hold = Some(Box::new(FlightHold { size, rest }));
            data.request_solve();
            Some(())
        })
        .flatten()
        .is_some();
    if !begun {
        return false;
    }
    #[cfg(any(test, feature = "test-support"))]
    HOLDS_BEGUN.with(|count| count.set(count.get() + 1));
    LayoutComponent::request_layout_solve(element);
    true
}

/// Lays the held element's inside out at `size` from the next solve. The
/// parent's layout is not invalidated: it doesn't read the flight.
pub(crate) fn set_size(element: &CoreHandle, size: (f32, f32)) {
    let changed = element
        .with_mut(|object| {
            let data = LayoutComponent::layout_data_of(object)?;
            let hold = data.flight_hold.as_mut()?;
            if hold.size == size {
                return None;
            }
            hold.size = size;
            data.request_solve();
            Some(())
        })
        .flatten()
        .is_some();
    if changed {
        LayoutComponent::request_layout_solve(element);
    }
}

/// Ends the hold: the next solve lays the element's inside out at rest
/// again, and its own box takes its resting layout at once.
pub(crate) fn end(element: &CoreHandle) {
    let ended = element
        .with_mut(|object| {
            let data = LayoutComponent::layout_data_of(object)?;
            data.flight_hold.take()?;
            data.snap_bounds_once();
            data.mark_dirty();
            Some(())
        })
        .flatten()
        .is_some();
    if ended {
        LayoutComponent::request_layout_solve(element);
    }
}

/// Where `element` rests, relative to its layout parent, as layout last
/// solved it: while held, its resting slot; otherwise its own layout. The
/// one reader of a resting box that a flight's size, its placement and a
/// zoom's ends read.
pub(crate) fn resting_box(element: &CoreHandle) -> Option<Layout> {
    element
        .with_mut(|object| {
            let data = LayoutComponent::layout_data_of(object)?;
            Some(
                data.flight_hold
                    .as_ref()
                    .map_or(data.solved_layout, |hold| hold.rest),
            )
        })
        .flatten()
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    static HOLDS_BEGUN: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The holds begun on this thread.
#[cfg(any(test, feature = "test-support"))]
pub fn holds_begun() -> usize {
    HOLDS_BEGUN.with(std::cell::Cell::get)
}
