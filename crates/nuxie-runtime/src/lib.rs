//! Rust runtime host API backed only by the pinned translated source owners.

#[doc(hidden)]
pub mod mechanical_port;

mod host_animation;
mod host_artboard;
mod host_assets;
mod host_state_machine;
mod host_viewmodel;
mod scripting;
#[path = "host_text.rs"]
mod text;
#[path = "host_viewmodel/cell.rs"]
pub mod view_model_cell;

// Compatibility module spellings used by the approved host boundaries.
// None of these aliases points to a superseded implementation.
pub(crate) use host_animation as animation;
pub(crate) use host_artboard as artboard;
pub(crate) use host_state_machine as state_machine;
pub(crate) use host_viewmodel as view_model;

pub use host_animation::*;
pub use host_artboard::*;
pub use host_assets::*;
pub use host_state_machine::*;
pub use host_viewmodel::*;
pub use scripting::*;
pub use text::*;

pub use mechanical_port::source::r#async::work_pool::*;
pub use mechanical_port::source::r#async::work_task::*;
pub use mechanical_port::source::component_dirt::ComponentDirt;
pub use mechanical_port::source::input::gamepad_batch::{
    GAMEPAD_BATCH_MAX_AXES, GAMEPAD_BATCH_MAX_BUTTONS, GAMEPAD_BATCH_WIRE_VERSION,
};
pub use mechanical_port::source::profiler::rive_profile::*;
pub use mechanical_port::source::semantic::semantic_snapshot::*;
pub use nuxie_audio::{
    AudioArtboardId, AudioDecodeError, AudioEngine, AudioEngineError, AudioFormat, AudioReader,
    AudioSound, AudioSource,
};
pub use nuxie_render_api::Mat2D;

#[cfg(test)]
mod host_public_api_tests;
