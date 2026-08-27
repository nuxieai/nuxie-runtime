#[derive(Debug, Clone)]
/// Direct owner for pinned C++ `src/animation/keyframe_uint.cpp`.
pub struct RuntimeKeyFrameUint {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: u64,
}

impl RuntimeKeyFrameUint {
    /// Both C++ apply overloads ignore mix, time, the next frame, and the
    /// animation context and write the retained unsigned value directly.
    fn applied_value(&self) -> u64 {
        self.value
    }
}
