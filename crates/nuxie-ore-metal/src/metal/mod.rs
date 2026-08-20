//! Concrete ORE Metal resource payloads.

pub mod bind_group;
pub mod buffer;
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod context;
pub mod pipeline;
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod render_pass;
pub mod sampler;
pub mod shader_module;
pub mod texture;

/// Shared backend identity for every native Metal resource downcast.
pub enum MetalBackend {}
