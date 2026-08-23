//! Exact generated-input translation of renderer/src/shaders/spirv/render_atlas.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/render_atlas.vert";
pub const PINNED_SOURCE_SHA256: &str = "109e973cde4ace814cc88b472a9b3c1d85152a85b0068108027b9fbecfd552cb";
pub const OWNERSHIP_UNIT: &str = "shader:source:render_atlas";
pub const PINNED_SOURCE_LINE_COUNT: usize = 3;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 98;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_render_atlas_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
