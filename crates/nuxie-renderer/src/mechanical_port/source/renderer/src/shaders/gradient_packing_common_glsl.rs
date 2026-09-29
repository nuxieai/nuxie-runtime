//! Exact upstream gradient packing shader program.
#![allow(dead_code)]
pub const PINNED_UPSTREAM_COMMIT: &str = "9463ff7b5b9a1452d0c32e41390a99cd39b6c946";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/gradient_packing_common.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "2fd7fe601521a8ee83c8c6ff8bc35537607dae26dfc64143578ef130e525c68d";
pub const PINNED_SOURCE_LINE_COUNT: usize = 69;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2230;
pub const PINNED_GRADIENT_PACKING_COMMON_SOURCE: &str = r###"#ifdef @VERTEX

// We pack the following into the result:
//  xy = gradient coordinate
//  z  = 2 for a complex gradient, otherwise the left texel x of the ramp.
//       (z will be negative for radial gradients)
//  w  = the y coordinate within the texture (will never be 0)
INLINE float4 packGradientData(float2 fragCoord,
                               float2x2 mat,
                               float2 translate,
                               float type,
                               float2 hSpan,
                               float y)
{
    float4 gradient;
    gradient.w = y;
    float2 gradientCoord = MUL(mat, fragCoord) + translate;

    float gradientSpan = hSpan.x;
    // gradientSpan is either ~1 (for a complex gradient that spans nearly
    // the whole width of the texture) or 1/GRAD_TEXTURE_WIDTH (a single
    // pixel)
    if (gradientSpan > 0.9)
    {
        // Complex ramps span the whole row - 2.0 is a special value to
        // convey this.
        gradient.z = 2.0;
    }
    else
    {
        // Otherwise a simple ramp gets the left texel x of the ramp
        gradient.z = hSpan.y;
    }

    if (type == float(LINEAR_GRADIENT_PAINT_TYPE))
    {
        // This is a linear gradient.
        gradient.x = gradientCoord.x;
        gradient.y = 0.0;
    }
    else
    {
        // Making this negative lets the fragment shader know this is a
        // radial gradient (its value will never be 0 because the ramp is on
        // pixel centers)
        gradient.z = -gradient.z;
        gradient.xy = gradientCoord;
    }

    return gradient;
}
#endif

#ifdef @FRAGMENT
// Given packed gradient information, this gets the UV coordinate for sampling
// the gradient texture.
INLINE float2 getGradientCoord(float4 gradient)
{
    float t = gradient.z > 0.0 ? /*linear*/ gradient.x
                               : /*radial*/ length(gradient.xy);
    t = clamp(t, 0.0, 1.0);
    float span = abs(gradient.z);
    float x = span > 1.0 ? /*entire row*/ (1.0 - 1.0 / GRAD_TEXTURE_WIDTH) * t +
                               (0.5 / GRAD_TEXTURE_WIDTH)
                         : /*two texels*/ (1.0 / GRAD_TEXTURE_WIDTH) * t + span;
    float row = gradient.w;
    return float2(x, row);
}
#endif"###;
pub const fn pinned_source() -> &'static str {
    PINNED_GRADIENT_PACKING_COMMON_SOURCE
}
