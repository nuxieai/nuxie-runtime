#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeKeyedCallback {
    pub(crate) target_local_id: usize,
    pub(crate) property_key: u16,
    pub(crate) seconds_delay: f32,
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameCallback {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
}
