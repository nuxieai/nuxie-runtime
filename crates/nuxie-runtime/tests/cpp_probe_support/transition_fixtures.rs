//! Byte-identical fixture writers extracted from cpp_probe.rs; no runtime behavior.
#![allow(dead_code)]

use super::cpp_probe_support::*;

#[derive(Debug, Clone, Copy)]
pub(super) enum SyntheticInputTransitionKind {
    Bool,
    Number,
    Trigger,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SyntheticTransitionOptions {
    pub(super) duration: u64,
    pub(super) flags: u64,
    pub(super) exit_time: Option<u64>,
    pub(super) any_state_transition: bool,
    pub(super) source_second_frame: u64,
    pub(super) source_second_value: f32,
    pub(super) source_animation_duration: u64,
    pub(super) cubic_transition_interpolator: bool,
    pub(super) elastic_transition_interpolator: bool,
}

impl Default for SyntheticTransitionOptions {
    fn default() -> Self {
        Self {
            duration: 0,
            flags: 0,
            exit_time: None,
            any_state_transition: false,
            source_second_frame: 10,
            source_second_value: 12.0,
            source_animation_duration: 20,
            cubic_transition_interpolator: false,
            elastic_transition_interpolator: false,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum SyntheticRandomBlendSource {
    Blend1D,
    Direct,
}

pub(super) fn synthetic_state_machine_input_transition(
    file_id: u64,
    kind: SyntheticInputTransitionKind,
) -> Vec<u8> {
    synthetic_state_machine_input_transition_with_options(
        file_id,
        kind,
        SyntheticTransitionOptions::default(),
    )
}

pub(super) fn synthetic_state_machine_input_transition_with_duration(
    file_id: u64,
    kind: SyntheticInputTransitionKind,
    transition_duration: u64,
) -> Vec<u8> {
    synthetic_state_machine_input_transition_with_options(
        file_id,
        kind,
        SyntheticTransitionOptions {
            duration: transition_duration,
            ..Default::default()
        },
    )
}

pub(super) fn synthetic_state_machine_input_transition_with_options(
    file_id: u64,
    kind: SyntheticInputTransitionKind,
    transition: SyntheticTransitionOptions,
) -> Vec<u8> {
    synthetic_state_machine_input_transition_with_condition(file_id, kind, kind, 0, transition)
}

pub(super) fn synthetic_state_machine_input_transition_with_condition(
    file_id: u64,
    input_kind: SyntheticInputTransitionKind,
    condition_kind: SyntheticInputTransitionKind,
    condition_input_id: u64,
    transition: SyntheticTransitionOptions,
) -> Vec<u8> {
    synthetic_state_machine_input_transition_with_condition_and_null_slot(
        file_id,
        input_kind,
        condition_kind,
        condition_input_id,
        transition,
        false,
    )
}

pub(super) fn synthetic_state_machine_input_transition_with_condition_and_null_slot(
    file_id: u64,
    input_kind: SyntheticInputTransitionKind,
    condition_kind: SyntheticInputTransitionKind,
    condition_input_id: u64,
    transition: SyntheticTransitionOptions,
    null_input_prefix: bool,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        let animated_local_id = if transition.cubic_transition_interpolator {
            push_object_with_properties(bytes, "CubicEaseInterpolator", |bytes| {
                push_f32_property(bytes, "CubicEaseInterpolator", "x1", 0.2);
                push_f32_property(bytes, "CubicEaseInterpolator", "y1", 0.0);
                push_f32_property(bytes, "CubicEaseInterpolator", "x2", 0.8);
                push_f32_property(bytes, "CubicEaseInterpolator", "y2", 0.0);
            });
            2
        } else if transition.elastic_transition_interpolator {
            push_object_with_properties(bytes, "ElasticInterpolator", |bytes| {
                push_uint_property(bytes, "ElasticInterpolator", "easingValue", 1);
                push_f32_property(bytes, "ElasticInterpolator", "amplitude", 1.2);
                push_f32_property(bytes, "ElasticInterpolator", "period", 0.4);
            });
            2
        } else {
            1
        };
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(
                bytes,
                "LinearAnimation",
                "duration",
                transition.source_animation_duration,
            );
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", animated_local_id);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "x")),
            );
        });
        push_keyframe_double(bytes, 0, 2.0, 1);
        push_keyframe_double(
            bytes,
            transition.source_second_frame,
            transition.source_second_value,
            0,
        );
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", 20);
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", animated_local_id);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "x")),
            );
        });
        push_keyframe_double(bytes, 0, 20.0, 1);
        push_keyframe_double(bytes, 10, 30.0, 0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        if null_input_prefix {
            // The abstract base decodes as a C++ null object. The
            // StateMachineImporter consumes it as an input occurrence.
            push_object_with_properties(bytes, "StateMachineInput", |_| {});
        }
        match input_kind {
            SyntheticInputTransitionKind::Bool => {
                push_object_with_properties(bytes, "StateMachineBool", |bytes| {
                    push_string_property(bytes, "StateMachineBool", "name", "armed");
                });
            }
            SyntheticInputTransitionKind::Number => {
                push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
                    push_string_property(bytes, "StateMachineNumber", "name", "level");
                });
            }
            SyntheticInputTransitionKind::Trigger => {
                push_object_with_properties(bytes, "StateMachineTrigger", |bytes| {
                    push_string_property(bytes, "StateMachineTrigger", "name", "go");
                });
            }
        }
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        if transition.any_state_transition {
            push_object_with_properties(bytes, "StateTransition", |bytes| {
                push_uint_property(bytes, "StateTransition", "stateToId", 3);
                push_synthetic_transition_options(bytes, transition);
            });
            push_synthetic_transition_condition(bytes, condition_kind, condition_input_id);
        }
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
            push_synthetic_transition_options(bytes, transition);
        });
        push_synthetic_transition_condition(bytes, condition_kind, condition_input_id);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn push_synthetic_transition_options(
    bytes: &mut Vec<u8>,
    transition: SyntheticTransitionOptions,
) {
    if transition.duration != 0 {
        push_uint_property(bytes, "StateTransition", "duration", transition.duration);
    }
    if transition.flags != 0 {
        push_uint_property(bytes, "StateTransition", "flags", transition.flags);
    }
    if let Some(exit_time) = transition.exit_time {
        push_uint_property(bytes, "StateTransition", "exitTime", exit_time);
    }
    if transition.cubic_transition_interpolator || transition.elastic_transition_interpolator {
        push_uint_property(bytes, "StateTransition", "interpolatorId", 1);
    }
}

pub(super) fn push_synthetic_transition_condition(
    bytes: &mut Vec<u8>,
    kind: SyntheticInputTransitionKind,
    input_id: u64,
) {
    match kind {
        SyntheticInputTransitionKind::Bool => {
            push_synthetic_bool_transition_condition(bytes, input_id);
        }
        SyntheticInputTransitionKind::Number => {
            push_object_with_properties(bytes, "TransitionNumberCondition", |bytes| {
                push_uint_property(bytes, "TransitionNumberCondition", "inputId", input_id);
                push_uint_property(bytes, "TransitionNumberCondition", "opValue", 5);
                push_f32_property(bytes, "TransitionNumberCondition", "value", 3.0);
            });
        }
        SyntheticInputTransitionKind::Trigger => {
            push_object_with_properties(bytes, "TransitionTriggerCondition", |bytes| {
                push_uint_property(bytes, "TransitionTriggerCondition", "inputId", input_id);
            });
        }
    }
}

pub(super) fn push_synthetic_bool_transition_condition(bytes: &mut Vec<u8>, input_index: u64) {
    push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
        push_uint_property(bytes, "TransitionBoolCondition", "inputId", input_index);
        push_uint_property(bytes, "TransitionBoolCondition", "opValue", 0);
    });
}

pub(super) fn synthetic_state_machine_entry_timed_transition(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", 20);
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 1);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "x")),
            );
        });
        push_keyframe_double(bytes, 0, 20.0, 1);
        push_keyframe_double(bytes, 10, 30.0, 0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
            push_uint_property(bytes, "StateTransition", "duration", 1000);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn push_animation_for_single_node(
    bytes: &mut Vec<u8>,
    target_local_id: u64,
    first_value: f32,
    second_value: f32,
) {
    push_animation_for_single_node_with_duration(
        bytes,
        target_local_id,
        first_value,
        second_value,
        20,
    );
}

pub(super) fn push_animation_for_single_node_with_duration(
    bytes: &mut Vec<u8>,
    target_local_id: u64,
    first_value: f32,
    second_value: f32,
    duration: u64,
) {
    push_object_with_properties(bytes, "LinearAnimation", |bytes| {
        push_uint_property(bytes, "LinearAnimation", "fps", 10);
        push_uint_property(bytes, "LinearAnimation", "duration", duration);
    });
    push_object_with_properties(bytes, "KeyedObject", |bytes| {
        push_uint_property(bytes, "KeyedObject", "objectId", target_local_id);
    });
    push_object_with_properties(bytes, "KeyedProperty", |bytes| {
        push_uint_property(
            bytes,
            "KeyedProperty",
            "propertyKey",
            u64::from(property_key_for_name("Node", "x")),
        );
    });
    push_keyframe_double(bytes, 0, first_value, 1);
    push_keyframe_double(bytes, 10, second_value, 0);
}

pub(super) fn synthetic_state_machine_early_exit_transition(file_id: u64) -> Vec<u8> {
    const ENABLE_EARLY_EXIT: u64 = 1 << 5;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_animation_for_single_node(bytes, 1, 40.0, 50.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "first");
        });
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "second");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
            push_uint_property(bytes, "StateTransition", "duration", 1000);
            push_uint_property(bytes, "StateTransition", "flags", ENABLE_EARLY_EXIT);
        });
        push_synthetic_bool_transition_condition(bytes, 0);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 4);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_blend_state_early_exit(file_id: u64) -> Vec<u8> {
    const ENABLE_EARLY_EXIT: u64 = 1 << 5;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_animation_for_single_node(bytes, 1, 40.0, 50.0);
        push_animation_for_single_node(bytes, 1, 60.0, 70.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "blend");
        });
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "first");
        });
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "second");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "BlendState1DInput", |bytes| {
            push_uint_property(bytes, "BlendState1DInput", "inputId", 0);
        });
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 3);
            push_uint_property(bytes, "BlendStateTransition", "duration", 1000);
            push_uint_property(bytes, "BlendStateTransition", "flags", ENABLE_EARLY_EXIT);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 0);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 4);
        });
        push_synthetic_bool_transition_condition(bytes, 2);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 3);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_random_transition(file_id: u64) -> Vec<u8> {
    const LAYER_STATE_RANDOM: u64 = 1 << 0;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_animation_for_single_node(bytes, 1, 40.0, 50.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "choose");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
            push_uint_property(bytes, "AnimationState", "flags", LAYER_STATE_RANDOM);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
            push_uint_property(bytes, "StateTransition", "randomWeight", 0);
        });
        push_synthetic_bool_transition_condition(bytes, 0);
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 4);
            push_uint_property(bytes, "StateTransition", "randomWeight", 1);
        });
        push_synthetic_bool_transition_condition(bytes, 0);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_blend_state_random_transition(
    file_id: u64,
    source: SyntheticRandomBlendSource,
) -> Vec<u8> {
    const LAYER_STATE_RANDOM: u64 = 1 << 0;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_animation_for_single_node(bytes, 1, 40.0, 50.0);
        push_animation_for_single_node(bytes, 1, 60.0, 70.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "blend");
        });
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "choose");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        match source {
            SyntheticRandomBlendSource::Blend1D => {
                push_object_with_properties(bytes, "BlendState1DInput", |bytes| {
                    push_uint_property(bytes, "BlendState1DInput", "inputId", 0);
                    push_uint_property(bytes, "BlendState1DInput", "flags", LAYER_STATE_RANDOM);
                });
                push_blend_animation_1d(bytes, 0, 0.0);
                push_blend_animation_1d(bytes, 1, 1.0);
            }
            SyntheticRandomBlendSource::Direct => {
                push_object_with_properties(bytes, "BlendStateDirect", |bytes| {
                    push_uint_property(bytes, "BlendStateDirect", "flags", LAYER_STATE_RANDOM);
                });
                push_blend_animation_direct_mix_value(bytes, 0, 25.0);
                push_blend_animation_direct_input(bytes, 1, 0);
            }
        }
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 3);
            push_uint_property(bytes, "BlendStateTransition", "duration", 1000);
            push_uint_property(bytes, "BlendStateTransition", "randomWeight", 0);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 0);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 4);
            push_uint_property(bytes, "BlendStateTransition", "duration", 1000);
            push_uint_property(bytes, "BlendStateTransition", "randomWeight", 1);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 0);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 3);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_animation_state(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", 20);
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 1);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "x")),
            );
        });
        push_keyframe_double(bytes, 0, 2.0, 1);
        push_keyframe_double(bytes, 10, 12.0, 0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_direct_blend_state_transition(file_id: u64) -> Vec<u8> {
    const ENABLE_EXIT_TIME: u64 = 1 << 2;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_animation_for_single_node(bytes, 1, 40.0, 50.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "direct");
        });
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "go");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "BlendStateDirect", |_| {});
        push_blend_animation_direct_mix_value(bytes, 0, 25.0);
        push_blend_animation_direct_input(bytes, 1, 0);
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 3);
            push_uint_property(bytes, "BlendStateTransition", "duration", 1000);
            push_uint_property(bytes, "BlendStateTransition", "exitTime", 1000);
            push_uint_property(bytes, "BlendStateTransition", "flags", ENABLE_EXIT_TIME);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 1);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

#[derive(Clone, Copy)]
enum BlendTransitionFixture {
    ExitTime,
    Reset,
    PercentageDuration,
    PercentageExitTime,
    PauseOnExit,
}

fn synthetic_state_machine_blend_transition_fixture(
    file_id: u64,
    fixture: BlendTransitionFixture,
) -> Vec<u8> {
    const DURATION_IS_PERCENTAGE: u64 = 1 << 1;
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const EXIT_TIME_IS_PERCENTAGE: u64 = 1 << 3;
    const PAUSE_ON_EXIT: u64 = 1 << 4;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        if matches!(fixture, BlendTransitionFixture::PercentageExitTime) {
            push_animation_for_single_node_with_duration(bytes, 1, 2.0, 12.0, 10);
            push_animation_for_single_node_with_duration(bytes, 1, 20.0, 30.0, 20);
        } else {
            push_animation_for_single_node(bytes, 1, 2.0, 12.0);
            push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        }
        push_animation_for_single_node(bytes, 1, 40.0, 50.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "blend");
        });
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "go");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "BlendState1DInput", |bytes| {
            push_uint_property(bytes, "BlendState1DInput", "inputId", 0);
        });
        push_blend_animation_1d(bytes, 0, 0.0);
        if !matches!(fixture, BlendTransitionFixture::ExitTime) {
            push_blend_animation_1d(bytes, 1, 1.0);
        }
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 3);
            let (duration, exit_time, flags, exit_animation) = match fixture {
                BlendTransitionFixture::ExitTime | BlendTransitionFixture::Reset => {
                    (1000, Some(1000), ENABLE_EXIT_TIME, 0)
                }
                BlendTransitionFixture::PercentageDuration => (50, None, DURATION_IS_PERCENTAGE, 1),
                BlendTransitionFixture::PercentageExitTime => (
                    1000,
                    Some(75),
                    ENABLE_EXIT_TIME | EXIT_TIME_IS_PERCENTAGE,
                    1,
                ),
                BlendTransitionFixture::PauseOnExit => {
                    (1000, Some(500), ENABLE_EXIT_TIME | PAUSE_ON_EXIT, 1)
                }
            };
            push_uint_property(bytes, "BlendStateTransition", "duration", duration);
            if let Some(exit_time) = exit_time {
                push_uint_property(bytes, "BlendStateTransition", "exitTime", exit_time);
            }
            push_uint_property(bytes, "BlendStateTransition", "flags", flags);
            push_uint_property(
                bytes,
                "BlendStateTransition",
                "exitBlendAnimationId",
                exit_animation,
            );
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(
                bytes,
                "AnimationState",
                "animationId",
                if matches!(fixture, BlendTransitionFixture::ExitTime) {
                    1
                } else {
                    2
                },
            );
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_blend_state_transition(file_id: u64) -> Vec<u8> {
    synthetic_state_machine_blend_transition_fixture(file_id, BlendTransitionFixture::ExitTime)
}

pub(super) fn synthetic_state_machine_blend_state_transition_reset(file_id: u64) -> Vec<u8> {
    synthetic_state_machine_blend_transition_fixture(file_id, BlendTransitionFixture::Reset)
}

pub(super) fn synthetic_state_machine_blend_state_percentage_duration(file_id: u64) -> Vec<u8> {
    synthetic_state_machine_blend_transition_fixture(
        file_id,
        BlendTransitionFixture::PercentageDuration,
    )
}

pub(super) fn synthetic_state_machine_blend_state_percentage_exit_time(file_id: u64) -> Vec<u8> {
    synthetic_state_machine_blend_transition_fixture(
        file_id,
        BlendTransitionFixture::PercentageExitTime,
    )
}

pub(super) fn synthetic_state_machine_blend_state_pause_on_exit(file_id: u64) -> Vec<u8> {
    synthetic_state_machine_blend_transition_fixture(file_id, BlendTransitionFixture::PauseOnExit)
}

pub(super) fn synthetic_state_machine_duplicate_system_states(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 4);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_generic_system_state(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "LayerState", |_| {});
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_fl_c5_layer_state_queries(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "enabled");
        });

        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        push_synthetic_bool_transition_condition(bytes, 0);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 4);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});

        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});

        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        push_synthetic_bool_transition_condition(bytes, 0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn push_blend_animation_1d(bytes: &mut Vec<u8>, animation_id: u64, value: f32) {
    push_object_with_properties(bytes, "BlendAnimation1D", |bytes| {
        push_uint_property(bytes, "BlendAnimation1D", "animationId", animation_id);
        push_f32_property(bytes, "BlendAnimation1D", "value", value);
    });
}

pub(super) fn push_blend_animation_direct_mix_value(
    bytes: &mut Vec<u8>,
    animation_id: u64,
    mix_value: f32,
) {
    push_object_with_properties(bytes, "BlendAnimationDirect", |bytes| {
        push_uint_property(bytes, "BlendAnimationDirect", "animationId", animation_id);
        push_uint_property(bytes, "BlendAnimationDirect", "blendSource", 1);
        push_f32_property(bytes, "BlendAnimationDirect", "mixValue", mix_value);
    });
}

pub(super) fn push_blend_animation_direct_input(
    bytes: &mut Vec<u8>,
    animation_id: u64,
    input_id: u64,
) {
    push_object_with_properties(bytes, "BlendAnimationDirect", |bytes| {
        push_uint_property(bytes, "BlendAnimationDirect", "animationId", animation_id);
        push_uint_property(bytes, "BlendAnimationDirect", "inputId", input_id);
    });
}

fn push_string_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &str) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

pub(super) fn synthetic_state_machine_blend_state_1d_input(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "blend");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "BlendState1DInput", |bytes| {
            push_uint_property(bytes, "BlendState1DInput", "inputId", 0);
        });
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_empty_baseline_reset(file_id: u64) -> Vec<u8> {
    const LAYER_STATE_RESET: u64 = 1 << 1;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 7.0, 3.0, 1.0, 1.0, 1.0);
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", 20);
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 1);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "y")),
            );
        });
        push_keyframe_double(bytes, 0, 3.0, 1);
        push_keyframe_double(bytes, 10, 13.0, 0);
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "x")),
            );
        });
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", 20);
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 1);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "y")),
            );
        });
        push_keyframe_double(bytes, 0, 3.0, 1);
        push_keyframe_double(bytes, 10, 13.0, 0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineNumber", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "BlendState1DInput", |bytes| {
            push_uint_property(bytes, "BlendState1DInput", "inputId", 0);
            push_uint_property(bytes, "BlendState1DInput", "flags", LAYER_STATE_RESET);
        });
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_blend_state_direct(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "direct");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "BlendStateDirect", |_| {});
        push_blend_animation_direct_mix_value(bytes, 0, 25.0);
        push_blend_animation_direct_input(bytes, 1, 0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_direct_nan_mix(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 7.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "BlendStateDirect", |_| {});
        push_blend_animation_direct_mix_value(bytes, 0, f32::NAN);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_blend_state_1d_missing_bindable_instance(
    file_id: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        // A bindable without a DataBind has no state-machine instance. C++
        // keeps the 1D blend state and evaluates it at the default value zero.
        push_object_with_properties(bytes, "BindablePropertyNumber", |_| {});
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_direct_missing_bindable_instance(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        // With no DataBind there is no number instance. Direct blends retain
        // their previous mix (initially zero) instead of dropping the animation.
        push_object_with_properties(bytes, "BindablePropertyNumber", |_| {});
        push_object_with_properties(bytes, "BlendStateDirect", |_| {});
        push_blend_animation_direct_bindable(bytes, 0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn push_blend_animation_direct_bindable(bytes: &mut Vec<u8>, animation_id: u64) {
    push_object_with_properties(bytes, "BlendAnimationDirect", |bytes| {
        push_uint_property(bytes, "BlendAnimationDirect", "animationId", animation_id);
        push_uint_property(bytes, "BlendAnimationDirect", "blendSource", 2);
    });
}

#[derive(Clone, Copy, Debug)]
pub(super) enum SyntheticCrossArtboardBlendKind {
    Blend1D,
    BlendDirect,
}

pub(super) fn synthetic_fl_c5_state_machine_definition(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "StateMachine", |bytes| {
            push_string_property(bytes, "StateMachine", "name", "definition");
        });
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "duplicate");
        });
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "duplicate");
        });
        push_object_with_properties(bytes, "StateMachineTrigger", |bytes| {
            push_string_property(bytes, "StateMachineTrigger", "name", "Case");
        });
        for name in ["duplicate", "duplicate", "Case"] {
            push_object_with_properties(bytes, "StateMachineLayer", |bytes| {
                push_string_property(bytes, "StateMachineLayer", "name", name);
            });
            push_object_with_properties(bytes, "AnyState", |_| {});
            push_object_with_properties(bytes, "EntryState", |_| {});
            push_object_with_properties(bytes, "ExitState", |_| {});
        }
        // No listener input type makes the first authored listener inert. Its
        // generated targetId remains u32::MAX. The second occurrence is valid
        // and must remain at authored index one.
        push_object_with_properties(bytes, "StateMachineListener", |_| {});
        push_object_with_properties(bytes, "StateMachineListenerSingle", |bytes| {
            push_uint_property(bytes, "StateMachineListenerSingle", "targetId", 0);
        });
    })
}

pub(super) fn synthetic_fl_c5_state_machine_definition_null_hole(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "StateMachine", |bytes| {
            push_string_property(bytes, "StateMachine", "name", "definition");
        });
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "x");
        });
        push_object_with_properties(bytes, "StateMachineInput", |bytes| {
            push_string_property(bytes, "StateMachineInput", "name", "hole");
        });
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "x");
        });
        push_object_with_properties(bytes, "StateMachineTrigger", |bytes| {
            push_string_property(bytes, "StateMachineTrigger", "name", "x");
        });
        for name in ["duplicate", "duplicate", "Case"] {
            push_object_with_properties(bytes, "StateMachineLayer", |bytes| {
                push_string_property(bytes, "StateMachineLayer", "name", name);
            });
            push_object_with_properties(bytes, "AnyState", |_| {});
            push_object_with_properties(bytes, "EntryState", |_| {});
            push_object_with_properties(bytes, "ExitState", |_| {});
        }
        push_object_with_properties(bytes, "StateMachineListener", |_| {});
        push_object_with_properties(bytes, "StateMachineListenerSingle", |bytes| {
            push_uint_property(bytes, "StateMachineListenerSingle", "targetId", 0);
        });
        // The binary replay identifies StateMachine-owned DataBinds through
        // their latest bindable target. The focused C++ seam can call
        // StateMachineImporter::addDataBind directly, so this inert carrier
        // exists only to express the same ownership in a serialized stream.
        push_object_with_properties(bytes, "BindablePropertyNumber", |_| {});
        for _ in 0..2 {
            push_object_with_properties(bytes, "DataBind", |bytes| {
                push_uint_property(bytes, "DataBind", "propertyKey", 586);
            });
        }
    })
}

pub(super) fn synthetic_fl_c5_malformed_listener(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "StateMachine", |_| {});
        // No listener input type makes the first authored listener inert. Its
        // generated targetId remains u32::MAX. The second occurrence is valid
        // and must remain at authored index one.
        push_object_with_properties(bytes, "StateMachineListener", |_| {});
        push_object_with_properties(bytes, "StateMachineListenerSingle", |bytes| {
            push_uint_property(bytes, "StateMachineListenerSingle", "targetId", 0);
        });
    })
}

pub(super) fn synthetic_fl_c5_typed_named_inputs(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "x");
        });
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "x");
        });
        push_object_with_properties(bytes, "StateMachineTrigger", |bytes| {
            push_string_property(bytes, "StateMachineTrigger", "name", "x");
        });
    })
}

pub(super) fn synthetic_fl_c5_empty_state_machine(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "StateMachine", |_| {});
    })
}

pub(super) fn synthetic_cross_artboard_blend_definition_owner(
    file_id: u64,
    kind: SyntheticCrossArtboardBlendKind,
    animations: [(f32, f32); 2],
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        // Keep the application target baseline identical between owner and
        // caller so the differential isolates definition ownership even when
        // sequential blend mixing retains part of the target's prior value.
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        for (from, to) in animations {
            push_animation_for_single_node(bytes, 1, from, to);
        }
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "blend");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        match kind {
            SyntheticCrossArtboardBlendKind::Blend1D => {
                push_object_with_properties(bytes, "BlendState1DInput", |bytes| {
                    push_uint_property(bytes, "BlendState1DInput", "inputId", 0);
                });
                push_blend_animation_1d(bytes, 0, 0.0);
                push_blend_animation_1d(bytes, 1, 1.0);
            }
            SyntheticCrossArtboardBlendKind::BlendDirect => {
                push_object_with_properties(bytes, "BlendStateDirect", |_| {});
                // A full first mix removes target-baseline history from the
                // clone/remount comparison while the second animation still
                // proves that both retained definitions belong to owner A.
                push_blend_animation_direct_mix_value(bytes, 0, 100.0);
                push_blend_animation_direct_input(bytes, 1, 0);
            }
        }
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_weighted_random_transition(
    file_id: u64,
    first_weight: u64,
    second_weight: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_animation_for_single_node(bytes, 1, 40.0, 50.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
            push_uint_property(bytes, "LayerState", "flags", 1);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
            push_uint_property(bytes, "StateTransition", "randomWeight", first_weight);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 4);
            push_uint_property(bytes, "StateTransition", "randomWeight", second_weight);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_weighted_random_wait_then_select(file_id: u64) -> Vec<u8> {
    const LAYER_STATE_RANDOM: u64 = 1 << 0;
    const ENABLE_EXIT_TIME: u64 = 1 << 2;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_animation_for_single_node(bytes, 1, 40.0, 50.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
            push_uint_property(bytes, "LayerState", "flags", LAYER_STATE_RANDOM);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
            push_uint_property(bytes, "StateTransition", "randomWeight", 1);
            push_uint_property(bytes, "StateTransition", "flags", ENABLE_EXIT_TIME);
            push_uint_property(bytes, "StateTransition", "exitTime", 1000);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 4);
            push_uint_property(bytes, "StateTransition", "randomWeight", 1);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_serial_layer_entry_initialization(file_id: u64) -> Vec<u8> {
    const STATE_AT_START: u64 = 2 << 1;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineNumber", |bytes| {
            push_string_property(bytes, "StateMachineNumber", "name", "level");
        });

        push_object_with_properties(bytes, "StateMachineLayer", |bytes| {
            push_string_property(bytes, "StateMachineLayer", "name", "producer");
        });
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "ListenerNumberChange", |bytes| {
            push_uint_property(bytes, "ListenerNumberChange", "inputId", 0);
            push_uint_property(bytes, "ListenerNumberChange", "flags", STATE_AT_START);
            push_f32_property(bytes, "ListenerNumberChange", "value", 7.0);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});

        push_object_with_properties(bytes, "StateMachineLayer", |bytes| {
            push_string_property(bytes, "StateMachineLayer", "name", "consumer");
        });
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_synthetic_transition_condition(bytes, SyntheticInputTransitionKind::Number, 0);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_transition_interruption(file_id: u64) -> Vec<u8> {
    const ENABLE_EARLY_EXIT: u64 = 1 << 5;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_animation_for_single_node(bytes, 1, 40.0, 50.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "first");
        });
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "second");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
            push_uint_property(bytes, "StateTransition", "duration", 1000);
            push_uint_property(bytes, "StateTransition", "flags", ENABLE_EARLY_EXIT);
        });
        push_synthetic_bool_transition_condition(bytes, 0);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 4);
            push_uint_property(bytes, "StateTransition", "duration", 1000);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn synthetic_state_machine_same_state_transition(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineBool", |bytes| {
            push_string_property(bytes, "StateMachineBool", "name", "go");
        });
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_synthetic_bool_transition_condition(bytes, 0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}
