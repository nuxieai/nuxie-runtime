//! Exact generated-input translation of renderer/src/shaders/rhi.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "c14cb2510071bd4cfa08d52ba5cd44d98c362237";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/rhi.glsl";
pub const PINNED_SOURCE_SHA256: &str = "1762e95a0cac23926472b303350cd06f5fee876980076050f129c850824d50ac";
pub const OWNERSHIP_UNIT: &str = "shader:source:rhi";
pub const PINNED_SOURCE_LINE_COUNT: usize = 610;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 25399;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_rhi_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
