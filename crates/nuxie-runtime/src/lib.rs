//! Rust runtime backed only by the pinned translated source owners.
//!
//! [`File::import`] requires the renderer [`RuntimeFactoryHandle`] up front.
//! The imported file and its artboards retain that factory; drawing takes only
//! a [`Renderer`]. The complete upstream-shaped API is available in [`source`].

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

// Internal module path used by the approved scripting boundary.
pub(crate) use host_state_machine as state_machine;

pub use host_animation::*;
pub use host_artboard::*;
pub use host_assets::*;
pub use host_state_machine::*;
pub use host_viewmodel::*;
pub use scripting::*;
pub use text::*;

pub use mechanical_port::source;
pub use mechanical_port::source::advance_flags::AdvanceFlags;
pub use mechanical_port::source::animation::state_machine_instance::{
    RuntimeStateMachineInstanceHandle, RuntimeStateMachineInstanceWeakHandle,
};
pub use mechanical_port::source::artboard::{
    Artboard, RuntimeArtboardInstanceHandle, RuntimeArtboardInstanceWeakHandle,
};
pub use mechanical_port::source::r#async::work_pool::*;
pub use mechanical_port::source::r#async::work_task::*;
pub use mechanical_port::source::component_dirt::ComponentDirt;
pub use mechanical_port::source::core::CoreHandle;
pub use mechanical_port::source::factory::{Factory, RuntimeFactoryHandle};
pub use mechanical_port::source::file::{
    File, ImportResult, RuntimeFileHandle, RuntimeFileWeakHandle,
};
pub use mechanical_port::source::file_asset_loader::{FileAssetLoader, FileAssetLoaderRef};
pub use mechanical_port::source::input::gamepad_batch::{
    GAMEPAD_BATCH_MAX_AXES, GAMEPAD_BATCH_MAX_BUTTONS, GAMEPAD_BATCH_WIRE_VERSION,
};
pub use mechanical_port::source::lua::scripting_vm::RuntimeScriptingVmHandle;
pub use mechanical_port::source::profiler::rive_profile::*;
pub use mechanical_port::source::semantic::semantic_snapshot::*;
pub use nuxie_audio::{
    AudioArtboardId, AudioDecodeError, AudioEngine, AudioEngineError, AudioFormat, AudioReader,
    AudioSound, AudioSource,
};
pub use nuxie_render_api::{Mat2D, PersistentFactoryContext, Renderer};

#[cfg(test)]
mod host_public_api_tests;
