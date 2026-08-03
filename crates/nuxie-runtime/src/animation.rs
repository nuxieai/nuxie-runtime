use crate::artboard_data_bind::build_key_frame_data_bind_templates;
use crate::data_bind_graph::{
    RuntimeDataBindGraph, RuntimeDataBindGraphApplyPhase, RuntimeDataBindGraphConverterBuildCache,
    RuntimeDataBindGraphTarget, RuntimeKeyFrameDataBindTemplate,
};
use crate::draw::color_lerp;
use crate::properties::{
    artboard_index_for_graph, mix_value, solid_color_value_property_key, transform_property_for_key,
};
use crate::scripted_interpolator::RuntimeScriptedInterpolatorState;
use crate::{ArtboardInstance, InstanceSlot, StateMachineReportedEvent, TransformProperty};
use crate::{RuntimeScriptedInterpolatorDiagnostic, ScriptInterpolatorMethod};
use nuxie_binary::{RuntimeFile, RuntimeImportStatus, RuntimeObject};
use nuxie_graph::ArtboardGraph;
use nuxie_schema::{
    CoreRegistryFieldKind, core_registry_field_kind_by_property_key,
    core_registry_setter_field_kind_by_property_key, definition_by_type_key,
    is_callback_property_key, object_supports_property,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RuntimeKeyFrameDataBindOccurrenceId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeKeyFrameDataBindEnrollment {
    Initial,
    Late,
}

include!("animation/keyframe_interpolator.rs");
include!("animation/cubic_interpolator.rs");
include!("animation/cubic_interpolator_solver.rs");
include!("animation/cubic_value_interpolator.rs");
include!("animation/elastic_interpolator.rs");
include!("animation/elastic_ease.rs");

fn callback_event_for_keyed_property(
    target_local_id: usize,
    target: &RuntimeObject,
    property_key: u16,
) -> Option<usize> {
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

    Some(target_local_id)
}

fn keyed_property_target(
    target_local_id: usize,
    target: &RuntimeObject,
    property_key: u16,
) -> Option<RuntimeKeyedPropertyTarget> {
    if is_callback_property_key(property_key) {
        return Some(RuntimeKeyedPropertyTarget::Callback {
            event_local_index: callback_event_for_keyed_property(
                target_local_id,
                target,
                property_key,
            ),
        });
    }

    let transform_property = transform_property_for_key(property_key);
    // Generated bitmask passthrough properties have no independent storage
    // kind, but their generated setter kind still determines the keyframe
    // representation (for example `SemanticData.isSelected` is a bool).
    let field_kind = core_registry_field_kind_by_property_key(property_key).or_else(|| {
        core_registry_setter_field_kind_by_property_key(property_key)
            .and_then(CoreRegistryFieldKind::from_field_kind)
    })?;
    match field_kind {
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
        CoreRegistryFieldKind::Int => Some(RuntimeKeyedPropertyTarget::Int),
        CoreRegistryFieldKind::StringOrBytes => Some(RuntimeKeyedPropertyTarget::String),
    }
}

include!("animation/keyframe.rs");

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
    let mut invalid_keyed_object_global_ids = Vec::<u32>::new();

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
            continue;
        }

        let Some(animation_index) = current_animation else {
            continue;
        };

        if object.type_name == "KeyedObject" {
            let (object_id, target_local_id) = if let Some((object_id, target_local_id, _target)) =
                keyed_object_target(file, slots, object)
            {
                (object_id, target_local_id)
            } else {
                // C++ imports the object and its following properties before
                // KeyedObject::onAddedDirty reports MissingObject. Retain the
                // same doomed owner as an importer sink until the final
                // LinearAnimation fail-closed erasure below.
                invalid_keyed_object_global_ids.push(global_id as u32);
                (
                    object
                        .uint_property("objectId")
                        .and_then(|id| usize::try_from(id).ok())
                        .unwrap_or(usize::MAX),
                    usize::MAX,
                )
            };

            let keyed_objects = Arc::make_mut(&mut animations[animation_index].keyed_objects);
            keyed_objects.push(RuntimeKeyedObject {
                global_id: global_id as u32,
                object_id,
                target_local_id,
                keyed_properties: Vec::new(),
            });
            current_keyed_object = Some((animation_index, keyed_objects.len() - 1));
            continue;
        }

        if object.type_name == "KeyedProperty" {
            let Some((owner_animation_index, keyed_object_index)) = current_keyed_object else {
                continue;
            };
            let Some(property_key) = object
                .uint_property("propertyKey")
                .and_then(|key| u16::try_from(key).ok())
            else {
                current_keyed_property = None;
                continue;
            };
            let keyed_object = &animations[owner_animation_index].keyed_objects[keyed_object_index];
            let object_id = keyed_object.object_id;
            let target_local_id = keyed_object.target_local_id;
            let invalid_owner = invalid_keyed_object_global_ids.contains(&keyed_object.global_id);
            let target = if invalid_owner {
                // keyed_property.cpp:186 replaces the property importer even
                // when keyed_object.cpp:18 will later reject its owner. This
                // placeholder is never observable: the complete keyed object,
                // including all frames attached through this cursor, is erased.
                RuntimeKeyedPropertyTarget::Bool
            } else {
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
                let Some(target) = keyed_property_target(target_local_id, target, property_key)
                else {
                    current_keyed_property = None;
                    continue;
                };
                target
            };

            let keyed_objects = Arc::make_mut(&mut animations[owner_animation_index].keyed_objects);
            keyed_objects[keyed_object_index]
                .keyed_properties
                .push(RuntimeKeyedProperty {
                    global_id: global_id as u32,
                    property_key,
                    target,
                    key_frames: Vec::new(),
                });
            current_keyed_property = Some((
                owner_animation_index,
                keyed_object_index,
                keyed_objects[keyed_object_index].keyed_properties.len() - 1,
                animation_index,
            ));
            continue;
        }

        if matches!(
            object.type_name,
            "KeyFrameDouble"
                | "KeyFrameColor"
                | "KeyFrameBool"
                | "KeyFrameUint"
                | "KeyFrameInt"
                | "KeyFrameId"
                | "KeyFrameString"
        ) && normalized_interpolator_id(object).is_some()
            && !key_frame_interpolator_id_resolves_to_expected_type(file, artboard_index, object)
        {
            if let Some((owner_animation_index, keyed_object_index, _, _)) = current_keyed_property
            {
                invalid_keyed_object_global_ids.push(
                    animations[owner_animation_index].keyed_objects[keyed_object_index].global_id,
                );
            }
        }

        if object.type_name == "KeyFrameDouble" {
            let Some((
                owner_animation_index,
                keyed_object_index,
                keyed_property_index,
                fps_animation_index,
            )) = current_keyed_property
            else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[fps_animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                owner_animation_index,
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
            let Some((
                owner_animation_index,
                keyed_object_index,
                keyed_property_index,
                fps_animation_index,
            )) = current_keyed_property
            else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[fps_animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                owner_animation_index,
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
            let Some((
                owner_animation_index,
                keyed_object_index,
                keyed_property_index,
                fps_animation_index,
            )) = current_keyed_property
            else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[fps_animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                owner_animation_index,
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
            let Some((
                owner_animation_index,
                keyed_object_index,
                keyed_property_index,
                fps_animation_index,
            )) = current_keyed_property
            else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[fps_animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                owner_animation_index,
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

        if object.type_name == "KeyFrameInt" {
            let Some((
                owner_animation_index,
                keyed_object_index,
                keyed_property_index,
                fps_animation_index,
            )) = current_keyed_property
            else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[fps_animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                owner_animation_index,
                keyed_object_index,
                keyed_property_index,
            )
            .key_frames
            .push(RuntimeKeyFrame::Int(RuntimeKeyFrameInt {
                global_id: global_id as u32,
                frame,
                seconds,
                interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                interpolator_id: normalized_interpolator_id(object),
                value: object.int_property("value").unwrap_or(0),
            }));
        }

        if object.type_name == "KeyFrameId" {
            let Some((
                owner_animation_index,
                keyed_object_index,
                keyed_property_index,
                fps_animation_index,
            )) = current_keyed_property
            else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[fps_animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                owner_animation_index,
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
            let Some((
                owner_animation_index,
                keyed_object_index,
                keyed_property_index,
                fps_animation_index,
            )) = current_keyed_property
            else {
                continue;
            };
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[fps_animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                owner_animation_index,
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
            let Some((
                owner_animation_index,
                keyed_object_index,
                keyed_property_index,
                fps_animation_index,
            )) = current_keyed_property
            else {
                continue;
            };
            animations[owner_animation_index].has_keyed_callbacks = true;
            let frame = object.uint_property("frame").unwrap_or(0);
            let seconds = retained_key_frame_seconds(frame, animations[fps_animation_index].fps);
            runtime_keyed_property_mut(
                &mut animations,
                owner_animation_index,
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

    for animation in &mut animations {
        Arc::make_mut(&mut animation.keyed_objects).retain(|keyed_object| {
            !invalid_keyed_object_global_ids.contains(&keyed_object.global_id)
        });
    }

    let templates = build_key_frame_data_bind_templates(file, artboard_index, converter_cache);
    if !templates.is_empty() {
        for animation in &mut animations {
            animation.key_frame_data_bind_templates = Arc::new(
                key_frame_data_bind_templates_in_animation_order(animation, &templates),
            );
        }
    }

    animations
}

/// C++ first selects the first authored DataBind for each target, then walks
/// animation → keyed object → keyed property → keyframe when it clones and
/// enrolls those binds. The shared template catalog retains the first-source
/// decision; this projection restores the separate keyframe traversal order.
fn key_frame_data_bind_templates_in_animation_order(
    animation: &RuntimeLinearAnimation,
    templates: &[RuntimeKeyFrameDataBindTemplate],
) -> Vec<RuntimeKeyFrameDataBindTemplate> {
    let template_by_key_frame = templates
        .iter()
        .map(|template| (template.key_frame_global_id, template))
        .collect::<HashMap<_, _>>();
    animation
        .keyed_objects
        .iter()
        .flat_map(|object| &object.keyed_properties)
        .flat_map(|property| &property.key_frames)
        .filter_map(RuntimeKeyFrame::bindable_global_id)
        .filter_map(|global_id| template_by_key_frame.get(&global_id).copied())
        .cloned()
        .collect()
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

fn key_frame_interpolator_id_resolves_to_expected_type(
    file: &RuntimeFile,
    artboard_index: usize,
    key_frame: &RuntimeObject,
) -> bool {
    let Some(local_index) =
        normalized_interpolator_id(key_frame).and_then(|id| usize::try_from(id).ok())
    else {
        return false;
    };
    file.artboard_local_object(artboard_index, local_index)
        .and_then(|interpolator| definition_by_type_key(interpolator.type_key))
        .is_some_and(|definition| definition.is_a("KeyFrameInterpolator"))
}

include!("animation/linear_animation.rs");

include!("animation/keyed_object.rs");

include!("animation/keyed_property.rs");

include!("animation/keyframe_double.rs");
include!("animation/keyframe_color.rs");
include!("animation/keyframe_bool.rs");
include!("animation/keyframe_uint.rs");
include!("animation/keyframe_int.rs");
include!("animation/keyframe_string.rs");
include!("animation/keyframe_callback.rs");

include!("animation/linear_animation_instance.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RuntimeDataBindGraphValue;
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

    fn key_frame_binding(
        data_bind_index: usize,
        key_frame_global_id: u32,
        target: RuntimeKeyFrameDataBindTarget,
        default_value: RuntimeDataBindGraphValue,
    ) -> RuntimeKeyFrameDataBindTemplate {
        RuntimeKeyFrameDataBindTemplate {
            data_bind_index,
            key_frame_global_id,
            target,
            path: vec![0, data_bind_index as u32],
            flags: 0,
            converter: None,
            default_value,
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

    fn upstream_test_animation(
        speed: f32,
        loop_value: u64,
        enable_work_area: bool,
    ) -> RuntimeLinearAnimation {
        RuntimeLinearAnimation {
            global_id: 2,
            name: Some(Arc::<str>::from("upstream unit-test animation")),
            fps: 2,
            duration: if enable_work_area { 100 } else { 10 },
            speed,
            loop_value,
            work_start: if enable_work_area { 4 } else { 0 },
            work_end: if enable_work_area { 10 } else { 0 },
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
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(7),
            &animation,
            1.0,
        );

        assert_eq!(instance.animation_index(), 7);
        assert_eq!(instance.loop_value_override, -1);
        assert_eq!(instance.loop_value(), 1);
        assert_eq!(instance.resolved_loop_kind(&animation), AnimationLoop::Loop);

        // linear_animation_instance.cpp:426-434 leaves the sentinel untouched
        // when the requested value already equals the definition.
        instance.set_loop_value(animation.loop_value as i32);
        assert_eq!(instance.loop_value_override, -1);

        instance.set_loop_value(2);
        assert_eq!(instance.loop_value_override, 2);
        assert_eq!(instance.loop_value(), 2);
        assert_eq!(
            instance.resolved_loop_kind(&animation),
            AnimationLoop::PingPong
        );

        instance.set_loop_value(-1);
        assert_eq!(instance.loop_value_override, -1);
        assert_eq!(instance.loop_value(), 1);
        assert_eq!(instance.resolved_loop_kind(&animation), AnimationLoop::Loop);

        instance.set_loop_value(-2);
        assert_eq!(instance.loop_value_override, -2);
        assert_eq!(instance.loop_value(), -2);
    }

    #[test]
    fn pre_advance_did_loop_is_safe_false_then_every_advance_writes_it() {
        let animation = animation_with_work_area(false);
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );

        // Binding adaptation: pinned C++ leaves m_didLoop indeterminate until
        // advance; safe Rust exposes a deterministic false.
        assert!(!instance.did_loop());
        assert!(instance.advance(2.0));
        assert!(instance.did_loop());
        assert!(!instance.advance(0.0));
        assert!(!instance.did_loop());
    }

    #[test]
    fn time_and_reset_follow_cpp_occurrence_state_rules() {
        let animation = animation_with_work_area(true);
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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
    fn upstream_animation_state_speed_start_and_spilled_time_matrix() {
        // Assertion-for-assertion port of animation_state_instance_test.cpp.
        for (label, animation_speed, state_speed, loop_value, elapsed, expected) in [
            ("speed 1", 1.0, 1.0, 0, 2.0, (2.0, 2.0, 0.0)),
            ("state speed 2", 1.0, 2.0, 0, 2.0, (4.0, 4.0, 0.0)),
            ("state speed 0.5", 1.0, 0.5, 0, 2.0, (1.0, 1.0, 0.0)),
            ("negative state speed", 1.0, -1.0, 1, 2.0, (3.0, 2.0, 0.0)),
        ] {
            let animation = upstream_test_animation(animation_speed, loop_value, false);
            let mut instance = LinearAnimationInstance::new_for_test(
                RuntimeLinearAnimationHandle::new(0),
                &animation,
                state_speed,
            );
            let _ = instance.advance(elapsed * state_speed);
            assert_eq!(instance.time(), expected.0, "{label} time");
            assert_eq!(instance.total_time(), expected.1, "{label} totalTime");
            assert_eq!(instance.spilled_time(), expected.2, "{label} spilledTime");
        }

        for (label, animation_speed, state_speed, expected_time) in [
            ("positive animation, positive state", 1.0, 1.0, 0.0),
            ("negative animation, positive state", -1.0, 1.0, 5.0),
            ("positive animation, negative state", 1.0, -1.0, 5.0),
            ("negative animation, negative state", -1.0, -1.0, 0.0),
        ] {
            let animation = upstream_test_animation(animation_speed, 0, false);
            let instance = LinearAnimationInstance::new_for_test(
                RuntimeLinearAnimationHandle::new(0),
                &animation,
                state_speed,
            );
            assert_eq!(instance.time(), expected_time, "{label} initial time");
        }

        for (label, animation_speed, loop_value, elapsed, expected) in [
            ("2x one-shot", 2.0, 0, 3.0, (2.0, 6.0, 2.0)),
            ("0.5x one-shot", 0.5, 0, 5.0, (2.0, 2.5, 1.0)),
            ("2x loop", 2.0, 1, 5.5, (1.0, 11.0, 0.5)),
            ("0.5x loop", 0.5, 1, 10.0, (1.0, 5.0, 2.0)),
            ("-2x one-shot", -2.0, 0, 3.0, (0.0, 6.0, 2.0)),
            ("-2x loop", -2.0, 1, 5.5, (1.0, 11.0, 0.5)),
        ] {
            let mut animation = upstream_test_animation(animation_speed, loop_value, false);
            animation.duration = 4;
            let mut instance = LinearAnimationInstance::new_for_test(
                RuntimeLinearAnimationHandle::new(0),
                &animation,
                1.0,
            );
            let _ = instance.advance(elapsed);
            assert_eq!(instance.time(), expected.0, "{label} time");
            assert_eq!(instance.total_time(), expected.1, "{label} totalTime");
            assert_eq!(instance.spilled_time(), expected.2, "{label} spilledTime");
        }
    }

    #[test]
    fn upstream_linear_animation_definition_timing_and_keep_going() {
        // Literal ports of the definition-only and work-area keep-going cases
        // in linear_animation_test.cpp.
        for (speed, expected_start_time, expected_end_time) in [(1.0, 0.0, 5.0), (-1.0, 5.0, 0.0)] {
            let animation = upstream_test_animation(speed, 0, false);
            assert_eq!(animation.start_seconds(), 0.0);
            assert_eq!(animation.end_seconds(), 5.0);
            assert_eq!(animation.start_time_with_speed(1.0), expected_start_time);
            assert_eq!(animation.start_time_with_speed(-1.0), expected_end_time);
            assert_eq!(animation.duration_seconds(), 5.0);
        }

        let animation = RuntimeLinearAnimation {
            global_id: 3,
            name: Some(Arc::<str>::from("upstream work-area animation")),
            fps: 60,
            duration: 60,
            speed: 1.0,
            loop_value: 0,
            work_start: 30,
            work_end: 42,
            enable_work_area: true,
            quantize: false,
            keyed_objects: Arc::new(Vec::new()),
            key_frame_data_bind_templates: Arc::new(Vec::new()),
            has_keyed_callbacks: false,
        };
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        assert!(!instance.advance(0.0));
        assert_eq!(instance.time(), 0.5);
        assert!(instance.advance(0.1));
        assert_eq!(instance.time(), 0.6);
        assert!(!instance.advance(0.2));
        assert_eq!(instance.time(), 0.7);
    }

    #[test]
    fn upstream_linear_animation_instance_sequences() {
        // Assertion-for-assertion port of linear_animation_instance_test.cpp.
        let animation = upstream_test_animation(1.0, 0, false);
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        assert!(instance.advance(2.0));
        assert_eq!(instance.time(), 2.0);
        assert_eq!(instance.total_time(), 2.0);
        assert!(!instance.did_loop());
        assert!(!instance.advance(10.0));
        assert_eq!(instance.time(), 5.0);
        assert_eq!(instance.total_time(), 12.0);
        assert!(instance.did_loop());

        let animation = upstream_test_animation(0.5, 0, false);
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        assert!(instance.advance(2.0));
        assert_eq!(instance.time(), 1.0);
        assert_eq!(instance.total_time(), 1.0);

        let animation = upstream_test_animation(1.0, 1, false);
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        assert!(instance.advance(-2.0));
        assert_eq!(instance.time(), 3.0);
        assert_eq!(instance.total_time(), 2.0);
        assert!(instance.did_loop());

        let animation = upstream_test_animation(1.0, 0, false);
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        instance.set_direction(-1);
        assert_eq!(instance.time(), 0.0);
        assert!(!instance.advance(2.0));
        assert_eq!(instance.time(), 0.0);
        assert_eq!(instance.total_time(), 2.0);
        assert!(instance.did_loop());
        instance.set_time(&animation, 5.0);
        assert_eq!(instance.total_time(), 5.0);
        instance.set_direction(-1);
        assert!(instance.advance(2.0));
        assert_eq!(instance.time(), 3.0);
        assert_eq!(instance.total_time(), 7.0);
        assert!(!instance.did_loop());
        assert!(!instance.advance(4.0));
        assert_eq!(instance.time(), 0.0);
        assert_eq!(instance.total_time(), 11.0);
        assert!(instance.did_loop());

        let animation = upstream_test_animation(1.0, 1, false);
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        assert!(instance.advance(2.0));
        assert_eq!(instance.time(), 2.0);
        assert_eq!(instance.total_time(), 2.0);
        assert!(!instance.did_loop());
        assert!(instance.advance(10.0));
        assert_eq!(instance.time(), 2.0);
        assert_eq!(instance.total_time(), 12.0);
        assert!(instance.did_loop());

        let mut reverse_loop = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        reverse_loop.set_direction(-1);
        assert_eq!(reverse_loop.time(), 0.0);
        for (elapsed, expected_time, expected_total, expected_looped) in [
            (2.0, 3.0, 2.0, true),
            (2.0, 1.0, 4.0, false),
            (4.0, 2.0, 8.0, true),
        ] {
            assert!(reverse_loop.advance(elapsed));
            assert_eq!(reverse_loop.direction(), -1.0);
            assert_eq!(reverse_loop.time(), expected_time);
            assert_eq!(reverse_loop.total_time(), expected_total);
            assert_eq!(reverse_loop.did_loop(), expected_looped);
        }

        let work_area = upstream_test_animation(1.0, 1, true);
        let mut reverse_work_area = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &work_area,
            1.0,
        );
        reverse_work_area.set_direction(-1);
        assert_eq!(reverse_work_area.time(), 2.0);
        assert!(!reverse_work_area.advance(0.0));
        assert_eq!(reverse_work_area.direction(), -1.0);
        assert_eq!(reverse_work_area.time(), 2.0);
        assert_eq!(reverse_work_area.total_time(), 0.0);
        assert!(!reverse_work_area.did_loop());
        for (expected_time, expected_total) in [(3.0, 2.0), (4.0, 4.0), (5.0, 6.0)] {
            assert!(reverse_work_area.advance(2.0));
            assert_eq!(reverse_work_area.direction(), -1.0);
            assert_eq!(reverse_work_area.time(), expected_time);
            assert_eq!(reverse_work_area.total_time(), expected_total);
            assert!(reverse_work_area.did_loop());
        }

        let ping_pong = upstream_test_animation(1.0, 2, false);
        let mut forward_ping_pong = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &ping_pong,
            1.0,
        );
        for (elapsed, expected_time, expected_total, expected_direction, expected_looped) in [
            (2.0, 2.0, 2.0, 1.0, false),
            (5.0, 3.0, 7.0, -1.0, true),
            (9.0, 4.0, 16.0, -1.0, true),
            (6.0, 2.0, 22.0, 1.0, true),
            (20.0, 2.0, 42.0, 1.0, true),
        ] {
            assert!(forward_ping_pong.advance(elapsed));
            assert_eq!(forward_ping_pong.time(), expected_time);
            assert_eq!(forward_ping_pong.total_time(), expected_total);
            assert_eq!(forward_ping_pong.direction(), expected_direction);
            assert_eq!(forward_ping_pong.did_loop(), expected_looped);
        }

        let mut reverse_ping_pong = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &ping_pong,
            1.0,
        );
        reverse_ping_pong.set_direction(-1);
        assert_eq!(reverse_ping_pong.time(), 0.0);
        for (elapsed, expected_time, expected_total, expected_direction, expected_looped) in [
            (2.0, 2.0, 2.0, 1.0, true),
            (4.0, 4.0, 6.0, -1.0, true),
            (2.0, 2.0, 8.0, -1.0, false),
        ] {
            assert!(reverse_ping_pong.advance(elapsed));
            assert_eq!(reverse_ping_pong.time(), expected_time);
            assert_eq!(reverse_ping_pong.total_time(), expected_total);
            assert_eq!(reverse_ping_pong.direction(), expected_direction);
            assert_eq!(reverse_ping_pong.did_loop(), expected_looped);
        }
    }

    #[test]
    fn upstream_elastic_ease_numeric_contract() {
        // Literal numeric assertions from elastic_easing_test.cpp.
        let amplitude = 0.5;
        let period = 3.14;
        let shift = period / 4.0;
        assert_eq!(elastic_actual_amplitude(0.0, amplitude, shift), 1.0);
        assert_eq!(elastic_actual_amplitude(1.57, amplitude, shift), 0.5);
        assert!((elastic_ease_out(0.22, amplitude, period, shift) - 0.8307).abs() <= 0.0001);
        assert!((elastic_ease_in(1.58, amplitude, period, shift) - 14.01086).abs() <= 0.0001);
        assert!((elastic_ease_in_out(1.58, amplitude, period, shift) - 1.0).abs() <= 0.0001);
    }

    #[test]
    fn key_frame_value_holders_are_isolated_per_animation_instance() {
        let animation = animation_with_work_area(false);
        let property = keyed_double_property(10, 1.0, 20, 2.0);
        let mut first = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        let mut second = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        let unbound = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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

        let mut state_machine_instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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

        let mut standalone_clone = state_machine_instance.clone();
        assert!(standalone_clone.key_frame_data_bind_graphs.is_empty());
        assert!(standalone_clone.key_frame_value_holders.is_none());
        assert!(standalone_clone.prepare_key_frame_data_binds(Some(&prototype)));
        assert_eq!(
            standalone_clone.key_frame_value_holder(10),
            Some(&RuntimeKeyFrameValue::Number(20.0)),
            "the snapshot rebuilds a fresh occurrence from the immutable prototype"
        );
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

        let mut first = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        let mut second = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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
                .key_frame_data_bind_graphs
                .first()
                .map(|graph| graph as *const RuntimeDataBindGraph),
            second
                .key_frame_data_bind_graphs
                .first()
                .map(|graph| graph as *const RuntimeDataBindGraph)
        );
        assert_eq!(
            first
                .key_frame_data_bind_graphs
                .first()
                .map(RuntimeDataBindGraph::formula_random_call_count),
            Some(1)
        );
        assert_eq!(
            second
                .key_frame_data_bind_graphs
                .first()
                .map(RuntimeDataBindGraph::formula_random_call_count),
            Some(1)
        );
    }

    #[test]
    fn fl_c5_keyframe_data_bind_supported_holders_and_live_resolution() {
        let animation = animation_with_work_area(false);
        let templates = [
            key_frame_binding(
                0,
                10,
                RuntimeKeyFrameDataBindTarget::Number,
                RuntimeDataBindGraphValue::Number(12.5),
            ),
            key_frame_binding(
                1,
                20,
                RuntimeKeyFrameDataBindTarget::Color,
                RuntimeDataBindGraphValue::Color(0xFF12_3456),
            ),
            key_frame_binding(
                2,
                30,
                RuntimeKeyFrameDataBindTarget::Boolean,
                RuntimeDataBindGraphValue::Boolean(true),
            ),
            key_frame_binding(
                3,
                40,
                RuntimeKeyFrameDataBindTarget::String,
                RuntimeDataBindGraphValue::String(b"bound".to_vec()),
            ),
        ];
        let mut traversal_animation = animation_with_work_area(false);
        traversal_animation.keyed_objects = Arc::new(vec![RuntimeKeyedObject {
            global_id: 1,
            object_id: 0,
            target_local_id: 0,
            keyed_properties: vec![keyed_double_property(10, 1.0, 40, 2.0)],
        }]);
        let reversed_templates = [
            templates[3].clone(),
            templates[0].clone(),
            templates[1].clone(),
            templates[2].clone(),
        ];
        assert_eq!(
            key_frame_data_bind_templates_in_animation_order(
                &traversal_animation,
                &reversed_templates,
            )
            .iter()
            .map(|template| template.key_frame_global_id)
            .collect::<Vec<_>>(),
            [10, 40],
            "keyframe traversal, not authored DataBind order, owns clone/enrollment order"
        );
        let mut prototype =
            RuntimeDataBindGraph::new_key_frame_bindings(&templates).expect("four holders");
        assert_eq!(
            prototype
                .targets
                .iter()
                .filter_map(|target| match target.target {
                    RuntimeDataBindGraphTarget::KeyFrameNumber { global_id }
                    | RuntimeDataBindGraphTarget::KeyFrameColor { global_id }
                    | RuntimeDataBindGraphTarget::KeyFrameBoolean { global_id }
                    | RuntimeDataBindGraphTarget::KeyFrameString { global_id } => Some(global_id),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [10, 20, 30, 40],
            "keyframe traversal/enrollment stays in authored template order"
        );
        assert!(prototype.bind_default_view_model_context());

        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        assert!(instance.build_key_frame_data_binds(
            &prototype,
            RuntimeKeyFrameDataBindEnrollment::Initial,
        ));
        assert_eq!(
            instance.key_frame_value_holder(10),
            Some(&RuntimeKeyFrameValue::Number(12.5))
        );
        assert_eq!(
            instance.key_frame_value_holder(20),
            Some(&RuntimeKeyFrameValue::Color(0xFF12_3456))
        );
        assert_eq!(
            instance.key_frame_value_holder(30),
            Some(&RuntimeKeyFrameValue::Boolean(true))
        );
        assert_eq!(
            instance.key_frame_value_holder(40),
            Some(&RuntimeKeyFrameValue::String(b"bound".to_vec()))
        );

        // The typed target enum has exactly these four variants. Uint/ID and
        // a null keyframe have no representable template/holder in Rust; the
        // former continues to sample its serialized value.
        let mut uint_property = keyed_double_property(50, 1.0, 60, 2.0);
        uint_property.target = RuntimeKeyedPropertyTarget::Uint;
        uint_property.key_frames = vec![RuntimeKeyFrame::Uint(RuntimeKeyFrameUint {
            global_id: 50,
            frame: 0,
            seconds: 0.0,
            interpolation_type: 0,
            interpolator_id: None,
            value: 7,
        })];
        assert_eq!(uint_property.uint_value_at(0.0), Some(7));
    }

    #[test]
    fn fl_c5_keyframe_data_bind_duplicate_build_tracks_and_removes_in_build_order() {
        let animation = animation_with_work_area(false);
        let mut prototype =
            RuntimeDataBindGraph::new_key_frame_bindings(&[number_key_frame_binding(None)])
                .expect("keyframe graph");
        assert!(prototype.bind_default_view_model_context());
        assert!(prototype.set_default_view_model_number_source_for_path(&[0, 0], 10.0));

        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        assert!(instance.build_key_frame_data_binds(
            &prototype,
            RuntimeKeyFrameDataBindEnrollment::Initial,
        ));
        assert_eq!(instance.key_frame_data_bind_graphs.len(), 1);

        assert!(prototype.set_default_view_model_number_source_for_path(&[0, 0], 20.0));
        assert!(
            instance
                .build_key_frame_data_binds(&prototype, RuntimeKeyFrameDataBindEnrollment::Late,)
        );
        assert_eq!(instance.key_frame_data_bind_graphs.len(), 2);
        assert!(
            !std::ptr::eq(
                &instance.key_frame_data_bind_graphs[0],
                &instance.key_frame_data_bind_graphs[1],
            ),
            "each build owns a distinct mutable bind graph"
        );
        assert_eq!(
            instance.key_frame_value_holder(10),
            Some(&RuntimeKeyFrameValue::Number(20.0)),
            "the later build overwrites the holder lookup while retaining both bind clones"
        );
        let mut next_occurrence_id = 0;
        instance.enroll_unassigned_key_frame_data_binds(&mut next_occurrence_id);
        let initial_ids = instance
            .key_frame_data_bind_occurrence_ids(RuntimeKeyFrameDataBindEnrollment::Initial)
            .collect::<Vec<_>>();
        let late_ids = instance
            .key_frame_data_bind_occurrence_ids(RuntimeKeyFrameDataBindEnrollment::Late)
            .collect::<Vec<_>>();
        assert_eq!(initial_ids.len(), 1);
        assert_eq!(late_ids.len(), 1);
        assert!(
            initial_ids[0] < late_ids[0],
            "typed occurrence ids retain cross-family enrollment chronology"
        );

        let expected_removal_order = instance
            .key_frame_data_bind_occurrences
            .iter()
            .filter_map(|(id, _)| *id)
            .collect::<Vec<_>>();
        instance.remove_key_frame_data_binds();
        assert!(instance.key_frame_data_bind_graphs.is_empty());
        assert!(instance.key_frame_value_holders.is_none());
        assert_eq!(
            instance.removed_key_frame_data_bind_occurrences, expected_removal_order,
            "each tracked graph is removed in its exact enrollment/build order"
        );
        instance.remove_key_frame_data_binds();
        assert!(instance.key_frame_data_bind_graphs.is_empty());
    }

    #[test]
    fn fl_c5_keyframe_data_bind_converter_advancement_keeps_going_per_occurrence() {
        let animation = animation_with_work_area(false);
        let converter = RuntimeDataBindGraphConverter::Interpolator {
            global_id: 70,
            duration: 1.0,
            interpolator: None,
        };
        let mut prototype =
            RuntimeDataBindGraph::new_key_frame_bindings(&[number_key_frame_binding(Some(
                converter,
            ))])
            .expect("stateful keyframe graph");
        assert!(prototype.bind_default_view_model_context());
        assert!(prototype.set_default_view_model_number_source_for_path(&[0, 0], 10.0));

        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        instance.build_key_frame_data_binds(&prototype, RuntimeKeyFrameDataBindEnrollment::Initial);
        // Pinned interpolators snap their first two converter advances before
        // later source changes begin smoothing.
        instance.advance_key_frame_data_binds(Some(&prototype), 0.25);
        instance.advance_key_frame_data_binds(Some(&prototype), 0.25);
        assert!(prototype.set_default_view_model_number_source_for_path(&[0, 0], 20.0));
        instance.prepare_key_frame_data_binds(Some(&prototype));
        let before_step = instance.key_frame_value_holder(10).cloned();
        assert!(
            instance.advance_key_frame_data_binds(Some(&prototype), 0.25),
            "the enrolled converter requests another frame"
        );
        assert_eq!(
            instance.key_frame_value_holder(10),
            before_step.as_ref(),
            "post-layer converter dirt must not update the holder in the same frame"
        );
        assert!(instance.prepare_key_frame_data_binds(Some(&prototype)));
        assert_ne!(
            instance.key_frame_value_holder(10),
            before_step.as_ref(),
            "the next normal pre-layer update consumes converter dirt"
        );
    }

    #[test]
    fn fl_c5_keyframe_data_bind_reentrant_removal_is_borrow_isolated_and_active_drop_is_safe() {
        let animation = animation_with_work_area(false);
        let prototype =
            RuntimeDataBindGraph::new_key_frame_bindings(&[number_key_frame_binding(None)])
                .expect("keyframe graph");
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
        instance.build_key_frame_data_binds(&prototype, RuntimeKeyFrameDataBindEnrollment::Initial);

        let occurrence = std::cell::RefCell::new(instance);
        let processing = occurrence.borrow_mut();
        assert!(
            occurrence
                .try_borrow_mut()
                .map(|mut instance| instance.remove_key_frame_data_binds())
                .is_err(),
            "Rust cannot alias a reentrant removal with the active graph update borrow"
        );
        drop(processing);
        drop(occurrence);
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
        let mut instance = LinearAnimationInstance::new_for_test(
            RuntimeLinearAnimationHandle::new(0),
            &animation,
            1.0,
        );
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
