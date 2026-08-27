// Mirrors src/animation/keyed_property.cpp and
// include/rive/animation/keyed_property.hpp.
//
// The approved AF-7 own-by-value/import-time-devirtualization adaptation runs
// `import`, `onAddedDirty`, and `onAddedClean` while flattening RuntimeFile:
// KeyedPropertyImporter transfers each imported frame in authored order,
// build_linear_animations resolves every InterpolatingKeyFrame and propagates
// a failed resolution through the owning KeyedObject, and no pinned KeyFrame
// subtype adds an onAddedClean body beyond the successful inherited callback.
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
    Int,
    String,
    Callback {
        event_local_index: Option<usize>,
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

impl RuntimeKeyedProperty {
    pub(crate) fn new(
        global_id: u32,
        property_key: u16,
        target: RuntimeKeyedPropertyTarget,
    ) -> Self {
        Self {
            global_id,
            property_key,
            target,
            key_frames: Vec::new(),
        }
    }

    pub(crate) fn add_key_frame(&mut self, key_frame: RuntimeKeyFrame) {
        self.key_frames.push(key_frame);
    }

    fn closest_frame_index(&self, seconds: f32) -> usize {
        self.closest_frame_index_with_exact_offset(seconds, 0)
    }

    fn closest_frame_index_with_exact_offset(&self, seconds: f32, exact_offset: usize) -> usize {
        let last = self.key_frames.len() - 1;
        if seconds > self.key_frames[last].seconds() {
            return self.key_frames.len();
        }

        let mut start = 0;
        let mut end = last;
        while start <= end {
            let mid = (start + end) >> 1;
            let closest_seconds = self.key_frames[mid].seconds();
            if closest_seconds < seconds {
                start = mid + 1;
            } else if closest_seconds > seconds {
                // C++ uses a signed end index. This is the equivalent
                // underflow guard for Rust's slice index type.
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

    pub(crate) fn first_double_value(&self) -> Option<f32> {
        self.key_frames
            .first()?
            .as_double()
            .map(|frame| frame.value)
    }

    pub(crate) fn first_color_value(&self) -> Option<u32> {
        self.key_frames.first()?.as_color().map(|frame| frame.value)
    }

    #[cfg(test)]
    fn double_frame_value_at(
        &self,
        seconds: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<f32> {
        self.double_value_at_with_script_context(seconds, 1.0, key_frame_values, None, || None)
    }

    fn double_value_at_with_script_context(
        &self,
        seconds: f32,
        mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
        script_context: Option<RuntimeScriptedInterpolationContext<'_>>,
        current: impl FnOnce() -> Option<f32>,
    ) -> Option<f32> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        if idx == 0 {
            self.key_frames[0]
                .as_double()?
                .apply(mix, key_frame_values, current)
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].as_double()?;
            let to = self.key_frames[idx].as_double()?;
            if seconds == to.seconds {
                to.apply(mix, key_frame_values, current)
            } else if from.interpolation_type == 0 {
                from.apply(mix, key_frame_values, current)
            } else {
                from.apply_interpolation(
                    seconds,
                    to,
                    mix,
                    key_frame_values,
                    script_context,
                    current,
                )
            }
        } else {
            self.key_frames
                .last()?
                .as_double()?
                .apply(mix, key_frame_values, current)
        }
    }

    #[cfg(test)]
    fn color_frame_value_at(
        &self,
        seconds: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<u32> {
        self.color_frame_value_at_with_script_context(seconds, key_frame_values, None)
    }

    fn color_frame_value_at_with_script_context(
        &self,
        seconds: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
        script_context: Option<RuntimeScriptedInterpolationContext<'_>>,
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
            } else {
                from.interpolation_value(seconds, to, key_frame_values, script_context)
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
        mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<bool> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        let value = if idx == 0 {
            self.key_frames[0].as_bool()?.apply(mix, key_frame_values)
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].as_bool()?;
            let to = self.key_frames[idx].as_bool()?;
            if seconds == to.seconds {
                to.apply(mix, key_frame_values)
            } else if from.interpolation_type == 0 {
                from.apply(mix, key_frame_values)
            } else {
                from.apply_interpolation(seconds, to, mix, key_frame_values)
            }
        } else {
            self.key_frames
                .last()?
                .as_bool()?
                .apply(mix, key_frame_values)
        };

        Some(value)
    }

    fn uint_value_at(&self, seconds: f32) -> Option<u64> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        let value = if idx == 0 {
            self.key_frames[0].unsigned_value()?
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].unsigned_value()?;
            let to = self.key_frames[idx].unsigned_value()?;
            if seconds == self.key_frames[idx].seconds() {
                to
            } else {
                from
            }
        } else {
            self.key_frames.last()?.unsigned_value()?
        };

        Some(value)
    }

    fn int_value_at(&self, seconds: f32) -> Option<i32> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        let value = if idx == 0 {
            self.key_frames[0].as_int()?.applied_value()
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].as_int()?;
            let to = self.key_frames[idx].as_int()?;
            if seconds == to.seconds {
                to.applied_value()
            } else {
                from.applied_value()
            }
        } else {
            self.key_frames.last()?.as_int()?.applied_value()
        };

        Some(value)
    }

    fn string_value_at(
        &self,
        seconds: f32,
        mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<Vec<u8>> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        let value = if idx == 0 {
            self.key_frames[0].as_string()?.apply(mix, key_frame_values)
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].as_string()?;
            let to = self.key_frames[idx].as_string()?;
            if seconds == to.seconds {
                to.apply(mix, key_frame_values)
            } else if from.interpolation_type == 0 {
                from.apply(mix, key_frame_values)
            } else {
                from.apply_interpolation(seconds, to, mix, key_frame_values)
            }
        } else {
            self.key_frames
                .last()?
                .as_string()?
                .apply(mix, key_frame_values)
        };

        Some(value)
    }

    fn report_keyed_callbacks(
        &self,
        target_local_id: usize,
        seconds_from: f32,
        seconds_to: f32,
        is_at_start_frame: bool,
        callback_sink: &mut dyn FnMut(RuntimeKeyedCallback, Option<StateMachineReportedEvent>),
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

        let mut index = self.closest_frame_index_with_exact_offset(seconds_from, from_exact_offset);
        let mut index_to = self.closest_frame_index_with_exact_offset(seconds_to, to_exact_offset);
        if index_to < index {
            std::mem::swap(&mut index, &mut index_to);
        }

        while index_to > index {
            let key_frame = &self.key_frames[index];
            let seconds_delay = seconds_to - key_frame.seconds();
            let callback = RuntimeKeyedCallback {
                target_local_id,
                property_key: self.property_key,
                seconds_delay,
            };
            callback_sink(callback, None);
            index += 1;
        }
    }

    fn apply(
        &self,
        instance: &mut ArtboardInstance,
        target_local_id: usize,
        seconds: f32,
        mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
        animation_instance: Option<&LinearAnimationInstance>,
    ) -> bool {
        // Pinned C++ asserts this precondition and then indexes the vector.
        // Rust cannot continue through the corresponding out-of-bounds path.
        if self.key_frames.is_empty() {
            return false;
        }

        // CoreRegistry assigns exactly one field type per property, matching
        // the pinned virtual dispatch through InterpolatingKeyFrame::apply.
        let actual_mix =
            keyed_property_actual_mix(&*instance, target_local_id, self.property_key, mix);
        match &self.target {
            RuntimeKeyedPropertyTarget::Double { transform_property } => {
                let Some(value) = self.double_value_at_with_script_context(
                    seconds,
                    actual_mix,
                    key_frame_values,
                    Some(effective_scripted_interpolation_context(
                        animation_instance,
                        &*instance,
                    )),
                    || match transform_property {
                        Some(transform_property) => instance.transform_property_with_key(
                            target_local_id,
                            *transform_property,
                            self.property_key,
                        ),
                        None => instance.double_property(target_local_id, self.property_key),
                    },
                ) else {
                    return false;
                };
                match transform_property {
                    Some(transform_property) => instance.set_transform_property_with_key(
                        target_local_id,
                        *transform_property,
                        self.property_key,
                        value,
                    ),
                    None => instance.set_keyed_double_property(
                        target_local_id,
                        self.property_key,
                        value,
                    ),
                }
            }
            RuntimeKeyedPropertyTarget::Color {
                solid_color_property,
                data_bind_observed,
            } => {
                let Some(frame_value) = self.color_frame_value_at_with_script_context(
                    seconds,
                    key_frame_values,
                    Some(effective_scripted_interpolation_context(
                        animation_instance,
                        &*instance,
                    )),
                ) else {
                    return false;
                };
                let Some(value) = apply_key_frame_color_mix(frame_value, actual_mix, || {
                    if *solid_color_property {
                        instance.solid_color_value(target_local_id)
                    } else {
                        instance.color_property(target_local_id, self.property_key)
                    }
                }) else {
                    return false;
                };
                if *solid_color_property {
                    instance.set_keyed_solid_color_property(
                        target_local_id,
                        self.property_key,
                        *data_bind_observed,
                        value,
                    )
                } else {
                    instance.set_keyed_color_property(target_local_id, self.property_key, value)
                }
            }
            RuntimeKeyedPropertyTarget::Bool => {
                let Some(value) = self.bool_value_at(seconds, actual_mix, key_frame_values) else {
                    return false;
                };
                instance.set_bool_property(target_local_id, self.property_key, value)
            }
            RuntimeKeyedPropertyTarget::Uint => {
                let Some(value) = self.uint_value_at(seconds) else {
                    return false;
                };
                instance.set_uint_property(target_local_id, self.property_key, value)
            }
            RuntimeKeyedPropertyTarget::Int => {
                let Some(value) = self.int_value_at(seconds) else {
                    return false;
                };
                instance.set_int_property(target_local_id, self.property_key, value)
            }
            RuntimeKeyedPropertyTarget::String => {
                let Some(value) = self.string_value_at(seconds, actual_mix, key_frame_values)
                else {
                    return false;
                };
                instance.set_string_property(target_local_id, self.property_key, value)
            }
            RuntimeKeyedPropertyTarget::Callback { .. } => false,
        }
    }

    pub(crate) fn first(&self) -> Option<&RuntimeKeyFrame> {
        self.key_frames.first()
    }

    pub(crate) fn num_key_frames(&self) -> usize {
        self.key_frames.len()
    }

    pub(crate) fn get_key_frame(&self, index: usize) -> Option<&RuntimeKeyFrame> {
        self.key_frames.get(index)
    }
}

/// Mirrors `InterpolatorHost::from` followed by
/// `InterpolatorHost::overridesKeyedInterpolation` in `KeyedProperty::apply`.
///
/// The pinned static dispatch checks `coreType()` rather than `isTypeOf()`, so
/// only the concrete `LayoutComponent` type is an interpolator host. Its
/// implementation overrides the caller's keyed mix for width and height only
/// while the component's own layout animation is active.
fn interpolator_host_overrides_keyed_interpolation(
    artboard: &ArtboardInstance,
    target_local_id: usize,
    property_key: u16,
) -> bool {
    let Some(component) = artboard.component(target_local_id) else {
        return false;
    };
    if component.type_name != "LayoutComponent" {
        return false;
    }
    let Some(layout) = component.concrete.layout.as_ref() else {
        return false;
    };

    layout.animates()
        && ["width", "height"].into_iter().any(|property_name| {
            crate::properties::property_key_for_name("LayoutComponent", property_name)
                == Some(property_key)
        })
}

/// Pinned `KeyedProperty::apply` forces a host-owned property to its complete
/// keyed value, leaving the host to perform its own interpolation.
fn keyed_property_actual_mix(
    artboard: &ArtboardInstance,
    target_local_id: usize,
    property_key: u16,
    mix: f32,
) -> f32 {
    if interpolator_host_overrides_keyed_interpolation(artboard, target_local_id, property_key) {
        1.0
    } else {
        mix
    }
}
