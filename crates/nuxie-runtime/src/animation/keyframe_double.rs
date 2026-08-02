#[derive(Debug, Clone)]
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
}
