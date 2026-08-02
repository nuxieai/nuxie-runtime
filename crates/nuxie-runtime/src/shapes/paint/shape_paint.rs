//! ShapePaint owns one RenderPaint, its mutator, optional Feather, and one
//! EffectPath per `(StrokeEffect, PathProvider)` occurrence. The renderer-facing
//! sidecars are retained in `RuntimeShapePaintOwner` and are dirtied only by
//! these owner-local callbacks.

use anyhow::Result;
use nuxie_render_api::{Factory as RenderFactory, FillRule as RenderFillRule, Renderer};

use crate::{
    ArtboardInstance, Mat2D,
    draw::{
        RuntimeShapePaintKind, RuntimeShapePaintOwner, RuntimeShapePaintPathKind,
        runtime_feather_has_offset, runtime_feather_uses_world_space, runtime_fill_rule_for_value,
        runtime_render_mat, runtime_translation,
    },
    shapes::paint::shape_paint_path::RuntimeShapePaintPathOwner,
};

/// Direct port of C++ `ShapePaint::draw` for clone-owned Shape paints. The
/// selected ShapePaintPath and RenderPaint are owner resources; no command
/// object is constructed or consulted on a clean draw.
#[allow(clippy::too_many_arguments)]
pub(crate) fn runtime_draw_live_owned_shape_paint(
    instance: &ArtboardInstance,
    shape_world: Mat2D,
    owner: &RuntimeShapePaintOwner,
    source_path: &RuntimeShapePaintPathOwner,
    live_path_kind: RuntimeShapePaintPathKind,
    needs_save_operation: bool,
    backend_context_id: u64,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
) -> Result<()> {
    let source_retained = source_path.retained.borrow();
    let Some(source_retained) = source_retained.as_ref() else {
        return Ok(());
    };
    let active_effect = owner.effect_paths.iter().rev().find(|effect| {
        effect
            .retained
            .borrow()
            .as_ref()
            .is_some_and(Option::is_some)
    });
    let effect_retained = active_effect.map(|effect| effect.retained.borrow());
    let effect_path = effect_retained
        .as_ref()
        .and_then(|retained| retained.as_ref())
        .and_then(|path| path.as_ref());
    let feather_state = owner.feather_state.borrow();
    let inner_feather_path = owner.inner_feather_path.borrow();
    let mut saved = !needs_save_operation;
    if let Some(feather) = feather_state.as_ref()
        && runtime_feather_uses_world_space(feather)
        && !feather.inner
        && runtime_feather_has_offset(feather)
    {
        if !saved {
            saved = true;
            renderer.save();
        }
        renderer.transform(runtime_translation(feather.offset_x, feather.offset_y));
    }
    if matches!(
        live_path_kind,
        RuntimeShapePaintPathKind::Local | RuntimeShapePaintPathKind::LocalClockwise
    ) {
        if !saved {
            saved = true;
            renderer.save();
        }
        renderer.transform(runtime_render_mat(shape_world));
    }

    if let Some(feather) = feather_state.as_ref() {
        if feather.inner {
            if feather.inner_path_commands.is_empty() {
                return Ok(());
            }
            if !saved {
                saved = true;
                renderer.save();
            }
            let clip_backend = active_effect
                .map(|effect| &effect.backend)
                .unwrap_or(&source_path.backend);
            let clip_raw_path = effect_path
                .map(|path| path.raw_path.as_ref())
                .unwrap_or_else(|| source_retained.raw_path.as_ref());
            clip_backend.with_path(
                backend_context_id,
                factory,
                clip_raw_path,
                RenderFillRule::Clockwise,
                None,
                |clip_path| renderer.clip_path(clip_path),
            );
        } else if !runtime_feather_uses_world_space(feather) && runtime_feather_has_offset(feather)
        {
            if !saved {
                saved = true;
                renderer.save();
            }
            renderer.transform(runtime_translation(feather.offset_x, feather.offset_y));
        }
    }

    let (draw_backend, draw_raw_path) = if let Some(inner_path) = inner_feather_path.as_ref() {
        (&owner.inner_feather_backend, inner_path.raw_path.as_ref())
    } else if let (Some(effect), Some(path)) = (active_effect, effect_path) {
        (&effect.backend, path.raw_path.as_ref())
    } else {
        (&source_path.backend, source_retained.raw_path.as_ref())
    };
    let draw_fill_rule = (owner.paint_type == RuntimeShapePaintKind::Fill).then(|| {
        instance
            .fill_rule(owner.paint_local)
            .map(runtime_fill_rule_for_value)
            .unwrap_or(owner.authored_fill_rule)
    });
    draw_backend.with_path(
        backend_context_id,
        factory,
        draw_raw_path,
        RenderFillRule::Clockwise,
        draw_fill_rule,
        |path| {
            let backend = owner.backend.value.borrow();
            let render_paint = backend
                .paint
                .as_deref()
                .expect("ShapePaint backend was realized before renderer side effects");
            renderer.draw_path(path, render_paint);
        },
    );
    if saved && needs_save_operation {
        renderer.restore();
    }
    Ok(())
}

pub(crate) fn blend_mode(paint_value: u32, parent_value: u32) -> u32 {
    if paint_value == 127 {
        parent_value
    } else {
        paint_value
    }
}
