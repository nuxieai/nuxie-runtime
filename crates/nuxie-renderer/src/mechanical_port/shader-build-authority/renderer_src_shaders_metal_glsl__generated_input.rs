//! Exact generated-input translation of renderer/src/shaders/metal.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "966499fffe2aadcbcd1fe4388160e4e7d5c0d967";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal.glsl";
pub const PINNED_SOURCE_SHA256: &str = "c95bc053c61db72e1709209dda94b609a5837bf9e7b61b7a171434c97d04bc3d";
pub const OWNERSHIP_UNIT: &str = "shader:source:metal";
pub const PINNED_SOURCE_LINE_COUNT: usize = 534;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 27098;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_metal_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
