// /*
//  * Copyright 2022 Rive
//  */
//
// #pragma once
//
// #include "rive/enums.hpp"
// #include "rive/math/aabb.hpp"
// #include "rive/math/bitwise.hpp"
// #include "rive/math/mat2d.hpp"
// #include "rive/math/vec2d.hpp"
// #include "rive/math/simd.hpp"
// #include "rive/shapes/paint/blend_mode.hpp"
// #include "rive/shapes/paint/color.hpp"
// #include "rive/renderer/trivial_block_allocator.hpp"
// #include "rive/shapes/paint/image_sampler.hpp"
//
// #include <functional>
// #include <optional>
//
// // Use the define to run the feather LUT code
// // #define RIVE_GENERATE_FEATHER_LUT
//
// namespace rive
// {
// class GrInnerFanTriangulator;
// class RenderBuffer;
// } // namespace rive
//
// // This header defines constants and data structures for Rive's pixel local
// // storage path rendering algorithm.
// //
// // Main algorithm:
// // https://docs.google.com/document/d/19Uk9eyFxav6dNSYsI2ZyiX9zHU1YOaJsMB2sdDFVz6s/edit
// //
// // Batching multiple unique paths:
// // https://docs.google.com/document/d/1DLrQimS5pbNaJJ2sAW5oSOsH6_glwDPo73-mtG5_zns/edit
// //
// // Batching strokes as well:
// // https://docs.google.com/document/d/1CRKihkFjbd1bwT08ErMCP4fwSR7D4gnHvgdw_esY9GM/edit
// namespace rive::gpu
// {
// class Draw;
// class Gradient;
// class RenderContextImpl;
// class RenderTarget;
// class Texture;
// enum class DitherMode;
//
// // Global MipMap LOD Bias to apply to samplers. Going lower leads to sharper
// // filtering at the expense of potential shimmering.
// constexpr static float MIP_MAP_LOD_BIAS = -.5f;
//
// // Tessellate in parametric space until each segment is within 1/4 pixel of the
// // true curve.
// constexpr static int kParametricPrecision = 4;
//
// // Tessellate in polar space until the outset edge is within 1/8 pixel of the
// // true stroke.
// constexpr static int kPolarPrecision = 8;
//
// // Maximum supported numbers of tessellated segments in a single curve.
// constexpr static uint32_t kMaxParametricSegments = 1023;
// constexpr static uint32_t kMaxPolarSegments = 1023;
//
// // The Gaussian distribution is very blurry on the outer edges. Regardless of
// // how wide a feather is, the polar segments never need to have a finer angle
// // than this value.
// constexpr static float FEATHER_POLAR_SEGMENT_MIN_ANGLE = math::PI / 16;
//
// // cos(FEATHER_MIN_POLAR_SEGMENT_ANGLE / 2)
// constexpr static float COS_FEATHER_POLAR_SEGMENT_MIN_ANGLE_OVER_2 =
//     0.99518472667f;
//
// // We allocate all our GPU buffers in rings. This ensures the CPU can prepare
// // frames in parallel while the GPU renders them.
// constexpr static int kBufferRingSize = 3;
//
// // Every coverage value in pixel local storage has an associated 16-bit path ID.
// // This ID enables us to batch multiple paths together without having to clear
// // the coverage buffer in between. This ID is implemented as an fp16, so the
// // maximum path ID therefore cannot be NaN (or conservatively, all 5 exponent
// // bits cannot be 1's). We also skip denormalized values (exp == 0) because they
// // have been empirically unreliable on Android as ID values.
// constexpr static int kLargestFP16BeforeExponentAll1s = (0x1f << 10) - 1;
// constexpr static int kLargestDenormalizedFP16 = 1023;
// constexpr static int MaxPathID(int granularity)
// {
//     // Floating point equality gets funky when the exponent bits are all 1's, so
//     // the largest pathID we can support is kLargestFP16BeforeExponentAll1s.
//     //
//     // The shader converts an integer path ID to fp16 as:
//     //
//     //     (id + kLargestDenormalizedFP16) * granularity
//     //
//     // So the largest path ID we can support is as follows.
//     return kLargestFP16BeforeExponentAll1s / granularity -
//            kLargestDenormalizedFP16;
// }
//
// // Each contour has its own unique ID, which it uses to index a data record
// // containing per-contour information. This value is currently 16 bit.
// constexpr static size_t kMaxContourID = 65535;
// constexpr static uint32_t kContourIDMask = 0xffff;
// static_assert((kMaxContourID & kContourIDMask) == kMaxContourID);
//
// // Tessellation is performed by rendering vertices into a data texture. These
// // values define the dimensions of the tessellation data texture.
// constexpr static size_t kTessTextureWidth =
//     2048; // GL_MAX_TEXTURE_SIZE spec minimum on ES3/WebGL2.
// constexpr static size_t kTessTextureWidthLog2 = 11;
// static_assert(1 << kTessTextureWidthLog2 == kTessTextureWidth);
//
// // Gradients are implemented by sampling a horizontal ramp of pixels allocated
// // in a global gradient texture.
// constexpr static uint32_t kGradTextureWidth = 512;
// constexpr static uint32_t kGradTextureWidthInSimpleRamps =
//     kGradTextureWidth / 2;
//
// // Depth/stencil parameters
// constexpr static float DEPTH_MIN = 0.0f;
// constexpr static float DEPTH_MAX = 1.0f;
// constexpr static uint8_t STENCIL_CLEAR = 0u;
//
// // Backend-specific capabilities/workarounds and fine tuning.
// struct PlatformFeatures
// {
//     // Supported InterlockModes.
//     // FIXME: MSAA is implicit even though it isn't implemented on all backends.
//     bool supportsRasterOrderingMode = false;
//     bool supportsAtomicMode = false;
//     bool supportsClockwiseMode = false;
//     // InterlockMode::Clockwise with fixedFunctionColorOutput and srcOver blend.
//     // (Only viable for frames that don't use advanced blend.)
//     bool supportsClockwiseFixedFunctionMode = false;
//     bool supportsClockwiseAtomicMode = false;
//     // Use KHR_blend_equation_advanced in msaa mode?
//     bool supportsBlendAdvancedKHR = false;
//     bool supportsBlendAdvancedCoherentKHR = false;
//     // Required for @ENABLE_CLIP_RECT in msaa mode.
//     bool supportsClipPlanes = false;
//     // The backend supports dynamic state that allows Rive to collapse multiple
//     // subpasses onto a single pipeline, namely:
//     //  * Depth (Vulkan 1.3)
//     //  * Stencil (Vulkan 1.3)
//     //  * Cull (Vulkan 1.3)
//     //  * Color-write (VK_EXT_color_write_enable)
//     // With this state being dynamic, we can combine multiple subpasses (e.g.,
//     // borrowed coverage, fans, stencil reset) onto a single dynamic pipeline
//     // with multiple draws and state updates in between.
//     bool supportsPipelineDynamicState = false;
//     bool avoidFlatVaryings = false;
//     // Vivo Y21 (PowerVR Rogue GE8320; OpenGL ES 3.2 build 1.13@5776728a) seems
//     // to hit some sort of reset condition that corrupts pixel local storage
//     // when rendering a complex feather. Provide a workaround that allows the
//     // implementation to opt in to always feathering to the feather atlas
//     // instead of rendering directly to the screen.
//     bool alwaysFeatherToAtlas = false;
//     // clipSpaceBottomUp specifies whether the top of the viewport, in clip
//     // coordinates, is at Y=+1 (OpenGL, Metal, D3D, WebGPU) or Y=-1 (Vulkan).
//     //
//     // framebufferBottomUp specifies whether "row 0" of the framebuffer is the
//     // bottom of the image (OpenGL) or the top (Metal, D3D, WebGPU, Vulkan).
//     //
//     //
//     //                                OpenGL
//     //           (clipSpaceBottomUp=true, framebufferBottomUp=true)
//     //
//     //  Rive Pixel Space             Clip Space              Framebuffer
//     //
//     //  0 ----------->                   ^ +1                ^ height
//     //  |          width                 |                   |
//     //  |                         -1     |     +1            |
//     //  |                 ===>    <------|------>    ===>    |
//     //  |                                |                   |
//     //  |                                |                   |          width
//     //  v height                         v -1                0 ----------->
//     //
//     //
//     //
//     //                            Metal/D3D/WebGPU
//     //           (clipSpaceBottomUp=true, framebufferBottomUp=false)
//     //
//     //  Rive Pixel Space             Clip Space              Framebuffer
//     //
//     //  0 ----------->                   ^ +1                0 ----------->
//     //  |          width                 |                   |          width
//     //  |                         -1     |     +1            |
//     //  |                 ===>    <------|------>    ===>    |
//     //  |                                |                   |
//     //  |                                |                   |
//     //  v height                         v -1                v height
//     //
//     //
//     //
//     //                                Vulkan
//     //          (clipSpaceBottomUp=false, framebufferBottomUp=false)
//     //
//     //  Rive Pixel Space             Clip Space              Framebuffer
//     //
//     //  0 ----------->                   ^ -1                0 ----------->
//     //  |          width                 |                   |          width
//     //  |                         -1     |     +1            |
//     //  |                 ===>    <------|------>    ===>    |
//     //  |                                |                   |
//     //  |                                |                   |
//     //  v height                         v +1                v height
//     //
//     bool clipSpaceBottomUp = false;
//     bool framebufferBottomUp = false;
//     // Backend cannot initialize PLS with typical clear/load APIs in atomic
//     // mode. Issue a "DrawType::renderPassInitialize" draw instead.
//     bool atomicPLSInitNeedsDraw = false;
//     // Backend API does not support initializing our (transient) MSAA color
//     // buffer with the existing (non-MSAA) target texture at the beginning of a
//     // render pass. Draw the previous renderTarget contents into it manually via
//     // DrawType::renderPassInitialize when LoadAction::preserveRenderTarget is
//     // specified.
//     bool msaaColorPreserveNeedsDraw = false;
//     // Workaround for Qualcomm. Framebuffer reads on Qualcomm seem to not work
//     // in clockwiseAtomic mode unless we issue a simple, 1-pixel draw that reads
//     // the framebuffer between borrowed coverage and the main draws.
//     bool clockwiseAtomicBorrowedCoverageBarrierNeedsRenderPassInit = false;
//     // Workaround for precision issues. Determines how far apart we space unique
//     // path IDs when they will be bit-casted to fp16.
//     uint8_t pathIDGranularity = 1;
//     // Maximum size (width or height) of a texture.
//     uint32_t maxTextureSize = 2048;
//     // Maximum length (in 32-bit uints) of the coverage buffer used for paths in
//     // clockwiseFill/atomic mode. 2^27 bytes is the minimum storage buffer size
//     // requirement in the Vulkan, GL, and D3D11 specs. Metal guarantees 256 MB.
//     size_t maxCoverageBufferLength = (1 << 27) / sizeof(uint32_t);
//
//     // True when the backend supports using the scissor rectangle for reducing
//     // the draw bounds of clip reads and writes.
//     // TODO: This should be possible to implement across all backends - at which
//     // point this bool could go away.
//     bool supportsClipScissor = false;
//
//     // GPU compressed texture format support (queried per backend at init).
//     bool supportsTextureCompressionBC = false;   // BC1/BC2/BC3/BC7
//     bool supportsTextureCompressionASTC = false; // ASTC LDR (any block size)
//     bool supportsTextureCompressionETC2 = false; // ETC2 RGB8 / RGBA8
// };
//
// // Gradient color stops are implemented as a horizontal span of pixels in a
// // global gradient texture. They are rendered by "GradientSpan" instances.
// struct GradientSpan
// {
//     // x0Fixed and x1Fixed are normalized texel x coordinates, in the
//     // fixed-point range 0..65535.
//     RIVE_ALWAYS_INLINE void set(uint32_t x0Fixed,
//                                 uint32_t x1Fixed,
//                                 uint32_t y,
//                                 uint32_t flags,
//                                 ColorInt color0_,
//                                 ColorInt color1_)
//     {
//         assert(x0Fixed < 65536);
//         assert(x1Fixed < 65536);
//         horizontalSpan = (x1Fixed << 16) | x0Fixed;
//         yWithFlags = flags | y;
//         color0 = color0_;
//         color1 = color1_;
//     }
//     uint32_t horizontalSpan;
//     uint32_t yWithFlags;
//     uint32_t color0;
//     uint32_t color1;
// };
// static_assert(sizeof(GradientSpan) == sizeof(uint32_t) * 4);
// static_assert(256 % sizeof(GradientSpan) == 0);
// // Metal requires vertex buffers to be 256-byte aligned.
// constexpr static size_t kGradSpanBufferAlignmentInElements =
//     256 / sizeof(GradientSpan);
//
// // Gradient spans are drawn as 1px-tall triangle strips with 3 sub-rectangles.
// constexpr uint32_t GRAD_SPAN_TRI_STRIP_VERTEX_COUNT = 8;
//
// // Each curve gets tessellated into vertices. This is performed by rendering a
// // horizontal span of positions and normals into the tessellation data texture,
// // GP-GPU style. TessVertexSpan defines one instance of a horizontal
// // tessellation span for rendering.
// //
// // Each span has an optional reflection, rendered right to left, with the same
// // vertices in reverse order. These are used to draw mirrored patches with
// // negative coverage when we have back-face culling enabled. This emits every
// // triangle twice, once clockwise and once counterclockwise, and back-face
// // culling naturally selects the triangle with the appropriately signed coverage
// // (discarding the other).
// struct TessVertexSpan
// {
//     RIVE_ALWAYS_INLINE void set(const Vec2D pts_[4],
//                                 Vec2D joinTangent_,
//                                 float y_,
//                                 int32_t x0,
//                                 int32_t x1,
//                                 uint32_t parametricSegmentCount,
//                                 uint32_t polarSegmentCount,
//                                 uint32_t joinSegmentCount,
//                                 uint32_t contourIDWithFlags_)
//     {
//         set(pts_,
//             joinTangent_,
//             y_,
//             x0,
//             x1,
//             std::numeric_limits<float>::quiet_NaN(), // Discard the reflection.
//             -1,
//             -1,
//             parametricSegmentCount,
//             polarSegmentCount,
//             joinSegmentCount,
//             contourIDWithFlags_);
//     }
//
//     RIVE_ALWAYS_INLINE void set(const Vec2D pts_[4],
//                                 Vec2D joinTangent_,
//                                 float y_,
//                                 int32_t x0,
//                                 int32_t x1,
//                                 float reflectionY_,
//                                 int32_t reflectionX0,
//                                 int32_t reflectionX1,
//                                 uint32_t parametricSegmentCount,
//                                 uint32_t polarSegmentCount,
//                                 uint32_t joinSegmentCount,
//                                 uint32_t contourIDWithFlags_)
//     {
// #ifndef NDEBUG
//         // Write to an intermediate local object in debug mode, so we can check
//         // its values. (Otherwise we can't read it because mapped memory is
//         // write-only.)
//         TessVertexSpan localCopy;
// #define LOCAL(VAR) localCopy.VAR
// #else
// #define LOCAL(VAR) VAR
// #endif
//         RIVE_INLINE_MEMCPY(LOCAL(pts), pts_, sizeof(LOCAL(pts)));
//         LOCAL(joinTangent) = joinTangent_;
//         LOCAL(y) = y_;
//         LOCAL(reflectionY) = reflectionY_;
//         LOCAL(x0x1) = (x1 << 16 | (x0 & 0xffff));
//         LOCAL(reflectionX0X1) = (reflectionX1 << 16 | (reflectionX0 & 0xffff));
//         LOCAL(segmentCounts) = (joinSegmentCount << 20) |
//                                (polarSegmentCount << 10) |
//                                parametricSegmentCount;
//         LOCAL(contourIDWithFlags) = contourIDWithFlags_;
// #undef LOCAL
//
//         // Ensure we didn't lose any data from packing.
//         assert(localCopy.x0x1 << 16 >> 16 == x0);
//         assert(localCopy.x0x1 >> 16 == x1);
//         assert(localCopy.reflectionX0X1 << 16 >> 16 == reflectionX0);
//         assert(localCopy.reflectionX0X1 >> 16 == reflectionX1);
//         assert((localCopy.segmentCounts & 0x3ff) == parametricSegmentCount);
//         assert(((localCopy.segmentCounts >> 10) & 0x3ff) == polarSegmentCount);
//         assert(localCopy.segmentCounts >> 20 == joinSegmentCount);
//
// #ifndef NDEBUG
//         memcpy(this, &localCopy, sizeof(*this));
// #endif
//     }
//
//     Vec2D pts[4];      // Cubic bezier curve.
//     Vec2D joinTangent; // Ending tangent of the join that follows the cubic.
//     float y;
//     float reflectionY;
//     int32_t x0x1;
//     int32_t reflectionX0X1;
//     uint32_t segmentCounts;      // [joinSegmentCount, polarSegmentCount,
//                                  // parametricSegmentCount]
//     uint32_t contourIDWithFlags; // flags | contourID
// };
// static_assert(sizeof(TessVertexSpan) == sizeof(float) * 16);
// static_assert(256 % sizeof(TessVertexSpan) == 0);
// // Metal requires vertex buffers to be 256-byte aligned.
// constexpr static size_t kTessVertexBufferAlignmentInElements =
//     256 / sizeof(TessVertexSpan);
//
// // Tessellation spans are drawn as two distinct, 1px-tall rectangles: the span
// // and its reflection.
// constexpr uint16_t kTessSpanIndices[4 * 3] =
//     {0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7};
//
// // ImageRects are a special type of non-overlapping antialiased draw that we
// // only have to use in atomic mode. They allow us to bind a texture and draw it
// // in its entirety in a single pass.
// struct ImageRectVertex
// {
//     float x;
//     float y;
//     float aaOffsetX;
//     float aaOffsetY;
// };
//
// constexpr ImageRectVertex kImageRectVertices[12] = {
//     {0, 0, .0, -1},
//     {1, 0, .0, -1},
//     {1, 0, +1, .0},
//     {1, 1, +1, .0},
//     {1, 1, .0, +1},
//     {0, 1, .0, +1},
//     {0, 1, -1, .0},
//     {0, 0, -1, .0},
//     {0, 0, +1, +1},
//     {1, 0, -1, +1},
//     {1, 1, -1, -1},
//     {0, 1, +1, -1},
// };
//
// constexpr uint16_t kImageRectIndices[14 * 3] = {
//     8,  0, 9, 9, 0, 1,  1,  2, 9, 9, 2, 10, 10, 2, 3, 3, 4,  10, 10, 4, 11,
//     11, 4, 5, 5, 6, 11, 11, 6, 8, 8, 6, 7,  7,  0, 8, 9, 10, 8,  10, 8, 11,
// };
//
// enum class PaintType : uint32_t
// {
//     clipUpdate, // Update the clip buffer instead of drawing to the framebuffer.
//     solidColor,
//     linearGradient,
//     radialGradient,
//     image,
// };
//
// // Specifies the location of a simple or complex horizontal color ramp within
// // the gradient texture. A simple color ramp is two texels wide, beginning at
// // the specified row and column. A complex color ramp spans the entire width of
// // the gradient texture, on the row:
// //     "GradTextureLayout::complexOffsetY + ColorRampLocation::row".
// struct ColorRampLocation
// {
//     constexpr static uint16_t kComplexGradientMarker = 0xffff;
//     bool isComplex() const { return col == kComplexGradientMarker; }
//     uint16_t row;
//     uint16_t col;
// };
//
// // Most of a paint's information can be described in a single value. Gradients
// // and images reference an additional Gradient* and Texture* respectively.
// union SimplePaintValue
// {
//     ColorInt color = 0xff000000;         // PaintType::solidColor
//     ColorRampLocation colorRampLocation; // Paintype::linear/radialGradient
//     float imageOpacity;                  // PaintType::image
//     uint32_t outerClipID;                // Paintype::clipUpdate
// };
// static_assert(sizeof(SimplePaintValue) == 4);
//
// // This class encapsulates a matrix that maps from _fragCoord to a space where
// // the clipRect is the normalized rectangle: [-1, -1, +1, +1]
// class ClipRectInverseMatrix
// {
// public:
//     // When the clipRect inverse matrix is singular (e.g., all 0 in scale and
//     // skew), the shader uses tx and ty as fixed clip coverage values instead of
//     // finding edge distances.
//     constexpr static ClipRectInverseMatrix WideOpen()
//     {
//         return Mat2D{0, 0, 0, 0, 1, 1};
//     }
//     constexpr static ClipRectInverseMatrix Empty()
//     {
//         return Mat2D{0, 0, 0, 0, 0, 0};
//     }
//
//     ClipRectInverseMatrix() = default;
//
//     ClipRectInverseMatrix(const Mat2D& clipMatrix, const AABB& clipRect)
//     {
//         reset(clipMatrix, clipRect);
//     }
//
//     void reset(const Mat2D& clipMatrix, const AABB& clipRect);
//
//     const Mat2D& inverseMatrix() const { return m_inverseMatrix; }
//
// private:
//     constexpr ClipRectInverseMatrix(const Mat2D& inverseMatrix) :
//         m_inverseMatrix(inverseMatrix)
//     {}
//     Mat2D m_inverseMatrix;
// };
//
// // Specifies the height of the gradient texture, and the row at which we
// // transition from simple color ramps to complex.
// //
// // This information is computed at flush time, once we know exactly how many
// // color ramps of each type will be in the gradient texture.
// struct GradTextureLayout
// {
//     uint32_t complexOffsetY; // Row of the first complex gradient.
//     float inverseHeight;     // 1 / textureHeight
// };
//
// // Once all curves in a contour have been tessellated, we render the tessellated
// // vertices in "patches" (aka specific instanced geometry).
// //
// // See:
// // https://docs.google.com/document/d/19Uk9eyFxav6dNSYsI2ZyiX9zHU1YOaJsMB2sdDFVz6s/edit#heading=h.fa4kubk3vimk
// //
// // With strokes:
// // https://docs.google.com/document/d/1CRKihkFjbd1bwT08ErMCP4fwSR7D4gnHvgdw_esY9GM/edit#heading=h.dcd0c58pxfs5
// //
// // A single patch spans N tessellation segments, connecting N + 1 tessellation
// // vertices. It is composed of a an AA border and fan triangles. The specifics
// // of the fan triangles depend on the PatchType.
// enum class PatchType
// {
//     // Patches fan around the contour midpoint. Outer edges are inset by ~1px,
//     // followed by a ~1px AA ramp.
//     midpointFan,
//
//     // Similar to midpointFan, except AA ramps are split down the center and
//     // drawn with a ~1/2px outset AA ramp and a ~1/2px inset AA ramp that
//     // overlaps the inner tessellation and has negative coverage.
//     midpointFanCenterAA,
//
//     // Patches only cover the AA ramps and interiors of bezier curves. The
//     // interior path triangles that connect the outer curves are triangulated on
//     // the CPU to eliminate overlap, and are drawn in a separate call. AA ramps
//     // are split down the center (on the same lines as the interior
//     // triangulation), and drawn with a ~1/2px outset AA ramp and a ~1/2px inset
//     // AA ramp that overlaps the inner tessellation and has negative coverage. A
//     // lone bowtie join is emitted at the end of the patch to tie the outer
//     // curves together.
//     outerCurves,
// };
//
// // When tessellating path vertices, we have the ability to generate the
// // triangles wound in forward or reverse order. Depending on the path and the
// // rendering algorithm, we will either want the triangles wound forward,
// // reverse, or BOTH.
// enum class ContourDirections
// {
//     forward,
//     reverse,
//     // Generate two tessellations of the contour: reverse first, then forward.
//     reverseThenForward,
//     // Generate two tessellations of the contour: forward first, then reverse.
//     forwardThenReverse,
// };
// constexpr static bool ContourDirectionsAreDoubleSided(
//     ContourDirections contourDirections)
// {
//     return contourDirections >= ContourDirections::reverseThenForward;
// }
//
// struct PatchVertex
// {
//     void set(float localVertexID_,
//              float outset_,
//              float fillCoverage_,
//              float params_)
//     {
//         localVertexID = localVertexID_;
//         outset = outset_;
//         fillCoverage = fillCoverage_;
//         params = params_;
//         setMirroredPosition(localVertexID_, outset_, fillCoverage_);
//     }
//
//     // Patch vertices can have an optional, alternate position when mirrored.
//     // This is so we can ensure the diagonals inside the stroke line up on both
//     // versions of the patch (mirrored and not).
//     void setMirroredPosition(float localVertexID_,
//                              float outset_,
//                              float fillCoverage_)
//     {
//         mirroredVertexID = localVertexID_;
//         mirroredOutset = outset_;
//         mirroredFillCoverage = fillCoverage_;
//     }
//
//     float localVertexID; // 0 or 1 -- which tessellated vertex of the two that
//                          // we are connecting?
//     float outset; // Outset from the tessellated position, in the direction of
//                   // the normal.
//     float fillCoverage; // 0..1 for the stroke. 1 all around for the triangles.
//                         // (Coverage will be negated later for counterclockwise
//                         // triangles.)
//     int32_t params;     // "(patchSize << 2) | [flags::kStrokeVertex,
//                         //                      flags::kFanVertex,
//                         //                      flags::kFanMidpointVertex]"
//     float mirroredVertexID;
//     float mirroredOutset;
//     float mirroredFillCoverage;
//     int32_t padding = 0;
// };
// static_assert(sizeof(PatchVertex) == sizeof(float) * 8);
//
// // # of tessellation segments spanned by the midpoint fan patch.
// constexpr static uint32_t kMidpointFanPatchSegmentSpan = 8;
//
// // # of tessellation segments spanned by the outer curve patch. (In this
// // particular instance, the final segment is a bowtie join with zero length and
// // no fan triangle.)
// constexpr static uint32_t kOuterCurvePatchSegmentSpan = 17;
//
// // Define vertex and index buffers that contain all the triangles in every
// // PatchType.
// constexpr static uint32_t kMidpointFanPatchVertexCount =
//     kMidpointFanPatchSegmentSpan * 4 /*Stroke and/or AA outer ramp*/ +
//     (kMidpointFanPatchSegmentSpan + 1) /*Curve fan*/ +
//     1 /*Triangle from path midpoint*/;
// constexpr static uint32_t kMidpointFanPatchBorderIndexCount =
//     kMidpointFanPatchSegmentSpan * 6 /*Stroke and/or AA outer ramp*/;
// constexpr static uint32_t kMidpointFanPatchIndexCount =
//     kMidpointFanPatchBorderIndexCount /*Stroke and/or AA outer ramp*/ +
//     (kMidpointFanPatchSegmentSpan - 1) * 3 /*Curve fan*/ +
//     3 /*Triangle from path midpoint*/;
// constexpr static uint32_t kMidpointFanPatchBaseIndex = 0;
// static_assert((kMidpointFanPatchBaseIndex * sizeof(uint16_t)) % 4 == 0);
//
// constexpr static uint32_t kMidpointFanCenterAAPatchVertexCount =
//     kMidpointFanPatchSegmentSpan * 4 * 2 /*Stroke and/or AA outer ramp*/ +
//     (kMidpointFanPatchSegmentSpan + 1) /*Curve fan*/ +
//     1 /*Triangle from path midpoint*/;
// constexpr static uint32_t kMidpointFanCenterAAPatchBorderIndexCount =
//     kMidpointFanPatchSegmentSpan * 12 /*Stroke and/or AA outer ramp*/;
// constexpr static uint32_t kMidpointFanCenterAAPatchIndexCount =
//     kMidpointFanCenterAAPatchBorderIndexCount /*Stroke and/or AA outer ramp*/ +
//     (kMidpointFanPatchSegmentSpan - 1) * 3 /*Curve fan*/ +
//     3 /*Triangle from path midpoint*/;
// constexpr static uint32_t kMidpointFanCenterAAPatchBaseIndex =
//     kMidpointFanPatchBaseIndex + kMidpointFanPatchIndexCount;
// static_assert((kMidpointFanCenterAAPatchBaseIndex * sizeof(uint16_t)) % 4 == 0);
//
// constexpr static uint32_t kOuterCurvePatchVertexCount =
//     kOuterCurvePatchSegmentSpan * 8 /*AA center ramp with bowtie*/ +
//     kOuterCurvePatchSegmentSpan /*Curve fan*/;
// constexpr static uint32_t kOuterCurvePatchBorderIndexCount =
//     kOuterCurvePatchSegmentSpan * 12 /*AA center ramp with bowtie*/;
// constexpr static uint32_t kOuterCurvePatchIndexCount =
//     kOuterCurvePatchBorderIndexCount /*AA center ramp with bowtie*/ +
//     (kOuterCurvePatchSegmentSpan - 2) * 3 /*Curve fan*/;
// constexpr static uint32_t kOuterCurvePatchBaseIndex =
//     kMidpointFanCenterAAPatchBaseIndex + kMidpointFanCenterAAPatchIndexCount;
// static_assert((kOuterCurvePatchBaseIndex * sizeof(uint16_t)) % 4 == 0);
//
// constexpr static uint32_t kPatchVertexBufferCount =
//     kMidpointFanPatchVertexCount + kMidpointFanCenterAAPatchVertexCount +
//     kOuterCurvePatchVertexCount;
// constexpr static uint32_t kPatchIndexBufferCount =
//     kMidpointFanPatchIndexCount + kMidpointFanCenterAAPatchIndexCount +
//     kOuterCurvePatchIndexCount;
// void GeneratePatchBufferData(PatchVertex[kPatchVertexBufferCount],
//                              uint16_t indices[kPatchIndexBufferCount]);
//
// enum class DrawType : uint8_t
// {
//     // Fills, strokes, feathered strokes.
//     midpointFanPatches,
//
//     // Feathered fills.
//     midpointFanCenterAAPatches,
//
//     // Just the outer curves of a path; the interior will be triangulated.
//     outerCurvePatches,
//
//     interiorTriangulation,
//     featherAtlasBlit,
//     imageRect,
//     imageMesh,
//
//     // MSAA strokes can't be merged with fills because they require their own
//     // dedicated stencil settings.
//     msaaStrokes,
//
//     // MSAA "fast" path: (effectively) single pass rendering.
//     msaaMidpointFanBorrowedCoverage,
//     msaaMidpointFans,
//     msaaMidpointFanStencilReset,
//
//     // Equivalent to msaaMidpointFanBorrowedCoverage + msaaMidpointFans +
//     // msaaMidpointFanStencilReset on a single pipeline, switching between them
//     // with dynamic color/depth/stencil/cull state. Keeps the three passes on
//     // one batch so the reorderer can still instance non-overlapping paths
//     // together, while collapsing three pipeline binds into one.
//     msaaDynamicMidpointFans,
//
//     // MSAA "slow" path: stencil-then-cover.
//     msaaMidpointFanPathsStencil,
//     msaaMidpointFanPathsCover,
//
//     // MSAA interior triangulation is not currently supported, but this one draw
//     // type is included in order to support the "retrofittedcubictriangles" GM.
//     msaaOuterCubics,
//
//     // Clear or intersect (based on DrawContents) the clip value.
//     clipReset,
//
//     // Clear/init render pass data with a fullscreen draw when we can't do it
//     // with existing clear/load APIs. (e.g., for pixel local storage in buffers
//     // that don't have copy/clear commands, or preserving existing color data in
//     // a transient MSAA arrachment).
//     renderPassInitialize,
//
//     // Resolve render pass data (e.g., by applying the final deferred color in
//     // atomic mode, or copying an offscreen attachment to the final
//     // renderTarget).
//     renderPassResolve,
//
// };
//
// constexpr static bool DrawTypeIsImageDraw(DrawType drawType)
// {
//     switch (drawType)
//     {
//         case DrawType::imageRect:
//         case DrawType::imageMesh:
//             return true;
//         case DrawType::midpointFanPatches:
//         case DrawType::midpointFanCenterAAPatches:
//         case DrawType::outerCurvePatches:
//         case DrawType::interiorTriangulation:
//         case DrawType::featherAtlasBlit:
//         case DrawType::msaaStrokes:
//         case DrawType::msaaMidpointFanBorrowedCoverage:
//         case DrawType::msaaDynamicMidpointFans:
//         case DrawType::msaaMidpointFans:
//         case DrawType::msaaMidpointFanStencilReset:
//         case DrawType::msaaMidpointFanPathsStencil:
//         case DrawType::msaaMidpointFanPathsCover:
//         case DrawType::msaaOuterCubics:
//         case DrawType::clipReset:
//         case DrawType::renderPassInitialize:
//         case DrawType::renderPassResolve:
//             return false;
//     }
//     RIVE_UNREACHABLE();
// }
//
// // Specifies what to do with the render target at the beginning of a flush.
// enum class LoadAction
// {
//     clear,
//     preserveRenderTarget,
//     dontCare,
// };
//
// // Synchronization method for pixel local storage with overlapping fragments.
// enum class InterlockMode
// {
//     rasterOrdering,
//     atomics,
//     // Overrides every path's fill rule with clockwise, and implements the
//     // clockwise algorithm using raster ordering hardware.
//     // TODO: Once polished, this mode can be mixed into "rasterOrdering" and
//     // used selectively for clockwise paths.
//     clockwise,
//     // Use an experimental path rendering algorithm that utilizes atomics
//     // without barriers. This requires that we override all paths' fill rules
//     // (winding or even/odd) with a "clockwise" fill rule, where only regions
//     // with a positive winding number get filled.
//     clockwiseAtomic,
//     msaa,
// };
// constexpr static size_t INTERLOCK_MODE_COUNT = 5;
// // # of bits required to contain an InterlockMode.
// constexpr static size_t INTERLOCK_MODE_BIT_COUNT = 3;
// static_assert(INTERLOCK_MODE_COUNT <= (1 << INTERLOCK_MODE_BIT_COUNT));
// static_assert(INTERLOCK_MODE_COUNT > (1 << (INTERLOCK_MODE_BIT_COUNT - 1)));
//
// // Low-level batch of scissored geometry for rendering to the offscreen atlas.
// struct AtlasDrawBatch
// {
//     AABBu16 scissor;
//     uint32_t patchCount;
//     uint32_t basePatch;
// };
//
// // "Uber shader" features that can be #defined in a draw shader.
// // This set is strictly limited to switches that don't *change* the behavior of
// // the shader, i.e., turning them all on will enable all types Rive content, but
// // simple content will still draw identically; we can turn a feature off if we
// // know a batch doesn't need it for better performance.
// enum class ShaderFeatures
// {
//     NONE = 0,
//
//     // Whole program features.
//     ENABLE_CLIPPING = 1 << 0,
//     ENABLE_CLIP_RECT = 1 << 1,
//     ENABLE_ADVANCED_BLEND = 1 << 2,
//     ENABLE_FEATHER = 1 << 3,
//
//     // Fragment-only features.
//     ENABLE_EVEN_ODD = 1 << 4,
//     ENABLE_NESTED_CLIPPING = 1 << 5,
//     ENABLE_HSL_BLEND_MODES = 1 << 6,
//     ENABLE_DITHER = 1 << 7,
// };
//
// constexpr static size_t kShaderFeatureCount = 8;
// constexpr static ShaderFeatures kAllShaderFeatures =
//     static_cast<gpu::ShaderFeatures>((1 << kShaderFeatureCount) - 1);
// constexpr static ShaderFeatures kVertexShaderFeaturesMask =
//     ShaderFeatures::ENABLE_CLIPPING | ShaderFeatures::ENABLE_CLIP_RECT |
//     ShaderFeatures::ENABLE_ADVANCED_BLEND | ShaderFeatures::ENABLE_FEATHER;
//
// // These shader features change the way atomic pipelines are set up (or cause
// //  validation failures when enabled but not used)
// constexpr static ShaderFeatures kExclusiveAtomicUbershaderFeaturesMask =
//     ShaderFeatures::ENABLE_ADVANCED_BLEND;
//
// constexpr static ShaderFeatures ShaderFeaturesMaskFor(
//     InterlockMode interlockMode)
// {
//     switch (interlockMode)
//     {
//         case InterlockMode::rasterOrdering:
//             return kAllShaderFeatures;
//         case InterlockMode::atomics:
//             return kAllShaderFeatures & ~ShaderFeatures::ENABLE_NESTED_CLIPPING;
//         case InterlockMode::clockwise:
//             return kAllShaderFeatures & ~ShaderFeatures::ENABLE_EVEN_ODD;
//         case InterlockMode::clockwiseAtomic:
//             return kAllShaderFeatures &
//                    // clockwiseAtomic never supports even/odd fill rule.
//                    ~ShaderFeatures::ENABLE_EVEN_ODD &
//                    // clockwiseAtomic requires special blend state for nested
//                    // clip updates, so they need their own draw anyway and the
//                    // ENABLE_NESTED_CLIPPING feature isn't necessary.
//                    ~ShaderFeatures::ENABLE_NESTED_CLIPPING;
//         case InterlockMode::msaa:
//             return ShaderFeatures::ENABLE_CLIP_RECT |
//                    ShaderFeatures::ENABLE_ADVANCED_BLEND |
//                    ShaderFeatures::ENABLE_HSL_BLEND_MODES |
//                    ShaderFeatures::ENABLE_DITHER;
//     }
//     RIVE_UNREACHABLE();
// }
//
// // Miscellaneous switches that *do* affect the behavior of the fragment shader.
// // The renderContext may add some of these, and a backend may also add them to a
// // shader key if it wants to implement the behavior.
// enum class ShaderMiscFlags : uint32_t
// {
//     none = 0,
//
//     // InterlockMode::atomics only (without advanced blend). Render color to a
//     // standard attachment instead of PLS. The backend implementation is
//     // responsible to turn on src-over blending.
//     fixedFunctionColorOutput = 1 << 0,
//
//     // Override all paths' fill rules (winding or even/odd) with an experimental
//     // "clockwise" fill rule, where only regions with a positive winding number
//     // get filled.
//     clockwiseFill = 1 << 1,
//
//     // clockwise and clockwiseAtomic only: This is a specialized shader that
//     // only renders to the clip buffer. It doesn't output color.
//     clipUpdateOnly = 1 << 2,
//
//     // clockwiseAtomic only: This is a specialized shader that only subtracts
//     // coverage from the existing clip contents (i.e., nested clip updates).
//     // It doesn't output color.
//     nestedClipUpdateOnly = 1 << 3,
//
//     // clockwise and clockwiseAtomic modes only. This shader renders a pass that
//     // only subtracts (counterclockwise) borrowed coverage from the coverage
//     // buffer. It doesn't output color or clip.
//     // If drawing interior triangulations, every fragment will be the first of
//     // the path at its pixel, so it can blindly overwrite coverage without
//     // reading the buffer and subtracting.
//     borrowedCoveragePass = 1 << 4,
//
//     // DrawType::renderPassInitialize only. Also store the color clear value to
//     // PLS when drawing a clear, in addition to clearing the other PLS planes.
//     storeColorClear = 1 << 5,
//
//     // DrawType::renderPassInitialize only. Seed the color PLS plane by
//     // sampling the framebuffer contents (previously copied into a dst color
//     // texture bound at IMAGE_TEXTURE_IDX). Used for
//     // LoadAction::preserveRenderTarget on backends that can't directly copy
//     // a texture into a storage buffer (e.g. WebGPU).
//     loadColorFromDstTexture = 1 << 6,
//
//     // DrawType::renderPassInitialize only. Swizzle the existing framebuffer
//     // contents from BGRA to RGBA. (For when this data had to get copied from a
//     // BGRA target.)
//     swizzleColorBGRAToRGBA = 1 << 7,
//
//     // DrawType::renderPassResolve only. Optimization for when rendering to an
//     // offscreen texture.
//     //
//     // It renders the final "resolve" operation directly to the renderTarget in
//     // a single pass, instead of (1) resolving the offscreen texture, and then
//     // (2) copying the offscreen texture to back the renderTarget.
//     coalescedResolveAndTransfer = 1 << 8,
// };
//
// constexpr static ShaderFeatures ShaderFeaturesMaskFor(
//     DrawType drawType,
//     InterlockMode interlockMode)
// {
//     ShaderFeatures mask = ShaderFeatures::NONE;
//     switch (drawType)
//     {
//         case DrawType::imageRect:
//         case DrawType::imageMesh:
//         case DrawType::featherAtlasBlit:
//             if (interlockMode != InterlockMode::atomics)
//             {
//                 mask = ShaderFeatures::ENABLE_CLIPPING |
//                        ShaderFeatures::ENABLE_CLIP_RECT |
//                        ShaderFeatures::ENABLE_ADVANCED_BLEND |
//                        ShaderFeatures::ENABLE_HSL_BLEND_MODES |
//                        ShaderFeatures::ENABLE_DITHER;
//                 break;
//             }
//             // Since atomic mode has to resolve previous draws, images need to
//             // consider the same shader features for path draws.
//             [[fallthrough]];
//         case DrawType::midpointFanPatches:
//         case DrawType::midpointFanCenterAAPatches:
//         case DrawType::outerCurvePatches:
//         case DrawType::interiorTriangulation:
//         case DrawType::msaaStrokes:
//         case DrawType::msaaMidpointFanBorrowedCoverage:
//         case DrawType::msaaDynamicMidpointFans:
//         case DrawType::msaaMidpointFans:
//         case DrawType::msaaMidpointFanStencilReset:
//         case DrawType::msaaMidpointFanPathsStencil:
//         case DrawType::msaaMidpointFanPathsCover:
//         case DrawType::msaaOuterCubics:
//             mask = kAllShaderFeatures;
//             break;
//         case DrawType::clipReset:
//             mask = ShaderFeatures::ENABLE_DITHER;
//             break;
//         case DrawType::renderPassInitialize:
//             if (interlockMode == InterlockMode::atomics)
//             {
//                 // Atomic mode initializes clipping and color (when advanced
//                 // blend is active).
//                 mask = ShaderFeatures::ENABLE_CLIPPING |
//                        ShaderFeatures::ENABLE_ADVANCED_BLEND |
//                        ShaderFeatures::ENABLE_DITHER;
//             }
//             else if (interlockMode == InterlockMode::msaa)
//             {
//                 // MSAA mode only needs to initialize color, and only when
//                 // preserving the render target but using a transient MSAA
//                 // attachment.
//                 mask = ShaderFeatures::ENABLE_DITHER;
//             }
//             else
//             {
//                 // The renderPassInitialize draw in clockwiseAtomic mode is just
//                 // a simple workaround that draws a single pixel. No Rive
//                 // ShaderFeatures needed.
//                 assert(interlockMode == InterlockMode::clockwiseAtomic);
//                 mask = ShaderFeatures::NONE;
//             }
//             break;
//         case DrawType::renderPassResolve:
//             if (interlockMode == InterlockMode::atomics)
//             {
//                 mask = kAllShaderFeatures;
//             }
//             else
//             {
//                 assert(interlockMode == InterlockMode::rasterOrdering ||
//                        interlockMode == InterlockMode::msaa);
//                 mask = ShaderFeatures::ENABLE_DITHER;
//             }
//             break;
//     }
//     return mask & ShaderFeaturesMaskFor(interlockMode);
// }
//
// // Returns the flags that are valid for an ubershader version of the currently-
// //  requested shader feature set. There are some shader features that change
// //  how the render passes are set up in atomic mode that need to be accounted
// //  for beyond just using ShaderFeaturesMaskFor.
// constexpr static ShaderFeatures UbershaderFeaturesMaskFor(
//     ShaderFeatures requestedFeatures,
//     DrawType drawType,
//     InterlockMode interlockMode,
//     ShaderMiscFlags shaderMiscFlags,
//     const PlatformFeatures& platformFeatures)
// {
//     ShaderFeatures outFeatures = ShaderFeaturesMaskFor(drawType, interlockMode);
//     if (interlockMode == InterlockMode::atomics)
//     {
//         // Turn off the exclusive atomic features unless they're set in our
//         //  requested feature flags.
//         outFeatures &=
//             (requestedFeatures | ~kExclusiveAtomicUbershaderFeaturesMask);
//     }
//
//     // Ensure that we haven't dropped features we care about somehow
//     assert((requestedFeatures & outFeatures) == requestedFeatures);
//
//     // ENABLE_CLIP_RECT shouldn't be set if we're in MSAA mode without clip
//     // plane support.
//     if (interlockMode == InterlockMode::msaa &&
//         !platformFeatures.supportsClipPlanes)
//     {
//         outFeatures &= ~ShaderFeatures::ENABLE_CLIP_RECT;
//     }
//
//     // Borrowed coverage and anything with fixedFunctionColorOutput cannot
//     // coexist with ENABLE_ADVANCED_BLEND
//     if (enums::any_flag_set(shaderMiscFlags,
//                             ShaderMiscFlags::borrowedCoveragePass |
//                                 ShaderMiscFlags::fixedFunctionColorOutput))
//     {
//         outFeatures &= ~ShaderFeatures::ENABLE_ADVANCED_BLEND;
//     }
//
//     // in atomic mode, coalescedResolveAndTransfer currently implies advanced
//     // blend.
//     if (interlockMode == InterlockMode::atomics &&
//         enums::is_flag_set(shaderMiscFlags,
//                            ShaderMiscFlags::coalescedResolveAndTransfer))
//     {
//         outFeatures |= ShaderFeatures::ENABLE_ADVANCED_BLEND;
//     }
//
//     return outFeatures;
// }
//
// // Returns a unique value that can be used to key a shader.
// uint32_t ShaderUniqueKey(DrawType,
//                          ShaderFeatures,
//                          InterlockMode,
//                          ShaderMiscFlags);
//
// extern const char* GetShaderFeatureGLSLName(ShaderFeatures feature);
//
// void ForEachUbershaderPermutation(
//     InterlockMode,
//     const PlatformFeatures&,
//     const std::function<bool(DrawType, ShaderFeatures, ShaderMiscFlags)>&);
//
// // Flags indicating the contents of a draw. These don't affect shaders, but in
// // msaa mode they are needed to break up batching. (msaa needs different
// // stencil/blend state, depending on the DrawContents.)
// //
// // These also affect the draw sort order, so we attempt associate more expensive
// // shader branch misses with higher flags.
// enum class DrawContents
// {
//     none = 0,
//     opaquePaint = 1 << 0,
//     // Put feathered fills down low because they only need to draw different
//     // geometry, which isn't really a context switch at all.
//     featheredFill = 1 << 1,
//     stroke = 1 << 2,
//     clockwiseFill = 1 << 3,
//     nonZeroFill = 1 << 4,
//     evenOddFill = 1 << 5,
//     activeClip = 1 << 6,
//     advancedBlend = 1 << 7,
//     // Put clip updates last because they use an entirely different shader in
//     // clockwise mode.
//     clipUpdate = 1 << 8,
//
// };
//
// // These are the only draw contents flags that apply to the pipeline state (and
// // they only matter for MSAA)
// constexpr static DrawContents DRAW_CONTENTS_FOR_MSAA_PIPELINE_STATE =
//     DrawContents::activeClip | DrawContents::clipUpdate |
//     DrawContents::clockwiseFill | DrawContents::evenOddFill |
//     DrawContents::opaquePaint;
//
// enum class StencilType
// {
//     disabled,
//     activeStencilClip,
//     borrowedCoverage,
//     forwardClippedByBackward,
//     backwardTriangleCleanup,
//     stencilNestedOrEvenOdd,
//     evenOddDrawAndReset,
//     nestedClipReset,
//     clipReset,
// };
//
// constexpr uint32_t STENCIL_TYPE_BIT_COUNT = 4;
//
// struct StencilInfo
// {
//     StencilType stencilType;
//     DrawContents drawContentsMask;
//     bool areDrawContentsValid = true;
// };
//
// StencilInfo get_stencil_info(InterlockMode, DrawType, DrawContents);
//
// // A nestedClip draw updates the clip buffer while simultaneously clipping
// // against the outerClip that is currently in the clip buffer.
// constexpr static gpu::DrawContents kNestedClipUpdateMask =
//     (gpu::DrawContents::activeClip | gpu::DrawContents::clipUpdate);
//
// // Types of barriers that may be required between DrawBatches.
// enum class BarrierFlags : uint8_t
// {
//     none = 0,
//
//     // Pixel-local dependency in the PLS planes. (Atomic mode only.) Ensure
//     // prior draws complete at each pixel before beginning new ones.
//     plsAtomic = 1 << 0,
//     plsAtomicPreResolve = 1 << 1, // Once before the final resolve.
//
//     // MSAA needs a special barrier (e.g., subpass transition) after manually
//     // loading the render target into the transient MSAA attachment.
//     msaaPostInit = 1 << 2,
//
//     // Pixel-local dependency in the coverage buffer. (clockwiseAtomic mode
//     // only.) All "borrowed coverage" draws have now been issued. Ensure they
//     // complete at each pixel before beginning the "forward coverage" draws.
//     clockwiseBorrowedCoverage = 1 << 3,
//
//     // The next DrawBatch needs to perform an advanced blend, but the current
//     // hardware requires an implementation-dependent barrier before reading the
//     // dstColor (pipeline barrier for input attachments, KHR blend barrier, or
//     // even a full MSAA resolve & blit into a separate texture.)
//     dstBlend = 1 << 4,
//
//     // Special barrier (e.g., subpass transition) issued prior to a manual
//     // render pass resolve. (Only applicable with
//     // FlushDescriptor::manuallyResolved.)
//     preManualResolve = 1 << 5,
//
//     // Only prevent future DrawBatches from being combined with the current
//     // drawList. (No GPU dependencies.)
//     drawBatchBreak = 1 << 6,
// };
//
// // Low-level batch of geometry to submit to the GPU.
// struct DrawBatch
// {
//     DrawBatch(DrawType drawType_,
//               ShaderMiscFlags shaderMiscFlags_,
//               DrawContents drawContents_,
//               uint32_t elementCount_,
//               uint32_t baseElement_,
//               rive::BlendMode blendMode_,
//               rive::ImageSampler imageSampler_,
//               BarrierFlags barrierFlags_) :
//         drawType(drawType_),
//         shaderMiscFlags(shaderMiscFlags_),
//         drawContents(drawContents_),
//         elementCount(elementCount_),
//         baseElement(baseElement_),
//         firstBlendMode(blendMode_),
//         barriers(barrierFlags_),
//         imageSampler(imageSampler_)
//     {}
//
//     const DrawType drawType;
//     ShaderMiscFlags shaderMiscFlags;
//     DrawContents drawContents;
//     // elementCount/baseElement are the "splice axis": the run that grows when
//     // adjacent batches combine. For instanced draws (paths, image draws) that
//     // is instances; for non-indexed triangle runs (interiorTriangulation,
//     // featherAtlasBlit, clipReset) it is vertices.
//     uint32_t elementCount; // Instance count, or vertex count for triangle runs.
//     uint32_t baseElement;  // Base instance, or base vertex for triangle runs.
//     // Geometry parameters for indexed-instanced types (paths and image draws).
//     uint32_t indexCountPerInstance = 0;
//     uint32_t baseIndex = 0;
//     rive::BlendMode firstBlendMode;
//     BarrierFlags barriers; // Barriers to execute before drawing this batch.
//     std::optional<AABBu16> scissorRect;
//
//     ShaderFeatures shaderFeatures = ShaderFeatures::NONE;
//
//     // DrawType::imageRect and DrawType::imageMesh.
//     Texture* imageTexture = nullptr;
//     const ImageSampler imageSampler = ImageSampler::LinearClamp();
//
//     // DrawType::imageMesh.
//     RenderBuffer* vertexBuffer;
//     RenderBuffer* uvBuffer;
//     RenderBuffer* indexBuffer;
//
//     // When shaders don't have a mechanism to read the framebuffer (e.g.,
//     // WebGL msaa), this is a linked list of all the draws in the batch whose
//     // bounding boxes needs to be blitted to the "dstRead" texture before
//     // drawing.
//     const Draw* dstReadList = nullptr;
//
//     // Pointer to the next DrawBatchatch in the list that has a
//     // "BarrierFlags::dstBlend" barrier.
//     // When we need advanced blend but the underlying graphics API doesn't
//     // support reading the framebuffer, this can be helpful for breaking up the
//     // drawList into multiple render passes with framebuffer copies in between.
//     const DrawBatch* nextDstBlendBarrier = nullptr;
//     // Link to the next batch to render in the drawList. DrawBatch always exists
//     // in a linked list.
//
//     const DrawBatch* next = nullptr;
// };
//
// // Simple gradients only have 2 texels, so we write them to mapped texture
// // memory from the CPU instead of rendering them.
// struct TwoTexelRamp
// {
//     ColorInt color0, color1;
// };
// static_assert(sizeof(TwoTexelRamp) == 8 * sizeof(uint8_t));
//
// #ifdef WITH_RIVE_TOOLS
//
// enum class SynthesizedFailureType
// {
//     none,
//     ubershaderLoad,
//     shaderCompilation,
//     pipelineCreation,
// };
//
// #endif
//
// // Detailed description of exactly how a RenderContextImpl should bind its
// // buffers and draw a flush. A typical flush is done in 4 steps:
// //
// //  1. Render the complex gradients from the gradSpanBuffer to the gradient
// //     texture (gradSpanCount, firstComplexGradSpan, complexGradRowsTop,
// //     complexGradRowsHeight).
// //
// //  2. Transfer the simple gradient texels from the simpleColorRampsBuffer to
// //     the top of the gradient texture (simpleGradTexelsWidth,
// //     simpleGradTexelsHeight, simpleGradDataOffsetInBytes, tessDataHeight).
// //
// //  3. Render the tessellation texture from the tessVertexSpanBuffer
// //     (tessVertexSpanCount, firstTessVertexSpan).
// //
// //  4. Execute the drawList, reading from the newly rendered resource textures.
// //
// struct FlushDescriptor
// {
//     RenderTarget* renderTarget = nullptr;
//     ShaderFeatures combinedShaderFeatures = ShaderFeatures::NONE;
//     InterlockMode interlockMode = InterlockMode::rasterOrdering;
//     int msaaSampleCount = 0; // (0 unless interlockMode is msaa.)
//
//     LoadAction colorLoadAction = LoadAction::clear;
//     ColorInt colorClearValue = 0; // When loadAction == LoadAction::clear.
//     uint32_t coverageClearValue = 0;
//     float depthClearValue = DEPTH_MAX;
//     uint8_t stencilClearValue = STENCIL_CLEAR;
//
//     IAABB renderTargetUpdateBounds; // drawBounds, or renderTargetBounds if
//                                     // loadAction == LoadAction::clear.
//
//     // If nonzero, frames are split up into virtual tiles of this size.
//     //
//     // As of now, each tile gets drawn in a separate render pass. The purpose of
//     // these virtual tiles, for now, is to break the frame up into smaller
//     // chunks so that Rive can be pre-empted by other rendering processes. This
//     // is only supported on Vulkan/non-msaa.
//     //
//     // TODO: We could also explore a different type of virtual tiling that
//     // reduces barriers in atomic mode, but that is not how this feature works
//     // currently.
//     uint32_t virtualTileWidth = 0;
//     uint32_t virtualTileHeight = 0;
//
//     // True if the drawList ends with a "renderPassResolve" draw, in which case
//     // the backend may need to perform special setup for a custom resolve.
//     bool manuallyResolved = false;
//
//     // True if shaders will never read the color buffer, meaning, the render
//     // pass can use a more efficient setup that renders to a standard color
//     // attachment and handles all blending via built-in blend state.
//     // NOTE: This may be false even if all paints use srcOver because some
//     // rendering modes (e.g., rasterOrdering with evenOdd/nonZero) always read
//     // the color buffer, regardless of blend mode.
//     bool fixedFunctionColorOutput = false;
//
//     // Physical size of the feather atlas texture.
//     uint16_t featherAtlasTextureWidth;
//     uint16_t featherAtlasTextureHeight;
//
//     // Boundaries of the content for this specific flush within the feather
//     // atlas texture.
//     uint16_t featherAtlasContentWidth;
//     uint16_t featherAtlasContentHeight;
//
//     // Monotonically increasing prefix that gets appended to the most
//     // significant "32 - CLOCKWISE_COVERAGE_BIT_COUNT" bits of coverage buffer
//     // values.
//     //
//     // The coverage buffer is used in clockwiseAtomic mode.
//     //
//     // Increasing this prefix implicitly clears the entire coverage buffer to
//     // zero.
//     uint32_t coverageBufferPrefix = 0;
//
//     // (clockwiseAtomic mode only.) We usually don't have to clear the coverage
//     // buffer because of coverageBufferPrefix, but when this value is true, the
//     // entire coverage buffer must be cleared to zero before rendering.
//     bool needsCoverageBufferClear = false;
//
//     size_t flushUniformDataOffsetInBytes = 0;
//     uint32_t pathCount = 0;
//     size_t firstPath = 0;
//     size_t firstPaint = 0;
//     size_t firstPaintAux = 0;
//     uint32_t contourCount = 0;
//     size_t firstContour = 0;
//     uint32_t gradSpanCount = 0;
//     size_t firstGradSpan = 0;
//     uint32_t tessVertexSpanCount = 0;
//     size_t firstTessVertexSpan = 0;
//     uint32_t gradDataHeight = 0;
//     uint32_t tessDataHeight = 0;
//     // Override path fill rules with "clockwise".
//     bool clockwiseFillOverride = false;
//     bool hasTriangleVertices = false;
//     bool wireframe = false;
//     DitherMode ditherMode;
// #ifdef WITH_RIVE_TOOLS
//     // Synthesize compilation failures to make sure the device handles them
//     // gracefully. (e.g., by falling back on an uber shader or at least not
//     // crashing.) Valid compilations may fail in the real world if the device is
//     // pressed for resources or in a bad state.
//     SynthesizedFailureType synthesizedFailureType =
//         SynthesizedFailureType::none;
// #endif
//
//     // Command buffer that rendering commands will be added to.
//     //  - VkCommandBuffer on Vulkan.
//     //  - id<MTLCommandBuffer> on Metal.
//     //  - Unused otherwise.
//     void* externalCommandBuffer = nullptr;
//
//     // List of feathered fills (if any) that must be rendered to the atlas
//     // before the main render pass.
//     const AtlasDrawBatch* featherAtlasFillBatches = nullptr;
//     size_t featherAtlasFillBatchCount = 0;
//
//     // List of feathered strokes (if any) that must be rendered to the atlas
//     // before the main render pass.
//     const AtlasDrawBatch* featherAtlasStrokeBatches = nullptr;
//     size_t featherAtlasStrokeBatchCount = 0;
//
//     // List of draws in the main render pass. These are rendered directly to the
//     // renderTarget.
//     const BlockAllocatedLinkedList<DrawBatch>* drawList = nullptr;
//     const DrawBatch* firstDstBlendBarrier = nullptr;
//
//     // This tracks any barriers that will not be handled by DrawBatches (e.g.,
//     // renderpass-specific barriers that won't be handled because the batch list
//     // is empty). The backend may need to issue these barriers before finishing
//     // the render pass.
//     BarrierFlags unresolvedBarriers = BarrierFlags::none;
// };
//
// // Returns the area of the (potentially non-rectangular) quadrilateral that
// // results from transforming the given bounds by the given matrix.
// float find_transformed_area(const AABB& bounds, const Mat2D&);
//
// // Convert a BlendMode to the tightly-packed range used by PLS shaders.
// uint32_t ConvertBlendModeToPLSBlendMode(BlendMode riveMode);
//
// // Swizzles the byte order of ColorInt to litte-endian RGBA (the order expected
// // by GLSL).
// RIVE_ALWAYS_INLINE uint32_t SwizzleRiveColorToRGBA(ColorInt riveColor)
// {
//     return (riveColor & 0xff00ff00) |
//            (math::rotateleft32(riveColor, 16) & 0x00ff00ff);
// }
//
// // Swizzles the byte order of ColorInt to litte-endian RGBA (the order expected
// // by GLSL), and premultiplies alpha.
// uint32_t SwizzleRiveColorToRGBAPremul(ColorInt riveColor);
//
// // Used for fields that are used to layout write-only mapped GPU memory.
// // "volatile" to discourage the compiler from generating code that reads these
// // values (e.g., don't let the compiler generate "x ^= x" instead of "x = 0").
// // "RIVE_MAYBE_UNUSED" to suppress -Wunused-private-field.
// #define WRITEONLY RIVE_MAYBE_UNUSED volatile
//
// // Per-flush shared uniforms used by all shaders.
// struct FlushUniforms
// {
// public:
//     FlushUniforms(const FlushDescriptor&, const PlatformFeatures&);
//
//     FlushUniforms(const FlushUniforms& other) { *this = other; }
//
//     void operator=(const FlushUniforms& rhs)
//     {
//         memcpy(static_cast<void*>(this),
//                &rhs,
//                sizeof(*this) - sizeof(m_padTo256Bytes));
//     }
//
//     bool operator!=(const FlushUniforms& rhs) const
//     {
//         return memcmp(this, &rhs, sizeof(*this) - sizeof(m_padTo256Bytes)) != 0;
//     }
//
// private:
//     class InverseViewports
//     {
//     public:
//         InverseViewports() = default;
//
//         InverseViewports(const FlushDescriptor&, const PlatformFeatures&);
//
//     private:
//         // [complexGradientsY, tessDataY, renderTargetX,  renderTargetY]
//         WRITEONLY float m_vals[4];
//     };
//
//     WRITEONLY InverseViewports m_inverseViewports;
//     WRITEONLY uint32_t m_renderTargetWidth;
//     WRITEONLY uint32_t m_renderTargetHeight;
//     // Only used if clears are implemented as draws.
//     WRITEONLY uint32_t m_colorClearValue;
//     // Only used if clears are implemented as draws.
//     WRITEONLY uint32_t m_coverageClearValue;
//     // drawBounds, or renderTargetBounds if there is a clear. (Used by the
//     // "@RESOLVE_PLS" step in InterlockMode::atomics.)
//     WRITEONLY IAABB m_renderTargetUpdateBounds;
//     WRITEONLY Vec2D m_featherAtlasTextureInverseSize; // 1 / [atlasWidth,Height]
//     WRITEONLY Vec2D
//         m_featherAtlasContentInverseViewport; // 2 / atlasContentBounds
//     // Monotonically increasing prefix that gets appended to the most
//     // significant "32 - CLOCKWISE_COVERAGE_BIT_COUNT" bits of coverage buffer
//     // values. (clockwiseAtomic mode only.)
//     WRITEONLY uint32_t m_coverageBufferPrefix;
//     // GLSL doesn't appear to provide a lightweight, region-local barrier for
//     // memory ordering outside of memoryBarrier*(), which have severe
//     // consequences for tiling. When we are already relying on other API level
//     // barriers and only need to guard against instruction reordering, we can
//     // multiply by a tiny epsilon instead, and introduce artifical dependencies
//     // that enforce ordering but don't actually have an effect on the final
//     // outcome.
//     WRITEONLY float m_epsilonForPseudoMemoryBarrier;
//     // Spacing between adjacent path IDs (1 if IEEE compliant).
//     WRITEONLY uint32_t m_pathIDGranularity;
//     WRITEONLY float m_vertexDiscardValue;
//     WRITEONLY float m_mipMapLODBias;
//     WRITEONLY uint32_t m_maxPathId;
//     WRITEONLY float m_ditherScale;
//     WRITEONLY float m_ditherBias;
//     // Amount by which to multiply a computed dither value when storing as
//     // RGB10 (as opposed to writing it out to the framebuffer).
//     WRITEONLY float m_ditherConversionToRGB10;
//     WRITEONLY uint32_t m_wireframeEnabled; // Forces coverage to solid.
//     // Uniform blocks must be multiples of 256 bytes in size.
//     WRITEONLY uint8_t m_padTo256Bytes[256 - 104];
// };
// static_assert(sizeof(FlushUniforms) == 256);
//
// // Storage buffers are logically layed out as arrays of structs on the CPU, but
// // the GPU shaders access them as arrays of basic types. We do it this way in
// // order to be able to easily polyfill them with textures.
// //
// // This enum defines the underlying basic type that each storage buffer struct
// // is layed on top of.
// enum StorageBufferStructure
// {
//     uint32x4,
//     uint32x2,
//     float32x4,
// };
//
// constexpr static uint32_t StorageBufferElementSizeInBytes(
//     StorageBufferStructure bufferStructure)
// {
//     switch (bufferStructure)
//     {
//         case StorageBufferStructure::uint32x4:
//             return sizeof(uint32_t) * 4;
//         case StorageBufferStructure::uint32x2:
//             return sizeof(uint32_t) * 2;
//         case StorageBufferStructure::float32x4:
//             return sizeof(float) * 4;
//     }
//     RIVE_UNREACHABLE();
// }
//
// // Defines a transform from screen space into a region of an atlas.
// // The atlas may have a different scale factor than the screen.
// struct AtlasTransform
// {
//     float scaleFactor;
//     float translateX;
//     float translateY;
// };
//
// // Defines a sub-allocation for a path's coverage data within the
// // renderContext's coverage buffer. (clockwiseAtomic mode only.)
// struct CoverageBufferRange
// {
//     // Index of the first pixel of this allocation within the coverage buffer.
//     // Must be a multiple of 32*32.
//     uint32_t offset;
//     // Line width in pixels of the image in this coverage allocation.
//     // Must be a multiple of 32.
//     uint32_t pitch;
//     // Offset from screen space to image coords within the coverage allocation.
//     float offsetX;
//     float offsetY;
// };
//
// // High level structure of the "path" storage buffer. Each path has a unique
// // data record on the GPU that is accessed from the vertex shader.
// struct PathData
// {
// public:
//     constexpr static StorageBufferStructure kBufferStructure =
//         StorageBufferStructure::uint32x4;
//
//     void set(const Mat2D&,
//              float strokeRadius,
//              float featherRadius,
//              uint32_t zIndex,
//              const AtlasTransform& featherAtlasTransform,
//              const CoverageBufferRange&);
//
// private:
//     WRITEONLY float m_matrix[6];
//     // "0" indicates that the path is filled, not stroked.
//     WRITEONLY float m_strokeRadius;
//     WRITEONLY float m_featherRadius;
//     // InterlockMode::msaa.
//     WRITEONLY uint32_t m_zIndex;
//     // Only used when rendering coverage via the atlas.
//     WRITEONLY AtlasTransform m_featherAtlasTransform;
//     // InterlockMode::clockwiseAtomic.
//     WRITEONLY CoverageBufferRange m_coverageBufferRange;
// };
// static_assert(sizeof(PathData) ==
//               StorageBufferElementSizeInBytes(PathData::kBufferStructure) * 4);
// static_assert(256 % sizeof(PathData) == 0);
// constexpr static size_t kPathBufferAlignmentInElements = 256 / sizeof(PathData);
//
// // High level structure of the "paint" storage buffer. Each path also has a data
// // small record describing its paint at a high level. Complex paints (gradients,
// // images, or any path with a clipRect) store additional rendering info in the
// // PaintAuxData buffer.
// struct PaintData
// {
// public:
//     constexpr static StorageBufferStructure kBufferStructure =
//         StorageBufferStructure::uint32x2;
//
//     void set(DrawContents singleDrawContents,
//              PaintType,
//              SimplePaintValue,
//              GradTextureLayout,
//              uint32_t clipID,
//              bool hasClipRect,
//              BlendMode);
//
// private:
//     WRITEONLY uint32_t m_params; // [clipID, flags, paintType]
//     union
//     {
//         WRITEONLY uint32_t m_color;     // PaintType::solidColor
//         WRITEONLY float m_gradTextureY; // Paintype::linearGradient,
//                                         // Paintype::radialGradient
//         WRITEONLY float m_opacity;      // PaintType::image
//         WRITEONLY uint32_t m_shiftedClipReplacementID; // PaintType::clipUpdate
//     };
// };
// static_assert(sizeof(PaintData) ==
//               StorageBufferElementSizeInBytes(PaintData::kBufferStructure));
// static_assert(256 % sizeof(PaintData) == 0);
// constexpr static size_t kPaintBufferAlignmentInElements =
//     256 / sizeof(PaintData);
//
// // Structure of the "paintAux" storage buffer. Gradients, images, and clipRects
// // store their details here, indexed by pathID.
// struct PaintAuxData
// {
// public:
//     constexpr static StorageBufferStructure kBufferStructure =
//         StorageBufferStructure::float32x4;
//
//     void set(const Mat2D& viewMatrix,
//              PaintType,
//              SimplePaintValue,
//              const Gradient*,
//              const Texture*,
//              const ClipRectInverseMatrix*,
//              const RenderTarget*,
//              const gpu::PlatformFeatures&);
//
// private:
//     WRITEONLY float m_matrix[6]; // Maps _fragCoord to paint coordinates.
//     union
//     {
//         WRITEONLY float
//             m_gradTextureHorizontalSpan[2]; // Paintype::linearGradient,
//                                             // Paintype::radialGradient
//         WRITEONLY float m_imageTextureLOD;  // PaintType::image
//     };
//
//     WRITEONLY float m_clipRectInverseMatrix[6]; // Maps _fragCoord to normalized
//                                                 // clipRect coords.
//     WRITEONLY Vec2D m_inverseFwidth; // -1 / fwidth(matrix * _fragCoord) -- for
//                                      // antialiasing.
// };
// static_assert(sizeof(PaintAuxData) ==
//               StorageBufferElementSizeInBytes(PaintAuxData::kBufferStructure) *
//                   4);
// static_assert(256 % sizeof(PaintAuxData) == 0);
// constexpr static size_t kPaintAuxBufferAlignmentInElements =
//     256 / sizeof(PaintAuxData);
//
// // High level structure of the "contour" storage buffer. Each contour of every
// // path has a data record describing its info.
// struct ContourData
// {
// public:
//     constexpr static StorageBufferStructure kBufferStructure =
//         StorageBufferStructure::uint32x4;
//
//     ContourData(Vec2D midpoint, uint32_t pathID, uint32_t vertexIndex0) :
//         m_midpoint(midpoint), m_pathID(pathID), m_vertexIndex0(vertexIndex0)
//     {}
//
// private:
//     WRITEONLY Vec2D
//         m_midpoint; // Midpoint of the curve endpoints in just this contour.
//     WRITEONLY uint32_t m_pathID; // ID of the path this contour belongs to.
//     WRITEONLY uint32_t m_vertexIndex0; // Index of the first tessellation vertex
//                                        // of the contour.
// };
// static_assert(sizeof(ContourData) ==
//               StorageBufferElementSizeInBytes(ContourData::kBufferStructure));
// static_assert(256 % sizeof(ContourData) == 0);
// constexpr static size_t kContourBufferAlignmentInElements =
//     256 / sizeof(ContourData);
//
// // Per-vertex data for shaders that draw triangles.
// struct TriangleVertex
// {
// public:
//     TriangleVertex() = default;
//     TriangleVertex(Vec2D point, int16_t weight, uint16_t pathID) :
//         m_point(point),
//         m_weight_pathID((static_cast<int32_t>(weight) << 16) | pathID)
//     {}
//
// #ifdef TESTING
//     Vec2D testing_point() const { return {m_point.x, m_point.y}; }
//     int32_t testing_weight_pathID() const { return m_weight_pathID; }
// #endif
//
// private:
//     WRITEONLY Vec2D m_point;
//     WRITEONLY int32_t m_weight_pathID; // [(weight << 16 | pathID]
// };
// static_assert(sizeof(TriangleVertex) == sizeof(float) * 3);
//
// // Per-draw instanced attributes used by imageMeshes and imageRects.
// struct ImageDrawInstance
// {
// public:
//     // This data is bound to image shaders as 4 tightly-packed instanced
//     // attributes. The vertex shader unpacks them.
//     //   attr 2 (float4): viewMatrix (2x2)
//     //   attr 3 (float4): clipRectInverseMatrix (2x2)
//     //   attr 4 (float4): translates for view & clipRectInverseMatrix
//     //   attr 5 (uint4) : opacity (uintBitsToFloat), clipID, blendMode, zIndex
//     constexpr static size_t FirstAttribIdx = 2;
//     constexpr static size_t LastAttribIdx = 5;
//     constexpr static size_t AttribCount = LastAttribIdx + 1 - FirstAttribIdx;
//
//     ImageDrawInstance() = default;
//
//     ImageDrawInstance(const Mat2D&,
//                       float opacity,
//                       const ClipRectInverseMatrix*,
//                       uint32_t clipID,
//                       BlendMode,
//                       uint32_t zIndex);
//
// private:
//     WRITEONLY float m_viewMatrix[4];
//     WRITEONLY float m_clipRectInverseMatrix[4];
//     WRITEONLY float m_translate[2];
//     WRITEONLY float m_clipRectInverseTranslate[2];
//     WRITEONLY float m_opacity;
//     WRITEONLY uint32_t m_clipID;
//     WRITEONLY uint32_t m_blendMode;
//     WRITEONLY uint32_t m_zIndex;
// };
//
// #undef WRITEONLY
//
// // The maximum number of storage buffers we will ever use in a vertex or
// // fragment shader.
// constexpr static size_t kMaxStorageBuffers = 4;
//
// // If the backend doesn't support "kMaxStorageBuffers" a shader, we polyfill
// // with textures. This function returns the dimensions to use for these
// // textures.
// std::tuple<uint32_t, uint32_t> StorageTextureSize(size_t bufferSizeInBytes,
//                                                   StorageBufferStructure);
//
// // If the backend doesn't support "kMaxStorageBuffers" in a shader, we polyfill
// // with textures. The polyfill texture needs to be updated in entire rows at a
// // time, meaning, its transfer buffer might need to be larger than requested.
// // This function returns a size that is large enough to service a worst-case
// // texture update.
// size_t StorageTextureBufferSize(size_t bufferSizeInBytes,
//                                 StorageBufferStructure);
//
// // Should the triangulator emit triangles with negative winding, positive
// // winding, or both?
// enum class WindingFaces
// {
//     none = 0,
//     negative = 1 << 0,
//     positive = 1 << 1,
//     all = negative | positive,
// };
//
// // Represents a block of mapped GPU memory. Since it can be extremely expensive
// // to read mapped memory, we use this class to enforce the write-only nature of
// // this memory.
// template <typename T> class WriteOnlyMappedMemory
// {
// public:
//     WriteOnlyMappedMemory() { reset(); }
//     WriteOnlyMappedMemory(T* ptr, size_t elementCount)
//     {
//         reset(ptr, elementCount);
//     }
//
//     void reset() { reset(nullptr, 0); }
//
//     void reset(T* ptr, size_t elementCount)
//     {
//         m_mappedMemory = ptr;
//         m_nextMappedItem = ptr;
//         m_mappingEnd = ptr + elementCount;
//     }
//
//     using MapResourceBufferFn =
//         void* (RenderContextImpl::*)(size_t mapSizeInBytes);
//     [[nodiscard]] bool mapElements(RenderContextImpl* impl,
//                                    MapResourceBufferFn mapFn,
//                                    size_t elementCount)
//     {
//         assert(m_mappedMemory == nullptr);
//         void* ptr = (impl->*mapFn)(elementCount * sizeof(T));
//         if (ptr == nullptr)
//         {
//             return false;
//         }
//
//         reset(reinterpret_cast<T*>(ptr), elementCount);
//         return true;
//     }
//
//     using UnmapResourceBufferFn =
//         void (RenderContextImpl::*)(size_t mapSizeInBytes);
//     void unmapElements(RenderContextImpl* impl,
//                        UnmapResourceBufferFn unmapFn,
//                        size_t elementCount)
//     {
//         if (m_mappedMemory != nullptr)
//         {
//             assert(m_mappingEnd - m_mappedMemory == elementCount);
//             (impl->*unmapFn)(elementCount * sizeof(T));
//             reset();
//         }
//     }
//
//     operator bool() const { return m_mappedMemory; }
//
//     // How many bytes have been written to the buffer?
//     size_t bytesWritten() const
//     {
//         return reinterpret_cast<uintptr_t>(m_nextMappedItem) -
//                reinterpret_cast<uintptr_t>(m_mappedMemory);
//     }
//
//     size_t elementsWritten() const { return bytesWritten() / sizeof(T); }
//
//     // Is there room to push() itemCount items to the buffer?
//     bool hasRoomFor(size_t itemCount)
//     {
//         return m_nextMappedItem + itemCount <= m_mappingEnd;
//     }
//
//     // Append and write a new item to the buffer. In order to enforce the
//     // write-only requirement of a mapped buffer, these methods do not return
//     // any pointers to the client.
//     template <typename... Args>
//     RIVE_ALWAYS_INLINE void emplace_back(Args&&... args)
//     {
//         new (&push()) T(std::forward<Args>(args)...);
//     }
//     template <typename... Args> RIVE_ALWAYS_INLINE void set_back(Args&&... args)
//     {
//         push().set(std::forward<Args>(args)...);
//     }
//     void push_back_n(const T* values, size_t count)
//     {
//         T* dst = push(count);
//         if (values != nullptr)
//         {
//             memcpy(static_cast<void*>(dst), values, count * sizeof(T));
//         }
//     }
//     void skip_back() { push(); }
//
// private:
//     RIVE_ALWAYS_INLINE T& push()
//     {
//         assert(hasRoomFor(1));
//         return *m_nextMappedItem++;
//     }
//     RIVE_ALWAYS_INLINE T* push(size_t count)
//     {
//         assert(hasRoomFor(count));
//         T* ret = m_nextMappedItem;
//         m_nextMappedItem += count;
//         return ret;
//     }
//
//     T* m_mappedMemory;
//     T* m_nextMappedItem;
//     const T* m_mappingEnd;
// };
//
// // Utility for tracking booleans that may be unknown (e.g., lazily computed
// // values, GL state, etc.)
// enum class TriState
// {
//     no,
//     yes,
//     unknown
// };
//
// enum class StencilOp : uint8_t
// {
//     keep,
//     replace,
//     zero,
//     decrClamp,
//     incrWrap,
//     decrWrap
// };
//
// enum class StencilCompareOp : uint8_t
// {
//     less,
//     equal,
//     lessOrEqual,
//     notEqual,
//     always,
// };
//
// struct StencilFaceOps
// {
//     StencilOp stencilFailOp = StencilOp::keep;
//     StencilOp depthFailOp = StencilOp::keep;
//     StencilOp depthStencilPassOp = StencilOp::keep;
//     StencilCompareOp compareOp = StencilCompareOp::always;
// };
//
// enum class CullFace : uint8_t
// {
//     none,
//     clockwise,
//     counterclockwise,
// };
//
// constexpr uint32_t CULL_FACE_BIT_COUNT = 2;
//
// // Blend equation to select for the fixed-function GPU pipeline (not our own
// // in-shader blending). For now, the backend is free to decide whether it will
// // use premultiplied alpha or not.
// enum class BlendEquation : uint8_t
// {
//     // Hardware blend is disabled.
//     none = 0,
//
//     // Core hardware blend equations supported on all platforms.
//     srcOver = static_cast<int>(rive::BlendMode::srcOver),
//     plus = srcOver + 1,
//     min = srcOver + 2,
//     max = srcOver + 3,
//
//     // "Advanced" hardware blend equations.
//     // PlatformFeatures::supportsKHRBlendEquations is required.
//     screen = static_cast<int>(rive::BlendMode::screen),
//     overlay = static_cast<int>(rive::BlendMode::overlay),
//     darken = static_cast<int>(rive::BlendMode::darken),
//     lighten = static_cast<int>(rive::BlendMode::lighten),
//     colorDodge = static_cast<int>(rive::BlendMode::colorDodge),
//     colorBurn = static_cast<int>(rive::BlendMode::colorBurn),
//     hardLight = static_cast<int>(rive::BlendMode::hardLight),
//     softLight = static_cast<int>(rive::BlendMode::softLight),
//     difference = static_cast<int>(rive::BlendMode::difference),
//     exclusion = static_cast<int>(rive::BlendMode::exclusion),
//     multiply = static_cast<int>(rive::BlendMode::multiply),
//     hue = static_cast<int>(rive::BlendMode::hue),
//     saturation = static_cast<int>(rive::BlendMode::saturation),
//     color = static_cast<int>(rive::BlendMode::color),
//     luminosity = static_cast<int>(rive::BlendMode::luminosity),
// };
//
// struct DepthState
// {
//     bool depthTestEnabled;
//     bool depthWriteEnabled;
// };
//
// DepthState get_depth_state(InterlockMode interlockMode,
//                            DrawType drawType,
//                            DrawContents drawContents);
//
// CullFace get_cull_face(DrawType drawType);
// bool get_color_write_enable(DrawType drawType,
//                             InterlockMode interlockMode,
//                             ShaderMiscFlags shaderMiscFlags,
//                             bool fixedFunctionColorOutput,
//                             DrawContents drawContents);
//
// // Common pipeline state that applies to every Rive draw and every backend.
// struct PipelineState
// {
//     // Depth.
//     bool depthTestEnabled = false;
//     bool depthWriteEnabled = true;
//
//     // Stencil.
//     bool stencilTestEnabled = false;
//     uint8_t stencilCompareMask = 0xff;
//     uint8_t stencilWriteMask = 0xff;
//     uint8_t stencilReference = 0;
//     StencilFaceOps stencilFrontOps;
//     StencilFaceOps stencilBackOps;
//     bool stencilDoubleSided = false; // Use stencilFrontOps for both faces?
//
//     CullFace cullFace = CullFace::none;
//     BlendEquation blendEquation = BlendEquation::none;
//     bool colorWriteEnabled = true;
// };
//
// // Returns a unique value that can be used to key a whole pipeline.
// uint64_t pipeline_unique_key(DrawType,
//                              ShaderFeatures,
//                              InterlockMode,
//                              ShaderMiscFlags,
//                              DrawContents,
//                              bool fixedFunctionColorOutput,
//                              rive::BlendMode,
//                              const PlatformFeatures&);
//
// PipelineState get_pipeline_state(DrawType,
//                                  InterlockMode,
//                                  ShaderMiscFlags,
//                                  DrawContents,
//                                  bool fixedFunctionColorOutput,
//                                  rive::BlendMode,
//                                  const PlatformFeatures&);
//
// void get_pipeline_state(const DrawBatch&,
//                         const FlushDescriptor&,
//                         const PlatformFeatures&,
//                         PipelineState*);
//
// // Default PipelineState values as specified in OpenGL.
// constexpr static PipelineState GL_DEFAULT_PIPELINE_STATE = {};
//
// // Helper to create PipelineState with no depth/stencil and custom blend/cull.
// constexpr inline PipelineState make_flat_pipeline_state(CullFace cull,
//                                                         BlendEquation blend)
// {
//     PipelineState s{};
//     s.depthTestEnabled = false;
//     s.depthWriteEnabled = false;
//     s.stencilTestEnabled = false;
//     s.stencilWriteMask = 0;
//     s.cullFace = cull;
//     s.blendEquation = blend;
//     s.colorWriteEnabled = true;
//     return s;
// }
//
// constexpr static PipelineState COLOR_ONLY_PIPELINE_STATE =
//     make_flat_pipeline_state(CullFace::none, BlendEquation::none);
//
// constexpr static PipelineState FEATHER_ATLAS_FILL_PIPELINE_STATE =
//     make_flat_pipeline_state(CullFace::none, BlendEquation::plus);
//
// constexpr static PipelineState FEATHER_ATLAS_STROKE_PIPELINE_STATE =
//     make_flat_pipeline_state(CullFace::counterclockwise, BlendEquation::max);
//
// float4 cast_f16_to_f32(uint16x4 x);
// uint16x4 cast_f32_to_f16(float4);
//
// // These tables integrate the gaussian function, and its inverse, covering a
// // spread of -GAUSSIAN_INTEGRAL_TEXTURE_STDDEVS to
// // +GAUSSIAN_INTEGRAL_TEXTURE_STDDEVS.
// constexpr static uint32_t GAUSSIAN_TABLE_SIZE = 512;
// extern const uint16_t g_gaussianIntegralTableF16[GAUSSIAN_TABLE_SIZE];
// extern const uint16_t g_inverseGaussianIntegralTableF16[GAUSSIAN_TABLE_SIZE];
//
// // Code to generate g_gaussianIntegralTableF16 and
// // g_inverseGaussianIntegralTableF32. This is left in the codebase but #ifdef'd
// // out in case we ever want to change any parameters of the built-in tables.
// #ifdef RIVE_GENERATE_FEATHER_LUT
// void generate_gausian_integral_table(float (&)[GAUSSIAN_TABLE_SIZE]);
// void generate_inverse_gausian_integral_table(float (&)[GAUSSIAN_TABLE_SIZE]);
// #endif
// } // namespace rive::gpu

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/gpu.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
// Ownership unit: generic-gpu-contract.
// Include/dependency authority: docs/render-context-metal-includes.tsv and
// docs/metal-port-source-dependencies.tsv.

// Rust declaration pass for the complete source header above.  The source
// comments are intentionally retained verbatim; this file is a mechanical
// owner and is not the place to introduce a cross-backend GPU abstraction.
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_void;
use core::marker::PhantomData;
use core::ops::{BitAnd, BitOr, Not};
use core::ptr::NonNull;

use nuxie_render_api::{Aabb as AABB, BlendMode, ColorInt, Mat2D, Vec2D};

// Mapped source dependencies.  These aliases keep the upstream names visible
// while the owning mechanical files are wired into the crate.
pub use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler;
pub use crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer;
pub use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    BlockAllocatedLinkedList, DitherMode, Draw,
};
pub use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImpl;
pub use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
pub use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AABBu16 {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IAABB {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl IAABB {
    pub const fn empty(&self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    pub const fn makeMaximal() -> Self {
        Self {
            left: i32::MIN,
            top: i32::MIN,
            right: i32::MAX,
            bottom: i32::MAX,
        }
    }

    pub const fn makeMaximallyNegative() -> Self {
        Self {
            left: i32::MAX,
            top: i32::MAX,
            right: i32::MIN,
            bottom: i32::MIN,
        }
    }
}

pub enum GrInnerFanTriangulator {}

pub const MIP_MAP_LOD_BIAS: f32 = -0.5;
pub const kParametricPrecision: i32 = 4;
pub const kPolarPrecision: i32 = 8;
pub const kMaxParametricSegments: u32 = 1023;
pub const kMaxPolarSegments: u32 = 1023;
pub const FEATHER_POLAR_SEGMENT_MIN_ANGLE: f32 = core::f32::consts::PI / 16.0;
pub const COS_FEATHER_POLAR_SEGMENT_MIN_ANGLE_OVER_2: f32 = 0.99518472667;
pub const kBufferRingSize: i32 = 3;
pub const kLargestFP16BeforeExponentAll1s: i32 = (0x1f << 10) - 1;
pub const kLargestDenormalizedFP16: i32 = 1023;

#[inline]
pub const fn MaxPathID(granularity: i32) -> i32 {
    kLargestFP16BeforeExponentAll1s / granularity - kLargestDenormalizedFP16
}

pub const kMaxContourID: usize = 65535;
pub const kContourIDMask: u32 = 0xffff;
pub const kTessTextureWidth: usize = 2048;
pub const kTessTextureWidthLog2: usize = 11;
pub const kGradTextureWidth: u32 = 512;
pub const kGradTextureWidthInSimpleRamps: u32 = kGradTextureWidth / 2;
pub const DEPTH_MIN: f32 = 0.0;
pub const DEPTH_MAX: f32 = 1.0;
pub const STENCIL_CLEAR: u8 = 0;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlatformFeatures {
    pub supportsRasterOrderingMode: bool,
    pub supportsAtomicMode: bool,
    pub supportsClockwiseMode: bool,
    pub supportsClockwiseFixedFunctionMode: bool,
    pub supportsClockwiseAtomicMode: bool,
    pub supportsBlendAdvancedKHR: bool,
    pub supportsBlendAdvancedCoherentKHR: bool,
    pub supportsClipPlanes: bool,
    pub supportsPipelineDynamicState: bool,
    pub avoidFlatVaryings: bool,
    pub alwaysFeatherToAtlas: bool,
    pub clipSpaceBottomUp: bool,
    pub framebufferBottomUp: bool,
    pub atomicPLSInitNeedsDraw: bool,
    pub msaaColorPreserveNeedsDraw: bool,
    pub clockwiseAtomicBorrowedCoverageBarrierNeedsRenderPassInit: bool,
    pub pathIDGranularity: u8,
    pub maxTextureSize: u32,
    pub maxCoverageBufferLength: usize,
    pub supportsClipScissor: bool,
    pub supportsTextureCompressionBC: bool,
    pub supportsTextureCompressionASTC: bool,
    pub supportsTextureCompressionETC2: bool,
}

impl Default for PlatformFeatures {
    fn default() -> Self {
        Self {
            supportsRasterOrderingMode: false,
            supportsAtomicMode: false,
            supportsClockwiseMode: false,
            supportsClockwiseFixedFunctionMode: false,
            supportsClockwiseAtomicMode: false,
            supportsBlendAdvancedKHR: false,
            supportsBlendAdvancedCoherentKHR: false,
            supportsClipPlanes: false,
            supportsPipelineDynamicState: false,
            avoidFlatVaryings: false,
            alwaysFeatherToAtlas: false,
            clipSpaceBottomUp: false,
            framebufferBottomUp: false,
            atomicPLSInitNeedsDraw: false,
            msaaColorPreserveNeedsDraw: false,
            clockwiseAtomicBorrowedCoverageBarrierNeedsRenderPassInit: false,
            pathIDGranularity: 1,
            maxTextureSize: 2048,
            maxCoverageBufferLength: (1usize << 27) / core::mem::size_of::<u32>(),
            supportsClipScissor: false,
            supportsTextureCompressionBC: false,
            supportsTextureCompressionASTC: false,
            supportsTextureCompressionETC2: false,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GradientSpan {
    pub horizontalSpan: u32,
    pub yWithFlags: u32,
    pub color0: u32,
    pub color1: u32,
}

impl GradientSpan {
    #[inline(always)]
    pub fn set(
        &mut self,
        x0Fixed: u32,
        x1Fixed: u32,
        y: u32,
        flags: u32,
        color0_: ColorInt,
        color1_: ColorInt,
    ) {
        debug_assert!(x0Fixed < 65536);
        debug_assert!(x1Fixed < 65536);
        self.horizontalSpan = (x1Fixed << 16) | x0Fixed;
        self.yWithFlags = flags | y;
        self.color0 = color0_;
        self.color1 = color1_;
    }
}

pub const kGradSpanBufferAlignmentInElements: usize = 256 / core::mem::size_of::<GradientSpan>();
pub const GRAD_SPAN_TRI_STRIP_VERTEX_COUNT: u32 = 8;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TessVertexSpan {
    pub pts: [Vec2D; 4],
    pub joinTangent: Vec2D,
    pub y: f32,
    pub reflectionY: f32,
    pub x0x1: i32,
    pub reflectionX0X1: i32,
    pub segmentCounts: u32,
    pub contourIDWithFlags: u32,
}

impl Default for TessVertexSpan {
    fn default() -> Self {
        Self {
            pts: [Vec2D::new(0.0, 0.0); 4],
            joinTangent: Vec2D::new(0.0, 0.0),
            y: 0.0,
            reflectionY: 0.0,
            x0x1: 0,
            reflectionX0X1: 0,
            segmentCounts: 0,
            contourIDWithFlags: 0,
        }
    }
}

impl TessVertexSpan {
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub fn set_without_reflection(
        &mut self,
        pts_: [Vec2D; 4],
        joinTangent_: Vec2D,
        y_: f32,
        x0: i32,
        x1: i32,
        parametricSegmentCount: u32,
        polarSegmentCount: u32,
        joinSegmentCount: u32,
        contourIDWithFlags_: u32,
    ) {
        self.set(
            pts_,
            joinTangent_,
            y_,
            x0,
            x1,
            f32::NAN,
            -1,
            -1,
            parametricSegmentCount,
            polarSegmentCount,
            joinSegmentCount,
            contourIDWithFlags_,
        );
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub fn set(
        &mut self,
        pts_: [Vec2D; 4],
        joinTangent_: Vec2D,
        y_: f32,
        x0: i32,
        x1: i32,
        reflectionY_: f32,
        reflectionX0: i32,
        reflectionX1: i32,
        parametricSegmentCount: u32,
        polarSegmentCount: u32,
        joinSegmentCount: u32,
        contourIDWithFlags_: u32,
    ) {
        // #ifndef NDEBUG: write a local copy because mapped memory is
        // write-only; #else write directly to the mapped destination.
        #[cfg(debug_assertions)]
        let mut localCopy = *self;
        #[cfg(debug_assertions)]
        let target = &mut localCopy;
        #[cfg(not(debug_assertions))]
        let target = &mut *self;

        target.pts = pts_;
        target.joinTangent = joinTangent_;
        target.y = y_;
        target.reflectionY = reflectionY_;
        target.x0x1 = (x1 << 16) | (x0 & 0xffff);
        target.reflectionX0X1 = (reflectionX1 << 16) | (reflectionX0 & 0xffff);
        target.segmentCounts =
            (joinSegmentCount << 20) | (polarSegmentCount << 10) | parametricSegmentCount;
        target.contourIDWithFlags = contourIDWithFlags_;
        debug_assert!((target.x0x1 << 16 >> 16) == x0);
        debug_assert!((target.x0x1 >> 16) == x1);
        debug_assert!((target.reflectionX0X1 << 16 >> 16) == reflectionX0);
        debug_assert!((target.reflectionX0X1 >> 16) == reflectionX1);
        debug_assert!((target.segmentCounts & 0x3ff) == parametricSegmentCount);
        debug_assert!(((target.segmentCounts >> 10) & 0x3ff) == polarSegmentCount);
        debug_assert!((target.segmentCounts >> 20) == joinSegmentCount);
        #[cfg(debug_assertions)]
        {
            *self = localCopy;
        }
    }
}

pub const kTessVertexBufferAlignmentInElements: usize =
    256 / core::mem::size_of::<TessVertexSpan>();
pub const kTessSpanIndices: [u16; 12] = [0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7];

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImageRectVertex {
    pub x: f32,
    pub y: f32,
    pub aaOffsetX: f32,
    pub aaOffsetY: f32,
}

pub const kImageRectVertices: [ImageRectVertex; 12] = [
    ImageRectVertex {
        x: 0.0,
        y: 0.0,
        aaOffsetX: 0.0,
        aaOffsetY: -1.0,
    },
    ImageRectVertex {
        x: 1.0,
        y: 0.0,
        aaOffsetX: 0.0,
        aaOffsetY: -1.0,
    },
    ImageRectVertex {
        x: 1.0,
        y: 0.0,
        aaOffsetX: 1.0,
        aaOffsetY: 0.0,
    },
    ImageRectVertex {
        x: 1.0,
        y: 1.0,
        aaOffsetX: 1.0,
        aaOffsetY: 0.0,
    },
    ImageRectVertex {
        x: 1.0,
        y: 1.0,
        aaOffsetX: 0.0,
        aaOffsetY: 1.0,
    },
    ImageRectVertex {
        x: 0.0,
        y: 1.0,
        aaOffsetX: 0.0,
        aaOffsetY: 1.0,
    },
    ImageRectVertex {
        x: 0.0,
        y: 1.0,
        aaOffsetX: -1.0,
        aaOffsetY: 0.0,
    },
    ImageRectVertex {
        x: 0.0,
        y: 0.0,
        aaOffsetX: -1.0,
        aaOffsetY: 0.0,
    },
    ImageRectVertex {
        x: 0.0,
        y: 0.0,
        aaOffsetX: 1.0,
        aaOffsetY: 1.0,
    },
    ImageRectVertex {
        x: 1.0,
        y: 0.0,
        aaOffsetX: -1.0,
        aaOffsetY: 1.0,
    },
    ImageRectVertex {
        x: 1.0,
        y: 1.0,
        aaOffsetX: -1.0,
        aaOffsetY: -1.0,
    },
    ImageRectVertex {
        x: 0.0,
        y: 1.0,
        aaOffsetX: 1.0,
        aaOffsetY: -1.0,
    },
];

pub const kImageRectIndices: [u16; 42] = [
    8, 0, 9, 9, 0, 1, 1, 2, 9, 9, 2, 10, 10, 2, 3, 3, 4, 10, 10, 4, 11, 11, 4, 5, 5, 6, 11, 11, 6,
    8, 8, 6, 7, 7, 0, 8, 9, 10, 8, 10, 8, 11,
];

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintType {
    clipUpdate = 0,
    solidColor = 1,
    linearGradient = 2,
    radialGradient = 3,
    image = 4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorRampLocation {
    pub row: u16,
    pub col: u16,
}

impl ColorRampLocation {
    pub const kComplexGradientMarker: u16 = 0xffff;
    pub const fn isComplex(&self) -> bool {
        self.col == Self::kComplexGradientMarker
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union SimplePaintValue {
    pub color: ColorInt,
    pub colorRampLocation: ColorRampLocation,
    pub imageOpacity: f32,
    pub outerClipID: u32,
}

impl Default for SimplePaintValue {
    fn default() -> Self {
        Self { color: 0xff000000 }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipRectInverseMatrix {
    m_inverseMatrix: Mat2D,
}

impl ClipRectInverseMatrix {
    pub const fn WideOpen() -> Self {
        Self {
            m_inverseMatrix: Mat2D([0.0, 0.0, 0.0, 0.0, 1.0, 1.0]),
        }
    }
    pub const fn Empty() -> Self {
        Self {
            m_inverseMatrix: Mat2D([0.0; 6]),
        }
    }
    pub const fn new() -> Self {
        Self {
            // Mat2D's default constructor is identity; the all-zero matrix is
            // reserved for ClipRectInverseMatrix::Empty().
            m_inverseMatrix: Mat2D::IDENTITY,
        }
    }
    pub const fn from_inverse(matrix: Mat2D) -> Self {
        Self {
            m_inverseMatrix: matrix,
        }
    }
    // void reset(const Mat2D& clipMatrix, const AABB& clipRect);
    pub const fn inverseMatrix(&self) -> &Mat2D {
        &self.m_inverseMatrix
    }
}

impl Default for ClipRectInverseMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GradTextureLayout {
    pub complexOffsetY: u32,
    pub inverseHeight: f32,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatchType {
    midpointFan = 0,
    midpointFanCenterAA = 1,
    outerCurves = 2,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContourDirections {
    forward = 0,
    reverse = 1,
    reverseThenForward = 2,
    forwardThenReverse = 3,
}

pub const fn ContourDirectionsAreDoubleSided(value: ContourDirections) -> bool {
    (value as u8) >= ContourDirections::reverseThenForward as u8
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PatchVertex {
    pub localVertexID: f32,
    pub outset: f32,
    pub fillCoverage: f32,
    pub params: i32,
    pub mirroredVertexID: f32,
    pub mirroredOutset: f32,
    pub mirroredFillCoverage: f32,
    pub padding: i32,
}

impl PatchVertex {
    pub fn set(&mut self, localVertexID_: f32, outset_: f32, fillCoverage_: f32, params_: i32) {
        self.localVertexID = localVertexID_;
        self.outset = outset_;
        self.fillCoverage = fillCoverage_;
        self.params = params_;
        self.setMirroredPosition(localVertexID_, outset_, fillCoverage_);
    }
    pub fn setMirroredPosition(&mut self, localVertexID_: f32, outset_: f32, fillCoverage_: f32) {
        self.mirroredVertexID = localVertexID_;
        self.mirroredOutset = outset_;
        self.mirroredFillCoverage = fillCoverage_;
    }
}

pub const kMidpointFanPatchSegmentSpan: u32 = 8;
pub const kOuterCurvePatchSegmentSpan: u32 = 17;
pub const kMidpointFanPatchVertexCount: u32 =
    kMidpointFanPatchSegmentSpan * 4 + (kMidpointFanPatchSegmentSpan + 1) + 1;
pub const kMidpointFanPatchBorderIndexCount: u32 = kMidpointFanPatchSegmentSpan * 6;
pub const kMidpointFanPatchIndexCount: u32 =
    kMidpointFanPatchBorderIndexCount + (kMidpointFanPatchSegmentSpan - 1) * 3 + 3;
pub const kMidpointFanPatchBaseIndex: u32 = 0;
pub const kMidpointFanCenterAAPatchVertexCount: u32 =
    kMidpointFanPatchSegmentSpan * 4 * 2 + (kMidpointFanPatchSegmentSpan + 1) + 1;
pub const kMidpointFanCenterAAPatchBorderIndexCount: u32 = kMidpointFanPatchSegmentSpan * 12;
pub const kMidpointFanCenterAAPatchIndexCount: u32 =
    kMidpointFanCenterAAPatchBorderIndexCount + (kMidpointFanPatchSegmentSpan - 1) * 3 + 3;
pub const kMidpointFanCenterAAPatchBaseIndex: u32 =
    kMidpointFanPatchBaseIndex + kMidpointFanPatchIndexCount;
pub const kOuterCurvePatchVertexCount: u32 =
    kOuterCurvePatchSegmentSpan * 8 + kOuterCurvePatchSegmentSpan;
pub const kOuterCurvePatchBorderIndexCount: u32 = kOuterCurvePatchSegmentSpan * 12;
pub const kOuterCurvePatchIndexCount: u32 =
    kOuterCurvePatchBorderIndexCount + (kOuterCurvePatchSegmentSpan - 2) * 3;
pub const kOuterCurvePatchBaseIndex: u32 =
    kMidpointFanCenterAAPatchBaseIndex + kMidpointFanCenterAAPatchIndexCount;
pub const kPatchVertexBufferCount: u32 = kMidpointFanPatchVertexCount
    + kMidpointFanCenterAAPatchVertexCount
    + kOuterCurvePatchVertexCount;
pub const kPatchIndexBufferCount: u32 =
    kMidpointFanPatchIndexCount + kMidpointFanCenterAAPatchIndexCount + kOuterCurvePatchIndexCount;

extern "C" {
    pub fn GeneratePatchBufferData(vertices: *mut PatchVertex, indices: *mut u16);
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawType {
    midpointFanPatches = 0,
    midpointFanCenterAAPatches = 1,
    outerCurvePatches = 2,
    interiorTriangulation = 3,
    featherAtlasBlit = 4,
    imageRect = 5,
    imageMesh = 6,
    msaaStrokes = 7,
    msaaMidpointFanBorrowedCoverage = 8,
    msaaMidpointFans = 9,
    msaaMidpointFanStencilReset = 10,
    msaaDynamicMidpointFans = 11,
    msaaMidpointFanPathsStencil = 12,
    msaaMidpointFanPathsCover = 13,
    msaaOuterCubics = 14,
    clipReset = 15,
    renderPassInitialize = 16,
    renderPassResolve = 17,
}

// Source-name spellings used by the translated Metal unit.  These are
// associated constants on the one canonical GPU enum, not a second DTO
// universe; all ABI and identity remain `gpu::DrawType`.
impl DrawType {
    pub const MidpointFanPatches: Self = Self::midpointFanPatches;
    pub const MidpointFanCenterAAPatches: Self = Self::midpointFanCenterAAPatches;
    pub const OuterCurvePatches: Self = Self::outerCurvePatches;
    pub const InteriorTriangulation: Self = Self::interiorTriangulation;
    pub const FeatherAtlasBlit: Self = Self::featherAtlasBlit;
    pub const ImageRect: Self = Self::imageRect;
    pub const ImageMesh: Self = Self::imageMesh;
    pub const MsaaStrokes: Self = Self::msaaStrokes;
    pub const MsaaMidpointFanBorrowedCoverage: Self = Self::msaaMidpointFanBorrowedCoverage;
    pub const MsaaMidpointFans: Self = Self::msaaMidpointFans;
    pub const MsaaMidpointFanStencilReset: Self = Self::msaaMidpointFanStencilReset;
    pub const MsaaDynamicMidpointFans: Self = Self::msaaDynamicMidpointFans;
    pub const MsaaMidpointFanPathsStencil: Self = Self::msaaMidpointFanPathsStencil;
    pub const MsaaMidpointFanPathsCover: Self = Self::msaaMidpointFanPathsCover;
    pub const MsaaOuterCubics: Self = Self::msaaOuterCubics;
    pub const ClipReset: Self = Self::clipReset;
    pub const RenderPassInitialize: Self = Self::renderPassInitialize;
    pub const RenderPassResolve: Self = Self::renderPassResolve;
}

pub const fn DrawTypeIsImageDraw(drawType: DrawType) -> bool {
    matches!(drawType, DrawType::imageRect | DrawType::imageMesh)
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadAction {
    clear = 0,
    preserveRenderTarget = 1,
    dontCare = 2,
}

impl LoadAction {
    pub const Clear: Self = Self::clear;
    pub const PreserveRenderTarget: Self = Self::preserveRenderTarget;
    pub const DontCare: Self = Self::dontCare;
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterlockMode {
    rasterOrdering = 0,
    atomics = 1,
    clockwise = 2,
    clockwiseAtomic = 3,
    msaa = 4,
}

impl InterlockMode {
    pub const RasterOrdering: Self = Self::rasterOrdering;
    pub const Atomics: Self = Self::atomics;
    pub const Clockwise: Self = Self::clockwise;
    pub const ClockwiseAtomic: Self = Self::clockwiseAtomic;
    pub const Msaa: Self = Self::msaa;
}

pub const INTERLOCK_MODE_COUNT: usize = 5;
pub const INTERLOCK_MODE_BIT_COUNT: usize = 3;

macro_rules! define_flag_type {
    ($name:ident, $repr:ty, $( $field:ident = $value:expr ),+ $(,)?) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct $name(pub $repr);
        impl $name { $( pub const $field: Self = Self($value); )+ }
        impl BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self { Self(self.0 | rhs.0) }
        }
        impl BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self { Self(self.0 & rhs.0) }
        }
        impl core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0; }
        }
        impl core::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) { self.0 &= rhs.0; }
        }
        impl Not for $name {
            type Output = Self;
            fn not(self) -> Self { Self(!self.0) }
        }
    };
}

define_flag_type!(
    ShaderFeatures,
    u32,
    NONE = 0,
    ENABLE_CLIPPING = 1 << 0,
    ENABLE_CLIP_RECT = 1 << 1,
    ENABLE_ADVANCED_BLEND = 1 << 2,
    ENABLE_FEATHER = 1 << 3,
    ENABLE_EVEN_ODD = 1 << 4,
    ENABLE_NESTED_CLIPPING = 1 << 5,
    ENABLE_HSL_BLEND_MODES = 1 << 6,
    ENABLE_DITHER = 1 << 7,
);

pub const kShaderFeatureCount: usize = 8;
pub const kAllShaderFeatures: ShaderFeatures = ShaderFeatures((1 << kShaderFeatureCount) - 1);
pub const kVertexShaderFeaturesMask: ShaderFeatures = ShaderFeatures(
    ShaderFeatures::ENABLE_CLIPPING.0
        | ShaderFeatures::ENABLE_CLIP_RECT.0
        | ShaderFeatures::ENABLE_ADVANCED_BLEND.0
        | ShaderFeatures::ENABLE_FEATHER.0,
);
pub const kExclusiveAtomicUbershaderFeaturesMask: ShaderFeatures =
    ShaderFeatures(ShaderFeatures::ENABLE_ADVANCED_BLEND.0);

pub const fn ShaderFeaturesMaskFor(interlockMode: InterlockMode) -> ShaderFeatures {
    match interlockMode {
        InterlockMode::rasterOrdering => kAllShaderFeatures,
        InterlockMode::atomics => {
            ShaderFeatures(kAllShaderFeatures.0 & !ShaderFeatures::ENABLE_NESTED_CLIPPING.0)
        }
        InterlockMode::clockwise => {
            ShaderFeatures(kAllShaderFeatures.0 & !ShaderFeatures::ENABLE_EVEN_ODD.0)
        }
        InterlockMode::clockwiseAtomic => ShaderFeatures(
            kAllShaderFeatures.0
                & !ShaderFeatures::ENABLE_EVEN_ODD.0
                & !ShaderFeatures::ENABLE_NESTED_CLIPPING.0,
        ),
        InterlockMode::msaa => ShaderFeatures(
            ShaderFeatures::ENABLE_CLIP_RECT.0
                | ShaderFeatures::ENABLE_ADVANCED_BLEND.0
                | ShaderFeatures::ENABLE_HSL_BLEND_MODES.0
                | ShaderFeatures::ENABLE_DITHER.0,
        ),
    }
}

define_flag_type!(
    ShaderMiscFlags,
    u32,
    none = 0,
    fixedFunctionColorOutput = 1 << 0,
    clockwiseFill = 1 << 1,
    clipUpdateOnly = 1 << 2,
    nestedClipUpdateOnly = 1 << 3,
    borrowedCoveragePass = 1 << 4,
    storeColorClear = 1 << 5,
    loadColorFromDstTexture = 1 << 6,
    swizzleColorBGRAToRGBA = 1 << 7,
    coalescedResolveAndTransfer = 1 << 8,
);

impl ShaderMiscFlags {
    pub const FIXED_FUNCTION_COLOR_OUTPUT: Self = Self::fixedFunctionColorOutput;
    pub const CLOCKWISE_FILL: Self = Self::clockwiseFill;
    pub const STORE_COLOR_CLEAR: Self = Self::storeColorClear;
    pub const SWIZZLE_COLOR_BGRA_TO_RGBA: Self = Self::swizzleColorBGRAToRGBA;
    pub const COALESCED_RESOLVE_AND_TRANSFER: Self = Self::coalescedResolveAndTransfer;

    #[inline]
    pub const fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }

    #[inline]
    pub const fn with(self, flag: Self) -> Self {
        Self(self.0 | flag.0)
    }
}

pub const fn ShaderFeaturesMaskForDraw(
    drawType: DrawType,
    interlockMode: InterlockMode,
) -> ShaderFeatures {
    let mask = match drawType {
        DrawType::imageRect | DrawType::imageMesh | DrawType::featherAtlasBlit
            if interlockMode as i32 != InterlockMode::atomics as i32 =>
        {
            ShaderFeatures(
                ShaderFeatures::ENABLE_CLIPPING.0
                    | ShaderFeatures::ENABLE_CLIP_RECT.0
                    | ShaderFeatures::ENABLE_ADVANCED_BLEND.0
                    | ShaderFeatures::ENABLE_HSL_BLEND_MODES.0
                    | ShaderFeatures::ENABLE_DITHER.0,
            )
        }
        DrawType::midpointFanPatches
        | DrawType::midpointFanCenterAAPatches
        | DrawType::outerCurvePatches
        | DrawType::interiorTriangulation
        | DrawType::msaaStrokes
        | DrawType::msaaMidpointFanBorrowedCoverage
        | DrawType::msaaDynamicMidpointFans
        | DrawType::msaaMidpointFans
        | DrawType::msaaMidpointFanStencilReset
        | DrawType::msaaMidpointFanPathsStencil
        | DrawType::msaaMidpointFanPathsCover
        | DrawType::msaaOuterCubics
        | DrawType::imageRect
        | DrawType::imageMesh
        | DrawType::featherAtlasBlit => kAllShaderFeatures,
        DrawType::clipReset => ShaderFeatures::ENABLE_DITHER,
        DrawType::renderPassInitialize => match interlockMode {
            InterlockMode::atomics => ShaderFeatures(
                ShaderFeatures::ENABLE_CLIPPING.0
                    | ShaderFeatures::ENABLE_ADVANCED_BLEND.0
                    | ShaderFeatures::ENABLE_DITHER.0,
            ),
            InterlockMode::msaa => ShaderFeatures::ENABLE_DITHER,
            InterlockMode::clockwiseAtomic => ShaderFeatures::NONE,
            InterlockMode::rasterOrdering | InterlockMode::clockwise => {
                debug_assert!(interlockMode as i32 == InterlockMode::clockwiseAtomic as i32);
                ShaderFeatures::NONE
            }
        },
        DrawType::renderPassResolve if interlockMode as i32 == InterlockMode::atomics as i32 => {
            kAllShaderFeatures
        }
        DrawType::renderPassResolve => {
            debug_assert!(
                interlockMode as i32 == InterlockMode::rasterOrdering as i32
                    || interlockMode as i32 == InterlockMode::msaa as i32
            );
            ShaderFeatures::ENABLE_DITHER
        }
    };
    ShaderFeatures(mask.0 & ShaderFeaturesMaskFor(interlockMode).0)
}

// Source spelling retained as an overload-like Rust companion.
pub const fn ShaderFeaturesMaskForDrawType(
    drawType: DrawType,
    interlockMode: InterlockMode,
) -> ShaderFeatures {
    ShaderFeaturesMaskForDraw(drawType, interlockMode)
}

pub const fn UbershaderFeaturesMaskFor(
    requestedFeatures: ShaderFeatures,
    drawType: DrawType,
    interlockMode: InterlockMode,
    shaderMiscFlags: ShaderMiscFlags,
    platformFeatures: &PlatformFeatures,
) -> ShaderFeatures {
    let mut outFeatures = ShaderFeaturesMaskForDraw(drawType, interlockMode);
    if interlockMode as u8 == InterlockMode::atomics as u8 {
        outFeatures = ShaderFeatures(
            outFeatures.0 & (requestedFeatures.0 | !kExclusiveAtomicUbershaderFeaturesMask.0),
        );
    }
    debug_assert!((requestedFeatures.0 & outFeatures.0) == requestedFeatures.0);
    if interlockMode as i32 == InterlockMode::msaa as i32 && !platformFeatures.supportsClipPlanes {
        outFeatures = ShaderFeatures(outFeatures.0 & !ShaderFeatures::ENABLE_CLIP_RECT.0);
    }
    if shaderMiscFlags.0
        & (ShaderMiscFlags::borrowedCoveragePass.0 | ShaderMiscFlags::fixedFunctionColorOutput.0)
        != 0
    {
        outFeatures = ShaderFeatures(outFeatures.0 & !ShaderFeatures::ENABLE_ADVANCED_BLEND.0);
    }
    if interlockMode as i32 == InterlockMode::atomics as i32
        && (shaderMiscFlags.0 & ShaderMiscFlags::coalescedResolveAndTransfer.0) != 0
    {
        outFeatures = ShaderFeatures(outFeatures.0 | ShaderFeatures::ENABLE_ADVANCED_BLEND.0);
    }
    outFeatures
}

extern "C" {
    pub fn ShaderUniqueKey(
        drawType: DrawType,
        shaderFeatures: ShaderFeatures,
        interlockMode: InterlockMode,
        shaderMiscFlags: ShaderMiscFlags,
    ) -> u32;
    pub fn GetShaderFeatureGLSLName(feature: ShaderFeatures) -> *const core::ffi::c_char;
}

// void ForEachUbershaderPermutation(
//     InterlockMode,
//     const PlatformFeatures&,
//     const std::function<bool(DrawType, ShaderFeatures, ShaderMiscFlags)>&);

define_flag_type!(
    DrawContents,
    u32,
    none = 0,
    opaquePaint = 1 << 0,
    featheredFill = 1 << 1,
    stroke = 1 << 2,
    clockwiseFill = 1 << 3,
    nonZeroFill = 1 << 4,
    evenOddFill = 1 << 5,
    activeClip = 1 << 6,
    advancedBlend = 1 << 7,
    clipUpdate = 1 << 8,
);

pub const DRAW_CONTENTS_FOR_MSAA_PIPELINE_STATE: DrawContents = DrawContents(
    DrawContents::activeClip.0
        | DrawContents::clipUpdate.0
        | DrawContents::clockwiseFill.0
        | DrawContents::evenOddFill.0
        | DrawContents::opaquePaint.0,
);

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StencilType {
    disabled = 0,
    activeStencilClip = 1,
    borrowedCoverage = 2,
    forwardClippedByBackward = 3,
    backwardTriangleCleanup = 4,
    stencilNestedOrEvenOdd = 5,
    evenOddDrawAndReset = 6,
    nestedClipReset = 7,
    clipReset = 8,
}

pub const STENCIL_TYPE_BIT_COUNT: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StencilInfo {
    pub stencilType: StencilType,
    pub drawContentsMask: DrawContents,
    pub areDrawContentsValid: bool,
}

// StencilInfo get_stencil_info(InterlockMode, DrawType, DrawContents);

pub const kNestedClipUpdateMask: DrawContents =
    DrawContents(DrawContents::activeClip.0 | DrawContents::clipUpdate.0);

define_flag_type!(
    BarrierFlags,
    u8,
    none = 0,
    plsAtomic = 1 << 0,
    plsAtomicPreResolve = 1 << 1,
    msaaPostInit = 1 << 2,
    clockwiseBorrowedCoverage = 1 << 3,
    dstBlend = 1 << 4,
    preManualResolve = 1 << 5,
    drawBatchBreak = 1 << 6,
);

impl BarrierFlags {
    pub const PLS_ATOMIC: u8 = Self::plsAtomic.0;
    pub const PLS_ATOMIC_PRE_RESOLVE: u8 = Self::plsAtomicPreResolve.0;

    #[inline]
    pub const fn needs_atomic(self) -> bool {
        self.0 & (Self::plsAtomic.0 | Self::plsAtomicPreResolve.0) != 0
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawBatch {
    pub drawType: DrawType,
    pub shaderMiscFlags: ShaderMiscFlags,
    pub drawContents: DrawContents,
    pub elementCount: u32,
    pub baseElement: u32,
    pub indexCountPerInstance: u32,
    pub baseIndex: u32,
    pub firstBlendMode: BlendMode,
    pub barriers: BarrierFlags,
    pub scissorRect: Option<AABBu16>,
    pub shaderFeatures: ShaderFeatures,
    pub imageTexture: Option<NonNull<Texture>>,
    pub imageSampler: ImageSampler,
    pub vertexBuffer: Option<NonNull<RenderBuffer>>,
    pub uvBuffer: Option<NonNull<RenderBuffer>>,
    pub indexBuffer: Option<NonNull<RenderBuffer>>,
    pub dstReadList: Option<NonNull<Draw>>,
    pub nextDstBlendBarrier: Option<NonNull<DrawBatch>>,
    pub next: Option<NonNull<DrawBatch>>,
}

impl DrawBatch {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        drawType_: DrawType,
        shaderMiscFlags_: ShaderMiscFlags,
        drawContents_: DrawContents,
        elementCount_: u32,
        baseElement_: u32,
        blendMode_: BlendMode,
        imageSampler_: ImageSampler,
        barrierFlags_: BarrierFlags,
    ) -> Self {
        Self {
            drawType: drawType_,
            shaderMiscFlags: shaderMiscFlags_,
            drawContents: drawContents_,
            elementCount: elementCount_,
            baseElement: baseElement_,
            indexCountPerInstance: 0,
            baseIndex: 0,
            firstBlendMode: blendMode_,
            barriers: barrierFlags_,
            scissorRect: None,
            shaderFeatures: ShaderFeatures::NONE,
            imageTexture: None,
            imageSampler: imageSampler_,
            vertexBuffer: None,
            uvBuffer: None,
            indexBuffer: None,
            dstReadList: None,
            nextDstBlendBarrier: None,
            next: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TwoTexelRamp {
    pub color0: ColorInt,
    pub color1: ColorInt,
}

#[cfg(feature = "with-rive-tools")]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SynthesizedFailureType {
    none = 0,
    ubershaderLoad = 1,
    shaderCompilation = 2,
    pipelineCreation = 3,
}

#[cfg(feature = "with-rive-tools")]
impl Default for SynthesizedFailureType {
    fn default() -> Self {
        Self::none
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtlasDrawBatch {
    pub scissor: AABBu16,
    pub patchCount: u32,
    pub basePatch: u32,
}

#[repr(C)]
pub struct FlushDescriptor {
    pub renderTarget: Option<NonNull<RenderTarget>>,
    pub combinedShaderFeatures: ShaderFeatures,
    pub interlockMode: InterlockMode,
    pub msaaSampleCount: i32,
    pub colorLoadAction: LoadAction,
    pub colorClearValue: ColorInt,
    pub coverageClearValue: u32,
    pub depthClearValue: f32,
    pub stencilClearValue: u8,
    pub renderTargetUpdateBounds: IAABB,
    pub virtualTileWidth: u32,
    pub virtualTileHeight: u32,
    pub manuallyResolved: bool,
    pub fixedFunctionColorOutput: bool,
    pub featherAtlasTextureWidth: u16,
    pub featherAtlasTextureHeight: u16,
    pub featherAtlasContentWidth: u16,
    pub featherAtlasContentHeight: u16,
    pub coverageBufferPrefix: u32,
    pub needsCoverageBufferClear: bool,
    pub flushUniformDataOffsetInBytes: usize,
    pub pathCount: u32,
    pub firstPath: usize,
    pub firstPaint: usize,
    pub firstPaintAux: usize,
    pub contourCount: u32,
    pub firstContour: usize,
    pub gradSpanCount: u32,
    pub firstGradSpan: usize,
    pub tessVertexSpanCount: u32,
    pub firstTessVertexSpan: usize,
    pub gradDataHeight: u32,
    pub tessDataHeight: u32,
    pub clockwiseFillOverride: bool,
    pub hasTriangleVertices: bool,
    pub wireframe: bool,
    pub ditherMode: DitherMode,
    #[cfg(feature = "with-rive-tools")]
    pub synthesizedFailureType: SynthesizedFailureType,
    pub externalCommandBuffer: Option<NonNull<c_void>>,
    pub featherAtlasFillBatches: Option<NonNull<AtlasDrawBatch>>,
    pub featherAtlasFillBatchCount: usize,
    pub featherAtlasStrokeBatches: Option<NonNull<AtlasDrawBatch>>,
    pub featherAtlasStrokeBatchCount: usize,
    pub drawList: Option<NonNull<BlockAllocatedLinkedList<DrawBatch>>>,
    pub firstDstBlendBarrier: Option<NonNull<DrawBatch>>,
    pub unresolvedBarriers: BarrierFlags,
}

// float find_transformed_area(const AABB& bounds, const Mat2D&);
// uint32_t ConvertBlendModeToPLSBlendMode(BlendMode riveMode);

#[inline(always)]
pub const fn SwizzleRiveColorToRGBA(riveColor: ColorInt) -> u32 {
    (riveColor & 0xff00ff00) | (riveColor.rotate_left(16) & 0x00ff00ff)
}

// uint32_t SwizzleRiveColorToRGBAPremul(ColorInt riveColor);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InverseViewports {
    pub m_vals: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlushUniforms {
    pub m_inverseViewports: InverseViewports,
    pub m_renderTargetWidth: u32,
    pub m_renderTargetHeight: u32,
    pub m_colorClearValue: u32,
    pub m_coverageClearValue: u32,
    pub m_renderTargetUpdateBounds: IAABB,
    pub m_featherAtlasTextureInverseSize: Vec2D,
    pub m_featherAtlasContentInverseViewport: Vec2D,
    pub m_coverageBufferPrefix: u32,
    pub m_epsilonForPseudoMemoryBarrier: f32,
    pub m_pathIDGranularity: u32,
    pub m_vertexDiscardValue: f32,
    pub m_mipMapLODBias: f32,
    pub m_maxPathId: u32,
    pub m_ditherScale: f32,
    pub m_ditherBias: f32,
    pub m_ditherConversionToRGB10: f32,
    pub m_wireframeEnabled: u32,
    pub m_padTo256Bytes: [u8; 256 - 104],
}

impl FlushUniforms {
    // FlushUniforms(const FlushDescriptor&, const PlatformFeatures&);
    // The constructor definition remains owned by renderer/src/gpu.cpp.
    // void operator=(const FlushUniforms& rhs) is a byte copy excluding
    // m_padTo256Bytes; operator!= is the corresponding byte comparison.
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageBufferStructure {
    uint32x4 = 0,
    uint32x2 = 1,
    float32x4 = 2,
}

pub const fn StorageBufferElementSizeInBytes(value: StorageBufferStructure) -> u32 {
    match value {
        StorageBufferStructure::uint32x4 => core::mem::size_of::<u32>() as u32 * 4,
        StorageBufferStructure::uint32x2 => core::mem::size_of::<u32>() as u32 * 2,
        StorageBufferStructure::float32x4 => core::mem::size_of::<f32>() as u32 * 4,
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AtlasTransform {
    pub scaleFactor: f32,
    pub translateX: f32,
    pub translateY: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CoverageBufferRange {
    pub offset: u32,
    pub pitch: u32,
    pub offsetX: f32,
    pub offsetY: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathData {
    pub m_matrix: [f32; 6],
    pub m_strokeRadius: f32,
    pub m_featherRadius: f32,
    pub m_zIndex: u32,
    pub m_featherAtlasTransform: AtlasTransform,
    pub m_coverageBufferRange: CoverageBufferRange,
}

impl PathData {
    pub const kBufferStructure: StorageBufferStructure = StorageBufferStructure::uint32x4;
    // void set(const Mat2D&, float, float, uint32_t, const AtlasTransform&, const CoverageBufferRange&);
}
pub const kPathBufferAlignmentInElements: usize = 256 / core::mem::size_of::<PathData>();

#[repr(C)]
#[derive(Clone, Copy)]
pub union PaintDataValue {
    pub m_color: u32,
    pub m_gradTextureY: f32,
    pub m_opacity: f32,
    pub m_shiftedClipReplacementID: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PaintData {
    pub m_params: u32,
    pub value: PaintDataValue,
}

impl PaintData {
    pub const kBufferStructure: StorageBufferStructure = StorageBufferStructure::uint32x2;
    // void set(DrawContents, PaintType, SimplePaintValue, GradTextureLayout,
    //          uint32_t clipID, bool hasClipRect, BlendMode);
}
pub const kPaintBufferAlignmentInElements: usize = 256 / core::mem::size_of::<PaintData>();

#[repr(C)]
#[derive(Clone, Copy)]
pub union PaintAuxDataValue {
    pub m_gradTextureHorizontalSpan: [f32; 2],
    pub m_imageTextureLOD: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PaintAuxData {
    pub m_matrix: [f32; 6],
    pub value: PaintAuxDataValue,
    pub m_clipRectInverseMatrix: [f32; 6],
    pub m_inverseFwidth: Vec2D,
}

impl PaintAuxData {
    pub const kBufferStructure: StorageBufferStructure = StorageBufferStructure::float32x4;
    // void set(const Mat2D&, PaintType, SimplePaintValue, const Gradient*,
    //          const Texture*, const ClipRectInverseMatrix*, const RenderTarget*,
    //          const gpu::PlatformFeatures&);
}
pub const kPaintAuxBufferAlignmentInElements: usize = 256 / core::mem::size_of::<PaintAuxData>();

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContourData {
    pub m_midpoint: Vec2D,
    pub m_pathID: u32,
    pub m_vertexIndex0: u32,
}

impl ContourData {
    pub const kBufferStructure: StorageBufferStructure = StorageBufferStructure::uint32x4;
    pub const fn new(midpoint: Vec2D, pathID: u32, vertexIndex0: u32) -> Self {
        Self {
            m_midpoint: midpoint,
            m_pathID: pathID,
            m_vertexIndex0: vertexIndex0,
        }
    }
}
pub const kContourBufferAlignmentInElements: usize = 256 / core::mem::size_of::<ContourData>();

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriangleVertex {
    pub m_point: Vec2D,
    pub m_weight_pathID: i32,
}

impl TriangleVertex {
    pub const fn new(point: Vec2D, weight: i16, pathID: u16) -> Self {
        Self {
            m_point: point,
            m_weight_pathID: ((weight as i32) << 16) | pathID as i32,
        }
    }
    #[cfg(test)]
    pub const fn testing_point(&self) -> Vec2D {
        self.m_point
    }
    #[cfg(test)]
    pub const fn testing_weight_pathID(&self) -> i32 {
        self.m_weight_pathID
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageDrawInstance {
    pub m_viewMatrix: [f32; 4],
    pub m_clipRectInverseMatrix: [f32; 4],
    pub m_translate: [f32; 2],
    pub m_clipRectInverseTranslate: [f32; 2],
    pub m_opacity: f32,
    pub m_clipID: u32,
    pub m_blendMode: u32,
    pub m_zIndex: u32,
}

impl ImageDrawInstance {
    pub const FirstAttribIdx: usize = 2;
    pub const LastAttribIdx: usize = 5;
    pub const AttribCount: usize = Self::LastAttribIdx + 1 - Self::FirstAttribIdx;
    // ImageDrawInstance(const Mat2D&, float, const ClipRectInverseMatrix*,
    //                   uint32_t, BlendMode, uint32_t);
}

pub const kMaxStorageBuffers: usize = 4;

// std::tuple<uint32_t, uint32_t> StorageTextureSize(size_t,
//                                                   StorageBufferStructure);
// size_t StorageTextureBufferSize(size_t, StorageBufferStructure);

define_flag_type!(
    WindingFaces,
    u32,
    none = 0,
    negative = 1 << 0,
    positive = 1 << 1,
    all = (1 << 0) | (1 << 1),
);

#[repr(C)]
pub struct WriteOnlyMappedMemory<T> {
    pub m_mappedMemory: *mut T,
    pub m_nextMappedItem: *mut T,
    pub m_mappingEnd: *const T,
    _marker: PhantomData<T>,
}

impl<T> Default for WriteOnlyMappedMemory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> WriteOnlyMappedMemory<T> {
    pub const fn new() -> Self {
        Self {
            m_mappedMemory: core::ptr::null_mut(),
            m_nextMappedItem: core::ptr::null_mut(),
            m_mappingEnd: core::ptr::null(),
            _marker: PhantomData,
        }
    }
    /// # Safety
    /// `ptr..ptr.add(elementCount)` must be one live, writable mapping owned
    /// by the caller for the entire lifetime of this cursor.
    pub unsafe fn from_raw_parts(ptr: *mut T, elementCount: usize) -> Self {
        let mut out = Self::new();
        unsafe { out.reset(ptr, elementCount) };
        out
    }
    /// # Safety
    /// Same mapping/provenance contract as `from_raw_parts`.
    pub unsafe fn reset(&mut self, ptr: *mut T, elementCount: usize) {
        self.m_mappedMemory = ptr;
        self.m_nextMappedItem = ptr;
        self.m_mappingEnd = ptr.wrapping_add(elementCount);
    }
    pub fn clear(&mut self) {
        // SAFETY: the null/zero pair is the authored empty mapping.
        unsafe { self.reset(core::ptr::null_mut(), 0) };
    }
    pub unsafe fn mapElements(
        &mut self,
        impl_ptr: *mut RenderContextImpl,
        mapFn: unsafe fn(*mut RenderContextImpl, usize) -> *mut c_void,
        elementCount: usize,
    ) -> bool {
        debug_assert!(self.m_mappedMemory.is_null());
        let ptr = unsafe { mapFn(impl_ptr, elementCount * core::mem::size_of::<T>()) };
        if ptr.is_null() {
            return false;
        }
        unsafe { self.reset(ptr.cast::<T>(), elementCount) };
        true
    }
    /// # Safety
    /// `map` must return a writable mapping for exactly
    /// `elementCount * size_of::<T>()` bytes, with the same lifetime contract
    /// as `mapElements`.
    pub unsafe fn mapElementsWith<F>(&mut self, elementCount: usize, map: F) -> bool
    where
        F: FnOnce(usize) -> *mut c_void,
    {
        debug_assert!(self.m_mappedMemory.is_null());
        let ptr = map(elementCount * core::mem::size_of::<T>());
        if ptr.is_null() {
            return false;
        }
        unsafe { self.reset(ptr.cast::<T>(), elementCount) };
        true
    }
    pub unsafe fn unmapElements(
        &mut self,
        impl_ptr: *mut RenderContextImpl,
        unmapFn: unsafe fn(*mut RenderContextImpl, usize),
        elementCount: usize,
    ) {
        if !self.m_mappedMemory.is_null() {
            debug_assert!(
                (self.m_mappingEnd as usize).wrapping_sub(self.m_mappedMemory as usize)
                    / core::mem::size_of::<T>()
                    == elementCount
            );
            unsafe { unmapFn(impl_ptr, elementCount * core::mem::size_of::<T>()) };
            self.clear();
        }
    }
    /// # Safety
    /// `unmap` must release the exact live mapping held by this cursor.
    pub unsafe fn unmapElementsWith<F>(&mut self, elementCount: usize, unmap: F)
    where
        F: FnOnce(usize),
    {
        if !self.m_mappedMemory.is_null() {
            debug_assert_eq!(
                (self.m_mappingEnd as usize).wrapping_sub(self.m_mappedMemory as usize)
                    / core::mem::size_of::<T>(),
                elementCount,
            );
            unmap(elementCount * core::mem::size_of::<T>());
            self.clear();
        }
    }
    pub const fn is_mapped(&self) -> bool {
        !self.m_mappedMemory.is_null()
    }
    pub fn bytesWritten(&self) -> usize {
        (self.m_nextMappedItem as usize).wrapping_sub(self.m_mappedMemory as usize)
    }
    pub fn elementsWritten(&self) -> usize {
        self.bytesWritten() / core::mem::size_of::<T>()
    }
    pub fn hasRoomFor(&self, itemCount: usize) -> bool {
        (self.m_nextMappedItem as usize).wrapping_add(itemCount * core::mem::size_of::<T>())
            <= self.m_mappingEnd as usize
    }
    unsafe fn push_ptr(&mut self) -> *mut T {
        debug_assert!(self.hasRoomFor(1));
        let ret = self.m_nextMappedItem;
        self.m_nextMappedItem = self.m_nextMappedItem.wrapping_add(1);
        ret
    }
    unsafe fn push_ptr_count(&mut self, count: usize) -> *mut T {
        debug_assert!(self.hasRoomFor(count));
        let ret = self.m_nextMappedItem;
        self.m_nextMappedItem = self.m_nextMappedItem.wrapping_add(count);
        ret
    }
    // Rust cannot spell C++'s variadic Args&& placement-new directly.  The
    // caller-supplied value/closure retains the source write-only ordering.
    pub unsafe fn emplace_back(&mut self, value: T) {
        let dst = unsafe { self.push_ptr() };
        unsafe { core::ptr::write(dst, value) };
    }
    pub unsafe fn set_back<F: FnOnce(&mut T)>(&mut self, set: F) {
        let dst = unsafe { self.push_ptr() };
        unsafe { set(&mut *dst) };
    }
    pub unsafe fn push_back_n(&mut self, values: *const T, count: usize) {
        let dst = unsafe { self.push_ptr_count(count) };
        if !values.is_null() {
            unsafe { core::ptr::copy_nonoverlapping(values, dst, count) };
        }
    }
    pub unsafe fn skip_back(&mut self) {
        let _ = unsafe { self.push_ptr() };
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriState {
    no = 0,
    yes = 1,
    unknown = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StencilOp {
    keep = 0,
    replace = 1,
    zero = 2,
    decrClamp = 3,
    incrWrap = 4,
    decrWrap = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StencilCompareOp {
    less = 0,
    equal = 1,
    lessOrEqual = 2,
    notEqual = 3,
    always = 4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StencilFaceOps {
    pub stencilFailOp: StencilOp,
    pub depthFailOp: StencilOp,
    pub depthStencilPassOp: StencilOp,
    pub compareOp: StencilCompareOp,
}

impl Default for StencilFaceOps {
    fn default() -> Self {
        Self {
            stencilFailOp: StencilOp::keep,
            depthFailOp: StencilOp::keep,
            depthStencilPassOp: StencilOp::keep,
            compareOp: StencilCompareOp::always,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CullFace {
    none = 0,
    clockwise = 1,
    counterclockwise = 2,
}

pub const CULL_FACE_BIT_COUNT: u32 = 2;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendEquation {
    none = 0,
    srcOver = 3,
    plus = 4,
    min = 5,
    max = 6,
    screen = 14,
    overlay = 15,
    darken = 16,
    lighten = 17,
    colorDodge = 18,
    colorBurn = 19,
    hardLight = 20,
    softLight = 21,
    difference = 22,
    exclusion = 23,
    multiply = 24,
    hue = 25,
    saturation = 26,
    color = 27,
    luminosity = 28,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DepthState {
    pub depthTestEnabled: bool,
    pub depthWriteEnabled: bool,
}

// DepthState get_depth_state(InterlockMode, DrawType, DrawContents);
// CullFace get_cull_face(DrawType);
// bool get_color_write_enable(DrawType, InterlockMode, ShaderMiscFlags,
//                             bool fixedFunctionColorOutput, DrawContents);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PipelineState {
    pub depthTestEnabled: bool,
    pub depthWriteEnabled: bool,
    pub stencilTestEnabled: bool,
    pub stencilCompareMask: u8,
    pub stencilWriteMask: u8,
    pub stencilReference: u8,
    pub stencilFrontOps: StencilFaceOps,
    pub stencilBackOps: StencilFaceOps,
    pub stencilDoubleSided: bool,
    pub cullFace: CullFace,
    pub blendEquation: BlendEquation,
    pub colorWriteEnabled: bool,
}

impl PipelineState {
    pub const fn new() -> Self {
        Self {
            depthTestEnabled: false,
            depthWriteEnabled: true,
            stencilTestEnabled: false,
            stencilCompareMask: 0xff,
            stencilWriteMask: 0xff,
            stencilReference: 0,
            stencilFrontOps: StencilFaceOps {
                stencilFailOp: StencilOp::keep,
                depthFailOp: StencilOp::keep,
                depthStencilPassOp: StencilOp::keep,
                compareOp: StencilCompareOp::always,
            },
            stencilBackOps: StencilFaceOps {
                stencilFailOp: StencilOp::keep,
                depthFailOp: StencilOp::keep,
                depthStencilPassOp: StencilOp::keep,
                compareOp: StencilCompareOp::always,
            },
            stencilDoubleSided: false,
            cullFace: CullFace::none,
            blendEquation: BlendEquation::none,
            colorWriteEnabled: true,
        }
    }
}

pub const GL_DEFAULT_PIPELINE_STATE: PipelineState = PipelineState::new();

pub const fn make_flat_pipeline_state(cull: CullFace, blend: BlendEquation) -> PipelineState {
    let mut state = PipelineState::new();
    state.depthTestEnabled = false;
    state.depthWriteEnabled = false;
    state.stencilTestEnabled = false;
    state.stencilWriteMask = 0;
    state.cullFace = cull;
    state.blendEquation = blend;
    state.colorWriteEnabled = true;
    state
}

pub const COLOR_ONLY_PIPELINE_STATE: PipelineState =
    make_flat_pipeline_state(CullFace::none, BlendEquation::none);
pub const FEATHER_ATLAS_FILL_PIPELINE_STATE: PipelineState =
    make_flat_pipeline_state(CullFace::none, BlendEquation::plus);
pub const FEATHER_ATLAS_STROKE_PIPELINE_STATE: PipelineState =
    make_flat_pipeline_state(CullFace::counterclockwise, BlendEquation::max);

pub type float4 = [f32; 4];
pub type uint16x4 = [u16; 4];

// float4 cast_f16_to_f32(uint16x4 x);
// uint16x4 cast_f32_to_f16(float4);

pub const GAUSSIAN_TABLE_SIZE: u32 = 512;
extern "C" {
    pub static g_gaussianIntegralTableF16: [u16; GAUSSIAN_TABLE_SIZE as usize];
    pub static g_inverseGaussianIntegralTableF16: [u16; GAUSSIAN_TABLE_SIZE as usize];
}

extern "C" {
    pub fn generate_gausian_integral_table(out: *mut f32);
    pub fn generate_inverse_gausian_integral_table(out: *mut f32);
}
