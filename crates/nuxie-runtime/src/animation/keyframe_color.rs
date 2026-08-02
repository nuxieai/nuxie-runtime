#[derive(Debug, Clone)]
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
}
