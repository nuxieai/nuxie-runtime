//! Exact generated-input translation of renderer/src/shaders/draw_image_mesh.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "f40c9dfe8a0c4accf3e963f48429e893798854a5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_image_mesh.vert";
pub const PINNED_SOURCE_SHA256: &str = "a657670a0b00b3dded6c4250eb501da2c51739b3cada654cbde90d56d037cee2";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_image_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 135;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 4405;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_image_mesh_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
