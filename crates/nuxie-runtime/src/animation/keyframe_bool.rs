#[derive(Debug, Clone)]
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
}
