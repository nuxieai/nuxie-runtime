// Rust-only storage adaptation for data-bound keyframe values. This is kept
// outside the LinearAnimation source owner because the pinned C++ animation
// has no corresponding retained value-holder state.

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RuntimeKeyFrameValue {
    Number(f32),
    Color(u32),
    Boolean(bool),
    String(Vec<u8>),
}

#[derive(Debug, Clone, Copy, Default)]
struct RuntimeKeyFrameValueContext<'a> {
    holders: Option<&'a HashMap<u32, RuntimeKeyFrameValue>>,
}

impl<'a> RuntimeKeyFrameValueContext<'a> {
    fn number(self, key_frame_global_id: u32) -> Option<f32> {
        match self.holders?.get(&key_frame_global_id)? {
            RuntimeKeyFrameValue::Number(value) => Some(*value),
            _ => None,
        }
    }

    fn color(self, key_frame_global_id: u32) -> Option<u32> {
        match self.holders?.get(&key_frame_global_id)? {
            RuntimeKeyFrameValue::Color(value) => Some(*value),
            _ => None,
        }
    }

    fn boolean(self, key_frame_global_id: u32) -> Option<bool> {
        match self.holders?.get(&key_frame_global_id)? {
            RuntimeKeyFrameValue::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    fn string(self, key_frame_global_id: u32) -> Option<&'a [u8]> {
        match self.holders?.get(&key_frame_global_id)? {
            RuntimeKeyFrameValue::String(value) => Some(value),
            _ => None,
        }
    }
}
