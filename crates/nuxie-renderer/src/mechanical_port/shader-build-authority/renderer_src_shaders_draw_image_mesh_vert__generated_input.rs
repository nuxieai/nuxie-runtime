//! Exact generated-input translation of renderer/src/shaders/draw_image_mesh.vert.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "9463ff7b5b9a1452d0c32e41390a99cd39b6c946";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_image_mesh.vert";
pub const PINNED_SOURCE_SHA256: &str = "b0fff7fe9498a42d90faebbfc58bc3242b73e9fc18ca824e1e0d35bb882af19d";
pub const OWNERSHIP_UNIT: &str = "shader:source:draw_image_mesh";
pub const PINNED_SOURCE_LINE_COUNT: usize = 132;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 4176;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_draw_image_mesh_vert__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
