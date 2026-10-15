//! Exact generated-input translation of renderer/src/shaders/rhi.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "ce891f526a3297f9498a3e524fce6385cf0f2f25";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/rhi.glsl";
pub const PINNED_SOURCE_SHA256: &str = "737c9bca6c6cbb0e81c91125088cd3cac7ba658ff326f28cced3a097efb1ee68";
pub const OWNERSHIP_UNIT: &str = "shader:source:rhi";
pub const PINNED_SOURCE_LINE_COUNT: usize = 617;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 25550;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_rhi_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
