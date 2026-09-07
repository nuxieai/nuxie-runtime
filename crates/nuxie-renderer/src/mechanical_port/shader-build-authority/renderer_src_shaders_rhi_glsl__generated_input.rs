//! Exact generated-input translation of renderer/src/shaders/rhi.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ad6fcf47526b033e5cbe16275e9219365551d76";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/rhi.glsl";
pub const PINNED_SOURCE_SHA256: &str = "b246364cde3d724f5d4b9682cff39ae17845011a0ebc2151f97911422fce0b9d";
pub const OWNERSHIP_UNIT: &str = "shader:source:rhi";
pub const PINNED_SOURCE_LINE_COUNT: usize = 590;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 24414;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_rhi_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
