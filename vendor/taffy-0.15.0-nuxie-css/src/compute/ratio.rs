//! nuxie-css: a box's size styles through its preferred aspect ratio, as CSS Box Sizing 4 §5
//! and Chrome size them. Ported from Exact's Taffy copy (its patch 12); see NUXIE_PATCH.md.
//!
//! - The ratio relates the sizes of the box `box-sizing` names.
//! - One dimension given: it is clamped by its own min/max, and the other is derived from the
//!   clamped value and clamped on its own. Two given: the ratio does nothing.
//! - Min/max transfer through the ratio only into a dimension that is not given (a block's
//!   stretched width takes a `max-height` as a max width; a set width does not).
//! - A derived height of a box that is neither replaced nor a scroll container, whose
//!   `min-height` is `auto`, is a floor its content can pass (§5.2's automatic minimum), not a
//!   fixed height.
use crate::geometry::Size;
use crate::util::sys::{f32_max, f32_min};
use crate::util::MaybeMath;
use crate::{BoxSizing, CoreStyle};

/// A usable preferred ratio and the inset between the border box and the box it relates.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Ratio {
    /// Width ÷ height, finite and positive.
    ratio: f32,
    /// What a border-box size loses to be of the related box.
    pub(crate) inset: Size<f32>,
    /// The padding + border height: the least border-box height the box can have.
    least_height: f32,
}

impl Ratio {
    /// The style's ratio, if it has a usable one. `pb_sum` is the resolved padding + border of
    /// each axis.
    pub(crate) fn of(style: &impl CoreStyle, pb_sum: Size<f32>) -> Option<Self> {
        Self::from_parts(style.aspect_ratio(), style.box_sizing() == BoxSizing::ContentBox, pb_sum)
    }

    /// A ratio from a style's parts, for an algorithm that keeps the parts and not the style:
    /// `content_box` is whether the ratio relates content-box sizes.
    pub(crate) fn from_parts(ratio: Option<f32>, content_box: bool, pb_sum: Size<f32>) -> Option<Self> {
        let ratio = ratio.filter(|r| r.is_finite() && *r > 0.0)?;
        Some(Self { ratio, inset: if content_box { pb_sum } else { Size::ZERO }, least_height: pb_sum.height })
    }

    /// The border-box height for a border-box width.
    pub(crate) fn height(self, width: f32) -> f32 {
        f32_max(width - self.inset.width, 0.0) / self.ratio + self.inset.height
    }

    /// The border-box width for a border-box height.
    pub(crate) fn width(self, height: f32) -> f32 {
        f32_max(height - self.inset.height, 0.0) * self.ratio + self.inset.width
    }

    /// The border-box size, min and max of a box with these given sizes and authored min/max
    /// (all border-box). `floor_height`: the derived height is an automatic minimum rather than
    /// a size (see the module note).
    pub(crate) fn resolve(
        self,
        given: Size<Option<f32>>,
        min: Size<Option<f32>>,
        max: Size<Option<f32>>,
        floor_height: bool,
    ) -> (Size<Option<f32>>, Size<Option<f32>>, Size<Option<f32>>) {
        let (mut min_out, mut max_out) = (min, max);
        if given.width.is_none() {
            if let Some(h) = min.height {
                // A transferred minimum never passes the maximum of the axis it lands in.
                min_out.width = Some(f32_max(min.width.unwrap_or(0.0), self.width(h).maybe_min(max.width)));
            }
            if let Some(h) = max.height {
                // A `max-height` under the padding and border is that much in effect.
                let h = f32_max(h, self.least_height);
                max_out.width = Some(max.width.map_or(self.width(h), |w| f32_min(w, self.width(h))));
            }
        }
        if given.height.is_none() {
            if let Some(w) = min.width {
                min_out.height = Some(f32_max(min.height.unwrap_or(0.0), self.height(w).maybe_min(max.height)));
            }
            // An automatic content-based height can exceed max-width's transfer (CSS Sizing 4
            // §4.3). Its ratio floor is derived from the capped width below; only an authored
            // max-height caps the content.
            if let Some(w) = max.width.filter(|_| !floor_height) {
                max_out.height = Some(max.height.map_or(self.height(w), |h| f32_min(h, self.height(w))));
            }
        }
        // With neither dimension given, the padding and border the height cannot go under
        // give a least width through the ratio (a border-box ratio; Chrome).
        if given.width.is_none() && given.height.is_none() {
            let least_width = self.width(self.least_height).maybe_min(max.width);
            min_out.width = Some(f32_max(min_out.width.unwrap_or(0.0), least_width));
        }
        let mut size = given;
        match (given.width, given.height) {
            (Some(w), None) => {
                let derived = self.height(w.maybe_clamp(min_out.width, max_out.width));
                if floor_height {
                    let floor = derived.maybe_min(max_out.height);
                    min_out.height = Some(f32_max(min_out.height.unwrap_or(0.0), floor));
                } else {
                    size.height = Some(derived);
                }
            }
            (None, Some(h)) => {
                size.width = Some(self.width(h.maybe_clamp(min_out.height, max_out.height)));
            }
            _ => {}
        }
        (size, min_out, max_out)
    }
}

/// Whether a box's derived height is a floor its content can pass: not replaced, not a scroll
/// container, `height: auto`, `min-height: auto`.
pub(crate) fn floors_height(style: &impl CoreStyle) -> bool {
    style.size().height.is_auto() && floors_unsized_height(style)
}

/// [`floors_height`] for a query in `sizing_mode`: a content-size probe leaves the box's own
/// `height` out, so it does not ask whether that height is `auto`.
pub(crate) fn floors_height_in(style: &impl CoreStyle, sizing_mode: crate::SizingMode) -> bool {
    if sizing_mode == crate::SizingMode::ContentSize {
        floors_unsized_height(style)
    } else {
        floors_height(style)
    }
}

/// [`floors_height`] without the box's own `height`.
fn floors_unsized_height(style: &impl CoreStyle) -> bool {
    let overflow = style.overflow();
    // The authored value: a percentage that resolves against nothing is not `auto` here.
    !style.is_compressible_replaced()
        && style.min_size().height.is_auto()
        && !overflow.x.is_scroll_container()
        && !overflow.y.is_scroll_container()
}

/// A style's resolved border-box size, min and max through its ratio, if it has one; unchanged
/// otherwise.
pub(crate) fn sizes_through_ratio(
    style: &impl CoreStyle,
    pb_sum: Size<f32>,
    size: Size<Option<f32>>,
    min: Size<Option<f32>>,
    max: Size<Option<f32>>,
) -> (Size<Option<f32>>, Size<Option<f32>>, Size<Option<f32>>) {
    match Ratio::of(style, pb_sum) {
        Some(ratio) => ratio.resolve(size, min, max, floors_height(style)),
        None => (size, min, max),
    }
}

/// Resolve a container's incoming sizes. A non-replaced content probe with no width and a
/// supplied height treats the transferred width as a floor when min-width is auto and neither
/// overflow axis scrolls. Wider content can exceed the transferred maximum; an
/// indefinite-height probe keeps it.
pub(crate) fn container_sizes_through_ratio(
    style: &impl CoreStyle,
    pb_sum: Size<f32>,
    given: Size<Option<f32>>,
    min: Size<Option<f32>>,
    max: Size<Option<f32>>,
    sizing_mode: crate::SizingMode,
) -> (Size<Option<f32>>, Size<Option<f32>>, Size<Option<f32>>) {
    let authored_max_width = max.width;
    let floor_height = floors_height_in(style, sizing_mode);
    let (mut size, mut min, mut max) = resolve_through(Ratio::of(style, pb_sum), given, min, max, floor_height);
    if sizing_mode == crate::SizingMode::ContentSize && !style.is_compressible_replaced() && given.width.is_none() {
        if given.height.is_some() {
            max.width = authored_max_width;
        }
        let overflow = style.overflow();
        if style.min_size().width.is_auto() && !overflow.x.is_scroll_container() && !overflow.y.is_scroll_container() {
            if let Some(width) = size.width.take() {
                min.width = Some(width.maybe_min(max.width).maybe_max(min.width));
            }
        }
    }
    (size, min, max)
}

/// `limits` (a min or a max) transferred through `ratio` into the axes the style does not size
/// (`sized`); a sized axis keeps its own limit only.
pub(crate) fn transfer_into_unsized(
    limits: Size<Option<f32>>,
    ratio: Option<Ratio>,
    sized: Size<bool>,
    maximum: bool,
) -> Size<Option<f32>> {
    let transferred = match ratio {
        Some(ratio) => Size {
            // A maximum height under the padding and border is that much in effect.
            width: limits.width.or_else(|| limits.height.map(|h| ratio.width(if maximum { f32_max(h, ratio.least_height) } else { h }))),
            height: limits.height.or_else(|| limits.width.map(|w| ratio.height(w))),
        },
        None => limits,
    };
    Size {
        width: if sized.width { limits.width } else { transferred.width },
        height: if sized.height { limits.height } else { transferred.height },
    }
}

/// [`Ratio::resolve`] for a ratio that may be absent.
pub(crate) fn resolve_through(
    ratio: Option<Ratio>,
    size: Size<Option<f32>>,
    min: Size<Option<f32>>,
    max: Size<Option<f32>>,
    floor_height: bool,
) -> (Size<Option<f32>>, Size<Option<f32>>, Size<Option<f32>>) {
    match ratio {
        Some(ratio) => ratio.resolve(size, min, max, floor_height),
        None => (size, min, max),
    }
}

/// The ratio's preferred block size, within the authored `min-height` and `max-height`, is
/// definite for percentage resolution even when the content-based automatic minimum makes the
/// used box taller.
pub(crate) fn percentage_height(
    style: &impl CoreStyle,
    pb: Size<f32>,
    width: Option<f32>,
    min_height: Option<f32>,
    max_height: Option<f32>,
) -> Option<f32> {
    // A length is the height itself; `auto`, or a percentage with nothing to resolve
    // against, leaves the ratio height.
    if style.size().height.tag() == crate::CompactLength::LENGTH_TAG {
        return None;
    }
    Some(Ratio::of(style, pb)?.height(width?).maybe_clamp(min_height, max_height))
}

/// The automatic minimum width of a non-replaced ratio box with a height and no width: the
/// larger of its min-content width and the height's transfer through the ratio, capped by
/// `max-width` (CSS Sizing 4 §5.1, Flexbox §4.5; Exact's patches 50, 54 and 64). `used_height`
/// is a border-box height flexing gave the box, in place of its preferred height.
pub(crate) fn minimum_ratio_width(
    tree: &mut impl crate::LayoutPartialTree,
    node: crate::NodeId,
    parent: Size<Option<f32>>,
    used_height: Option<f32>,
) -> Option<f32> {
    use crate::{LayoutPartialTreeExt, MaybeResolve, ResolveOrZero};
    let style = tree.get_core_container_style(node);
    let overflow = style.overflow();
    if style.aspect_ratio().is_none()
        || !style.size().width.is_auto()
        || !style.min_size().width.is_auto()
        || style.is_compressible_replaced()
        || overflow.x.is_scroll_container()
        || overflow.y.is_scroll_container()
    {
        return None;
    }
    let pb = (style.padding().resolve_or_zero(parent.width, |v, b| tree.calc(v, b))
        + style.border().resolve_or_zero(parent.width, |v, b| tree.calc(v, b)))
    .sum_axes();
    let ratio = Ratio::of(&style, pb)?;
    let adjustment = if style.box_sizing() == BoxSizing::ContentBox { pb } else { Size::ZERO };
    let min_height =
        style.min_size().height.maybe_resolve(parent.height, |v, b| tree.calc(v, b)).maybe_add(adjustment.height);
    let max = style.max_size().maybe_resolve(parent, |v, b| tree.calc(v, b)).maybe_add(adjustment);
    let height = used_height.or_else(|| {
        style
            .size()
            .height
            .maybe_resolve(parent.height, |v, b| tree.calc(v, b))
            .map(|height| (height + adjustment.height).maybe_clamp(min_height, max.height))
    })?;
    let transferred = ratio.width(height);
    // With `height: auto` (a height flexing gave), `max-height` transfers into the width too
    // and caps the minimum.
    let max_width = if style.size().height.is_auto() {
        // `min-height` wins over a smaller `max-height` before the transfer.
        let max_height = max.height.maybe_max(min_height);
        max.width.maybe_min(max_height.map(|h| ratio.width(h))).or(max_height.map(|h| ratio.width(h)))
    } else {
        max.width
    };
    let height_is_auto = style.size().height.is_auto();
    drop(style);
    // A height flexing gave a column item with `height: auto` transfers alone: the content
    // reaches the item through its automatic minimum height instead (Chrome).
    let intrinsic = if used_height.is_some() && height_is_auto {
        0.0
    } else {
        tree.measure_child_size(
            node,
            Size { width: None, height: Some(height) },
            parent,
            Size { width: crate::AvailableSpace::MinContent, height: crate::AvailableSpace::MinContent },
            crate::SizingMode::ContentSize,
            crate::AbsoluteAxis::Horizontal,
            crate::geometry::Line::FALSE,
        )
    };
    // The padding and border are a floor a smaller `max-width` does not take away.
    Some(f32_max(f32_max(intrinsic, transferred).maybe_min(max_width), pb.width))
}
