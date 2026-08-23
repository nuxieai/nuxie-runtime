/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/advanced_blend.glsl.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger branches, exports, functions, and dependencies as literal
 * source-shaped data. It does not compile, evaluate, simplify, or generate
 * shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/advanced_blend.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "d7f8d9cec8e095c7e6d331a9f3ba48cdb18ea63f961d9223e3dfc509bcd8794b";
pub const PINNED_SOURCE_LINE_COUNT: usize = 330;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 13219;

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_ADVANCED_BLEND_GLSL_SOURCE: &str = r###"/*
 * Copyright 2022 Rive
 */

// From the KHR_blend_equation_advanced spec:
//
//    The advanced blend equations are those listed in tables X.1 and X.2.  When
//    using one of these equations, blending is performed according to the
//    following equations:
//
//      R = f(Rs',Rd')*p0(As,Ad) + Y*Rs'*p1(As,Ad) + Z*Rd'*p2(As,Ad)
//      G = f(Gs',Gd')*p0(As,Ad) + Y*Gs'*p1(As,Ad) + Z*Gd'*p2(As,Ad)
//      B = f(Bs',Bd')*p0(As,Ad) + Y*Bs'*p1(As,Ad) + Z*Bd'*p2(As,Ad)
//      A =          X*p0(As,Ad) +     Y*p1(As,Ad) +     Z*p2(As,Ad)
//
//    where the function f and terms X, Y, and Z are specified in the table.
//    The R, G, and B components of the source color used for blending are
//    considered to have been premultiplied by the A component prior to
//    blending.  The base source color (Rs',Gs',Bs') is obtained by dividing
//    through by the A component:
//
//      (Rs', Gs', Bs') =
//        (0, 0, 0),              if As == 0
//        (Rs/As, Gs/As, Bs/As),  otherwise
//
//    The destination color components are always considered to have been
//    premultiplied by the destination A component and the base destination
//    color (Rd', Gd', Bd') is obtained by dividing through by the A component:
//
//      (Rd', Gd', Bd') =
//        (0, 0, 0),               if Ad == 0
//        (Rd/Ad, Gd/Ad, Bd/Ad),   otherwise
//
//    When blending using advanced blend equations, we expect that the R, G, and
//    B components of premultiplied source and destination color inputs be
//    stored as the product of non-premultiplied R, G, and B components and the
//    A component of the color.  If any R, G, or B component of a premultiplied
//    input color is non-zero and the A component is zero, the color is
//    considered ill-formed, and the corresponding component of the blend result
//    will be undefined.
//
//    The weighting functions p0, p1, and p2 are defined as follows:
//
//      p0(As,Ad) = As*Ad
//      p1(As,Ad) = As*(1 - Ad)
//      p2(As,Ad) = Ad*(1 - As)
//
//    In these functions, the A components of the source and destination colors
//    are taken to indicate the portion of the pixel covered by the fragment
//    (source) and the fragments previously accumulated in the pixel
//    (destination).  The functions p0, p1, and p2 approximate the relative
//    portion of the pixel covered by the intersection of the source and
//    destination, covered only by the source, and covered only by the
//    destination, respectively.  The equations defined here assume that there
//    is no correlation between the source and destination coverage.
//

#ifdef @FRAGMENT

#ifdef @ENABLE_KHR_BLEND
layout(
#ifdef @ENABLE_HSL_BLEND_MODES
    blend_support_all_equations
#else
    blend_support_multiply,
    blend_support_screen,
    blend_support_overlay,
    blend_support_darken,
    blend_support_lighten,
    blend_support_colordodge,
    blend_support_colorburn,
    blend_support_hardlight,
    blend_support_softlight,
    blend_support_difference,
    blend_support_exclusion
#endif
    ) out;
#endif // ENABLE_KHR_BLEND

#ifdef @ENABLE_ADVANCED_BLEND
#ifdef @ENABLE_HSL_BLEND_MODES
// Note: the following routines are mathematically equivalent to those in
// https://registry.khronos.org/OpenGL/extensions/KHR/KHR_blend_equation_advanced.txt
// but have been rearranged to be more efficient.

// When using one of the HSL blend equations in table X.2 as the blend equation,
// the blend coefficients are effectively obtained by converting both the
// non-premultiplied source and destination colors to the HSL (hue, saturation,
// luminosity) color space, generating a new HSL color by selecting H, S, and L
// components from the source or destination according to the blend equation,
// and then converting the result back to RGB. The HSL blend equations are only
// well defined when the values of the input color components are in the range
// [0..1].
half lum_from_rgb(half3 c) { return dot(c, make_half3(.30, .59, .11)); }

// Take the base RGB color and override its luminosity with that of another
// RGB color (lumColor).
half3 set_lum(half3 baseColor, half3 lumColor)
{
    half lumTarget = lum_from_rgb(lumColor);

    // Bias the color such that its original luminance is now the 0 point.
    half3 biased = baseColor - lum_from_rgb(baseColor);

    // Now we potentially need to rescale the color to get it to fit within
    // 0..1. Effectively to keep the relative luminance, we may need to squish
    // the rgb range so it still fits.

    //  Calculate the scale values necessary to push the min or max component
    //  back into range. One is for if the luminance pushes the rgb values
    //  negative (pushing min component back into range), the other for if they
    //  go above 1.0 (pushing max component).
    half2 scales =
        make_half2(lumTarget, 1.0 - lumTarget) /
        max(make_half2(EPSILON_FP16_NON_DENORM),
            make_half2(-min_component(biased), max_component(biased)));

    // Take the minimum scale of the above (but nothing larger than 1, since we
    // only ever want to scale *down* the range)
    half satScale = min(make_half(1.0), min(scales.x, scales.y));

    // Now we can apply the scale to get the rgb range right, then re-bias it
    // back into the correct place (at the new luminance)
    return biased * satScale + lumTarget;
}

// Take the hue from hueColor, the saturation from satColor, and the luminance
// from lumColor and combine them into one resulting color.
half3 set_lum_sat(half3 hueColor, half3 satColor, half3 lumColor)
{
    // The saturation of a color is the difference between its max and min
    // components.
    float satTarget = max_component(satColor) - min_component(satColor);

    // Bias the hue color so its min component is 0 (this is not *strictly*
    // required but it simplifies the saturation calculation and matches the
    // canonical math).
    hueColor -= min_component(hueColor);

    // Because of that bias, the minimum component is now 0, so the saturation
    // is just the max component.
    float satSource = max_component(hueColor);

    // Rescale hueColor to have the new saturation. If satSource == 0, then
    // hueColor == {0, 0, 0}, so do a max on the denominator to avoid a divide
    // by 0.
    float scale = satTarget / max(EPSILON_FP16_NON_DENORM, satSource);

    // Now apply the luminance from lumColor to the rescaled color.
    return set_lum(hueColor * scale, lumColor);
}
#endif // ENABLE_HSL_BLEND_MODES

// The advanced blend coefficients are generated from un-multiplied RGB values,
// and control the look of each blend mode.
half3 advanced_blend_coeffs(half3 src, half4 dstPremul, ushort mode)
{
    half3 dst = unmultiply_rgb(dstPremul);
    half3 coeffs;
    switch (mode)
    {
        case BLEND_MODE_MULTIPLY:
            coeffs = src.rgb * dst.rgb;
            break;
        case BLEND_MODE_SCREEN:
            coeffs = src.rgb + dst.rgb - src.rgb * dst.rgb;
            break;
        case BLEND_MODE_OVERLAY:
        {
            // This logic is equivalent to the following, but should be more
            // efficient, and works around a Vulkan Adreno 6-series Android 9/10
            // driver bug:
            //  f(Cs,Cd) = 2*Cs*Cd, if Cd <= 0.5
            //             1-2*(1-Cs)*(1-Cd), otherwise
            half3 sd = src * dst;
            coeffs = 2.0 * mix(sd,
                               src + dst - sd - 0.5,
                               greaterThan(dst, make_half3(0.5)));
            break;
        }
        case BLEND_MODE_DARKEN:
            coeffs = min(src.rgb, dst.rgb);
            break;
        case BLEND_MODE_LIGHTEN:
            coeffs = max(src.rgb, dst.rgb);
            break;
        case BLEND_MODE_COLORDODGE:
        {
            dstPremul.rgb = clamp(dstPremul.rgb, make_half3(.0), dstPremul.aaa);
            half3 denom =
                clamp(1. - src, make_half3(.0), make_half3(1.)) * dstPremul.a;
            coeffs = mix(min(make_half3(1.), dstPremul.rgb / denom),
                         sign(dstPremul.rgb),
                         equal(denom, make_half3(.0)));
            break;
        }
        case BLEND_MODE_COLORBURN:
        {
            src = clamp(src, make_half3(.0), make_half3(1.));
            dstPremul.rgb = clamp(dstPremul.rgb, make_half3(.0), dstPremul.aaa);
            if (dstPremul.a == .0)
                dstPremul.a = 1.;
            half3 numer = dstPremul.a - dstPremul.rgb;
            coeffs = 1. - mix(min(make_half3(1.), numer / (src * dstPremul.a)),
                              sign(numer),
                              equal(src, make_half3(.0)));
            break;
        }
        case BLEND_MODE_HARDLIGHT:
        {
            // This logic is equivalent to the following, but should be more
            // efficient, and works around a Vulkan Adreno 6-series Android 9/10
            // driver bug:
            //   f(Cs,Cd) = 2*Cs*Cd, if Cs <= 0.5
            //              1-2*(1-Cs)*(1-Cd), otherwise
            half3 sd = src * dst;
            coeffs = 2.0 * mix(sd,
                               src + dst - sd - 0.5,
                               greaterThan(src, make_half3(0.5)));
            break;
        }
        case BLEND_MODE_SOFTLIGHT:
        {
            // This logic is equivalent to the following, but should be more
            // efficient, and works around a Vulkan Adreno 6-series Android 9/10
            // driver bug:
            //   f(Cs,Cd) =
            //     Cd-(1-2*Cs)*Cd*(1-Cd),
            //       if Cs <= 0.5
            //     Cd+(2*Cs-1)*Cd*((16*Cd-12)*Cd+3),
            //       if Cs > 0.5 and Cd <= 0.25
            //     Cd+(2*Cs-1)*(sqrt(Cd)-Cd),
            //       if Cs > 0.5 and Cd > 0.25
            for (int i = 0; i < 3; ++i)
            {
                if (src[i] <= 0.5)
                    coeffs[i] = (1.0 - dst[i]);
                else if (dst[i] <= 0.25)
                    coeffs[i] = ((16.0 * dst[i] - 12.0) * dst[i] + 3.0);
                else
                    coeffs[i] = (inversesqrt(dst[i]) - 1.0);
            }

            coeffs = dst + dst * (2.0 * src - 1.0) * coeffs;
            break;
        }
        case BLEND_MODE_DIFFERENCE:
            coeffs = abs(dst.rgb - src.rgb);
            break;
        case BLEND_MODE_EXCLUSION:
            coeffs = src.rgb + dst.rgb - 2. * src.rgb * dst.rgb;
            break;
#ifdef @ENABLE_HSL_BLEND_MODES
        // The HSL blend equations are only well defined when the values of the
        // input color components are in the range [0..1].
        case BLEND_MODE_HUE:
            if (@ENABLE_HSL_BLEND_MODES)
            {
                src.rgb = clamp(src.rgb, make_half3(.0), make_half3(1.));
                coeffs = set_lum_sat(src.rgb, dst.rgb, dst.rgb);
            }
            break;
        case BLEND_MODE_SATURATION:
            if (@ENABLE_HSL_BLEND_MODES)
            {
                src.rgb = clamp(src.rgb, make_half3(.0), make_half3(1.));
                coeffs = set_lum_sat(dst.rgb, src.rgb, dst.rgb);
            }
            break;
        case BLEND_MODE_COLOR:
            if (@ENABLE_HSL_BLEND_MODES)
            {
                src.rgb = clamp(src.rgb, make_half3(.0), make_half3(1.));
                coeffs = set_lum(src.rgb, dst.rgb);
            }
            break;
        case BLEND_MODE_LUMINOSITY:
            if (@ENABLE_HSL_BLEND_MODES)
            {
                src.rgb = clamp(src.rgb, make_half3(.0), make_half3(1.));
                coeffs = set_lum(dst.rgb, src.rgb);
            }
            break;
#endif
    }
    return coeffs;
}

// Performs the given advanced blend operation with a solid RGB src color (no
// srcAlpha).
//
// NOTE: This method is sufficient for all blending because alpha in the src
// can be accounted for afterward using a standard src-over blend operation.
//
// e.g., dst = blend_src_over(
//           premultiply(advanced_color_blend(src.rgb, dstPremul)),
//           dstPremul)
INLINE half3 advanced_color_blend(half3 src, half4 dstPremul, ushort mode)
{
    // The weighting functions p0, p1, and p2 are defined as follows:
    //
    //     p0(As,Ad) = As*Ad
    //     p1(As,Ad) = As*(1 - Ad)
    //     p2(As,Ad) = Ad*(1 - As)
    //
    // Since srcAlpha (As) == 1, this simplifies to:
    //
    //     p0(As,Ad) = Ad
    //     p1(As,Ad) = (1 - Ad)
    //     p2(As,Ad) = 0
    //
    // Blending is performed according to the following equations:
    //
    //     R = coeffs(Rs',Rd')*p0 + Y*Rs'*p1 + Z*Rd'*p2
    //     G = coeffs(Gs',Gd')*p0 + Y*Gs'*p1 + Z*Gd'*p2
    //     B = coeffs(Bs',Bd')*p0 + Y*Bs'*p1 + Z*Bd'*p2
    //     A =               X*p0 +     Y*p1 +     Z*p2
    //
    // NOTE: (X,Y,Z) always == 1, so it is ignored in this implementation.
    //       Also, since (X,Y,Z) == 1, alpha simplifies to standard src-over
    //       rules: A = Ad * (1 - As) + As
    half3 coeffs = advanced_blend_coeffs(src, dstPremul, mode);

    // Because p0 is (Ad), p1 is (1 - Ad), and p2 is 0, this is equivalent to
    // that matrix multiply:
    return mix(src, coeffs, make_half3(dstPremul.a));
}
#endif // ENABLE_ADVANCED_BLEND

#endif // FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_ADVANCED_BLEND_SOURCE: &str = PINNED_ADVANCED_BLEND_GLSL_SOURCE;
pub const ADVANCED_BLEND_GLSL_SOURCE: &str = PINNED_ADVANCED_BLEND_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_ADVANCED_BLEND_GLSL_SOURCE
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
    pub source_sha256: &'static str,
    pub source_line_count: usize,
    pub source_byte_count: usize,
    pub target_path: &'static str,
    pub translation_disposition: &'static str,
}

pub const SOURCE_METADATA: SourceMetadata = SourceMetadata {
    upstream_commit: PINNED_UPSTREAM_COMMIT,
    upstream_path: PINNED_SOURCE_PATH,
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    target_path: "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/advanced_blend_glsl.rs",
    translation_disposition: "full-translation-source / source-shaped provenance",
};

/// Every semantic preprocessor block in the pinned source, including blocks
/// whose branch body is only a source-visible layout or function declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBlock {
    pub block_id: &'static str,
    pub block_start: u16,
    pub block_end: u16,
    pub block_depth: u8,
    pub branch_count: u8,
}

pub const CONDITIONAL_BLOCKS: &[ConditionalBlock] = &[
    ConditionalBlock {
        block_id: "pp-0099",
        block_start: 58,
        block_end: 330,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0100",
        block_start: 60,
        block_end: 78,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0101",
        block_start: 62,
        block_end: 76,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0102",
        block_start: 80,
        block_end: 328,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0103",
        block_start: 81,
        block_end: 152,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0104",
        block_start: 253,
        block_end: 284,
        block_depth: 2,
        branch_count: 1,
    },
];

/// Every branch entry remains literal, in authority/source order. The active
/// paths are ledger spellings; they are not evaluated as Rust cfg expressions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBranch {
    pub block_id: &'static str,
    pub branch_ordinal: u8,
    pub branch_line: u16,
    pub directive: &'static str,
    pub active_branch_path: &'static str,
}

pub const CONDITIONAL_BRANCHES: &[ConditionalBranch] = &[
    ConditionalBranch {
        block_id: "pp-0099",
        branch_ordinal: 1,
        branch_line: 58,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0100",
        branch_ordinal: 1,
        branch_line: 60,
        directive: "#ifdef @ENABLE_KHR_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_KHR_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0101",
        branch_ordinal: 1,
        branch_line: 62,
        directive: "#ifdef @ENABLE_HSL_BLEND_MODES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_KHR_BLEND)) && (defined(@ENABLE_HSL_BLEND_MODES))",
    },
    ConditionalBranch {
        block_id: "pp-0101",
        branch_ordinal: 2,
        branch_line: 64,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_KHR_BLEND)) && (!((defined(@ENABLE_HSL_BLEND_MODES))))",
    },
    ConditionalBranch {
        block_id: "pp-0102",
        branch_ordinal: 1,
        branch_line: 80,
        directive: "#ifdef @ENABLE_ADVANCED_BLEND",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    ConditionalBranch {
        block_id: "pp-0103",
        branch_ordinal: 1,
        branch_line: 81,
        directive: "#ifdef @ENABLE_HSL_BLEND_MODES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@ENABLE_HSL_BLEND_MODES))",
    },
    ConditionalBranch {
        block_id: "pp-0104",
        branch_ordinal: 1,
        branch_line: 253,
        directive: "#ifdef @ENABLE_HSL_BLEND_MODES",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@ENABLE_HSL_BLEND_MODES))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The four @-prefixed switches exported by minify.py for this source. The
/// generated names are the pinned batch-minifier outputs.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 58,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 60,
        source_name: "@ENABLE_KHR_BLEND",
        generated_name: "ZD",
        generated_header_name: "GLSL_ENABLE_KHR_BLEND",
    },
    ExportedSymbol {
        source_line: 62,
        source_name: "@ENABLE_HSL_BLEND_MODES",
        generated_name: "XB",
        generated_header_name: "GLSL_ENABLE_HSL_BLEND_MODES",
    },
    ExportedSymbol {
        source_line: 80,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
];

pub const EXPORTED_SWITCHES: &[ExportedSymbol] = EXPORTED_SYMBOLS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderFunction {
    pub source_line: u16,
    pub end_line: u16,
    pub name: &'static str,
    pub signature: &'static str,
    pub guard_path: &'static str,
    pub inline_qualifier: &'static str,
}

/// Function declarations are retained as source spellings and ranges. Their
/// bodies remain in PINNED_ADVANCED_BLEND_GLSL_SOURCE rather than being
/// translated into executable Rust.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 94,
        end_line: 94,
        name: "lum_from_rgb",
        signature: "half lum_from_rgb(half3 c) { return dot(c, make_half3(.30, .59, .11)); }",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@ENABLE_HSL_BLEND_MODES))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 98,
        end_line: 125,
        name: "set_lum",
        signature: "half3 set_lum(half3 baseColor, half3 lumColor)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@ENABLE_HSL_BLEND_MODES))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 129,
        end_line: 151,
        name: "set_lum_sat",
        signature: "half3 set_lum_sat(half3 hueColor, half3 satColor, half3 lumColor)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@ENABLE_HSL_BLEND_MODES))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 156,
        end_line: 287,
        name: "advanced_blend_coeffs",
        signature: "half3 advanced_blend_coeffs(half3 src, half4 dstPremul, ushort mode)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 298,
        end_line: 327,
        name: "advanced_color_blend",
        signature: "INLINE half3 advanced_color_blend(half3 src, half4 dstPremul, ushort mode)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
        inline_qualifier: "INLINE",
    },
];

pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlendModeCase {
    pub source_line: u16,
    pub name: &'static str,
    pub guard_path: &'static str,
}

pub const BLEND_MODE_CASES: &[BlendModeCase] = &[
    BlendModeCase {
        source_line: 162,
        name: "BLEND_MODE_MULTIPLY",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 165,
        name: "BLEND_MODE_SCREEN",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 168,
        name: "BLEND_MODE_OVERLAY",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 181,
        name: "BLEND_MODE_DARKEN",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 184,
        name: "BLEND_MODE_LIGHTEN",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 187,
        name: "BLEND_MODE_COLORDODGE",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 197,
        name: "BLEND_MODE_COLORBURN",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 209,
        name: "BLEND_MODE_HARDLIGHT",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 222,
        name: "BLEND_MODE_SOFTLIGHT",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 247,
        name: "BLEND_MODE_DIFFERENCE",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 250,
        name: "BLEND_MODE_EXCLUSION",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND))",
    },
    BlendModeCase {
        source_line: 256,
        name: "BLEND_MODE_HUE",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@ENABLE_HSL_BLEND_MODES))",
    },
    BlendModeCase {
        source_line: 263,
        name: "BLEND_MODE_SATURATION",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@ENABLE_HSL_BLEND_MODES))",
    },
    BlendModeCase {
        source_line: 270,
        name: "BLEND_MODE_COLOR",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@ENABLE_HSL_BLEND_MODES))",
    },
    BlendModeCase {
        source_line: 277,
        name: "BLEND_MODE_LUMINOSITY",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_ADVANCED_BLEND)) && (defined(@ENABLE_HSL_BLEND_MODES))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncludeDependency {
    pub including_source: &'static str,
    pub include_line: u16,
    pub include_token: &'static str,
    pub include_syntax: &'static str,
    pub active_branch_path: &'static str,
    pub resolution_kind: &'static str,
    pub resolved_source: &'static str,
    pub source_unit: &'static str,
    pub dependency_unit: &'static str,
    pub translation_disposition: &'static str,
}

/// The advanced_blend owner has no direct #include/#import directive. These
/// two incoming generated-source edges are retained from the include/source
/// dependency authorities because they determine its artifact consumers.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[
    IncludeDependency {
        including_source: "renderer/src/metal/background_shader_compiler.mm",
        include_line: 11,
        include_token: "generated/shaders/advanced_blend.glsl.hpp",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/advanced_blend.glsl",
        source_unit: "metal-background-shader-compiler",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/draw.metal",
        include_line: 36,
        include_token: "advanced_blend.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/advanced_blend.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
];

pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;

/// Pinned source-level KHR layout alternatives remain visible without
/// selecting one branch in Rust.
pub const KHR_BLEND_LAYOUT_ALTERNATIVES: &[&str] = &[
    "blend_support_all_equations",
    "blend_support_multiply",
    "blend_support_screen",
    "blend_support_overlay",
    "blend_support_darken",
    "blend_support_lighten",
    "blend_support_colordodge",
    "blend_support_colorburn",
    "blend_support_hardlight",
    "blend_support_softlight",
    "blend_support_difference",
    "blend_support_exclusion",
];
