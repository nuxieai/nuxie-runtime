//! CSS replaced picture sizing. All sizes returned here are border boxes.
use taffy::{
    BoxSizing, Size, Style,
    tree::{LayoutInput, SizingMode},
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
    let min = to_basis(style.min_size.maybe_resolve(input.parent_size, calc))
        .map(|value| value.unwrap_or(0.0));
    let max = to_basis(style.max_size.maybe_resolve(input.parent_size, calc))
        .map(|value| value.unwrap_or(f32::INFINITY))
        .zip_map(min, f32::max);
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
    (size + basis_inset).zip_map(inset, f32::max)
}
