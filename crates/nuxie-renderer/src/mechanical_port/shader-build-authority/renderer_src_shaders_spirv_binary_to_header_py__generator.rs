//! Complete build-time owner for renderer/src/shaders/spirv_binary_to_header.py.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/spirv_binary_to_header.py";
pub const PINNED_SOURCE_SHA256: &str =
    "07f4b6f91bade4af1dcc62024447982e08ce9af671155e047ac1185460dc79d0";
pub const PINNED_SOURCE_LINE_COUNT: usize = 28;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 992;
pub const PINNED_SOURCE: &[u8] =
    include_bytes!("source/renderer_src_shaders_spirv_binary_to_header_py.source");
pub const INPUT_WORD_ENDIANNESS: &str = "little-endian u32";
pub const WORDS_PER_OUTPUT_LINE: usize = 8;
pub const OUTPUT_STAGE: &str = "spirv-header";
pub const FAILURE_CONTRACT: &str = "Require exactly three arguments and input byte length divisible by four; preserve the source's failing printf name on the malformed-length branch rather than silently accepting it.";
pub const EMISSION_CONTRACT: &str = "Emit pragma once, const uint32_t <array_name>[], eight zero-padded hex words per indented line, a trailing comma per word, and the exact closing brace/newline.";

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
