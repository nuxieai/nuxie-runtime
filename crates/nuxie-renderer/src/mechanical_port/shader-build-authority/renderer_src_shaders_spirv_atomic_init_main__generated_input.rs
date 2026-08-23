//! Exact generated-input translation of renderer/src/shaders/spirv/atomic_init.main.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/atomic_init.main";
pub const PINNED_SOURCE_SHA256: &str = "c9c707cb27960486c50e5b5fe5686c1b831b222682b3e3a49dcb631652abafba";
pub const OWNERSHIP_UNIT: &str = "shader:source:atomic_init";
pub const PINNED_SOURCE_LINE_COUNT: usize = 5;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 154;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_atomic_init_main__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
