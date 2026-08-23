/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/draw_path_common.glsl.
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
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/draw_path_common.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "3a6e72e80eec81b2eb467134f62188e2a86f7debfb0798a8c4ed5873beb7e86e";
pub const PINNED_SOURCE_LINE_COUNT: usize = 914;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 39516;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/draw_path_common_glsl.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_DRAW_PATH_COMMON_GLSL_SOURCE: &str = r###"/*
 * Copyright 2023 Rive
 */

// Common functions shared by draw shaders.

// Feathered coverage values get shifted by "FEATHER_COVERAGE_BIAS" in order
// to classify the coverage as belonging to a feather.
#define FEATHER_COVERAGE_BIAS -2.

// Fragment shaders test if a coverage value is less than
// "FEATHER_COVERAGE_THRESHOLD" to see if the coverage belongs to a feather.
#define FEATHER_COVERAGE_THRESHOLD -1.5

// Since the sign of x dictates the sign of coverage, and since x may be 0 when
// feathering, bias x slightly upward so we don't lose the sign when it's 0.
#define FEATHER_X_COORD_BIAS .25

// Magnitude of cotan(theta) at which we decide an angle is flat when processing
// feathers.
#define HORIZONTAL_COTANGENT_THRESHOLD 1e3

// Value to assign cotTheta to ensure it gets treated as flat.
#define HORIZONTAL_COTANGENT_VALUE                                             \
    (HORIZONTAL_COTANGENT_THRESHOLD * HORIZONTAL_COTANGENT_THRESHOLD)

#ifdef @VERTEX
VERTEX_TEXTURE_BLOCK_BEGIN
TEXTURE_TESSDATA4(PER_FLUSH_BINDINGS_SET,
                  TESS_VERTEX_TEXTURE_IDX,
                  @tessVertexTexture);
#ifdef @ENABLE_FEATHER
TEXTURE_R16F_1D_ARRAY(PER_FLUSH_BINDINGS_SET,
                      GAUSSIAN_INTEGRAL_TEXTURE_IDX,
                      @gaussianIntegralTexture);
#endif
VERTEX_TEXTURE_BLOCK_END

VERTEX_STORAGE_BUFFER_BLOCK_BEGIN
STORAGE_BUFFER_U32x4(PATH_BUFFER_IDX, PathBuffer, @pathBuffer);
STORAGE_BUFFER_U32x2(PAINT_BUFFER_IDX, PaintBuffer, @paintBuffer);
STORAGE_BUFFER_F32x4(PAINT_AUX_BUFFER_IDX, PaintAuxBuffer, @paintAuxBuffer);
STORAGE_BUFFER_U32x4(CONTOUR_BUFFER_IDX, ContourBuffer, @contourBuffer);
VERTEX_STORAGE_BUFFER_BLOCK_END
#endif // @VERTEX

#if defined(@ENABLE_FEATHER) || defined(@FEATHER_ATLAS_BLIT)
SAMPLER_LINEAR(GAUSSIAN_INTEGRAL_TEXTURE_IDX, gaussianIntegralSampler)
#endif

#ifdef @FRAGMENT
FRAG_TEXTURE_BLOCK_BEGIN
TEXTURE_RGBA8(PER_FLUSH_BINDINGS_SET, GRAD_TEXTURE_IDX, @gradTexture);
#if defined(@ENABLE_FEATHER) || defined(@FEATHER_ATLAS_BLIT)
TEXTURE_R16F_1D_ARRAY(PER_FLUSH_BINDINGS_SET,
                      GAUSSIAN_INTEGRAL_TEXTURE_IDX,
                      @gaussianIntegralTexture);
#endif
#ifdef @FEATHER_ATLAS_BLIT
TEXTURE_R16F(PER_FLUSH_BINDINGS_SET,
             FEATHER_ATLAS_TEXTURE_IDX,
             @featherAtlasTexture);
#endif
TEXTURE_RGBA8(PER_DRAW_BINDINGS_SET, IMAGE_TEXTURE_IDX, @imageTexture);
// The Qualcomm compiler can't handle line breaks in #ifs.
// clang-format off
#if defined(@RENDER_MODE_MSAA) && defined(@ENABLE_ADVANCED_BLEND) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)
// clang-format on
DST_COLOR_TEXTURE(@dstColorTexture);
#endif
FRAG_TEXTURE_BLOCK_END

SAMPLER_LINEAR(GRAD_TEXTURE_IDX, gradSampler)
// Metal defines @VERTEX and @FRAGMENT at the same time, so yield to the vertex
// definition of gaussianIntegralSampler in this case.
#ifdef @FEATHER_ATLAS_BLIT
SAMPLER_LINEAR(FEATHER_ATLAS_TEXTURE_IDX, featherAtlasSampler)
#endif
DYNAMIC_SAMPLER_BLOCK_BEGIN
SAMPLER_DYNAMIC_IMAGE(imageSampler)
DYNAMIC_SAMPLER_BLOCK_END
#endif // @FRAGMENT

// We distinguish between strokes and fills by the sign of coverages.y,
// regardless of whether feathering is enabled (coverages are float4), or
// disabled (coverages are half2),
#ifdef @FRAGMENT
INLINE bool is_stroke(float4 coverages) { return coverages.y >= .0; }
INLINE bool is_stroke(half2 coverages) { return coverages.y >= .0; }
#endif // FRAGMENT

#if defined(@FRAGMENT) && defined(@ENABLE_FEATHER)
// We can also classify a fragments as feathered/not-feathered strokes/fills by
// looking at coverages.
INLINE bool is_feathered_stroke(float4 coverages)
{
    return coverages.x < FEATHER_COVERAGE_THRESHOLD;
}

INLINE bool is_feathered_fill(float4 coverages)
{
    return coverages.y < FEATHER_COVERAGE_THRESHOLD;
}
#endif // @FRAGMENT && @ENABLE_FEATHER

#ifdef @VERTEX
// Packs all the info to evaluate a feathered fill into 4 varying floats.
float4 pack_feathered_fill_coverages(float cornerTheta,
                                     float2 spokeNorm,
                                     float outset)
{
    // Find the corner's local coordinate within the feather convolution, where
    // the convolution matrix is centered on [.5, .5] and spans 0..1, and the
    // first edge of the corner runs horizontal and to the left.
    float2 cornerLocalCoord = (1. - spokeNorm * abs(outset)) * .5;

    // Calculate cotTheta and y0 for the fragment shader.
    // (See eval_feathered_fill() for details.)
    float cotTheta, y0;
    if (abs(cornerTheta - PI_OVER_2) < 1. / HORIZONTAL_COTANGENT_THRESHOLD)
    {
        cotTheta = .0;
        y0 = .0;
    }
    else
    {
        float tanTheta = tan(cornerTheta);
        cotTheta = sign(PI_OVER_2 - cornerTheta) /
                   max(abs(tanTheta), 1. / HORIZONTAL_COTANGENT_VALUE);
        y0 = cotTheta >= .0
                 ? cornerLocalCoord.y - (1. - cornerLocalCoord.x) * tanTheta
                 : cornerLocalCoord.y + cornerLocalCoord.x * tanTheta;
    }

    // Bias x & y for additional information:
    //
    //  * x will be negated later on if the triangle is back-facing. This tells
    //    the fragment shader what sign to give feathered coverage. Ensure it's
    //    greater than zero.
    //
    //  * y < FEATHER_COVERAGE_BIAS tells the fragment shader this is a feather.
    //
    float4 coverages;
    coverages.x = max(cornerLocalCoord.x, .0) + FEATHER_X_COORD_BIAS;
    coverages.y = -cornerLocalCoord.y + FEATHER_COVERAGE_BIAS;
    coverages.z = cotTheta;
    coverages.w = y0;
    return coverages;
}
#endif // @VERTEX

#ifdef @ENABLE_FEATHER
INLINE half eval_feathered_fill(float4 coverages TEXTURE_CONTEXT_DECL)
{
    // x and y are the relative coordinates of the corner vertex within the
    // feather convolution. They are oriented above center=[.5, .5], with the
    // first edge running horizontal and to the left.
    //
    // The second edge exits x,y at an angle of theta. "y0" is the location
    // where it intersects with either the left or right edge of the
    // convolution (left if cotTheta < 0, right if cotTheta > 0).
    //
    // y0 and cotTheta are both 0 when the corner angle is pi/2.
    half cotTheta = coverages.z;
    half y0 = max(coverages.w, .0); // Clamp y0 at the top of the convolution.

    // First compute the upper area of the convolution that is fully contained
    // within both edges. (i.e., the area above y0.)
    //
    // NOTE: If we aren't a corner, this will be the entire feather.
    half featherCoverage = cotTheta >= .0 ? FEATHER(y0) : .0;

    // If we're a (not flat) corner, the bottom area of the convolution needs to
    // account for both edges.
    //
    // Integrate the area contained within both edges, taking advantage of the
    // separability property of our convolution:
    //           _
    //    t=y   / \.
    //          |
    //          |  FEATHER(t * m + b) * FEATHER'(t) * dt
    //          |
    //   t=y0 \_/
    //
    // NOTE: The derivative FEATHER'(t) is the normal distribution with:
    //
    //   mu = 1/2
    //   sigma = 1 / (2 * GAUSSIAN_INTEGRAL_TEXTURE_STDDEVS)
    //
    // We can evaluate this directly without a lookup table.
    //
    // For performance constraints, we only take 4 samples on the integral.
    if (abs(cotTheta) < HORIZONTAL_COTANGENT_THRESHOLD)
    {
        // Unpack x & y from the varying coverages.
        half x = abs(coverages.x) - FEATHER_X_COORD_BIAS;
        half y = -coverages.y + FEATHER_COVERAGE_BIAS;

        // Find the height of each sample, and pre-scale by 1/(sigma*sqrt(2pi))
        // from the normal distribution (to save on multiplies later).
        //
        //   dt = (y - y0) / 4 / (sigma * sqrt(2pi))
        //
        half dt = (y - y0) * 0.5984134206;

        // Subdivide into 4 bars of even height, and sample at the centers.
        // (Reuse dt so we don't have to recompute "y - y1" again.)
        //
        //   t = lerp(y0, y1, [1/8, 3/8, 5/8, 7/8])
        //
        half4 t = y0 + dt * make_half4(0.20888568955,
                                       0.62665706865,
                                       1.04442844776,
                                       1.46219982687);

        // Feather horizontally where each sample intersects the second edge.
        half4 u = t * -cotTheta + (y * cotTheta + x);
        half4 feathers = make_half4(FEATHER(u[0]),
                                    FEATHER(u[1]),
                                    FEATHER(u[2]),
                                    FEATHER(u[3]));

        // Evaluate the normal distribution at each vertical sample.
        // (Scale t_ by sqrt(log2(e)) to change the base of the function from
        // e^x to 2^x.)
        //
        //   t_ = 1/sqrt(2) * (x - mu) / sigma * sqrt(log2(e))
        //   normalDistro = 2^(-t_ * t_)
        //
        half4 t_ = t * 5.09593080173 + -2.54796540086;
        half4 ddtFeather = exp2(-t_ * t_);

        // Take the sum of "FEATHER(u) * FEATHER'(t) * dt" at all 4 samples.
        featherCoverage += dot(feathers, ddtFeather) * dt;
    }

    // Clockwise triangles add to the featherCoverage, counterclockwise
    // triangles subtract from it.
    return featherCoverage * sign(coverages.x);
}

INLINE half eval_feathered_stroke(float4 coverages TEXTURE_CONTEXT_DECL)
{
    // Feathered stroke is:
    // 1 - feather(1 - leftCoverage) - feather(1 - rightCoverage)
    float featherCoverage = 1.;

    // The portion OUTSIDE the featherCoverage is "1 - featherCoverage".
    // (coverages.x is biased in order to classify this featherCoverage as a
    // feather, so also remove the bias.)
    float leftOutsideCoverage = (1. - FEATHER_COVERAGE_BIAS) + coverages.x;
    featherCoverage -= FEATHER(leftOutsideCoverage);

    float rightOutsideCoverage = 1. - coverages.y;
    featherCoverage -= FEATHER(rightOutsideCoverage);

    return featherCoverage;
}
#endif // @ENABLE_FEATHER

#if defined(@VERTEX) && defined(@DRAW_PATH)
INLINE int2 tess_texel_coord(int texelIndex)
{
    return int2(texelIndex & ((1 << TESS_TEXTURE_WIDTH_LOG2) - 1),
                texelIndex >> TESS_TEXTURE_WIDTH_LOG2);
}

INLINE float manhattan_pixel_width(float2x2 M, float2 normalized)
{

    float2 v = MUL(M, normalized);
    return (abs(v.x) + abs(v.y)) * (1. / dot(v, v));
}

INLINE bool unpack_tessellated_path_vertex(float4 patchVertexData,
                                           float4 mirroredVertexData,
                                           int _instanceID,
                                           OUT(uint) outPathID,
                                           OUT(float2) outVertexPosition
#ifndef @RENDER_MODE_MSAA
                                           ,
                                           OUT(float4) outCoverages
#else
                                           ,
                                           OUT(ushort) outPathZIndex
#endif
                                               VERTEX_CONTEXT_DECL)
{
    // Unpack patchVertexData.
    int localVertexID = int(patchVertexData.x);
    float outset = patchVertexData.y;
    float fillCoverage = patchVertexData.z;
    int patchSegmentSpan = floatBitsToInt(patchVertexData.w) >> 2;
    int vertexType = floatBitsToInt(patchVertexData.w) & 3;

    // Fetch a vertex that definitely belongs to the contour we're drawing.
    int vertexIDOnContour = min(localVertexID, patchSegmentSpan - 1);
    int tessVertexIdx = _instanceID * patchSegmentSpan + vertexIDOnContour;
    TESSDATA4 tessVertexData =
        TEXEL_FETCH(@tessVertexTexture, tess_texel_coord(tessVertexIdx));
    uint contourIDWithFlags = TESSDATA_AS_UINT(tessVertexData.w);

    // Fetch and unpack the contour referenced by the tessellation vertex.
    // NOTE: The contourID is guaranteed to be >= 1 at this point, but clamp it
    // anyway because in the event of a bug, a buffer load at index "0u - 1" can
    // be very serious and hard to catch.
    uint contourID = max(contourIDWithFlags & CONTOUR_ID_MASK, 1u);
    uint4 contourData = STORAGE_BUFFER_LOAD4(@contourBuffer, contourID - 1u);
    float2 midpoint = uintBitsToFloat(contourData.xy);
    outPathID = contourData.z & 0xffffu;
    uint vertexIndex0 = contourData.w;

    // Fetch and unpack the path.
    float2x2 M = make_float2x2(
        uintBitsToFloat(STORAGE_BUFFER_LOAD4(@pathBuffer, outPathID * 4u)));
    uint4 pathData = STORAGE_BUFFER_LOAD4(@pathBuffer, outPathID * 4u + 1u);
    float2 translate = uintBitsToFloat(pathData.xy);
    float strokeRadius = uintBitsToFloat(pathData.z);
    float featherRadius = uintBitsToFloat(pathData.w);

    // Fix the tessellation vertex if we fetched the wrong one in order to
    // guarantee we got the correct contour ID and flags, or if we belong to a
    // mirrored contour and this vertex has an alternate position when mirrored.
    uint mirroredContourFlag =
        contourIDWithFlags & MIRRORED_CONTOUR_CONTOUR_FLAG;
    if (mirroredContourFlag != 0u)
    {
        localVertexID = int(mirroredVertexData.x);
        outset = mirroredVertexData.y;
        fillCoverage = mirroredVertexData.z;
    }
    if (localVertexID != vertexIDOnContour)
    {
        // This can peek one vertex before or after the contour, but the
        // tessellator guarantees there is always at least one padding vertex at
        // the beginning and end of the data.
        int replacementTessVertexIdx =
            tessVertexIdx + localVertexID - vertexIDOnContour;
        TESSDATA4 replacementTessVertexData =
            TEXEL_FETCH(@tessVertexTexture,
                        tess_texel_coord(replacementTessVertexIdx));
        if ((TESSDATA_AS_UINT(replacementTessVertexData.w) &
             (MIRRORED_CONTOUR_CONTOUR_FLAG | 0xffffu)) !=
            (contourIDWithFlags & (MIRRORED_CONTOUR_CONTOUR_FLAG | 0xffffu)))
        {
            // We crossed over into a new contour. Either wrap to the first
            // vertex in the contour or leave it clamped at the final vertex of
            // the contour.
            bool isClosed = strokeRadius == .0 || // filled
                            midpoint.x != .0;     // explicity closed stroke
            if (isClosed)
            {
                tessVertexIdx = int(vertexIndex0);
                tessVertexData = TEXEL_FETCH(@tessVertexTexture,
                                             tess_texel_coord(tessVertexIdx));
            }
        }
        else
        {
            tessVertexIdx = replacementTessVertexIdx;
            tessVertexData = replacementTessVertexData;
        }
        // MIRRORED_CONTOUR_CONTOUR_FLAG is not preserved at vertexIndex0.
        // Preserve it here. By not preserving this flag, the normal and
        // mirrored contour can both share the same contour record.
        contourIDWithFlags = (TESSDATA_AS_UINT(tessVertexData.w) &
                              ~MIRRORED_CONTOUR_CONTOUR_FLAG) |
                             mirroredContourFlag;
    }

    // Find the tangent angle of the curve at our vertex.
    float theta;
#ifdef @ENABLE_FEATHER
    float featherJoinEdge0Theta;
    float featherJoinCornerTheta;
    if ((contourIDWithFlags & JOIN_TYPE_MASK) == FEATHER_JOIN_CONTOUR_FLAG &&
        vertexType == STROKE_VERTEX)
    {
        // Feather joins work out their stepping here in the vertex shader.
        // Instead of emitting just the tangent angle, the tessellation shader
        // gave us the original tessellation parameters.
        uint joinDataPacked = TESSDATA_AS_UINT(tessVertexData.z);
        float joinVertexID = float(joinDataPacked & 0xffffu);
        float joinSegmentCount = float(joinDataPacked >> 16);

        // Find the tessellation vertices immediately before and after the
        // feather join in order to work out the corner angles.
        int2 edgeVertexOffsets =
            int2(-joinVertexID - 1., joinSegmentCount - joinVertexID + 1.);
        if ((contourIDWithFlags & MIRRORED_CONTOUR_CONTOUR_FLAG) != 0u)
            edgeVertexOffsets = -edgeVertexOffsets;
        TESSDATA4 tessDataBeforeJoin =
            TEXEL_FETCH(@tessVertexTexture,
                        tess_texel_coord(tessVertexIdx + edgeVertexOffsets.x));
        TESSDATA4 tessDataAfterJoin =
            TEXEL_FETCH(@tessVertexTexture,
                        tess_texel_coord(tessVertexIdx + edgeVertexOffsets.y));
        if ((TESSDATA_AS_UINT(tessDataAfterJoin.w) &
             (MIRRORED_CONTOUR_CONTOUR_FLAG | 0xffffu)) !=
            (TESSDATA_AS_UINT(tessDataBeforeJoin.w) &
             (MIRRORED_CONTOUR_CONTOUR_FLAG | 0xffffu)))
        {
            // We reached over into a new contour. The edge immediately after
            // this feather join is actually the first vertex in the countour.
            tessDataAfterJoin =
                TEXEL_FETCH(@tessVertexTexture,
                            tess_texel_coord(int(vertexIndex0)));
        }

        featherJoinEdge0Theta = TESSDATA_AS_FLOAT(tessDataBeforeJoin.z);
        float featherJoinEdge1Theta = TESSDATA_AS_FLOAT(tessDataAfterJoin.z);
        featherJoinCornerTheta = featherJoinEdge1Theta - featherJoinEdge0Theta;
        if (abs(featherJoinCornerTheta) > PI)
            featherJoinCornerTheta -= _2PI * sign(featherJoinCornerTheta);

        // Feather joins draw backwards segments across the angle outside the
        // join, in order to erase some of the coverage that got written. Divide
        // the forward and backward segments proportionally to their respective
        // angles.
        float nonHelperSegmentCount =
            joinSegmentCount + 1. - float(FEATHER_JOIN_HELPER_VERTEX_COUNT);
        float forwardSegmentCount = clamp(
            round(abs(featherJoinCornerTheta) / PI * nonHelperSegmentCount),
            1.,
            nonHelperSegmentCount - 1.);
        float backwardSegmentCount =
            nonHelperSegmentCount - forwardSegmentCount;
        if (joinVertexID <= backwardSegmentCount)
        {
            // We're a backwards segment of the feather join.
            featherJoinCornerTheta =
                -(PI * sign(featherJoinCornerTheta) - featherJoinCornerTheta);
            joinSegmentCount = backwardSegmentCount;
            // On the final backward vertex, negate outset (later we will use
            // theta=featherJoinEdge1Theta instead of
            // featherJoinEdge1Theta - PI). This creates a crack-free
            // tessellation with the edge we're joining.
            if (joinVertexID == backwardSegmentCount)
                outset = -outset;
        }
        else if (joinVertexID == backwardSegmentCount + 1.)
        {
            // There's a discontinuous jump between the backward and forward
            // segments. This is a throwaway vertex to disconnect them.
            joinVertexID = .0;
            joinSegmentCount = .0;
            outset = .0;
        }
        else
        {
            // We're a forward segment of the feather join.
            joinVertexID -= backwardSegmentCount + 2.;
            joinSegmentCount = forwardSegmentCount;
        }

        if (joinVertexID == joinSegmentCount)
        {
            // Emit "featherJoinEdge1Theta" precisely (instead of the
            // approximate lerp below) to create crack-free tessellation with
            // the edges we're joining.
            theta = featherJoinEdge1Theta;
        }
        else
        {
            theta = featherJoinEdge0Theta +
                    featherJoinCornerTheta * (joinVertexID / joinSegmentCount);
        }
    }
    else
#endif // @ENABLE_FEATHER
    {
        theta = TESSDATA_AS_FLOAT(tessVertexData.z);
    }
    float2 norm = float2(sin(theta), -cos(theta));
    float2 origin = TESSDATA_AS_FLOAT(tessVertexData.xy);
    float2 postTransformVertexOffset = float2(0, 0);

    if (featherRadius != .0)
    {
        // Never use a feather harder than 1.5 standard deviations across a
        // radius of 1/2px. This is the point where feathering just looks like
        // antialiasing, and any harder looks aliased.
        featherRadius = max(featherRadius,
                            (GAUSSIAN_INTEGRAL_TEXTURE_STDDEVS / 3.) /
                                length(MUL(M, norm)));
    }

    if (strokeRadius != .0) // Is this a stroke?
    {
        // Ensure strokes always emit clockwise triangles.
        outset *= sign(determinant(M));

        // Joins only emanate from the outer side of the stroke.
        if ((contourIDWithFlags & LEFT_JOIN_CONTOUR_FLAG) != 0u)
            outset = min(outset, .0);
        if ((contourIDWithFlags & RIGHT_JOIN_CONTOUR_FLAG) != 0u)
            outset = max(outset, .0);

        float aaRadius = featherRadius != .0
                             ? featherRadius
                             : manhattan_pixel_width(M, norm) * AA_RADIUS;
        half globalCoverage = 1.;
        if (aaRadius > strokeRadius && featherRadius == .0)
        {
            // The stroke is narrower than the AA ramp. Instead of emitting
            // subpixel geometry, make the stroke as wide as the AA ramp and
            // apply a global coverage multiplier.
            globalCoverage =
                cast_float_to_half(strokeRadius) / cast_float_to_half(aaRadius);
            strokeRadius = aaRadius;
        }

        // Extend the vertex by half the width of the AA ramp.
        float2 vertexOffset =
            norm * (strokeRadius + aaRadius); // Bloat stroke width for AA.

#ifndef @RENDER_MODE_MSAA
        // Calculate the AA distance to both the outset and inset edges of the
        // stroke. The fragment shader will use whichever is lesser.
        float x = outset * (strokeRadius + aaRadius);
        outCoverages.xy =
            (1. / (aaRadius * 2.)) * (float2(x, -x) + strokeRadius) + .5;
        outCoverages.zw = make_float2(.0);
#endif

        uint joinType = contourIDWithFlags & JOIN_TYPE_MASK;
        if (joinType > ROUND_JOIN_CONTOUR_FLAG)
        {
            // This vertex belongs to a miter or bevel join. Begin by finding
            // the bisector, which is the same as the miter line. The first two
            // vertices in the join peek forward to figure out the bisector, and
            // the final two peek backward.
            int peekDir = 2;
            if ((contourIDWithFlags & JOIN_TANGENT_0_CONTOUR_FLAG) == 0u)
                peekDir = -peekDir;
            if ((contourIDWithFlags & MIRRORED_CONTOUR_CONTOUR_FLAG) != 0u)
                peekDir = -peekDir;
            int2 otherJoinTexelCoord =
                tess_texel_coord(tessVertexIdx + peekDir);
            TESSDATA4 otherJoinData =
                TEXEL_FETCH(@tessVertexTexture, otherJoinTexelCoord);
            float otherJoinTheta = TESSDATA_AS_FLOAT(otherJoinData.z);
            float joinAngle = abs(otherJoinTheta - theta);
            if (joinAngle > PI)
                joinAngle = _2PI - joinAngle;
            bool isTan0 =
                (contourIDWithFlags & JOIN_TANGENT_0_CONTOUR_FLAG) != 0u;
            bool isLeftJoin =
                (contourIDWithFlags & LEFT_JOIN_CONTOUR_FLAG) != 0u;
            float bisectTheta =
                joinAngle * (isTan0 == isLeftJoin ? -.5 : .5) + theta;
            float2 bisector = float2(sin(bisectTheta), -cos(bisectTheta));
            float bisectPixelWidth = manhattan_pixel_width(M, bisector);

            // Generalize everything to a "miter-clip", which is proposed in the
            // SVG-2 draft. Bevel joins are converted to miter-clip joins with a
            // miter limit of 1/2 pixel. They technically bleed out 1/2 pixel
            // when drawn this way, but they seem to look fine and there is not
            // an obvious solution to antialias them without an ink bleed.
            float miterRatio = cos(joinAngle * .5);
            float clipRadius;
            if ((joinType == MITER_CLIP_JOIN_CONTOUR_FLAG) ||
                (joinType == MITER_REVERT_JOIN_CONTOUR_FLAG &&
                 miterRatio >= .25))
            {
                // Miter! (Or square cap.)
                // We currently use hard coded miter limits:
                //   * 1 for square caps being emulated as miter-clip joins.
                //   * 4, which is the SVG default, for all other miter joins.
                float miterInverseLimit =
                    (contourIDWithFlags & EMULATED_STROKE_CAP_CONTOUR_FLAG) !=
                            0u
                        ? 1.
                        : .25;
                clipRadius =
                    strokeRadius * (1. / max(miterRatio, miterInverseLimit));
            }
            else
            {
                // Bevel! (Or butt cap.)
                clipRadius = strokeRadius * miterRatio +
                             /* 1/2px bleed! */ bisectPixelWidth * .5;
            }
            float clipAARadius = clipRadius + bisectPixelWidth * AA_RADIUS;
            if ((contourIDWithFlags & JOIN_TANGENT_INNER_CONTOUR_FLAG) != 0u)
            {
                // Reposition the inner join vertices at the miter-clip
                // positions. Leave the outer join vertices as duplicates on the
                // surrounding curve endpoints. We emit duplicate vertex
                // positions because we need a hard stop on the clip distance
                // (see below).
                //
                // Use aaRadius here because we're tracking AA on the mitered
                // edge, NOT the outer clip edge.
                float strokeAARaidus = strokeRadius + aaRadius;
                // clipAARadius must be 1/16 of an AA ramp (~1/16 pixel) longer
                // than the miter length before we start clipping, to ensure we
                // are solving for a numerically stable intersection.
                float slop = aaRadius * .125;
                if (strokeAARaidus <= clipAARadius * miterRatio + slop)
                {
                    // The miter point is before the clip line. Extend out to
                    // the miter point.
                    float miterAARadius = strokeAARaidus * (1. / miterRatio);
                    vertexOffset = bisector * miterAARadius;
                }
                else
                {
                    // The clip line is before the miter point. Find where the
                    // clip line and the mitered edge intersect.
                    float2 bisectAAOffset = bisector * clipAARadius;
                    float2 k = float2(dot(vertexOffset, vertexOffset),
                                      dot(bisectAAOffset, bisectAAOffset));
                    vertexOffset =
                        MUL(k, inverse(float2x2(vertexOffset, bisectAAOffset)));
                }
            }
            // The clip distance tells us how to antialias the outer clipped
            // edge. Since joins only emanate from the outset side of the
            // stroke, we can repurpose the inset distance as the clip distance.
            float2 pt = abs(outset) * vertexOffset;
            float clipDistance = (clipAARadius - dot(pt, bisector)) /
                                 (bisectPixelWidth * (AA_RADIUS * 2.));
#ifndef @RENDER_MODE_MSAA
            if ((contourIDWithFlags & LEFT_JOIN_CONTOUR_FLAG) != 0u)
                outCoverages.y = clipDistance;
            else
                outCoverages.x = clipDistance;
#endif
        }

#ifndef @RENDER_MODE_MSAA
        outCoverages.xy *= globalCoverage;

        // Bias outCoverages.y slightly upwards in order to guarantee
        // outCoverages.y is >= 0 at every pixel. "outCoverages.y < 0" is
        // used to differentiate between strokes and fills.
        outCoverages.y = max(outCoverages.y, 1e-4);

        if (featherRadius != .0)
        {
            // Bias x to tell the fragment shader that this is a feathered
            // stroke.
            outCoverages.x = FEATHER_COVERAGE_BIAS - outCoverages.x;
        }
#endif

        postTransformVertexOffset = MUL(M, outset * vertexOffset);

        // Throw away the fan triangles since we're a stroke.
        if (vertexType != STROKE_VERTEX)
            return false;
    }
    else // This is a fill.
    {
#ifndef @RENDER_MODE_MSAA
        // "outCoverages.y < 0" indicates to the fragment shader that this is
        // a fill, as opposed to a stroke.
        outCoverages = float4(fillCoverage, -1., .0, .0);

#ifdef @ENABLE_FEATHER
        if (featherRadius != .0)
        {
            // Bias y to tell the fragment shader that this is a feathered edge.
            outCoverages.y = FEATHER_COVERAGE_BIAS;

            // "outCoverages.z = HORIZONTAL_COTANGENT_VALUE" initializes us
            // in a default state of feathering a flat edge (as opposed to a
            // corner).
            outCoverages.z = HORIZONTAL_COTANGENT_VALUE;

            // eval_feathered_fill() just feathers outCoverages.w=y0 when
            // we're a flat edge, so initialize it with fillCoverage.
            outCoverages.w = fillCoverage;

            if ((contourIDWithFlags & JOIN_TYPE_MASK) ==
                    FEATHER_JOIN_CONTOUR_FLAG &&
                vertexType == STROKE_VERTEX)
            {
                // Feathered corners are symmetric; swap the first and second
                // edge if needed so the corner angle is always positive.
                if (featherJoinCornerTheta < .0)
                {
                    featherJoinEdge0Theta += featherJoinCornerTheta;
                    featherJoinCornerTheta = -featherJoinCornerTheta;
                }

                // Find the angle and local outset direction of our specific
                // spoke in the feather join, relative to the first edge. Take
                // advantage of the fact that feathered corners are symmetric
                // again, and limit spokeTheta to the first half of the join
                // angle.
                float spokeTheta = theta - featherJoinEdge0Theta;
                spokeTheta = mod(spokeTheta + PI_OVER_2, _2PI) - PI_OVER_2;
                spokeTheta = clamp(spokeTheta, .0, featherJoinCornerTheta);
                if (spokeTheta > featherJoinCornerTheta * .5)
                {
                    spokeTheta = featherJoinCornerTheta - spokeTheta;
                }
                float2 spokeNorm = float2(sin(spokeTheta), cos(spokeTheta));

                // TODO: This contraction logic generates cracks in geometry. It
                // needs more investigation.
#if 0
                // When coners have stong curvature, their feather diminishes
                // faster than it does for flat edges. In this scenario we can
                // contract the tessellation a little to save on performance
                // without losing visual fidelity.
                //
                // This code attempts to be somewhat methodical, but it's just
                // hackery. The idea is to measure actual feather coverage at an
                // outset of N standard deviations, compare that to what
                // coverage would have been for a flat edge, and contract
                // accordingly. By observation, a logarithmic function of
                // featherJoinCornerTheta gives values for N with a good balance
                // of perf and quality.
                float N =
                    1. + .33 * log2(PI_OVER_2 /
                            (PI - min(featherJoinCornerTheta, PI - PI / 16.)));
                float4 coveragesAtNStddevOutset =
                    pack_feathered_fill_coverages(featherJoinCornerTheta,
                            spokeNorm,
                            .5 * (N / 3.));
                float featherAtNStddevOutset = eval_feathered_fill(
                        coveragesAtNStddevOutset TEXTURE_CONTEXT_FORWARD);
                float inverseFeather =
                    INVERSE_FEATHER(featherAtNStddevOutset);
                float stddevsAwayFromCenter =
                    (.5 - inverseFeather) * (GAUSSIAN_INTEGRAL_TEXTURE_STDDEVS * 2.);
                float contraction = N / max(stddevsAwayFromCenter, N);
                outset *= contraction;
#endif

                // Emit coverage values for the fragment shader.
                outCoverages =
                    pack_feathered_fill_coverages(featherJoinCornerTheta,
                                                  spokeNorm,
                                                  outset);
            }
            // Offset the vertex for feathering.
            postTransformVertexOffset = MUL(M, (outset * featherRadius) * norm);
        }
        else
#endif // @ENABLE_FEATHER
        {
            // Offset the vertex for Manhattan AA.
            postTransformVertexOffset =
                sign(MUL(outset * norm, inverse(M))) * AA_RADIUS;
        }

        if (bool(contourIDWithFlags & MIRRORED_CONTOUR_CONTOUR_FLAG) !=
            bool(contourIDWithFlags & NEGATE_PATH_FILL_COVERAGE_FLAG))
        {
            // Effectively: outCoverages.x = -outCoverages.x
            //
            // ... But don't write that because it hits a bug in the Mali T720
            // compiler that also negates Y.
            outCoverages *= float4(-1., +1., +1., +1.);
        }
#endif // !RENDER_MODE_MSAA

        // Place the fan point.
        if (vertexType == FAN_MIDPOINT_VERTEX)
            origin = midpoint;

        // If we're actually just drawing a triangle, throw away the entire
        // patch except a single fan triangle.
        if ((contourIDWithFlags & RETROFITTED_TRIANGLE_CONTOUR_FLAG) != 0u &&
            vertexType != FAN_VERTEX)
        {
            return false;
        }
    }

    outVertexPosition = MUL(M, origin) + postTransformVertexOffset + translate;

#ifdef @RENDER_MODE_MSAA
    uint4 pathData2 = STORAGE_BUFFER_LOAD4(@pathBuffer, outPathID * 4u + 2u);
    outPathZIndex = cast_uint_to_ushort(pathData2.r);
#else
    // Force coverage to solid when wireframe is enabled so we can see the
    // triangles.
    outCoverages.xy = mix(outCoverages.xy,
                          float2(1., -1.),
                          make_bool2(uniforms.wireframeEnabled != 0u));
#endif

    return true;
}
#endif // @VERTEX && @DRAW_PATH

#if defined(@VERTEX) && defined(@DRAW_INTERIOR_TRIANGLES)
INLINE float2 unpack_interior_triangle_vertex(float3 triangleVertex,
                                              OUT(uint) outPathID
#ifdef @RENDER_MODE_MSAA
                                              ,
                                              OUT(ushort) outPathZIndex
#else
                                              ,
                                              OUT(half) outWindingWeight
#endif
                                                  VERTEX_CONTEXT_DECL)
{
    outPathID = floatBitsToUint(triangleVertex.z) & 0xffffu;
#ifdef @RENDER_MODE_MSAA
    uint4 pathData2 = STORAGE_BUFFER_LOAD4(@pathBuffer, outPathID * 4u + 2u);
    outPathZIndex = cast_uint_to_ushort(pathData2.x);
#else
    outWindingWeight = cast_int_to_half(floatBitsToInt(triangleVertex.z) >> 16);
#endif
    float2 vertexPos = triangleVertex.xy;
    // FEATHER_ATLAS_BLIT draws vertices in screen space.
    float2x2 M = make_float2x2(
        uintBitsToFloat(STORAGE_BUFFER_LOAD4(@pathBuffer, outPathID * 4u)));
    uint4 pathData = STORAGE_BUFFER_LOAD4(@pathBuffer, outPathID * 4u + 1u);
    float2 translate = uintBitsToFloat(pathData.xy);
    vertexPos = MUL(M, vertexPos) + translate;
    return vertexPos;
}
#endif // @VERTEX && @DRAW_INTERIOR_TRIANGLES

#if defined(@VERTEX) && defined(@FEATHER_ATLAS_BLIT)
INLINE float2
unpack_atlas_coverage_vertex(float3 triangleVertex,
                             OUT(uint) outPathID,
#ifdef @RENDER_MODE_MSAA
                             OUT(ushort) outPathZIndex,
#endif
                             OUT(float2) outAtlasCoord VERTEX_CONTEXT_DECL)
{
    outPathID = floatBitsToUint(triangleVertex.z) & 0xffffu;
    uint4 pathData2 = STORAGE_BUFFER_LOAD4(@pathBuffer, outPathID * 4u + 2u);
#ifdef @RENDER_MODE_MSAA
    outPathZIndex = cast_uint_to_ushort(pathData2.x);
#endif
    float2 vertexPos = triangleVertex.xy;
    // outAtlasCoord tells the fragment shader where to fetch coverage from the
    // atlas, when using atlas coverage.
    float3 atlasTransform = uintBitsToFloat(pathData2.yzw);
    outAtlasCoord = (vertexPos * atlasTransform.x + atlasTransform.yz) *
                    uniforms.atlasTextureInverseSize;
    return vertexPos;
}
#endif // @VERTEX && @FEATHER_ATLAS_BLIT

// Calculates a coverage value to multiply into the paintColor that will
// convert the current framebuffer value from "paint blended on top with
// coverage of c0" to "paint blended on top with coverage of c1".
//
// i.e., The paint has already been blended into the framebuffer with coverage
// "c0". After this fragment blends, it will be equivalent to the paint having
// been blended into the framebuffer with coverage "c1".
//
// NOTE: c1 must be > c0, which is why this is only applicable in clockwise
// modes.
INLINE half incremental_clockwise_coverage(half c0, half c1, half paintAlpha)
{
    // NOTE: "max(, eps)" is just to avoid a divide by zero. When the
    // denominator would be 0, c0 == 1, which also means c1 == 1, and there is
    // no coverage to apply. Since c0 == c1 == 1, (c1 - c0) / eps == 0, which is
    // the result we want in this case.
    return (c1 - c0) / max(1. - c0 * paintAlpha, EPSILON_FP16_NON_DENORM);
}

// Converts an x,y image coordinate into a buffer index, swizzling into
// BUFFER_IMAGE_TILE_SIZE x BUFFER_IMAGE_TILE_SIZE tiles for better cache
// performance.
// imageWidth must be a multiple of BUFFER_IMAGE_TILE_SIZE.
INLINE uint swizzle_image_buffer_idx(uint2 imageCoord, uint imageWidth)
{
    uint idx = (imageCoord.y >> BUFFER_IMAGE_TILE_SIZE_LOG2) *
                   (imageWidth << BUFFER_IMAGE_TILE_SIZE_LOG2) +
               ((imageCoord.x >> BUFFER_IMAGE_TILE_SIZE_LOG2)
                << (BUFFER_IMAGE_TILE_SIZE_LOG2 << 1));
    // Subdivide each main tile into 4x4 column-major tiles.
    idx += ((imageCoord.x & 0x1cu) << BUFFER_IMAGE_TILE_SIZE_LOG2) +
           ((imageCoord.y & 0x1cu) << 2);
    // Let the 4x4 tiles be row-major.
    idx += ((imageCoord.y & 0x3u) << 2) + (imageCoord.x & 0x3u);
    return idx;
}

#ifdef @RENDER_MODE_CLOCKWISE_ATOMIC

#ifdef @FIXED_FUNCTION_COLOR_OUTPUT
#define CLOCKWISE_ATOMIC_PLS_MAIN PLS_FRAG_COLOR_MAIN
#define EMIT_CLOCKWISE_ATOMIC_PLS(FRAG_COLOR)                                  \
    _fragColor = FRAG_COLOR;                                                   \
    EMIT_PLS_AND_FRAG_COLOR
#else
#define CLOCKWISE_ATOMIC_PLS_MAIN PLS_MAIN
#define EMIT_CLOCKWISE_ATOMIC_PLS(FRAG_COLOR)                                  \
    PLS_STORE4F(colorBuffer, FRAG_COLOR);                                      \
    EMIT_PLS;
#endif

// Extracts coverage from its fixed-point encoding in a coverage buffer value.
INLINE half clockwise_atomic_fixed_to_coverage(uint coverageFixed)
{
    return cast_int_to_half(int((coverageFixed & CLOCKWISE_COVERAGE_MASK) -
                                CLOCKWISE_FILL_ZERO_VALUE)) *
           CLOCKWISE_COVERAGE_INVERSE_PRECISION;
}

// Converts a coverage to a fixed point delta that may be added to a coverage
// buffer value.
// NOTE: This is not the same as converting it to a plain coverage value, since
// those must be biased by CLOCKWISE_FILL_ZERO_VALUE.
INLINE uint clockwise_atomic_coverage_delta_to_fixed(half coverage)
{
    return uint(coverage * CLOCKWISE_COVERAGE_PRECISION + .5);
}

#endif // @RENDER_MODE_CLOCKWISE_ATOMIC
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_DRAW_PATH_COMMON_SOURCE: &str = PINNED_DRAW_PATH_COMMON_GLSL_SOURCE;
pub const DRAW_PATH_COMMON_GLSL_SOURCE: &str = PINNED_DRAW_PATH_COMMON_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_DRAW_PATH_COMMON_GLSL_SOURCE
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
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
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    target_path: TRANSLATION_TARGET,
    translation_unit: TRANSLATION_UNIT,
    translation_disposition: "full-translation-source / source-shaped provenance",
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
        block_id: "pp-0441",
        block_start: 27,
        block_end: 45,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0442",
        block_start: 32,
        block_end: 36,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0443",
        block_start: 47,
        block_end: 49,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0444",
        block_start: 51,
        block_end: 82,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0445",
        block_start: 54,
        block_end: 58,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0446",
        block_start: 59,
        block_end: 63,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0447",
        block_start: 67,
        block_end: 70,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0448",
        block_start: 76,
        block_end: 78,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0449",
        block_start: 87,
        block_end: 90,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0450",
        block_start: 92,
        block_end: 104,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0451",
        block_start: 106,
        block_end: 150,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0452",
        block_start: 152,
        block_end: 259,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0453",
        block_start: 261,
        block_end: 790,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0454",
        block_start: 280,
        block_end: 286,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0455",
        block_start: 373,
        block_end: 470,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0456",
        block_start: 517,
        block_end: 524,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0457",
        block_start: 624,
        block_end: 629,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0458",
        block_start: 632,
        block_end: 646,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0459",
        block_start: 656,
        block_end: 760,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0460",
        block_start: 661,
        block_end: 744,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0461",
        block_start: 704,
        block_end: 732,
        block_depth: 3,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0462",
        block_start: 777,
        block_end: 786,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0463",
        block_start: 792,
        block_end: 820,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0464",
        block_start: 795,
        block_end: 801,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0465",
        block_start: 805,
        block_end: 810,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0466",
        block_start: 822,
        block_end: 844,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0467",
        block_start: 826,
        block_end: 828,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0468",
        block_start: 833,
        block_end: 835,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0469",
        block_start: 883,
        block_end: 914,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0470",
        block_start: 885,
        block_end: 895,
        block_depth: 1,
        branch_count: 2,
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
        block_id: "pp-0441",
        branch_ordinal: 1,
        branch_line: 27,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0442",
        branch_ordinal: 1,
        branch_line: 32,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@VERTEX)) && (defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0443",
        branch_ordinal: 1,
        branch_line: 47,
        directive: "#if defined(@ENABLE_FEATHER) || defined(@FEATHER_ATLAS_BLIT)",
        active_branch_path: "(defined(@ENABLE_FEATHER) || defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0444",
        branch_ordinal: 1,
        branch_line: 51,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0445",
        branch_ordinal: 1,
        branch_line: 54,
        directive: "#if defined(@ENABLE_FEATHER) || defined(@FEATHER_ATLAS_BLIT)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@ENABLE_FEATHER) || defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0446",
        branch_ordinal: 1,
        branch_line: 59,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0447",
        branch_ordinal: 1,
        branch_line: 67,
        directive: "#if defined(@RENDER_MODE_MSAA) && defined(@ENABLE_ADVANCED_BLEND) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT)",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@RENDER_MODE_MSAA) && defined(@ENABLE_ADVANCED_BLEND) && !defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0448",
        branch_ordinal: 1,
        branch_line: 76,
        directive: "#ifdef @FEATHER_ATLAS_BLIT",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0449",
        branch_ordinal: 1,
        branch_line: 87,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0450",
        branch_ordinal: 1,
        branch_line: 92,
        directive: "#if defined(@FRAGMENT) && defined(@ENABLE_FEATHER)",
        active_branch_path: "(defined(@FRAGMENT) && defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0451",
        branch_ordinal: 1,
        branch_line: 106,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0452",
        branch_ordinal: 1,
        branch_line: 152,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0453",
        branch_ordinal: 1,
        branch_line: 261,
        directive: "#if defined(@VERTEX) && defined(@DRAW_PATH)",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH))",
    },
    ConditionalBranch {
        block_id: "pp-0454",
        branch_ordinal: 1,
        branch_line: 280,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0454",
        branch_ordinal: 2,
        branch_line: 283,
        directive: "#else",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (!((!defined(@RENDER_MODE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0455",
        branch_ordinal: 1,
        branch_line: 373,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0456",
        branch_ordinal: 1,
        branch_line: 517,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0457",
        branch_ordinal: 1,
        branch_line: 624,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0458",
        branch_ordinal: 1,
        branch_line: 632,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0459",
        branch_ordinal: 1,
        branch_line: 656,
        directive: "#ifndef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (!defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0460",
        branch_ordinal: 1,
        branch_line: 661,
        directive: "#ifdef @ENABLE_FEATHER",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (!defined(@RENDER_MODE_MSAA)) && (defined(@ENABLE_FEATHER))",
    },
    ConditionalBranch {
        block_id: "pp-0461",
        branch_ordinal: 1,
        branch_line: 704,
        directive: "#if 0",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (!defined(@RENDER_MODE_MSAA)) && (defined(@ENABLE_FEATHER)) && (0)",
    },
    ConditionalBranch {
        block_id: "pp-0462",
        branch_ordinal: 1,
        branch_line: 777,
        directive: "#ifdef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0462",
        branch_ordinal: 2,
        branch_line: 780,
        directive: "#else",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_PATH)) && (!((defined(@RENDER_MODE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0463",
        branch_ordinal: 1,
        branch_line: 792,
        directive: "#if defined(@VERTEX) && defined(@DRAW_INTERIOR_TRIANGLES)",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_INTERIOR_TRIANGLES))",
    },
    ConditionalBranch {
        block_id: "pp-0464",
        branch_ordinal: 1,
        branch_line: 795,
        directive: "#ifdef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_INTERIOR_TRIANGLES)) && (defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0464",
        branch_ordinal: 2,
        branch_line: 798,
        directive: "#else",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_INTERIOR_TRIANGLES)) && (!((defined(@RENDER_MODE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0465",
        branch_ordinal: 1,
        branch_line: 805,
        directive: "#ifdef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_INTERIOR_TRIANGLES)) && (defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0465",
        branch_ordinal: 2,
        branch_line: 808,
        directive: "#else",
        active_branch_path: "(defined(@VERTEX) && defined(@DRAW_INTERIOR_TRIANGLES)) && (!((defined(@RENDER_MODE_MSAA))))",
    },
    ConditionalBranch {
        block_id: "pp-0466",
        branch_ordinal: 1,
        branch_line: 822,
        directive: "#if defined(@VERTEX) && defined(@FEATHER_ATLAS_BLIT)",
        active_branch_path: "(defined(@VERTEX) && defined(@FEATHER_ATLAS_BLIT))",
    },
    ConditionalBranch {
        block_id: "pp-0467",
        branch_ordinal: 1,
        branch_line: 826,
        directive: "#ifdef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX) && defined(@FEATHER_ATLAS_BLIT)) && (defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0468",
        branch_ordinal: 1,
        branch_line: 833,
        directive: "#ifdef @RENDER_MODE_MSAA",
        active_branch_path: "(defined(@VERTEX) && defined(@FEATHER_ATLAS_BLIT)) && (defined(@RENDER_MODE_MSAA))",
    },
    ConditionalBranch {
        block_id: "pp-0469",
        branch_ordinal: 1,
        branch_line: 883,
        directive: "#ifdef @RENDER_MODE_CLOCKWISE_ATOMIC",
        active_branch_path: "(defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
    },
    ConditionalBranch {
        block_id: "pp-0470",
        branch_ordinal: 1,
        branch_line: 885,
        directive: "#ifdef @FIXED_FUNCTION_COLOR_OUTPUT",
        active_branch_path: "(defined(@RENDER_MODE_CLOCKWISE_ATOMIC)) && (defined(@FIXED_FUNCTION_COLOR_OUTPUT))",
    },
    ConditionalBranch {
        block_id: "pp-0470",
        branch_ordinal: 2,
        branch_line: 890,
        directive: "#else",
        active_branch_path: "(defined(@RENDER_MODE_CLOCKWISE_ATOMIC)) && (!((defined(@FIXED_FUNCTION_COLOR_OUTPUT))))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The direct @-prefixed identifiers occurring in draw_path_common.glsl,
/// retained in first-occurrence source order.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 27,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 31,
        source_name: "@tessVertexTexture",
        generated_name: "DC",
        generated_header_name: "GLSL_tessVertexTexture",
    },
    ExportedSymbol {
        source_line: 32,
        source_name: "@ENABLE_FEATHER",
        generated_name: "HB",
        generated_header_name: "GLSL_ENABLE_FEATHER",
    },
    ExportedSymbol {
        source_line: 35,
        source_name: "@gaussianIntegralTexture",
        generated_name: "QC",
        generated_header_name: "GLSL_featherTexture",
    },
    ExportedSymbol {
        source_line: 40,
        source_name: "@pathBuffer",
        generated_name: "MB",
        generated_header_name: "GLSL_pathBuffer",
    },
    ExportedSymbol {
        source_line: 41,
        source_name: "@paintBuffer",
        generated_name: "TC",
        generated_header_name: "GLSL_paintBuffer",
    },
    ExportedSymbol {
        source_line: 42,
        source_name: "@paintAuxBuffer",
        generated_name: "PB",
        generated_header_name: "GLSL_paintAuxBuffer",
    },
    ExportedSymbol {
        source_line: 43,
        source_name: "@contourBuffer",
        generated_name: "XC",
        generated_header_name: "GLSL_contourBuffer",
    },
    ExportedSymbol {
        source_line: 47,
        source_name: "@FEATHER_ATLAS_BLIT",
        generated_name: "EB",
        generated_header_name: "GLSL_FEATHER_ATLAS_BLIT",
    },
    ExportedSymbol {
        source_line: 51,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 53,
        source_name: "@gradTexture",
        generated_name: "DD",
        generated_header_name: "GLSL_gradTexture",
    },
    ExportedSymbol {
        source_line: 62,
        source_name: "@featherAtlasTexture",
        generated_name: "UC",
        generated_header_name: "GLSL_atlasTexture",
    },
    ExportedSymbol {
        source_line: 64,
        source_name: "@imageTexture",
        generated_name: "AC",
        generated_header_name: "GLSL_imageTexture",
    },
    ExportedSymbol {
        source_line: 67,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 67,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 67,
        source_name: "@RENDER_MODE_MSAA",
        generated_name: "BB",
        generated_header_name: "GLSL_RENDER_MODE_MSAA",
    },
    ExportedSymbol {
        source_line: 69,
        source_name: "@dstColorTexture",
        generated_name: "LD",
        generated_header_name: "GLSL_dstColorTexture",
    },
    ExportedSymbol {
        source_line: 261,
        source_name: "@DRAW_PATH",
        generated_name: "BD",
        generated_header_name: "GLSL_DRAW_PATH",
    },
    ExportedSymbol {
        source_line: 792,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 883,
        source_name: "@RENDER_MODE_CLOCKWISE_ATOMIC",
        generated_name: "QB",
        generated_header_name: "GLSL_RENDER_MODE_CLOCKWISE_ATOMIC",
    },
];

/// The preprocessor switch subset of EXPORTED_SYMBOLS.
pub const EXPORTED_SWITCHES: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 27,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 32,
        source_name: "@ENABLE_FEATHER",
        generated_name: "HB",
        generated_header_name: "GLSL_ENABLE_FEATHER",
    },
    ExportedSymbol {
        source_line: 47,
        source_name: "@FEATHER_ATLAS_BLIT",
        generated_name: "EB",
        generated_header_name: "GLSL_FEATHER_ATLAS_BLIT",
    },
    ExportedSymbol {
        source_line: 51,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 67,
        source_name: "@ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
        generated_header_name: "GLSL_ENABLE_ADVANCED_BLEND",
    },
    ExportedSymbol {
        source_line: 67,
        source_name: "@FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
        generated_header_name: "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
    },
    ExportedSymbol {
        source_line: 67,
        source_name: "@RENDER_MODE_MSAA",
        generated_name: "BB",
        generated_header_name: "GLSL_RENDER_MODE_MSAA",
    },
    ExportedSymbol {
        source_line: 261,
        source_name: "@DRAW_PATH",
        generated_name: "BD",
        generated_header_name: "GLSL_DRAW_PATH",
    },
    ExportedSymbol {
        source_line: 792,
        source_name: "@DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
        generated_header_name: "GLSL_DRAW_INTERIOR_TRIANGLES",
    },
    ExportedSymbol {
        source_line: 883,
        source_name: "@RENDER_MODE_CLOCKWISE_ATOMIC",
        generated_name: "QB",
        generated_header_name: "GLSL_RENDER_MODE_CLOCKWISE_ATOMIC",
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

/// Function declarations are retained as source spellings and ranges. Their
/// bodies remain in PINNED_DRAW_PATH_COMMON_GLSL_SOURCE rather than being
/// translated into executable Rust.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 88,
        end_line: 88,
        name: "is_stroke",
        signature: "INLINE bool is_stroke(float4 coverages)",
        guard_path: "(defined(@FRAGMENT))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 89,
        end_line: 89,
        name: "is_stroke",
        signature: "INLINE bool is_stroke(half2 coverages)",
        guard_path: "(defined(@FRAGMENT))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 95,
        end_line: 98,
        name: "is_feathered_stroke",
        signature: "INLINE bool is_feathered_stroke(float4 coverages)",
        guard_path: "(defined(@FRAGMENT) && defined(@ENABLE_FEATHER))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 100,
        end_line: 103,
        name: "is_feathered_fill",
        signature: "INLINE bool is_feathered_fill(float4 coverages)",
        guard_path: "(defined(@FRAGMENT) && defined(@ENABLE_FEATHER))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 108,
        end_line: 149,
        name: "pack_feathered_fill_coverages",
        signature: "float4 pack_feathered_fill_coverages(float cornerTheta, float2 spokeNorm, float outset)",
        guard_path: "(defined(@VERTEX))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 153,
        end_line: 240,
        name: "eval_feathered_fill",
        signature: "INLINE half eval_feathered_fill(float4 coverages TEXTURE_CONTEXT_DECL)",
        guard_path: "(defined(@ENABLE_FEATHER))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 242,
        end_line: 258,
        name: "eval_feathered_stroke",
        signature: "INLINE half eval_feathered_stroke(float4 coverages TEXTURE_CONTEXT_DECL)",
        guard_path: "(defined(@ENABLE_FEATHER))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 262,
        end_line: 266,
        name: "tess_texel_coord",
        signature: "INLINE int2 tess_texel_coord(int texelIndex)",
        guard_path: "(defined(@VERTEX) && defined(@DRAW_PATH))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 268,
        end_line: 273,
        name: "manhattan_pixel_width",
        signature: "INLINE float manhattan_pixel_width(float2x2 M, float2 normalized)",
        guard_path: "(defined(@VERTEX) && defined(@DRAW_PATH))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 275,
        end_line: 789,
        name: "unpack_tessellated_path_vertex",
        signature: "INLINE bool unpack_tessellated_path_vertex(float4 patchVertexData, float4 mirroredVertexData, int _instanceID, OUT(uint) outPathID, OUT(float2) outVertexPosition VERTEX_CONTEXT_DECL)",
        guard_path: "(defined(@VERTEX) && defined(@DRAW_PATH))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 793,
        end_line: 819,
        name: "unpack_interior_triangle_vertex",
        signature: "INLINE float2 unpack_interior_triangle_vertex(float3 triangleVertex, OUT(uint) outPathID VERTEX_CONTEXT_DECL)",
        guard_path: "(defined(@VERTEX) && defined(@DRAW_INTERIOR_TRIANGLES))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 823,
        end_line: 843,
        name: "unpack_atlas_coverage_vertex",
        signature: "INLINE float2 unpack_atlas_coverage_vertex(float3 triangleVertex, OUT(uint) outPathID, OUT(float2) outAtlasCoord VERTEX_CONTEXT_DECL)",
        guard_path: "(defined(@VERTEX) && defined(@FEATHER_ATLAS_BLIT))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 856,
        end_line: 863,
        name: "incremental_clockwise_coverage",
        signature: "INLINE half incremental_clockwise_coverage(half c0, half c1, half paintAlpha)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 869,
        end_line: 881,
        name: "swizzle_image_buffer_idx",
        signature: "INLINE uint swizzle_image_buffer_idx(uint2 imageCoord, uint imageWidth)",
        guard_path: "all",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 898,
        end_line: 903,
        name: "clockwise_atomic_fixed_to_coverage",
        signature: "INLINE half clockwise_atomic_fixed_to_coverage(uint coverageFixed)",
        guard_path: "(defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
        inline_qualifier: "INLINE",
    },
    ShaderFunction {
        source_line: 909,
        end_line: 912,
        name: "clockwise_atomic_coverage_delta_to_fixed",
        signature: "INLINE uint clockwise_atomic_coverage_delta_to_fixed(half coverage)",
        guard_path: "(defined(@RENDER_MODE_CLOCKWISE_ATOMIC))",
        inline_qualifier: "INLINE",
    },
];

pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedIdentifier {
    pub source_name: &'static str,
    pub generated_name: &'static str,
}

/// Direct export inventory with source spellings (without the leading @) and
/// generated names assigned by the pinned batch minifier.
pub const EXPORT_INVENTORY: &[ExportedIdentifier] = &[
    ExportedIdentifier {
        source_name: "VERTEX",
        generated_name: "CB",
    },
    ExportedIdentifier {
        source_name: "tessVertexTexture",
        generated_name: "DC",
    },
    ExportedIdentifier {
        source_name: "ENABLE_FEATHER",
        generated_name: "HB",
    },
    ExportedIdentifier {
        source_name: "gaussianIntegralTexture",
        generated_name: "QC",
    },
    ExportedIdentifier {
        source_name: "pathBuffer",
        generated_name: "MB",
    },
    ExportedIdentifier {
        source_name: "paintBuffer",
        generated_name: "TC",
    },
    ExportedIdentifier {
        source_name: "paintAuxBuffer",
        generated_name: "PB",
    },
    ExportedIdentifier {
        source_name: "contourBuffer",
        generated_name: "XC",
    },
    ExportedIdentifier {
        source_name: "FEATHER_ATLAS_BLIT",
        generated_name: "EB",
    },
    ExportedIdentifier {
        source_name: "FRAGMENT",
        generated_name: "FB",
    },
    ExportedIdentifier {
        source_name: "gradTexture",
        generated_name: "DD",
    },
    ExportedIdentifier {
        source_name: "featherAtlasTexture",
        generated_name: "UC",
    },
    ExportedIdentifier {
        source_name: "imageTexture",
        generated_name: "AC",
    },
    ExportedIdentifier {
        source_name: "ENABLE_ADVANCED_BLEND",
        generated_name: "GB",
    },
    ExportedIdentifier {
        source_name: "FIXED_FUNCTION_COLOR_OUTPUT",
        generated_name: "K",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_MSAA",
        generated_name: "BB",
    },
    ExportedIdentifier {
        source_name: "dstColorTexture",
        generated_name: "LD",
    },
    ExportedIdentifier {
        source_name: "DRAW_PATH",
        generated_name: "BD",
    },
    ExportedIdentifier {
        source_name: "DRAW_INTERIOR_TRIANGLES",
        generated_name: "DB",
    },
    ExportedIdentifier {
        source_name: "RENDER_MODE_CLOCKWISE_ATOMIC",
        generated_name: "QB",
    },
];

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "VERTEX",
    "tessVertexTexture",
    "ENABLE_FEATHER",
    "gaussianIntegralTexture",
    "pathBuffer",
    "paintBuffer",
    "paintAuxBuffer",
    "contourBuffer",
    "FEATHER_ATLAS_BLIT",
    "FRAGMENT",
    "gradTexture",
    "featherAtlasTexture",
    "imageTexture",
    "ENABLE_ADVANCED_BLEND",
    "FIXED_FUNCTION_COLOR_OUTPUT",
    "RENDER_MODE_MSAA",
    "dstColorTexture",
    "DRAW_PATH",
    "DRAW_INTERIOR_TRIANGLES",
    "RENDER_MODE_CLOCKWISE_ATOMIC",
];

/// These two source spellings share generated names with differently named
/// global export-header identifiers in the pinned generated shader batch.
pub const EXPORT_MAPPING_AMBIGUITIES: &[(&str, &str, &str)] = &[
    ("gaussianIntegralTexture", "featherTexture", "QC"),
    ("featherAtlasTexture", "atlasTexture", "UC"),
];

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

/// draw_path_common.glsl has no direct #include/#import directive.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// Incoming generated-source include edges retained from the include authority.
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[
    ShaderInclude {
        upstream_file: "renderer/src/metal/background_shader_compiler.mm",
        include_line: 12,
        directive: "include",
        include_token: "generated/shaders/draw_path_common.glsl.hpp",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/draw_path_common.glsl",
        source_unit: "metal-background-shader-compiler",
        dependency_unit: "metal-shader-source-batch",
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
    },
    ShaderInclude {
        upstream_file: "renderer/src/shaders/metal/draw.metal",
        include_line: 20,
        directive: "include",
        include_token: "draw_path_common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/draw_path_common.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        correspondence_owner: "-",
        mapping_status: "-",
        translation_status: "pending",
        translation_disposition: "required-source-edge",
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

pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[
    IncludeDependency {
        including_source: "renderer/src/metal/background_shader_compiler.mm",
        include_line: 12,
        include_token: "generated/shaders/draw_path_common.glsl.hpp",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/draw_path_common.glsl",
        source_unit: "metal-background-shader-compiler",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
    IncludeDependency {
        including_source: "renderer/src/shaders/metal/draw.metal",
        include_line: 20,
        include_token: "draw_path_common.minified.glsl",
        include_syntax: "quote",
        active_branch_path: "all",
        resolution_kind: "generated-shader-source",
        resolved_source: "renderer/src/shaders/draw_path_common.glsl",
        source_unit: "metal-shader-source-batch",
        dependency_unit: "metal-shader-source-batch",
        translation_disposition: "preserve-source-dependency",
    },
];

pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
