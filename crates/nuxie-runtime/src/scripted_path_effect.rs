use nuxie_graph::{ShapePaintNode, StrokeEffectNode};

use crate::{
    ArtboardInstance,
    draw::{RuntimePathCommand, runtime_path_commands_from_raw_path},
    math::raw_path::runtime_raw_path_from_commands,
    scripting::{ScriptNode, script_paint_for_shape},
};

/// `ScriptedPathEffect::updateEffect`.
///
/// The retained `EffectPath` cache and its invalidation are owned by the draw
/// graph. This function is the live rebuild callback for that exact owner.
pub(crate) fn update_effect(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    // C++ leaves the newly-created retained ScriptedEffectPath empty when the
    // authored ScriptAsset does not implement update.
    if !artboard
        .script_implemented_methods_for_global(effect.global_id)
        .is_some_and(|methods| methods.updates())
    {
        return Some(Vec::new());
    }

    let output = match artboard.apply_scripted_path_effect(
        effect.global_id,
        runtime_raw_path_from_commands(source),
        ScriptNode::snapshot(None, Some(script_paint_for_shape(artboard, paint))),
    ) {
        Ok(output) => output,
        Err(_) => {
            // C++ rewinds ScriptedEffectPath before resolving the state and
            // invoking update, so a missing state or callback failure leaves
            // an empty retained output.
            eprintln!("update function failed");
            return Some(Vec::new());
        }
    };
    Some(runtime_path_commands_from_raw_path(&output))
}
