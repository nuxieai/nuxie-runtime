#[derive(Debug, Clone)]
/// Direct owner for pinned C++ `src/animation/animation_reset.cpp`.
pub(crate) struct AnimationReset {
    // `StateMachineInstance::clone` is a Rust snapshot API with no C++
    // occurrence-copy counterpart. Share this completed reset lease instead
    // of duplicating the factory-owned storage.
    pub(super) storage: Arc<AnimationResetStorage>,
}

#[derive(Debug)]
pub(super) struct AnimationResetStorage {
    pub(super) entries: Vec<AnimationResetEntry>,
    pub(super) data: Vec<u8>,
}

#[derive(Debug)]
pub(super) enum AnimationResetEntry {
    Double {
        local_id: usize,
        property_key: u16,
        transform_property: Option<TransformProperty>,
        value: Option<f32>,
    },
    Color {
        local_id: usize,
        property_key: u16,
        solid_color_property: bool,
        data_bind_observed: bool,
        value: Option<AnimationResetColorValue>,
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

    pub(super) fn encoded(self) -> f32 {
        match self {
            Self::DefinedFloat(value) | Self::SaturatingFloatToInt(value) => value,
        }
    }

    fn replay_encoded(value: f32) -> u32 {
        (value as i32) as u32
    }
}

impl AnimationReset {
    /// Applies the completed reset stream in object/property order.
    pub(crate) fn apply(&self, artboard: &mut ArtboardInstance) -> bool {
        if self.storage.data.is_empty() {
            return false;
        }

        let mut changed = false;
        let mut reader = nuxie_binary::BinaryDataReader::new(&self.storage.data);
        while !reader.is_eof() {
            let local_id = reader.read_var_uint32() as usize;
            let total_properties = reader.read_var_uint32();
            for _ in 0..total_properties {
                let property_key = reader.read_var_uint32();
                let property_value = reader.read_float32();
                let Ok(property_key) = u16::try_from(property_key) else {
                    continue;
                };
                match nuxie_schema::core_registry_field_kind_by_property_key(property_key) {
                    Some(nuxie_schema::CoreRegistryFieldKind::Double) => {
                        let transform_property =
                            self.storage.entries.iter().find_map(|entry| match entry {
                                AnimationResetEntry::Double {
                                    local_id: entry_local_id,
                                    property_key: entry_property_key,
                                    transform_property,
                                    ..
                                } if *entry_local_id == local_id
                                    && *entry_property_key == property_key =>
                                {
                                    Some(*transform_property)
                                }
                                _ => None,
                            });
                        changed |= match transform_property.flatten() {
                            Some(transform_property) => artboard.set_transform_property_with_key(
                                local_id,
                                transform_property,
                                property_key,
                                property_value,
                            ),
                            None => artboard.set_keyed_double_property(
                                local_id,
                                property_key,
                                property_value,
                            ),
                        };
                    }
                    Some(nuxie_schema::CoreRegistryFieldKind::Color) => {
                        let routing = self.storage.entries.iter().find_map(|entry| match entry {
                            AnimationResetEntry::Color {
                                local_id: entry_local_id,
                                property_key: entry_property_key,
                                solid_color_property,
                                data_bind_observed,
                                ..
                            } if *entry_local_id == local_id
                                && *entry_property_key == property_key =>
                            {
                                Some((*solid_color_property, *data_bind_observed))
                            }
                            _ => None,
                        });
                        let value = AnimationResetColorValue::replay_encoded(property_value);
                        changed |= match routing {
                            Some((true, data_bind_observed)) => artboard
                                .set_keyed_solid_color_property(
                                    local_id,
                                    property_key,
                                    data_bind_observed,
                                    value,
                                ),
                            _ => artboard.set_keyed_color_property(local_id, property_key, value),
                        };
                    }
                    _ => {}
                }
            }
        }
        changed
    }
}
