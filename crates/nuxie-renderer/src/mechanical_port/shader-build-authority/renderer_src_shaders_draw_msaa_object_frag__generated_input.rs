//! Exact generated-input translation of renderer/src/shaders/draw_msaa_object.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_msaa_object.frag";
pub const PINNED_SOURCE_SHA256: &str = "28ec08b53f7f32a12439d5f85481c4f5d66660f0e80e867b44fcbd35adba9d85";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_msaa_object";
pub const PINNED_SOURCE_LINE_COUNT: usize = 103;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3424;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_msaa_object_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
