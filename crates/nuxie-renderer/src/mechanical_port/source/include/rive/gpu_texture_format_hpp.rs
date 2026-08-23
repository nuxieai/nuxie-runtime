// Mechanical translation of the complete pinned source header
// include/rive/gpu_texture_format.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// #pragma once

// #include <cstdint>
// `uint8_t` is represented by Rust's fixed-width `u8` below.

// namespace rive
// {

// Enum describing the format of a GPU-uploadable texture. Only formats which
// the GPU samples directly are listed. Lives at the `rive/` root so both the
// decoders library (which produces these tags) and the renderer (which
// consumes them) can include it without cross-layer dependencies.
// enum class GPUTextureFormat : uint8_t
// {
//     rgba32,
//     bc1,
//     bc2,
//     bc3,
//     bc7,
//     astc,
//     etc2,
// };
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GPUTextureFormat {
    // The source has implicit sequential discriminants; they are explicit in
    // Rust to preserve the serialized/upload tag values.
    rgba32 = 0,
    bc1 = 1,
    bc2 = 2,
    bc3 = 3,
    bc7 = 4,
    astc = 5,
    etc2 = 6,
}

// } // namespace rive
