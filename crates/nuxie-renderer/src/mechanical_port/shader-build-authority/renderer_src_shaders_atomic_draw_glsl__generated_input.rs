//! Exact generated-input translation of renderer/src/shaders/atomic_draw.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "7732f41ef93e4cb74286934ee596e1041d0a0ba7";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/atomic_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str = "65732d116a16d4c560fc565b54d06a50408d4e37c12c039c89180427742d5ad6";
pub const OWNERSHIP_UNIT: &str = "shader:source:atomic_draw";
pub const PINNED_SOURCE_LINE_COUNT: usize = 1160;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 39896;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_atomic_draw_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
