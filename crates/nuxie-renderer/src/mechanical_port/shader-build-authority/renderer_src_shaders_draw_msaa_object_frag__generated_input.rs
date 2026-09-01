//! Exact generated-input translation of renderer/src/shaders/draw_msaa_object.frag.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "3ed35ee0ded0d58fb8d380930a156041a4624a2f";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_msaa_object.frag";
pub const PINNED_SOURCE_SHA256: &str = "3b61972533dfebe2c908d98ef42b50c615d4ead4115fecc43a53cca6007de64f";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_msaa_object";
pub const PINNED_SOURCE_LINE_COUNT: usize = 110;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 3616;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_msaa_object_frag__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
