//! Concrete ORE Metal resource payloads.

pub mod sampler;
pub mod shader_module;
pub mod texture;

/// Shared backend identity for every native Metal resource downcast.
pub enum MetalBackend {}
