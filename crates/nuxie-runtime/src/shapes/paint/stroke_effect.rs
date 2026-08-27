//! StrokeEffect invalidation is occurrence-local. `invalidateEffectFromLocal`
//! rewinds this effect's retained paths and then invalidates only downstream
//! effects through the parent EffectsContainer.

use nuxie_graph::{ShapePaintNode, StrokeEffectNode};

use crate::{
    ArtboardInstance,
    draw::{RuntimePathCommand, runtime_dash_path_effect_commands},
};

pub(crate) fn runtime_stroke_effect_path_commands(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    match effect.type_name {
        "DashPath" => runtime_dash_path_effect_commands(artboard, effect, paint, source),
        "ScriptedPathEffect" => {
            crate::scripted_path_effect::update_effect(artboard, effect, paint, source)
        }
        "TargetEffect" => crate::shapes::paint::target_effect::update_effect(
            &source.to_vec(),
            &effect.group_effects,
            |group_effect, current| {
                runtime_stroke_effect_path_commands(artboard, group_effect, paint, current)
            },
        ),
        "TrimPath" => super::trim_path::runtime_trim_path_line_effect_commands(
            artboard, effect, paint, source,
        ),
        _ => None,
    }
}

pub(crate) fn invalidate_effect_from_local(
    artboard: &mut ArtboardInstance,
    local_id: usize,
) -> bool {
    artboard.invalidate_runtime_stroke_effect_from_local(local_id)
}
