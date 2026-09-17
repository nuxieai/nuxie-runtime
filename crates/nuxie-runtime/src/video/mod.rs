//! Nuxie-owned video scene objects. Rive's generated owners remain unchanged.
pub mod admission;
pub mod captions;
mod objects;
pub mod playback;
pub mod readiness;
pub mod resources;
pub mod sync;
pub use objects::{Video, VideoAsset};

pub(crate) fn make_core(type_key: i32) -> Option<Box<dyn crate::source::core::CoreObject>> {
    match type_key {
        60000 => Some(Box::new(VideoAsset::default())),
        60001 => Some(Box::new(Video::default())),
        _ => crate::source::generated::core_registry::CoreRegistry::make_core_box(type_key),
    }
}
