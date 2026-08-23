//! Exact generated-input translation of renderer/src/shaders/spirv/atomic_draw_path.main.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/atomic_draw_path.main";
pub const PINNED_SOURCE_SHA256: &str = "03327dcdaa878b265f76d721cb321e047f87e5df4e8b40dc4cb923623e4ab99b";
pub const OWNERSHIP_UNIT: &str = "shader:source:atomic_draw_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 4;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 108;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_atomic_draw_path_main__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
