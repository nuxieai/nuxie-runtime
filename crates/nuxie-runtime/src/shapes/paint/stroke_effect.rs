//! StrokeEffect invalidation is occurrence-local. `invalidateEffectFromLocal`
//! rewinds this effect's retained paths and then invalidates only downstream
//! effects through the parent EffectsContainer.

use nuxie_graph::{ShapePaintNode, StrokeEffectNode};

use crate::{
    ArtboardInstance,
    draw::{
        RuntimePathCommand, runtime_dash_path_effect_commands, runtime_path_commands_from_raw_path,
    },
    math::raw_path::runtime_raw_path_from_commands,
    scripting::{ScriptNode, script_paint_for_shape},
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
            let output = match artboard.apply_scripted_path_effect(
                effect.global_id,
                runtime_raw_path_from_commands(source),
                ScriptNode {
                    path: None,
                    paint: Some(script_paint_for_shape(artboard, paint)),
                },
            ) {
                Ok(output) => output,
                Err(_) => {
                    // C++ still exposes the ScriptedEffectPath when scripting
                    // is unavailable or its update fails. That path was
                    // rewound before the attempted update, so it is empty
                    // rather than a signal to fall back to the source path.
                    eprintln!("update function failed");
                    return Some(Vec::new());
                }
            };
            Some(runtime_path_commands_from_raw_path(&output))
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
