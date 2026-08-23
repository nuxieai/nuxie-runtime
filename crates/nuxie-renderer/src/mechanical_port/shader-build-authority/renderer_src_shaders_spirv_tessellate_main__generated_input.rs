//! Exact generated-input translation of renderer/src/shaders/spirv/tessellate.main.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/tessellate.main";
pub const PINNED_SOURCE_SHA256: &str = "9a50cb20894f19978864152d4852293a2052cba80573b80246dbce5f2f5a55a0";
pub const OWNERSHIP_UNIT: &str = "shader:source:tessellate";
pub const PINNED_SOURCE_LINE_COUNT: usize = 9;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 331;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_tessellate_main__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
