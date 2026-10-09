//! Exact generated-input translation of renderer/src/shaders/spirv/atomic_base.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/atomic_base.glsl";
pub const PINNED_SOURCE_SHA256: &str = "9c01a4730239527c2f7303b569c6a68202d8559070acf855c0818bdfd9d900c9";
pub const OWNERSHIP_UNIT: &str = "shader:source:atomic_base";
pub const PINNED_SOURCE_LINE_COUNT: usize = 20;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 707;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_atomic_base_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
