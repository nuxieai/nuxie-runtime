#[derive(Debug, Clone)]
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
}
