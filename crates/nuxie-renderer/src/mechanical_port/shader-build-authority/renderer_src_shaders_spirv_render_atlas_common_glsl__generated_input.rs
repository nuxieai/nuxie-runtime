//! Exact generated-input translation of renderer/src/shaders/spirv/render_atlas_common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/render_atlas_common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "95ffda17e87a1a09be1be4440e6905ae67ae9083fcb3d6467ca603d2e8494cd7";
pub const OWNERSHIP_UNIT: &str = "shader:source:render_atlas_common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 11;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 380;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_render_atlas_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
