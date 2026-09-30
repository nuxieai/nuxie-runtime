//! Exact generated-input translation of renderer/src/shaders/common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "57dddb3727306e284773ec20c653cf686c45abee";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "890749dc58a9dc4fe18c9242dd852abe198f2b130362c20744524b9d045f201f";
pub const OWNERSHIP_UNIT: &str = "shader:source:common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 525;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 17962;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
