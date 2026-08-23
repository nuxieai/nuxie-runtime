//! Exact generated-input translation of renderer/src/shaders/rhi.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/rhi.glsl";
pub const PINNED_SOURCE_SHA256: &str = "b08bc3a81cd7e88eb82ffba447fd073630aaa51f996641e8f7cd367678617f96";
pub const OWNERSHIP_UNIT: &str = "shader:source:rhi";
pub const PINNED_SOURCE_LINE_COUNT: usize = 560;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 22901;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_rhi_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
