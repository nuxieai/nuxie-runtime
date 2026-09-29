//! Complete source-owner translation of renderer/src/shaders/Makefile.

#![allow(dead_code)]

#[path = "../source/renderer/src/shaders/minify_py.rs"]
mod minify_py;
#[path = "../source/renderer/src/shaders/makefile.rs"]
pub mod executable_translation;

pub const PINNED_UPSTREAM_COMMIT: &str = "c18b32511bfeaeee6b7c54e35152aea3fdbb5964";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/Makefile";
pub const PINNED_SOURCE_SHA256: &str =
    "5b7af29c582e5731f07ba5a4f623cc33469a7e8332242e4cabf6384ce851b9f7";
pub const PINNED_SOURCE_LINE_COUNT: usize = 502;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 22173;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_makefile.source");
pub const FROZEN_GENERATED_ARTIFACT_COUNT: usize = 520;
pub const EXECUTION_CONTRACT: &str = "Execute the independently compiled complete Rust translation or the pinned upstream Makefile with the frozen toolchain; compare emitted bytes with the retained shader inputs and backend import checksums before replacing artifacts.";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
