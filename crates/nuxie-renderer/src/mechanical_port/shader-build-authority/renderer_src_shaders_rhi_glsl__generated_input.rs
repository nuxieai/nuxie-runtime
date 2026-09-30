//! Exact generated-input translation of renderer/src/shaders/rhi.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "b86b7ecb0256842cc37823f63c8699d5bffe081e";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/rhi.glsl";
pub const PINNED_SOURCE_SHA256: &str = "685dde8c6a3520bf8df87c21bb167e7200e3b4e1d246d1834ded658dcba701d3";
pub const OWNERSHIP_UNIT: &str = "shader:source:rhi";
pub const PINNED_SOURCE_LINE_COUNT: usize = 608;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 25248;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_rhi_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
