//! Fill chooses local/local-clockwise retained paths from its live fill rule;
//! draw-time fill-rule replay remains distinct from path construction.

use crate::draw::RuntimeShapePaintPathKind;
use nuxie_render_api::FillRule as RenderFillRule;

pub(crate) fn fill_rule(value: u64) -> RenderFillRule {
    match value {
        1 => RenderFillRule::EvenOdd,
        2 => RenderFillRule::Clockwise,
        _ => RenderFillRule::NonZero,
    }
}

pub(crate) fn pick_path(fill_rule: RenderFillRule) -> RuntimeShapePaintPathKind {
    if fill_rule == RenderFillRule::Clockwise {
        RuntimeShapePaintPathKind::LocalClockwise
    } else {
        RuntimeShapePaintPathKind::Local
    }
}
