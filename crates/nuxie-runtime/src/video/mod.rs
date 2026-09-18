//! Nuxie-owned video scene objects. Rive's generated owners remain unchanged.
pub mod admission;
pub mod captions;
mod objects;
pub mod playback;
pub mod readiness;
pub mod resources;
pub mod sync;
pub use objects::{Video, VideoAsset};
