//! CSS replaced picture sizing. All sizes returned here are border boxes.
use taffy::{
    BoxSizing, Size, Style,
    tree::{LayoutInput, RequestedAxis, SizingMode},
    util::{MaybeResolve, ResolveOrZero},
};

/// Resolve the preferred ratio and constraints together, before Taffy's leaf
/// path can apply an authored ratio a second time after clamping.
/// Min-content and max-content share the intrinsic object size, including a
/// transfer from a definite opposite axis. Available space alone never shrinks
/// a picture. Flex distributes free space and passes its solved size as known.
/// Natural dimensions are content sizes. An authored ratio instead describes
/// the box selected by box-sizing. Known dimensions are always border sizes.
pub(crate) fn resolve_replaced_size(
    style: &Style,
    input: LayoutInput,
    natural: Size<f32>,
) -> Size<f32> {
    let calc = |_: *const (), _: f32| 0.0;
    let inset = (style.padding.resolve_or_zero(input.parent_size.width, calc)
        + style.border.resolve_or_zero(input.parent_size.width, calc))
    .sum_axes();
    let ratio = style
        .aspect_ratio
        .or_else(|| {
            (natural.width > 0.0 && natural.height > 0.0).then_some(natural.width / natural.height)
        })
        .filter(|ratio| ratio.is_finite() && *ratio > 0.0);
    let ratio_uses_border =
        style.aspect_ratio.is_some() && style.box_sizing == BoxSizing::BorderBox;
    let basis_inset = if ratio_uses_border { Size::ZERO } else { inset };
    let authored_inset = if style.box_sizing == BoxSizing::ContentBox {
        inset
    } else {
        Size::ZERO
    };
    let to_basis = |value: Size<Option<f32>>| {
        value.zip_map(authored_inset - basis_inset, |value, adjustment| {
            value.map(|value| (value + adjustment).max(0.0))
        })
    };
    let mut min = to_basis(style.min_size.maybe_resolve(input.parent_size, calc))
        .map(|value| value.unwrap_or(0.0));
    let mut max = to_basis(style.max_size.maybe_resolve(input.parent_size, calc))
        .map(|value| value.unwrap_or(f32::INFINITY))
        .zip_map(min, f32::max);
    // ContentSize is also used for the flex base, before main-axis bounds.
    // Taffy applies those bounds to the hypothetical and final main sizes.
    // Opposite-axis bounds still participate in a ratio transfer here.
    if input.sizing_mode == SizingMode::ContentSize {
        match input.axis {
            RequestedAxis::Horizontal => {
                min.width = 0.0;
                max.width = f32::INFINITY;
            }
            RequestedAxis::Vertical => {
                min.height = 0.0;
                max.height = f32::INFINITY;
            }
            RequestedAxis::Both => {}
        }
    }
    let authored = if input.sizing_mode == SizingMode::InherentSize {
        to_basis(style.size.maybe_resolve(input.parent_size, calc))
    } else {
        Size::NONE
    };
    let known = input
        .known_dimensions
        .zip_map(basis_inset, |value, inset| {
            value.map(|value| (value - inset).max(0.0))
        })
        .or(authored);
    let known = known
        .zip_map(min, |value, min| value.map(|value| value.max(min)))
        .zip_map(max, |value, max| value.map(|value| value.min(max)));
    let intrinsic = natural + inset - basis_inset;
    let mut size = match (known.width, known.height, ratio) {
        (Some(width), Some(height), _) => Size { width, height },
        (Some(width), None, Some(ratio)) => Size {
            width,
            height: width / ratio,
        },
        (None, Some(height), Some(ratio)) => Size {
            width: height * ratio,
            height,
        },
        (None, None, Some(ratio)) => {
            let width = if intrinsic.width > 0.0 {
                intrinsic.width
            } else {
                (intrinsic.height * ratio)
                    .max(min.width)
                    .max(min.height * ratio)
            };
            let tentative = Size {
                width,
                height: width / ratio,
            };
            if tentative.width > 0.0 && tentative.height > 0.0 {
                // CSS 2.1 10.4: preserve the ratio when both limits permit it.
                // Opposing constraints break it; a minimum wins on its axis.
                let lower = (min.width / tentative.width).max(min.height / tentative.height);
                let upper = (max.width / tentative.width).min(max.height / tentative.height);
                let scale = if lower <= upper {
                    1.0f32.clamp(lower, upper)
                } else {
                    lower
                };
                tentative.map(|value| value * scale)
            } else {
                tentative
            }
        }
        _ => Size {
            width: known.width.unwrap_or(intrinsic.width),
            height: known.height.unwrap_or(intrinsic.height),
        },
    };
    size = size.zip_map(max, f32::min).zip_map(min, f32::max);
    // A picture whose width or max-width is a percentage of a zero width is
    // as wide as its own minimum. ContentSize ignores the authored width, so
    // the percentage is applied here. Flex passes a zero parent width while
    // it measures its own min-content width (CSS Sizing 3, 5.2.2,
    // compressible replaced elements; Chrome).
    let width_is_compressible = style.size.width.into_raw().uses_percentage()
        || style.max_size.width.into_raw().uses_percentage();
    if width_is_compressible
        && known.width.is_none()
        && input.parent_size.width == Some(0.0)
    {
        let width = min.width.min(size.width);
        size = Size {
            width,
            height: match (known.height, ratio) {
                (None, Some(ratio)) => (width / ratio).max(min.height).min(max.height),
                _ => size.height,
            },
        };
    }
    (size + basis_inset).zip_map(inset, f32::max)
}
