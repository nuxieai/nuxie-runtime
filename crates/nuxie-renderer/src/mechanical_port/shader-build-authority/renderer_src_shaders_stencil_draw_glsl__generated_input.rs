//! Exact generated-input translation of renderer/src/shaders/stencil_draw.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/stencil_draw.glsl";
pub const PINNED_SOURCE_SHA256: &str = "9df944e40e0f66f0a7f4e2114fe2644d426a44bc236eab969e3bdf75bb70c0bd";
pub const OWNERSHIP_UNIT: &str = "shader:source:stencil_draw";
pub const PINNED_SOURCE_LINE_COUNT: usize = 31;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 763;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_stencil_draw_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
