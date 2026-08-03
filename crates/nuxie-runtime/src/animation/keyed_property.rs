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

    #[cfg(test)]
    fn double_frame_value_at(
        &self,
        seconds: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Option<f32> {
        self.double_frame_value_at_with_script_context(seconds, key_frame_values, None)
    }

    fn double_frame_value_at_with_script_context(
        &self,
        seconds: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
        script_context: Option<RuntimeScriptedInterpolationContext<'_>>,
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
                let from_value = from.effective_value(key_frame_values);
                let to_value = to.effective_value(key_frame_values);
                match from.interpolator? {
                    RuntimeInterpolator::Scripted { global_id } => script_context.map_or_else(
                        || from_value + (to_value - from_value) * frame_mix,
                        |context| {
                            context.evaluate(
                                from.global_id,
                                global_id,
                                ScriptInterpolatorMethod::TransformValue,
                                &[from_value, to_value, frame_mix],
                                from_value + (to_value - from_value) * frame_mix,
                            )
                        },
                    ),
                    interpolator => interpolator.transform_value(from_value, to_value, frame_mix),
                }
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
            } else if from.interpolator_id.is_some() {
                let frame_mix = frame_mix(seconds, from.seconds, to.seconds);
                let factor = match from.interpolator? {
                    RuntimeInterpolator::Scripted { global_id } => {
                        script_context.map_or(frame_mix, |context| {
                            context.evaluate(
                                from.global_id,
                                global_id,
                                ScriptInterpolatorMethod::Transform,
                                &[frame_mix],
                                frame_mix,
                            )
                        })
                    }
                    interpolator => interpolator.transform(frame_mix),
                };
                color_lerp(
                    from.effective_value(key_frame_values),
                    to.effective_value(key_frame_values),
                    factor,
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

    fn int_value_at(&self, seconds: f32) -> Option<i32> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds);
        let value = if idx == 0 {
            self.key_frames[0].as_int()?.value
        } else if idx < self.key_frames.len() {
            let from = self.key_frames[idx - 1].as_int()?;
            let to = self.key_frames[idx].as_int()?;
            if seconds == to.seconds {
                to.value
            } else {
                from.value
            }
        } else {
            self.key_frames.last()?.as_int()?.value
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
            let callback = RuntimeKeyedCallback {
                target_local_id,
                property_key: self.property_key,
                seconds_delay,
            };
            callback_sink(callback, None);
            index += 1;
        }
    }

    fn closest_frame_index(&self, seconds: f32) -> usize {
        closest_key_frame_index(&self.key_frames, seconds)
    }
}
