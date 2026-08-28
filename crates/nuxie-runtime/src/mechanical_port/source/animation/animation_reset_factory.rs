use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
};

use crate::mechanical_port::source::{
    animation::{
        animation_reset::AnimationReset, keyed_object::KeyedObject, keyed_property::KeyedProperty,
        keyframe_color::KeyFrameColor, keyframe_double::KeyFrameDouble,
        linear_animation::LinearAnimation,
    },
    core::CoreHandle,
    generated::core_registry::CoreRegistry,
};

const CORE_DOUBLE_TYPE_ID: i32 = 2;
const CORE_COLOR_TYPE_ID: i32 = 3;

pub trait ResetKeyedProperty {
    fn property_key(&self) -> u32;
    fn field_id(&self) -> i32;
    fn first_value(&self) -> Option<f32>;
}

pub trait ResetKeyedObject {
    fn object_id(&self) -> u32;
    fn keyed_properties(&self) -> Vec<&dyn ResetKeyedProperty>;
}

pub trait ResetLinearAnimation {
    fn keyed_objects(&self) -> Vec<&dyn ResetKeyedObject>;
}

pub trait ResetStateInstance {
    fn animation_state_animation(&self) -> Option<&dyn ResetLinearAnimation>;
}

pub trait ResetArtboard {
    fn resolves(&self, object_id: u32) -> bool;
    fn double_value(&self, object_id: u32, property_key: u32) -> f32;
    fn color_value(&self, object_id: u32, property_key: u32) -> u32;
}

#[derive(Clone, Copy)]
struct KeyedPropertyData<'a> {
    keyed_property: &'a dyn ResetKeyedProperty,
    is_baseline: bool,
}

struct KeyedObjectData<'a> {
    keyed_properties_data: Vec<KeyedPropertyData<'a>>,
    keyed_properties_set: HashSet<u32>,
    object_id: u32,
}

impl<'a> KeyedObjectData<'a> {
    fn new(object_id: u32) -> Self {
        Self {
            keyed_properties_data: Vec::new(),
            keyed_properties_set: HashSet::new(),
            object_id,
        }
    }

    fn add_properties(&mut self, keyed_object: &'a dyn ResetKeyedObject, is_baseline: bool) {
        for keyed_property in keyed_object.keyed_properties() {
            let property_key = keyed_property.property_key();
            if self.keyed_properties_set.contains(&property_key) {
                continue;
            }
            match keyed_property.field_id() {
                CORE_DOUBLE_TYPE_ID | CORE_COLOR_TYPE_ID => {
                    self.keyed_properties_set.insert(property_key);
                    self.keyed_properties_data.push(KeyedPropertyData {
                        keyed_property,
                        is_baseline,
                    });
                }
                _ => {}
            }
        }
    }
}

struct AnimationsData<'a> {
    keyed_objects_data: Vec<KeyedObjectData<'a>>,
}

impl<'a> AnimationsData<'a> {
    fn new(animations: &[&'a dyn ResetLinearAnimation], use_first_as_baseline: bool) -> Self {
        let mut data = Self {
            keyed_objects_data: Vec::new(),
        };
        let mut is_first_animation = use_first_as_baseline;
        for animation in animations {
            data.find_keyed_objects(*animation, is_first_animation);
            is_first_animation = false;
        }
        data
    }

    fn keyed_object_data(&mut self, object_id: u32) -> &mut KeyedObjectData<'a> {
        if let Some(index) = self
            .keyed_objects_data
            .iter()
            .position(|data| data.object_id == object_id)
        {
            return &mut self.keyed_objects_data[index];
        }
        self.keyed_objects_data
            .push(KeyedObjectData::new(object_id));
        self.keyed_objects_data.last_mut().unwrap()
    }

    fn find_keyed_objects(
        &mut self,
        animation: &'a dyn ResetLinearAnimation,
        is_first_animation: bool,
    ) {
        for keyed_object in animation.keyed_objects() {
            self.keyed_object_data(keyed_object.object_id())
                .add_properties(keyed_object, is_first_animation);
        }
    }

    fn write_objects(&self, animation_reset: &mut AnimationReset, artboard: &dyn ResetArtboard) {
        for keyed_object_data in &self.keyed_objects_data {
            if !artboard.resolves(keyed_object_data.object_id) {
                continue;
            }
            if keyed_object_data.keyed_properties_data.is_empty() {
                continue;
            }

            animation_reset.write_object_id(keyed_object_data.object_id);
            animation_reset
                .write_total_properties(keyed_object_data.keyed_properties_data.len() as u32);
            for property_data in &keyed_object_data.keyed_properties_data {
                let property = property_data.keyed_property;
                let property_key = property.property_key();
                match property.field_id() {
                    CORE_DOUBLE_TYPE_ID => {
                        animation_reset.write_property_key(property_key);
                        if property_data.is_baseline {
                            if let Some(value) = property.first_value() {
                                animation_reset.write_property_value(value);
                            }
                        } else {
                            animation_reset.write_property_value(
                                artboard.double_value(keyed_object_data.object_id, property_key),
                            );
                        }
                    }
                    CORE_COLOR_TYPE_ID => {
                        animation_reset.write_property_key(property_key);
                        if property_data.is_baseline {
                            if let Some(value) = property.first_value() {
                                animation_reset.write_property_value(value);
                            }
                        } else {
                            animation_reset.write_property_value(
                                artboard.color_value(keyed_object_data.object_id, property_key)
                                    as f32,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        animation_reset.complete();
    }
}

fn resources() -> &'static Mutex<Vec<AnimationReset>> {
    static RESOURCES: OnceLock<Mutex<Vec<AnimationReset>>> = OnceLock::new();
    RESOURCES.get_or_init(|| Mutex::new(Vec::new()))
}

pub struct AnimationResetFactory;

impl AnimationResetFactory {
    pub fn get_instance() -> AnimationReset {
        resources().lock().unwrap().pop().unwrap_or_default()
    }

    fn from_state<'a>(
        state_instance: Option<&'a dyn ResetStateInstance>,
        animations: &mut Vec<&'a dyn ResetLinearAnimation>,
    ) {
        if let Some(animation) = state_instance.and_then(|state| state.animation_state_animation())
        {
            animations.push(animation);
        }
    }

    pub fn from_states<'a>(
        state_from: Option<&'a dyn ResetStateInstance>,
        current_state: Option<&'a dyn ResetStateInstance>,
        artboard: &dyn ResetArtboard,
    ) -> AnimationReset {
        let mut animations = Vec::new();
        Self::from_state(state_from, &mut animations);
        Self::from_state(current_state, &mut animations);
        Self::from_animations(&animations, artboard, false)
    }

    pub fn from_animations(
        animations: &[&dyn ResetLinearAnimation],
        artboard: &dyn ResetArtboard,
        use_first_as_baseline: bool,
    ) -> AnimationReset {
        let animations_data = AnimationsData::new(animations, use_first_as_baseline);
        let mut animation_reset = Self::get_instance();
        animations_data.write_objects(&mut animation_reset, artboard);
        animation_reset
    }

    pub fn from_animation_handles(
        animations: &[CoreHandle],
        artboard: &dyn ResetArtboard,
        use_first_as_baseline: bool,
    ) -> AnimationReset {
        let mut properties: Vec<(u32, CoreHandle, bool)> = Vec::new();
        let mut seen = HashSet::new();
        for (animation_index, animation) in animations.iter().enumerate() {
            let keyed_objects = animation
                .with_downcast::<LinearAnimation, _>(|animation| {
                    (0..animation.num_keyed_objects())
                        .filter_map(|index| animation.get_object(index))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for keyed_object in keyed_objects {
                let Some((object_id, keyed_properties)) = keyed_object
                    .with_downcast::<KeyedObject, _>(|object| {
                        (
                            object.base.object_id(),
                            (0..object.num_keyed_properties())
                                .filter_map(|index| object.get_property(index))
                                .collect::<Vec<_>>(),
                        )
                    })
                else {
                    continue;
                };
                for property in keyed_properties {
                    let Some(property_key) = property
                        .with_downcast::<KeyedProperty, _>(|property| property.base.property_key())
                    else {
                        continue;
                    };
                    if !seen.insert((object_id, property_key)) {
                        continue;
                    }
                    match CoreRegistry::property_field_id(property_key as i32) {
                        CORE_DOUBLE_TYPE_ID | CORE_COLOR_TYPE_ID => properties.push((
                            object_id,
                            property,
                            use_first_as_baseline && animation_index == 0,
                        )),
                        _ => {}
                    }
                }
            }
        }

        let mut grouped: Vec<(u32, Vec<(CoreHandle, bool)>)> = Vec::new();
        for (object_id, property, baseline) in properties {
            if let Some((_, entries)) = grouped.iter_mut().find(|(id, _)| *id == object_id) {
                entries.push((property, baseline));
            } else {
                grouped.push((object_id, vec![(property, baseline)]));
            }
        }

        let mut animation_reset = Self::get_instance();
        for (object_id, entries) in grouped {
            if !artboard.resolves(object_id) || entries.is_empty() {
                continue;
            }
            animation_reset.write_object_id(object_id);
            animation_reset.write_total_properties(entries.len() as u32);
            for (property, baseline) in entries {
                let Some((property_key, first)) =
                    property.with_downcast::<KeyedProperty, _>(|property| {
                        (property.base.property_key(), property.first())
                    })
                else {
                    continue;
                };
                animation_reset.write_property_key(property_key);
                let field_id = CoreRegistry::property_field_id(property_key as i32);
                let baseline_value = baseline
                    .then(|| {
                        first.and_then(|first| {
                            first
                                .with_downcast::<KeyFrameDouble, _>(|frame| frame.base.value())
                                .or_else(|| {
                                    first.with_downcast::<KeyFrameColor, _>(|frame| {
                                        frame.base.value() as f32
                                    })
                                })
                        })
                    })
                    .flatten();
                let value = baseline_value.unwrap_or_else(|| {
                    if field_id == CORE_DOUBLE_TYPE_ID {
                        artboard.double_value(object_id, property_key)
                    } else {
                        artboard.color_value(object_id, property_key) as f32
                    }
                });
                animation_reset.write_property_value(value);
            }
        }
        animation_reset.complete();
        animation_reset
    }

    pub fn release(mut value: AnimationReset) {
        value.clear();
        resources().lock().unwrap().push(value);
    }

    #[cfg(test)]
    pub fn resources_count() -> usize {
        resources().lock().unwrap().len()
    }

    #[cfg(test)]
    pub fn release_resources() {
        resources().lock().unwrap().clear();
    }
}
