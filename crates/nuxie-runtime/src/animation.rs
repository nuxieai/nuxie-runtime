use crate::artboard_data_bind::build_key_frame_data_bind_templates;
use crate::data_bind_graph::{
    RuntimeDataBindGraph, RuntimeDataBindGraphApplyPhase, RuntimeDataBindGraphTarget,
    RuntimeKeyFrameDataBindTemplate,
};
use crate::draw::color_lerp;
use crate::properties::{
    artboard_index_for_graph, mix_value, runtime_object_bool_property_by_key,
    runtime_object_color_property_by_key, runtime_object_double_property_by_key,
    runtime_object_field_kind_by_key, solid_color_value_property_key, transform_property_for_key,
};
use crate::{ArtboardInstance, InstanceSlot, StateMachineReportedEvent, TransformProperty};
use nuxie_binary::{RuntimeFile, RuntimeImportStatus, RuntimeObject};
use nuxie_graph::ArtboardGraph;
use nuxie_schema::{
    CoreRegistryFieldKind, FieldKind, core_registry_field_kind_by_property_key,
    definition_by_type_key, is_callback_property_key, object_supports_property,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RuntimeInterpolator {
    CubicEase {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    CubicValue {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Elastic {
        amplitude: f32,
        period: f32,
        easing_value: u64,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeJoystick {
    pub(crate) local_id: usize,
    pub(crate) can_apply_before_update: bool,
    pub(crate) x_animation_index: Option<usize>,
    pub(crate) y_animation_index: Option<usize>,
    pub(crate) nested_remap_dependents: Vec<usize>,
}

pub(crate) fn build_runtime_joysticks(
    graph: &ArtboardGraph,
    linear_animations: &[RuntimeLinearAnimation],
) -> Vec<RuntimeJoystick> {
    graph
        .joysticks
        .iter()
        .map(|joystick| RuntimeJoystick {
            local_id: joystick.local_id,
            can_apply_before_update: joystick.can_apply_before_update,
            x_animation_index: joystick.x_animation_global.and_then(|global_id| {
                linear_animations
                    .iter()
                    .position(|animation| animation.global_id == global_id)
            }),
            y_animation_index: joystick.y_animation_global.and_then(|global_id| {
                linear_animations
                    .iter()
                    .position(|animation| animation.global_id == global_id)
            }),
            nested_remap_dependents: joystick
                .nested_remap_dependents
                .iter()
                .map(|dependent| dependent.local_id)
                .collect(),
        })
        .collect()
}

impl RuntimeInterpolator {
    pub(crate) fn from_object(object: &RuntimeObject) -> Option<Self> {
        match object.type_name {
            "CubicEaseInterpolator" => Some(Self::CubicEase {
                x1: object.double_property("x1").unwrap_or(0.42),
                y1: object.double_property("y1").unwrap_or(0.0),
                x2: object.double_property("x2").unwrap_or(0.58),
                y2: object.double_property("y2").unwrap_or(1.0),
            }),
            "CubicValueInterpolator" => Some(Self::CubicValue {
                x1: object.double_property("x1").unwrap_or(0.42),
                y1: object.double_property("y1").unwrap_or(0.0),
                x2: object.double_property("x2").unwrap_or(0.58),
                y2: object.double_property("y2").unwrap_or(1.0),
            }),
            "ElasticInterpolator" => Some(Self::Elastic {
                amplitude: object.double_property("amplitude").unwrap_or(1.0),
                period: object.double_property("period").unwrap_or(1.0),
                easing_value: object.uint_property("easingValue").unwrap_or(1),
            }),
            _ => None,
        }
    }

    pub(crate) fn transform_value(self, value_from: f32, value_to: f32, factor: f32) -> f32 {
        match self {
            Self::CubicValue { x1, y1, x2, y2 } => {
                let t = cubic_interpolator_get_t(factor, x1, x2);
                cubic_interpolator_calc_cubic_value(t, value_from, y1, y2, value_to)
            }
            _ => value_from + (value_to - value_from) * self.transform(factor),
        }
    }

    pub(crate) fn transform(self, factor: f32) -> f32 {
        match self {
            Self::CubicEase { x1, y1, x2, y2 } => {
                let t = cubic_interpolator_get_t(factor, x1, x2);
                cubic_interpolator_calc_bezier(t, y1, y2)
            }
            Self::CubicValue { .. } => factor,
            Self::Elastic {
                amplitude,
                period,
                easing_value,
            } => elastic_interpolator_transform(factor, amplitude, period, easing_value),
        }
    }
}

fn cubic_interpolator_calc_bezier(t: f32, a1: f32, a2: f32) -> f32 {
    (((1.0 - 3.0 * a2 + 3.0 * a1) * t + (3.0 * a2 - 6.0 * a1)) * t + (3.0 * a1)) * t
}

fn cubic_interpolator_calc_cubic_value(
    t: f32,
    value_from: f32,
    control_1: f32,
    control_2: f32,
    value_to: f32,
) -> f32 {
    let a = value_to + 3.0 * (control_1 - control_2) - value_from;
    let b = 3.0 * (control_2 - control_1 * 2.0 + value_from);
    let c = 3.0 * (control_1 - value_from);
    ((a * t + b) * t + c) * t + value_from
}

fn cubic_interpolator_slope(t: f32, a1: f32, a2: f32) -> f32 {
    3.0 * (1.0 - 3.0 * a2 + 3.0 * a1) * t * t + 2.0 * (3.0 * a2 - 6.0 * a1) * t + (3.0 * a1)
}

fn cubic_interpolator_get_t(x: f32, x1: f32, x2: f32) -> f32 {
    const SPLINE_TABLE_SIZE: usize = 11;
    const SAMPLE_STEP_SIZE: f32 = 1.0 / (SPLINE_TABLE_SIZE as f32 - 1.0);
    const NEWTON_ITERATIONS: usize = 4;
    const NEWTON_MIN_SLOPE: f32 = 0.001;
    const SUBDIVISION_PRECISION: f32 = 0.0000001;
    const SUBDIVISION_MAX_ITERATIONS: usize = 10;

    let mut values = [0.0; SPLINE_TABLE_SIZE];
    for (i, value) in values.iter_mut().enumerate() {
        *value = cubic_interpolator_calc_bezier(i as f32 * SAMPLE_STEP_SIZE, x1, x2);
    }

    let mut interval_start = 0.0;
    let mut current_sample = 1;
    let last_sample = SPLINE_TABLE_SIZE - 1;
    while current_sample != last_sample && values[current_sample] <= x {
        interval_start += SAMPLE_STEP_SIZE;
        current_sample += 1;
    }
    current_sample -= 1;

    let dist = (x - values[current_sample]) / (values[current_sample + 1] - values[current_sample]);
    let mut guess_for_t = interval_start + dist * SAMPLE_STEP_SIZE;
    let initial_slope = cubic_interpolator_slope(guess_for_t, x1, x2);
    if initial_slope >= NEWTON_MIN_SLOPE {
        for _ in 0..NEWTON_ITERATIONS {
            let current_slope = cubic_interpolator_slope(guess_for_t, x1, x2);
            if current_slope == 0.0 {
                return guess_for_t;
            }
            let current_x = cubic_interpolator_calc_bezier(guess_for_t, x1, x2) - x;
            guess_for_t -= current_x / current_slope;
        }
        guess_for_t
    } else if initial_slope == 0.0 {
        guess_for_t
    } else {
        let mut upper_bound = interval_start + SAMPLE_STEP_SIZE;
        let mut iterations = 0;
        loop {
            let current_t = interval_start + (upper_bound - interval_start) / 2.0;
            let current_x = cubic_interpolator_calc_bezier(current_t, x1, x2) - x;
            if current_x > 0.0 {
                upper_bound = current_t;
            } else {
                interval_start = current_t;
            }
            iterations += 1;
            if current_x.abs() <= SUBDIVISION_PRECISION || iterations >= SUBDIVISION_MAX_ITERATIONS
            {
                return current_t;
            }
        }
    }
}

fn elastic_interpolator_transform(
    factor: f32,
    amplitude: f32,
    serialized_period: f32,
    easing_value: u64,
) -> f32 {
    let period = if serialized_period == 0.0 {
        0.5
    } else {
        serialized_period
    };
    let shift = if amplitude < 1.0 {
        period / 4.0
    } else {
        period / (2.0 * std::f32::consts::PI) * (1.0 / amplitude).asin()
    };

    match easing_value {
        0 => elastic_ease_in(factor, amplitude, period, shift),
        1 => elastic_ease_out(factor, amplitude, period, shift),
        2 => elastic_ease_in_out(factor, amplitude, period, shift),
        _ => factor,
    }
}

fn elastic_actual_amplitude(time: f32, amplitude: f32, shift: f32) -> f32 {
    if amplitude < 1.0 {
        let shift_abs = shift.abs();
        let time_abs = time.abs();
        if time_abs < shift_abs {
            let l = time_abs / shift_abs;
            return (amplitude * l) + (1.0 - l);
        }
    }

    amplitude
}

fn elastic_ease_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    actual_amplitude
        * 2.0_f32.powf(10.0 * -time)
        * ((time - shift) * (2.0 * std::f32::consts::PI) / period).sin()
        + 1.0
}

fn elastic_ease_in(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor - 1.0;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    -(actual_amplitude
        * 2.0_f32.powf(10.0 * time)
        * ((-time - shift) * (2.0 * std::f32::consts::PI) / period).sin())
}

fn elastic_ease_in_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor * 2.0 - 1.0;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    if time < 0.0 {
        -0.5 * actual_amplitude
            * 2.0_f32.powf(10.0 * time)
            * ((-time - shift) * (2.0 * std::f32::consts::PI) / period).sin()
    } else {
        0.5 * (actual_amplitude
            * 2.0_f32.powf(10.0 * -time)
            * ((time - shift) * (2.0 * std::f32::consts::PI) / period).sin())
            + 1.0
    }
}

fn callback_event_for_keyed_property(
    target_local_id: usize,
    target: &RuntimeObject,
    property_key: u16,
) -> Option<StateMachineReportedEvent> {
    if !is_callback_property_key(property_key) {
        return None;
    }
    let definition = definition_by_type_key(target.type_key)?;
    if !definition.is_a("Event") {
        return None;
    }
    let property = definition.property_by_key_in_hierarchy(property_key)?;
    if property.name != "trigger" {
        return None;
    }

    Some(StateMachineReportedEvent {
        event_local_index: target_local_id,
        event_core_type: u32::from(target.type_key),
        name: target
            .string_property("name")
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned),
        seconds_delay: 0.0,
    })
}

pub(crate) fn build_linear_animations(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    slots: &[InstanceSlot],
) -> Vec<RuntimeLinearAnimation> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let Some((start, end)) = artboard_object_range(file, graph.global_id as usize) else {
        return Vec::new();
    };

    let mut animations = Vec::<RuntimeLinearAnimation>::new();
    let mut current_animation = None;
    let mut current_keyed_object = None;
    let mut current_keyed_property = None;

    for global_id in start..end {
        let Some(object) = file.object(global_id) else {
            continue;
        };
        if file.import_status(global_id) != Some(RuntimeImportStatus::Imported) {
            continue;
        }

        if object.type_name == "LinearAnimation" {
            animations.push(RuntimeLinearAnimation {
                global_id: global_id as u32,
                name: object.string_property("name").map(Arc::<str>::from),
                fps: object.uint_property("fps").unwrap_or(60),
                duration: object.uint_property("duration").unwrap_or(60),
                speed: object.double_property("speed").unwrap_or(1.0),
                loop_value: object.uint_property("loopValue").unwrap_or(0),
                work_start: object.uint_property("workStart").unwrap_or(0),
                work_end: object.uint_property("workEnd").unwrap_or(0),
                enable_work_area: object.bool_property("enableWorkArea").unwrap_or(false),
                quantize: object.bool_property("quantize").unwrap_or(false),
                keyed_objects: Arc::new(Vec::new()),
                key_frame_data_bind_templates: Arc::new(Vec::new()),
                has_keyed_callbacks: false,
            });
            current_animation = Some(animations.len() - 1);
            current_keyed_object = None;
            current_keyed_property = None;
            continue;
        }

        let Some(animation_index) = current_animation else {
            continue;
        };

        if object.type_name == "KeyedObject" {
            let Some((object_id, target_local_id, _target)) =
                keyed_object_target(file, slots, object)
            else {
                current_keyed_object = None;
                current_keyed_property = None;
                continue;
            };

            let keyed_objects = Arc::make_mut(&mut animations[animation_index].keyed_objects);
            keyed_objects.push(RuntimeKeyedObject {
                global_id: global_id as u32,
                object_id,
                target_local_id,
                keyed_properties: Vec::new(),
            });
            current_keyed_object = Some(keyed_objects.len() - 1);
            current_keyed_property = None;
            continue;
        }

        if object.type_name == "KeyedProperty" {
            let Some(keyed_object_index) = current_keyed_object else {
                continue;
            };
            let Some(property_key) = object
                .uint_property("propertyKey")
                .and_then(|key| u16::try_from(key).ok())
            else {
                current_keyed_property = None;
                continue;
            };
            let keyed_object = &animations[animation_index].keyed_objects[keyed_object_index];
            let object_id = keyed_object.object_id;
            let target_local_id = keyed_object.target_local_id;
            let Some(target) = slots
                .get(object_id)
                .and_then(|slot| file.object(slot.source_global_id as usize))
            else {
                current_keyed_property = None;
                continue;
            };
            if !object_supports_property(target.type_key, property_key) {
                current_keyed_property = None;
                continue;
            }

            let keyed_objects = Arc::make_mut(&mut animations[animation_index].keyed_objects);
            keyed_objects[keyed_object_index]
                .keyed_properties
                .push(RuntimeKeyedProperty {
                    global_id: global_id as u32,
                    property_key,
                    transform_property: transform_property_for_key(property_key),
                    double_property: core_registry_field_kind_by_property_key(property_key)
                        == Some(CoreRegistryFieldKind::Double),
                    double_source_value: runtime_object_double_property_by_key(
                        target,
                        property_key,
                    )
                    .unwrap_or(0.0),
                    color_property: core_registry_field_kind_by_property_key(property_key)
                        == Some(CoreRegistryFieldKind::Color),
                    solid_color_property: target.type_name == "SolidColor"
                        && solid_color_value_property_key() == Some(property_key),
                    data_bind_observed: false,
                    color_source_value: runtime_object_color_property_by_key(target, property_key)
                        .unwrap_or(0),
                    bool_property: core_registry_field_kind_by_property_key(property_key)
                        == Some(CoreRegistryFieldKind::Bool),
                    bool_source_value: runtime_object_bool_property_by_key(target, property_key)
                        .unwrap_or(false),
                    uint_property: core_registry_field_kind_by_property_key(property_key)
                        == Some(CoreRegistryFieldKind::Uint),
                    string_property: runtime_object_field_kind_by_key(target, property_key)
                        == Some(FieldKind::String),
                    callback_event: callback_event_for_keyed_property(
                        target_local_id,
                        target,
                        property_key,
                    ),
                    key_frames: Vec::new(),
                    color_key_frames: Vec::new(),
                    bool_key_frames: Vec::new(),
                    uint_key_frames: Vec::new(),
                    string_key_frames: Vec::new(),
                    callback_key_frames: Vec::new(),
                });
            current_keyed_property = Some((
                keyed_object_index,
                keyed_objects[keyed_object_index].keyed_properties.len() - 1,
            ));
            continue;
        }

        if object.type_name == "KeyFrameDouble" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .key_frames
            .push(RuntimeKeyFrameDouble {
                global_id: global_id as u32,
                frame: object.uint_property("frame").unwrap_or(0),
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                interpolator: runtime_key_frame_interpolator(file, artboard_index, object),
                value: object.double_property("value").unwrap_or(0.0),
            });
        }

        if object.type_name == "KeyFrameColor" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .color_key_frames
            .push(RuntimeKeyFrameColor {
                global_id: global_id as u32,
                frame: object.uint_property("frame").unwrap_or(0),
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                interpolator: runtime_key_frame_interpolator(file, artboard_index, object),
                value: object.color_property("value").unwrap_or(0),
            });
        }

        if object.type_name == "KeyFrameBool" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .bool_key_frames
            .push(RuntimeKeyFrameBool {
                global_id: global_id as u32,
                frame: object.uint_property("frame").unwrap_or(0),
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                value: object.bool_property("value").unwrap_or(false),
            });
        }

        if object.type_name == "KeyFrameUint" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .uint_key_frames
            .push(RuntimeKeyFrameUint {
                global_id: global_id as u32,
                frame: object.uint_property("frame").unwrap_or(0),
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                value: object.uint_property("value").unwrap_or(0),
            });
        }

        if object.type_name == "KeyFrameId" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .uint_key_frames
            .push(RuntimeKeyFrameUint {
                global_id: global_id as u32,
                frame: object.uint_property("frame").unwrap_or(0),
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                value: object.uint_property("value").unwrap_or(0),
            });
        }

        if object.type_name == "KeyFrameString" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .string_key_frames
            .push(RuntimeKeyFrameString {
                global_id: global_id as u32,
                frame: object.uint_property("frame").unwrap_or(0),
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                value: object
                    .string_property_bytes("value")
                    .unwrap_or_default()
                    .to_vec(),
            });
        }

        if object.type_name == "KeyFrameCallback" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            animations[animation_index].has_keyed_callbacks = true;
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .callback_key_frames
            .push(RuntimeKeyFrameCallback {
                global_id: global_id as u32,
                frame: object.uint_property("frame").unwrap_or(0),
            });
        }
    }

    let templates = build_key_frame_data_bind_templates(file, artboard_index);
    if !templates.is_empty() {
        for animation in &mut animations {
            let key_frame_ids = animation
                .keyed_objects
                .iter()
                .flat_map(|object| &object.keyed_properties)
                .flat_map(|property| {
                    property
                        .key_frames
                        .iter()
                        .map(|frame| frame.global_id)
                        .chain(
                            property
                                .color_key_frames
                                .iter()
                                .map(|frame| frame.global_id),
                        )
                        .chain(property.bool_key_frames.iter().map(|frame| frame.global_id))
                        .chain(
                            property
                                .string_key_frames
                                .iter()
                                .map(|frame| frame.global_id),
                        )
                })
                .collect::<std::collections::HashSet<_>>();
            animation.key_frame_data_bind_templates = Arc::new(
                templates
                    .iter()
                    .filter(|template| key_frame_ids.contains(&template.key_frame_global_id))
                    .cloned()
                    .collect(),
            );
        }
    }

    animations
}

fn runtime_keyed_property_mut(
    animations: &mut [RuntimeLinearAnimation],
    animation_index: usize,
    keyed_object_index: usize,
    keyed_property_index: usize,
) -> &mut RuntimeKeyedProperty {
    &mut Arc::make_mut(&mut animations[animation_index].keyed_objects)[keyed_object_index]
        .keyed_properties[keyed_property_index]
}

fn artboard_object_range(file: &RuntimeFile, start: usize) -> Option<(usize, usize)> {
    let artboard = file.object(start)?;
    if artboard.type_name != "Artboard" {
        return None;
    }
    let end = ((start + 1)..file.objects.len())
        .find(|index| {
            file.object(*index)
                .is_some_and(|object| object.type_name == "Artboard")
        })
        .unwrap_or(file.objects.len());
    Some((start, end))
}

fn keyed_object_target<'a>(
    file: &'a RuntimeFile,
    slots: &[InstanceSlot],
    keyed_object: &RuntimeObject,
) -> Option<(usize, usize, &'a RuntimeObject)> {
    let object_id = usize::try_from(keyed_object.uint_property("objectId")?).ok()?;
    let slot = slots.get(object_id)?;
    let target = file.object(slot.source_global_id as usize)?;
    Some((object_id, slot.local_id, target))
}

fn normalized_interpolator_id(object: &RuntimeObject) -> Option<u64> {
    object
        .uint_property("interpolatorId")
        .filter(|id| *id != u64::from(u32::MAX) && *id != u64::MAX)
}

fn runtime_key_frame_interpolator(
    file: &RuntimeFile,
    artboard_index: usize,
    key_frame: &RuntimeObject,
) -> Option<RuntimeInterpolator> {
    let local_index = usize::try_from(normalized_interpolator_id(key_frame)?).ok()?;
    let interpolator = file.artboard_local_object(artboard_index, local_index)?;
    RuntimeInterpolator::from_object(interpolator)
}

// Mirrors src/animation/linear_animation.cpp plus keyed object/property keyframe sampling.
#[derive(Debug, Clone)]
pub struct RuntimeLinearAnimation {
    pub global_id: u32,
    pub name: Option<Arc<str>>,
    pub fps: u64,
    pub duration: u64,
    pub speed: f32,
    pub loop_value: u64,
    pub work_start: u64,
    pub work_end: u64,
    pub enable_work_area: bool,
    pub quantize: bool,
    pub keyed_objects: Arc<Vec<RuntimeKeyedObject>>,
    pub(crate) key_frame_data_bind_templates: Arc<Vec<RuntimeKeyFrameDataBindTemplate>>,
    /// Authored callback frames are immutable after import. Retain their
    /// presence so ordinary animations do not enter Rust's deferred callback
    /// collection path on every advance.
    pub(crate) has_keyed_callbacks: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RuntimeKeyFrameValue {
    Number(f32),
    Color(u32),
    Boolean(bool),
    String(Vec<u8>),
}

#[derive(Debug, Clone, Copy, Default)]
struct RuntimeKeyFrameValueContext<'a> {
    holders: Option<&'a HashMap<u32, RuntimeKeyFrameValue>>,
}

impl<'a> RuntimeKeyFrameValueContext<'a> {
    fn number(self, key_frame_global_id: u32) -> Option<f32> {
        match self.holders?.get(&key_frame_global_id)? {
            RuntimeKeyFrameValue::Number(value) => Some(*value),
            _ => None,
        }
    }

    fn color(self, key_frame_global_id: u32) -> Option<u32> {
        match self.holders?.get(&key_frame_global_id)? {
            RuntimeKeyFrameValue::Color(value) => Some(*value),
            _ => None,
        }
    }

    fn boolean(self, key_frame_global_id: u32) -> Option<bool> {
        match self.holders?.get(&key_frame_global_id)? {
            RuntimeKeyFrameValue::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    fn string(self, key_frame_global_id: u32) -> Option<&'a [u8]> {
        match self.holders?.get(&key_frame_global_id)? {
            RuntimeKeyFrameValue::String(value) => Some(value),
            _ => None,
        }
    }
}

impl RuntimeLinearAnimation {
    pub(crate) fn apply(&self, instance: &mut ArtboardInstance, seconds: f32, mix: f32) -> bool {
        self.apply_with_key_frame_values(
            instance,
            seconds,
            mix,
            RuntimeKeyFrameValueContext::default(),
        )
    }

    fn apply_with_key_frame_values(
        &self,
        instance: &mut ArtboardInstance,
        seconds: f32,
        mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> bool {
        let seconds = if self.quantize && self.fps != 0 {
            let fps = self.fps as f32;
            (seconds * fps).floor() / fps
        } else {
            seconds
        };

        let mut changed = false;
        for keyed_object in self.keyed_objects.iter() {
            for keyed_property in &keyed_object.keyed_properties {
                // CoreRegistry assigns exactly one field type per property,
                // matching C++ KeyedProperty's single virtual apply dispatch.
                if let Some(property) = keyed_property.transform_property {
                    let Some(frame_value) =
                        keyed_property.double_frame_value_at(seconds, self.fps, key_frame_values)
                    else {
                        continue;
                    };
                    let Some(value) = apply_key_frame_double_mix(frame_value, mix, || {
                        instance.transform_property_with_key(
                            keyed_object.target_local_id,
                            property,
                            keyed_property.property_key,
                        )
                    }) else {
                        continue;
                    };
                    changed |= instance.set_transform_property_with_key(
                        keyed_object.target_local_id,
                        property,
                        keyed_property.property_key,
                        value,
                    );
                } else if keyed_property.double_property {
                    let Some(frame_value) =
                        keyed_property.double_frame_value_at(seconds, self.fps, key_frame_values)
                    else {
                        continue;
                    };
                    let Some(value) = apply_key_frame_double_mix(frame_value, mix, || {
                        Some(
                            instance
                                .double_property(
                                    keyed_object.target_local_id,
                                    keyed_property.property_key,
                                )
                                .unwrap_or(keyed_property.double_source_value),
                        )
                    }) else {
                        continue;
                    };
                    changed |= instance.set_keyed_double_property(
                        keyed_object.target_local_id,
                        keyed_property.property_key,
                        value,
                    );
                } else if keyed_property.color_property {
                    let Some(frame_value) =
                        keyed_property.color_frame_value_at(seconds, self.fps, key_frame_values)
                    else {
                        continue;
                    };
                    let Some(value) = apply_key_frame_color_mix(frame_value, mix, || {
                        Some(
                            if keyed_property.solid_color_property {
                                instance.solid_color_value(keyed_object.target_local_id)
                            } else {
                                instance.color_property(
                                    keyed_object.target_local_id,
                                    keyed_property.property_key,
                                )
                            }
                            .unwrap_or(keyed_property.color_source_value),
                        )
                    }) else {
                        continue;
                    };
                    changed |= if keyed_property.solid_color_property {
                        instance.set_keyed_solid_color_property(
                            keyed_object.target_local_id,
                            keyed_property.property_key,
                            keyed_property.data_bind_observed,
                            value,
                        )
                    } else {
                        instance.set_keyed_color_property(
                            keyed_object.target_local_id,
                            keyed_property.property_key,
                            value,
                        )
                    };
                } else if keyed_property.bool_property {
                    let Some(value) =
                        keyed_property.bool_value_at(seconds, self.fps, key_frame_values)
                    else {
                        continue;
                    };
                    changed |= instance.set_bool_property(
                        keyed_object.target_local_id,
                        keyed_property.property_key,
                        value,
                    );
                } else if keyed_property.uint_property {
                    let Some(value) = keyed_property.uint_value_at(seconds, self.fps) else {
                        continue;
                    };
                    changed |= instance.set_uint_property(
                        keyed_object.target_local_id,
                        keyed_property.property_key,
                        value,
                    );
                } else if keyed_property.string_property {
                    let Some(value) =
                        keyed_property.string_value_at(seconds, self.fps, key_frame_values)
                    else {
                        continue;
                    };
                    changed |= instance.set_string_property(
                        keyed_object.target_local_id,
                        keyed_property.property_key,
                        value,
                    );
                }
            }
        }
        changed
    }

    fn report_keyed_callbacks(
        &self,
        seconds_from: f32,
        seconds_to: f32,
        speed_direction: f32,
        from_pong: bool,
        reported_events: &mut Vec<StateMachineReportedEvent>,
        keyed_callbacks: &mut Vec<RuntimeKeyedCallback>,
    ) {
        let starting_time = self.start_time_with_speed(speed_direction);
        let is_at_start_frame = starting_time == seconds_from;

        if is_at_start_frame && from_pong {
            return;
        }

        for keyed_object in self.keyed_objects.iter() {
            for keyed_property in &keyed_object.keyed_properties {
                keyed_property.report_keyed_callbacks(
                    keyed_object.target_local_id,
                    seconds_from,
                    seconds_to,
                    self.fps,
                    is_at_start_frame,
                    reported_events,
                    keyed_callbacks,
                );
            }
        }
    }

    pub(crate) fn start_seconds(&self) -> f32 {
        self.frame_to_seconds(self.start_frame())
    }

    fn end_seconds(&self) -> f32 {
        self.frame_to_seconds(self.end_frame())
    }

    pub(crate) fn duration_seconds(&self) -> f32 {
        (self.end_seconds() - self.start_seconds()).abs()
    }

    pub(crate) fn global_to_local_seconds(&self, seconds: f32) -> f32 {
        match AnimationLoop::from_loop_value(self.loop_value) {
            AnimationLoop::OneShot => seconds + self.start_seconds(),
            AnimationLoop::Loop => {
                positive_mod(seconds, self.duration_seconds()) + self.start_seconds()
            }
            AnimationLoop::PingPong => {
                let duration = self.duration_seconds();
                // Guard zero-length animations. C++ (linear_animation.cpp:142-144)
                // computes `(int)(seconds / durationSeconds())`, which is UB when
                // durationSeconds()==0 (division by zero -> inf/NaN cast to int);
                // Rust instead saturates the cast (inf->i32::MAX, NaN->0), a
                // different-but-defined result. Neither is meaningful. Since
                // duration_seconds() == (end - start).abs(), a zero duration means
                // start_seconds() == end_seconds(), so both ping-pong branches
                // collapse to that single frame. Return it directly -- a
                // documented deterministic choice (the C++ value is UB and not
                // reliably reproducible).
                if duration == 0.0 {
                    return self.start_seconds();
                }
                let local_time = positive_mod(seconds, duration);
                let direction = (seconds / duration) as i32 % 2;
                if direction == 0 {
                    local_time + self.start_seconds()
                } else {
                    self.end_seconds() - local_time
                }
            }
        }
    }

    fn start_time_with_speed(&self, speed_multiplier: f32) -> f32 {
        if self.speed * speed_multiplier >= 0.0 {
            self.start_seconds()
        } else {
            self.end_seconds()
        }
    }

    fn fps_as_f32(&self) -> f32 {
        self.fps as f32
    }

    fn start_frame(&self) -> f32 {
        if self.enable_work_area {
            self.work_start as f32
        } else {
            0.0
        }
    }

    fn end_frame(&self) -> f32 {
        if self.enable_work_area {
            self.work_end as f32
        } else {
            self.duration as f32
        }
    }

    fn frame_to_seconds(&self, frame: f32) -> f32 {
        if self.fps == 0 {
            return 0.0;
        }
        frame / self.fps_as_f32()
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyedObject {
    pub global_id: u32,
    pub object_id: usize,
    pub target_local_id: usize,
    pub keyed_properties: Vec<RuntimeKeyedProperty>,
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyedProperty {
    pub global_id: u32,
    pub property_key: u16,
    pub transform_property: Option<TransformProperty>,
    pub double_property: bool,
    pub double_source_value: f32,
    pub color_property: bool,
    /// The import-time equivalent of C++'s concrete `SolidColor*` target.
    /// Avoids repeating CoreRegistry/type discovery for every sampled frame.
    pub(crate) solid_color_property: bool,
    /// C++ keeps an intrusive observer head on each concrete Core object.
    /// Rust resolves the equivalent subscription once at artboard build time.
    pub(crate) data_bind_observed: bool,
    pub color_source_value: u32,
    pub bool_property: bool,
    pub bool_source_value: bool,
    pub uint_property: bool,
    pub string_property: bool,
    pub(crate) callback_event: Option<StateMachineReportedEvent>,
    pub key_frames: Vec<RuntimeKeyFrameDouble>,
    pub color_key_frames: Vec<RuntimeKeyFrameColor>,
    pub bool_key_frames: Vec<RuntimeKeyFrameBool>,
    pub uint_key_frames: Vec<RuntimeKeyFrameUint>,
    pub string_key_frames: Vec<RuntimeKeyFrameString>,
    pub(crate) callback_key_frames: Vec<RuntimeKeyFrameCallback>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeKeyedCallback {
    pub(crate) target_local_id: usize,
    pub(crate) property_key: u16,
    pub(crate) seconds_delay: f32,
}

// Mirrors KeyFrameDouble::applyDouble and KeyFrameColor::applyColor. Keep the
// current-value read lazy: C++ writes the sampled keyframe target directly at
// a full mix, and only reads the property when a partial blend is required.
fn apply_key_frame_double_mix(
    value: f32,
    mix: f32,
    current: impl FnOnce() -> Option<f32>,
) -> Option<f32> {
    if mix == 1.0 {
        Some(value)
    } else {
        current().map(|current| mix_value(current, value, mix))
    }
}

fn apply_key_frame_color_mix(
    value: u32,
    mix: f32,
    current: impl FnOnce() -> Option<u32>,
) -> Option<u32> {
    if mix == 1.0 {
        Some(value)
    } else {
        current().map(|current| color_lerp(current, value, mix))
    }
}

impl RuntimeKeyedProperty {
    fn double_frame_value_at(
        &self,
        seconds: f32,
        fps: u64,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<f32> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds, fps);
        let value = if idx == 0 {
            self.key_frames[0].effective_value(key_frame_values)
        } else if idx < self.key_frames.len() {
            let from = &self.key_frames[idx - 1];
            let to = &self.key_frames[idx];
            if seconds == to.seconds(fps) {
                to.effective_value(key_frame_values)
            } else if from.interpolation_type == 0 {
                from.effective_value(key_frame_values)
            } else if from.interpolator_id.is_some() {
                let frame_mix = frame_mix(seconds, from.seconds(fps), to.seconds(fps));
                from.interpolator?.transform_value(
                    from.effective_value(key_frame_values),
                    to.effective_value(key_frame_values),
                    frame_mix,
                )
            } else {
                let frame_mix = frame_mix(seconds, from.seconds(fps), to.seconds(fps));
                let from_value = from.effective_value(key_frame_values);
                let to_value = to.effective_value(key_frame_values);
                from_value + (to_value - from_value) * frame_mix
            }
        } else {
            self.key_frames.last()?.effective_value(key_frame_values)
        };

        Some(value)
    }

    fn color_frame_value_at(
        &self,
        seconds: f32,
        fps: u64,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<u32> {
        if self.color_key_frames.is_empty() {
            return None;
        }

        let idx = closest_key_frame_index(&self.color_key_frames, seconds, fps);
        let value = if idx == 0 {
            self.color_key_frames[0].effective_value(key_frame_values)
        } else if idx < self.color_key_frames.len() {
            let from = &self.color_key_frames[idx - 1];
            let to = &self.color_key_frames[idx];
            if seconds == to.seconds(fps) {
                to.effective_value(key_frame_values)
            } else if from.interpolation_type == 0 {
                from.effective_value(key_frame_values)
            } else if from.interpolator_id.is_some() {
                let frame_mix = frame_mix(seconds, from.seconds(fps), to.seconds(fps));
                color_lerp(
                    from.effective_value(key_frame_values),
                    to.effective_value(key_frame_values),
                    from.interpolator?.transform(frame_mix),
                )
            } else {
                let frame_mix = frame_mix(seconds, from.seconds(fps), to.seconds(fps));
                color_lerp(
                    from.effective_value(key_frame_values),
                    to.effective_value(key_frame_values),
                    frame_mix,
                )
            }
        } else {
            self.color_key_frames
                .last()?
                .effective_value(key_frame_values)
        };

        Some(value)
    }

    fn bool_value_at(
        &self,
        seconds: f32,
        fps: u64,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<bool> {
        if self.bool_key_frames.is_empty() {
            return None;
        }

        let idx = closest_key_frame_index(&self.bool_key_frames, seconds, fps);
        let value = if idx == 0 {
            self.bool_key_frames[0].effective_value(key_frame_values)
        } else if idx < self.bool_key_frames.len() {
            let from = &self.bool_key_frames[idx - 1];
            let to = &self.bool_key_frames[idx];
            if seconds == to.seconds(fps) {
                to.effective_value(key_frame_values)
            } else {
                from.effective_value(key_frame_values)
            }
        } else {
            self.bool_key_frames
                .last()?
                .effective_value(key_frame_values)
        };

        Some(value)
    }

    fn uint_value_at(&self, seconds: f32, fps: u64) -> Option<u64> {
        if self.uint_key_frames.is_empty() {
            return None;
        }

        let idx = closest_key_frame_index(&self.uint_key_frames, seconds, fps);
        let value = if idx == 0 {
            self.uint_key_frames[0].value
        } else if idx < self.uint_key_frames.len() {
            let from = &self.uint_key_frames[idx - 1];
            let to = &self.uint_key_frames[idx];
            if seconds == to.seconds(fps) {
                to.value
            } else {
                from.value
            }
        } else {
            self.uint_key_frames.last()?.value
        };

        Some(value)
    }

    fn string_value_at(
        &self,
        seconds: f32,
        fps: u64,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<Vec<u8>> {
        if self.string_key_frames.is_empty() {
            return None;
        }

        let idx = closest_key_frame_index(&self.string_key_frames, seconds, fps);
        let key_frame = if idx == 0 {
            &self.string_key_frames[0]
        } else if idx < self.string_key_frames.len() {
            let from = &self.string_key_frames[idx - 1];
            let to = &self.string_key_frames[idx];
            if seconds == to.seconds(fps) { to } else { from }
        } else {
            self.string_key_frames.last()?
        };

        Some(key_frame.effective_value(key_frame_values))
    }

    fn report_keyed_callbacks(
        &self,
        target_local_id: usize,
        seconds_from: f32,
        seconds_to: f32,
        fps: u64,
        is_at_start_frame: bool,
        reported_events: &mut Vec<StateMachineReportedEvent>,
        keyed_callbacks: &mut Vec<RuntimeKeyedCallback>,
    ) {
        if self.callback_key_frames.is_empty() || seconds_from == seconds_to {
            return;
        }

        let is_forward = seconds_from <= seconds_to;
        let mut from_exact_offset = 0;
        let to_exact_offset = usize::from(is_forward);
        if is_forward {
            if !is_at_start_frame {
                from_exact_offset = 1;
            }
        } else if is_at_start_frame {
            from_exact_offset = 1;
        }

        let mut index = closest_key_frame_index_with_exact_offset(
            &self.callback_key_frames,
            seconds_from,
            fps,
            from_exact_offset,
        );
        let mut index_to = closest_key_frame_index_with_exact_offset(
            &self.callback_key_frames,
            seconds_to,
            fps,
            to_exact_offset,
        );
        if index_to < index {
            std::mem::swap(&mut index, &mut index_to);
        }

        while index_to > index {
            let key_frame = &self.callback_key_frames[index];
            let seconds_delay = seconds_to - key_frame.seconds(fps);
            keyed_callbacks.push(RuntimeKeyedCallback {
                target_local_id,
                property_key: self.property_key,
                seconds_delay,
            });
            if let Some(event) = self.callback_event.as_ref() {
                let mut reported_event = event.clone();
                reported_event.seconds_delay = seconds_delay;
                reported_events.push(reported_event);
            }
            index += 1;
        }
    }

    fn closest_frame_index(&self, seconds: f32, fps: u64) -> usize {
        closest_key_frame_index(&self.key_frames, seconds, fps)
    }
}

trait RuntimeKeyFrameTiming {
    fn seconds(&self, fps: u64) -> f32;
}

fn closest_key_frame_index<T: RuntimeKeyFrameTiming>(
    key_frames: &[T],
    seconds: f32,
    fps: u64,
) -> usize {
    closest_key_frame_index_with_exact_offset(key_frames, seconds, fps, 0)
}

fn closest_key_frame_index_with_exact_offset<T: RuntimeKeyFrameTiming>(
    key_frames: &[T],
    seconds: f32,
    fps: u64,
    exact_offset: usize,
) -> usize {
    let last = key_frames.len() - 1;
    if seconds > key_frames[last].seconds(fps) {
        return key_frames.len();
    }

    let mut start = 0;
    let mut end = last;
    while start <= end {
        let mid = (start + end) >> 1;
        let closest = key_frames[mid].seconds(fps);
        if closest < seconds {
            start = mid + 1;
        } else if closest > seconds {
            if mid == 0 {
                break;
            }
            end = mid - 1;
        } else {
            return mid + exact_offset;
        }
    }
    start
}

fn frame_mix(seconds: f32, from_seconds: f32, to_seconds: f32) -> f32 {
    if to_seconds == from_seconds {
        1.0
    } else {
        (seconds - from_seconds) / (to_seconds - from_seconds)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameDouble {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub(crate) interpolator: Option<RuntimeInterpolator>,
    pub value: f32,
}

impl RuntimeKeyFrameDouble {
    fn effective_value(&self, key_frame_values: RuntimeKeyFrameValueContext<'_>) -> f32 {
        key_frame_values
            .number(self.global_id)
            .unwrap_or(self.value)
    }

    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameDouble {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameColor {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub(crate) interpolator: Option<RuntimeInterpolator>,
    pub value: u32,
}

impl RuntimeKeyFrameColor {
    fn effective_value(&self, key_frame_values: RuntimeKeyFrameValueContext<'_>) -> u32 {
        key_frame_values.color(self.global_id).unwrap_or(self.value)
    }

    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameColor {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameBool {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: bool,
}

impl RuntimeKeyFrameBool {
    fn effective_value(&self, key_frame_values: RuntimeKeyFrameValueContext<'_>) -> bool {
        key_frame_values
            .boolean(self.global_id)
            .unwrap_or(self.value)
    }

    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameBool {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameUint {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: u64,
}

impl RuntimeKeyFrameUint {
    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameUint {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameString {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: Vec<u8>,
}

impl RuntimeKeyFrameString {
    fn effective_value(&self, key_frame_values: RuntimeKeyFrameValueContext<'_>) -> Vec<u8> {
        key_frame_values
            .string(self.global_id)
            .unwrap_or(&self.value)
            .to_vec()
    }

    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameString {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameCallback {
    pub global_id: u32,
    pub frame: u64,
}

impl RuntimeKeyFrameCallback {
    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameCallback {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

// Mirrors src/animation/linear_animation_instance.cpp and include/rive/animation/loop.hpp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnimationLoop {
    OneShot,
    Loop,
    PingPong,
}

impl AnimationLoop {
    pub(crate) fn from_loop_value(value: u64) -> Self {
        match value {
            1 => Self::Loop,
            2 => Self::PingPong,
            _ => Self::OneShot,
        }
    }
}

fn positive_mod(value: f32, range: f32) -> f32 {
    if range == 0.0 {
        return 0.0;
    }
    ((value % range) + range) % range
}

#[derive(Debug)]
pub struct LinearAnimationInstance {
    pub(crate) animation_index: usize,
    /// C++ retains a `const LinearAnimation*` on each instance. Keep the
    /// immutable Rust animation descriptor alongside the index too, avoiding
    /// a vector lookup and `Arc`-backed descriptor clone on every apply.
    animation: RuntimeLinearAnimation,
    pub(crate) time: f32,
    pub(crate) speed_direction: f32,
    pub(crate) total_time: f32,
    pub(crate) last_total_time: f32,
    pub(crate) spilled_time: f32,
    pub(crate) direction: f32,
    pub(crate) did_loop: bool,
    pub(crate) loop_value: Option<u64>,
    key_frame_value_holders: Option<Box<HashMap<u32, RuntimeKeyFrameValue>>>,
    key_frame_data_bind_graph: Option<Box<RuntimeDataBindGraph>>,
    key_frame_prototype_revision: u64,
}

impl Clone for LinearAnimationInstance {
    fn clone(&self) -> Self {
        Self {
            animation_index: self.animation_index,
            animation: self.animation.clone(),
            time: self.time,
            speed_direction: self.speed_direction,
            total_time: self.total_time,
            last_total_time: self.last_total_time,
            spilled_time: self.spilled_time,
            direction: self.direction,
            did_loop: self.did_loop,
            loop_value: self.loop_value,
            // Keyframe holders model C++'s per-LAI runtime-owned bind targets.
            // A copied LAI starts unbound; state transitions move the outgoing
            // instance when they need to preserve its concrete binding identity.
            key_frame_value_holders: None,
            key_frame_data_bind_graph: None,
            key_frame_prototype_revision: 0,
        }
    }
}

impl LinearAnimationInstance {
    pub(crate) fn new(
        animation_index: usize,
        animation: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> Self {
        Self {
            animation_index,
            animation: animation.clone(),
            time: animation.start_time_with_speed(speed_multiplier),
            speed_direction: if speed_multiplier >= 0.0 { 1.0 } else { -1.0 },
            total_time: 0.0,
            last_total_time: 0.0,
            spilled_time: 0.0,
            direction: 1.0,
            did_loop: false,
            loop_value: None,
            key_frame_value_holders: None,
            key_frame_data_bind_graph: None,
            key_frame_prototype_revision: 0,
        }
    }

    fn initialize_key_frame_data_bind_graph(&mut self, prototype: &RuntimeDataBindGraph) {
        if self.key_frame_data_bind_graph.is_some() {
            return;
        }
        for target in &prototype.targets {
            let (global_id, value) = match target.target {
                RuntimeDataBindGraphTarget::KeyFrameNumber { global_id } => {
                    (global_id, RuntimeKeyFrameValue::Number(0.0))
                }
                RuntimeDataBindGraphTarget::KeyFrameColor { global_id } => {
                    (global_id, RuntimeKeyFrameValue::Color(0xFF1D1D1D))
                }
                RuntimeDataBindGraphTarget::KeyFrameBoolean { global_id } => {
                    (global_id, RuntimeKeyFrameValue::Boolean(false))
                }
                RuntimeDataBindGraphTarget::KeyFrameString { global_id } => {
                    (global_id, RuntimeKeyFrameValue::String(Vec::new()))
                }
                _ => continue,
            };
            self.add_key_frame_value_holder(global_id, value);
        }
        self.key_frame_data_bind_graph = Some(Box::new(prototype.clone_for_key_frame_instance()));
        self.key_frame_prototype_revision = prototype.key_frame_source_revision();
    }

    fn sync_key_frame_data_bind_graph(&mut self, prototype: &RuntimeDataBindGraph) {
        self.initialize_key_frame_data_bind_graph(prototype);
        if self.key_frame_prototype_revision == prototype.key_frame_source_revision() {
            return;
        }
        if let Some(graph) = self.key_frame_data_bind_graph.as_deref_mut() {
            graph.sync_key_frame_sources_from(prototype);
        }
        self.key_frame_prototype_revision = prototype.key_frame_source_revision();
    }

    fn apply_key_frame_data_bind_updates(
        &mut self,
        updates: Vec<(RuntimeDataBindGraphTarget, crate::RuntimeDataBindGraphValue)>,
    ) -> bool {
        let mut changed = false;
        for (target, value) in updates {
            let (global_id, value) = match (target, value) {
                (
                    RuntimeDataBindGraphTarget::KeyFrameNumber { global_id },
                    crate::RuntimeDataBindGraphValue::Number(value),
                ) => (global_id, RuntimeKeyFrameValue::Number(value)),
                (
                    RuntimeDataBindGraphTarget::KeyFrameColor { global_id },
                    crate::RuntimeDataBindGraphValue::Color(value),
                ) => (global_id, RuntimeKeyFrameValue::Color(value)),
                (
                    RuntimeDataBindGraphTarget::KeyFrameBoolean { global_id },
                    crate::RuntimeDataBindGraphValue::Boolean(value),
                ) => (global_id, RuntimeKeyFrameValue::Boolean(value)),
                (
                    RuntimeDataBindGraphTarget::KeyFrameString { global_id },
                    crate::RuntimeDataBindGraphValue::String(value),
                ) => (global_id, RuntimeKeyFrameValue::String(value)),
                _ => continue,
            };
            let Some(holder) = self.key_frame_value_holder_mut(global_id) else {
                continue;
            };
            if *holder != value {
                *holder = value;
                changed = true;
            }
        }
        changed
    }

    pub(crate) fn prepare_key_frame_data_binds(
        &mut self,
        prototype: Option<&RuntimeDataBindGraph>,
    ) -> bool {
        let Some(prototype) = prototype else {
            return false;
        };
        self.sync_key_frame_data_bind_graph(prototype);
        let updates = self
            .key_frame_data_bind_graph
            .as_deref_mut()
            .map(|graph| {
                graph.take_key_frame_binding_updates(
                    RuntimeDataBindGraphApplyPhase::BeforeStatefulAdvance,
                )
            })
            .unwrap_or_default();
        self.apply_key_frame_data_bind_updates(updates)
    }

    pub(crate) fn advance_key_frame_data_binds(
        &mut self,
        prototype: Option<&RuntimeDataBindGraph>,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(prototype) = prototype else {
            return false;
        };
        let mut keep_going = false;
        let mut changed = self.prepare_key_frame_data_binds(Some(prototype));
        if let Some(graph) = self.key_frame_data_bind_graph.as_deref_mut() {
            let advance = graph.advance_stateful_converters(elapsed_seconds);
            changed |= advance.changed;
            keep_going |= advance.keep_going;
        }
        let updates = self
            .key_frame_data_bind_graph
            .as_deref_mut()
            .map(|graph| {
                graph.take_key_frame_binding_updates(
                    RuntimeDataBindGraphApplyPhase::AfterStatefulAdvance,
                )
            })
            .unwrap_or_default();
        changed |= self.apply_key_frame_data_bind_updates(updates);
        changed || keep_going
    }

    pub(crate) fn add_key_frame_value_holder(
        &mut self,
        key_frame_global_id: u32,
        value: RuntimeKeyFrameValue,
    ) {
        self.key_frame_value_holders
            .get_or_insert_with(|| Box::new(HashMap::new()))
            .insert(key_frame_global_id, value);
    }

    pub(crate) fn key_frame_value_holder(
        &self,
        key_frame_global_id: u32,
    ) -> Option<&RuntimeKeyFrameValue> {
        self.key_frame_value_holders
            .as_deref()?
            .get(&key_frame_global_id)
    }

    pub(crate) fn key_frame_value_holder_mut(
        &mut self,
        key_frame_global_id: u32,
    ) -> Option<&mut RuntimeKeyFrameValue> {
        self.key_frame_value_holders
            .as_deref_mut()?
            .get_mut(&key_frame_global_id)
    }

    fn key_frame_value_context(&self) -> RuntimeKeyFrameValueContext<'_> {
        RuntimeKeyFrameValueContext {
            holders: self.key_frame_value_holders.as_deref(),
        }
    }

    pub(crate) fn apply(&self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        self.animation.apply_with_key_frame_values(
            artboard,
            self.time,
            mix,
            self.key_frame_value_context(),
        )
    }

    pub fn animation_index(&self) -> usize {
        self.animation_index
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn speed_direction(&self) -> f32 {
        self.speed_direction
    }

    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    pub fn last_total_time(&self) -> f32 {
        self.last_total_time
    }

    pub fn spilled_time(&self) -> f32 {
        self.spilled_time
    }

    pub fn direction(&self) -> f32 {
        self.direction
    }

    pub fn did_loop(&self) -> bool {
        self.did_loop
    }

    pub fn loop_value(&self) -> Option<u64> {
        self.loop_value
    }

    pub(crate) fn set_time(&mut self, animation: &RuntimeLinearAnimation, value: f32) {
        if self.time == value {
            return;
        }
        self.time = value;
        let diff = self.total_time - self.last_total_time;
        self.total_time = value - animation.start_seconds();
        self.last_total_time = self.total_time - diff;
        self.direction = 1.0;
    }

    pub fn directed_speed(&self, animation: &RuntimeLinearAnimation) -> f32 {
        self.direction * animation.speed
    }

    pub(crate) fn resolved_loop_kind(&self, animation: &RuntimeLinearAnimation) -> AnimationLoop {
        AnimationLoop::from_loop_value(self.loop_value.unwrap_or(animation.loop_value))
    }

    pub(crate) fn keep_going(&self, animation: &RuntimeLinearAnimation) -> bool {
        self.resolved_loop_kind(animation) != AnimationLoop::OneShot
            || (self.directed_speed(animation) > 0.0 && self.time < animation.end_seconds())
            || (self.directed_speed(animation) < 0.0 && self.time > animation.start_seconds())
    }

    pub(crate) fn keep_going_with_speed_multiplier(
        &self,
        animation: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> bool {
        self.resolved_loop_kind(animation) != AnimationLoop::OneShot
            || (self.directed_speed(animation) * speed_multiplier > 0.0
                && self.time < animation.end_seconds())
            || (self.directed_speed(animation) * speed_multiplier < 0.0
                && self.time > animation.start_seconds())
    }

    pub(crate) fn advance(
        &mut self,
        animation: &RuntimeLinearAnimation,
        elapsed_seconds: f32,
    ) -> bool {
        self.advance_and_report(animation, elapsed_seconds, None, None)
    }

    pub(crate) fn advance_with_events(
        &mut self,
        animation: &RuntimeLinearAnimation,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
        keyed_callbacks: &mut Vec<RuntimeKeyedCallback>,
    ) -> bool {
        self.advance_and_report(
            animation,
            elapsed_seconds,
            Some(reported_events),
            Some(keyed_callbacks),
        )
    }

    fn advance_and_report(
        &mut self,
        animation: &RuntimeLinearAnimation,
        elapsed_seconds: f32,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
        mut keyed_callbacks: Option<&mut Vec<RuntimeKeyedCallback>>,
    ) -> bool {
        let delta_seconds = elapsed_seconds * animation.speed * self.direction;
        self.spilled_time = 0.0;
        if delta_seconds == 0.0 {
            self.did_loop = false;
            return false;
        }

        self.last_total_time = self.total_time;
        self.total_time += delta_seconds.abs();
        let kill_spilled_time = !self.keep_going_with_speed_multiplier(animation, elapsed_seconds);

        let mut last_time = self.time;
        self.time += delta_seconds;
        if let (Some(events), Some(callbacks)) = (
            reported_events.as_deref_mut(),
            keyed_callbacks.as_deref_mut(),
        ) {
            animation.report_keyed_callbacks(
                last_time,
                self.time,
                self.speed_direction,
                false,
                events,
                callbacks,
            );
        }
        let fps = animation.fps_as_f32();
        if fps == 0.0 {
            self.did_loop = false;
            return self.keep_going_with_speed_multiplier(animation, elapsed_seconds);
        }

        let mut frames = self.time * fps;
        let start = animation.start_frame();
        let end = animation.end_frame();
        let range = end - start;
        let mut did_loop = false;
        let mut direction = if delta_seconds < 0.0 { -1 } else { 1 };

        match self.resolved_loop_kind(animation) {
            AnimationLoop::OneShot => {
                if direction == 1 && frames > end {
                    let delta_frames = delta_seconds * fps;
                    let spilled_frames_ratio = (frames - end) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end;
                    self.time = frames / fps;
                    did_loop = true;
                } else if direction == -1 && frames < start {
                    let delta_frames = (delta_seconds * fps).abs();
                    let spilled_frames_ratio = (start - frames) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start;
                    self.time = frames / fps;
                    did_loop = true;
                }
            }
            AnimationLoop::Loop => {
                if range != 0.0 && direction == 1 && frames >= end {
                    let delta_frames = delta_seconds * fps;
                    let remainder = (frames - start) % range;
                    let spilled_frames_ratio = remainder / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start + remainder;
                    self.time = frames / fps;
                    did_loop = true;
                    if let (Some(events), Some(callbacks)) = (
                        reported_events.as_deref_mut(),
                        keyed_callbacks.as_deref_mut(),
                    ) {
                        animation.report_keyed_callbacks(
                            0.0,
                            self.time,
                            self.speed_direction,
                            false,
                            events,
                            callbacks,
                        );
                    }
                } else if range != 0.0 && direction == -1 && frames <= start {
                    let delta_frames = delta_seconds * fps;
                    let remainder = ((start - frames) % range).abs();
                    let spilled_frames_ratio = (remainder / delta_frames).abs();
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end - remainder;
                    self.time = frames / fps;
                    did_loop = true;
                    if let (Some(events), Some(callbacks)) = (
                        reported_events.as_deref_mut(),
                        keyed_callbacks.as_deref_mut(),
                    ) {
                        animation.report_keyed_callbacks(
                            end / fps,
                            self.time,
                            self.speed_direction,
                            false,
                            events,
                            callbacks,
                        );
                    }
                }
            }
            AnimationLoop::PingPong => {
                let mut from_pong = true;
                loop {
                    if direction == 1 && frames >= end {
                        self.spilled_time = (frames - end) / fps;
                        frames = end + (end - frames);
                        last_time = end / fps;
                    } else if direction == -1 && frames < start {
                        self.spilled_time = (start - frames) / fps;
                        frames = start + (start - frames);
                        last_time = start / fps;
                    } else {
                        break;
                    }
                    self.time = frames / fps;
                    self.direction *= -1.0;
                    direction *= -1;
                    did_loop = true;
                    if let (Some(events), Some(callbacks)) = (
                        reported_events.as_deref_mut(),
                        keyed_callbacks.as_deref_mut(),
                    ) {
                        animation.report_keyed_callbacks(
                            last_time,
                            self.time,
                            self.speed_direction,
                            from_pong,
                            events,
                            callbacks,
                        );
                    }
                    from_pong = !from_pong;
                }
            }
        }

        if kill_spilled_time {
            self.spilled_time = 0.0;
        }
        self.did_loop = did_loop;
        self.keep_going_with_speed_multiplier(animation, elapsed_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_bind_graph::{
        RuntimeDataBindGraphConverter, RuntimeDataBindGraphFormulaToken,
        RuntimeKeyFrameDataBindTarget,
    };

    fn number_key_frame_binding(
        converter: Option<RuntimeDataBindGraphConverter>,
    ) -> RuntimeKeyFrameDataBindTemplate {
        RuntimeKeyFrameDataBindTemplate {
            data_bind_index: 0,
            key_frame_global_id: 10,
            target: RuntimeKeyFrameDataBindTarget::Number,
            path: vec![0, 0],
            flags: 0,
            converter,
            default_value: crate::RuntimeDataBindGraphValue::Number(0.0),
        }
    }

    fn animation_with_work_area(enable_work_area: bool) -> RuntimeLinearAnimation {
        RuntimeLinearAnimation {
            global_id: 1,
            name: Some(Arc::<str>::from("work area")),
            fps: 60,
            duration: 60,
            speed: 1.0,
            loop_value: 1,
            work_start: 10,
            work_end: 40,
            enable_work_area,
            quantize: false,
            keyed_objects: Arc::new(Vec::new()),
            key_frame_data_bind_templates: Arc::new(Vec::new()),
            has_keyed_callbacks: false,
        }
    }

    fn keyed_double_property(
        from_global_id: u32,
        from_value: f32,
        to_global_id: u32,
        to_value: f32,
    ) -> RuntimeKeyedProperty {
        RuntimeKeyedProperty {
            global_id: 1,
            property_key: 1,
            transform_property: None,
            double_property: true,
            double_source_value: 0.0,
            color_property: false,
            solid_color_property: false,
            data_bind_observed: false,
            color_source_value: 0,
            bool_property: false,
            bool_source_value: false,
            uint_property: false,
            string_property: false,
            callback_event: None,
            key_frames: vec![
                RuntimeKeyFrameDouble {
                    global_id: from_global_id,
                    frame: 0,
                    interpolation_type: 1,
                    interpolator_id: None,
                    interpolator: None,
                    value: from_value,
                },
                RuntimeKeyFrameDouble {
                    global_id: to_global_id,
                    frame: 10,
                    interpolation_type: 1,
                    interpolator_id: None,
                    interpolator: None,
                    value: to_value,
                },
            ],
            color_key_frames: Vec::new(),
            bool_key_frames: Vec::new(),
            uint_key_frames: Vec::new(),
            string_key_frames: Vec::new(),
            callback_key_frames: Vec::new(),
        }
    }

    #[test]
    fn data_bound_double_interpolation_uses_both_effective_endpoints() {
        let animation = animation_with_work_area(false);
        let mut instance = LinearAnimationInstance::new(0, &animation, 1.0);
        instance.add_key_frame_value_holder(10, RuntimeKeyFrameValue::Number(100.0));
        instance.add_key_frame_value_holder(20, RuntimeKeyFrameValue::Number(200.0));
        let property = keyed_double_property(10, 1.0, 20, 2.0);

        assert_eq!(
            property.double_frame_value_at(0.5, 10, instance.key_frame_value_context(),),
            Some(150.0)
        );
    }

    #[test]
    fn data_bound_color_interpolation_uses_both_effective_endpoints() {
        let animation = animation_with_work_area(false);
        let mut instance = LinearAnimationInstance::new(0, &animation, 1.0);
        let bound_from = 0xFF00_0000;
        let bound_to = 0xFFFF_FFFF;
        instance.add_key_frame_value_holder(30, RuntimeKeyFrameValue::Color(bound_from));
        instance.add_key_frame_value_holder(40, RuntimeKeyFrameValue::Color(bound_to));
        let mut property = keyed_double_property(10, 1.0, 20, 2.0);
        property.double_property = false;
        property.key_frames.clear();
        property.color_property = true;
        property.color_key_frames = vec![
            RuntimeKeyFrameColor {
                global_id: 30,
                frame: 0,
                interpolation_type: 1,
                interpolator_id: None,
                interpolator: None,
                value: 0xFF00_FF00,
            },
            RuntimeKeyFrameColor {
                global_id: 40,
                frame: 10,
                interpolation_type: 1,
                interpolator_id: None,
                interpolator: None,
                value: 0xFF00_00FF,
            },
        ];

        assert_eq!(
            property.color_frame_value_at(0.5, 10, instance.key_frame_value_context(),),
            Some(color_lerp(bound_from, bound_to, 0.5))
        );
    }

    #[test]
    fn full_key_frame_mix_skips_current_value_reads_for_double_and_color() {
        let mut read_double = false;
        assert_eq!(
            apply_key_frame_double_mix(42.0, 1.0, || {
                read_double = true;
                Some(-1.0)
            }),
            Some(42.0)
        );
        assert!(!read_double);

        let mut read_color = false;
        assert_eq!(
            apply_key_frame_color_mix(0xA1B2_C3D4, 1.0, || {
                read_color = true;
                Some(0)
            }),
            Some(0xA1B2_C3D4)
        );
        assert!(!read_color);

        assert_eq!(
            apply_key_frame_double_mix(42.0, 0.25, || Some(2.0)),
            Some(12.0)
        );
        assert_eq!(
            apply_key_frame_color_mix(0xFFFF_FFFF, 0.5, || Some(0xFF00_0000)),
            Some(color_lerp(0xFF00_0000, 0xFFFF_FFFF, 0.5))
        );
    }

    #[test]
    fn data_bound_boolean_step_uses_the_effective_current_key_frame() {
        let animation = animation_with_work_area(false);
        let mut instance = LinearAnimationInstance::new(0, &animation, 1.0);
        instance.add_key_frame_value_holder(50, RuntimeKeyFrameValue::Boolean(true));
        let mut property = keyed_double_property(10, 1.0, 20, 2.0);
        property.double_property = false;
        property.key_frames.clear();
        property.bool_property = true;
        property.bool_key_frames = vec![
            RuntimeKeyFrameBool {
                global_id: 50,
                frame: 0,
                interpolation_type: 1,
                interpolator_id: None,
                value: false,
            },
            RuntimeKeyFrameBool {
                global_id: 60,
                frame: 10,
                interpolation_type: 1,
                interpolator_id: None,
                value: false,
            },
        ];

        assert_eq!(
            property.bool_value_at(0.5, 10, instance.key_frame_value_context()),
            Some(true)
        );
    }

    #[test]
    fn data_bound_string_step_uses_the_effective_current_key_frame() {
        let animation = animation_with_work_area(false);
        let mut instance = LinearAnimationInstance::new(0, &animation, 1.0);
        instance
            .add_key_frame_value_holder(70, RuntimeKeyFrameValue::String(b"bound start".to_vec()));
        let mut property = keyed_double_property(10, 1.0, 20, 2.0);
        property.double_property = false;
        property.key_frames.clear();
        property.string_property = true;
        property.string_key_frames = vec![
            RuntimeKeyFrameString {
                global_id: 70,
                frame: 0,
                interpolation_type: 1,
                interpolator_id: None,
                value: b"authored start".to_vec(),
            },
            RuntimeKeyFrameString {
                global_id: 80,
                frame: 10,
                interpolation_type: 1,
                interpolator_id: None,
                value: b"authored end".to_vec(),
            },
        ];

        assert_eq!(
            property.string_value_at(0.5, 10, instance.key_frame_value_context()),
            Some(b"bound start".to_vec())
        );
    }

    #[test]
    fn cloned_animation_instance_starts_without_key_frame_value_holders() {
        let animation = animation_with_work_area(false);
        let mut instance = LinearAnimationInstance::new(0, &animation, 1.0);
        instance.add_key_frame_value_holder(10, RuntimeKeyFrameValue::Number(123.0));

        let cloned = instance.clone();

        assert_eq!(
            instance.key_frame_value_holder(10),
            Some(&RuntimeKeyFrameValue::Number(123.0))
        );
        assert_eq!(cloned.key_frame_value_holder(10), None);
    }

    #[test]
    fn key_frame_value_holders_are_isolated_per_animation_instance() {
        let animation = animation_with_work_area(false);
        let property = keyed_double_property(10, 1.0, 20, 2.0);
        let mut first = LinearAnimationInstance::new(0, &animation, 1.0);
        let mut second = LinearAnimationInstance::new(0, &animation, 1.0);
        let unbound = LinearAnimationInstance::new(0, &animation, 1.0);
        first.add_key_frame_value_holder(10, RuntimeKeyFrameValue::Number(100.0));
        second.add_key_frame_value_holder(10, RuntimeKeyFrameValue::Number(200.0));
        *first.key_frame_value_holder_mut(10).unwrap() = RuntimeKeyFrameValue::Number(150.0);

        assert_eq!(
            property.double_frame_value_at(0.0, 10, first.key_frame_value_context()),
            Some(150.0)
        );
        assert_eq!(
            property.double_frame_value_at(0.0, 10, second.key_frame_value_context()),
            Some(200.0)
        );
        assert_eq!(
            property.double_frame_value_at(0.0, 10, unbound.key_frame_value_context()),
            Some(1.0)
        );
    }

    #[test]
    fn uint_and_id_sampling_ignore_key_frame_value_holders() {
        let animation = animation_with_work_area(false);
        let mut instance = LinearAnimationInstance::new(0, &animation, 1.0);
        instance.add_key_frame_value_holder(90, RuntimeKeyFrameValue::Number(999.0));
        let mut property = keyed_double_property(10, 1.0, 20, 2.0);
        property.double_property = false;
        property.key_frames.clear();
        property.uint_property = true;
        property.uint_key_frames = vec![RuntimeKeyFrameUint {
            global_id: 90,
            frame: 0,
            interpolation_type: 1,
            interpolator_id: None,
            value: 7,
        }];

        // KeyFrameUint and KeyFrameId share this runtime sampler. Upstream
        // intentionally leaves both types unsupported by keyframe value binds.
        assert_eq!(property.uint_value_at(0.0, 10), Some(7));
    }

    #[test]
    fn state_machine_key_frame_graph_updates_one_instance_without_binding_standalone_clones() {
        let animation = animation_with_work_area(false);
        let mut prototype =
            RuntimeDataBindGraph::new_key_frame_bindings(&[number_key_frame_binding(None)])
                .expect("keyframe binding graph");
        assert!(prototype.bind_default_view_model_context());
        assert!(prototype.set_default_view_model_number_source_for_path(&[0, 0], 10.0));

        let mut state_machine_instance = LinearAnimationInstance::new(0, &animation, 1.0);
        assert!(state_machine_instance.prepare_key_frame_data_binds(Some(&prototype)));
        assert_eq!(
            state_machine_instance.key_frame_value_holder(10),
            Some(&RuntimeKeyFrameValue::Number(10.0))
        );

        assert!(prototype.set_default_view_model_number_source_for_path(&[0, 0], 20.0));
        assert!(state_machine_instance.prepare_key_frame_data_binds(Some(&prototype)));
        assert_eq!(
            state_machine_instance.key_frame_value_holder(10),
            Some(&RuntimeKeyFrameValue::Number(20.0))
        );

        let standalone_clone = state_machine_instance.clone();
        assert!(standalone_clone.key_frame_data_bind_graph.is_none());
        assert!(standalone_clone.key_frame_value_holders.is_none());
    }

    #[test]
    fn key_frame_formula_random_state_is_isolated_per_animation_instance() {
        let animation = animation_with_work_area(false);
        let converter = RuntimeDataBindGraphConverter::Formula {
            tokens: vec![RuntimeDataBindGraphFormulaToken::Function {
                function_type: 16,
                arguments_count: 0,
                random_mode: 1,
            }],
        };
        let mut prototype =
            RuntimeDataBindGraph::new_key_frame_bindings(&[number_key_frame_binding(Some(
                converter,
            ))])
            .expect("keyframe binding graph");
        prototype.set_formula_random_values(&[0.25, 0.75]);
        assert!(prototype.bind_default_view_model_context());

        let mut first = LinearAnimationInstance::new(0, &animation, 1.0);
        let mut second = LinearAnimationInstance::new(0, &animation, 1.0);
        first.prepare_key_frame_data_binds(Some(&prototype));
        second.prepare_key_frame_data_binds(Some(&prototype));

        assert_eq!(
            first.key_frame_value_holder(10),
            Some(&RuntimeKeyFrameValue::Number(0.25))
        );
        assert_eq!(
            second.key_frame_value_holder(10),
            Some(&RuntimeKeyFrameValue::Number(0.25))
        );
        assert_ne!(
            first
                .key_frame_data_bind_graph
                .as_deref()
                .map(|graph| graph as *const RuntimeDataBindGraph),
            second
                .key_frame_data_bind_graph
                .as_deref()
                .map(|graph| graph as *const RuntimeDataBindGraph)
        );
        assert_eq!(
            first
                .key_frame_data_bind_graph
                .as_deref()
                .map(RuntimeDataBindGraph::formula_random_call_count),
            Some(1)
        );
        assert_eq!(
            second
                .key_frame_data_bind_graph
                .as_deref()
                .map(RuntimeDataBindGraph::formula_random_call_count),
            Some(1)
        );
    }

    #[test]
    fn empty_context_uses_cpp_typed_key_frame_holder_defaults() {
        let animation = animation_with_work_area(false);
        let template = RuntimeKeyFrameDataBindTemplate {
            data_bind_index: 0,
            key_frame_global_id: 30,
            target: RuntimeKeyFrameDataBindTarget::Color,
            path: vec![0, 0],
            flags: 0,
            converter: None,
            default_value: crate::RuntimeDataBindGraphValue::Color(0),
        };
        let mut prototype = RuntimeDataBindGraph::new_key_frame_bindings(&[template])
            .expect("keyframe binding graph");
        assert!(prototype.bind_empty_data_context());
        let mut instance = LinearAnimationInstance::new(0, &animation, 1.0);
        instance.prepare_key_frame_data_binds(Some(&prototype));
        assert_eq!(
            instance.key_frame_value_holder(30),
            Some(&RuntimeKeyFrameValue::Color(0xFF1D1D1D))
        );
    }

    #[test]
    fn duration_seconds_respects_enabled_work_area() {
        let animation = animation_with_work_area(true);

        assert_eq!(animation.start_seconds(), 10.0 / 60.0);
        assert_eq!(animation.duration_seconds(), 30.0 / 60.0);
    }

    #[test]
    fn duration_seconds_uses_serialized_duration_without_work_area() {
        let animation = animation_with_work_area(false);

        assert_eq!(animation.start_seconds(), 0.0);
        assert_eq!(animation.duration_seconds(), 1.0);
    }

    #[test]
    fn ping_pong_zero_duration_is_finite_and_deterministic() {
        // duration==0 makes C++ compute (int)(seconds / 0) -> UB and Rust
        // saturate an inf/NaN cast; guard collapses both branches to the single
        // frame. Must never return NaN/inf regardless of the sampled second.
        let animation = RuntimeLinearAnimation {
            global_id: 1,
            name: None,
            fps: 60,
            duration: 0,
            speed: 1.0,
            loop_value: 2, // PingPong
            work_start: 0,
            work_end: 0,
            enable_work_area: false,
            quantize: false,
            keyed_objects: Arc::new(Vec::new()),
            key_frame_data_bind_templates: Arc::new(Vec::new()),
            has_keyed_callbacks: false,
        };

        assert_eq!(animation.duration_seconds(), 0.0);
        for seconds in [-2.0_f32, -0.5, 0.0, 0.5, 2.0, 1000.0] {
            let local = animation.global_to_local_seconds(seconds);
            assert!(local.is_finite(), "local time not finite for {seconds}");
            assert_eq!(local, animation.start_seconds());
        }
    }
}
