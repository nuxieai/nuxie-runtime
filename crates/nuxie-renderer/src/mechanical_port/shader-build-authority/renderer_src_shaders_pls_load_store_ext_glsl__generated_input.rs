//! Exact generated-input translation of renderer/src/shaders/pls_load_store_ext.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/pls_load_store_ext.glsl";
pub const PINNED_SOURCE_SHA256: &str = "39d167247268280cac6bbf5d9febdd30fea9fcf1bce5016eca1170e4544feb82";
pub const OWNERSHIP_UNIT: &str = "shader:source:pls_load_store_ext";
pub const PINNED_SOURCE_LINE_COUNT: usize = 105;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2218;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_pls_load_store_ext_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
