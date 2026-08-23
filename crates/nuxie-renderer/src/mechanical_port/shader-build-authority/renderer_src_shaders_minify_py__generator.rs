//! Complete source-owner translation of renderer/src/shaders/minify.py.

#![allow(dead_code)]

#[path = "../source/renderer/src/shaders/minify_py.rs"]
pub mod executable_translation;

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/minify.py";
pub const PINNED_SOURCE_SHA256: &str =
    "bf4b9f529a19765c5e6f28b68ef8a73f5bd65433cd87ce723df5df923e6bc22b";
pub const PINNED_SOURCE_LINE_COUNT: usize = 642;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 33_034;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_minify_py.source");
pub const OUTPUT_STAGES: &[&str] = &["minify-export", "minify-header"];
pub const EXECUTION_CONTRACT: &str = "The independently compiled complete Rust translation retains argument parsing, lexer, preprocessor, symbol renaming, source emission, and failure behavior; pinned Python output remains the generated-byte oracle.";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
