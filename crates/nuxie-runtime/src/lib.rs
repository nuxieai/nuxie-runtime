//! Rust runtime backed only by the pinned translated source owners.
//!
//! [`File::import`] requires the renderer [`RuntimeFactoryHandle`] up front.
//! The imported file and its artboards retain that factory; drawing takes only
//! a [`Renderer`]. The complete upstream-shaped API is available in [`source`].

#[doc(hidden)]
pub mod css_layout;
pub mod mechanical_port;
/// Shared test-only scene builders, for this package's tests and the
/// renderer's. The non-default `test-support` feature keeps them out of
/// every shipping build.
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub mod test_support;
// Lets `test_support` name this crate as its integration tests do.
#[cfg(feature = "test-support")]
extern crate self as nuxie_runtime;

pub mod css_text;
pub mod hosted_scene;
pub mod live_move;
#[cfg(feature = "test-support")]
pub mod live_move_fixture;
pub mod page_player;
pub mod text_decoration;
pub mod utils;
pub mod video;

/// Makes a Nuxie-owned type (keys 60000 and up) or, for any other key, the
/// upstream type. Nuxie types share the Rive record grammar and ordinary
/// arena lifecycle.
pub(crate) fn make_core(type_key: i32) -> Option<Box<dyn source::core::CoreObject>> {
    match type_key {
        60000 => Some(Box::new(video::VideoAsset::default())),
        60001 => Some(Box::new(video::Video::default())),
        61005 => Some(Box::new(css_text::TextStyleCssText::default())),
        60002 => Some(Box::new(text_decoration::TextStyleDecoration::default())),
        62000 => Some(Box::new(live_move::records::LiveSlot::default())),
        62001 => Some(Box::new(live_move::records::LiveStep::default())),
        62002 => Some(Box::new(live_move::records::LiveStepUnit::default())),
        _ => css_layout::make_core(type_key)
            .or_else(|| source::generated::core_registry::CoreRegistry::make_core_box(type_key)),
    }
}

mod host_animation;
mod host_artboard;
mod host_assets;
mod host_geometry;
mod host_semantics;
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
pub use host_geometry::*;
pub use host_semantics::*;
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
    AudioArtboardId, AudioDecodeError, AudioEngine, AudioEngineError, AudioFormat, AudioProcessor,
    AudioReader, AudioSound, AudioSource, OscillatorConfig, ProcessorError, ProcessorMeter,
    ProcessorState, ProcessorStatus, ProcessorUpdate,
};
pub use nuxie_render_api::{Mat2D, PersistentFactoryContext, Renderer};

#[cfg(test)]
mod host_public_api_tests;

#[cfg(test)]
mod frame_probe_test;
