#[derive(Debug, Clone)]
/// Direct owner for pinned C++ `src/animation/keyframe_bool.cpp`.
pub struct RuntimeKeyFrameBool {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: bool,
}

impl RuntimeKeyFrameBool {
    fn effective_value(&self, key_frame_values: RuntimeKeyFrameValueContext<'_>) -> bool {
        key_frame_values
            .boolean(self.global_id)
            .unwrap_or(self.value)
    }

    /// Mirrors `KeyFrameBool::apply`: mix does not affect a boolean keyframe;
    /// the effective value is written directly.
    fn apply(
        &self,
        _mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> bool {
        self.effective_value(key_frame_values)
    }

    /// Mirrors `KeyFrameBool::applyInterpolation`: interpolation time, the next
    /// frame, and mix are ignored and the effective value is written directly.
    fn apply_interpolation(
        &self,
        _current_time: f32,
        _next_frame: &Self,
        _mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> bool {
        self.effective_value(key_frame_values)
    }
}
