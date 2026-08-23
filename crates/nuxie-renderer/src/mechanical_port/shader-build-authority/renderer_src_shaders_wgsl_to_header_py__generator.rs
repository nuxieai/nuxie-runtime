//! Complete build-time owner for renderer/src/shaders/wgsl_to_header.py.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/wgsl_to_header.py";
pub const PINNED_SOURCE_SHA256: &str =
    "74d8f360cda602df5dac16664a5175d6ddae35d360728e22905373175857e7e7";
pub const PINNED_SOURCE_LINE_COUNT: usize = 188;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 7_593;
pub const PINNED_SOURCE: &[u8] =
    include_bytes!("source/renderer_src_shaders_wgsl_to_header_py.source");
pub const SAFE_NAME_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
pub const RAW_STRING_DELIMITER: &str = "WGSL";
pub const OUTPUT_STAGE: &str = "wgsl-header";
pub const TRANSFORM_ORDER: &[&str] = &[
    "derive label from .wgsl basename",
    "read SPECIALIZATION_COUNT from sibling constants.glsl",
    "rewrite flat[,first] interpolation to flat,either",
    "unless --raw, strip blank lines and punctuation whitespace",
    "unless --raw, frequency-sort and bijective-base62 rename only Naga identifiers when shorter",
    "collect and range-check reachable @id override indices",
    "reject embedded raw-string terminator",
    "emit guarded Shader struct and exact source/usedOverrides/label initializer",
];
pub const FAILURE_CONTRACTS: &[&str] = &[
    "missing SPECIALIZATION_COUNT exits 1",
    "out-of-range reachable override exits 1",
    "embedded )WGSL raw-string terminator exits 1",
];

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
const _: [(); 8] = [(); TRANSFORM_ORDER.len()];
const _: [(); 3] = [(); FAILURE_CONTRACTS.len()];
