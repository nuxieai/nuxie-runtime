use super::system_state_instance::RuntimeSystemStateInstance;
use super::*;

/// One mutable occurrence of an immutable `RuntimeLayerState` definition.
///
/// This corresponds to pinned C++ `StateInstance`: the definition handle is
/// retained once, and animation/blend mutation lives inside the occurrence
/// instead of parallel fields on the layer.
#[derive(Debug, Clone)]
pub(super) struct RuntimeStateInstance {
    state_index: usize,
    kind: RuntimeStateInstanceKind,
}

#[derive(Debug, Clone)]
enum RuntimeStateInstanceKind {
    System(RuntimeSystemStateInstance),
    Animation {
        animation: LinearAnimationInstance,
        keep_going: bool,
    },
    Blend1D(BlendState1DInstance),
    BlendDirect(BlendStateDirectInstance),
}

impl RuntimeStateInstance {
    pub(super) fn make(
        layer: &RuntimeStateMachineLayer,
        state_index: usize,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) -> Option<Self> {
        let state = layer.states.get(state_index)?;
        let kind = if let Some(definition) = state.blend_state_1d.as_ref() {
            let mut occurrence =
                BlendState1DInstance::new(definition, artboard, state.resets_blend_values());
            occurrence.advance(definition, artboard, inputs, bindable_numbers, 0.0);
            RuntimeStateInstanceKind::Blend1D(occurrence)
        } else if let Some(definition) = state.blend_state_direct.as_ref() {
            let mut occurrence = BlendStateDirectInstance::new(definition, artboard);
            occurrence.advance(definition, artboard, inputs, bindable_numbers, 0.0);
            RuntimeStateInstanceKind::BlendDirect(occurrence)
        } else if let Some(handle) = state.animation {
            let definition = artboard.linear_animation_definition(handle)?;
            let mut occurrence = LinearAnimationInstance::new(handle, definition, state.speed);
            let keep_going = occurrence.advance(definition, 0.0);
            RuntimeStateInstanceKind::Animation {
                animation: occurrence,
                keep_going,
            }
        } else {
            RuntimeStateInstanceKind::System(RuntimeSystemStateInstance)
        };
        Some(Self { state_index, kind })
    }

    pub(super) fn state_index(&self) -> usize {
        self.state_index
    }

    pub(super) fn state<'a>(
        &self,
        layer: &'a RuntimeStateMachineLayer,
    ) -> Option<&'a RuntimeLayerState> {
        layer.states.get(self.state_index)
    }

    pub(super) fn is_same_definition(&self, state_index: usize) -> bool {
        self.state_index == state_index
    }

    pub(super) fn keep_going(&self) -> bool {
        match &self.kind {
            RuntimeStateInstanceKind::System(_) => false,
            RuntimeStateInstanceKind::Animation { keep_going, .. } => *keep_going,
            RuntimeStateInstanceKind::Blend1D(_) | RuntimeStateInstanceKind::BlendDirect(_) => true,
        }
    }

    pub(super) fn advance(
        &mut self,
        layer: &RuntimeStateMachineLayer,
        artboard: &mut ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        let Some(state) = layer.states.get(self.state_index) else {
            return false;
        };
        match &mut self.kind {
            RuntimeStateInstanceKind::System(instance) => instance.advance(),
            RuntimeStateInstanceKind::Animation {
                animation,
                keep_going,
            } => {
                *keep_going = artboard.advance_linear_animation_instance_with_events(
                    animation,
                    elapsed_seconds * state.speed,
                    reported_events,
                );
                *keep_going
            }
            RuntimeStateInstanceKind::Blend1D(instance) => {
                let Some(definition) = state.blend_state_1d.as_ref() else {
                    return false;
                };
                instance.advance_with_events(
                    definition,
                    artboard,
                    inputs,
                    bindable_numbers,
                    elapsed_seconds,
                    reported_events,
                )
            }
            RuntimeStateInstanceKind::BlendDirect(instance) => {
                let Some(definition) = state.blend_state_direct.as_ref() else {
                    return false;
                };
                instance.advance_with_events(
                    definition,
                    artboard,
                    inputs,
                    bindable_numbers,
                    elapsed_seconds,
                    reported_events,
                )
            }
        }
    }

    pub(super) fn apply(&self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        match &self.kind {
            RuntimeStateInstanceKind::System(instance) => instance.apply(),
            RuntimeStateInstanceKind::Animation { animation, .. } => animation.apply(artboard, mix),
            RuntimeStateInstanceKind::Blend1D(instance) => instance.apply(artboard, mix),
            RuntimeStateInstanceKind::BlendDirect(instance) => instance.apply(artboard, mix),
        }
    }

    pub(super) fn plain_animation(&self) -> Option<&LinearAnimationInstance> {
        match &self.kind {
            RuntimeStateInstanceKind::Animation { animation, .. } => Some(animation),
            _ => None,
        }
    }

    pub(super) fn plain_animation_mut(&mut self) -> Option<&mut LinearAnimationInstance> {
        match &mut self.kind {
            RuntimeStateInstanceKind::Animation { animation, .. } => Some(animation),
            _ => None,
        }
    }

    pub(super) fn transition_animation(
        &self,
        blend_animation_index: Option<usize>,
    ) -> Option<&LinearAnimationInstance> {
        match &self.kind {
            RuntimeStateInstanceKind::Animation { animation, .. } => Some(animation),
            RuntimeStateInstanceKind::Blend1D(instance) => {
                instance.animation_instance(blend_animation_index?)
            }
            RuntimeStateInstanceKind::BlendDirect(instance) => {
                instance.animation_instance(blend_animation_index?)
            }
            RuntimeStateInstanceKind::System(_) => None,
        }
    }

    pub(super) fn spilled_time(&self) -> f32 {
        self.plain_animation()
            .map(LinearAnimationInstance::spilled_time)
            .unwrap_or(0.0)
    }

    pub(super) fn clear_spilled_time(&mut self) {
        if let Some(animation) = self.plain_animation_mut() {
            animation.clear_spilled_time();
        }
    }

    pub(super) fn prepare_key_frame_data_binds(
        &mut self,
        graphs: &[Option<crate::RuntimeDataBindGraph>],
    ) {
        self.for_each_animation_instance_mut(|instance| {
            let prototype = graphs
                .get(instance.animation_index())
                .and_then(Option::as_ref);
            instance.prepare_key_frame_data_binds(prototype);
        });
    }

    pub(super) fn for_each_animation_instance_mut(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        match &mut self.kind {
            RuntimeStateInstanceKind::System(_) => {}
            RuntimeStateInstanceKind::Animation { animation, .. } => callback(animation),
            RuntimeStateInstanceKind::Blend1D(instance) => {
                instance.for_each_animation_instance_mut(callback)
            }
            RuntimeStateInstanceKind::BlendDirect(instance) => {
                instance.for_each_animation_instance_mut(callback)
            }
        }
    }
}
