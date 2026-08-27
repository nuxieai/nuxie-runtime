#[derive(Debug, Clone)]
/// Direct owner for pinned C++ `src/animation/keyframe_int.cpp`.
pub struct RuntimeKeyFrameInt {
    pub global_id: u32,
    pub frame: u64,
    pub seconds: f32,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: i32,
}

impl RuntimeKeyFrameInt {
    /// Both C++ apply overloads ignore mix, time, the next frame, and the
    /// animation context and write the retained signed value directly.
    fn applied_value(&self) -> i32 {
        self.value
    }
}
