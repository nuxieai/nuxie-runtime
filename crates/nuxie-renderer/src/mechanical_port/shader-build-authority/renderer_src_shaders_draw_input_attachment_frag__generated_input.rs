//! Exact generated-input translation of renderer/src/shaders/draw_input_attachment.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_input_attachment.frag";
pub const PINNED_SOURCE_SHA256: &str = "8af2574495c71e8282116b3e598daf5e1c705d0b50c39d2439ad18e6be1e8694";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_input_attachment";
pub const PINNED_SOURCE_LINE_COUNT: usize = 18;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 392;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_input_attachment_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
