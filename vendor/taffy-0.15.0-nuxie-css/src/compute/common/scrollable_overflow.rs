//! Generic CSS scrollable overflow code that is shared between all CSS algorithms.
use crate::geometry::{Point, Rect, Size};
use crate::style::{Contain, Overflow};
use crate::util::sys::{f32_max, f32_min};

// nuxie-css: the in-flow margin-area contribution (see NUXIE_PATCH.md).
/// Returns `compute_scrollable_overflow_contribution` extended by the node's margin box, for
/// in-flow block, flex and grid children. Chrome counts an in-flow child's margin box in its
/// scroll container's scrollable overflow, so a last child's end margin extends the scroll
/// range; stock Taffy counts only the border box.
///
/// This applies at every level, not only directly inside a scroll container, so a box whose
/// children overflow it (such as a scroller's content box sized to its viewport) passes their
/// margin boxes on. Chrome counts the margins of a scroll container's own children only.
///
/// `margin` holds the used physical margins, auto margins included, or `None` for a child that
/// keeps the stock contribution. Block layout passes inline margins only and adds its block-axis
/// end, the trailing collapsed margins, once for the whole flow. `relative_offset` is the
/// physical `position: relative` shift already in `location`: Chrome counts the margin box at
/// the in-flow position and the shifted border box separately. Callers mirror `location` for
/// RTL, so the logical start edge is then the physical right one. The margin box counts even
/// when the border box is empty, as in Chrome. A margin box wholly before a scroll container's
/// origin stays excluded, and margins never move or enlarge descendant overflow or shrink the
/// stock contribution.
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
) -> Rect<f32> {
    let contribution = compute_scrollable_overflow_contribution(
        location,
        size,
        scrollable_overflow_rect,
        overflow,
        contain,
        parent_is_scroll_container,
    );
    let Some(margin) = margin else {
        return contribution;
    };
    let origin = Point {
        x: location.x - if is_rtl { -relative_offset.x } else { relative_offset.x },
        y: location.y - relative_offset.y,
    };
    let margin_box = Rect {
        left: origin.x - if is_rtl { margin.right } else { margin.left },
        right: origin.x + size.width + if is_rtl { margin.left } else { margin.right },
        top: origin.y - margin.top,
        bottom: origin.y + size.height + margin.bottom,
    };
    if parent_is_scroll_container && (margin_box.right <= 0.0 || margin_box.bottom <= 0.0) {
        return contribution;
    }
    contribution.union(margin_box)
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
