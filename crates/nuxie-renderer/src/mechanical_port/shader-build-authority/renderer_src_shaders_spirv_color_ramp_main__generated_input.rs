//! Exact generated-input translation of renderer/src/shaders/spirv/color_ramp.main.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv/color_ramp.main";
pub const PINNED_SOURCE_SHA256: &str = "076d5d45b3af0d27a138f41ac6fc32fd7411384cccc74c9a7b85c2cf376ea2e5";
pub const OWNERSHIP_UNIT: &str = "shader:source:color_ramp";
pub const PINNED_SOURCE_LINE_COUNT: usize = 7;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 235;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_spirv_color_ramp_main__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
