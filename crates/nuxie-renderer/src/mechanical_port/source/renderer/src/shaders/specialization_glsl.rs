/*
 * Exact pinned upstream source bytes and provenance for
 * renderer/src/shaders/specialization.glsl.
 *
 * Upstream source revision: c73593c3a868f87b5dcd328432255db33e3e2266
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "c73593c3a868f87b5dcd328432255db33e3e2266";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/specialization.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "afd2d84e69cc09c49c66e524f89d0ad23435daf6e559b5142c0cef8a2de44add";
pub const PINNED_SOURCE_LINE_COUNT: usize = 52;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2493;

/// Exact pinned upstream source bytes.
pub const PINNED_SPECIALIZATION_GLSL_SOURCE: &str = r###"layout(constant_id = CLIPPING_SPECIALIZATION_IDX) const
    bool EnableClipping = true;
layout(constant_id = CLIP_RECT_SPECIALIZATION_IDX) const
    bool EnableClipRect = true;
layout(constant_id = ADVANCED_BLEND_SPECIALIZATION_IDX) const
    bool EnableAdvancedBlend = true;
layout(constant_id = FEATHER_SPECIALIZATION_IDX) const
    bool EnableFeather = true;
layout(constant_id = EVEN_ODD_SPECIALIZATION_IDX) const
    bool EnableEvenOdd = true;
layout(constant_id = NESTED_CLIPPING_SPECIALIZATION_IDX) const
    bool EnableNestedClipping = true;
layout(constant_id = HSL_BLEND_MODES_SPECIALIZATION_IDX) const
    bool EnableHSLBlendModes = true;
layout(constant_id = DITHER_SPECIALIZATION_IDX) const bool EnableDither = true;
layout(constant_id = MODULATED_IMAGE_SPECIALIZATION_IDX) const
    bool EnableModulatedImage = true;
layout(constant_id = CLOCKWISE_FILL_SPECIALIZATION_IDX) const
    bool ClockwiseFill = true;
layout(constant_id = NESTED_CLIP_UPDATE_ONLY_SPECIALIZATION_IDX) const
    bool NestedClipUpdateOnly = false;
layout(constant_id = BORROWED_COVERAGE_PASS_SPECIALIZATION_IDX) const
    bool BorrowedCoveragePrepass = false;
layout(constant_id = STORE_COLOR_CLEAR_SPECIALIZATION_IDX) const
    bool StoreColorClear = false;
layout(constant_id = LOAD_COLOR_FROM_DST_TEXTURE_SPECIALIZATION_IDX) const
    bool LoadColorFromDstTexture = false;
layout(constant_id = VULKAN_VENDOR_ARM_SPECIALIZATION_IDX) const
    bool VulkanVendorARM = false;
layout(constant_id = DS_POLAR_STROKE_SPECIALIZATION_IDX) const
    bool DSPolarStroke = false;
layout(constant_id = DS_HAIRLINE_STROKE_SPECIALIZATION_IDX) const
    bool DSHairlineStroke = false;

#define @ENABLE_CLIPPING EnableClipping
#define @ENABLE_CLIP_RECT EnableClipRect
#define @ENABLE_ADVANCED_BLEND EnableAdvancedBlend
#define @DISABLE_ADVANCED_BLEND DisableAdvancedBlend
#define @ENABLE_FEATHER EnableFeather
#define @ENABLE_EVEN_ODD EnableEvenOdd
#define @ENABLE_NESTED_CLIPPING EnableNestedClipping
#define @ENABLE_HSL_BLEND_MODES EnableHSLBlendModes
#define @ENABLE_DITHER EnableDither
#define @ENABLE_MODULATED_IMAGE EnableModulatedImage
#define @CLOCKWISE_FILL ClockwiseFill
#define @NESTED_CLIP_UPDATE_ONLY NestedClipUpdateOnly
#define @BORROWED_COVERAGE_PASS BorrowedCoveragePrepass
#define @STORE_COLOR_CLEAR StoreColorClear
#define @LOAD_COLOR_FROM_DST_TEXTURE LoadColorFromDstTexture
#define @VULKAN_VENDOR_ARM VulkanVendorARM
#define @DS_POLAR_STROKE DSPolarStroke
#define @DS_HAIRLINE_STROKE DSHairlineStroke
"###;

/// Stable source aliases.
pub const PINNED_SPECIALIZATION_SOURCE: &str = PINNED_SPECIALIZATION_GLSL_SOURCE;
pub const SPECIALIZATION_GLSL_SOURCE: &str = PINNED_SPECIALIZATION_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_SPECIALIZATION_GLSL_SOURCE
}
