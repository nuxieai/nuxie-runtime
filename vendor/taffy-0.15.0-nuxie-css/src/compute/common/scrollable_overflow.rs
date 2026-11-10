//! Generic CSS scrollable overflow code that is shared between all CSS algorithms.
use crate::geometry::{Point, Rect, Size};
use crate::style::{Contain, Overflow};
use crate::util::sys::{f32_max, f32_min};

// nuxie-css: scrollable overflow in two parts (see NUXIE_PATCH.md).
/// A container's scrollable overflow before its own end padding: `overflow` is the stock
/// rectangle (border boxes and the descendant overflow that escapes them), `margin_boxes` the
/// union of its in-flow children's margin boxes. Chrome extends a scroll container's end padding
/// past the margin boxes of its own in-flow children and counts everything else as it is, so a
/// last child's end margin and the padding extend a scroll range, and a descendant overflowing
/// its parent gets neither.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InflowOverflow {
    /// Border boxes and escaping descendant overflow
    pub(crate) overflow: Rect<f32>,
    /// In-flow children's margin boxes, at their in-flow positions
    pub(crate) margin_boxes: Rect<f32>,
}

impl InflowOverflow {
    /// Nothing yet: both rectangles hold only the scroll origin.
    pub(crate) const ZERO: Self = Self { overflow: Rect::ZERO, margin_boxes: Rect::ZERO };

    /// Both parts joined with another's.
    pub(crate) fn union(self, other: Self) -> Self {
        Self { overflow: self.overflow.union(other.overflow), margin_boxes: self.margin_boxes.union(other.margin_boxes) }
    }

    /// The container's scrollable overflow rectangle. `end_padding` is a scroll container's
    /// inline-end and bottom padding, `None` for a box that is not a scroll container: its
    /// children's margins do not count, as in Chrome.
    pub(crate) fn rect(self, end_padding: Option<Size<f32>>) -> Rect<f32> {
        match end_padding {
            Some(padding) => self.overflow.union(Rect {
                right: self.margin_boxes.right + padding.width,
                bottom: self.margin_boxes.bottom + padding.height,
                ..self.margin_boxes
            }),
            None => self.overflow,
        }
    }
}

/// Whether a box hands its children's margin boxes to its scroll container: it is marked as the
/// container's content box (`Style::scroll_content`) and lets overflow out on both axes. A
/// content box that clips or contains its overflow keeps its children's margin boxes in.
pub(crate) fn passes_scroll_content(style: &impl crate::CoreStyle) -> bool {
    let overflow = style.overflow();
    style.scroll_content()
        && overflow.x == Overflow::Visible
        && overflow.y == Overflow::Visible
        && !style.contain().contains_scrollable_overflow()
}

/// What an in-flow block, flex or grid child adds to its parent's [`InflowOverflow`]: the stock
/// contribution, and its margin box.
///
/// `margin` holds the used physical margins, auto margins included, or `None` for a child that
/// has no margin box here (a float or an absolutely positioned box). Block layout passes inline
/// margins and a zero height: its block-axis end is the flow's end after the trailing collapsed
/// margins, added once for the whole flow. `relative_offset` is the physical
/// `position: relative` shift already in `location`: Chrome counts the margin box at the in-flow
/// position and the shifted border box separately. Callers mirror `location` for RTL, so the
/// logical start edge is then the physical right one. The margin box counts even when the border
/// box is empty, as in Chrome. `end_stops_at_start`: a negative end margin pulls the end edge
/// back no further than the border box's start edge, as Chrome has it for flex items and block
/// children and not for grid items.
///
/// `scroll_content_margin_boxes` is the child's own children's margin boxes when the child is
/// its scroll container's content box (`Style::scroll_content`): they count as the parent's.
#[allow(clippy::too_many_arguments)]
pub(crate) fn compute_inflow_overflow_contribution(
    location: Point<f32>,
    size: Size<f32>,
    scrollable_overflow_rect: Rect<f32>,
    overflow: Point<Overflow>,
    contain: Contain,
    parent_is_scroll_container: bool,
    margin: Option<Rect<f32>>,
    relative_offset: Point<f32>,
    is_rtl: bool,
    end_stops_at_start: bool,
    scroll_content_margin_boxes: Option<Rect<f32>>,
) -> InflowOverflow {
    InflowOverflow {
        overflow: compute_scrollable_overflow_contribution(
            location,
            size,
            scrollable_overflow_rect,
            overflow,
            contain,
            parent_is_scroll_container,
        ),
        margin_boxes: match margin {
            Some(margin) => compute_inflow_margin_boxes(
                location,
                size,
                margin,
                relative_offset,
                is_rtl,
                end_stops_at_start,
                scroll_content_margin_boxes,
            ),
            None => Rect::ZERO,
        },
    }
}

/// The margin-box half of [`compute_inflow_overflow_contribution`].
pub(crate) fn compute_inflow_margin_boxes(
    location: Point<f32>,
    size: Size<f32>,
    margin: Rect<f32>,
    relative_offset: Point<f32>,
    is_rtl: bool,
    end_stops_at_start: bool,
    scroll_content_margin_boxes: Option<Rect<f32>>,
) -> Rect<f32> {
    let mut margin_boxes = Rect::ZERO;
    let origin = Point {
        x: location.x - if is_rtl { -relative_offset.x } else { relative_offset.x },
        y: location.y - relative_offset.y,
    };
    // Each axis counts on its own: a box wholly above the origin still counts its inline extent
    // (Chrome). With `end_stops_at_start`, a negative end margin pulls the end edge back no
    // further than the border box's start edge.
    let floor = if end_stops_at_start { origin } else { Point { x: f32::MIN, y: f32::MIN } };
    let margin_box = Rect {
        left: origin.x - if is_rtl { margin.right } else { margin.left },
        right: f32_max(floor.x, origin.x + size.width + if is_rtl { margin.left } else { margin.right }),
        top: origin.y - margin.top,
        bottom: f32_max(floor.y, origin.y + size.height + margin.bottom),
    };
    margin_boxes = margin_boxes.union(margin_box);
    if let Some(inner) = scroll_content_margin_boxes {
        margin_boxes = margin_boxes.union(Rect {
            left: location.x + inner.left,
            right: location.x + inner.right,
            top: location.y + inner.top,
            bottom: location.y + inner.bottom,
        });
    }
    margin_boxes
}

#[inline(always)]
/// Determine the rectangle that a given node contributes to its parent's scrollable overflow
/// rectangle.
///
/// `location` is the position of the node's border box measured from the parent's scroll origin
/// (logical start edge): callers pass coordinates that are mirrored for RTL. The node's own
/// `scrollable_overflow_rect` propagates to the parent only where the node's overflow allows it
/// to escape: a scroll container (`Hidden`/`Scroll` in either axis) clips both axes and
/// contributes only its border box, while for non-scroll-containers `Clip` clips just its own
/// axis and `Visible` propagates.
///
/// Boxes positioned wholly in the unreachable scrollable overflow region (entirely before the
/// scroll origin in either axis) must be clipped by a scroll container and are excluded from its
/// scrollable overflow region (<https://www.w3.org/TR/css-overflow-3/#scrollable>). This
/// exclusion only applies when the parent is a scroll container: boxes with `overflow: visible`
/// have no unreachable region of their own, so all of their content contributes.
///
/// A box whose containment contains its scrollable overflow (layout or paint containment)
/// contributes only its border box, regardless of its overflow style.
pub(crate) fn compute_scrollable_overflow_contribution(
    location: Point<f32>,
    size: Size<f32>,
    scrollable_overflow_rect: Rect<f32>,
    overflow: Point<Overflow>,
    contain: Contain,
    parent_is_scroll_container: bool,
) -> Rect<f32> {
    let is_scroll_container = overflow.x.is_scroll_container() || overflow.y.is_scroll_container();
    let overflow_is_contained = contain.contains_scrollable_overflow();
    let propagates = Point {
        x: !is_scroll_container && !overflow_is_contained && overflow.x == Overflow::Visible,
        y: !is_scroll_container && !overflow_is_contained && overflow.y == Overflow::Visible,
    };
    let end_extent = Size {
        width: if propagates.x { f32_max(size.width, scrollable_overflow_rect.right) } else { size.width },
        height: if propagates.y { f32_max(size.height, scrollable_overflow_rect.bottom) } else { size.height },
    };
    if end_extent.width <= 0.0 || end_extent.height <= 0.0 {
        return Rect::ZERO;
    }
    let start_extent = Point {
        x: if propagates.x { f32_min(0.0, scrollable_overflow_rect.left) } else { 0.0 },
        y: if propagates.y { f32_min(0.0, scrollable_overflow_rect.top) } else { 0.0 },
    };
    let contribution = Rect {
        left: location.x + start_extent.x,
        right: location.x + end_extent.width,
        top: location.y + start_extent.y,
        bottom: location.y + end_extent.height,
    };
    let is_wholly_unreachable = contribution.right <= 0.0 || contribution.bottom <= 0.0;
    if parent_is_scroll_container && is_wholly_unreachable {
        Rect::ZERO
    } else {
        contribution
    }
}
