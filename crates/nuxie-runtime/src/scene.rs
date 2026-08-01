//! Direct Rust owner for pinned C++ `src/scene.cpp`.
//!
//! Owns the thin default-scene, root-advance, and pointer facade. Ratcheted
//! public signatures and concrete pointer/event dispatch stay in their owners.

use crate::components::ComponentDirt;
use crate::properties::property_key_for_name;
use crate::scripting::ScriptError;
use crate::{ArtboardInstance, NoopScriptHost, StateMachineInstance};

/// The player selected by pinned C++ `Artboard::defaultScene`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeArtboardDefaultScene {
    StateMachine(usize),
    LinearAnimation(usize),
}

pub(crate) fn select_default_state_machine(
    authored_index: Option<u64>,
    state_machine_count: usize,
) -> Option<usize> {
    authored_index
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < state_machine_count)
}

fn select_default_scene(
    authored_index: Option<u64>,
    state_machine_count: usize,
    animation_count: usize,
) -> Option<RuntimeArtboardDefaultScene> {
    select_default_state_machine(authored_index, state_machine_count)
        .map(RuntimeArtboardDefaultScene::StateMachine)
        .or_else(|| {
            (state_machine_count != 0).then_some(RuntimeArtboardDefaultScene::StateMachine(0))
        })
        .or_else(|| {
            (animation_count != 0).then_some(RuntimeArtboardDefaultScene::LinearAnimation(0))
        })
}

/// Root C++ `Artboard::advance` settlement boundary.
///
/// C++ polls retained decoder promises immediately before this call.
/// Rust decoders are synchronously resolved by the owning host/File seam,
/// so this occurrence has no additional async queue to poll.
pub(crate) fn advance(
    artboard: &mut ArtboardInstance,
    elapsed_seconds: f32,
) -> Result<bool, ScriptError> {
    let component_result = artboard.advance_frame_components(elapsed_seconds);
    let mut changed = component_result.as_ref().copied().unwrap_or(false);
    let update_result = artboard.update_pass_with_script_errors();
    changed |= update_result.as_ref().copied().unwrap_or(false);
    if let Err(error) = component_result {
        return Err(error);
    }
    if let Err(error) = update_result {
        return Err(error);
    }
    Ok(changed || artboard.has_dirt(ComponentDirt::COMPONENTS))
}

pub(crate) fn pointer_down(
    scene: &mut StateMachineInstance,
    artboard: &mut ArtboardInstance,
    x: f32,
    y: f32,
    pointer_id: i32,
) -> bool {
    let result =
        scene.try_pointer_down_with_script_host(artboard, x, y, pointer_id, &mut NoopScriptHost);
    scene.retain_script_result(result)
}

pub(crate) fn pointer_move(
    scene: &mut StateMachineInstance,
    artboard: &mut ArtboardInstance,
    x: f32,
    y: f32,
    seconds: f32,
    pointer_id: i32,
) -> bool {
    let result = scene.try_pointer_move_with_timestamp_and_script_host(
        artboard,
        x,
        y,
        pointer_id,
        seconds,
        &mut NoopScriptHost,
    );
    scene.retain_script_result(result)
}

pub(crate) fn pointer_up(
    scene: &mut StateMachineInstance,
    artboard: &mut ArtboardInstance,
    x: f32,
    y: f32,
    pointer_id: i32,
) -> bool {
    let result =
        scene.try_pointer_up_with_script_host(artboard, x, y, pointer_id, &mut NoopScriptHost);
    scene.retain_script_result(result)
}

pub(crate) fn pointer_exit(
    scene: &mut StateMachineInstance,
    artboard: &mut ArtboardInstance,
    x: f32,
    y: f32,
    pointer_id: i32,
) -> bool {
    let result =
        scene.try_pointer_exit_with_script_host(artboard, x, y, pointer_id, &mut NoopScriptHost);
    scene.retain_script_result(result)
}

impl ArtboardInstance {
    /// Pinned C++ selection order: explicit default state machine, state
    /// machine zero, linear animation zero, then null.
    pub fn default_scene(&self) -> Option<RuntimeArtboardDefaultScene> {
        select_default_scene(
            property_key_for_name("Artboard", "defaultStateMachineId")
                .and_then(|key| self.uint_property(0, key)),
            self.state_machines.len(),
            self.linear_animations.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scene_selection_covers_explicit_fallback_and_null_branches() {
        assert_eq!(select_default_state_machine(Some(1), 2), Some(1));
        assert_eq!(select_default_state_machine(None, 2), None);
        assert_eq!(select_default_state_machine(Some(2), 2), None);
        assert_eq!(
            select_default_scene(Some(1), 2, 1),
            Some(RuntimeArtboardDefaultScene::StateMachine(1))
        );
        assert_eq!(
            select_default_scene(Some(9), 2, 1),
            Some(RuntimeArtboardDefaultScene::StateMachine(0))
        );
        assert_eq!(
            select_default_scene(None, 0, 1),
            Some(RuntimeArtboardDefaultScene::LinearAnimation(0))
        );
        assert_eq!(select_default_scene(None, 0, 0), None);
    }
}
