//! Exact generated-input translation of renderer/src/shaders/metal.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "2210ed8799c0128504dd664a7179f4f8f299e85a";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal.glsl";
pub const PINNED_SOURCE_SHA256: &str = "d9f1e0790e5df593bf91794f68f30120c39fcbbed39df1e43f086f0ac5c8aee4";
pub const OWNERSHIP_UNIT: &str = "shader:source:metal";
pub const PINNED_SOURCE_LINE_COUNT: usize = 534;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 27098;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_metal_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
