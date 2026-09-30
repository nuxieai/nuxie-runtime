//! Complete source-owner translation of renderer/src/shaders/Makefile.

#![allow(dead_code)]

#[path = "../source/renderer/src/shaders/makefile.rs"]
pub mod executable_translation;
#[path = "../source/renderer/src/shaders/minify_py.rs"]
mod minify_py;

pub const PINNED_UPSTREAM_COMMIT: &str = "c14cb2510071bd4cfa08d52ba5cd44d98c362237";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/Makefile";
pub const PINNED_SOURCE_SHA256: &str =
    "85dca2a94cc9fbeab7dfac7539b8ca2da8371dacdcc3d2e0e1e8f095387cca21";
pub const PINNED_SOURCE_LINE_COUNT: usize = 527;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 23204;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_makefile.source");
pub const FROZEN_GENERATED_ARTIFACT_COUNT: usize = 526;
pub const EXECUTION_CONTRACT: &str = "Execute the independently compiled complete Rust translation or the pinned upstream Makefile with the frozen toolchain; compare emitted bytes with the retained shader inputs and backend import checksums before replacing artifacts.";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
