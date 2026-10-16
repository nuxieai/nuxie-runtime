//! Exact generated-input translation of renderer/src/shaders/tessellate.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "1c75867de17587c541ad2d7ca88b10577b033bd2";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/tessellate.glsl";
pub const PINNED_SOURCE_SHA256: &str = "1899b1bda75733ac02a97b1169969ee76e7cc5edc09ba2bd5c580a619155018d";
pub const OWNERSHIP_UNIT: &str = "shader:source:tessellate";
pub const PINNED_SOURCE_LINE_COUNT: usize = 618;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 27327;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_tessellate_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
