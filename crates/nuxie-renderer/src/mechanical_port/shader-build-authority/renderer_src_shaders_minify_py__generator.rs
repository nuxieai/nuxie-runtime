//! Complete source-owner translation of renderer/src/shaders/minify.py.

#![allow(dead_code)]

#[path = "../source/renderer/src/shaders/minify_py.rs"]
pub mod executable_translation;

pub const PINNED_UPSTREAM_COMMIT: &str = "7d59acedbf37270e538a3093ec777f5b66b7ffb9";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/minify.py";
pub const PINNED_SOURCE_SHA256: &str =
    "1144ea77fb55558c8e51c2536888b492bf4235add068c3bf41887e5946eae897";
pub const PINNED_SOURCE_LINE_COUNT: usize = 565;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 24573;
pub const PINNED_SOURCE: &[u8] = include_bytes!("source/renderer_src_shaders_minify_py.source");
pub const OUTPUT_STAGES: &[&str] = &[
    "minify-export",
    "minified-source:<root>.minified<ext>",
    "minify-header",
];
pub const EXECUTION_CONTRACT: &str = "The independently compiled complete Rust translation retains argument parsing, lexer, preprocessor, symbol renaming, source emission, and failure behavior; pinned Python output remains the generated-byte oracle.";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
