use rive_binary::{RuntimeFile, read_runtime_file};
use rive_graph::GraphFile;
use rive_runtime::{
    ArtboardInstance, ComponentDirt, Mat2D, RuntimeComponent, RuntimeDrawCommandKind,
    RuntimeFeatherState, RuntimeGradientStop, RuntimePathCommand, RuntimeShapePaintKind,
    RuntimeShapePaintPathKind, RuntimeShapePaintState, StateMachineInputKind, TransformProperty,
};
use rive_schema::definition_by_name;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn default_probe_path() -> PathBuf {
    let os = match std::env::consts::OS {
        "macos" => "macosx",
        other => other,
    };

    repo_root()
        .join("tools/cpp-probe/build")
        .join(os)
        .join("bin/debug/rive_cpp_probe")
}

fn probe_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("RIVE_CPP_PROBE") {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            return Some(path);
        }
        return Some(repo_root().join(path));
    }

    let path = default_probe_path();
    path.exists().then_some(path)
}

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn type_key_for_name(type_name: &str) -> u16 {
    definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
        .type_key
        .int
}

fn property_key_for_name(type_name: &str, property_name: &str) -> u16 {
    let definition = definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
    if let Some(property) = definition
        .properties
        .iter()
        .find(|property| property.name == property_name)
    {
        return property.key.int;
    }

    for ancestor in definition.ancestors {
        let ancestor = definition_by_name(ancestor)
            .unwrap_or_else(|| panic!("missing ancestor schema definition {ancestor}"));
        if let Some(property) = ancestor
            .properties
            .iter()
            .find(|property| property.name == property_name)
        {
            return property.key.int;
        }
    }

    panic!("missing property {type_name}.{property_name}");
}

fn push_object_with_properties(
    bytes: &mut Vec<u8>,
    type_name: &str,
    properties: impl FnOnce(&mut Vec<u8>),
) {
    push_var_uint(bytes, u64::from(type_key_for_name(type_name)));
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u64) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    push_var_uint(bytes, value);
}

fn push_string_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &str) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn push_f32_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: f32) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_bool_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: bool) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    bytes.push(u8::from(value));
}

fn push_color_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u32) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn synthetic_runtime_file(file_id: u64, object_stream: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIVE");
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, file_id);
    push_var_uint(&mut bytes, 0);
    object_stream(&mut bytes);
    bytes
}

fn push_transform_node(
    bytes: &mut Vec<u8>,
    parent_id: u64,
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
    opacity: f32,
) {
    push_object_with_properties(bytes, "Node", |bytes| {
        push_uint_property(bytes, "Node", "parentId", parent_id);
        push_f32_property(bytes, "Node", "x", x);
        push_f32_property(bytes, "Node", "y", y);
        push_f32_property(bytes, "Node", "scaleX", scale_x);
        push_f32_property(bytes, "Node", "scaleY", scale_y);
        push_f32_property(bytes, "Node", "opacity", opacity);
    });
}

fn synthetic_transform_hierarchy() -> Vec<u8> {
    synthetic_runtime_file(8101, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 4.0, 5.0, 0.5);
        push_transform_node(bytes, 1, 7.0, 11.0, 2.0, 3.0, 0.25);
    })
}

fn push_keyframe_double(bytes: &mut Vec<u8>, frame: u64, value: f32, interpolation_type: u64) {
    push_object_with_properties(bytes, "KeyFrameDouble", |bytes| {
        push_uint_property(bytes, "KeyFrameDouble", "frame", frame);
        push_uint_property(
            bytes,
            "KeyFrameDouble",
            "interpolationType",
            interpolation_type,
        );
        push_f32_property(bytes, "KeyFrameDouble", "value", value);
    });
}

#[derive(Debug, Clone, Copy)]
struct LinearAnimationFixtureOptions {
    duration: u64,
    loop_value: u64,
    speed: f32,
    enable_work_area: bool,
    work_start: u64,
    work_end: u64,
    quantize: bool,
}

impl Default for LinearAnimationFixtureOptions {
    fn default() -> Self {
        Self {
            duration: 20,
            loop_value: 0,
            speed: 1.0,
            enable_work_area: false,
            work_start: 0,
            work_end: 0,
            quantize: false,
        }
    }
}

fn synthetic_linear_animation(
    file_id: u64,
    first_frame: u64,
    first_value: f32,
    second_frame: u64,
    second_value: f32,
    first_interpolation_type: u64,
    quantize: bool,
) -> Vec<u8> {
    synthetic_linear_animation_with_options(
        file_id,
        first_frame,
        first_value,
        second_frame,
        second_value,
        first_interpolation_type,
        LinearAnimationFixtureOptions {
            quantize,
            ..Default::default()
        },
    )
}

fn synthetic_linear_animation_with_options(
    file_id: u64,
    first_frame: u64,
    first_value: f32,
    second_frame: u64,
    second_value: f32,
    first_interpolation_type: u64,
    options: LinearAnimationFixtureOptions,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", options.duration);
            push_f32_property(bytes, "LinearAnimation", "speed", options.speed);
            push_uint_property(bytes, "LinearAnimation", "loopValue", options.loop_value);
            push_uint_property(bytes, "LinearAnimation", "workStart", options.work_start);
            push_uint_property(bytes, "LinearAnimation", "workEnd", options.work_end);
            if options.enable_work_area {
                push_bool_property(bytes, "LinearAnimation", "enableWorkArea", true);
            }
            if options.quantize {
                push_bool_property(bytes, "LinearAnimation", "quantize", true);
            }
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
        push_keyframe_double(bytes, first_frame, first_value, first_interpolation_type);
        push_keyframe_double(bytes, second_frame, second_value, 0);
    })
}

fn synthetic_state_machine_animation_state(file_id: u64) -> Vec<u8> {
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

#[derive(Debug, Clone, Copy)]
enum SyntheticInputTransitionKind {
    Bool,
    Number,
    Trigger,
}

#[derive(Debug, Clone, Copy)]
struct SyntheticTransitionOptions {
    duration: u64,
    flags: u64,
    exit_time: Option<u64>,
    any_state_transition: bool,
    source_second_frame: u64,
    source_second_value: f32,
    cubic_transition_interpolator: bool,
    elastic_transition_interpolator: bool,
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
            cubic_transition_interpolator: false,
            elastic_transition_interpolator: false,
        }
    }
}

fn synthetic_state_machine_input_transition(
    file_id: u64,
    kind: SyntheticInputTransitionKind,
) -> Vec<u8> {
    synthetic_state_machine_input_transition_with_options(
        file_id,
        kind,
        SyntheticTransitionOptions::default(),
    )
}

fn synthetic_state_machine_input_transition_with_duration(
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

fn synthetic_state_machine_input_transition_with_options(
    file_id: u64,
    kind: SyntheticInputTransitionKind,
    transition: SyntheticTransitionOptions,
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
        match kind {
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
            push_synthetic_transition_condition(bytes, kind);
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
        push_synthetic_transition_condition(bytes, kind);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn push_synthetic_transition_options(bytes: &mut Vec<u8>, transition: SyntheticTransitionOptions) {
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

fn push_synthetic_transition_condition(bytes: &mut Vec<u8>, kind: SyntheticInputTransitionKind) {
    match kind {
        SyntheticInputTransitionKind::Bool => {
            push_synthetic_bool_transition_condition(bytes, 0);
        }
        SyntheticInputTransitionKind::Number => {
            push_object_with_properties(bytes, "TransitionNumberCondition", |bytes| {
                push_uint_property(bytes, "TransitionNumberCondition", "inputId", 0);
                push_uint_property(bytes, "TransitionNumberCondition", "opValue", 5);
                push_f32_property(bytes, "TransitionNumberCondition", "value", 3.0);
            });
        }
        SyntheticInputTransitionKind::Trigger => {
            push_object_with_properties(bytes, "TransitionTriggerCondition", |bytes| {
                push_uint_property(bytes, "TransitionTriggerCondition", "inputId", 0);
            });
        }
    }
}

fn push_synthetic_bool_transition_condition(bytes: &mut Vec<u8>, input_index: u64) {
    push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
        push_uint_property(bytes, "TransitionBoolCondition", "inputId", input_index);
        push_uint_property(bytes, "TransitionBoolCondition", "opValue", 0);
    });
}

fn synthetic_state_machine_bool_transition(file_id: u64) -> Vec<u8> {
    synthetic_state_machine_input_transition(file_id, SyntheticInputTransitionKind::Bool)
}

fn push_animation_for_single_node(
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

fn push_animation_for_single_node_with_duration(
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

fn synthetic_state_machine_early_exit_transition(file_id: u64) -> Vec<u8> {
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

fn synthetic_state_machine_blend_state_early_exit(file_id: u64) -> Vec<u8> {
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

fn synthetic_state_machine_random_transition(file_id: u64) -> Vec<u8> {
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

#[derive(Clone, Copy)]
enum SyntheticRandomBlendSource {
    Blend1D,
    Direct,
}

fn synthetic_state_machine_blend_state_random_transition(
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

fn synthetic_state_machine_fire_events(file_id: u64) -> Vec<u8> {
    const AT_START: u64 = 0;
    const AT_END: u64 = 1;
    const SOURCE_START_EVENT: u64 = 2;
    const SOURCE_END_EVENT: u64 = 3;
    const TARGET_START_EVENT: u64 = 4;
    const TRANSITION_START_EVENT: u64 = 5;
    const TRANSITION_END_EVENT: u64 = 6;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        for name in [
            "source-start",
            "source-end",
            "target-start",
            "transition-start",
            "transition-end",
        ] {
            push_object_with_properties(bytes, "Event", |bytes| {
                push_string_property(bytes, "Event", "name", name);
            });
        }
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
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
        push_state_machine_fire_event(bytes, SOURCE_START_EVENT, AT_START);
        push_state_machine_fire_event(bytes, SOURCE_END_EVENT, AT_END);
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        push_synthetic_bool_transition_condition(bytes, 0);
        push_state_machine_fire_event(bytes, TRANSITION_START_EVENT, AT_START);
        push_state_machine_fire_event(bytes, TRANSITION_END_EVENT, AT_END);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_state_machine_fire_event(bytes, TARGET_START_EVENT, AT_START);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn push_state_machine_fire_event(bytes: &mut Vec<u8>, event_local_id: u64, occurs_value: u64) {
    push_object_with_properties(bytes, "StateMachineFireEvent", |bytes| {
        push_uint_property(bytes, "StateMachineFireEvent", "eventId", event_local_id);
        if occurs_value != 0 {
            push_uint_property(bytes, "StateMachineFireEvent", "occursValue", occurs_value);
        }
    });
}

fn synthetic_state_machine_scheduled_listener_fire_events(file_id: u64) -> Vec<u8> {
    const STATE_AT_START: u64 = 2 << 1;
    const STATE_AT_END: u64 = (2 << 1) | 1;
    const TRANSITION_AT_START: u64 = 1 << 1;
    const TRANSITION_AT_END: u64 = (1 << 1) | 1;
    const SOURCE_START_EVENT: u64 = 2;
    const SOURCE_END_EVENT: u64 = 3;
    const TARGET_START_EVENT: u64 = 4;
    const TRANSITION_START_EVENT: u64 = 5;
    const TRANSITION_END_EVENT: u64 = 6;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        for name in [
            "listener-source-start",
            "listener-source-end",
            "listener-target-start",
            "listener-transition-start",
            "listener-transition-end",
        ] {
            push_object_with_properties(bytes, "Event", |bytes| {
                push_string_property(bytes, "Event", "name", name);
            });
        }
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
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
        push_scheduled_listener_fire_event(bytes, SOURCE_START_EVENT, STATE_AT_START);
        push_scheduled_listener_fire_event(bytes, SOURCE_END_EVENT, STATE_AT_END);
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        push_synthetic_bool_transition_condition(bytes, 0);
        push_scheduled_listener_fire_event(bytes, TRANSITION_START_EVENT, TRANSITION_AT_START);
        push_scheduled_listener_fire_event(bytes, TRANSITION_END_EVENT, TRANSITION_AT_END);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_scheduled_listener_fire_event(bytes, TARGET_START_EVENT, STATE_AT_START);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn push_scheduled_listener_fire_event(bytes: &mut Vec<u8>, event_local_id: u64, flags: u64) {
    push_object_with_properties(bytes, "ListenerFireEvent", |bytes| {
        push_uint_property(bytes, "ListenerFireEvent", "eventId", event_local_id);
        push_uint_property(bytes, "ListenerFireEvent", "flags", flags);
    });
}

fn synthetic_state_machine_scheduled_listener_input_change(
    file_id: u64,
    kind: SyntheticInputTransitionKind,
) -> Vec<u8> {
    const STATE_AT_START: u64 = 2 << 1;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
        push_object_with_properties(bytes, "StateMachine", |_| {});
        match kind {
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
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 2);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_scheduled_listener_input_change(bytes, kind, 0, STATE_AT_START);
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        push_synthetic_transition_condition(bytes, kind);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn push_scheduled_listener_input_change(
    bytes: &mut Vec<u8>,
    kind: SyntheticInputTransitionKind,
    input_index: u64,
    flags: u64,
) {
    match kind {
        SyntheticInputTransitionKind::Bool => {
            push_object_with_properties(bytes, "ListenerBoolChange", |bytes| {
                push_uint_property(bytes, "ListenerBoolChange", "inputId", input_index);
                push_uint_property(bytes, "ListenerBoolChange", "flags", flags);
                push_uint_property(bytes, "ListenerBoolChange", "value", 1);
            });
        }
        SyntheticInputTransitionKind::Number => {
            push_object_with_properties(bytes, "ListenerNumberChange", |bytes| {
                push_uint_property(bytes, "ListenerNumberChange", "inputId", input_index);
                push_uint_property(bytes, "ListenerNumberChange", "flags", flags);
                push_f32_property(bytes, "ListenerNumberChange", "value", 4.0);
            });
        }
        SyntheticInputTransitionKind::Trigger => {
            push_object_with_properties(bytes, "ListenerTriggerChange", |bytes| {
                push_uint_property(bytes, "ListenerTriggerChange", "inputId", input_index);
                push_uint_property(bytes, "ListenerTriggerChange", "flags", flags);
            });
        }
    }
}

fn synthetic_state_machine_blend_state_1d_input(file_id: u64) -> Vec<u8> {
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

fn synthetic_state_machine_blend_state_1d_view_model(file_id: u64, value: f32) -> Vec<u8> {
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
        push_bindable_number_data_bind(bytes, value);
        push_object_with_properties(bytes, "BlendState1DViewModel", |_| {});
        push_blend_animation_1d(bytes, 0, 0.0);
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn push_blend_animation_1d(bytes: &mut Vec<u8>, animation_id: u64, value: f32) {
    push_object_with_properties(bytes, "BlendAnimation1D", |bytes| {
        push_uint_property(bytes, "BlendAnimation1D", "animationId", animation_id);
        push_f32_property(bytes, "BlendAnimation1D", "value", value);
    });
}

fn push_bindable_number_data_bind(bytes: &mut Vec<u8>, value: f32) {
    push_object_with_properties(bytes, "BindablePropertyNumber", |bytes| {
        push_f32_property(bytes, "BindablePropertyNumber", "propertyValue", value);
    });
    push_object_with_properties(bytes, "DataBind", |_| {});
}

fn synthetic_state_machine_blend_state_direct(file_id: u64) -> Vec<u8> {
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

fn synthetic_state_machine_direct_bindable_blend_state(file_id: u64, value: f32) -> Vec<u8> {
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
        push_bindable_number_data_bind(bytes, value);
        push_object_with_properties(bytes, "BlendStateDirect", |_| {});
        push_blend_animation_direct_bindable(bytes, 0);
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_viewmodel_number_condition(
    file_id: u64,
    initial_value: f32,
    threshold: f32,
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
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 3);
        });
        push_bindable_number_data_bind(bytes, initial_value);
        push_object_with_properties(bytes, "TransitionViewModelCondition", |bytes| {
            push_uint_property(bytes, "TransitionViewModelCondition", "opValue", 5);
        });
        push_object_with_properties(bytes, "TransitionPropertyViewModelComparator", |_| {});
        push_object_with_properties(bytes, "TransitionValueNumberComparator", |bytes| {
            push_f32_property(bytes, "TransitionValueNumberComparator", "value", threshold);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_direct_blend_state_transition(file_id: u64) -> Vec<u8> {
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

fn push_blend_animation_direct_mix_value(bytes: &mut Vec<u8>, animation_id: u64, mix_value: f32) {
    push_object_with_properties(bytes, "BlendAnimationDirect", |bytes| {
        push_uint_property(bytes, "BlendAnimationDirect", "animationId", animation_id);
        push_uint_property(bytes, "BlendAnimationDirect", "blendSource", 1);
        push_f32_property(bytes, "BlendAnimationDirect", "mixValue", mix_value);
    });
}

fn push_blend_animation_direct_input(bytes: &mut Vec<u8>, animation_id: u64, input_id: u64) {
    push_object_with_properties(bytes, "BlendAnimationDirect", |bytes| {
        push_uint_property(bytes, "BlendAnimationDirect", "animationId", animation_id);
        push_uint_property(bytes, "BlendAnimationDirect", "inputId", input_id);
    });
}

fn push_blend_animation_direct_bindable(bytes: &mut Vec<u8>, animation_id: u64) {
    push_object_with_properties(bytes, "BlendAnimationDirect", |bytes| {
        push_uint_property(bytes, "BlendAnimationDirect", "animationId", animation_id);
        push_uint_property(bytes, "BlendAnimationDirect", "blendSource", 2);
    });
}

fn synthetic_state_machine_blend_state_transition(file_id: u64) -> Vec<u8> {
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
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 3);
            push_uint_property(bytes, "BlendStateTransition", "duration", 1000);
            push_uint_property(bytes, "BlendStateTransition", "exitTime", 1000);
            push_uint_property(bytes, "BlendStateTransition", "flags", ENABLE_EXIT_TIME);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 0);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 1);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_blend_state_transition_reset(file_id: u64) -> Vec<u8> {
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
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 3);
            push_uint_property(bytes, "BlendStateTransition", "duration", 1000);
            push_uint_property(bytes, "BlendStateTransition", "exitTime", 1000);
            push_uint_property(bytes, "BlendStateTransition", "flags", ENABLE_EXIT_TIME);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 0);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_blend_state_percentage_duration(file_id: u64) -> Vec<u8> {
    const DURATION_IS_PERCENTAGE: u64 = 1 << 1;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
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
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 3);
            push_uint_property(bytes, "BlendStateTransition", "duration", 50);
            push_uint_property(
                bytes,
                "BlendStateTransition",
                "flags",
                DURATION_IS_PERCENTAGE,
            );
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 1);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_blend_state_percentage_exit_time(file_id: u64) -> Vec<u8> {
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const EXIT_TIME_IS_PERCENTAGE: u64 = 1 << 3;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node_with_duration(bytes, 1, 2.0, 12.0, 10);
        push_animation_for_single_node_with_duration(bytes, 1, 20.0, 30.0, 20);
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
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 3);
            push_uint_property(bytes, "BlendStateTransition", "duration", 1000);
            push_uint_property(bytes, "BlendStateTransition", "exitTime", 75);
            push_uint_property(
                bytes,
                "BlendStateTransition",
                "flags",
                ENABLE_EXIT_TIME | EXIT_TIME_IS_PERCENTAGE,
            );
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 1);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn synthetic_state_machine_blend_state_pause_on_exit(file_id: u64) -> Vec<u8> {
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const PAUSE_ON_EXIT: u64 = 1 << 4;

    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_animation_for_single_node(bytes, 1, 2.0, 12.0);
        push_animation_for_single_node(bytes, 1, 20.0, 30.0);
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
        push_blend_animation_1d(bytes, 1, 1.0);
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 3);
            push_uint_property(bytes, "BlendStateTransition", "duration", 1000);
            push_uint_property(bytes, "BlendStateTransition", "exitTime", 500);
            push_uint_property(
                bytes,
                "BlendStateTransition",
                "flags",
                ENABLE_EXIT_TIME | PAUSE_ON_EXIT,
            );
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 1);
        });
        push_synthetic_bool_transition_condition(bytes, 1);
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 2);
        });
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

fn read_cpp_probe_bytes(probe: &Path, label: &str, bytes: &[u8]) -> CppProbeFile {
    read_cpp_probe_bytes_with_args(probe, label, bytes, &[])
}

fn read_cpp_probe_bytes_with_args(
    probe: &Path,
    label: &str,
    bytes: &[u8],
    extra_args: &[String],
) -> CppProbeFile {
    let path = std::env::temp_dir().join(format!(
        "rive-rust-runtime-{}-{}.riv",
        std::process::id(),
        label.replace('/', "-")
    ));
    std::fs::write(&path, bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));

    let output = Command::new(probe)
        .arg("--no-advance")
        .arg("--instance-artboards")
        .arg("--runtime-update")
        .args(extra_args)
        .arg("--file")
        .arg(&path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));

    let _ = std::fs::remove_file(&path);

    assert!(
        output.status.success(),
        "C++ probe failed for {label}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("invalid probe JSON for {label}: {err}"))
}

fn read_rust_instance_from_bytes(bytes: &[u8], label: &str) -> (RuntimeFile, ArtboardInstance) {
    let (runtime, _graph, instance) = read_rust_graph_instance_from_bytes(bytes, label);
    (runtime, instance)
}

fn read_rust_graph_instance_from_bytes(
    bytes: &[u8],
    label: &str,
) -> (RuntimeFile, GraphFile, ArtboardInstance) {
    let runtime = read_runtime_file(bytes).unwrap_or_else(|err| {
        panic!("failed to import {label}: {err:#}");
    });
    let graph = GraphFile::from_runtime_file(&runtime).unwrap_or_else(|err| {
        panic!("failed to build Rust graph for {label}: {err:#}");
    });
    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let instance = ArtboardInstance::from_graph(&runtime, artboard).unwrap_or_else(|err| {
        panic!("failed to build Rust artboard instance for {label}: {err:#}");
    });

    (runtime, graph, instance)
}

#[test]
fn runtime_update_matches_cpp_for_transform_hierarchy() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_transform_hierarchy.riv";
    let bytes = synthetic_transform_hierarchy();
    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    let cpp_update = cpp_artboard
        .runtime_update
        .as_ref()
        .unwrap_or_else(|| panic!("missing C++ runtimeUpdate for {label}"));

    assert_eq!(cpp_artboard.object_count, rust.slots().len());
    assert_eq!(cpp_artboard.objects.len(), rust.slots().len());
    for (local_id, (cpp_object, rust_slot)) in
        cpp_artboard.objects.iter().zip(rust.slots()).enumerate()
    {
        assert_eq!(rust_slot.local_id, local_id);
        match (cpp_object, rust_slot.type_name) {
            (Some(cpp_object), Some(type_name)) => {
                assert_eq!(cpp_object.local_id, local_id);
                assert_eq!(
                    cpp_object.core_type,
                    type_key_for_name(type_name),
                    "slot core type mismatch for local {local_id} in {label}"
                );
                assert_eq!(cpp_object.is_component, rust_slot.component_index.is_some());
            }
            (None, None) => {}
            _ => panic!("slot presence mismatch for local {local_id} in {label}"),
        }
    }

    assert_eq!(cpp_update.did_update, report.did_update);
    assert_eq!(
        cpp_update.has_components_dirt,
        rust.has_dirt(ComponentDirt::COMPONENTS)
    );

    for cpp_component in &cpp_update.components {
        let rust_component = rust
            .component(cpp_component.local_id)
            .unwrap_or_else(|| panic!("missing Rust component {}", cpp_component.local_id));
        compare_component(cpp_component, rust_component, label);
    }
}

#[test]
fn runtime_draw_command_stream_filters_hidden_and_opacity_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_draw_command_filtering.riv";
    let bytes = synthetic_runtime_file(8201, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "opacity", 0.0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_uint_property(bytes, "Drawable", "drawableFlags", 1);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_commands = rust
        .draw_commands(artboard)
        .into_iter()
        .map(|command| (command.local_id, command.kind, command.needs_save_operation))
        .collect::<Vec<_>>();
    let cpp_commands = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .map(|command| {
            (
                command.local_id,
                command.kind(),
                command.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_commands,
        vec![(Some(1), RuntimeDrawCommandKind::Draw, true)],
        "C++ draw command stream should skip hidden and opacity-zero shapes"
    );
    assert_eq!(
        rust_commands, cpp_commands,
        "Rust runtime draw command stream should match C++ willDraw filtering for simple shapes"
    );
}

#[test]
fn runtime_draw_command_stream_suppresses_empty_clips_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_empty_clip_filtering.riv";
    let bytes = synthetic_runtime_file(8202, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 2);
        });
        push_object_with_properties(bytes, "ClippingShape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 2);
            push_uint_property(bytes, "ClippingShape", "sourceId", 1);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_commands = rust
        .draw_commands(artboard)
        .into_iter()
        .map(|command| (command.local_id, command.kind, command.needs_save_operation))
        .collect::<Vec<_>>();
    let cpp_commands = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .map(|command| {
            (
                command.local_id,
                command.kind(),
                command.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_commands,
        vec![(Some(1), RuntimeDrawCommandKind::Draw, true)],
        "C++ draw command stream should suppress drawables inside an empty visible clipping shape"
    );
    assert_eq!(
        rust_commands, cpp_commands,
        "Rust runtime draw command stream should match C++ empty-clip suppression for source shapes with no paths"
    );
}

#[test]
fn runtime_draw_command_stream_treats_hidden_clip_paths_as_empty_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_hidden_clip_path_filtering.riv";
    let bytes = synthetic_runtime_file(8203, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_uint_property(bytes, "Path", "pathFlags", 1);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 3);
        });
        push_object_with_properties(bytes, "ClippingShape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 3);
            push_uint_property(bytes, "ClippingShape", "sourceId", 1);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_commands = rust
        .draw_commands(artboard)
        .into_iter()
        .map(|command| (command.local_id, command.kind, command.needs_save_operation))
        .collect::<Vec<_>>();
    let cpp_commands = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .map(|command| {
            (
                command.local_id,
                command.kind(),
                command.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_commands,
        vec![(Some(1), RuntimeDrawCommandKind::Draw, true)],
        "C++ draw command stream should suppress drawables clipped by a source shape with only hidden paths"
    );
    assert_eq!(
        rust_commands, cpp_commands,
        "Rust runtime draw command stream should match C++ empty-clip suppression for hidden source paths"
    );
}

#[test]
fn runtime_draw_command_stream_treats_collapsed_clip_paths_as_empty_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_collapsed_clip_path_filtering.riv";
    let bytes = synthetic_runtime_file(8204, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 3);
        });
        push_object_with_properties(bytes, "ClippingShape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 3);
            push_uint_property(bytes, "ClippingShape", "sourceId", 1);
        });
    });

    let cpp = read_cpp_probe_bytes_with_args(
        &probe,
        label,
        &bytes,
        &[
            "--runtime-collapse-component".to_owned(),
            "2".to_owned(),
            "true".to_owned(),
        ],
    );
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    assert!(rust.collapse_component(2, true));
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_commands = rust
        .draw_commands(artboard)
        .into_iter()
        .map(|command| (command.local_id, command.kind, command.needs_save_operation))
        .collect::<Vec<_>>();
    let cpp_commands = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .map(|command| {
            (
                command.local_id,
                command.kind(),
                command.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_commands,
        vec![(Some(1), RuntimeDrawCommandKind::Draw, true)],
        "C++ draw command stream should suppress drawables clipped by a source shape with only collapsed paths"
    );
    assert_eq!(
        rust_commands, cpp_commands,
        "Rust runtime draw command stream should match C++ empty-clip suppression for collapsed source paths"
    );
}

#[test]
fn runtime_draw_command_stream_exposes_shape_paint_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_shape_paint_payloads.riv";
    let bytes = synthetic_runtime_file(8205, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
            push_f32_property(bytes, "Node", "opacity", 0.5);
            push_uint_property(bytes, "Drawable", "blendModeValue", 24);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_uint_property(bytes, "Fill", "fillRule", 2);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0x8040_2010);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_bool_property(bytes, "ShapePaint", "isVisible", false);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 4);
        });
        push_object_with_properties(bytes, "Stroke", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_uint_property(bytes, "ShapePaint", "blendModeValue", 14);
            push_bool_property(bytes, "Stroke", "transformAffectsStroke", false);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 6);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff11_2233);
        });
        push_object_with_properties(bytes, "Stroke", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_f32_property(bytes, "Stroke", "thickness", 0.0);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 8);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_f32_property(bytes, "Node", "x", 10.0);
            push_bool_property(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 10);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "CubicAsymmetricVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 10);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
            push_f32_property(bytes, "CubicAsymmetricVertex", "inDistance", 5.0);
            push_f32_property(bytes, "CubicAsymmetricVertex", "outDistance", 5.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 10);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 20.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_payloads = rust
        .draw_commands(artboard)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .map(|paint| {
            (
                Some(paint.paint_local),
                paint.mutator_local,
                paint.paint_type,
                paint.path_kind,
                paint.blend_mode_value,
                paint.render_blend_mode_value,
                paint.paint_state,
                paint.feather_state,
                paint.path_commands,
                paint.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();
    let cpp_payloads = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .map(|paint| {
            (
                paint.paint_local,
                paint.mutator_local,
                paint.paint_type(),
                paint.path_kind(),
                paint.blend_mode_value,
                paint.render_blend_mode_value,
                paint.paint_state(),
                paint.feather_state(),
                paint.path_commands(),
                paint.needs_save_operation,
            )
        })
        .collect::<Vec<_>>();
    let expected_local_path_commands = vec![
        RuntimePathCommand::Move { x: 10.0, y: 0.0 },
        RuntimePathCommand::Cubic {
            x1: 10.0,
            y1: 0.0,
            x2: 15.0,
            y2: 0.0,
            x3: 20.0,
            y3: 0.0,
        },
        RuntimePathCommand::Cubic {
            x1: 25.0,
            y1: 0.0,
            x2: 20.0,
            y2: 20.0,
            x3: 20.0,
            y3: 20.0,
        },
        RuntimePathCommand::Line { x: 10.0, y: 0.0 },
        RuntimePathCommand::Close,
    ];
    let expected_world_path_commands = vec![
        RuntimePathCommand::Move { x: 110.0, y: 0.0 },
        RuntimePathCommand::Cubic {
            x1: 110.0,
            y1: 0.0,
            x2: 115.0,
            y2: 0.0,
            x3: 120.0,
            y3: 0.0,
        },
        RuntimePathCommand::Cubic {
            x1: 125.0,
            y1: 0.0,
            x2: 120.0,
            y2: 20.0,
            x3: 120.0,
            y3: 20.0,
        },
        RuntimePathCommand::Line { x: 110.0, y: 0.0 },
        RuntimePathCommand::Close,
    ];

    assert_eq!(
        cpp_payloads,
        vec![
            (
                Some(2),
                Some(3),
                RuntimeShapePaintKind::Fill,
                RuntimeShapePaintPathKind::LocalClockwise,
                127,
                24,
                Some(RuntimeShapePaintState::SolidColor {
                    color: 0x8040_2010,
                    render_color: 0x4040_2010,
                }),
                None,
                expected_local_path_commands,
                true
            ),
            (
                Some(6),
                Some(7),
                RuntimeShapePaintKind::Stroke,
                RuntimeShapePaintPathKind::World,
                14,
                14,
                Some(RuntimeShapePaintState::SolidColor {
                    color: 0xff11_2233,
                    render_color: 0x8011_2233,
                }),
                None,
                expected_world_path_commands,
                true
            ),
        ],
        "C++ shape payloads should include visible paints and skip invisible/zero-thickness paints"
    );
    assert_eq!(
        rust_payloads, cpp_payloads,
        "Rust shape paint command payloads should match C++ Shape::draw paint filtering and path selection"
    );
}

#[test]
fn runtime_draw_command_stream_exposes_rounded_point_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_rounded_point_path_payloads.riv";
    let bytes = synthetic_runtime_file(8214, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_uint_property(bytes, "Fill", "fillRule", 2);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xffa0_3020);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_bool_property(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 4);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
            push_f32_property(bytes, "StraightVertex", "radius", 2.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 4);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
            push_f32_property(bytes, "StraightVertex", "radius", -2.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 4);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 10.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 4);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 10.0);
            push_f32_property(bytes, "StraightVertex", "radius", 2.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_paints = rust
        .draw_commands(artboard)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(
        cpp_paints.len(),
        1,
        "C++ should emit one visible shape paint"
    );
    assert_eq!(
        rust_paints.len(),
        1,
        "Rust should emit one visible shape paint"
    );

    let cpp_paint = cpp_paints[0];
    let rust_paint = &rust_paints[0];
    assert_eq!(cpp_paint.paint_local, Some(rust_paint.paint_local));
    assert_eq!(cpp_paint.mutator_local, rust_paint.mutator_local);
    assert_eq!(cpp_paint.paint_type(), rust_paint.paint_type);
    assert_eq!(cpp_paint.path_kind(), rust_paint.path_kind);
    assert_eq!(cpp_paint.blend_mode_value, rust_paint.blend_mode_value);
    assert_eq!(
        cpp_paint.render_blend_mode_value,
        rust_paint.render_blend_mode_value
    );
    assert_eq!(cpp_paint.paint_state(), rust_paint.paint_state);
    assert_eq!(cpp_paint.feather_state(), rust_paint.feather_state);
    assert_eq!(
        cpp_paint.needs_save_operation,
        rust_paint.needs_save_operation
    );

    let expected_path_commands = vec![
        RuntimePathCommand::Move { x: 0.0, y: 2.0 },
        RuntimePathCommand::Cubic {
            x1: 0.0,
            y1: 0.895_430_3,
            x2: 0.895_430_3,
            y2: 0.0,
            x3: 2.0,
            y3: 0.0,
        },
        RuntimePathCommand::Line { x: 8.0, y: 0.0 },
        RuntimePathCommand::Cubic {
            x1: 8.0,
            y1: 1.104_569_4,
            x2: 8.895_431,
            y2: 2.0,
            x3: 10.0,
            y3: 2.0,
        },
        RuntimePathCommand::Line { x: 10.0, y: 10.0 },
        RuntimePathCommand::Line { x: 2.0, y: 10.0 },
        RuntimePathCommand::Cubic {
            x1: 0.895_430_3,
            y1: 10.0,
            x2: 0.0,
            y2: 9.104_569,
            x3: 0.0,
            y3: 8.0,
        },
        RuntimePathCommand::Line { x: 0.0, y: 2.0 },
        RuntimePathCommand::Close,
    ];
    let cpp_path_commands = cpp_paint.path_commands();
    assert_path_commands_close(
        &cpp_path_commands,
        &expected_path_commands,
        "C++ rounded point path commands",
    );
    assert_path_commands_close(
        &rust_paint.path_commands,
        &cpp_path_commands,
        "Rust rounded point path commands",
    );
}

#[test]
fn runtime_draw_command_stream_exposes_gradient_paint_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_gradient_paint_payloads.riv";
    let bytes = synthetic_runtime_file(8215, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "opacity", 0.5);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "LinearGradient", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_f32_property(bytes, "LinearGradient", "startX", 1.0);
            push_f32_property(bytes, "LinearGradient", "startY", 2.0);
            push_f32_property(bytes, "LinearGradient", "endX", 11.0);
            push_f32_property(bytes, "LinearGradient", "endY", 12.0);
            push_f32_property(bytes, "LinearGradient", "opacity", 0.5);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 3);
            push_color_property(bytes, "GradientStop", "colorValue", 0xff00_ff00);
            push_f32_property(bytes, "GradientStop", "position", 1.5);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 3);
            push_color_property(bytes, "GradientStop", "colorValue", 0x8000_00ff);
            push_f32_property(bytes, "GradientStop", "position", -0.25);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 3);
            push_color_property(bytes, "GradientStop", "colorValue", 0xffff_0000);
            push_f32_property(bytes, "GradientStop", "position", 0.5);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_bool_property(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 7);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 7);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 7);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 10.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_paints = rust
        .draw_commands(artboard)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one gradient paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one gradient paint");

    let expected_state = RuntimeShapePaintState::LinearGradient {
        start_x: 1.0,
        start_y: 2.0,
        end_x: 11.0,
        end_y: 12.0,
        opacity: 0.5,
        render_opacity: 0.5,
        stops: vec![
            RuntimeGradientStop {
                color: 0x8000_00ff,
                render_color: 0x2000_00ff,
                position: 0.0,
            },
            RuntimeGradientStop {
                color: 0xffff_0000,
                render_color: 0x40ff_0000,
                position: 0.5,
            },
            RuntimeGradientStop {
                color: 0xff00_ff00,
                render_color: 0x4000_ff00,
                position: 1.0,
            },
        ],
    };

    let cpp_paint = cpp_paints[0];
    let rust_paint = &rust_paints[0];
    assert_eq!(cpp_paint.paint_state(), Some(expected_state.clone()));
    assert_eq!(rust_paint.paint_state, Some(expected_state));
    assert_eq!(rust_paint.paint_state, cpp_paint.paint_state());
    assert_eq!(cpp_paint.feather_state(), None);
    assert_eq!(rust_paint.feather_state, None);
}

#[test]
fn runtime_draw_command_stream_exposes_feather_paint_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_feather_paint_payloads.riv";
    let bytes = synthetic_runtime_file(8216, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xffaa_5500);
        });
        push_object_with_properties(bytes, "Feather", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_uint_property(bytes, "Feather", "spaceValue", 1);
            push_f32_property(bytes, "Feather", "strength", 8.0);
            push_f32_property(bytes, "Feather", "offsetX", 3.0);
            push_f32_property(bytes, "Feather", "offsetY", -4.0);
            push_bool_property(bytes, "Feather", "inner", true);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_bool_property(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 10.0);
            push_f32_property(bytes, "Vertex", "y", 0.0);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 5);
            push_f32_property(bytes, "Vertex", "x", 0.0);
            push_f32_property(bytes, "Vertex", "y", 10.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_paints = rust
        .draw_commands(artboard)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one feather paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one feather paint");

    let expected_feather = RuntimeFeatherState {
        feather_local: 4,
        space_value: 1,
        strength: 8.0,
        offset_x: 3.0,
        offset_y: -4.0,
        inner: true,
    };

    let cpp_paint = cpp_paints[0];
    let rust_paint = &rust_paints[0];
    assert_eq!(cpp_paint.feather_state(), Some(expected_feather));
    assert_eq!(rust_paint.feather_state, Some(expected_feather));
    assert_eq!(rust_paint.feather_state, cpp_paint.feather_state());
}

#[test]
fn runtime_draw_command_stream_exposes_rectangle_parametric_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_rectangle_parametric_path_payloads.riv";
    let bytes = synthetic_runtime_file(8217, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff22_8844);
        });
        push_object_with_properties(bytes, "Rectangle", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_f32_property(bytes, "Node", "x", 7.0);
            push_f32_property(bytes, "Node", "y", -2.0);
            push_f32_property(bytes, "ParametricPath", "width", 20.0);
            push_f32_property(bytes, "ParametricPath", "height", 10.0);
            push_f32_property(bytes, "ParametricPath", "originX", 0.25);
            push_f32_property(bytes, "ParametricPath", "originY", 0.5);
            push_bool_property(bytes, "Rectangle", "linkCornerRadius", true);
            push_f32_property(bytes, "Rectangle", "cornerRadiusTL", 2.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_paints = rust
        .draw_commands(artboard)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one rectangle paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one rectangle paint");

    let cpp_path_commands = cpp_paints[0].path_commands();
    assert_eq!(
        cpp_path_commands.len(),
        10,
        "rounded rectangle should produce move + four rounded corners + close"
    );
    assert_path_commands_close(
        &rust_paints[0].path_commands,
        &cpp_path_commands,
        "Rust rectangle parametric path commands",
    );
}

#[test]
fn runtime_draw_command_stream_exposes_ellipse_parametric_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_ellipse_parametric_path_payloads.riv";
    let bytes = synthetic_runtime_file(8218, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff33_66aa);
        });
        push_object_with_properties(bytes, "Ellipse", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_f32_property(bytes, "Node", "x", 3.0);
            push_f32_property(bytes, "Node", "y", 4.0);
            push_f32_property(bytes, "ParametricPath", "width", 30.0);
            push_f32_property(bytes, "ParametricPath", "height", 12.0);
            push_f32_property(bytes, "ParametricPath", "originX", 0.25);
            push_f32_property(bytes, "ParametricPath", "originY", 0.75);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_paints = rust
        .draw_commands(artboard)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one ellipse paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one ellipse paint");

    let cpp_path_commands = cpp_paints[0].path_commands();
    assert_eq!(
        cpp_path_commands.len(),
        6,
        "ellipse should produce move + four cubics + close"
    );
    assert_path_commands_close(
        &rust_paints[0].path_commands,
        &cpp_path_commands,
        "Rust ellipse parametric path commands",
    );
}

#[test]
fn runtime_draw_command_stream_exposes_polygon_parametric_path_payloads_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_polygon_parametric_path_payloads.riv";
    let bytes = synthetic_runtime_file(8219, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 100.0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 2);
            push_color_property(bytes, "SolidColor", "colorValue", 0xff66_44aa);
        });
        push_object_with_properties(bytes, "Polygon", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 1);
            push_f32_property(bytes, "Node", "x", 3.0);
            push_f32_property(bytes, "Node", "y", 4.0);
            push_f32_property(bytes, "ParametricPath", "width", 30.0);
            push_f32_property(bytes, "ParametricPath", "height", 12.0);
            push_f32_property(bytes, "ParametricPath", "originX", 0.25);
            push_f32_property(bytes, "ParametricPath", "originY", 0.75);
            push_uint_property(bytes, "Polygon", "points", 5);
            push_f32_property(bytes, "Polygon", "cornerRadius", 1.5);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, graph, mut rust) = read_rust_graph_instance_from_bytes(&bytes, label);
    rust.update_components();

    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let rust_paints = rust
        .draw_commands(artboard)
        .into_iter()
        .flat_map(|command| command.shape_paints)
        .collect::<Vec<_>>();
    let cpp_paints = cpp.artboards[0]
        .draw_command_stream
        .iter()
        .flat_map(|command| command.shape_paint_commands.iter())
        .collect::<Vec<_>>();

    assert_eq!(cpp_paints.len(), 1, "C++ should emit one polygon paint");
    assert_eq!(rust_paints.len(), 1, "Rust should emit one polygon paint");

    let cpp_path_commands = cpp_paints[0].path_commands();
    assert_eq!(
        cpp_path_commands.len(),
        12,
        "rounded five-point polygon should produce move + five rounded corners + close"
    );
    assert_path_commands_close(
        &rust_paints[0].path_commands,
        &cpp_path_commands,
        "Rust polygon parametric path commands",
    );
}

#[test]
fn mutated_instance_transform_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_mutated_transform_hierarchy.riv";
    let bytes = synthetic_transform_hierarchy();
    let x_key = property_key_for_name("Node", "x");
    let cpp = read_cpp_probe_bytes_with_args(
        &probe,
        label,
        &bytes,
        &[
            "--runtime-set-double".to_owned(),
            "1".to_owned(),
            x_key.to_string(),
            "12.0".to_owned(),
        ],
    );
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    assert!(rust.set_transform_property(1, TransformProperty::X, 12.0));
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    let cpp_update = cpp_artboard
        .runtime_update
        .as_ref()
        .unwrap_or_else(|| panic!("missing C++ runtimeUpdate for {label}"));

    assert_eq!(cpp_update.did_update, report.did_update);
    assert_eq!(
        cpp_update.has_components_dirt,
        rust.has_dirt(ComponentDirt::COMPONENTS)
    );

    for cpp_component in &cpp_update.components {
        let rust_component = rust
            .component(cpp_component.local_id)
            .unwrap_or_else(|| panic!("missing Rust component {}", cpp_component.local_id));
        compare_component(cpp_component, rust_component, label);
    }
}

#[test]
fn mutable_instance_transform_does_not_mutate_source_object() {
    let label = "synthetic/runtime_transform_source_separation.riv";
    let bytes = synthetic_transform_hierarchy();
    let (runtime, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let child_slot = rust.slot(1).expect("child slot should exist");
    let source_child = runtime
        .object(child_slot.source_global_id as usize)
        .expect("source child should exist");

    assert_eq!(source_child.double_property("x"), Some(2.0));

    assert!(rust.set_transform_property(1, TransformProperty::X, 12.0));
    assert_eq!(rust.component(1).unwrap().transform.x, 12.0);
    assert_eq!(
        source_child.double_property("x"),
        Some(2.0),
        "mutating instance transform state must not mutate imported source data"
    );

    rust.update_components();

    assert_eq!(
        rust.component(1).unwrap().transform.local_transform.0[4],
        12.0
    );
    assert_eq!(source_child.double_property("x"), Some(2.0));
}

#[test]
fn linear_animation_apply_interpolates_keyframe_double_transform() {
    let label = "synthetic/runtime_linear_animation_interpolation.riv";
    let bytes = synthetic_linear_animation(8201, 0, 2.0, 10, 12.0, 1, false);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);

    assert_eq!(rust.linear_animations().len(), 1);
    assert_eq!(
        rust.linear_animation(0).unwrap().keyed_objects[0].keyed_properties[0]
            .key_frames
            .len(),
        2
    );

    assert!(rust.apply_linear_animation(0, 0.5, 1.0));
    assert_close(
        rust.component(1).unwrap().transform.x,
        7.0,
        "interpolated x",
    );
    assert!(
        rust.component(1)
            .unwrap()
            .dirt
            .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
    );
}

#[test]
fn linear_animation_apply_holds_when_interpolation_type_is_zero() {
    let label = "synthetic/runtime_linear_animation_hold.riv";
    let bytes = synthetic_linear_animation(8202, 0, 4.0, 10, 12.0, 0, false);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);

    assert!(rust.apply_linear_animation(0, 0.5, 1.0));
    assert_close(rust.component(1).unwrap().transform.x, 4.0, "held x");
}

#[test]
fn linear_animation_apply_uses_first_and_last_keyframes_outside_range() {
    let before_label = "synthetic/runtime_linear_animation_before_first.riv";
    let before_bytes = synthetic_linear_animation(8203, 10, 4.0, 20, 12.0, 1, false);
    let (_, mut before) = read_rust_instance_from_bytes(&before_bytes, before_label);

    assert!(before.apply_linear_animation(0, 0.0, 1.0));
    assert_close(
        before.component(1).unwrap().transform.x,
        4.0,
        "before first x",
    );

    let after_label = "synthetic/runtime_linear_animation_after_last.riv";
    let after_bytes = synthetic_linear_animation(8204, 0, 4.0, 10, 12.0, 1, false);
    let (_, mut after) = read_rust_instance_from_bytes(&after_bytes, after_label);

    assert!(after.apply_linear_animation(0, 2.0, 1.0));
    assert_close(
        after.component(1).unwrap().transform.x,
        12.0,
        "after last x",
    );
}

#[test]
fn linear_animation_apply_quantizes_seconds_before_sampling() {
    let label = "synthetic/runtime_linear_animation_quantized.riv";
    let bytes = synthetic_linear_animation(8205, 0, 2.0, 10, 12.0, 1, true);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);

    assert!(rust.apply_linear_animation(0, 0.99, 1.0));
    assert_close(rust.component(1).unwrap().transform.x, 11.0, "quantized x");
}

#[test]
fn linear_animation_apply_mixes_with_current_transform_value() {
    let label = "synthetic/runtime_linear_animation_mixed.riv";
    let bytes = synthetic_linear_animation(8206, 0, 2.0, 10, 12.0, 1, false);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);

    assert!(rust.apply_linear_animation(0, 1.0, 0.25));
    assert_close(rust.component(1).unwrap().transform.x, 4.5, "mixed x");
}

#[test]
fn linear_animation_apply_does_not_mutate_source_object() {
    let label = "synthetic/runtime_linear_animation_source_separation.riv";
    let bytes = synthetic_linear_animation(8207, 0, 2.0, 10, 12.0, 1, false);
    let (runtime, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let child_slot = rust.slot(1).expect("child slot should exist");
    let source_child = runtime
        .object(child_slot.source_global_id as usize)
        .expect("source child should exist");

    assert_eq!(source_child.double_property("x"), Some(2.0));

    assert!(rust.apply_linear_animation(0, 1.0, 1.0));
    assert_close(rust.component(1).unwrap().transform.x, 12.0, "applied x");
    assert_eq!(source_child.double_property("x"), Some(2.0));
}

#[test]
fn linear_animation_apply_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let cases = [
        (
            "synthetic/runtime_linear_animation_cpp_exact.riv",
            synthetic_linear_animation(8211, 0, 2.0, 10, 12.0, 1, false),
            1.0,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_before_first.riv",
            synthetic_linear_animation(8212, 10, 4.0, 20, 12.0, 1, false),
            0.0,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_interpolated.riv",
            synthetic_linear_animation(8213, 0, 2.0, 10, 12.0, 1, false),
            0.5,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_held.riv",
            synthetic_linear_animation(8214, 0, 4.0, 10, 12.0, 0, false),
            0.5,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_after_last.riv",
            synthetic_linear_animation(8215, 0, 4.0, 10, 12.0, 1, false),
            2.0,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_quantized.riv",
            synthetic_linear_animation(8216, 0, 2.0, 10, 12.0, 1, true),
            0.99,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_mixed.riv",
            synthetic_linear_animation(8217, 0, 2.0, 10, 12.0, 1, false),
            1.0,
            0.25,
        ),
    ];

    for (label, bytes, seconds, mix) in cases {
        let cpp = read_cpp_probe_bytes_with_args(
            &probe,
            label,
            &bytes,
            &[
                "--runtime-apply-animation".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
                mix.to_string(),
            ],
        );
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        rust.apply_linear_animation(0, seconds, mix);
        let report = rust.update_components();

        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn linear_animation_instance_advance_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let cases = [
        (
            "synthetic/runtime_linear_animation_instance_one_shot.riv",
            synthetic_linear_animation_with_options(
                8221,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    ..Default::default()
                },
            ),
            1.5,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_loop.riv",
            synthetic_linear_animation_with_options(
                8222,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    loop_value: 1,
                    ..Default::default()
                },
            ),
            1.25,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_ping_pong.riv",
            synthetic_linear_animation_with_options(
                8223,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    loop_value: 2,
                    ..Default::default()
                },
            ),
            1.25,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_work_area.riv",
            synthetic_linear_animation_with_options(
                8224,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 20,
                    enable_work_area: true,
                    work_start: 2,
                    work_end: 8,
                    ..Default::default()
                },
            ),
            0.3,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_negative_speed.riv",
            synthetic_linear_animation_with_options(
                8225,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    speed: -1.0,
                    ..Default::default()
                },
            ),
            0.25,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_mixed.riv",
            synthetic_linear_animation_with_options(
                8226,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    ..Default::default()
                },
            ),
            0.5,
            0.25,
        ),
    ];

    for (label, bytes, seconds, mix) in cases {
        let cpp = read_cpp_probe_bytes_with_args(
            &probe,
            label,
            &bytes,
            &[
                "--runtime-advance-animation".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
                mix.to_string(),
            ],
        );
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut animation = rust
            .linear_animation_instance(0)
            .unwrap_or_else(|| panic!("missing Rust animation instance for {label}"));
        let keep_going = rust.advance_linear_animation_instance(&mut animation, seconds);
        rust.apply_linear_animation_instance(&animation, mix);
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        let cpp_animation = cpp_artboard
            .runtime_animation_advances
            .first()
            .unwrap_or_else(|| panic!("missing C++ animation advance report for {label}"));
        let runtime_animation = rust
            .linear_animation(0)
            .unwrap_or_else(|| panic!("missing Rust runtime animation for {label}"));
        let keep_going_after = rust.linear_animation_instance_keep_going(&animation);
        compare_animation_advance(
            cpp_animation,
            runtime_animation,
            &animation,
            keep_going,
            keep_going_after,
            label,
        );
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_animation_state_advances_through_public_runtime_seam() {
    let label = "synthetic/runtime_state_machine_animation_state_public.riv";
    let bytes = synthetic_state_machine_animation_state(8230);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);

    assert_eq!(rust.state_machines().len(), 1);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(rust.advance_state_machine_instance(&mut state_machine, 0.0));
    assert_eq!(state_machine.current_animation_count(), 1);
    assert_close(
        state_machine.current_animation(0).unwrap().time(),
        0.0,
        "entered animation state time",
    );
    assert_close(
        rust.component(1).unwrap().transform.x,
        2.0,
        "entered animation state x",
    );

    assert!(rust.advance_state_machine_instance(&mut state_machine, 0.5));
    assert_close(
        state_machine.current_animation(0).unwrap().time(),
        0.5,
        "advanced animation state time",
    );
    assert_close(
        rust.component(1).unwrap().transform.x,
        7.0,
        "advanced animation state x",
    );

    assert!(
        !rust.advance_state_machine_instance(&mut state_machine, 0.0),
        "zero-second state-machine advance after entering an animation state matches C++ cached keep-going behavior"
    );
}

#[test]
fn state_machine_bool_input_drives_zero_duration_transition() {
    let label = "synthetic/runtime_state_machine_bool_transition_public.riv";
    let bytes = synthetic_state_machine_bool_transition(8232);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert_eq!(state_machine.input_count(), 1);
    let input = state_machine
        .input_named("armed")
        .unwrap_or_else(|| panic!("missing named input for {label}"));
    assert_eq!(input.kind(), StateMachineInputKind::Bool);
    assert_eq!(input.bool_value(), Some(false));

    assert!(rust.advance_state_machine_instance(&mut state_machine, 0.0));
    assert_close(
        rust.component(1).unwrap().transform.x,
        2.0,
        "initial bool transition state x",
    );

    assert!(state_machine.set_bool(0, true));
    assert_eq!(
        state_machine.input(0).and_then(|input| input.bool_value()),
        Some(true)
    );
    assert!(rust.advance_state_machine_instance(&mut state_machine, 0.0));
    assert_eq!(state_machine.changed_state_count(), 1);
    assert_close(
        rust.component(1).unwrap().transform.x,
        20.0,
        "bool transition target state x",
    );
}

#[test]
fn state_machine_number_input_drives_zero_duration_transition() {
    let label = "synthetic/runtime_state_machine_number_transition_public.riv";
    let bytes =
        synthetic_state_machine_input_transition(8233, SyntheticInputTransitionKind::Number);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let input = state_machine
        .input_named("level")
        .unwrap_or_else(|| panic!("missing named input for {label}"));
    assert_eq!(input.kind(), StateMachineInputKind::Number);
    assert_eq!(input.number_value(), Some(0.0));

    assert!(rust.advance_state_machine_instance(&mut state_machine, 0.0));
    assert_close(
        rust.component(1).unwrap().transform.x,
        2.0,
        "initial number transition state x",
    );

    assert!(state_machine.set_number(0, 4.0));
    assert!(rust.advance_state_machine_instance(&mut state_machine, 0.0));
    assert_eq!(state_machine.changed_state_count(), 1);
    assert_close(
        rust.component(1).unwrap().transform.x,
        20.0,
        "number transition target state x",
    );
}

#[test]
fn state_machine_trigger_input_drives_zero_duration_transition_once() {
    let label = "synthetic/runtime_state_machine_trigger_transition_public.riv";
    let bytes =
        synthetic_state_machine_input_transition(8234, SyntheticInputTransitionKind::Trigger);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let input = state_machine
        .input_named("go")
        .unwrap_or_else(|| panic!("missing named input for {label}"));
    assert_eq!(input.kind(), StateMachineInputKind::Trigger);
    assert_eq!(input.trigger_fired(), Some(false));

    assert!(rust.advance_state_machine_instance(&mut state_machine, 0.0));
    assert_close(
        rust.component(1).unwrap().transform.x,
        2.0,
        "initial trigger transition state x",
    );

    assert!(state_machine.fire_trigger(0));
    assert_eq!(
        state_machine
            .input(0)
            .and_then(|input| input.trigger_fired()),
        Some(true)
    );
    assert!(rust.advance_state_machine_instance(&mut state_machine, 0.0));
    assert_eq!(state_machine.changed_state_count(), 1);
    assert_eq!(
        state_machine
            .input(0)
            .and_then(|input| input.trigger_fired()),
        Some(false),
        "triggers clear after advance like C++ SMITrigger::advanced"
    );
    assert_close(
        rust.component(1).unwrap().transform.x,
        20.0,
        "trigger transition target state x",
    );
}

#[test]
fn state_machine_input_transitions_match_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (file_id, kind, label) in [
        (
            8235,
            SyntheticInputTransitionKind::Bool,
            "synthetic/runtime_state_machine_bool_transition_cpp.riv",
        ),
        (
            8236,
            SyntheticInputTransitionKind::Number,
            "synthetic/runtime_state_machine_number_transition_cpp.riv",
        ),
        (
            8237,
            SyntheticInputTransitionKind::Trigger,
            "synthetic/runtime_state_machine_trigger_transition_cpp.riv",
        ),
    ] {
        let bytes = synthetic_state_machine_input_transition(file_id, kind);
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ];
        match kind {
            SyntheticInputTransitionKind::Bool => {
                args.extend([
                    "--runtime-set-state-machine-bool".to_owned(),
                    "0".to_owned(),
                    "0".to_owned(),
                    "true".to_owned(),
                ]);
            }
            SyntheticInputTransitionKind::Number => {
                args.extend([
                    "--runtime-set-state-machine-number".to_owned(),
                    "0".to_owned(),
                    "0".to_owned(),
                    "4".to_owned(),
                ]);
            }
            SyntheticInputTransitionKind::Trigger => {
                args.extend([
                    "--runtime-fire-state-machine-trigger".to_owned(),
                    "0".to_owned(),
                    "0".to_owned(),
                ]);
            }
        }
        args.extend([
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ]);

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut state_machine = rust
            .state_machine_instance(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
        rust_reports.push((advanced, state_machine.clone()));
        match kind {
            SyntheticInputTransitionKind::Bool => {
                assert!(state_machine.set_bool(0, true));
            }
            SyntheticInputTransitionKind::Number => {
                assert!(state_machine.set_number(0, 4.0));
            }
            SyntheticInputTransitionKind::Trigger => {
                assert!(state_machine.fire_trigger(0));
            }
        }
        let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
        rust_reports.push((advanced, state_machine.clone()));
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_timed_transition_mixing_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (file_id, label, post_transition_advances) in [
        (
            8238,
            "synthetic/runtime_state_machine_timed_transition_start_cpp.riv",
            Vec::<f32>::new(),
        ),
        (
            8239,
            "synthetic/runtime_state_machine_timed_transition_half_cpp.riv",
            vec![0.5],
        ),
        (
            8240,
            "synthetic/runtime_state_machine_timed_transition_complete_cpp.riv",
            vec![0.5, 0.5],
        ),
    ] {
        let bytes = synthetic_state_machine_input_transition_with_duration(
            file_id,
            SyntheticInputTransitionKind::Bool,
            1000,
        );
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bool".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "true".to_owned(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ];
        for seconds in &post_transition_advances {
            args.extend([
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
            ]);
        }

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut state_machine = rust
            .state_machine_instance(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
        rust_reports.push((advanced, state_machine.clone()));
        assert!(state_machine.set_bool(0, true));
        let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
        rust_reports.push((advanced, state_machine.clone()));
        for seconds in post_transition_advances {
            let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
            rust_reports.push((advanced, state_machine.clone()));
        }
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_exit_time_transition_matches_cpp_probe() {
    const ENABLE_EXIT_TIME: u64 = 1 << 2;

    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (label, bytes, post_set_advances) in [
        (
            "synthetic/runtime_state_machine_exit_time_transition_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8241,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    flags: ENABLE_EXIT_TIME,
                    exit_time: Some(1000),
                    ..Default::default()
                },
            ),
            vec![0.5, 0.5],
        ),
        (
            "synthetic/runtime_state_machine_any_exit_time_transition_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8242,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    flags: ENABLE_EXIT_TIME,
                    exit_time: Some(1000),
                    any_state_transition: true,
                    ..Default::default()
                },
            ),
            vec![0.0],
        ),
    ] {
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bool".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "true".to_owned(),
        ];
        for seconds in &post_set_advances {
            args.extend([
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
            ]);
        }

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut state_machine = rust
            .state_machine_instance(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
        rust_reports.push((advanced, state_machine.clone()));
        assert!(state_machine.set_bool(0, true));
        for seconds in post_set_advances {
            let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
            rust_reports.push((advanced, state_machine.clone()));
        }
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_transition_handoff_matches_cpp_probe() {
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const PAUSE_ON_EXIT: u64 = 1 << 4;

    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (label, bytes, post_set_advances) in [
        (
            "synthetic/runtime_state_machine_spilled_time_handoff_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8243,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions::default(),
            ),
            vec![2.5],
        ),
        (
            "synthetic/runtime_state_machine_pause_on_exit_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8244,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    duration: 1000,
                    flags: ENABLE_EXIT_TIME | PAUSE_ON_EXIT,
                    exit_time: Some(1000),
                    source_second_frame: 20,
                    source_second_value: 22.0,
                    ..Default::default()
                },
            ),
            vec![1.5, 0.5],
        ),
    ] {
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bool".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "true".to_owned(),
        ];
        for seconds in &post_set_advances {
            args.extend([
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
            ]);
        }

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut state_machine = rust
            .state_machine_instance(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
        rust_reports.push((advanced, state_machine.clone()));
        assert!(state_machine.set_bool(0, true));
        for seconds in post_set_advances {
            let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
            rust_reports.push((advanced, state_machine.clone()));
        }
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_percentage_timing_matches_cpp_probe() {
    const DURATION_IS_PERCENTAGE: u64 = 1 << 1;
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const EXIT_TIME_IS_PERCENTAGE: u64 = 1 << 3;

    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (label, bytes, post_set_advances) in [
        (
            "synthetic/runtime_state_machine_percentage_duration_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8245,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    duration: 50,
                    flags: DURATION_IS_PERCENTAGE,
                    ..Default::default()
                },
            ),
            vec![0.0, 0.5, 0.5],
        ),
        (
            "synthetic/runtime_state_machine_percentage_exit_time_cpp.riv",
            synthetic_state_machine_input_transition_with_options(
                8246,
                SyntheticInputTransitionKind::Bool,
                SyntheticTransitionOptions {
                    flags: ENABLE_EXIT_TIME | EXIT_TIME_IS_PERCENTAGE,
                    exit_time: Some(50),
                    ..Default::default()
                },
            ),
            vec![0.5, 0.5],
        ),
    ] {
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bool".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "true".to_owned(),
        ];
        for seconds in &post_set_advances {
            args.extend([
                "--runtime-advance-state-machine".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
            ]);
        }

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut state_machine = rust
            .state_machine_instance(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
        rust_reports.push((advanced, state_machine.clone()));
        assert!(state_machine.set_bool(0, true));
        for seconds in post_set_advances {
            let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
            rust_reports.push((advanced, state_machine.clone()));
        }
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_cubic_transition_interpolator_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_cubic_transition_interpolator_cpp.riv";
    let bytes = synthetic_state_machine_input_transition_with_options(
        8247,
        SyntheticInputTransitionKind::Bool,
        SyntheticTransitionOptions {
            duration: 1000,
            cubic_transition_interpolator: true,
            ..Default::default()
        },
    );
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(0, true));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.5);
    rust_reports.push((advanced, state_machine.clone()));
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_elastic_transition_interpolator_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_elastic_transition_interpolator_cpp.riv";
    let bytes = synthetic_state_machine_input_transition_with_options(
        8248,
        SyntheticInputTransitionKind::Bool,
        SyntheticTransitionOptions {
            duration: 1000,
            elastic_transition_interpolator: true,
            ..Default::default()
        },
    );
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(0, true));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.5);
    rust_reports.push((advanced, state_machine.clone()));
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_early_exit_transition_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_early_exit_transition_cpp.riv";
    let bytes = synthetic_state_machine_early_exit_transition(8249);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(0, true));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(1, true));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.5);
    rust_reports.push((advanced, state_machine.clone()));
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_blend_state_early_exit_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_blend_state_early_exit_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_early_exit(8264);
    let args = [
        "--runtime-set-state-machine-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "2".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(state_machine.set_number(0, 0.5));
    let mut rust_reports = Vec::new();
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(1, true));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(2, true));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.5);
    rust_reports.push((advanced, state_machine.clone()));
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_random_transition_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_random_transition_cpp.riv";
    let bytes = synthetic_state_machine_random_transition(8250);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(0, true));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_blend_state_random_transition_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (label, bytes, number_value) in [
        (
            "synthetic/runtime_state_machine_blend_state_1d_random_transition_cpp.riv",
            synthetic_state_machine_blend_state_random_transition(
                8265,
                SyntheticRandomBlendSource::Blend1D,
            ),
            0.5,
        ),
        (
            "synthetic/runtime_state_machine_blend_state_direct_random_transition_cpp.riv",
            synthetic_state_machine_blend_state_random_transition(
                8266,
                SyntheticRandomBlendSource::Direct,
            ),
            50.0,
        ),
    ] {
        let args = [
            "--runtime-set-state-machine-number".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            number_value.to_string(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bool".to_owned(),
            "0".to_owned(),
            "1".to_owned(),
            "true".to_owned(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0.5".to_owned(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0.5".to_owned(),
        ];

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut state_machine = rust
            .state_machine_instance(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        assert!(state_machine.set_number(0, number_value));
        let mut rust_reports = Vec::new();
        let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
        rust_reports.push((advanced, state_machine.clone()));
        assert!(state_machine.set_bool(1, true));
        for seconds in [0.0, 0.5, 0.5] {
            let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
            rust_reports.push((advanced, state_machine.clone()));
        }
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_fire_events_match_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_fire_events_cpp.riv";
    let bytes = synthetic_state_machine_fire_events(8251);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(0, true));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_scheduled_listener_fire_events_match_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_scheduled_listener_fire_events_cpp.riv";
    let bytes = synthetic_state_machine_scheduled_listener_fire_events(8252);
    let args = [
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(0, true));
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_scheduled_listener_input_changes_match_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (file_id, label, kind) in [
        (
            8253,
            "synthetic/runtime_state_machine_scheduled_listener_bool_change_cpp.riv",
            SyntheticInputTransitionKind::Bool,
        ),
        (
            8254,
            "synthetic/runtime_state_machine_scheduled_listener_number_change_cpp.riv",
            SyntheticInputTransitionKind::Number,
        ),
        (
            8255,
            "synthetic/runtime_state_machine_scheduled_listener_trigger_change_cpp.riv",
            SyntheticInputTransitionKind::Trigger,
        ),
    ] {
        let bytes = synthetic_state_machine_scheduled_listener_input_change(file_id, kind);
        let args = [
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ];

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut state_machine = rust
            .state_machine_instance(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
        let rust_reports = [(advanced, state_machine.clone())];
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_blend_state_1d_input_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_blend_state_1d_input_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_1d_input(8256);
    let args = [
        "--runtime-set-state-machine-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(state_machine.set_number(0, 0.5));
    let rust_reports = [
        (
            rust.advance_state_machine_instance(&mut state_machine, 0.0),
            state_machine.clone(),
        ),
        (
            rust.advance_state_machine_instance(&mut state_machine, 1.0),
            state_machine.clone(),
        ),
    ];
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_blend_state_direct_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_blend_state_direct_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_direct(8257);
    let args = [
        "--runtime-set-state-machine-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "50".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(state_machine.set_number(0, 50.0));
    let rust_reports = [
        (
            rust.advance_state_machine_instance(&mut state_machine, 0.0),
            state_machine.clone(),
        ),
        (
            rust.advance_state_machine_instance(&mut state_machine, 1.0),
            state_machine.clone(),
        ),
    ];
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_bindable_blend_sources_match_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (label, bytes) in [
        (
            "synthetic/runtime_state_machine_blend_state_1d_view_model_cpp.riv",
            synthetic_state_machine_blend_state_1d_view_model(8267, 0.5),
        ),
        (
            "synthetic/runtime_state_machine_direct_bindable_blend_state_cpp.riv",
            synthetic_state_machine_direct_bindable_blend_state(8268, 50.0),
        ),
    ] {
        let args = [
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "1".to_owned(),
        ];

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut state_machine = rust
            .state_machine_instance(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let rust_reports = [
            (
                rust.advance_state_machine_instance(&mut state_machine, 0.0),
                state_machine.clone(),
            ),
            (
                rust.advance_state_machine_instance(&mut state_machine, 1.0),
                state_machine.clone(),
            ),
        ];
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_mutable_bindable_blend_sources_match_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (label, bytes, value) in [
        (
            "synthetic/runtime_state_machine_mutable_blend_state_1d_view_model_cpp.riv",
            synthetic_state_machine_blend_state_1d_view_model(8269, 0.0),
            1.0,
        ),
        (
            "synthetic/runtime_state_machine_mutable_direct_bindable_blend_state_cpp.riv",
            synthetic_state_machine_direct_bindable_blend_state(8270, 0.0),
            100.0,
        ),
    ] {
        let args = [
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-set-state-machine-bindable-number".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            value.to_string(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "1".to_owned(),
        ];

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut state_machine = rust
            .state_machine_instance(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let rust_reports = [
            (
                rust.advance_state_machine_instance(&mut state_machine, 0.0),
                state_machine.clone(),
            ),
            {
                assert!(
                    state_machine.set_bindable_number_for_data_bind(0, value),
                    "{label} failed to mutate bindable number"
                );
                (
                    rust.advance_state_machine_instance(&mut state_machine, 1.0),
                    state_machine.clone(),
                )
            },
        ];
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_viewmodel_number_conditions_match_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (label, bytes, bind_context, mutated_value) in [
        (
            "synthetic/runtime_state_machine_viewmodel_number_condition_no_context_cpp.riv",
            synthetic_state_machine_viewmodel_number_condition(8271, 4.0, 3.0),
            false,
            None,
        ),
        (
            "synthetic/runtime_state_machine_viewmodel_number_condition_static_cpp.riv",
            synthetic_state_machine_viewmodel_number_condition(8272, 4.0, 3.0),
            true,
            None,
        ),
        (
            "synthetic/runtime_state_machine_viewmodel_number_condition_mutated_cpp.riv",
            synthetic_state_machine_viewmodel_number_condition(8273, 0.0, 3.0),
            true,
            Some(4.0),
        ),
    ] {
        let mut args = vec![
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ];
        if bind_context {
            args.extend([
                "--runtime-bind-empty-state-machine-context".to_owned(),
                "0".to_owned(),
            ]);
        }
        if let Some(value) = mutated_value {
            args.extend([
                "--runtime-set-state-machine-bindable-number".to_owned(),
                "0".to_owned(),
                "0".to_owned(),
                value.to_string(),
            ]);
        }
        args.extend([
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            "1".to_owned(),
        ]);

        let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut state_machine = rust
            .state_machine_instance(0)
            .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

        let mut rust_reports = Vec::new();
        rust_reports.push((
            rust.advance_state_machine_instance(&mut state_machine, 0.0),
            state_machine.clone(),
        ));
        if bind_context {
            assert!(
                state_machine.bind_empty_data_context(),
                "{label} failed to bind empty data context"
            );
        }
        if let Some(value) = mutated_value {
            assert!(
                state_machine.set_bindable_number_for_data_bind(0, value),
                "{label} failed to mutate bindable number"
            );
        }
        rust_reports.push((
            rust.advance_state_machine_instance(&mut state_machine, 0.0),
            state_machine.clone(),
        ));
        rust_reports.push((
            rust.advance_state_machine_instance(&mut state_machine, 1.0),
            state_machine.clone(),
        ));
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        assert_eq!(
            cpp_artboard.runtime_state_machine_advances.len(),
            rust_reports.len(),
            "{label} state-machine report count mismatch"
        );
        for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
            .runtime_state_machine_advances
            .iter()
            .zip(&rust_reports)
        {
            compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
        }
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn state_machine_direct_blend_state_transition_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_direct_blend_state_transition_cpp.riv";
    let bytes = synthetic_state_machine_direct_blend_state_transition(8260);
    let args = [
        "--runtime-set-state-machine-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "50".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(state_machine.set_number(0, 50.0));
    let mut rust_reports = Vec::new();
    for seconds in [0.0, 0.5] {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    assert!(state_machine.set_bool(1, true));
    for seconds in [0.0, 0.5, 0.5] {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_blend_state_transition_exit_time_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_blend_state_transition_exit_time_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_transition(8258);
    let args = [
        "--runtime-set-state-machine-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(state_machine.set_number(0, 0.5));
    let mut rust_reports = Vec::new();
    for seconds in [0.0, 0.5] {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    assert!(state_machine.set_bool(1, true));
    for seconds in [0.0, 0.5, 0.5] {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_blend_state_transition_reset_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_blend_state_transition_reset_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_transition_reset(8259);
    let args = [
        "--runtime-set-state-machine-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(state_machine.set_number(0, 0.5));
    let mut rust_reports = Vec::new();
    for seconds in [0.0, 0.5] {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    assert!(state_machine.set_bool(1, true));
    for seconds in [0.0, 0.5, 0.5] {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_blend_state_percentage_duration_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_blend_state_percentage_duration_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_percentage_duration(8261);
    let args = [
        "--runtime-set-state-machine-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(state_machine.set_number(0, 0.5));
    let mut rust_reports = Vec::new();
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(1, true));
    for seconds in [0.0, 0.5] {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_blend_state_percentage_exit_time_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_blend_state_percentage_exit_time_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_percentage_exit_time(8262);
    let args = [
        "--runtime-set-state-machine-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(state_machine.set_number(0, 0.5));
    let mut rust_reports = Vec::new();
    let advanced = rust.advance_state_machine_instance(&mut state_machine, 0.0);
    rust_reports.push((advanced, state_machine.clone()));
    assert!(state_machine.set_bool(1, true));
    for seconds in [1.0, 0.5, 0.5, 0.5] {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_blend_state_pause_on_exit_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_blend_state_pause_on_exit_cpp.riv";
    let bytes = synthetic_state_machine_blend_state_pause_on_exit(8263);
    let args = [
        "--runtime-set-state-machine-number".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-set-state-machine-bool".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "true".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
        "--runtime-advance-state-machine".to_owned(),
        "0".to_owned(),
        "0.5".to_owned(),
    ];

    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    assert!(state_machine.set_number(0, 0.5));
    let mut rust_reports = Vec::new();
    for seconds in [0.0, 0.5] {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    assert!(state_machine.set_bool(1, true));
    for seconds in [0.0, 0.5, 0.5] {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

#[test]
fn state_machine_animation_state_advance_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_state_machine_animation_state.riv";
    let bytes = synthetic_state_machine_animation_state(8231);
    let advances = [0.0_f32, 0.5, 0.0];
    let cpp = read_cpp_probe_bytes_with_args(
        &probe,
        label,
        &bytes,
        &[
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            advances[0].to_string(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            advances[1].to_string(),
            "--runtime-advance-state-machine".to_owned(),
            "0".to_owned(),
            advances[2].to_string(),
        ],
    );
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let mut state_machine = rust
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("missing Rust state-machine instance for {label}"));

    let mut rust_reports = Vec::new();
    for seconds in advances {
        let advanced = rust.advance_state_machine_instance(&mut state_machine, seconds);
        rust_reports.push((advanced, state_machine.clone()));
    }
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    assert_eq!(
        cpp_artboard.runtime_state_machine_advances.len(),
        rust_reports.len(),
        "{label} state-machine report count mismatch"
    );
    for (cpp_state_machine, (advanced, rust_state_machine)) in cpp_artboard
        .runtime_state_machine_advances
        .iter()
        .zip(&rust_reports)
    {
        compare_state_machine_advance(cpp_state_machine, rust_state_machine, *advanced, label);
    }
    compare_cpp_runtime_update(&cpp, &rust, &report, label);
}

fn assert_close(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "{label} mismatch: expected {expected}, got {actual}"
    );
}

fn assert_path_commands_close(
    actual: &[RuntimePathCommand],
    expected: &[RuntimePathCommand],
    label: &str,
) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{label} command count mismatch: expected {expected:?}, got {actual:?}"
    );
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        let point_label = |field: &str| format!("{label}[{index}].{field}");
        match (actual, expected) {
            (
                RuntimePathCommand::Move {
                    x: actual_x,
                    y: actual_y,
                },
                RuntimePathCommand::Move {
                    x: expected_x,
                    y: expected_y,
                },
            )
            | (
                RuntimePathCommand::Line {
                    x: actual_x,
                    y: actual_y,
                },
                RuntimePathCommand::Line {
                    x: expected_x,
                    y: expected_y,
                },
            ) => {
                assert_close(*actual_x, *expected_x, &point_label("x"));
                assert_close(*actual_y, *expected_y, &point_label("y"));
            }
            (
                RuntimePathCommand::Cubic {
                    x1: actual_x1,
                    y1: actual_y1,
                    x2: actual_x2,
                    y2: actual_y2,
                    x3: actual_x3,
                    y3: actual_y3,
                },
                RuntimePathCommand::Cubic {
                    x1: expected_x1,
                    y1: expected_y1,
                    x2: expected_x2,
                    y2: expected_y2,
                    x3: expected_x3,
                    y3: expected_y3,
                },
            ) => {
                assert_close(*actual_x1, *expected_x1, &point_label("x1"));
                assert_close(*actual_y1, *expected_y1, &point_label("y1"));
                assert_close(*actual_x2, *expected_x2, &point_label("x2"));
                assert_close(*actual_y2, *expected_y2, &point_label("y2"));
                assert_close(*actual_x3, *expected_x3, &point_label("x3"));
                assert_close(*actual_y3, *expected_y3, &point_label("y3"));
            }
            (RuntimePathCommand::Close, RuntimePathCommand::Close) => {}
            _ => panic!("{label}[{index}] command mismatch: expected {expected:?}, got {actual:?}"),
        }
    }
}

fn compare_animation_advance(
    cpp: &CppRuntimeAnimationAdvance,
    runtime_animation: &rive_runtime::RuntimeLinearAnimation,
    rust: &rive_runtime::LinearAnimationInstance,
    keep_going: bool,
    keep_going_after: bool,
    label: &str,
) {
    assert_eq!(cpp.animation_index, rust.animation_index(), "{label}");
    assert_eq!(cpp.advanced, keep_going, "{label} advance return mismatch");
    assert_eq!(
        cpp.keep_going, keep_going_after,
        "{label} keepGoing mismatch"
    );
    assert_close(cpp.time, rust.time(), &format!("{label} time"));
    assert_close(
        cpp.direction,
        rust.direction(),
        &format!("{label} direction"),
    );
    assert_close(
        cpp.directed_speed,
        rust.directed_speed(runtime_animation),
        &format!("{label} directedSpeed"),
    );
    assert_close(
        cpp.total_time,
        rust.total_time(),
        &format!("{label} totalTime"),
    );
    assert_close(
        cpp.last_total_time,
        rust.last_total_time(),
        &format!("{label} lastTotalTime"),
    );
    assert_close(
        cpp.spilled_time,
        rust.spilled_time(),
        &format!("{label} spilledTime"),
    );
    assert_eq!(cpp.did_loop, rust.did_loop(), "{label} didLoop mismatch");
    assert_eq!(
        cpp.loop_value as u64,
        rust.loop_value().unwrap_or(runtime_animation.loop_value),
        "{label} loopValue override mismatch"
    );
}

fn compare_state_machine_advance(
    cpp: &CppRuntimeStateMachineAdvance,
    rust: &rive_runtime::StateMachineInstance,
    advanced: bool,
    label: &str,
) {
    assert_eq!(
        cpp.state_machine_index,
        rust.state_machine_index(),
        "{label} stateMachineIndex mismatch"
    );
    assert_eq!(cpp.advanced, advanced, "{label} advance return mismatch");
    assert_eq!(
        cpp.current_animation_count,
        rust.current_animation_count(),
        "{label} currentAnimationCount mismatch"
    );
    assert_eq!(
        cpp.changed_state_count,
        rust.changed_state_count(),
        "{label} changedStateCount mismatch"
    );
    assert_eq!(
        cpp.reported_event_count,
        rust.reported_event_count(),
        "{label} reportedEventCount mismatch"
    );
    for (animation_index, cpp_animation) in cpp.current_animations.iter().enumerate() {
        let rust_animation = rust.current_animation(animation_index).unwrap_or_else(|| {
            panic!("missing Rust current animation {animation_index} for {label}")
        });
        assert_close(
            cpp_animation.time,
            rust_animation.time(),
            &format!("{label} current animation {animation_index} time"),
        );
        assert_eq!(
            cpp_animation.did_loop,
            rust_animation.did_loop(),
            "{label} current animation {animation_index} didLoop mismatch"
        );
    }
    for (event_index, cpp_event) in cpp.reported_events.iter().enumerate() {
        let rust_event = rust
            .reported_event(event_index)
            .unwrap_or_else(|| panic!("missing Rust reported event {event_index} for {label}"));
        assert_eq!(
            cpp_event.event_local,
            Some(rust_event.event_local_index()),
            "{label} reported event {event_index} local ID mismatch"
        );
        assert_eq!(
            cpp_event.event_core_type,
            Some(rust_event.event_core_type()),
            "{label} reported event {event_index} core type mismatch"
        );
        assert_eq!(
            cpp_event.event_name.as_deref(),
            rust_event.name(),
            "{label} reported event {event_index} name mismatch"
        );
        assert_close(
            cpp_event.seconds_delay,
            rust_event.seconds_delay(),
            &format!("{label} reported event {event_index} secondsDelay"),
        );
    }
}

fn compare_cpp_runtime_update(
    cpp: &CppProbeFile,
    rust: &ArtboardInstance,
    report: &rive_runtime::UpdateComponentsReport,
    label: &str,
) {
    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    let cpp_update = cpp_artboard
        .runtime_update
        .as_ref()
        .unwrap_or_else(|| panic!("missing C++ runtimeUpdate for {label}"));

    assert_eq!(cpp_update.did_update, report.did_update);
    assert_eq!(
        cpp_update.has_components_dirt,
        rust.has_dirt(ComponentDirt::COMPONENTS)
    );

    for cpp_component in &cpp_update.components {
        let rust_component = rust
            .component(cpp_component.local_id)
            .unwrap_or_else(|| panic!("missing Rust component {}", cpp_component.local_id));
        compare_component(cpp_component, rust_component, label);
    }
}

fn compare_component(cpp: &CppRuntimeComponent, rust: &RuntimeComponent, label: &str) {
    assert_eq!(
        cpp.graph_order, rust.graph_order,
        "graph order mismatch for {label}"
    );
    assert_eq!(
        cpp.dirt, rust.dirt.0,
        "dirt mismatch for local {} in {label}",
        cpp.local_id
    );
    assert_eq!(
        cpp.collapsed,
        rust.is_collapsed(),
        "collapsed flag mismatch for local {} in {label}",
        cpp.local_id
    );

    let rust_local = rust
        .capabilities
        .transform
        .then_some(rust.transform.local_transform);
    compare_mat2d(
        cpp.local_transform,
        rust_local,
        "local transform",
        cpp.local_id,
        label,
    );

    let rust_world = rust
        .capabilities
        .world_transform
        .then_some(rust.transform.world_transform);
    compare_mat2d(
        cpp.world_transform,
        rust_world,
        "world transform",
        cpp.local_id,
        label,
    );

    let rust_opacity = rust
        .capabilities
        .transform
        .then_some(rust.transform.render_opacity);
    compare_optional_f32(
        cpp.render_opacity,
        rust_opacity,
        "render opacity",
        cpp.local_id,
        label,
    );
}

fn compare_mat2d(
    cpp: Option<[f32; 6]>,
    rust: Option<Mat2D>,
    field: &str,
    local_id: usize,
    label: &str,
) {
    match (cpp, rust) {
        (Some(cpp), Some(rust)) => {
            for (index, (cpp_value, rust_value)) in cpp.into_iter().zip(rust.0).enumerate() {
                assert!(
                    (cpp_value - rust_value).abs() <= 0.0001,
                    "{field}[{index}] mismatch for local {local_id} in {label}: C++ {cpp_value}, Rust {rust_value}"
                );
            }
        }
        (None, None) => {}
        _ => panic!("{field} presence mismatch for local {local_id} in {label}"),
    }
}

fn compare_optional_f32(
    cpp: Option<f32>,
    rust: Option<f32>,
    field: &str,
    local_id: usize,
    label: &str,
) {
    match (cpp, rust) {
        (Some(cpp), Some(rust)) => assert!(
            (cpp - rust).abs() <= 0.0001,
            "{field} mismatch for local {local_id} in {label}: C++ {cpp}, Rust {rust}"
        ),
        (None, None) => {}
        _ => panic!("{field} presence mismatch for local {local_id} in {label}"),
    }
}

#[derive(Debug, Deserialize)]
struct CppProbeFile {
    artboards: Vec<CppArtboard>,
}

#[derive(Debug, Deserialize)]
struct CppArtboard {
    #[serde(rename = "objectCount")]
    object_count: usize,
    objects: Vec<Option<CppObject>>,
    #[serde(default, rename = "drawCommandStream")]
    draw_command_stream: Vec<CppDrawCommand>,
    #[serde(rename = "runtimeUpdate")]
    runtime_update: Option<CppRuntimeUpdate>,
    #[serde(default, rename = "runtimeAnimationAdvances")]
    runtime_animation_advances: Vec<CppRuntimeAnimationAdvance>,
    #[serde(default, rename = "runtimeStateMachineAdvances")]
    runtime_state_machine_advances: Vec<CppRuntimeStateMachineAdvance>,
}

#[derive(Debug, Deserialize)]
struct CppDrawCommand {
    #[serde(rename = "localId")]
    local_id: Option<usize>,
    #[serde(rename = "isClipStart")]
    is_clip_start: bool,
    #[serde(rename = "isClipEnd")]
    is_clip_end: bool,
    #[serde(rename = "needsSaveOperation")]
    needs_save_operation: bool,
    #[serde(default, rename = "shapePaintCommands")]
    shape_paint_commands: Vec<CppShapePaintCommand>,
}

impl CppDrawCommand {
    fn kind(&self) -> RuntimeDrawCommandKind {
        if self.is_clip_start {
            RuntimeDrawCommandKind::ClipStart
        } else if self.is_clip_end {
            RuntimeDrawCommandKind::ClipEnd
        } else {
            RuntimeDrawCommandKind::Draw
        }
    }
}

#[derive(Debug, Deserialize)]
struct CppShapePaintCommand {
    #[serde(rename = "paintLocal")]
    paint_local: Option<usize>,
    #[serde(rename = "mutatorLocal")]
    mutator_local: Option<usize>,
    #[serde(rename = "paintType")]
    paint_type: String,
    #[serde(rename = "pathKind")]
    path_kind: String,
    #[serde(rename = "blendModeValue")]
    blend_mode_value: u32,
    #[serde(rename = "renderBlendModeValue")]
    render_blend_mode_value: u32,
    #[serde(rename = "paintState")]
    paint_state: Option<CppShapePaintState>,
    #[serde(default)]
    feather: Option<CppFeatherState>,
    #[serde(default, rename = "pathCommands")]
    path_commands: Vec<CppPathCommand>,
    #[serde(rename = "needsSaveOperation")]
    needs_save_operation: bool,
}

impl CppShapePaintCommand {
    fn paint_type(&self) -> RuntimeShapePaintKind {
        match self.paint_type.as_str() {
            "fill" => RuntimeShapePaintKind::Fill,
            "stroke" => RuntimeShapePaintKind::Stroke,
            _ => RuntimeShapePaintKind::Unknown,
        }
    }

    fn path_kind(&self) -> RuntimeShapePaintPathKind {
        match self.path_kind.as_str() {
            "local" => RuntimeShapePaintPathKind::Local,
            "localClockwise" => RuntimeShapePaintPathKind::LocalClockwise,
            "world" => RuntimeShapePaintPathKind::World,
            other => panic!("unexpected C++ shape paint path kind {other}"),
        }
    }

    fn paint_state(&self) -> Option<RuntimeShapePaintState> {
        let state = self.paint_state.as_ref()?;
        match state.kind.as_str() {
            "solidColor" => Some(RuntimeShapePaintState::SolidColor {
                color: state.color,
                render_color: state.render_color,
            }),
            "linearGradient" => Some(RuntimeShapePaintState::LinearGradient {
                start_x: state.start_x,
                start_y: state.start_y,
                end_x: state.end_x,
                end_y: state.end_y,
                opacity: state.opacity,
                render_opacity: state.render_opacity,
                stops: state
                    .stops
                    .iter()
                    .map(CppGradientStopState::gradient_stop)
                    .collect(),
            }),
            "radialGradient" => Some(RuntimeShapePaintState::RadialGradient {
                start_x: state.start_x,
                start_y: state.start_y,
                end_x: state.end_x,
                end_y: state.end_y,
                opacity: state.opacity,
                render_opacity: state.render_opacity,
                stops: state
                    .stops
                    .iter()
                    .map(CppGradientStopState::gradient_stop)
                    .collect(),
            }),
            other => panic!("unexpected C++ shape paint state kind {other}"),
        }
    }

    fn feather_state(&self) -> Option<RuntimeFeatherState> {
        self.feather.as_ref().map(CppFeatherState::feather_state)
    }

    fn path_commands(&self) -> Vec<RuntimePathCommand> {
        self.path_commands
            .iter()
            .map(CppPathCommand::path_command)
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct CppShapePaintState {
    kind: String,
    #[serde(default, rename = "startX")]
    start_x: f32,
    #[serde(default, rename = "startY")]
    start_y: f32,
    #[serde(default, rename = "endX")]
    end_x: f32,
    #[serde(default, rename = "endY")]
    end_y: f32,
    #[serde(default)]
    opacity: f32,
    #[serde(default, rename = "renderOpacity")]
    render_opacity: f32,
    #[serde(default)]
    stops: Vec<CppGradientStopState>,
    #[serde(default)]
    color: u32,
    #[serde(default, rename = "renderColor")]
    render_color: u32,
}

#[derive(Debug, Deserialize)]
struct CppGradientStopState {
    color: u32,
    #[serde(rename = "renderColor")]
    render_color: u32,
    position: f32,
}

impl CppGradientStopState {
    fn gradient_stop(&self) -> RuntimeGradientStop {
        RuntimeGradientStop {
            color: self.color,
            render_color: self.render_color,
            position: self.position,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CppFeatherState {
    local: usize,
    #[serde(rename = "spaceValue")]
    space_value: u32,
    strength: f32,
    #[serde(rename = "offsetX")]
    offset_x: f32,
    #[serde(rename = "offsetY")]
    offset_y: f32,
    inner: bool,
}

impl CppFeatherState {
    fn feather_state(&self) -> RuntimeFeatherState {
        RuntimeFeatherState {
            feather_local: self.local,
            space_value: self.space_value,
            strength: self.strength,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
            inner: self.inner,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CppPathCommand {
    verb: String,
    points: Vec<[f32; 2]>,
}

impl CppPathCommand {
    fn path_command(&self) -> RuntimePathCommand {
        match self.verb.as_str() {
            "move" => RuntimePathCommand::Move {
                x: self.points[0][0],
                y: self.points[0][1],
            },
            "line" => RuntimePathCommand::Line {
                x: self.points[0][0],
                y: self.points[0][1],
            },
            "cubic" => RuntimePathCommand::Cubic {
                x1: self.points[0][0],
                y1: self.points[0][1],
                x2: self.points[1][0],
                y2: self.points[1][1],
                x3: self.points[2][0],
                y3: self.points[2][1],
            },
            "close" => RuntimePathCommand::Close,
            other => panic!("unexpected C++ raw path verb {other}"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CppObject {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "isComponent")]
    is_component: bool,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeUpdate {
    #[serde(rename = "didUpdate")]
    did_update: bool,
    #[serde(rename = "hasComponentsDirt")]
    has_components_dirt: bool,
    components: Vec<CppRuntimeComponent>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeAnimationAdvance {
    #[serde(rename = "animationIndex")]
    animation_index: usize,
    advanced: bool,
    #[serde(rename = "keepGoing")]
    keep_going: bool,
    time: f32,
    direction: f32,
    #[serde(rename = "directedSpeed")]
    directed_speed: f32,
    #[serde(rename = "totalTime")]
    total_time: f32,
    #[serde(rename = "lastTotalTime")]
    last_total_time: f32,
    #[serde(rename = "spilledTime")]
    spilled_time: f32,
    #[serde(rename = "didLoop")]
    did_loop: bool,
    #[serde(rename = "loopValue")]
    loop_value: i64,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineAdvance {
    #[serde(rename = "stateMachineIndex")]
    state_machine_index: usize,
    advanced: bool,
    #[serde(rename = "currentAnimationCount")]
    current_animation_count: usize,
    #[serde(rename = "changedStateCount")]
    changed_state_count: usize,
    #[serde(default, rename = "reportedEventCount")]
    reported_event_count: usize,
    #[serde(rename = "currentAnimations")]
    current_animations: Vec<CppRuntimeStateMachineCurrentAnimation>,
    #[serde(default, rename = "reportedEvents")]
    reported_events: Vec<CppRuntimeStateMachineReportedEvent>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineCurrentAnimation {
    time: f32,
    #[serde(rename = "didLoop")]
    did_loop: bool,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeStateMachineReportedEvent {
    #[serde(rename = "eventLocal")]
    event_local: Option<usize>,
    #[serde(rename = "eventCoreType")]
    event_core_type: Option<u32>,
    #[serde(rename = "eventName")]
    event_name: Option<String>,
    #[serde(rename = "secondsDelay")]
    seconds_delay: f32,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeComponent {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "graphOrder")]
    graph_order: usize,
    dirt: u16,
    collapsed: bool,
    #[serde(rename = "worldTransform")]
    world_transform: Option<[f32; 6]>,
    #[serde(rename = "localTransform")]
    local_transform: Option<[f32; 6]>,
    #[serde(rename = "renderOpacity")]
    render_opacity: Option<f32>,
}
