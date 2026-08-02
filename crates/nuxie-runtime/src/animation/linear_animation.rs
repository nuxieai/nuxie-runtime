// Mirrors src/animation/linear_animation.cpp plus keyed object/property keyframe sampling.
#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
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

#[derive(Clone, Copy)]
enum RuntimeScriptedInterpolationContext<'a> {
    Shared(&'a ArtboardInstance),
    Stateful(&'a LinearAnimationInstance, &'a ArtboardInstance),
}

impl RuntimeScriptedInterpolationContext<'_> {
    fn evaluate(
        self,
        key_frame_global_id: u32,
        interpolator_global_id: u32,
        method: ScriptInterpolatorMethod,
        arguments: &[f32],
        fallback: f32,
    ) -> f32 {
        match self {
            Self::Shared(artboard) => artboard.evaluate_shared_scripted_interpolator(
                key_frame_global_id,
                interpolator_global_id,
                method,
                arguments,
                fallback,
            ),
            Self::Stateful(animation, artboard) => animation.evaluate_scripted_interpolator(
                artboard,
                key_frame_global_id,
                interpolator_global_id,
                method,
                arguments,
                fallback,
            ),
        }
    }
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
    /// File-global ScriptedInterpolator ids referenced by this animation's
    /// keyframes, in first-use order.
    #[doc(hidden)]
    pub fn scripted_interpolator_global_ids(&self) -> Vec<u32> {
        let mut ids = Vec::new();
        for interpolator in self
            .keyed_objects
            .iter()
            .flat_map(|object| &object.keyed_properties)
            .flat_map(|property| &property.key_frames)
            .filter_map(|frame| match frame {
                RuntimeKeyFrame::Double(frame) => frame.interpolator,
                RuntimeKeyFrame::Color(frame) => frame.interpolator,
                _ => None,
            })
        {
            if let RuntimeInterpolator::Scripted { global_id } = interpolator
                && !ids.contains(&global_id)
            {
                ids.push(global_id);
            }
        }
        ids
    }

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
            None,
        )
    }

    fn apply_with_key_frame_values(
        &self,
        instance: &mut ArtboardInstance,
        seconds: f32,
        mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
        animation_instance: Option<&LinearAnimationInstance>,
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
                        let Some(frame_value) = keyed_property
                            .double_frame_value_at_with_script_context(
                                seconds,
                                key_frame_values,
                                Some(match animation_instance {
                                    Some(animation) => {
                                        RuntimeScriptedInterpolationContext::Stateful(
                                            animation, &*instance,
                                        )
                                    }
                                    None => RuntimeScriptedInterpolationContext::Shared(&*instance),
                                }),
                            )
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
                        let Some(frame_value) = keyed_property
                            .color_frame_value_at_with_script_context(
                                seconds,
                                key_frame_values,
                                Some(match animation_instance {
                                    Some(animation) => {
                                        RuntimeScriptedInterpolationContext::Stateful(
                                            animation, &*instance,
                                        )
                                    }
                                    None => RuntimeScriptedInterpolationContext::Shared(&*instance),
                                }),
                            )
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
        callback_sink: &mut dyn FnMut(RuntimeKeyedCallback, Option<StateMachineReportedEvent>),
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
                    callback_sink,
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
        let (start_time, end_time) = if self.speed >= 0.0 {
            (self.start_seconds(), self.end_seconds())
        } else {
            (self.end_seconds(), self.start_seconds())
        };
        match AnimationLoop::from_loop_value(self.loop_value as i32) {
            AnimationLoop::OneShot => seconds + start_time,
            AnimationLoop::Loop => positive_mod(seconds, self.duration_seconds()) + start_time,
            AnimationLoop::PingPong => {
                let duration = self.duration_seconds();
                let local_time = positive_mod(seconds, duration);
                let direction = (seconds / duration) as i32 % 2;
                if direction == 0 {
                    local_time + start_time
                } else {
                    end_time - local_time
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