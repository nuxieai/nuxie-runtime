//! Exact generated-input translation of renderer/src/shaders/resolve_atlas.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/resolve_atlas.glsl";
pub const PINNED_SOURCE_SHA256: &str = "a31d945c9b29dd4ba74cff3c9c9010e108f5cd82bb0b82474b199725e59aa04f";
pub const OWNERSHIP_UNIT: &str = "shader:source:resolve_atlas";
pub const PINNED_SOURCE_LINE_COUNT: usize = 93;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2615;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_resolve_atlas_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
