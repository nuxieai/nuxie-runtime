//! The one reader of where an element is drawn. `drawn` gives the whole matrix
//! from a component's content to the page's drawing space, in the order the
//! draw applies it. The host's fit is outside that space: a host applies it
//! once to the whole page, so it cancels across anything read here.
use super::follow::draw_pose;
use crate::{
    Artboard, CoreHandle,
    css_layout::hosted_frame,
    source::math::{mat2d::Mat2D, vec2d::Vec2D},
};
use anyhow::Result;

/// The moment `drawn` reads a component at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    /// As it draws now.
    Live,
    /// At its resting place. On every level above it, layout is as laid out
    /// when read, a value no timeline keys is kept, static or bound, and a
    /// transform property a timeline keys is at identity. So a key that ends
    /// off identity rests at identity, not on the value it holds.
    Rest,
}

/// The whole matrix from `component`'s content to the page's drawing space.
///
/// Rest is the live matrix corrected at each level a timeline keys. For a level
/// k with D its live matrix, T its own transform now and T* its own transform
/// at rest, the correction is `D * T⁻¹ * T* * D⁻¹`, since `D * T⁻¹` is the chain
/// down to k without k's own transform. A level whose D or T can't be inverted
/// (a zero scale now) leaves no correction to make, and stays as it draws.
#[cfg_attr(not(feature = "test-support"), allow(dead_code))]
pub(crate) fn drawn(component: &CoreHandle, when: When) -> Result<Mat2D> {
    let live = draw_pose(component)?.0;
    if when == When::Live {
        return Ok(live);
    }
    let mut rest = live;
    for level in keyed_levels(component) {
        if let Some(correction) = correction(&level)? {
            rest = correction * rest;
        }
    }
    Ok(rest)
}

/// The levels from `component` up to the page root that a timeline keys,
/// innermost first. Each artboard's next level is its host, as the draw's.
/// The mark is read only on CSS levels, so Rive-made content has none.
fn keyed_levels(component: &CoreHandle) -> Vec<CoreHandle> {
    let mut levels = Vec::new();
    let mut level = Some(component.clone());
    while let Some(current) = level {
        let (keyed, next) = current
            .with(|o| {
                let keyed = o
                    .as_layout_component()
                    .is_some_and(|box_| box_.is_css_mode() && box_.transform_keyed());
                let next = match o.as_artboard() {
                    Some(board) => board.host(),
                    None => o.component_parent_handle(),
                };
                (keyed, next)
            })
            .unwrap_or((false, None));
        if keyed {
            levels.push(current);
        }
        level = next;
    }
    levels
}

fn correction(level: &CoreHandle) -> Result<Option<Mat2D>> {
    let Some((own, at_rest)) = level.with(own_transforms).flatten() else {
        return Ok(None);
    };
    let live = draw_pose(level)?.0;
    let (mut live_inverse, mut own_inverse) = (Mat2D::identity(), Mat2D::identity());
    if !live.invert(&mut live_inverse) || !own.invert(&mut own_inverse) {
        return Ok(None);
    }
    Ok(Some(live * own_inverse * at_rest * live_inverse))
}

/// A level's own transform now and at rest, each from the function the draw
/// uses at that level: an artboard's `self_transform`, or a box's builder.
fn own_transforms(level: &dyn crate::source::core::CoreObject) -> Option<(Mat2D, Mat2D)> {
    if let Some(board) = level.as_artboard() {
        return Some((board.self_transform(), artboard_at_rest(board)));
    }
    let box_ = level.as_layout_component()?;
    Some((box_.build_own_transform(), box_.own_transform_at_rest()))
}

/// `Artboard::self_transform` with its keyed properties at identity.
fn artboard_at_rest(board: &Artboard) -> Mat2D {
    let layout = &board.base.base;
    let values = layout.rest_values();
    hosted_frame::self_transform(
        layout
            .is_css_slot_frame()
            .then(|| Vec2D::new(values.x, values.y)),
        values.rotation,
        Vec2D::new(values.scale_x, values.scale_y),
        board.flight_stretch(),
        board.origin_is_pivot_only().then(|| board.origin_offset()),
    )
}
