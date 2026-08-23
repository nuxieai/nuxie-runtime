/*
 * Copyright 2026 Rive
 */

// Mechanical translation of the complete pinned source header
// decoders/include/rive/decoders/astc_footprints.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// #ifndef _RIVE_ASTC_FOOTPRINTS_HPP_
// #define _RIVE_ASTC_FOOTPRINTS_HPP_

// #include <cstdint>
// `uint8_t` is represented by Rust's fixed-width `u8` below.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// namespace rive
// {

// LDR ASTC block footprints in canonical (Vulkan / KHR_ldr) spec order. The
// index into this table also indexes the corresponding GPU enums:
//   VkFormat (UNORM) = VK_FORMAT_ASTC_4x4_UNORM_BLOCK (157) + 2 * idx
//   VkFormat (SRGB)  = UNORM + 1
//   GL enum  (UNORM) = 0x93B0 + idx
//   GL enum  (SRGB)  = 0x93D0 + idx
// struct AstcFootprint
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AstcFootprint {
    // uint8_t width;
    pub width: u8,
    // uint8_t height;
    pub height: u8,
}

// constexpr AstcFootprint AstcFootprints[] = {
//     {4, 4},
//     {5, 4},
//     {5, 5},
//     {6, 5},
//     {6, 6},
//     {8, 5},
//     {8, 6},
//     {8, 8},
//     {10, 5},
//     {10, 6},
//     {10, 8},
//     {10, 10},
//     {12, 10},
//     {12, 12},
// };
pub static AstcFootprints: [AstcFootprint; 14] = [
    AstcFootprint {
        width: 4,
        height: 4,
    },
    AstcFootprint {
        width: 5,
        height: 4,
    },
    AstcFootprint {
        width: 5,
        height: 5,
    },
    AstcFootprint {
        width: 6,
        height: 5,
    },
    AstcFootprint {
        width: 6,
        height: 6,
    },
    AstcFootprint {
        width: 8,
        height: 5,
    },
    AstcFootprint {
        width: 8,
        height: 6,
    },
    AstcFootprint {
        width: 8,
        height: 8,
    },
    AstcFootprint {
        width: 10,
        height: 5,
    },
    AstcFootprint {
        width: 10,
        height: 6,
    },
    AstcFootprint {
        width: 10,
        height: 8,
    },
    AstcFootprint {
        width: 10,
        height: 10,
    },
    AstcFootprint {
        width: 12,
        height: 10,
    },
    AstcFootprint {
        width: 12,
        height: 12,
    },
];

// constexpr int AstcFootprintCount =
//     sizeof(AstcFootprints) / sizeof(AstcFootprints[0]);
// The source `constexpr int` remains signed and fixed-width as `i32`; the
// array-length calculation preserves the source element-count expression.
pub const AstcFootprintCount: i32 =
    (core::mem::size_of_val(&AstcFootprints) / core::mem::size_of::<AstcFootprint>()) as i32;

// Returns -1 if (blockWidth, blockHeight) is not a recognised LDR ASTC
// footprint.
// inline int astcFootprintIndex(uint8_t blockWidth, uint8_t blockHeight)
#[inline]
pub fn astcFootprintIndex(blockWidth: u8, blockHeight: u8) -> i32 {
    // for (int i = 0; i < AstcFootprintCount; ++i)
    for i in 0..AstcFootprintCount {
        // if (AstcFootprints[i].width == blockWidth &&
        //     AstcFootprints[i].height == blockHeight)
        let footprint = &AstcFootprints[i as usize];
        if footprint.width == blockWidth && footprint.height == blockHeight {
            // return i;
            return i;
        }
    }
    // return -1;
    -1
}

// } // namespace rive

// #endif
