/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/render_atlas.glsl.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger conditionals, includes, exports, functions, and source
 * metadata as literal source-shaped data. It does not compile, evaluate,
 * simplify, or generate shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/render_atlas.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "9d720063cf3360342205dcfa385e5c43e034aef3dfbd22014991de7aef61e4e6";
pub const PINNED_SOURCE_LINE_COUNT: usize = 249;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 7918;
pub const PINNED_SOURCE_STAGE: &str = "minify-input-glsl";
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str =
    "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/render_atlas_glsl.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_RENDER_ATLAS_GLSL_SOURCE: &str = r###"/*
 * Copyright 2025 Rive
 */

#ifdef @VERTEX
ATTR_BLOCK_BEGIN(Attrs)
// [localVertexID, outset, fillCoverage, vertexType]
ATTR(0, float4, @a_patchVertexData);
ATTR(1, float4, @a_mirroredVertexData);
ATTR_BLOCK_END
#endif

VARYING_BLOCK_BEGIN
NO_PERSPECTIVE VARYING(0, float4, v_coverages);
VARYING_BLOCK_END

#ifdef @VERTEX
VERTEX_MAIN(@atlasVertexMain, Attrs, attrs, _vertexID, _instanceID)
{
    ATTR_UNPACK(_vertexID, attrs, @a_patchVertexData, float4);
    ATTR_UNPACK(_vertexID, attrs, @a_mirroredVertexData, float4);

    VARYING_INIT(v_coverages, float4);

    float4 pos;
    uint pathID;
    float2 vertexPosition;
    if (unpack_tessellated_path_vertex(@a_patchVertexData,
                                       @a_mirroredVertexData,
                                       _instanceID,
                                       pathID,
                                       vertexPosition,
                                       v_coverages VERTEX_CONTEXT_UNPACK))
    {
        // Offset from on-screen coordinates to atlas coordinates.
        uint4 pathData2 = STORAGE_BUFFER_LOAD4(@pathBuffer, pathID * 4u + 2u);
        float3 atlasTransform = uintBitsToFloat(pathData2.yzw);
        vertexPosition = vertexPosition * atlasTransform.x + atlasTransform.yz;

        pos = pixel_coord_to_clip_coord(vertexPosition,
                                        uniforms.atlasContentInverseViewport.x,
                                        uniforms.atlasContentInverseViewport.y);
#ifdef @POST_INVERT_Y
        pos.y = -pos.y;
#endif
    }
    else
    {
        pos = float4(uniforms.vertexDiscardValue,
                     uniforms.vertexDiscardValue,
                     uniforms.vertexDiscardValue,
                     uniforms.vertexDiscardValue);
    }

    VARYING_PACK(v_coverages);
    EMIT_VERTEX(pos);
}
#endif // @VERTEX

#ifdef @FRAGMENT

#ifdef @ATLAS_FEATHERED_FILL
INLINE half signed_fill_coverage(float4 coverages,
                                 bool clockwise TEXTURE_CONTEXT_DECL)
{
    half coverage = eval_feathered_fill(coverages TEXTURE_CONTEXT_FORWARD);
    if (!clockwise)
        coverage = -coverage;
    return coverage;
}
#endif

#ifdef @ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH

// Store coverage as fp32 data bits in an r32ui color buffer, and use
// framebuffer-fetch to manipulate it.
layout(location = 0) inout uint4 coverageCount;

#ifdef @ATLAS_FEATHERED_FILL
void main()
{
    float coverage = uintBitsToFloat(coverageCount.r);
    coverage += signed_fill_coverage(v_coverages,
                                     gl_FrontFacing TEXTURE_CONTEXT_FORWARD);
    coverageCount.r = floatBitsToUint(coverage);
}
#endif

#ifdef @ATLAS_FEATHERED_STROKE
void main()
{
    float coverage = uintBitsToFloat(coverageCount.r);
    coverage = max(coverage, eval_feathered_stroke(v_coverages));
    coverageCount.r = floatBitsToUint(coverage);
}
#endif

#elif defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)

// Manipulate fp32 coverage in pixel local storage, which will be written out
// to an r32ui color buffer during a separate resolve step.
__pixel_localEXT PLS { layout(r32f) float coverageCount; };

#ifdef @ATLAS_FEATHERED_FILL
void main()
{
    coverageCount +=
        signed_fill_coverage(v_coverages,
                             gl_FrontFacing TEXTURE_CONTEXT_FORWARD);
}
#endif

#ifdef @ATLAS_FEATHERED_STROKE
void main()
{
    coverageCount = max(coverageCount, eval_feathered_stroke(v_coverages));
}
#endif

#elif defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)

// Store and manipulate coverage as fp32 data bits in r32ui-texture-backed pixel
// local storage.
layout(binding = 0, r32ui) uniform highp upixelLocalANGLE coverageCount;

#ifdef @ATLAS_FEATHERED_FILL
void main()
{
    float coverage = uintBitsToFloat(pixelLocalLoadANGLE(coverageCount).r);
    coverage += signed_fill_coverage(v_coverages,
                                     gl_FrontFacing TEXTURE_CONTEXT_FORWARD);
    pixelLocalStoreANGLE(coverageCount, uint4(floatBitsToUint(coverage)));
}
#endif

#ifdef @ATLAS_FEATHERED_STROKE
void main()
{
    float coverage = uintBitsToFloat(pixelLocalLoadANGLE(coverageCount).r);
    coverage = max(coverage, eval_feathered_stroke(v_coverages));
    pixelLocalStoreANGLE(coverageCount, uint4(floatBitsToUint(coverage)));
}
#endif

#elif defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)

// Store coverage as 16:16 fixed point in an r32i texture, which we manipulate
// with atomics.
layout(binding = 0, r32i) uniform highp coherent iimage2D _atlasImage;
ivec2 image_coord() { return ivec2(floor(_fragCoord)); }
int fixedpoint_coverage(float coverage)
{
    return int(coverage * ATLAS_R32I_FIXED_POINT_FACTOR);
}

#ifdef @ATLAS_FEATHERED_FILL
void main()
{
    int coverage = fixedpoint_coverage(
        signed_fill_coverage(v_coverages,
                             gl_FrontFacing TEXTURE_CONTEXT_FORWARD));
    imageAtomicAdd(_atlasImage, image_coord(), coverage);
}
#endif

#ifdef @ATLAS_FEATHERED_STROKE
void main()
{
    int coverage = fixedpoint_coverage(eval_feathered_stroke(v_coverages));
    imageAtomicMax(_atlasImage, image_coord(), coverage);
}
#endif

#elif defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)

// We don't have any extensions to count high precision coverage. (This is very
// rare.). Just split up coverage across rgba8 components and hope for the best.

#ifdef @ATLAS_FEATHERED_FILL
FRAG_DATA_MAIN_WITH_CLOCKWISE(half4, @atlasFillFragmentMain)
{
    VARYING_UNPACK(v_coverages, float4);
    half coverage =
        signed_fill_coverage(v_coverages, _clockwise TEXTURE_CONTEXT_FORWARD);
    // i.e., is abs(coverage) ~= FEATHER(1), allowing for some sub-8-bit slop in
    // the texture unit performing a clamp to edge.
    if (abs(coverage) > MAX_FEATHER - 1e-3)
    {
        // All the "fan triangles" in a feather have solid coverage. This is a
        // substantial number of triangles, so we dedicate 2 channels to
        // counting solid coverage (i.e, +1 or -1). These channels are also much
        // slower to overflow, so it preserves a basic skeleton of the feather
        // when the fractional channels overflow.
        EMIT_FRAG_DATA(coverage > .0
                           // B counts integer, positive coverage.
                           ? make_half4(.0, .0, 1. / 255., .0)
                           // A counts integer, negative coverage.
                           : make_half4(.0, .0, .0, 1. / 255.));
    }
    else
    {
        coverage *= 1. / ATLAS_UNORM8_COVERAGE_SCALE_FACTOR;
        EMIT_FRAG_DATA(make_half4(
            max(coverage, .0),  // R counts fractional, positive coverage.
            max(-coverage, .0), // G counts fractional, negative coverage.
            .0,
            .0));
    }
}
#endif // @ATLAS_FEATHERED_FILL

#ifdef @ATLAS_FEATHERED_STROKE
FRAG_DATA_MAIN(half4, @atlasStrokeFragmentMain)
{
    VARYING_UNPACK(v_coverages, float4);
    half coverage = eval_feathered_stroke(v_coverages TEXTURE_CONTEXT_FORWARD);
    // Strokes only have positive coverage, and since we only need to saturate
    // the max for stroking, we can just use the R channel.
    coverage *= 1. / ATLAS_UNORM8_COVERAGE_SCALE_FACTOR;
    EMIT_FRAG_DATA(make_half4(coverage, .0, .0, .0));
}
#endif // @ATLAS_FEATHERED_STROKE

#else

// This is the ideal case. We have full support for floating point color
// buffers, including blending. Render to float and let the fixed function blend
// hardware count the coverage.

#ifdef @ATLAS_FEATHERED_FILL
FRAG_DATA_MAIN_WITH_CLOCKWISE(float, @atlasFillFragmentMain)
{
    VARYING_UNPACK(v_coverages, float4);
    EMIT_FRAG_DATA(
        signed_fill_coverage(v_coverages, _clockwise TEXTURE_CONTEXT_FORWARD));
}
#endif

#ifdef @ATLAS_FEATHERED_STROKE
FRAG_DATA_MAIN(float, @atlasStrokeFragmentMain)
{
    VARYING_UNPACK(v_coverages, float4);
    EMIT_FRAG_DATA(eval_feathered_stroke(v_coverages TEXTURE_CONTEXT_FORWARD));
}
#endif

#endif

#endif // FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_RENDER_ATLAS_SOURCE: &str = PINNED_RENDER_ATLAS_GLSL_SOURCE;
pub const RENDER_ATLAS_GLSL_SOURCE: &str = PINNED_RENDER_ATLAS_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_RENDER_ATLAS_GLSL_SOURCE
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
    pub source_stage: &'static str,
    pub source_sha256: &'static str,
    pub source_line_count: usize,
    pub source_byte_count: usize,
    pub target_path: &'static str,
    pub translation_unit: &'static str,
    pub translation_disposition: &'static str,
    pub translation_behavior: &'static str,
}

pub const SOURCE_METADATA: SourceMetadata = SourceMetadata {
    upstream_commit: PINNED_UPSTREAM_COMMIT,
    upstream_path: PINNED_SOURCE_PATH,
    source_stage: PINNED_SOURCE_STAGE,
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    target_path: TRANSLATION_TARGET,
    translation_unit: TRANSLATION_UNIT,
    translation_disposition: TRANSLATION_DISPOSITION,
    translation_behavior: TRANSLATION_BEHAVIOR,
};

/// Every semantic preprocessor block in the pinned source remains literal.
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
        block_id: "pp-0576",
        block_start: 5,
        block_end: 11,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0577",
        block_start: 17,
        block_end: 58,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0578",
        block_start: 43,
        block_end: 45,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0579",
        block_start: 60,
        block_end: 249,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0580",
        block_start: 62,
        block_end: 71,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0581",
        block_start: 73,
        block_end: 247,
        block_depth: 1,
        branch_count: 6,
    },
    ConditionalBlock {
        block_id: "pp-0582",
        block_start: 79,
        block_end: 87,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0583",
        block_start: 89,
        block_end: 96,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0584",
        block_start: 104,
        block_end: 111,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0585",
        block_start: 113,
        block_end: 118,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0586",
        block_start: 126,
        block_end: 134,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0587",
        block_start: 136,
        block_end: 143,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0588",
        block_start: 156,
        block_end: 164,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0589",
        block_start: 166,
        block_end: 172,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0590",
        block_start: 179,
        block_end: 210,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0591",
        block_start: 212,
        block_end: 222,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0592",
        block_start: 230,
        block_end: 237,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0593",
        block_start: 239,
        block_end: 245,
        block_depth: 2,
        branch_count: 1,
    },
];

/// Every branch entry remains literal, in authority/source order. Active
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
        block_id: "pp-0576",
        branch_ordinal: 1,
        branch_line: 5,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0577",
        branch_ordinal: 1,
        branch_line: 17,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0578",
        branch_ordinal: 1,
        branch_line: 43,
        directive: "#ifdef @POST_INVERT_Y",
        active_branch_path: "(defined(@VERTEX)) && (defined(@POST_INVERT_Y))",
    },
    ConditionalBranch {
        block_id: "pp-0579",
        branch_ordinal: 1,
        branch_line: 60,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0580",
        branch_ordinal: 1,
        branch_line: 62,
        directive: "#ifdef @ATLAS_FEATHERED_FILL",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ATLAS_FEATHERED_FILL))",
    },
    ConditionalBranch {
        block_id: "pp-0581",
        branch_ordinal: 1,
        branch_line: 73,
        directive: "#ifdef @ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))",
    },
    ConditionalBranch {
        block_id: "pp-0581",
        branch_ordinal: 2,
        branch_line: 98,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))) && (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)))",
    },
    ConditionalBranch {
        block_id: "pp-0581",
        branch_ordinal: 3,
        branch_line: 120,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)))",
    },
    ConditionalBranch {
        block_id: "pp-0581",
        branch_ordinal: 4,
        branch_line: 145,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)))",
    },
    ConditionalBranch {
        block_id: "pp-0581",
        branch_ordinal: 5,
        branch_line: 174,
        directive: "#elif defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE))) && (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)))",
    },
    ConditionalBranch {
        block_id: "pp-0581",
        branch_ordinal: 6,
        branch_line: 224,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)) || (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM))))",
    },
    ConditionalBranch {
        block_id: "pp-0582",
        branch_ordinal: 1,
        branch_line: 79,
        directive: "#ifdef @ATLAS_FEATHERED_FILL",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) && (defined(@ATLAS_FEATHERED_FILL))",
    },
    ConditionalBranch {
        block_id: "pp-0583",
        branch_ordinal: 1,
        branch_line: 89,
        directive: "#ifdef @ATLAS_FEATHERED_STROKE",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) && (defined(@ATLAS_FEATHERED_STROKE))",
    },
    ConditionalBranch {
        block_id: "pp-0584",
        branch_ordinal: 1,
        branch_line: 104,
        directive: "#ifdef @ATLAS_FEATHERED_FILL",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))) && (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (defined(@ATLAS_FEATHERED_FILL))",
    },
    ConditionalBranch {
        block_id: "pp-0585",
        branch_ordinal: 1,
        branch_line: 113,
        directive: "#ifdef @ATLAS_FEATHERED_STROKE",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH))) && (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (defined(@ATLAS_FEATHERED_STROKE))",
    },
    ConditionalBranch {
        block_id: "pp-0586",
        branch_ordinal: 1,
        branch_line: 126,
        directive: "#ifdef @ATLAS_FEATHERED_FILL",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_FEATHERED_FILL))",
    },
    ConditionalBranch {
        block_id: "pp-0587",
        branch_ordinal: 1,
        branch_line: 136,
        directive: "#ifdef @ATLAS_FEATHERED_STROKE",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT))) && (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_FEATHERED_STROKE))",
    },
    ConditionalBranch {
        block_id: "pp-0588",
        branch_ordinal: 1,
        branch_line: 156,
        directive: "#ifdef @ATLAS_FEATHERED_FILL",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE))) && (defined(@ATLAS_FEATHERED_FILL))",
    },
    ConditionalBranch {
        block_id: "pp-0589",
        branch_ordinal: 1,
        branch_line: 166,
        directive: "#ifdef @ATLAS_FEATHERED_STROKE",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE))) && (defined(@ATLAS_FEATHERED_STROKE))",
    },
    ConditionalBranch {
        block_id: "pp-0590",
        branch_ordinal: 1,
        branch_line: 179,
        directive: "#ifdef @ATLAS_FEATHERED_FILL",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE))) && (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM))) && (defined(@ATLAS_FEATHERED_FILL))",
    },
    ConditionalBranch {
        block_id: "pp-0591",
        branch_ordinal: 1,
        branch_line: 212,
        directive: "#ifdef @ATLAS_FEATHERED_STROKE",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE))) && (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM))) && (defined(@ATLAS_FEATHERED_STROKE))",
    },
    ConditionalBranch {
        block_id: "pp-0592",
        branch_ordinal: 1,
        branch_line: 230,
        directive: "#ifdef @ATLAS_FEATHERED_FILL",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)) || (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)))) && (defined(@ATLAS_FEATHERED_FILL))",
    },
    ConditionalBranch {
        block_id: "pp-0593",
        branch_ordinal: 1,
        branch_line: 239,
        directive: "#ifdef @ATLAS_FEATHERED_STROKE",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)) || (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)))) && (defined(@ATLAS_FEATHERED_STROKE))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// Direct @-prefixed exports occurring in the pinned source, in source order.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 8,
        source_name: "@a_patchVertexData",
        generated_name: "SB",
        generated_header_name: "GLSL_a_patchVertexData",
    },
    ExportedSymbol {
        source_line: 9,
        source_name: "@a_mirroredVertexData",
        generated_name: "TB",
        generated_header_name: "GLSL_a_mirroredVertexData",
    },
    ExportedSymbol {
        source_line: 18,
        source_name: "@atlasVertexMain",
        generated_name: "KF",
        generated_header_name: "GLSL_atlasVertexMain",
    },
    ExportedSymbol {
        source_line: 36,
        source_name: "@pathBuffer",
        generated_name: "MB",
        generated_header_name: "GLSL_pathBuffer",
    },
    ExportedSymbol {
        source_line: 43,
        source_name: "@POST_INVERT_Y",
        generated_name: "JC",
        generated_header_name: "GLSL_POST_INVERT_Y",
    },
    ExportedSymbol {
        source_line: 60,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 62,
        source_name: "@ATLAS_FEATHERED_FILL",
        generated_name: "FC",
        generated_header_name: "GLSL_ATLAS_FEATHERED_FILL",
    },
    ExportedSymbol {
        source_line: 73,
        source_name: "@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        generated_name: "MD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
    },
    ExportedSymbol {
        source_line: 89,
        source_name: "@ATLAS_FEATHERED_STROKE",
        generated_name: "MC",
        generated_header_name: "GLSL_ATLAS_FEATHERED_STROKE",
    },
    ExportedSymbol {
        source_line: 98,
        source_name: "@ATLAS_RENDER_TARGET_R8_PLS_EXT",
        generated_name: "ND",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R8_PLS_EXT",
    },
    ExportedSymbol {
        source_line: 120,
        source_name: "@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_name: "EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    },
    ExportedSymbol {
        source_line: 145,
        source_name: "@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
        generated_name: "OD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
    },
    ExportedSymbol {
        source_line: 174,
        source_name: "@ATLAS_RENDER_TARGET_RGBA8_UNORM",
        generated_name: "ME",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_RGBA8_UNORM",
    },
    ExportedSymbol {
        source_line: 180,
        source_name: "@atlasFillFragmentMain",
        generated_name: "NE",
        generated_header_name: "GLSL_atlasFillFragmentMain",
    },
    ExportedSymbol {
        source_line: 213,
        source_name: "@atlasStrokeFragmentMain",
        generated_name: "OE",
        generated_header_name: "GLSL_atlasStrokeFragmentMain",
    },
];

/// The preprocessor-switch subset of EXPORTED_SYMBOLS.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 5,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 43,
        source_name: "@POST_INVERT_Y",
        generated_name: "JC",
        generated_header_name: "GLSL_POST_INVERT_Y",
    },
    ExportedSymbol {
        source_line: 60,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 62,
        source_name: "@ATLAS_FEATHERED_FILL",
        generated_name: "FC",
        generated_header_name: "GLSL_ATLAS_FEATHERED_FILL",
    },
    ExportedSymbol {
        source_line: 73,
        source_name: "@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        generated_name: "MD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
    },
    ExportedSymbol {
        source_line: 89,
        source_name: "@ATLAS_FEATHERED_STROKE",
        generated_name: "MC",
        generated_header_name: "GLSL_ATLAS_FEATHERED_STROKE",
    },
    ExportedSymbol {
        source_line: 98,
        source_name: "@ATLAS_RENDER_TARGET_R8_PLS_EXT",
        generated_name: "ND",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R8_PLS_EXT",
    },
    ExportedSymbol {
        source_line: 120,
        source_name: "@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_name: "EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    },
    ExportedSymbol {
        source_line: 145,
        source_name: "@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
        generated_name: "OD",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
    },
    ExportedSymbol {
        source_line: 174,
        source_name: "@ATLAS_RENDER_TARGET_RGBA8_UNORM",
        generated_name: "ME",
        generated_header_name: "GLSL_ATLAS_RENDER_TARGET_RGBA8_UNORM",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderFunction {
    pub source_line: u16,
    pub end_line: u16,
    pub name: &'static str,
    pub signature: &'static str,
    pub guard_path: &'static str,
    pub inline_qualifier: &'static str,
}

/// Named function declarations and macro-defined entrypoints remain literal.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 18,
        end_line: 57,
        name: "atlasVertexMain",
        signature: "VERTEX_MAIN(@atlasVertexMain, Attrs, attrs, _vertexID, _instanceID)",
        guard_path: "(defined(@VERTEX))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 63,
        end_line: 70,
        name: "signed_fill_coverage",
        signature: "INLINE half signed_fill_coverage(float4 coverages, bool clockwise TEXTURE_CONTEXT_DECL)",
        guard_path: "(defined(@FRAGMENT)) && (defined(@ATLAS_FEATHERED_FILL))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 150,
        end_line: 150,
        name: "image_coord",
        signature: "ivec2 image_coord()",
        guard_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 151,
        end_line: 154,
        name: "fixedpoint_coverage",
        signature: "int fixedpoint_coverage(float coverage)",
        guard_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE))) && (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 180,
        end_line: 209,
        name: "atlasFillFragmentMain",
        signature: "FRAG_DATA_MAIN_WITH_CLOCKWISE(half4, @atlasFillFragmentMain)",
        guard_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE))) && (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM))) && (defined(@ATLAS_FEATHERED_FILL))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 213,
        end_line: 221,
        name: "atlasStrokeFragmentMain",
        signature: "FRAG_DATA_MAIN(half4, @atlasStrokeFragmentMain)",
        guard_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE))) && (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM))) && (defined(@ATLAS_FEATHERED_STROKE))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 231,
        end_line: 236,
        name: "atlasFillFragmentMain",
        signature: "FRAG_DATA_MAIN_WITH_CLOCKWISE(float, @atlasFillFragmentMain)",
        guard_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)) || (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)))) && (defined(@ATLAS_FEATHERED_FILL))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 240,
        end_line: 244,
        name: "atlasStrokeFragmentMain",
        signature: "FRAG_DATA_MAIN(float, @atlasStrokeFragmentMain)",
        guard_path: "(defined(@FRAGMENT)) && (!((defined(@ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH)) || (defined(@ATLAS_RENDER_TARGET_R8_PLS_EXT)) || (defined(@ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE)) || (defined(@ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE)) || (defined(@ATLAS_RENDER_TARGET_RGBA8_UNORM)))) && (defined(@ATLAS_FEATHERED_STROKE))",
        inline_qualifier: "",
    },
];

pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

/// Direct export inventory with source spellings without the leading @.
pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[
    ExportedIdentifier {
        source_name: "VERTEX",
        generated_name: "CB",
    },
    ExportedIdentifier {
        source_name: "a_patchVertexData",
        generated_name: "SB",
    },
    ExportedIdentifier {
        source_name: "a_mirroredVertexData",
        generated_name: "TB",
    },
    ExportedIdentifier {
        source_name: "atlasVertexMain",
        generated_name: "KF",
    },
    ExportedIdentifier {
        source_name: "pathBuffer",
        generated_name: "MB",
    },
    ExportedIdentifier {
        source_name: "POST_INVERT_Y",
        generated_name: "JC",
    },
    ExportedIdentifier {
        source_name: "FRAGMENT",
        generated_name: "FB",
    },
    ExportedIdentifier {
        source_name: "ATLAS_FEATHERED_FILL",
        generated_name: "FC",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
        generated_name: "MD",
    },
    ExportedIdentifier {
        source_name: "ATLAS_FEATHERED_STROKE",
        generated_name: "MC",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R8_PLS_EXT",
        generated_name: "ND",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
        generated_name: "EXPORTED_ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
        generated_name: "OD",
    },
    ExportedIdentifier {
        source_name: "ATLAS_RENDER_TARGET_RGBA8_UNORM",
        generated_name: "ME",
    },
    ExportedIdentifier {
        source_name: "atlasFillFragmentMain",
        generated_name: "NE",
    },
    ExportedIdentifier {
        source_name: "atlasStrokeFragmentMain",
        generated_name: "OE",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "VERTEX",
    "a_patchVertexData",
    "a_mirroredVertexData",
    "atlasVertexMain",
    "pathBuffer",
    "POST_INVERT_Y",
    "FRAGMENT",
    "ATLAS_FEATHERED_FILL",
    "ATLAS_RENDER_TARGET_R32UI_FRAMEBUFFER_FETCH",
    "ATLAS_FEATHERED_STROKE",
    "ATLAS_RENDER_TARGET_R8_PLS_EXT",
    "ATLAS_RENDER_TARGET_R32UI_PLS_ANGLE",
    "ATLAS_RENDER_TARGET_R32I_ATOMIC_TEXTURE",
    "ATLAS_RENDER_TARGET_RGBA8_UNORM",
    "atlasFillFragmentMain",
    "atlasStrokeFragmentMain",
];

/// No source spelling maps ambiguously in this owner.
pub const EXPORT_MAPPING_AMBIGUITIES: &[(&str, &str, &str)] = &[];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderInclude {
    pub upstream_file: &'static str,
    pub include_line: u16,
    pub directive: &'static str,
    pub include_token: &'static str,
    pub include_syntax: &'static str,
    pub active_branch_path: &'static str,
    pub resolution_kind: &'static str,
    pub resolved_source: &'static str,
    pub source_unit: &'static str,
    pub dependency_unit: &'static str,
    pub correspondence_owner: &'static str,
    pub mapping_status: &'static str,
    pub translation_status: &'static str,
    pub translation_disposition: &'static str,
}

/// render_atlas.glsl has no direct #include/#import directive.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// Incoming generated-source edge retained from the include authority.
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[ShaderInclude {
    upstream_file: "renderer/src/shaders/metal/draw.metal",
    include_line: 24,
    directive: "include",
    include_token: "render_atlas.minified.glsl",
    include_syntax: "quote",
    active_branch_path: "all",
    resolution_kind: "generated-shader-source",
    resolved_source: "renderer/src/shaders/render_atlas.glsl",
    source_unit: "metal-shader-source-batch",
    dependency_unit: "metal-shader-source-batch",
    correspondence_owner: "-",
    mapping_status: "-",
    translation_status: "pending",
    translation_disposition: "required-source-edge",
}];

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

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[IncludeDependency {
    including_source: "renderer/src/shaders/metal/draw.metal",
    include_line: 24,
    include_token: "render_atlas.minified.glsl",
    include_syntax: "quote",
    active_branch_path: "all",
    resolution_kind: "generated-shader-source",
    resolved_source: "renderer/src/shaders/render_atlas.glsl",
    source_unit: "metal-shader-source-batch",
    dependency_unit: "metal-shader-source-batch",
    translation_disposition: "preserve-source-dependency",
}];

pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
