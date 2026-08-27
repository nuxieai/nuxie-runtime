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

impl AnimationReset {
    /// Applies the completed reset stream in object/property order. Rust keeps
    /// the decoded entries in owner-safe storage rather than replaying a raw
    /// byte reader, while retaining C++'s Double/Color-only dispatch.
    pub(crate) fn apply(&self, artboard: &mut ArtboardInstance) -> bool {
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
