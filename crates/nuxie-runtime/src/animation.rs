use crate::artboard_data_bind::build_key_frame_data_bind_templates;
use crate::data_bind_graph::{
    RuntimeDataBindGraphConverterBuildCache, RuntimeKeyFrameDataBindTemplate,
};
use crate::draw::color_lerp;
use crate::properties::{
    artboard_index_for_graph, mix_value, solid_color_value_property_key, transform_property_for_key,
};
use crate::{ArtboardInstance, InstanceSlot, StateMachineReportedEvent, TransformProperty};
use nuxie_binary::{RuntimeFile, RuntimeImportStatus, RuntimeObject};
use nuxie_graph::ArtboardGraph;
use nuxie_schema::{
    CoreRegistryFieldKind, core_registry_field_kind_by_property_key, definition_by_type_key,
    is_callback_property_key, object_supports_property,
};
use std::collections::HashMap;
use std::sync::Arc;

mod linear_animation_instance;

pub use linear_animation_instance::{LinearAnimationInstance, RuntimeLinearAnimationHandle};

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

    Some(StateMachineReportedEvent::from_runtime_event(
        target_local_id,
        target,
    ))
}

fn keyed_property_target(
    target_local_id: usize,
    target: &RuntimeObject,
    property_key: u16,
) -> Option<RuntimeKeyedPropertyTarget> {
    if is_callback_property_key(property_key) {
        return Some(RuntimeKeyedPropertyTarget::Callback {
            event: callback_event_for_keyed_property(target_local_id, target, property_key),
        });
    }

    let transform_property = transform_property_for_key(property_key);
    match core_registry_field_kind_by_property_key(property_key)? {
        CoreRegistryFieldKind::Double => {
            Some(RuntimeKeyedPropertyTarget::Double { transform_property })
        }
        CoreRegistryFieldKind::Color => Some(RuntimeKeyedPropertyTarget::Color {
            solid_color_property: target.type_name == "SolidColor"
                && solid_color_value_property_key() == Some(property_key),
            data_bind_observed: false,
        }),
        CoreRegistryFieldKind::Bool => Some(RuntimeKeyedPropertyTarget::Bool),
        CoreRegistryFieldKind::Uint => Some(RuntimeKeyedPropertyTarget::Uint),
        CoreRegistryFieldKind::StringOrBytes => Some(RuntimeKeyedPropertyTarget::String),
    }
}

// Mirrors KeyFrame::computeSeconds (`src/animation/keyframe.cpp`) as invoked
// once by KeyedPropertyImporter::resolve (`src/importers/keyed_property_importer.cpp`).
fn retained_key_frame_seconds(frame: u64, fps: u64) -> f32 {
    frame as f32 / fps as f32
}

pub(crate) fn build_linear_animations<'a>(
    file: &'a RuntimeFile,
    graph: &ArtboardGraph,
    slots: &[InstanceSlot],
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
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
            let Some(target) = keyed_property_target(target_local_id, target, property_key) else {
                current_keyed_property = None;
                continue;
            };

            let keyed_objects = Arc::make_mut(&mut animations[animation_index].keyed_objects);
            keyed_objects[keyed_object_index]
                .keyed_properties
                .push(RuntimeKeyedProperty {
                    global_id: global_id as u32,
                    property_key,
                    target,
                    key_frames: Vec::new(),
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
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .key_frames
            .push(RuntimeKeyFrame::Double(RuntimeKeyFrameDouble {
                global_id: global_id as u32,
                frame,
                seconds,
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                interpolator: runtime_key_frame_interpolator(file, artboard_index, object),
                value: object.double_property("value").unwrap_or(0.0),
            }));
        }

        if object.type_name == "KeyFrameColor" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .key_frames
            .push(RuntimeKeyFrame::Color(RuntimeKeyFrameColor {
                global_id: global_id as u32,
                frame,
                seconds,
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                interpolator: runtime_key_frame_interpolator(file, artboard_index, object),
                value: object.color_property("value").unwrap_or(0),
            }));
        }

        if object.type_name == "KeyFrameBool" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .key_frames
            .push(RuntimeKeyFrame::Bool(RuntimeKeyFrameBool {
                global_id: global_id as u32,
                frame,
                seconds,
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                value: object.bool_property("value").unwrap_or(false),
            }));
        }

        if object.type_name == "KeyFrameUint" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .key_frames
            .push(RuntimeKeyFrame::Uint(RuntimeKeyFrameUint {
                global_id: global_id as u32,
                frame,
                seconds,
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                value: object.uint_property("value").unwrap_or(0),
            }));
        }

        if object.type_name == "KeyFrameId" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .key_frames
            .push(RuntimeKeyFrame::Uint(RuntimeKeyFrameUint {
                global_id: global_id as u32,
                frame,
                seconds,
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                value: object.uint_property("value").unwrap_or(0),
            }));
        }

        if object.type_name == "KeyFrameString" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .key_frames
            .push(RuntimeKeyFrame::String(RuntimeKeyFrameString {
                global_id: global_id as u32,
                frame,
                seconds,
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                value: object
                    .string_property_bytes("value")
                    .unwrap_or_default()
                    .to_vec(),
            }));
        }

        if object.type_name == "KeyFrameCallback" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            animations[animation_index].has_keyed_callbacks = true;
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .key_frames
            .push(RuntimeKeyFrame::Callback(RuntimeKeyFrameCallback {
                global_id: global_id as u32,
                frame,
                seconds,
            }));
        }
    }

    let templates = build_key_frame_data_bind_templates(file, artboard_index, converter_cache);
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
                        .filter_map(RuntimeKeyFrame::bindable_global_id)
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
#[derive(Debug)]
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
    pub(crate) fn empty() -> Self {
        Self {
            global_id: u32::MAX,
            name: None,
            fps: 60,
            duration: 60,
            speed: 1.0,
            loop_value: 0,
            work_start: u64::from(u32::MAX),
            work_end: u64::from(u32::MAX),
            enable_work_area: false,
            quantize: false,
            keyed_objects: Arc::new(Vec::new()),
            key_frame_data_bind_templates: Arc::new(Vec::new()),
            has_keyed_callbacks: false,
        }
    }

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
        let seconds = if self.quantize {
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
                match &keyed_property.target {
                    RuntimeKeyedPropertyTarget::Double { transform_property } => {
                        let Some(frame_value) =
                            keyed_property.double_frame_value_at(seconds, key_frame_values)
                        else {
                            continue;
                        };
                        let Some(value) =
                            apply_key_frame_double_mix(
                                frame_value,
                                mix,
                                || match transform_property {
                                    Some(property) => instance.transform_property_with_key(
                                        keyed_object.target_local_id,
                                        *property,
                                        keyed_property.property_key,
                                    ),
                                    None => instance.double_property(
                                        keyed_object.target_local_id,
                                        keyed_property.property_key,
                                    ),
                                },
                            )
                        else {
                            continue;
                        };
                        changed |= match transform_property {
                            Some(property) => instance.set_transform_property_with_key(
                                keyed_object.target_local_id,
                                *property,
                                keyed_property.property_key,
                                value,
                            ),
                            None => instance.set_keyed_double_property(
                                keyed_object.target_local_id,
                                keyed_property.property_key,
                                value,
                            ),
                        };
                    }
                    RuntimeKeyedPropertyTarget::Color {
                        solid_color_property,
                        data_bind_observed,
                    } => {
                        let Some(frame_value) =
                            keyed_property.color_frame_value_at(seconds, key_frame_values)
                        else {
                            continue;
                        };
                        let Some(value) = apply_key_frame_color_mix(frame_value, mix, || {
                            if *solid_color_property {
                                instance.solid_color_value(keyed_object.target_local_id)
                            } else {
                                instance.color_property(
                                    keyed_object.target_local_id,
                                    keyed_property.property_key,
                                )
                            }
                        }) else {
                            continue;
                        };
                        changed |= if *solid_color_property {
                            instance.set_keyed_solid_color_property(
                                keyed_object.target_local_id,
                                keyed_property.property_key,
                                *data_bind_observed,
                                value,
                            )
                        } else {
                            instance.set_keyed_color_property(
                                keyed_object.target_local_id,
                                keyed_property.property_key,
                                value,
                            )
                        };
                    }
                    RuntimeKeyedPropertyTarget::Bool => {
                        let Some(value) = keyed_property.bool_value_at(seconds, key_frame_values)
                        else {
                            continue;
                        };
                        changed |= instance.set_bool_property(
                            keyed_object.target_local_id,
                            keyed_property.property_key,
                            value,
                        );
                    }
                    RuntimeKeyedPropertyTarget::Uint => {
                        let Some(value) = keyed_property.uint_value_at(seconds) else {
                            continue;
                        };
                        changed |= instance.set_uint_property(
                            keyed_object.target_local_id,
                            keyed_property.property_key,
                            value,
                        );
                    }
                    RuntimeKeyedPropertyTarget::String => {
                        let Some(value) = keyed_property.string_value_at(seconds, key_frame_values)
                        else {
                            continue;
                        };
                        changed |= instance.set_string_property(
                            keyed_object.target_local_id,
                            keyed_property.property_key,
                            value,
                        );
                    }
                    RuntimeKeyedPropertyTarget::Callback { .. } => {}
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
        match AnimationLoop::from_loop_value(self.loop_value as i32) {
            AnimationLoop::OneShot => seconds + self.start_seconds(),
            AnimationLoop::Loop => {
                positive_mod(seconds, self.duration_seconds()) + self.start_seconds()
            }
            AnimationLoop::PingPong => {
                let duration = self.duration_seconds();
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
    /// Rust's type-safe binding for C++ CoreRegistry's single virtual property
    /// dispatch. Exactly one target family is retained per KeyedProperty.
    pub target: RuntimeKeyedPropertyTarget,
    /// Mirrors C++ `KeyedProperty::m_keyFrames`: one insertion-ordered owner
    /// sequence containing the concrete KeyFrame occurrence.
    pub key_frames: Vec<RuntimeKeyFrame>,
}

#[derive(Debug, Clone)]
pub enum RuntimeKeyedPropertyTarget {
    Double {
        transform_property: Option<TransformProperty>,
    },
    Color {
        /// The import-time equivalent of C++'s concrete `SolidColor*` target.
        solid_color_property: bool,
        /// C++ keeps an intrusive observer head on each concrete Core object.
        /// Rust resolves the equivalent subscription once at artboard build.
        data_bind_observed: bool,
    },
    Bool,
    Uint,
    String,
    Callback {
        event: Option<StateMachineReportedEvent>,
    },
}

impl RuntimeKeyedPropertyTarget {
    pub(crate) fn set_data_bind_observed(&mut self, observed: bool) {
        if let Self::Color {
            data_bind_observed, ..
        } = self
        {
            *data_bind_observed = observed;
        }
    }
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
    pub(crate) fn first_double_value(&self) -> Option<f32> {
        self.key_frames
            .first()?
            .as_double()
            .map(|frame| frame.value)
    }

    pub(crate) fn first_color_value(&self) -> Option<u32> {
        self.key_frames.first()?.as_color().map(|frame| frame.value)
    }

    fn double_frame_value_at(
        &self,
        seconds: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<f32> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        let value = if idx == 0 {
            self.key_frames[0]
                .as_double()?
                .effective_value(key_frame_values)
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].as_double()?;
            let to = self.key_frames[idx].as_double()?;
            if seconds == to.seconds {
                to.effective_value(key_frame_values)
            } else if from.interpolation_type == 0 {
                from.effective_value(key_frame_values)
            } else if from.interpolator_id.is_some() {
                let frame_mix = frame_mix(seconds, from.seconds, to.seconds);
                from.interpolator?.transform_value(
                    from.effective_value(key_frame_values),
                    to.effective_value(key_frame_values),
                    frame_mix,
                )
            } else {
                let frame_mix = frame_mix(seconds, from.seconds, to.seconds);
                let from_value = from.effective_value(key_frame_values);
                let to_value = to.effective_value(key_frame_values);
                from_value + (to_value - from_value) * frame_mix
            }
        } else {
            self.key_frames
                .last()?
                .as_double()?
                .effective_value(key_frame_values)
        };

        Some(value)
    }

    fn color_frame_value_at(
        &self,
        seconds: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<u32> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        let value = if idx == 0 {
            self.key_frames[0]
                .as_color()?
                .effective_value(key_frame_values)
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].as_color()?;
            let to = self.key_frames[idx].as_color()?;
            if seconds == to.seconds {
                to.effective_value(key_frame_values)
            } else if from.interpolation_type == 0 {
                from.effective_value(key_frame_values)
            } else if from.interpolator_id.is_some() {
                let frame_mix = frame_mix(seconds, from.seconds, to.seconds);
                color_lerp(
                    from.effective_value(key_frame_values),
                    to.effective_value(key_frame_values),
                    from.interpolator?.transform(frame_mix),
                )
            } else {
                let frame_mix = frame_mix(seconds, from.seconds, to.seconds);
                color_lerp(
                    from.effective_value(key_frame_values),
                    to.effective_value(key_frame_values),
                    frame_mix,
                )
            }
        } else {
            self.key_frames
                .last()?
                .as_color()?
                .effective_value(key_frame_values)
        };

        Some(value)
    }

    fn bool_value_at(
        &self,
        seconds: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<bool> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        let value = if idx == 0 {
            self.key_frames[0]
                .as_bool()?
                .effective_value(key_frame_values)
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].as_bool()?;
            let to = self.key_frames[idx].as_bool()?;
            if seconds == to.seconds {
                to.effective_value(key_frame_values)
            } else {
                from.effective_value(key_frame_values)
            }
        } else {
            self.key_frames
                .last()?
                .as_bool()?
                .effective_value(key_frame_values)
        };

        Some(value)
    }

    fn uint_value_at(&self, seconds: f32) -> Option<u64> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        let value = if idx == 0 {
            self.key_frames[0].as_uint()?.value
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].as_uint()?;
            let to = self.key_frames[idx].as_uint()?;
            if seconds == to.seconds {
                to.value
            } else {
                from.value
            }
        } else {
            self.key_frames.last()?.as_uint()?.value
        };

        Some(value)
    }

    fn string_value_at(
        &self,
        seconds: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<Vec<u8>> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        let key_frame = if idx == 0 {
            self.key_frames[0].as_string()?
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].as_string()?;
            let to = self.key_frames[idx].as_string()?;
            if seconds == to.seconds { to } else { from }
        } else {
            self.key_frames.last()?.as_string()?
        };

        Some(key_frame.effective_value(key_frame_values))
    }

    fn report_keyed_callbacks(
        &self,
        target_local_id: usize,
        seconds_from: f32,
        seconds_to: f32,
        is_at_start_frame: bool,
        reported_events: &mut Vec<StateMachineReportedEvent>,
        keyed_callbacks: &mut Vec<RuntimeKeyedCallback>,
    ) {
        if self.key_frames.is_empty() || seconds_from == seconds_to {
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
            &self.key_frames,
            seconds_from,
            from_exact_offset,
        );
        let mut index_to = closest_key_frame_index_with_exact_offset(
            &self.key_frames,
            seconds_to,
            to_exact_offset,
        );
        if index_to < index {
            std::mem::swap(&mut index, &mut index_to);
        }

        while index_to > index {
            let key_frame = &self.key_frames[index];
            let seconds_delay = seconds_to - key_frame.seconds();
            keyed_callbacks.push(RuntimeKeyedCallback {
                target_local_id,
                property_key: self.property_key,
                seconds_delay,
            });
            if let RuntimeKeyedPropertyTarget::Callback { event: Some(event) } = &self.target {
                let mut reported_event = event.clone();
                reported_event.seconds_delay = seconds_delay;
                reported_events.push(reported_event);
            }
            index += 1;
        }
    }

    fn closest_frame_index(&self, seconds: f32) -> usize {
        closest_key_frame_index(&self.key_frames, seconds)
    }
}

fn closest_key_frame_index(key_frames: &[RuntimeKeyFrame], seconds: f32) -> usize {
    closest_key_frame_index_with_exact_offset(key_frames, seconds, 0)
}

fn closest_key_frame_index_with_exact_offset(
    key_frames: &[RuntimeKeyFrame],
    seconds: f32,
    exact_offset: usize,
) -> usize {
    let last = key_frames.len() - 1;
    if seconds > key_frames[last].seconds() {
        return key_frames.len();
    }

    let mut start = 0;
    let mut end = last;
    while start <= end {
        let mid = (start + end) >> 1;
        let closest = key_frames[mid].seconds();
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

/// The concrete keyframe occurrence owned by a `RuntimeKeyedProperty`.
///
/// Mirrors C++ `KeyedProperty::m_keyFrames`: concrete subclasses share one
/// insertion-ordered owner sequence instead of being partitioned by Rust type.
#[derive(Debug, Clone)]
pub enum RuntimeKeyFrame {
    Double(RuntimeKeyFrameDouble),
    Color(RuntimeKeyFrameColor),
    Bool(RuntimeKeyFrameBool),
    Uint(RuntimeKeyFrameUint),
    String(RuntimeKeyFrameString),
    Callback(RuntimeKeyFrameCallback),
}

impl RuntimeKeyFrame {
    fn global_id(&self) -> u32 {
        match self {
            Self::Double(frame) => frame.global_id,
            Self::Color(frame) => frame.global_id,
            Self::Bool(frame) => frame.global_id,
            Self::Uint(frame) => frame.global_id,
            Self::String(frame) => frame.global_id,
            Self::Callback(frame) => frame.global_id,
        }
    }

    fn seconds(&self) -> f32 {
        match self {
            Self::Double(frame) => frame.seconds,
            Self::Color(frame) => frame.seconds,
            Self::Bool(frame) => frame.seconds,
            Self::Uint(frame) => frame.seconds,
            Self::String(frame) => frame.seconds,
            Self::Callback(frame) => frame.seconds,
        }
    }

    fn bindable_global_id(&self) -> Option<u32> {
        match self {
            Self::Double(_) | Self::Color(_) | Self::Bool(_) | Self::String(_) => {
                Some(self.global_id())
            }
            Self::Uint(_) | Self::Callback(_) => None,
        }
    }

    fn as_double(&self) -> Option<&RuntimeKeyFrameDouble> {
        match self {
            Self::Double(frame) => Some(frame),
            _ => None,
        }
    }

    fn as_color(&self) -> Option<&RuntimeKeyFrameColor> {
        match self {
            Self::Color(frame) => Some(frame),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<&RuntimeKeyFrameBool> {
        match self {
            Self::Bool(frame) => Some(frame),
            _ => None,
        }
    }

    fn as_uint(&self) -> Option<&RuntimeKeyFrameUint> {
        match self {
            Self::Uint(frame) => Some(frame),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&RuntimeKeyFrameString> {
        match self {
            Self::String(frame) => Some(frame),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameDouble {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
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
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameColor {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub(crate) interpolator: Option<RuntimeInterpolator>,
    pub value: u32,
}

impl RuntimeKeyFrameColor {
    fn effective_value(&self, key_frame_values: RuntimeKeyFrameValueContext<'_>) -> u32 {
        key_frame_values.color(self.global_id).unwrap_or(self.value)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameBool {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
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
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameUint {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: u64,
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameString {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
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
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameCallback {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
}

// Mirrors src/animation/linear_animation_instance.cpp and include/rive/animation/loop.hpp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnimationLoop {
    OneShot,
    Loop,
    PingPong,
}

impl AnimationLoop {
    pub(crate) fn from_loop_value(value: i32) -> Self {
        match value {
            1 => Self::Loop,
            2 => Self::PingPong,
            _ => Self::OneShot,
        }
    }
}

fn positive_mod(value: f32, range: f32) -> f32 {
    ((value % range) + range) % range
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_bind_graph::{
        RuntimeDataBindGraph, RuntimeDataBindGraphConverter, RuntimeDataBindGraphFormulaToken,
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
            target: RuntimeKeyedPropertyTarget::Double {
                transform_property: None,
            },
            key_frames: vec![
                RuntimeKeyFrame::Double(RuntimeKeyFrameDouble {
                    global_id: from_global_id,
                    frame: 0,
                    seconds: 0.0,
                    interpolation_type: 1,
                    interpolator_id: None,
                    interpolator: None,
                    value: from_value,
                }),
                RuntimeKeyFrame::Double(RuntimeKeyFrameDouble {
                    global_id: to_global_id,
                    frame: 10,
                    seconds: 1.0,
                    interpolation_type: 1,
                    interpolator_id: None,
                    interpolator: None,
                    value: to_value,
                }),
            ],
        }
    }

    fn callback_frame(global_id: u32, frame: u64, seconds: f32) -> RuntimeKeyFrame {
        RuntimeKeyFrame::Callback(RuntimeKeyFrameCallback {
            global_id,
            frame,
            seconds,
        })
    }

    #[test]
    fn key_frame_seconds_are_retained_at_attachment() {
        let mut animation = animation_with_work_area(false);
        let property = keyed_double_property(10, 1.0, 20, 2.0);

        animation.fps = 20;

        assert_eq!(property.key_frames[1].seconds(), 1.0);
        assert_eq!(
            property.double_frame_value_at(0.5, RuntimeKeyFrameValueContext::default()),
            Some(1.5)
        );
    }

    #[test]
    fn zero_fps_seconds_follow_cpp_float_division() {
        assert!(retained_key_frame_seconds(0, 0).is_nan());
        assert_eq!(retained_key_frame_seconds(10, 0), f32::INFINITY);
    }

    #[test]
    fn keyed_property_retains_one_mixed_concrete_frame_sequence() {
        let frames = vec![
            callback_frame(10, 0, 0.0),
            RuntimeKeyFrame::Double(RuntimeKeyFrameDouble {
                global_id: 20,
                frame: 10,
                seconds: 1.0,
                interpolation_type: 1,
                interpolator_id: None,
                interpolator: None,
                value: 2.0,
            }),
            RuntimeKeyFrame::Color(RuntimeKeyFrameColor {
                global_id: 30,
                frame: 20,
                seconds: 2.0,
                interpolation_type: 1,
                interpolator_id: None,
                interpolator: None,
                value: 0,
            }),
        ];

        assert_eq!(
            frames
                .iter()
                .map(RuntimeKeyFrame::global_id)
                .collect::<Vec<_>>(),
            vec![10, 20, 30]
        );
        assert!(matches!(frames[0], RuntimeKeyFrame::Callback(_)));
        assert!(matches!(frames[1], RuntimeKeyFrame::Double(_)));
        assert!(matches!(frames[2], RuntimeKeyFrame::Color(_)));
    }

    #[test]
    fn closest_frame_index_matches_cpp_exact_offsets_and_duplicates() {
        let frames = vec![
            callback_frame(10, 0, 0.0),
            callback_frame(20, 10, 1.0),
            callback_frame(30, 10, 1.0),
            callback_frame(40, 20, 2.0),
        ];

        assert_eq!(closest_key_frame_index(&frames, -1.0), 0);
        assert_eq!(closest_key_frame_index(&frames, 0.5), 1);
        assert_eq!(
            closest_key_frame_index_with_exact_offset(&frames, 1.0, 0),
            1
        );
        assert_eq!(
            closest_key_frame_index_with_exact_offset(&frames, 1.0, 1),
            2
        );
        assert_eq!(closest_key_frame_index(&frames, 2.1), frames.len());
    }

    #[test]
    fn data_bound_double_interpolation_uses_both_effective_endpoints() {
        let animation = animation_with_work_area(false);
        let mut instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        instance.add_key_frame_value_holder(10, RuntimeKeyFrameValue::Number(100.0));
        instance.add_key_frame_value_holder(20, RuntimeKeyFrameValue::Number(200.0));
        let property = keyed_double_property(10, 1.0, 20, 2.0);

        assert_eq!(
            property.double_frame_value_at(0.5, instance.key_frame_value_context()),
            Some(150.0)
        );
    }

    #[test]
    fn data_bound_color_interpolation_uses_both_effective_endpoints() {
        let animation = animation_with_work_area(false);
        let mut instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        let bound_from = 0xFF00_0000;
        let bound_to = 0xFFFF_FFFF;
        instance.add_key_frame_value_holder(30, RuntimeKeyFrameValue::Color(bound_from));
        instance.add_key_frame_value_holder(40, RuntimeKeyFrameValue::Color(bound_to));
        let mut property = keyed_double_property(10, 1.0, 20, 2.0);
        property.key_frames.clear();
        property.target = RuntimeKeyedPropertyTarget::Color {
            solid_color_property: false,
            data_bind_observed: false,
        };
        property.key_frames = vec![
            RuntimeKeyFrame::Color(RuntimeKeyFrameColor {
                global_id: 30,
                frame: 0,
                seconds: 0.0,
                interpolation_type: 1,
                interpolator_id: None,
                interpolator: None,
                value: 0xFF00_FF00,
            }),
            RuntimeKeyFrame::Color(RuntimeKeyFrameColor {
                global_id: 40,
                frame: 10,
                seconds: 1.0,
                interpolation_type: 1,
                interpolator_id: None,
                interpolator: None,
                value: 0xFF00_00FF,
            }),
        ];

        assert_eq!(
            property.color_frame_value_at(0.5, instance.key_frame_value_context()),
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
        let mut instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        instance.add_key_frame_value_holder(50, RuntimeKeyFrameValue::Boolean(true));
        let mut property = keyed_double_property(10, 1.0, 20, 2.0);
        property.key_frames.clear();
        property.target = RuntimeKeyedPropertyTarget::Bool;
        property.key_frames = vec![
            RuntimeKeyFrame::Bool(RuntimeKeyFrameBool {
                global_id: 50,
                frame: 0,
                seconds: 0.0,
                interpolation_type: 1,
                interpolator_id: None,
                value: false,
            }),
            RuntimeKeyFrame::Bool(RuntimeKeyFrameBool {
                global_id: 60,
                frame: 10,
                seconds: 1.0,
                interpolation_type: 1,
                interpolator_id: None,
                value: false,
            }),
        ];

        assert_eq!(
            property.bool_value_at(0.5, instance.key_frame_value_context()),
            Some(true)
        );
    }

    #[test]
    fn data_bound_string_step_uses_the_effective_current_key_frame() {
        let animation = animation_with_work_area(false);
        let mut instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        instance
            .add_key_frame_value_holder(70, RuntimeKeyFrameValue::String(b"bound start".to_vec()));
        let mut property = keyed_double_property(10, 1.0, 20, 2.0);
        property.key_frames.clear();
        property.target = RuntimeKeyedPropertyTarget::String;
        property.key_frames = vec![
            RuntimeKeyFrame::String(RuntimeKeyFrameString {
                global_id: 70,
                frame: 0,
                seconds: 0.0,
                interpolation_type: 1,
                interpolator_id: None,
                value: b"authored start".to_vec(),
            }),
            RuntimeKeyFrame::String(RuntimeKeyFrameString {
                global_id: 80,
                frame: 10,
                seconds: 1.0,
                interpolation_type: 1,
                interpolator_id: None,
                value: b"authored end".to_vec(),
            }),
        ];

        assert_eq!(
            property.string_value_at(0.5, instance.key_frame_value_context()),
            Some(b"bound start".to_vec())
        );
    }

    #[test]
    fn cloned_animation_instance_starts_without_key_frame_value_holders() {
        let animation = animation_with_work_area(false);
        let mut instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        instance.add_key_frame_value_holder(10, RuntimeKeyFrameValue::Number(123.0));

        let cloned = instance.clone();

        assert_eq!(
            instance.key_frame_value_holder(10),
            Some(&RuntimeKeyFrameValue::Number(123.0))
        );
        assert_eq!(cloned.key_frame_value_holder(10), None);
        assert_eq!(cloned.animation, instance.animation);
        assert_eq!(cloned.loop_value_override, instance.loop_value_override);
    }

    #[test]
    fn raw_loop_override_retains_cpp_minus_one_sentinel() {
        let animation = animation_with_work_area(false);
        let mut instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(7), &animation, 1.0);

        assert_eq!(instance.animation_index(), 7);
        assert_eq!(instance.loop_value_override, -1);
        assert_eq!(instance.resolved_loop_kind(&animation), AnimationLoop::Loop);

        // linear_animation_instance.cpp:426-434 leaves the sentinel untouched
        // when the requested value already equals the definition.
        instance.set_loop_value(&animation, animation.loop_value as i32);
        assert_eq!(instance.loop_value_override, -1);

        instance.set_loop_value(&animation, 2);
        assert_eq!(instance.loop_value_override, 2);
        assert_eq!(
            instance.resolved_loop_kind(&animation),
            AnimationLoop::PingPong
        );

        instance.set_loop_value(&animation, -1);
        assert_eq!(instance.loop_value_override, -1);
        assert_eq!(instance.resolved_loop_kind(&animation), AnimationLoop::Loop);
    }

    #[test]
    fn pre_advance_did_loop_is_safe_false_then_every_advance_writes_it() {
        let animation = animation_with_work_area(false);
        let mut instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);

        // Binding adaptation: pinned C++ leaves m_didLoop indeterminate until
        // advance; safe Rust exposes a deterministic false.
        assert!(!instance.did_loop());
        assert!(instance.advance(&animation, 2.0));
        assert!(instance.did_loop());
        assert!(!instance.advance(&animation, 0.0));
        assert!(!instance.did_loop());
    }

    #[test]
    fn time_and_reset_follow_cpp_occurrence_state_rules() {
        let animation = animation_with_work_area(true);
        let mut instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        instance.total_time = 9.0;
        instance.last_total_time = 7.0;
        instance.direction = -1.0;

        instance.set_time(&animation, 4.0);

        // linear_animation_instance.cpp:367-380 retains the prior total-time
        // delta and uses the authored workStart*fps expression verbatim.
        assert_eq!(instance.total_time, 4.0 - 10.0 * 60.0);
        assert_eq!(instance.last_total_time, instance.total_time - 2.0);
        assert_eq!(instance.direction, 1.0);

        instance.direction = -1.0;
        instance.did_loop = true;
        instance.reset(&animation, -1.0);
        assert_eq!(instance.time(), animation.end_seconds());
        assert_eq!(instance.direction(), -1.0);
        assert!(instance.did_loop());
    }

    #[test]
    fn key_frame_value_holders_are_isolated_per_animation_instance() {
        let animation = animation_with_work_area(false);
        let property = keyed_double_property(10, 1.0, 20, 2.0);
        let mut first =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        let mut second =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        let unbound =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        first.add_key_frame_value_holder(10, RuntimeKeyFrameValue::Number(100.0));
        second.add_key_frame_value_holder(10, RuntimeKeyFrameValue::Number(200.0));
        *first.key_frame_value_holder_mut(10).unwrap() = RuntimeKeyFrameValue::Number(150.0);

        assert_eq!(
            property.double_frame_value_at(0.0, first.key_frame_value_context()),
            Some(150.0)
        );
        assert_eq!(
            property.double_frame_value_at(0.0, second.key_frame_value_context()),
            Some(200.0)
        );
        assert_eq!(
            property.double_frame_value_at(0.0, unbound.key_frame_value_context()),
            Some(1.0)
        );
    }

    #[test]
    fn uint_and_id_sampling_ignore_key_frame_value_holders() {
        let animation = animation_with_work_area(false);
        let mut instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        instance.add_key_frame_value_holder(90, RuntimeKeyFrameValue::Number(999.0));
        let mut property = keyed_double_property(10, 1.0, 20, 2.0);
        property.key_frames.clear();
        property.target = RuntimeKeyedPropertyTarget::Uint;
        property.key_frames = vec![RuntimeKeyFrame::Uint(RuntimeKeyFrameUint {
            global_id: 90,
            frame: 0,
            seconds: 0.0,
            interpolation_type: 1,
            interpolator_id: None,
            value: 7,
        })];

        // KeyFrameUint and KeyFrameId share this runtime sampler. Upstream
        // intentionally leaves both types unsupported by keyframe value binds.
        assert_eq!(property.uint_value_at(0.0), Some(7));
    }

    #[test]
    fn state_machine_key_frame_graph_updates_one_instance_without_binding_standalone_clones() {
        let animation = animation_with_work_area(false);
        let mut prototype =
            RuntimeDataBindGraph::new_key_frame_bindings(&[number_key_frame_binding(None)])
                .expect("keyframe binding graph");
        assert!(prototype.bind_default_view_model_context());
        assert!(prototype.set_default_view_model_number_source_for_path(&[0, 0], 10.0));

        let mut state_machine_instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
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

        let mut first =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
        let mut second =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
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
        let mut instance =
            LinearAnimationInstance::new(RuntimeLinearAnimationHandle::new(0), &animation, 1.0);
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
    fn ping_pong_zero_duration_keeps_the_unguarded_cpp_arithmetic_shape() {
        // linear_animation.cpp:132-144 has no zero-duration branch. Rust float
        // remainder therefore produces NaN directly; its float-to-int cast is
        // defined and memory-safe, while C++'s corresponding cast is undefined.
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
            assert!(
                local.is_nan(),
                "local time unexpectedly finite for {seconds}"
            );
        }
    }
}
