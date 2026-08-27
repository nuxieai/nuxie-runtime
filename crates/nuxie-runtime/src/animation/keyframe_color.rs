#[derive(Debug, Clone)]
/// Direct owner for pinned C++ `src/animation/keyframe_color.cpp`.
pub struct RuntimeKeyFrameColor {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub(crate) interpolator: Option<RuntimeInterpolator>,
    pub value: u32,
}

impl RuntimeKeyFrameColor {
    fn effective_value(&self, key_frame_values: RuntimeKeyFrameValueContext<'_>) -> u32 {
        key_frame_values.color(self.global_id).unwrap_or(self.value)
    }

    /// Mirrors `KeyFrameColor::applyInterpolation` through the value handed to
    /// `applyColor`; the caller retains Rust's type-safe CoreRegistry dispatch.
    fn interpolation_value(
        &self,
        current_time: f32,
        next: &Self,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
        script_context: Option<RuntimeScriptedInterpolationContext<'_>>,
    ) -> u32 {
        let from_value = self.effective_value(key_frame_values);
        let to_value = next.effective_value(key_frame_values);
        let factor = (current_time - self.seconds) / (next.seconds - self.seconds);
        let factor = match self.interpolator {
            Some(RuntimeInterpolator::Scripted { global_id }) => {
                script_context.map_or(factor, |context| {
                    context.evaluate(
                        self.global_id,
                        global_id,
                        ScriptInterpolatorMethod::Transform,
                        &[factor],
                        factor,
                    )
                })
            }
            Some(interpolator) => interpolator.transform(factor),
            None => factor,
        };
        color_lerp(from_value, to_value, factor)
    }
}

/// Mirrors the source-local `applyColor` helper. Keep the current-value read
/// lazy because pinned C++ does not call `CoreRegistry::getColor` at full mix.
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
