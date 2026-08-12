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

/// A default scene selected together with its instantiated player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeDefaultSceneSelection<S, A> {
    StateMachine { index: usize, instance: S },
    LinearAnimation { index: usize, instance: A },
    StaticArtboard,
}

/// Pinned C++ `defaultScene` order. Keeping the decision in this small generic
/// helper makes every branch testable without requiring a binary Rive file.
pub fn select_default_scene<C, S, A>(
    context: &mut C,
    authored_default: Option<usize>,
    mut state_machine_at: impl FnMut(&mut C, usize) -> Option<S>,
    mut animation_at: impl FnMut(&mut C, usize) -> Option<A>,
) -> RuntimeDefaultSceneSelection<S, A> {
    if let Some(index) = authored_default
        && let Some(instance) = state_machine_at(context, index)
    {
        return RuntimeDefaultSceneSelection::StateMachine { index, instance };
    }
    if let Some(instance) = state_machine_at(context, 0) {
        return RuntimeDefaultSceneSelection::StateMachine { index: 0, instance };
    }
    if let Some(instance) = animation_at(context, 0) {
        return RuntimeDefaultSceneSelection::LinearAnimation { index: 0, instance };
    }
    RuntimeDefaultSceneSelection::StaticArtboard
}

pub(crate) fn select_default_state_machine(
    authored_index: Option<u64>,
    state_machine_count: usize,
) -> Option<usize> {
    authored_index
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < state_machine_count)
}

fn select_default_scene_from_counts(
    authored_index: Option<u64>,
    state_machine_count: usize,
    animation_count: usize,
) -> Option<RuntimeArtboardDefaultScene> {
    let mut counts = (state_machine_count, animation_count);
    match select_default_scene(
        &mut counts,
        select_default_state_machine(authored_index, state_machine_count),
        |counts, index| (index < counts.0).then_some(()),
        |counts, index| (index < counts.1).then_some(()),
    ) {
        RuntimeDefaultSceneSelection::StateMachine { index, .. } => {
            Some(RuntimeArtboardDefaultScene::StateMachine(index))
        }
        RuntimeDefaultSceneSelection::LinearAnimation { index, .. } => {
            Some(RuntimeArtboardDefaultScene::LinearAnimation(index))
        }
        RuntimeDefaultSceneSelection::StaticArtboard => None,
    }
}

/// Root C++ `Artboard::advance` settlement boundary.
///
/// C++ polls retained decoder promises immediately before this call. Rust's
/// WorkPool uses the same boundary, then drains each VM-owned completion queue
/// before advancing components so parked/event-only scripts can settle.
pub(crate) fn advance(
    artboard: &mut ArtboardInstance,
    elapsed_seconds: f32,
) -> Result<bool, ScriptError> {
    crate::poll_async_work();
    let async_result = artboard.poll_script_async_work_tree();
    let component_result = artboard.advance_frame_components(elapsed_seconds);
    let mut changed = async_result.as_ref().copied().unwrap_or(false)
        | component_result.as_ref().copied().unwrap_or(false);
    let update_result = artboard.update_pass_with_script_errors();
    changed |= update_result.as_ref().copied().unwrap_or(false);
    if let Err(error) = async_result {
        return Err(error);
    }
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
        select_default_scene_from_counts(
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
            select_default_scene_from_counts(Some(1), 2, 1),
            Some(RuntimeArtboardDefaultScene::StateMachine(1))
        );
        assert_eq!(
            select_default_scene_from_counts(Some(9), 2, 1),
            Some(RuntimeArtboardDefaultScene::StateMachine(0))
        );
        assert_eq!(
            select_default_scene_from_counts(None, 0, 1),
            Some(RuntimeArtboardDefaultScene::LinearAnimation(0))
        );
        assert_eq!(select_default_scene_from_counts(None, 0, 0), None);
    }

    #[test]
    fn shared_default_scene_selection_covers_all_four_rungs() {
        let select = |authored_default, state_machine_count, animation_count| {
            let mut counts = (state_machine_count, animation_count);
            select_default_scene(
                &mut counts,
                authored_default,
                |counts, index| (index < counts.0).then_some(index),
                |counts, index| (index < counts.1).then_some(index),
            )
        };

        assert_eq!(
            select(Some(1), 2, 1),
            RuntimeDefaultSceneSelection::StateMachine {
                index: 1,
                instance: 1,
            }
        );
        assert_eq!(
            select(Some(7), 1, 1),
            RuntimeDefaultSceneSelection::StateMachine {
                index: 0,
                instance: 0,
            }
        );
        assert_eq!(
            select(None, 0, 1),
            RuntimeDefaultSceneSelection::LinearAnimation {
                index: 0,
                instance: 0,
            }
        );
        assert_eq!(
            select(None, 0, 0),
            RuntimeDefaultSceneSelection::StaticArtboard
        );
    }
}
