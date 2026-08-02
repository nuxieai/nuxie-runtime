// Mirrors KeyFrame::computeSeconds (`src/animation/keyframe.cpp`) as invoked
// once by KeyedPropertyImporter::resolve (`src/importers/keyed_property_importer.cpp`).
fn retained_key_frame_seconds(frame: u64, fps: u64) -> f32 {
    frame as f32 / fps as f32
}

fn closest_key_frame_index(key_frames: &[RuntimeKeyFrame], seconds: f32) -> usize {
    closest_key_frame_index_with_exact_offset(key_frames, seconds, 0)
}

fn closest_key_frame_index_with_exact_offset(
    key_frames: &[RuntimeKeyFrame],
    seconds: f32,
    exact_offset: usize,
) -> usize {
    let last = key_frames.len() - 1;
    if seconds > key_frames[last].seconds() {
        return key_frames.len();
    }

    let mut start = 0;
    let mut end = last;
    while start <= end {
        let mid = (start + end) >> 1;
        let closest = key_frames[mid].seconds();
        if closest < seconds {
            start = mid + 1;
        } else if closest > seconds {
            if mid == 0 {
                break;
            }
            end = mid - 1;
        } else {
            return mid + exact_offset;
        }
    }
    start
}

fn frame_mix(seconds: f32, from_seconds: f32, to_seconds: f32) -> f32 {
    if to_seconds == from_seconds {
        1.0
    } else {
        (seconds - from_seconds) / (to_seconds - from_seconds)
    }
}

/// The concrete keyframe occurrence owned by a `RuntimeKeyedProperty`.
///
/// Mirrors C++ `KeyedProperty::m_keyFrames`: concrete subclasses share one
/// insertion-ordered owner sequence instead of being partitioned by Rust type.
#[derive(Debug, Clone)]
pub enum RuntimeKeyFrame {
    Double(RuntimeKeyFrameDouble),
    Color(RuntimeKeyFrameColor),
    Bool(RuntimeKeyFrameBool),
    Uint(RuntimeKeyFrameUint),
    String(RuntimeKeyFrameString),
    Callback(RuntimeKeyFrameCallback),
}

impl RuntimeKeyFrame {
    fn global_id(&self) -> u32 {
        match self {
            Self::Double(frame) => frame.global_id,
            Self::Color(frame) => frame.global_id,
            Self::Bool(frame) => frame.global_id,
            Self::Uint(frame) => frame.global_id,
            Self::String(frame) => frame.global_id,
            Self::Callback(frame) => frame.global_id,
        }
    }

    fn seconds(&self) -> f32 {
        match self {
            Self::Double(frame) => frame.seconds,
            Self::Color(frame) => frame.seconds,
            Self::Bool(frame) => frame.seconds,
            Self::Uint(frame) => frame.seconds,
            Self::String(frame) => frame.seconds,
            Self::Callback(frame) => frame.seconds,
        }
    }

    fn bindable_global_id(&self) -> Option<u32> {
        match self {
            Self::Double(_) | Self::Color(_) | Self::Bool(_) | Self::String(_) => {
                Some(self.global_id())
            }
            Self::Uint(_) | Self::Callback(_) => None,
        }
    }

    fn as_double(&self) -> Option<&RuntimeKeyFrameDouble> {
        match self {
            Self::Double(frame) => Some(frame),
            _ => None,
        }
    }

    fn as_color(&self) -> Option<&RuntimeKeyFrameColor> {
        match self {
            Self::Color(frame) => Some(frame),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<&RuntimeKeyFrameBool> {
        match self {
            Self::Bool(frame) => Some(frame),
            _ => None,
        }
    }

    fn as_uint(&self) -> Option<&RuntimeKeyFrameUint> {
        match self {
            Self::Uint(frame) => Some(frame),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&RuntimeKeyFrameString> {
        match self {
            Self::String(frame) => Some(frame),
            _ => None,
        }
    }
}
