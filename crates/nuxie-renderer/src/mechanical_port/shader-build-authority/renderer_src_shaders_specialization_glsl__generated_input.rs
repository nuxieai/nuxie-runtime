//! Exact generated-input translation of renderer/src/shaders/specialization.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "2b2203f45a67f813cb662272962192ecfdfd923e";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/specialization.glsl";
pub const PINNED_SOURCE_SHA256: &str = "186ab355b6e3cd6321fa017fd6dd55102c52538526bd3c84595304583781c77c";
pub const OWNERSHIP_UNIT: &str = "shader:source:specialization";
pub const PINNED_SOURCE_LINE_COUNT: usize = 57;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2745;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_specialization_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
