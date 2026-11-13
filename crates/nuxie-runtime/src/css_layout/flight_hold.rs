//! PROBE (one shape for a Screen): the layout hold.
//!
//! While a matched element flies in place, its parent must keep seeing the
//! box it rests in, so its neighbors don't move, while the element's inside
//! is laid out at the flight box each frame (Layout mode).
//!
//! Mechanism: for the step only, the element's node in its parent's Taffy
//! child list is replaced by a placeholder node (`LayoutNodeKind::Held`)
//! carrying the element's resting style, frozen at its resting border box.
//! The element's own node becomes the placeholder's only child, absolutely
//! placed at the placeholder's padding-box origin, at the flight size. The
//! placeholder is positioned, so it is the element's containing block and
//! its published offset is the placeholder's. No Rive object is added; the
//! placeholder lives in the Taffy tree only, and is swept when the hold ends.
use crate::mechanical_port::source::layout_component::Layout;
use taffy::prelude::{Dimension, LengthPercentage, LengthPercentageAuto, NodeId, Style};
use taffy::style::{BoxSizing, Display, Overflow, Position};

#[derive(Clone, Debug)]
pub(crate) struct FlightHold {
    /// The element's own style as it rests, frozen at take-off at its resting
    /// border box: the placeholder's style.
    pub(crate) rest_style: Style,
    /// The box the element's inside is laid out at, border box.
    pub(crate) size: (f32, f32),
    /// The placeholder's Taffy node, once its parent's children were synced.
    pub(crate) placeholder: Option<NodeId>,
    /// The placeholder's last solved box, relative to the element's layout
    /// parent, as the element's own published layout would read at rest.
    pub(crate) rest: Option<Layout>,
}

impl FlightHold {
    /// Freeze `style` (the element's synced style) at its solved border box.
    pub(crate) fn new(style: &Style, rest: Layout) -> Self {
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
        // and positioned so that it is that child's containing block.
        frozen.padding = taffy::geometry::Rect {
            left: LengthPercentage::length(0.0),
            right: LengthPercentage::length(0.0),
            top: LengthPercentage::length(0.0),
            bottom: LengthPercentage::length(0.0),
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
            size: (w, h),
            placeholder: None,
            rest: Some(rest),
        }
    }

    /// The element's own style while held: absolute at its placeholder's
    /// origin, exactly the flight size, outside every flow rule.
    pub(crate) fn apply_to_element(&self, style: &mut Style) {
        let (w, h) = self.size;
        style.position = Position::Absolute;
        style.inset = taffy::geometry::Rect {
            left: LengthPercentageAuto::length(0.0),
            top: LengthPercentageAuto::length(0.0),
            right: LengthPercentageAuto::auto(),
            bottom: LengthPercentageAuto::auto(),
        };
        style.margin = taffy::geometry::Rect {
            left: LengthPercentageAuto::length(0.0),
            top: LengthPercentageAuto::length(0.0),
            right: LengthPercentageAuto::length(0.0),
            bottom: LengthPercentageAuto::length(0.0),
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
