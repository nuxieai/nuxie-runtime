#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameInt {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: i32,
}
