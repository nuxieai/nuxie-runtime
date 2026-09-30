//! Exact generated-input translation of renderer/src/shaders/common.glsl.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "5705446d6aeb0dad34a63d8ddadbb79fbe327a37";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/common.glsl";
pub const PINNED_SOURCE_SHA256: &str = "d687e007cf2ce52420bbec02e677e53ef27984b7c5b9ed8f05423ace31b1bf92";
pub const OWNERSHIP_UNIT: &str = "shader:source:common";
pub const PINNED_SOURCE_LINE_COUNT: usize = 479;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 16063;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_common_glsl__generated_input.source");

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
