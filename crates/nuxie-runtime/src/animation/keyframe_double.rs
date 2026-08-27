#[derive(Debug, Clone)]
/// Direct owner for pinned C++ `src/animation/keyframe_double.cpp`.
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

    /// Mirrors `KeyFrameDouble::apply`; the caller supplies Rust's type-safe
    /// equivalent of the CoreRegistry current-value read.
    fn apply(
        &self,
        mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
        current: impl FnOnce() -> Option<f32>,
    ) -> Option<f32> {
        apply_key_frame_double_mix(self.effective_value(key_frame_values), mix, current)
    }

    /// Mirrors `KeyFrameDouble::applyInterpolation`; the value returned here
    /// is handed to Rust's type-safe CoreRegistry write by the caller.
    fn apply_interpolation(
        &self,
        current_time: f32,
        next: &Self,
        mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
        script_context: Option<RuntimeScriptedInterpolationContext<'_>>,
        current: impl FnOnce() -> Option<f32>,
    ) -> Option<f32> {
        let from_value = self.effective_value(key_frame_values);
        let to_value = next.effective_value(key_frame_values);
        let factor = (current_time - self.seconds) / (next.seconds - self.seconds);

        let frame_value = match self.interpolator {
            Some(RuntimeInterpolator::Scripted { global_id }) => script_context.map_or_else(
                || from_value + (to_value - from_value) * factor,
                |context| {
                    context.evaluate(
                        self.global_id,
                        global_id,
                        ScriptInterpolatorMethod::TransformValue,
                        &[from_value, to_value, factor],
                        from_value + (to_value - from_value) * factor,
                    )
                },
            ),
            Some(interpolator) => interpolator.transform_value(from_value, to_value, factor),
            None => from_value + (to_value - from_value) * factor,
        };

        apply_key_frame_double_mix(frame_value, mix, current)
    }
}

/// Mirrors the source-local `applyDouble` helper. Keep the current-value read
/// lazy because pinned C++ does not call `CoreRegistry::getDouble` at full mix.
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
