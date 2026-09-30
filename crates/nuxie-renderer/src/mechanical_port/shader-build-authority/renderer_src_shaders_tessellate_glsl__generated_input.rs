//! Exact generated-input translation of renderer/src/shaders/tessellate.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "3b615b829a58b67379f9304a161b3e87119bbf04";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/tessellate.glsl";
pub const PINNED_SOURCE_SHA256: &str = "3edcd4077a7fcc5d3f619c1ad280d05bb2b7c9bc418e06ba860e52874e9c5a24";
pub const OWNERSHIP_UNIT: &str = "shader:source:tessellate";
pub const PINNED_SOURCE_LINE_COUNT: usize = 587;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 25857;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_tessellate_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
