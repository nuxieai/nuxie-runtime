use super::*;

#[derive(Debug, Clone)]
pub(super) struct AnimationReset {
    // `StateMachineInstance::clone` is a Rust snapshot API with no C++
    // occurrence-copy counterpart. Share this immutable reset lease so a
    // snapshot never clones factory state; the final Arc owner returns the
    // cleared storage to the C++-shaped global pool.
    pub(super) storage: Arc<AnimationResetStorage>,
}

#[derive(Debug)]
pub(super) struct AnimationResetStorage {
    pub(super) entries: Vec<AnimationResetEntry>,
}

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
pub(super) enum AnimationResetEntry {
    Double {
        local_id: usize,
        property_key: u16,
        transform_property: Option<TransformProperty>,
        value: f32,
    },
    Color {
        local_id: usize,
        property_key: u16,
        solid_color_property: bool,
        data_bind_observed: bool,
        value: AnimationResetColorValue,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum AnimationResetColorValue {
    /// Pinned C++ serializes the signed color through `float` and converts the
    /// decoded float back to `int` in `CoreRegistry::setColor`.
    DefinedFloat(f32),
    /// A positive `int` close enough to `INT_MAX` rounds to 2^31 as `float`;
    /// converting that value back to `int` is undefined in C++. Preserve the
    /// serialized float and apply the project's explicit saturating conversion
    /// decision instead of attempting to emulate undefined behavior.
    SaturatingFloatToInt(f32),
}

impl AnimationResetColorValue {
    pub(super) fn from_color(value: u32) -> Self {
        let encoded = (value as i32) as f32;
        if encoded < 2_147_483_648.0 {
            Self::DefinedFloat(encoded)
        } else {
            Self::SaturatingFloatToInt(encoded)
        }
    }

    pub(super) fn replay(self) -> u32 {
        match self {
            Self::DefinedFloat(value) => (value as i32) as u32,
            // Project divergence D2 binds Rust's saturating conversion where
            // the corresponding C++ float-to-int conversion is undefined.
            Self::SaturatingFloatToInt(value) => (value as i32) as u32,
        }
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

pub(super) struct AnimationResetFactory;

fn animation_reset_pool() -> &'static Mutex<Vec<Vec<AnimationResetEntry>>> {
    static POOL: OnceLock<Mutex<Vec<Vec<AnimationResetEntry>>>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(Vec::new()))
}

impl AnimationResetFactory {
    pub(super) fn from_animation_instances<'a>(
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

impl AnimationReset {
    pub(super) fn apply(&self, artboard: &mut ArtboardInstance) -> bool {
        let mut changed = false;
        for entry in &self.storage.entries {
            match entry {
                AnimationResetEntry::Double {
                    local_id,
                    property_key,
                    transform_property,
                    value,
                } => {
                    changed |= match transform_property {
                        Some(transform_property) => artboard.set_transform_property_with_key(
                            *local_id,
                            *transform_property,
                            *property_key,
                            *value,
                        ),
                        None => {
                            artboard.set_keyed_double_property(*local_id, *property_key, *value)
                        }
                    };
                }
                AnimationResetEntry::Color {
                    local_id,
                    property_key,
                    solid_color_property,
                    data_bind_observed,
                    value,
                } => {
                    changed |= if *solid_color_property {
                        artboard.set_keyed_solid_color_property(
                            *local_id,
                            *property_key,
                            *data_bind_observed,
                            value.replay(),
                        )
                    } else {
                        artboard.set_keyed_color_property(*local_id, *property_key, value.replay())
                    };
                }
            }
        }
        changed
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
