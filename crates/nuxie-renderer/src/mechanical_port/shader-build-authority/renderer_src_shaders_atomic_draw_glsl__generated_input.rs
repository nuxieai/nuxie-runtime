//! Exact generated-input translation of renderer/src/shaders/atomic_draw.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/atomic_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str = "fad587733e5990e4ba77e194326dacaf27022026f9621e58a1aac2c131935849";
pub const OWNERSHIP_UNIT: &str = "shader:source:atomic_draw";
pub const PINNED_SOURCE_LINE_COUNT: usize = 1104;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 37201;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_atomic_draw_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
