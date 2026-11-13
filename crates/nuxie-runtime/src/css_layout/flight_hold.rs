//! The layout hold of an element flying in place (docs/live-moves.md, Flying
//! in place).
//!
//! While a matched element flies in place in Layout mode, its parent keeps
//! seeing the box it rests in, so its neighbors don't move, and its inside
//! is laid out at the flight box each frame. For the step only, the
//! element's node in its parent's Taffy children is replaced by a
//! placeholder node (`LayoutNodeKind::Held`) with the element's resting
//! style frozen at its resting border box. The element's own node becomes
//! the placeholder's only child, absolute at the placeholder's origin, at
//! the flight size. The placeholder is positioned, so it is the element's
//! containing block, and the element publishes the placeholder's offset: it
//! stays where it rests. The element is positioned too, so its absolute
//! descendants resolve against it, as against a unit root.
//!
//! The element is a layout node's owner: a box, or a participant (a text,
//! an image or a video). Its synced style stays as authored; only the copy
//! its tree receives is the held one (`LayoutData::tree_style`). No Rive
//! object is added. Taffy is driven differently only at a held element:
//! the placeholder lives in the Taffy tree alone, is published as the
//! element's resting box (`resting_box`), and is swept when the hold ends.
//!
//! The rest is frozen at take-off, as a slot leaf's is: a percentage size
//! does not follow a resize during the step, while the place does. While
//! held, and in the frame the hold ends, the element's own box does not
//! interpolate: the flight drives it.
use crate::mechanical_port::source::{
    core::CoreHandle,
    layout_component::{Layout, LayoutComponent},
};
use taffy::prelude::{Dimension, LengthPercentage, LengthPercentageAuto, NodeId, Style};
use taffy::style::{BoxSizing, Display, Overflow, Position};

#[derive(Clone, Debug)]
pub(crate) struct FlightHold {
    /// The element's own style as it rests, frozen at take-off at its
    /// resting border box: the placeholder's style.
    rest_style: Style,
    /// The border box the element's inside is laid out at.
    size: (f32, f32),
    /// The placeholder's node, once the parent's children were synced.
    pub(crate) placeholder: Option<NodeId>,
    /// The placeholder's last solved box, relative to the element's layout
    /// parent: the element's own published layout, were it at rest.
    pub(crate) rest: Layout,
}

impl FlightHold {
    /// Freeze `style` (the element's synced style) at its solved box `rest`.
    fn new(style: &Style, rest: Layout, size: (f32, f32)) -> Self {
        let mut frozen = style.clone();
        let (w, h) = (rest.width(), rest.height());
        frozen.box_sizing = BoxSizing::BorderBox;
        frozen.size.width = Dimension::length(w);
        frozen.size.height = Dimension::length(h);
        frozen.min_size.width = LengthPercentageAuto::length(w);
        frozen.min_size.height = LengthPercentageAuto::length(h);
        frozen.max_size = frozen.min_size;
        frozen.flex_grow = 0.0;
        frozen.flex_shrink = 0.0;
        frozen.flex_basis = Dimension::auto();
        frozen.aspect_ratio = None;
        // The placeholder paints nothing and holds one absolute child: no
        // padding, border or overflow of its own, a plain block container,
        // and positioned so that it is that child's containing block. Its
        // margins, insets and grid placement are the element's.
        let zero = LengthPercentage::length(0.0);
        frozen.padding = taffy::geometry::Rect {
            left: zero,
            right: zero,
            top: zero,
            bottom: zero,
        };
        frozen.border = frozen.padding;
        frozen.overflow = taffy::geometry::Point {
            x: Overflow::Visible,
            y: Overflow::Visible,
        };
        frozen.scroll_content = false;
        frozen.item_is_replaced = false;
        frozen.display = Display::Block;
        if frozen.position == Position::Static {
            frozen.position = Position::Relative;
        }
        Self {
            rest_style: frozen,
            size,
            placeholder: None,
            rest,
        }
    }

    /// The placeholder's style.
    pub(crate) fn rest_style(&self) -> &Style {
        &self.rest_style
    }

    /// The element's own style while held: absolute at its placeholder's
    /// origin, exactly the flight size, outside every flow rule.
    pub(crate) fn apply_to_element(&self, style: &mut Style) {
        let (w, h) = self.size;
        let zero = LengthPercentageAuto::length(0.0);
        style.position = Position::Absolute;
        style.inset = taffy::geometry::Rect {
            left: zero,
            top: zero,
            right: LengthPercentageAuto::auto(),
            bottom: LengthPercentageAuto::auto(),
        };
        style.margin = taffy::geometry::Rect {
            left: zero,
            top: zero,
            right: zero,
            bottom: zero,
        };
        style.box_sizing = BoxSizing::BorderBox;
        style.size.width = Dimension::length(w);
        style.size.height = Dimension::length(h);
        style.min_size.width = LengthPercentageAuto::length(w);
        style.min_size.height = LengthPercentageAuto::length(h);
        style.max_size = style.min_size;
        style.flex_grow = 0.0;
        style.flex_shrink = 0.0;
        style.flex_basis = Dimension::auto();
        style.aspect_ratio = None;
        style.grid_row = Default::default();
        style.grid_column = Default::default();
    }
}

/// Starts holding `element`'s place in its parent's layout at the box it
/// was last laid out at; from the next solve its inside is laid out at
/// `size`, a border box. False, and nothing held, for an element that is
/// not laid out by a parent or is already held.
pub(crate) fn begin(element: &CoreHandle, size: (f32, f32)) -> bool {
    let parent = element
        .with_mut(|object| {
            let parent =
                LayoutComponent::layout_parent_of(object).filter(|parent| parent != element)?;
            let data = LayoutComponent::layout_data_of(object)?;
            if data.flight_hold.is_some() || data.node().is_none() {
                return None;
            }
            let rest = data.solved_layout;
            if !(rest.width().is_finite() && rest.height().is_finite()) {
                return None;
            }
            data.flight_hold = Some(Box::new(FlightHold::new(data.style(), rest, size)));
            data.hold_changed();
            Some(parent)
        })
        .flatten();
    let Some(parent) = parent else {
        return false;
    };
    #[cfg(any(test, feature = "test-support"))]
    HOLDS_BEGUN.with(|count| count.set(count.get() + 1));
    LayoutComponent::flight_hold_changed(&parent);
    true
}

/// Lays the held element's inside out at `size` from the next solve.
pub(crate) fn set_size(element: &CoreHandle, size: (f32, f32)) {
    let parent = element
        .with_mut(|object| {
            let parent = LayoutComponent::layout_parent_of(object)?;
            let data = LayoutComponent::layout_data_of(object)?;
            let hold = data.flight_hold.as_mut()?;
            if hold.size == size {
                return None;
            }
            hold.size = size;
            data.hold_changed();
            Some(parent)
        })
        .flatten();
    if let Some(parent) = parent {
        LayoutComponent::mark_layout_node_dirty_occurrence(&parent, false);
    }
}

/// Ends the hold: the next solve gives the element back its place, and its
/// own box takes its resting layout at once.
pub(crate) fn end(element: &CoreHandle) {
    let parent = element
        .with_mut(|object| {
            let parent = LayoutComponent::layout_parent_of(object);
            let data = LayoutComponent::layout_data_of(object)?;
            data.flight_hold.take()?;
            data.hold_changed();
            data.snap_bounds_once();
            parent
        })
        .flatten();
    if let Some(parent) = parent {
        LayoutComponent::flight_hold_changed(&parent);
    }
}

/// Where `element` rests, relative to its layout parent, as layout last
/// solved it: while held, its placeholder's box; otherwise its own. The one
/// reader of a resting box that a flight's size, its placement and a zoom's
/// ends read.
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
