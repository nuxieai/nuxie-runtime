use super::*;

impl Drop for AnimationResetStorage {
    fn drop(&mut self) {
        let mut entries = std::mem::take(&mut self.entries);
        entries.clear();
        animation_reset_pool()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(entries);
    }
}


#[derive(Debug)]
struct AnimationResetObjectData {
    local_id: usize,
    property_keys: BTreeSet<u16>,
    entries: Vec<AnimationResetEntry>,
}

impl AnimationResetObjectData {
    fn new(local_id: usize) -> Self {
        Self {
            local_id,
            property_keys: BTreeSet::new(),
            entries: Vec::new(),
        }
    }
}

pub(crate) struct AnimationResetFactory;

fn animation_reset_pool() -> &'static Mutex<Vec<Vec<AnimationResetEntry>>> {
    static POOL: OnceLock<Mutex<Vec<Vec<AnimationResetEntry>>>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(Vec::new()))
}

impl AnimationResetFactory {
    pub(crate) fn from_animation_instances<'a>(
        artboard: &ArtboardInstance,
        animation_instances: impl IntoIterator<Item = &'a LinearAnimationInstance>,
        use_first_as_baseline: bool,
    ) -> AnimationReset {
        let mut entries = animation_reset_pool()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pop()
            .unwrap_or_default();
        debug_assert!(entries.is_empty());
        let mut objects = Vec::<AnimationResetObjectData>::new();

        for (animation_order, animation_instance) in animation_instances.into_iter().enumerate() {
            let Some(animation) = animation_instance.retained_definition() else {
                continue;
            };
            let use_baseline = use_first_as_baseline && animation_order == 0;
            for keyed_object in animation.keyed_objects.iter() {
                let object_index = objects
                    .iter()
                    .position(|object| object.local_id == keyed_object.target_local_id)
                    .unwrap_or_else(|| {
                        objects.push(AnimationResetObjectData::new(keyed_object.target_local_id));
                        objects.len() - 1
                    });
                let object = &mut objects[object_index];
                for keyed_property in &keyed_object.keyed_properties {
                    match &keyed_property.target {
                        RuntimeKeyedPropertyTarget::Double { transform_property } => {
                            if !object.property_keys.insert(keyed_property.property_key) {
                                continue;
                            }
                            let value = if use_baseline {
                                Some(keyed_property.first_double_value().unwrap_or(0.0))
                            } else {
                                current_animation_reset_double_value(
                                    artboard,
                                    keyed_object.target_local_id,
                                    keyed_property.property_key,
                                    *transform_property,
                                )
                            };
                            if let Some(value) = value {
                                object.entries.push(AnimationResetEntry::Double {
                                    local_id: keyed_object.target_local_id,
                                    property_key: keyed_property.property_key,
                                    transform_property: *transform_property,
                                    value,
                                });
                            }
                        }
                        RuntimeKeyedPropertyTarget::Color {
                            solid_color_property,
                            data_bind_observed,
                        } => {
                            if !object.property_keys.insert(keyed_property.property_key) {
                                continue;
                            }
                            let value = if use_baseline {
                                Some(keyed_property.first_color_value().unwrap_or(0))
                            } else if *solid_color_property {
                                artboard.solid_color_value(keyed_object.target_local_id)
                            } else {
                                artboard.color_property(
                                    keyed_object.target_local_id,
                                    keyed_property.property_key,
                                )
                            };
                            if let Some(value) = value {
                                object.entries.push(AnimationResetEntry::Color {
                                    local_id: keyed_object.target_local_id,
                                    property_key: keyed_property.property_key,
                                    solid_color_property: *solid_color_property,
                                    data_bind_observed: *data_bind_observed,
                                    value: AnimationResetColorValue::from_color(value),
                                });
                            }
                        }
                        RuntimeKeyedPropertyTarget::Bool
                        | RuntimeKeyedPropertyTarget::Uint
                        | RuntimeKeyedPropertyTarget::Int
                        | RuntimeKeyedPropertyTarget::String
                        | RuntimeKeyedPropertyTarget::Callback { .. } => {}
                    }
                }
            }
        }

        for object in objects {
            entries.extend(object.entries);
        }
        AnimationReset {
            storage: Arc::new(AnimationResetStorage { entries }),
        }
    }
}

fn current_animation_reset_double_value(
    artboard: &ArtboardInstance,
    local_id: usize,
    property_key: u16,
    transform_property: Option<TransformProperty>,
) -> Option<f32> {
    if let Some(property) = transform_property {
        artboard.transform_property(local_id, property)
    } else {
        artboard.double_property(local_id, property_key)
    }
}
