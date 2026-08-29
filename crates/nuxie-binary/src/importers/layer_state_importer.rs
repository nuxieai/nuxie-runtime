use super::*;

/// Stable occurrence coordinates are the Rust equivalent of pinned
/// `LayerState* m_State`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LayerStateImporter {
    pub(super) state_machine_index: usize,
    pub(super) layer_index: usize,
    pub(super) state_index: usize,
}

impl LayerStateImporter {
    pub(super) fn new(state_machine_index: usize, layer_index: usize, state_index: usize) -> Self {
        Self {
            state_machine_index,
            layer_index,
            state_index,
        }
    }

    /// The returned slot is the stable-coordinate adaptation used to create
    /// the transition's own importer after the pinned append callback.
    pub(super) fn add_transition<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        transition: RuntimeStateTransition<'a>,
    ) -> usize {
        let transitions = &mut state_machines[self.state_machine_index].layers[self.layer_index]
            .states[self.state_index]
            .transitions;
        transitions.push(transition);
        transitions.len() - 1
    }

    pub(super) fn add_blend_animation<'a>(
        self,
        state_machines: &mut [RuntimeStateMachine<'a>],
        animation: RuntimeBlendAnimation<'a>,
    ) -> bool {
        let state = &mut state_machines[self.state_machine_index].layers[self.layer_index].states
            [self.state_index];
        let is_blend_state = state.object.is_some_and(|object| {
            definition_by_type_key(object.type_key)
                .is_some_and(|definition| definition.is_a("BlendState"))
        });
        if !is_blend_state {
            return false;
        }

        state.blend_animations.push(animation);
        true
    }

    /// Pinned `resolve` is infallible. Rust retains stable indices alongside
    /// borrowed objects so downstream transition code can replace C++ pointer
    /// traversal without changing the selected authored occurrences.
    pub(super) fn resolve<'a>(self, state_machines: &mut [RuntimeStateMachine<'a>]) {
        let state = &mut state_machines[self.state_machine_index].layers[self.layer_index].states
            [self.state_index];
        let is_blend_state = state.object.is_some_and(|object| {
            definition_by_type_key(object.type_key)
                .is_some_and(|definition| definition.is_a("BlendState"))
        });
        if !is_blend_state {
            return;
        }

        for transition in &mut state.transitions {
            let is_blend_state_transition = definition_by_type_key(transition.object.type_key)
                .is_some_and(|definition| definition.is_a("BlendStateTransition"));
            if !is_blend_state_transition {
                continue;
            }

            let exit_id = usize::try_from(
                transition
                    .object
                    .uint_property("exitBlendAnimationId")
                    .unwrap_or(u64::from(u32::MAX)),
            )
            .ok();
            if let Some(exit_id) = exit_id.filter(|index| *index < state.blend_animations.len()) {
                let blend_animation = &state.blend_animations[exit_id];
                transition.exit_blend_animation_index = Some(exit_id);
                transition.exit_blend_animation = Some(blend_animation.object);
                transition.exit_animation_index = blend_animation.animation_index;
                transition.exit_animation = blend_animation.animation;
            }
        }
    }
}

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.is_a("LayerState") || definition.is_a("BlendAnimation") {
        return Some(
            imports_successfully(object, definition, context)
                .expect("layer state child is owned by LayerStateImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.is_a("LayerState") {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.is_a("BlendAnimation") {
        return Some(
            context.latest(ImportStackKey::Artboard) && context.latest(ImportStackKey::LayerState),
        );
    }
    definition
        .is_a("LayerState")
        .then(|| context.latest(ImportStackKey::StateMachineLayer))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.is_a("LayerState") {
        context.make_latest(ImportStackKey::LayerState);
        context.latest_layer_state_accepts_blend_animation = definition.is_a("BlendState");
    }
}
