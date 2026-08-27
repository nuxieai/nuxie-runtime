#[derive(Debug, Clone)]
/// Direct owner for pinned C++ `src/animation/keyframe_string.cpp`.
pub struct RuntimeKeyFrameString {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: Vec<u8>,
}

impl RuntimeKeyFrameString {
    fn effective_value(&self, key_frame_values: RuntimeKeyFrameValueContext<'_>) -> Vec<u8> {
        key_frame_values
            .string(self.global_id)
            .unwrap_or(&self.value)
            .to_vec()
    }

    /// Mirrors `KeyFrameString::apply`: mix does not affect a string keyframe;
    /// the effective value is written directly.
    fn apply(
        &self,
        _mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Vec<u8> {
        self.effective_value(key_frame_values)
    }

    /// Mirrors `KeyFrameString::applyInterpolation`: interpolation time, the
    /// next frame, and mix are ignored and the effective value is written
    /// directly.
    fn apply_interpolation(
        &self,
        _current_time: f32,
        _next_frame: &Self,
        _mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
    ) -> Vec<u8> {
        self.effective_value(key_frame_values)
    }
}
