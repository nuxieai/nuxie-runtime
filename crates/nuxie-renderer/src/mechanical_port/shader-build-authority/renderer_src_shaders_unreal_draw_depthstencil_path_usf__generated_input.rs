//! Exact generated-input translation of renderer/src/shaders/unreal/draw_depthstencil_path.usf.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/unreal/draw_depthstencil_path.usf";
pub const PINNED_SOURCE_SHA256: &str = "3e8785853676125abdbfe5fa94b2ba22496c966b18fcf9579da07ffa44ccbfa0";
pub const OWNERSHIP_UNIT: &str = "shader:source:unreal_draw_depthstencil_path";
pub const PINNED_SOURCE_LINE_COUNT: usize = 25;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 680;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_unreal_draw_depthstencil_path_usf__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
