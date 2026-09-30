//! Exact generated-input translation of renderer/src/shaders/tessellate.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "5705446d6aeb0dad34a63d8ddadbb79fbe327a37";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/tessellate.glsl";
pub const PINNED_SOURCE_SHA256: &str = "88708289263a011612a54effd01533cf6593bf93b016315d240de7b4bbfa48c2";
pub const OWNERSHIP_UNIT: &str = "shader:source:tessellate";
pub const PINNED_SOURCE_LINE_COUNT: usize = 567;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 24765;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_tessellate_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
