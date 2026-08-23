/*
 * Copyright 2022 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/render_context.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// The complete pinned source is retained below in declaration order. The
// active Rust declarations after it preserve the source-shaped owner graph,
// defaults, field order, configuration branches, and inline side effects.

// /*
//  * Copyright 2022 Rive
//  */
//
// #pragma once
//
// #include "rive/math/vec2d.hpp"
// #include "rive/renderer/gpu.hpp"
// #include "rive/renderer/rive_render_factory.hpp"
// #ifdef RIVE_CANVAS
// #include "rive/renderer/render_canvas.hpp"
// #include "rive/renderer/ore/ore_context.hpp"
// #endif
// #include "rive/renderer/render_target.hpp"
// #include "rive/renderer/shader_compilation_mode.hpp"
// #include "rive/renderer/sk_rectanizer_skyline.hpp"
// #include "rive/renderer/trivial_block_allocator.hpp"
// #include "rive/shapes/paint/color.hpp"
// #include <array>
// #include <unordered_map>
//
// class PushRetrofittedTrianglesGMDraw;
// class RenderContextTest;
//
// namespace rive
// {
// class RawPath;
// class RiveRenderPaint;
// class RiveRenderPath;
// } // namespace rive
//
// namespace rive::gpu
// {
// class GradientLibrary;
// class IntersectionBoard;
// class ImageMeshDraw;
// class ImageRectDraw;
// class ClipReset;
// class Draw;
// class Gradient;
// class RenderContextImpl;
// class PathDraw;
//
// // Various types of ordered dithering we can add to reduce banding
// // https://en.wikipedia.org/wiki/Ordered_dithering
// // https://blog.demofox.org/2022/01/01/interleaved-gradient-noise-a-different-kind-of-low-discrepancy-sequence/
// enum class DitherMode
// {
//     none,
//     interleavedGradientNoise,
// };
//
// // Used as a key for complex gradients.
// class GradientContentKey
// {
// public:
//     inline GradientContentKey(rcp<const Gradient> gradient);
//     inline GradientContentKey(GradientContentKey&& other);
//     bool operator==(const GradientContentKey&) const;
//     const Gradient* gradient() const { return m_gradient.get(); }
//
// private:
//     rcp<const Gradient> m_gradient;
// };
//
// // Hashes all stops and all colors in a complex gradient.
// class DeepHashGradient
// {
// public:
//     size_t operator()(const GradientContentKey&) const;
// };
//
// // Even though Draw is block-allocated, we still need to call releaseRefs() on
// // each individual instance before releasing the block. This smart pointer
// // guarantees we always call releaseRefs() (implementation in pls_draw.hpp).
// struct DrawReleaseRefs
// {
//     void operator()(Draw* draw);
// };
// using DrawUniquePtr = std::unique_ptr<Draw, DrawReleaseRefs>;
//
// // Top-level, API agnostic rendering context for RiveRenderer. This class
// // manages all the GPU buffers, context state, and other resources required for
// // Rive's pixel local storage path rendering algorithm.
// class RenderContext : public RiveRenderFactory
// {
// public:
//     RenderContext(std::unique_ptr<RenderContextImpl>);
//     ~RenderContext();
//
//     RenderContextImpl* impl() { return m_impl.get(); }
//     template <typename T> T* static_impl_cast()
//     {
//         return static_cast<T*>(m_impl.get());
//     }
//
//     const gpu::PlatformFeatures& platformFeatures() const;
//
//     // Options for controlling how and where a frame is rendered.
//     struct FrameDescriptor
//     {
//         uint32_t renderTargetWidth = 0;
//         uint32_t renderTargetHeight = 0;
//         LoadAction loadAction = LoadAction::clear;
//         ColorInt clearColor = 0;
//         // If nonzero, the number of MSAA samples to use.
//         // Setting this to a nonzero value forces msaa mode.
//         uint32_t msaaSampleCount = 0;
//         // Use atomic mode (preferred) or msaa instead of rasterOrdering.
//         bool disableRasterOrdering = false;
//         DitherMode ditherMode = DitherMode::interleavedGradientNoise;
//
//         // If nonzero, frames are split up into virtual tiles of this size.
//         //
//         // As of now, each tile gets drawn in a separate render pass. The
//         // purpose of these virtual tiles, for now, is to break the frame up
//         // into smaller chunks so that Rive can be pre-empted by other rendering
//         // processes. This is only supported on Vulkan/non-msaa.
//         //
//         // TODO: We could also explore a different type of virtual tiling that
//         // reduces barriers in atomic mode, but that is not how this feature
//         // works currently.
//         uint32_t virtualTileWidth = 0;
//         uint32_t virtualTileHeight = 0;
//
//         // Testing flags.
//         bool wireframe = false;
//         bool fillsDisabled = false;
//         bool strokesDisabled = false;
//         // Override all paths' fill rules (winding or even/odd) to emulate
//         // clockwiseAtomic mode.
//         bool clockwiseFillOverride = false;
// #ifdef WITH_RIVE_TOOLS
//         // Synthesize compilation failures to make sure the device handles them
//         // gracefully. (e.g., by falling back on an uber shader or at least not
//         // crashing.) Valid compilations may fail in the real world if the
//         // device is pressed for resources or in a bad state.
//         gpu::SynthesizedFailureType synthesizedFailureType =
//             gpu::SynthesizedFailureType::none;
// #endif
//     };
//
//     // Called at the beginning of a frame and establishes where and how it will
//     // be rendered.
//     //
//     // All rendering related calls must be made between beginFrame() and
//     // flush().
//     void beginFrame(const FrameDescriptor&);
//
//     const FrameDescriptor& frameDescriptor() const
//     {
//         assert(m_didBeginFrame);
//         return m_frameDescriptor;
//     }
//
//     // True if bounds is empty or outside [0, 0, renderTargetWidth,
//     // renderTargetHeight].
//     bool isOutsideCurrentFrame(const IAABB& pixelBounds);
//
//     // True if the current frame supports draws with clipRects
//     // (clipRectInverseMatrix != null). If false, all clipping must be done with
//     // clipPaths.
//     bool frameSupportsClipRects() const;
//
//     // If the frame doesn't support image paints, the client must draw images
//     // with pushImageRect(). If it DOES support image paints, the client CANNOT
//     // use pushImageRect(); it should draw images as rectangular paths with an
//     // image paint.
//     bool frameSupportsImagePaintForPaths() const;
//
//     const gpu::InterlockMode frameInterlockMode() const
//     {
//         return m_frameInterlockMode;
//     }
//
//     // Generates a unique clip ID that is guaranteed to not exist in the current
//     // clip buffer, and assigns a contentBounds to it.
//     // `tightenedBounds` is the contentBounds, clipped against the render
//     // target area as well as any parent clips.
//     //
//     // Returns 0 if a unique ID could not be generated, at which point the
//     // caller must issue a logical flush and try again.
//
//     uint32_t generateClipID(IAABB contentBounds,
//                             uint32_t parentClipID,
//                             AABBu16 tightenedBounds);
//
//     // Screen-space bounding box of the region inside the given clip.
//     const IAABB& getClipContentBounds(uint32_t clipID) const
//     {
//         assert(m_didBeginFrame);
//         assert(!m_logicalFlushes.empty());
//         return m_logicalFlushes.back()->getClipInfo(clipID).contentBounds;
//     }
//
//     // Screen-space bounding box of the area that covers all of the reads of the
//     // given clip, clipped to the area of its parent (if there is one) and the
//     // screen edges.
//     const AABBu16& getTightenedClipBounds(uint32_t clipID) const
//     {
//         assert(m_didBeginFrame);
//         assert(!m_logicalFlushes.empty());
//         return m_logicalFlushes.back()->getClipInfo(clipID).tightenedBounds;
//     }
//
//     // Get/set a "clip content ID" that uniquely identifies the current contents
//     // of the clip buffer. This ID is reset to 0 on every logical flush.
//     void setClipContentID(uint32_t clipID)
//     {
//         assert(m_didBeginFrame);
//         m_clipContentID = clipID;
//     }
//
//     uint32_t getClipContentID() const
//     {
//         assert(m_didBeginFrame);
//         return m_clipContentID;
//     }
//
//     // Appends a list of high-level Draws to the current frame.
//     // Returns false if the draws don't fit within the current resource
//     // constraints, at which point the caller must issue a logical flush and try
//     // again.
//     [[nodiscard]] bool pushDraws(DrawUniquePtr draws[], size_t drawCount);
//
//     // Records a "logical" flush, in that it builds up commands to break up the
//     // render pass and re-render the resource textures, but it won't submit any
//     // command buffers or rotate/synchronize the buffer rings.
//     void logicalFlush();
//
//     // GPU resources required to execute the GPU commands for a frame.
//     struct FlushResources
//     {
//         RenderTarget* renderTarget = nullptr;
//
//         // Command buffer that rendering commands will be added to.
//         //  - VkCommandBuffer on Vulkan.
//         //  - id<MTLCommandBuffer> on Metal.
//         //  - WGPUCommandEncoder on WebGPU.
//         //  - Unused otherwise.
//         void* externalCommandBuffer = nullptr;
//
//         // Resource lifetime counters. Resources used during the upcoming flush
//         // will belong to 'currentFrameNumber'. Resources last used on or before
//         // 'safeFrameNumber' are safe to be released or recycled.
//         uint64_t currentFrameNumber = 0;
//         uint64_t safeFrameNumber = 0;
//     };
//
//     // Submits all GPU commands that have been built up since beginFrame().
//     void flush(const FlushResources&);
//
//     // Called when the client will stop rendering. Releases all CPU and GPU
//     // resources associated with this render context.
//     void releaseResources();
//
//     // Returns the context's TrivialBlockAllocator, which is automatically reset
//     // at the end of every frame. (Memory in this allocator is preserved between
//     // logical flushes.)
//     TrivialBlockAllocator& perFrameAllocator()
//     {
//         assert(m_didBeginFrame);
//         return m_perFrameAllocator;
//     }
//
//     // Allocators for intermediate path processing buffers.
//     TrivialArrayAllocator<uint8_t>& numChopsAllocator()
//     {
//         return m_numChopsAllocator;
//     }
//     TrivialArrayAllocator<Vec2D>& chopVerticesAllocator()
//     {
//         return m_chopVerticesAllocator;
//     }
//     TrivialArrayAllocator<std::array<Vec2D, 2>>& tangentPairsAllocator()
//     {
//         return m_tangentPairsAllocator;
//     }
//     TrivialArrayAllocator<uint32_t, alignof(float4)>&
//     polarSegmentCountsAllocator()
//     {
//         return m_polarSegmentCountsAllocator;
//     }
//     TrivialArrayAllocator<uint32_t, alignof(float4)>&
//     parametricSegmentCountsAllocator()
//     {
//         return m_parametricSegmentCountsAllocator;
//     }
//
//     // Allocates a trivially destructible object that will be automatically
//     // dropped at the end of the current frame.
//     template <typename T, typename... Args> T* make(Args&&... args)
//     {
//         assert(m_didBeginFrame);
//         return m_perFrameAllocator.make<T>(std::forward<Args>(args)...);
//     }
//
//     // Backend-specific RiveRenderFactory implementation.
//     rcp<RenderBuffer> makeRenderBuffer(RenderBufferType,
//                                        RenderBufferFlags,
//                                        size_t) override;
//     rcp<RenderImage> decodeImage(Span<const uint8_t>) override;
//
// #ifdef RIVE_CANVAS
//     // Creates a RenderCanvas: a GPU texture usable as both a render target
//     // (for rendering into) and a render image (for compositing into draws).
//     rcp<RenderCanvas> makeRenderCanvas(uint32_t width, uint32_t height);
//     rive::ore::Context* ore() override;
//     rive::ore::Context* getOreContext() { return ore(); }
// #endif
//
// private:
//     friend class Draw;
//     friend class PathDraw;
//     friend class ImageRectDraw;
//     friend class ImageMeshDraw;
//     friend class ClipReset;
//     friend class ::PushRetrofittedTrianglesGMDraw; // For testing.
//     friend class ::RenderContextTest;              // For testing.
//
//     // Resets the CPU-side STL containers so they don't have unbounded growth.
//     void resetContainers();
//
//     // Throttled width/height of the atlas texture. If drawing to a render
//     // target larger than this, we may create a larger atlas anyway.
//     uint32_t featherAtlasMaxSize() const
//     {
//         constexpr static uint32_t FeatherAtlasMaxSize = 4096;
//         return std::min(platformFeatures().maxTextureSize, FeatherAtlasMaxSize);
//     }
//
//     // Defines the exact size of each of our GPU resources. Computed during
//     // flush(), based on LogicalFlush::ResourceCounters and
//     // LogicalFlush::LayoutCounters.
//     struct ResourceAllocationCounts
//     {
//         constexpr static int NUM_ELEMENTS = 19;
//         using VecType = simd::gvec<size_t, NUM_ELEMENTS>;
//
//         RIVE_ALWAYS_INLINE VecType toVec() const
//         {
//             static_assert(sizeof(*this) == sizeof(size_t) * NUM_ELEMENTS);
//             static_assert(sizeof(VecType) >= sizeof(*this));
//             VecType vec;
//             RIVE_INLINE_MEMCPY(&vec, this, sizeof(*this));
//             return vec;
//         }
//
//         static RIVE_ALWAYS_INLINE ResourceAllocationCounts
//         FromVec(const VecType& vec)
//         {
//             ResourceAllocationCounts allocs;
//             static_assert(sizeof(allocs) == sizeof(size_t) * NUM_ELEMENTS);
//             static_assert(sizeof(VecType) >= sizeof(allocs));
//             RIVE_INLINE_MEMCPY(&allocs, &vec, sizeof(allocs));
//             return allocs;
//         }
//
//         size_t flushUniformBufferCount = 0;
//         size_t pathBufferCount = 0;
//         size_t paintBufferCount = 0;
//         size_t paintAuxBufferCount = 0;
//         size_t contourBufferCount = 0;
//         size_t gradSpanBufferCount = 0;
//         size_t tessSpanBufferCount = 0;
//         size_t triangleVertexBufferCount = 0;
//         size_t imageDrawInstanceBufferCount = 0;
//         size_t gradTextureHeight = 0;
//         size_t tessTextureHeight = 0;
//         size_t featherAtlasTextureWidth = 0;
//         size_t featherAtlasTextureHeight = 0;
//         size_t plsTransientBackingWidth = 0;
//         size_t plsTransientBackingHeight = 0;
//         size_t plsTransientBackingPlaneCount = 0;
//         size_t plsAtomicCoverageBackingWidth = 0;  // atomic mode only.
//         size_t plsAtomicCoverageBackingHeight = 0; // atomic mode only.
//         size_t coverageBufferLength = 0;           // clockwiseAtomic mode only.
//     };
//
//     // Reallocates GPU resources and updates m_currentResourceAllocations.
//     // If forceRealloc is true, every GPU resource is allocated, even if the
//     // size would not change.
//     void setResourceSizes(ResourceAllocationCounts, bool forceRealloc = false);
//
//     [[nodiscard]] bool mapResourceBuffers(const ResourceAllocationCounts&);
//     void unmapResourceBuffers(const ResourceAllocationCounts&);
//
//     // Returns the next coverage buffer prefix to use in a logical flush.
//     // Sets needsCoverageBufferClear if the coverage buffer must be cleared in
//     // order to support the returned coverage buffer prefix.
//     // (clockwiseAtomic mode only.)
//     uint32_t incrementCoverageBufferPrefix(bool* needsCoverageBufferClear);
//     const std::unique_ptr<RenderContextImpl> m_impl;
//     const size_t m_maxPathID;
//
// #ifdef RIVE_CANVAS
//     std::unique_ptr<rive::ore::Context> m_oreContext = nullptr;
// #endif
//
//     ResourceAllocationCounts m_currentResourceAllocations;
//     ResourceAllocationCounts m_maxRecentResourceRequirements;
//     double m_lastResourceTrimTimeInSeconds;
//
//     // Per-frame state.
//     FrameDescriptor m_frameDescriptor;
//     gpu::InterlockMode m_frameInterlockMode;
//     gpu::ShaderFeatures m_frameShaderFeaturesMask;
//     RIVE_DEBUG_CODE(bool m_didBeginFrame = false;)
//
//     // Clipping state.
//     uint32_t m_clipContentID = 0;
//
//     // Monotonically increasing prefix that gets appended to the most
//     // significant "32 - CLOCKWISE_COVERAGE_BIT_COUNT" bits of coverage buffer
//     // values.
//     //
//     // Increasing this prefix implicitly clears the entire coverage buffer to
//     // zero.
//     //
//     // (clockwiseAtomic mode only.)
//     uint32_t m_coverageBufferPrefix = 0;
//
//     struct DrawSortEntry
//     {
//         int64_t sortKey;
//         int16_t drawIndex;
//     };
//
//     // A simple class to allow std::unordered_map to use the scissor AABB as a
//     // key.
//     struct ScissorAABBHasher
//     {
//         size_t operator()(AABBu16 aabb) const
//         {
//             // Hash the AABB as a single 64-bit int value.
//             return std::hash<uint64_t>{}(math::bit_cast<uint64_t>(aabb));
//         }
//     };
//
//     // Used by LogicalFlushes for re-ordering high level draws.
//     std::vector<DrawSortEntry> m_indirectDrawList;
//     std::unique_ptr<IntersectionBoard> m_intersectionBoard;
//     std::unordered_map<AABBu16, int16_t, ScissorAABBHasher> m_scissorIDLookup;
//     int16_t m_prevScissorID = 0;
//
//     WriteOnlyMappedMemory<gpu::FlushUniforms> m_flushUniformData;
//     WriteOnlyMappedMemory<gpu::PathData> m_pathData;
//     WriteOnlyMappedMemory<gpu::PaintData> m_paintData;
//     WriteOnlyMappedMemory<gpu::PaintAuxData> m_paintAuxData;
//     WriteOnlyMappedMemory<gpu::ContourData> m_contourData;
//     WriteOnlyMappedMemory<gpu::GradientSpan> m_gradSpanData;
//     WriteOnlyMappedMemory<gpu::TessVertexSpan> m_tessSpanData;
//     WriteOnlyMappedMemory<gpu::TriangleVertex> m_triangleVertexData;
//     WriteOnlyMappedMemory<gpu::ImageDrawInstance> m_imageDrawInstanceData;
//
//     // Simple allocator for trivially-destructible data that needs to persist
//     // until the current frame has completed. All memory in this allocator is
//     // dropped at the end of the every frame.
//     constexpr static size_t kPerFlushAllocatorInitialBlockSize =
//         1024 * 1024; // 1 MiB.
//     TrivialBlockAllocator m_perFrameAllocator{
//         kPerFlushAllocatorInitialBlockSize};
//
//     // Allocators for intermediate path processing buffers.
//     constexpr static size_t kIntermediateDataInitialStrokes =
//         8192; // * 84 == 688 KiB.
//     constexpr static size_t kIntermediateDataInitialFillCurves =
//         32768; // * 4 == 128 KiB.
//     TrivialArrayAllocator<uint8_t> m_numChopsAllocator{
//         kIntermediateDataInitialStrokes * 4}; // 4 byte per stroke curve.
//     TrivialArrayAllocator<Vec2D> m_chopVerticesAllocator{
//         kIntermediateDataInitialStrokes * 4}; // 32 bytes per stroke curve.
//     TrivialArrayAllocator<std::array<Vec2D, 2>> m_tangentPairsAllocator{
//         kIntermediateDataInitialStrokes * 2}; // 32 bytes per stroke curve.
//     TrivialArrayAllocator<uint32_t, alignof(float4)>
//         m_polarSegmentCountsAllocator{kIntermediateDataInitialStrokes *
//                                       4}; // 16 bytes per stroke curve.
//     TrivialArrayAllocator<uint32_t, alignof(float4)>
//         m_parametricSegmentCountsAllocator{
//             kIntermediateDataInitialFillCurves}; // 4 bytes per fill curve.
//
//     class TessellationWriter;
//
//     // Manages a list of high-level Draws and their required resources.
//     //
//     // Since textures have hard size limits, we can't always fit an entire frame
//     // into one flush. It's rare for us to require more than one flush in a
//     // single frame, but for the times that we do, this flush logic is
//     // encapsulated in a nested class that can be built up into a list and
//     // executed the end of a frame.
//     class LogicalFlush
//     {
//     public:
//         LogicalFlush(RenderContext* parent);
//
//         // Rewinds this flush object back to an empty state without shrinking
//         // any internal allocations held by CPU-side STL containers.
//         void rewind();
//
//         // Resets the CPU-side STL containers so they don't have unbounded
//         // growth.
//         void resetContainers();
//
//         const FrameDescriptor& frameDescriptor() const
//         {
//             return m_ctx->frameDescriptor();
//         }
//         gpu::InterlockMode interlockMode() const
//         {
//             return m_ctx->frameInterlockMode();
//         }
//         const gpu::PlatformFeatures& platformFeatures() const
//         {
//             return m_ctx->platformFeatures();
//         }
//
//         // Access this flush's gpu::FlushDescriptor (which is not valid until
//         // layoutResources()). NOTE: Some fields in the FlushDescriptor
//         // (tessVertexSpanCount, hasTriangleVertices, drawList, and
//         // combinedShaderFeatures) do not become valid until after
//         // writeResources().
//         const gpu::FlushDescriptor& desc()
//         {
//             assert(m_hasDoneLayout);
//             return m_flushDesc;
//         }
//
//         // Generates a unique clip ID that is guaranteed to not exist in the
//         // current clip buffer.
//         //
//         // Returns 0 if a unique ID could not be generated, at which point the
//         // caller must issue a logical flush and try again.
//         uint32_t generateClipID(IAABB contentBounds,
//                                 uint32_t parentClipID,
//                                 AABBu16 tightenedBounds);
//
//         struct ClipInfo
//         {
//             ClipInfo(IAABB contentBounds_,
//                      uint32_t parentClipID_,
//                      AABBu16 tightenedBounds_) :
//                 parentClipID(parentClipID_),
//                 contentBounds(contentBounds_),
//                 tightenedBounds(tightenedBounds_)
//             {}
//
//             const uint32_t parentClipID = 0;
//
//             // Screen-space bounding box of the region inside the clip
//             const IAABB contentBounds;
//
//             // The minimally necessary write bounds, which will ultimately take
//             // into account the content bounds, the screen dimensions, any
//             // parent clips, and where all of the reads of this clip are.
//             AABBu16 tightenedBounds;
//
//             // Union of screen-space bounding boxes from all draws that read the
//             // clip.
//             //
//             // (Initialized with a maximally negative rectangle whose union with
//             // any other rectangle will be equal to that same rectangle.)
//             AABBu16 readBounds = AABBu16::makeMaximallyNegative();
//         };
//
//         const ClipInfo& getClipInfo(uint32_t clipID)
//         {
//             return getWritableClipInfo(clipID);
//         }
//
//         // Appends a list of high-level Draws to the flush.
//         // Returns false if the draws don't fit within the current resource
//         // constraints, at which point the context must append a new logical
//         // flush and try again.
//         [[nodiscard]] bool pushDraws(DrawUniquePtr draws[], size_t drawCount);
//
//         // Running counts of data records required by Draws that need to be
//         // allocated in the render context's various GPU buffers.
//         struct ResourceCounters
//         {
//             constexpr static int NUM_ELEMENTS = 7;
//             using VecType = simd::gvec<size_t, NUM_ELEMENTS>;
//
//             VecType toVec() const
//             {
//                 static_assert(sizeof(*this) == sizeof(size_t) * NUM_ELEMENTS);
//                 static_assert(sizeof(VecType) >= sizeof(*this));
//                 VecType vec;
//                 RIVE_INLINE_MEMCPY(&vec, this, sizeof(*this));
//                 return vec;
//             }
//
//             ResourceCounters(const VecType& vec)
//             {
//                 static_assert(sizeof(*this) == sizeof(size_t) * NUM_ELEMENTS);
//                 static_assert(sizeof(VecType) >= sizeof(*this));
//                 RIVE_INLINE_MEMCPY(this, &vec, sizeof(*this));
//             }
//
//             ResourceCounters() = default;
//
//             size_t midpointFanTessVertexCount = 0;
//             size_t outerCubicTessVertexCount = 0;
//             size_t pathCount = 0;
//             size_t contourCount = 0;
//             // lines, curves, lone joins, emulated caps, etc.
//             size_t maxTessellatedSegmentCount = 0;
//             size_t maxTriangleVertexCount = 0;
//             size_t imageDrawCount = 0; // imageRect or imageMesh.
//         };
//
//         // Additional counters for layout state that don't need to be tracked by
//         // individual draws.
//         struct LayoutCounters
//         {
//             uint32_t pathPaddingCount = 0;
//             uint32_t paintPaddingCount = 0;
//             uint32_t paintAuxPaddingCount = 0;
//             uint32_t contourPaddingCount = 0;
//             uint32_t gradSpanCount = 0;
//             uint32_t gradSpanPaddingCount = 0;
//             uint32_t maxGradTextureHeight = 0;
//             uint32_t maxTessTextureHeight = 0;
//             uint32_t maxFeatherAtlasWidth = 0;
//             uint32_t maxFeatherAtlasHeight = 0;
//             uint32_t maxPLSTransientBackingPlaneCount = 0;
//             size_t maxCoverageBufferLength = 0;
//         };
//
//         // Allocates a horizontal span of texels in the gradient texture and
//         // schedules either a texture upload or a draw that fills it with the
//         // given gradient's color ramp.
//         //
//         // Fills out a ColorRampLocation record that tells the shader how to
//         // access the gradient.
//         //
//         // Returns false if the gradient texture is out of space, at which point
//         // the caller must issue a logical flush and try again.
//         [[nodiscard]] bool allocateGradient(const Gradient*,
//                                             gpu::ColorRampLocation*);
//
//         // Allocates a rectangular region in the atlas for this draw to use, and
//         // registers a future callback to
//         // PathDraw::pushFeatherAtlasTessellation() where it will render its
//         // coverage data to this same region in the atlas.
//         //
//         // Attempts to leave a border of "desiredPadding" pixels surrounding the
//         // rectangular region, but the allocation may not be padded if the path
//         // is up against an edge.
//         bool allocateFeatherAtlasDraw(PathDraw*,
//                                       uint16_t drawWidth,
//                                       uint16_t drawHeight,
//                                       uint16_t desiredPadding,
//                                       uint16_t* x,
//                                       uint16_t* y,
//                                       AABBu16* paddedRegion);
//
//         // Reserves a range within the coverage buffer for a path to use in
//         // clockwiseAtomic mode.
//         //
//         // "length" is the length in pixels of this allocation and must be a
//         // multiple of BUFFER_IMAGE_TILE_SIZE^2, in order to support internal
//         // tiling.
//         //
//         // Returns the offset of the allocated range within the coverage buffer,
//         // or -1 if there was not room.
//         size_t allocateCoverageBufferRange(size_t length);
//
//         // Carves out space for this specific flush within the total frame's
//         // resource buffers and lays out the flush-specific resource textures.
//         // Updates the total frame running conters based on layout.
//         void layoutResources(const FlushResources&,
//                              size_t logicalFlushIdx,
//                              ResourceCounters* runningFrameResourceCounts,
//                              LayoutCounters* runningFrameLayoutCounts);
//
//         // Called after all flushes in a frame have done their layout and the
//         // render context has allocated and mapped its resource buffers. Writes
//         // the GPU data for this flush to the context's actively mapped resource
//         // buffers.
//         void writeResources();
//
//         // Reserves a span of "count" vertices from the "midpointFanPatches"
//         // section of the tessellation texture.
//         //
//         // This method must be called for a total count of precisely
//         // "m_resourceCounts.midpointFanTessVertexCount" vertices.
//         //
//         // The caller must fill these vertices in with TessellationWriter.
//         //
//         // Returns the index of the first vertex in the newly allocated span.
//         uint32_t allocateMidpointFanTessVertices(uint32_t count);
//
//         // Reserves a span of "count" vertices from the "outerCurvePatches"
//         // section of the tessellation texture.
//         //
//         // This method must be called for a total count of precisely
//         // "m_resourceCounts.outerCubicTessVertexCount" vertices.
//         //
//         // The caller must fill these vertices in with TessellationWriter.
//         //
//         // Returns the index of the first vertex in the newly allocated span.
//         uint32_t allocateOuterCubicTessVertices(uint32_t count);
//
//         // Allocates and initializes a record on the GPU for the given path.
//         //
//         // Returns a unique 16-bit "pathID" handle for this specific record.
//         //
//         // This method does not add the path to the draw list. The caller must
//         // define that draw specifically with a separate call to
//         // pushMidpointFanDraw() or pushOuterCubicsDraw().
//         [[nodiscard]] uint32_t pushPath(const PathDraw* draw);
//
//         // Pushes a contour record to the GPU that references the given path.
//         //
//         // "vertexIndex0" is the index within the tessellation where the first
//         // vertex of the contour resides. Shaders need this when the contour is
//         // closed.
//         //
//         // Returns a unique 16-bit "contourID" handle for this specific record.
//         // This ID may be or-ed with '*_CONTOUR_FLAG' bits from constants.glsl.
//         [[nodiscard]] uint32_t pushContour(uint32_t pathID,
//                                            Vec2D midpoint,
//                                            bool isStroke,
//                                            bool closed,
//                                            uint32_t vertexIndex0);
//
//         // Writes padding vertices to the tessellation texture, with an invalid
//         // contour ID that is guaranteed to not be the same ID as any neighbors.
//         void pushPaddingVertices(uint32_t count, uint32_t tessLocation);
//
//         // Schedules barriers that will be issued immediately before the next
//         // draw.
//         void pushBarriers(BarrierFlags);
//
//         // Pushes a "midpointFanPatches" draw to the list. Path, contour, and
//         // cubic data are pushed separately.
//         //
//         // Also adds the PathDraw to a dstRead list if one is
//         // required, and if this is the path's first subpass.
//         gpu::DrawBatch& pushMidpointFanDraw(
//             const PathDraw*,
//             gpu::DrawType,
//             uint32_t tessVertexCount,
//             uint32_t tessLocation,
//             gpu::ShaderMiscFlags = gpu::ShaderMiscFlags::none);
//
//         // Pushes an "outerCurvePatches" draw to the list. Path, contour, and
//         // cubic data are pushed separately.
//         //
//         // Also adds the PathDraw to a dstRead list if one is
//         // required, and if this is the path's first subpass.
//         gpu::DrawBatch& pushOuterCubicsDraw(
//             const PathDraw*,
//             gpu::DrawType,
//             uint32_t tessVertexCount,
//             uint32_t tessLocation,
//             gpu::ShaderMiscFlags = gpu::ShaderMiscFlags::none);
//
//         // Writes out triangle verties for the desired WindingFaces and pushes
//         // an "interiorTriangulation" draw to the list.
//         // Returns the number of vertices actually written.
//         gpu::DrawBatch* pushInteriorTriangulationDraw(
//             const PathDraw*,
//             uint32_t pathID,
//             gpu::WindingFaces,
//             gpu::ShaderMiscFlags RIVE_DEBUG_CODE(, size_t* vertexCounter));
//
//         // Pushes a screen-space rectangle to the draw list, whose pixel
//         // coverage is determined by the feather atlas region associated with
//         // the given pathID.
//         gpu::DrawBatch& pushFeatherAtlasBlit(PathDraw*, uint32_t pathID);
//
//         // Pushes an "imageRect" to the draw list.
//         // This should only be used when we in atomic mode. Otherwise, images
//         // should be drawn as rectangular paths with an image paint.
//         gpu::DrawBatch& pushImageRectDraw(ImageRectDraw*);
//
//         // Pushes an "imageMesh" draw to the list.
//         gpu::DrawBatch& pushImageMeshDraw(ImageMeshDraw*);
//
//         // Pushes a "clipReset" draw to the list.
//         gpu::DrawBatch& pushClipResetDraw(ClipReset*);
//
//     private:
//         friend class TessellationWriter;
//
//         ClipInfo& getWritableClipInfo(uint32_t clipID);
//
//         // Either appends a new drawBatch to m_drawList or merges into
//         // m_drawList.tail(). Updates the batch's ShaderFeatures according to
//         // the passed parameters.
//         DrawBatch& pushPathDraw(const PathDraw*,
//                                 DrawType,
//                                 gpu::ShaderMiscFlags,
//                                 uint32_t vertexCount,
//                                 uint32_t baseVertex);
//         DrawBatch& pushDraw(const Draw*,
//                             DrawType,
//                             gpu::ShaderMiscFlags,
//                             gpu::PaintType,
//                             uint32_t elementCount,
//                             uint32_t baseElement);
//
//         // Do a bottom-up pass on the draws in the list, computing bounds for
//         // each clip update to be the intersection of the clip update itself and
//         // any reads that use it.
//         void tightenClipBounds();
//
//         // Adds a batch to the list of draws that use a dstBarrier.
//         void addBatchToDstBarrierList(DrawBatch* batch)
//         {
//             assert(m_dstBlendBarrierListTail != nullptr);
//             assert(*m_dstBlendBarrierListTail == nullptr);
//             assert(batch->nextDstBlendBarrier == nullptr);
//             assert(enums::is_flag_set(batch->barriers, BarrierFlags::dstBlend));
//             *m_dstBlendBarrierListTail = batch;
//             m_dstBlendBarrierListTail = &batch->nextDstBlendBarrier;
//         }
//
//         // Instance pointer to the outer parent class.
//         RenderContext* const m_ctx;
//
//         // Running counts of GPU data records that need to be allocated for
//         // draws.
//         ResourceCounters m_resourceCounts;
//
//         // Running count of combined prepasses and subpasses from every draw in
//         // m_draws.
//         int m_drawPassCount;
//
//         // Simple gradients have one stop at t=0 and one stop at t=1. They're
//         // implemented with 2 texels.
//         std::unordered_map<uint64_t, uint32_t>
//             m_simpleGradients; // [color0, color1] -> texelsIdx.
//         std::vector<gpu::TwoTexelRamp> m_pendingSimpleGradDraws;
//
//         // Complex gradients have stop(s) between t=0 and t=1. In theory they
//         // should be scaled to a ramp where every stop lands exactly on a pixel
//         // center, but for now we just always scale them to the entire gradient
//         // texture width.
//         std::unordered_map<GradientContentKey, uint16_t, DeepHashGradient>
//             m_complexGradients; // [colors[0..n], stops[0..n]] -> rowIdx
//         std::vector<const Gradient*> m_pendingComplexGradDraws;
//
//         // Simple and complex gradients both get uploaded to the GPU as sets of
//         // "GradientSpan" instances.
//         size_t m_pendingGradSpanCount;
//
//         std::vector<ClipInfo> m_clips;
//
//         // High-level draw list. These get built into a low-level list of
//         // gpu::DrawBatch objects during writeResources().
//         std::vector<DrawUniquePtr> m_draws;
//         IAABB m_combinedDrawBounds;
//         gpu::DrawContents m_combinedDrawContents;
//
//         // State computed during layout.
//         uint32_t m_pathPaddingCount;
//         uint32_t m_paintPaddingCount;
//         uint32_t m_paintAuxPaddingCount;
//         uint32_t m_contourPaddingCount;
//         uint32_t m_gradSpanPaddingCount;
//         uint32_t m_midpointFanTessEndLocation;
//         uint32_t m_outerCubicTessEndLocation;
//         uint32_t m_outerCubicTessVertexIdx;
//         uint32_t m_midpointFanTessVertexIdx;
//         gpu::GradTextureLayout m_gradTextureLayout;
//         gpu::ShaderMiscFlags m_baselineShaderMiscFlags;
//
//         gpu::FlushDescriptor m_flushDesc;
//
//         BlockAllocatedLinkedList<DrawBatch> m_drawList;
//         const DrawBatch* m_firstDstBlendBarrier;
//         // Final "next" pointer in the list of DrawBatches that have dstBlend
//         // barriers.
//         const DrawBatch** m_dstBlendBarrierListTail;
//
//         gpu::ShaderFeatures m_combinedShaderFeatures;
//
//         // Most recent path and contour state.
//         uint32_t m_currentPathID;
//         uint32_t m_currentContourID;
//
//         // Atlas for offscreen feathering.
//         std::unique_ptr<rive::RectanizerSkyline> m_featherAtlasRectanizer;
//         uint32_t m_featherAtlasMaxX = 0;
//         uint32_t m_featherAtlasMaxY = 0;
//         std::vector<PathDraw*> m_pendingFeatherAtlasDraws;
//
//         // Total coverage allocated via allocateCoverageBufferRange().
//         // (clockwiseAtomic mode only.)
//         uint32_t m_coverageBufferLength = 0;
//
//         // Barriers that must execute before pushing the next DrawBatch
//         // (pushPathDraw()/pushDraw()). If any barriers are pending, this also
//         // prevents DrawBatches from being combined with the existing drawList.
//         BarrierFlags m_pendingBarriers;
//
//         // Stateful Z index of the current draw being pushed. Used by msaa mode
//         // to avoid double hits and to reverse-sort opaque paths front to back.
//         uint32_t m_currentZIndex;
//
//         RIVE_DEBUG_CODE(bool m_hasDoneLayout = false;)
//     };
//
//     std::vector<std::unique_ptr<LogicalFlush>> m_logicalFlushes;
//
//     // Writes out TessVertexSpans that are used to tessellate the vertices
//     // in a path.
//     class TessellationWriter
//     {
//     public:
//         // forwardTessLocation & mirroredTessLocation are allocated by
//         // allocate*TessVertices().
//         //
//         // forwardTessLocation starts at the beginning of the vertex span
//         // and advances forward.
//         //
//         // mirroredTessLocation starts at the end of the vertex span and
//         // advances backward.
//         //
//         // If the ContourDirections are double sided, forwardTessVertexCount
//         // & mirroredTessVertexCount must both be equal, and
//         // forwardTessLocation & mirroredTessLocation must both be valid.
//         // Otherwise, one span or the other may be empty.
//         TessellationWriter(LogicalFlush* flush,
//                            uint32_t pathID,
//                            gpu::ContourDirections,
//                            uint32_t forwardTessVertexCount,
//                            uint32_t forwardTessLocation,
//                            uint32_t mirroredTessVertexCount = 0,
//                            uint32_t mirroredTessLocation = 0);
//
//         ~TessellationWriter();
//
//         // Returns the index of the next vertex to be written.
//         //
//         // In the case of double-sided tessellations the next vertex gets
//         // tessellated twice, and either index will be identical. So we just
//         // return the next *forward* tessellation index when it's double sided.
//         uint32_t nextVertexIndex()
//         {
//             return m_contourDirections != gpu::ContourDirections::reverse
//                        ? m_pathTessLocation
//                        : m_pathMirroredTessLocation - 1;
//         }
//
//         // Wrapper around LogicalFlush::pushContour(), with an additional
//         // padding option.
//         //
//         // The first curve of the contour will be pre-padded with
//         // 'paddingVertexCount' tessellation vertices, colocated at T=0. The
//         // caller must use this argument to align the end of the contour on
//         // a boundary of the patch size. (See gpu::PaddingToAlignUp().)
//         [[nodiscard]] uint32_t pushContour(Vec2D midpoint,
//                                            bool isStroke,
//                                            bool closed,
//                                            uint32_t paddingVertexCount);
//
//         // Wites out (potentially wrapped) TessVertexSpan(s) that tessellate
//         // a cubic curve & join at the current tessellation location(s).
//         // Advances the tessellation location(s).
//         //
//         // The bottom 16 bits of contourIDWithFlags must match the most
//         // recent contourID returned by pushContour(), but it may also have
//         // extra '*_CONTOUR_FLAG' bits from constants.glsl
//         //
//         // An instance consists of a cubic curve with
//         // "parametricSegmentCount + polarSegmentCount" segments, followed
//         // by a join with "joinSegmentCount" segments, for a grand total of
//         // "parametricSegmentCount + polarSegmentCount + joinSegmentCount -
//         // 1" vertices.
//         //
//         // If a cubic has already been pushed to the current contour, pts[0]
//         // must be equal to the former cubic's pts[3].
//         //
//         // "joinTangent" is the ending tangent of the join that follows the
//         // cubic.
//         void pushCubic(const Vec2D pts[4],
//                        gpu::ContourDirections,
//                        Vec2D joinTangent,
//                        uint32_t parametricSegmentCount,
//                        uint32_t polarSegmentCount,
//                        uint32_t joinSegmentCount,
//                        uint32_t contourIDWithFlags);
//
//         // pushCubic() impl for forward tessellations.
//         RIVE_ALWAYS_INLINE void pushTessellationSpans(
//             const Vec2D pts[4],
//             Vec2D joinTangent,
//             uint32_t totalVertexCount,
//             uint32_t parametricSegmentCount,
//             uint32_t polarSegmentCount,
//             uint32_t joinSegmentCount,
//             uint32_t contourIDWithFlags);
//
//         // pushCubic() impl for mirrored tessellations.
//         RIVE_ALWAYS_INLINE void pushMirroredTessellationSpans(
//             const Vec2D pts[4],
//             Vec2D joinTangent,
//             uint32_t totalVertexCount,
//             uint32_t parametricSegmentCount,
//             uint32_t polarSegmentCount,
//             uint32_t joinSegmentCount,
//             uint32_t contourIDWithFlags);
//
//         // Functionally equivalent to "pushMirroredTessellationSpans();
//         // pushTessellationSpans();", but packs each forward and mirrored
//         // pair into a single gpu::TessVertexSpan.
//         RIVE_ALWAYS_INLINE void pushDoubleSidedTessellationSpans(
//             const Vec2D pts[4],
//             Vec2D joinTangent,
//             uint32_t totalVertexCount,
//             uint32_t parametricSegmentCount,
//             uint32_t polarSegmentCount,
//             uint32_t joinSegmentCount,
//             uint32_t contourIDWithFlags);
//
//     private:
//         LogicalFlush* const m_flush;
//         WriteOnlyMappedMemory<gpu::TessVertexSpan>& m_tessSpanData;
//         const uint32_t m_pathID;
//         const gpu::ContourDirections m_contourDirections;
//         uint32_t m_pathTessLocation;
//         uint32_t m_pathMirroredTessLocation;
//         // Padding to add to the next curve.
//         uint32_t m_nextCubicPaddingVertexCount = 0;
//         RIVE_DEBUG_CODE(uint32_t m_expectedPathTessEndLocation;)
//         RIVE_DEBUG_CODE(uint32_t m_expectedPathMirroredTessEndLocation;)
//     };
// };
// } // namespace rive::gpu

// Source-shaped Rust value mappings.  The neighboring mechanical owners remain
// the authority for shared GPU records; these declarations intentionally retain
// the complete header's owner graph and field order.

use core::ffi::c_void;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::marker::PhantomPinned;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::ptr::NonNull;
use std::collections::HashMap;

pub use crate::mechanical_port::source::include::rive::factory_hpp::OreContext;
use crate::mechanical_port::source::include::rive::factory_hpp::{
    Factory, FactoryAccess, FactoryContract,
};
use crate::mechanical_port::source::include::rive::refcnt_hpp::{rcp, RefCnt, RefCntTarget};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
use crate::mechanical_port::source::renderer::include::rive::renderer::{
    render_canvas_hpp::RenderCanvas,
    render_target_hpp::RenderTarget,
    rive_render_factory_hpp::{
        RiveRenderFactory, RiveRenderFactoryAccess, RiveRenderFactoryContract,
    },
};

pub type ColorInt = u32;
pub type float4 = [f32; 4];
pub type Vec2D = [f32; 2];
pub type AABB = [f32; 4];
pub use gpu::IAABB;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AABBu16 {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}

impl AABBu16 {
    pub const fn makeMaximallyNegative() -> Self {
        Self {
            left: u16::MAX,
            top: u16::MAX,
            right: 0,
            bottom: 0,
        }
    }
}

pub use gpu::LoadAction;
// The concrete Gradient owner is declared by the pinned renderer/src/gradient.hpp
// translation.  This re-export preserves the source render-context include's
// forward-declared `gpu::Gradient` spelling for existing consumers without
// creating a second intrusive owner.
pub use crate::mechanical_port::source::renderer::src::gradient_hpp::Gradient;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DitherMode {
    none = 0,
    interleavedGradientNoise = 1,
}

pub type DrawReleaseRefsFn = unsafe fn(*mut Draw);
pub type DrawCountSubpassesFn = unsafe fn(*mut Draw, &gpu::PlatformFeatures);
pub type DrawAllocateResourcesFn = unsafe fn(*mut Draw, *mut LogicalFlush) -> bool;
pub type DrawPushFn = unsafe fn(*mut Draw, *mut LogicalFlush, i32) -> *mut gpu::DrawBatch;

unsafe fn default_release_refs(_: *mut Draw) {}
unsafe fn default_count_subpasses(draw: *mut Draw, _: &gpu::PlatformFeatures) {
    debug_assert!((*draw).prepass_count == 0 && (*draw).subpass_count == 1);
}
unsafe fn default_allocate_resources(_: *mut Draw, _: *mut LogicalFlush) -> bool {
    true
}
unsafe fn default_push(_: *mut Draw, _: *mut LogicalFlush, _: i32) -> *mut gpu::DrawBatch {
    core::ptr::null_mut()
}

#[repr(C)]
pub struct Draw {
    // Explicit virtual slot for Draw::releaseRefs(). Concrete draw owners set
    // this to their most-derived override at construction.
    pub(crate) release_refs: DrawReleaseRefsFn,
    pub(crate) count_subpasses: DrawCountSubpassesFn,
    pub(crate) allocate_resources: DrawAllocateResourcesFn,
    pub(crate) push_to_render_context: DrawPushFn,
    pub(crate) image_texture: *mut gpu::Texture,
    pub(crate) image_sampler: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler,
    pub(crate) pixel_bounds: IAABB,
    pub(crate) matrix: nuxie_render_api::Mat2D,
    pub(crate) blend_mode: nuxie_render_api::BlendMode,
    pub(crate) draw_type: DrawObjectType,
    pub(crate) clipped_pixel_bounds: IAABB,
    pub(crate) clipping_pixel_bounds: Option<IAABB>,
    pub(crate) clip_id: u32,
    pub(crate) clip_rect_inverse_matrix: *const gpu::ClipRectInverseMatrix,
    pub(crate) scissor_rect: Option<AABBu16>,
    pub(crate) draw_contents: gpu::DrawContents,
    pub(crate) resource_counts: ResourceCounters,
    pub(crate) prepass_count: i32,
    pub(crate) subpass_count: i32,
    pub(crate) simple_paint_value: gpu::SimplePaintValue,
    pub(crate) next_dst_read: *const Draw,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawObjectType {
    path,
    imageRect,
    imageMesh,
    stencilClipReset,
}

impl Draw {
    pub fn new() -> Self {
        Self {
            release_refs: default_release_refs,
            count_subpasses: default_count_subpasses,
            allocate_resources: default_allocate_resources,
            push_to_render_context: default_push,
            image_texture: core::ptr::null_mut(),
            image_sampler: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler::LinearClamp(),
            pixel_bounds: IAABB::default(),
            matrix: nuxie_render_api::Mat2D::IDENTITY,
            blend_mode: nuxie_render_api::BlendMode::SrcOver,
            draw_type: DrawObjectType::path,
            clipped_pixel_bounds: IAABB::default(),
            clipping_pixel_bounds: None,
            clip_id: 0,
            clip_rect_inverse_matrix: core::ptr::null(),
            scissor_rect: None,
            draw_contents: gpu::DrawContents::none,
            resource_counts: ResourceCounters::default(),
            prepass_count: 0,
            subpass_count: 1,
            simple_paint_value: gpu::SimplePaintValue { color: 0 },
            next_dst_read: core::ptr::null(),
        }
    }

    /// # Safety
    /// `self` must be the live complete draw allocation installed with this
    /// dispatch entry.
    pub unsafe fn releaseRefs(&mut self) {
        unsafe { (self.release_refs)(self) }
    }
    pub fn pixelBounds(&self) -> &IAABB {
        &self.pixel_bounds
    }
    pub fn clippedPixelBounds(&self) -> &IAABB {
        &self.clipped_pixel_bounds
    }
    pub fn clippingPixelBounds(&self) -> Option<IAABB> {
        self.clipping_pixel_bounds
    }
    pub fn clipRectInverseMatrix(&self) -> *const gpu::ClipRectInverseMatrix {
        self.clip_rect_inverse_matrix
    }
    pub fn resourceCounts(&self) -> &ResourceCounters {
        &self.resource_counts
    }
    pub fn prepassCount(&self) -> i32 {
        self.prepass_count
    }
    pub fn subpassCount(&self) -> i32 {
        self.subpass_count
    }
    pub fn drawContents(&self) -> gpu::DrawContents {
        self.draw_contents
    }
    pub fn isOpaque(&self) -> bool {
        (self.draw_contents.0 & gpu::DrawContents::opaquePaint.0) != 0
    }
    pub fn clipID(&self) -> u32 {
        self.clip_id
    }
    pub fn hasClipRect(&self) -> bool {
        !self.clip_rect_inverse_matrix.is_null()
    }
    pub fn hasActiveClip(&self) -> bool {
        (self.draw_contents.0 & gpu::DrawContents::activeClip.0) != 0
    }
    pub fn hasAdvancedBlend(&self) -> bool {
        (self.draw_contents.0 & gpu::DrawContents::advancedBlend.0) != 0
    }
    pub fn isClipUpdate(&self) -> bool {
        (self.draw_contents.0 & gpu::DrawContents::clipUpdate.0) != 0
    }
    pub fn blendMode(&self) -> nuxie_render_api::BlendMode {
        self.blend_mode
    }
    pub fn imageTexture(&self) -> *mut gpu::Texture {
        self.image_texture
    }
    pub fn imageSampler(
        &self,
    ) -> crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler
    {
        self.image_sampler
    }
    pub fn matrix(&self) -> &nuxie_render_api::Mat2D {
        &self.matrix
    }
    pub fn r#type(&self) -> DrawObjectType {
        self.draw_type
    }
    pub fn simplePaintValue(&self) -> gpu::SimplePaintValue {
        self.simple_paint_value
    }
    pub fn scissorRect(&self) -> Option<AABBu16> {
        self.scissor_rect
    }
    pub fn setScissorRect(&mut self, rect: AABBu16) {
        self.scissor_rect = Some(rect);
    }
    pub fn setClipID(&mut self, clip_id: u32) {
        self.clip_id = clip_id;
        // Clip updates write `clip_id`; they do not read it as an active clip.
        if (self.draw_contents.0 & gpu::DrawContents::clipUpdate.0) == 0 {
            if clip_id == 0 {
                self.draw_contents &= !gpu::DrawContents::activeClip;
            } else {
                self.draw_contents |= gpu::DrawContents::activeClip;
            }
        }
    }
    pub fn setClipRect(
        &mut self,
        inverse_matrix: *const gpu::ClipRectInverseMatrix,
        clipping_pixel_bounds: IAABB,
    ) {
        self.clip_rect_inverse_matrix = inverse_matrix;
        self.clipping_pixel_bounds = Some(clipping_pixel_bounds);
        self.clipped_pixel_bounds =
            intersect_iaabb(self.clipped_pixel_bounds, clipping_pixel_bounds);
    }
    pub fn nextDstRead(&self) -> *const Draw {
        self.next_dst_read
    }
    pub unsafe fn addToDstReadList(&mut self, head: *const Draw) -> *const Draw {
        debug_assert!(self.next_dst_read.is_null());
        self.next_dst_read = head;
        self
    }
    pub unsafe fn countSubpasses(&mut self, features: &gpu::PlatformFeatures) {
        unsafe { (self.count_subpasses)(self, features) }
    }
    pub unsafe fn allocateResources(&mut self, flush: *mut LogicalFlush) -> bool {
        unsafe { (self.allocate_resources)(self, flush) }
    }
    pub unsafe fn pushToRenderContext(
        &mut self,
        flush: *mut LogicalFlush,
        subpass: i32,
    ) -> *mut gpu::DrawBatch {
        unsafe { (self.push_to_render_context)(self, flush, subpass) }
    }
}

fn intersect_iaabb(a: IAABB, b: IAABB) -> IAABB {
    IAABB {
        left: a.left.max(b.left),
        top: a.top.max(b.top),
        right: a.right.min(b.right),
        bottom: a.bottom.min(b.bottom),
    }
}

pub use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImpl;
pub struct IntersectionBoard {
    width: u32,
    height: u32,
    rectangles: Vec<(IAABB, i16)>,
}
impl IntersectionBoard {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rectangles: Vec::new(),
        }
    }
    pub fn resizeAndReset(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.rectangles.clear();
    }
    pub fn addRectangle(&mut self, bounds: IAABB, max_passes: i8) -> i16 {
        let mut group = 1i16;
        for (other, last_group) in self.rectangles.iter() {
            let overlaps = bounds.left < other.right
                && bounds.right > other.left
                && bounds.top < other.bottom
                && bounds.bottom > other.top;
            if overlaps {
                group = group.max(last_group.saturating_add(1));
            }
        }
        self.rectangles
            .push((bounds, group.saturating_add(max_passes as i16 - 1)));
        group
    }
}
#[repr(C)]
pub struct ImageRectDraw {
    pub base: Draw,
    pub(crate) opacity: f32,
}
impl ImageRectDraw {
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
}
#[repr(C)]
pub struct ImageMeshDraw {
    pub base: Draw,
    pub(crate) opacity: f32,
    pub(crate) index_count: u32,
    pub(crate) vertex_buffer: *mut RenderBuffer,
    pub(crate) uv_buffer: *mut RenderBuffer,
    pub(crate) index_buffer: *mut RenderBuffer,
}
impl ImageMeshDraw {
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
    pub fn indexCount(&self) -> u32 {
        self.index_count
    }
}
#[repr(C)]
pub struct ClipReset {
    pub base: Draw,
    pub(crate) previous_clip_id: u32,
}
impl ClipReset {
    pub fn previousClipID(&self) -> u32 {
        self.previous_clip_id
    }
}
pub type PushFeatherAtlasFn = unsafe fn(*mut PathDraw, *mut LogicalFlush, *mut u32, *mut u32);
pub type PushInteriorTrianglesFn = unsafe fn(
    *const PathDraw,
    u32,
    gpu::WindingFaces,
    *mut WriteOnlyMappedMemory<gpu::TriangleVertex>,
) -> usize;
unsafe fn default_push_feather(
    _: *mut PathDraw,
    _: *mut LogicalFlush,
    count: *mut u32,
    base: *mut u32,
) {
    *count = 0;
    *base = 0;
}
unsafe fn default_push_triangles(
    _: *const PathDraw,
    _: u32,
    _: gpu::WindingFaces,
    _: *mut WriteOnlyMappedMemory<gpu::TriangleVertex>,
) -> usize {
    0
}
#[repr(C)]
pub struct PathDraw {
    pub base: Draw,
    pub(crate) is_stroke: bool,
    pub(crate) feather_atlas_scissor_enabled: bool,
    pub(crate) feather_atlas_scissor: AABBu16,
    pub(crate) push_feather_atlas: PushFeatherAtlasFn,
    pub(crate) gradient: *const Gradient,
    pub(crate) paint_type: gpu::PaintType,
    pub(crate) stroke_radius: f32,
    pub(crate) feather_radius: f32,
    pub(crate) feather_atlas_transform: gpu::AtlasTransform,
    pub(crate) coverage_buffer_range: gpu::CoverageBufferRange,
    pub(crate) push_interior_triangles: PushInteriorTrianglesFn,
}
impl PathDraw {
    pub fn isStroke(&self) -> bool {
        self.is_stroke
    }
    pub fn featherAtlasScissorEnabled(&self) -> bool {
        self.feather_atlas_scissor_enabled
    }
    pub fn featherAtlasScissor(&self) -> AABBu16 {
        self.feather_atlas_scissor
    }
    pub unsafe fn pushFeatherAtlasTessellation(
        &mut self,
        flush: *mut LogicalFlush,
        count: *mut u32,
        base: *mut u32,
    ) {
        unsafe { (self.push_feather_atlas)(self, flush, count, base) }
    }
    pub fn gradient(&self) -> *const Gradient {
        self.gradient
    }
    pub fn paintType(&self) -> gpu::PaintType {
        self.paint_type
    }
    pub fn strokeRadius(&self) -> f32 {
        self.stroke_radius
    }
    pub fn featherRadius(&self) -> f32 {
        self.feather_radius
    }
    pub fn featherAtlasTransform(&self) -> gpu::AtlasTransform {
        self.feather_atlas_transform
    }
    pub fn coverageBufferRange(&self) -> gpu::CoverageBufferRange {
        self.coverage_buffer_range
    }
    pub unsafe fn pushInteriorTriangles(
        &self,
        path_id: u32,
        winding: gpu::WindingFaces,
        writer: *mut WriteOnlyMappedMemory<gpu::TriangleVertex>,
    ) -> usize {
        unsafe { (self.push_interior_triangles)(self, path_id, winding, writer) }
    }
}
impl core::ops::Deref for PathDraw {
    type Target = Draw;
    fn deref(&self) -> &Draw {
        &self.base
    }
}
impl core::ops::DerefMut for PathDraw {
    fn deref_mut(&mut self) -> &mut Draw {
        &mut self.base
    }
}
impl core::ops::Deref for ImageRectDraw {
    type Target = Draw;
    fn deref(&self) -> &Draw {
        &self.base
    }
}
impl core::ops::DerefMut for ImageRectDraw {
    fn deref_mut(&mut self) -> &mut Draw {
        &mut self.base
    }
}
impl core::ops::Deref for ImageMeshDraw {
    type Target = Draw;
    fn deref(&self) -> &Draw {
        &self.base
    }
}
impl core::ops::DerefMut for ImageMeshDraw {
    fn deref_mut(&mut self) -> &mut Draw {
        &mut self.base
    }
}
impl core::ops::Deref for ClipReset {
    type Target = Draw;
    fn deref(&self) -> &Draw {
        &self.base
    }
}
impl core::ops::DerefMut for ClipReset {
    fn deref_mut(&mut self) -> &mut Draw {
        &mut self.base
    }
}
#[derive(Clone, Copy)]
struct SkylineSegment {
    x: i32,
    y: i32,
    width: i32,
}
pub struct RectanizerSkyline {
    width: i32,
    height: i32,
    skyline: Vec<SkylineSegment>,
    area_so_far: i32,
}
impl RectanizerSkyline {
    pub fn new(width: i32, height: i32) -> Self {
        let mut out = Self {
            width,
            height,
            skyline: Vec::new(),
            area_so_far: 0,
        };
        out.reset();
        out
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn reset(&mut self) {
        self.area_so_far = 0;
        self.skyline.clear();
        self.skyline.push(SkylineSegment {
            x: 0,
            y: 0,
            width: self.width,
        });
    }
    fn rectangle_fits(&self, index: usize, width: i32, height: i32) -> Option<i32> {
        let x = self.skyline[index].x;
        if x + width > self.width {
            return None;
        }
        let mut width_left = width;
        let mut i = index;
        let mut y = self.skyline[index].y;
        while width_left > 0 {
            y = y.max(self.skyline[i].y);
            if y + height > self.height {
                return None;
            }
            width_left -= self.skyline[i].width;
            i += 1;
            debug_assert!(i < self.skyline.len() || width_left <= 0);
        }
        Some(y)
    }
    fn add_skyline_level(&mut self, index: usize, x: i32, y: i32, width: i32, height: i32) {
        self.skyline.insert(
            index,
            SkylineSegment {
                x,
                y: y + height,
                width,
            },
        );
        let mut i = index + 1;
        while i < self.skyline.len() {
            if self.skyline[i].x < self.skyline[i - 1].x + self.skyline[i - 1].width {
                let shrink = self.skyline[i - 1].x + self.skyline[i - 1].width - self.skyline[i].x;
                self.skyline[i].x += shrink;
                self.skyline[i].width -= shrink;
                if self.skyline[i].width <= 0 {
                    self.skyline.remove(i);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let mut i = 0;
        while i + 1 < self.skyline.len() {
            if self.skyline[i].y == self.skyline[i + 1].y {
                let width = self.skyline[i + 1].width;
                self.skyline[i].width += width;
                self.skyline.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }
    pub fn addRect(&mut self, width: i32, height: i32, x: &mut i16, y: &mut i16) -> bool {
        if width as u32 > self.width as u32 || height as u32 > self.height as u32 {
            return false;
        }
        let mut best_width = self.width + 1;
        let mut best_x = 0;
        let mut best_y = self.height + 1;
        let mut best_index = None;
        for i in 0..self.skyline.len() {
            if let Some(candidate_y) = self.rectangle_fits(i, width, height) {
                if candidate_y < best_y
                    || (candidate_y == best_y && self.skyline[i].width < best_width)
                {
                    best_index = Some(i);
                    best_width = self.skyline[i].width;
                    best_x = self.skyline[i].x;
                    best_y = candidate_y;
                }
            }
        }
        if let Some(index) = best_index {
            self.add_skyline_level(index, best_x, best_y, width, height);
            *x = best_x as i16;
            *y = best_y as i16;
            self.area_so_far += width * height;
            true
        } else {
            *x = 0;
            *y = 0;
            false
        }
    }
}
pub use gpu::{
    BarrierFlags, ColorRampLocation, ContourData, ContourDirections, DrawBatch, DrawContents,
    DrawType, FlushDescriptor, FlushUniforms, GradTextureLayout, GradientSpan, ImageDrawInstance,
    PaintAuxData, PaintData, PaintType, PathData, ShaderFeatures, ShaderMiscFlags, TessVertexSpan,
    TriangleVertex, TwoTexelRamp, WindingFaces,
};

#[derive(Default)]
pub struct TrivialBlockAllocator {
    allocations: Vec<(*mut u8, core::alloc::Layout)>,
}

impl TrivialBlockAllocator {
    pub fn with_capacity_in_bytes(_: usize) -> Self {
        Self::default()
    }

    pub fn make<T>(&mut self, value: T) -> *mut T {
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        self.allocations
            .push((ptr.cast(), core::alloc::Layout::new::<T>()));
        ptr
    }

    pub unsafe fn makePODArray<T>(&mut self, count: usize) -> *mut T {
        if count == 0 {
            return core::ptr::null_mut();
        }
        let layout = core::alloc::Layout::array::<T>(count).expect("arena POD array layout");
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) }.cast::<T>();
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        self.allocations.push((ptr.cast(), layout));
        ptr
    }

    pub fn reset(&mut self) {
        for (ptr, layout) in self.allocations.drain(..) {
            // `Box::into_raw` may return a dangling-but-aligned sentinel for a
            // zero-sized type. The source arena owns no storage in that case.
            if layout.size() != 0 {
                unsafe { std::alloc::dealloc(ptr, layout) };
            }
        }
    }
}

impl Drop for TrivialBlockAllocator {
    fn drop(&mut self) {
        self.reset();
    }
}

pub struct TrivialArrayAllocator<T, const ALIGN: usize = 1> {
    allocations: Vec<(*mut T, usize, core::alloc::Layout)>,
}

impl<T, const ALIGN: usize> TrivialArrayAllocator<T, ALIGN> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            allocations: Vec::with_capacity(capacity.max(1)),
        }
    }
    pub fn alloc(&mut self, count: usize) -> *mut T {
        if count == 0 {
            return core::ptr::null_mut();
        }
        let align = ALIGN.max(core::mem::align_of::<T>());
        assert!(align.is_power_of_two());
        let size = core::mem::size_of::<T>()
            .checked_mul(count)
            .expect("trivial array size");
        let layout =
            core::alloc::Layout::from_size_align(size, align).expect("trivial array alignment");
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) }.cast::<T>();
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        debug_assert_eq!((ptr as usize) & (align - 1), 0);
        self.allocations.push((ptr, count, layout));
        ptr
    }
    pub fn rewindLastAllocation(&mut self, rewind_count: usize) {
        let (_, count, _) = self
            .allocations
            .last_mut()
            .expect("rewind requires an allocation");
        assert!(rewind_count <= *count);
        *count -= rewind_count;
    }
    pub fn reset(&mut self) {
        for (ptr, _, layout) in self.allocations.drain(..) {
            if layout.size() != 0 {
                unsafe { std::alloc::dealloc(ptr.cast(), layout) };
            }
        }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self.allocations.last_mut() {
            Some((ptr, count, _)) => unsafe { core::slice::from_raw_parts_mut(*ptr, *count) },
            None => &mut [],
        }
    }
}

impl<T, const ALIGN: usize> Default for TrivialArrayAllocator<T, ALIGN> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<T, const ALIGN: usize> Drop for TrivialArrayAllocator<T, ALIGN> {
    fn drop(&mut self) {
        self.reset()
    }
}

pub struct BlockAllocatedLinkedList<T> {
    nodes: Vec<Box<T>>,
}

impl<T> Default for BlockAllocatedLinkedList<T> {
    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl<T> BlockAllocatedLinkedList<T> {
    pub fn push_back(&mut self, value: T) -> *mut T {
        self.nodes.push(Box::new(value));
        self.nodes.last_mut().unwrap().as_mut()
    }
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    pub fn tail(&mut self) -> *mut T {
        self.nodes
            .last_mut()
            .map_or(core::ptr::null_mut(), |node| node.as_mut())
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.nodes.iter().map(|node| node.as_ref())
    }
}

pub type WriteOnlyMappedMemory<T> = gpu::WriteOnlyMappedMemory<T>;

pub enum Fit {}
pub enum Alignment {}
#[repr(C)]
pub struct Span<T> {
    pub data: *const T,
    pub size: usize,
}

#[cfg(feature = "rive-ktx2")]
#[derive(Clone, Copy, Debug)]
pub struct Ktx2HwSupport {
    pub supports_bc: bool,
    pub supports_astc: bool,
    pub supports_etc2: bool,
}

#[cfg(feature = "rive-ktx2")]
pub struct Ktx2DecodeResult {
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub level_count: u32,
    pub format:
        crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat,
    pub blocks: Vec<u8>,
    pub block_width: u8,
    pub block_height: u8,
    pub srgb: bool,
}

#[cfg(feature = "rive-ktx2")]
pub trait Ktx2DecoderContract {
    fn decodeKtx2(&mut self, encoded: &[u8], support: Ktx2HwSupport) -> Option<Ktx2DecodeResult>;
}

#[cfg(feature = "rive-decoders")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitmapPixelFormat {
    rgbaPremul,
    other,
}

#[cfg(feature = "rive-decoders")]
pub struct BitmapDecodeResult {
    pub width: u32,
    pub height: u32,
    pub pixel_format: BitmapPixelFormat,
    pub bytes: Vec<u8>,
}

#[cfg(feature = "rive-decoders")]
pub trait BitmapDecoderContract {
    fn decodeBitmap(&mut self, encoded: &[u8]) -> Option<BitmapDecodeResult>;
    fn convertToRGBAPremul(&mut self, bitmap: &mut BitmapDecodeResult);
}

pub struct GradientContentKey {
    // rcp<const Gradient> m_gradient;
    // The pointee is intrusive-owned; constness is a source immutability
    // qualifier, not a nullable or borrowed conversion.
    m_gradient: rcp<Gradient>,
}

impl PartialEq for GradientContentKey {
    fn eq(&self, other: &Self) -> bool {
        let lhs = self.gradient();
        let rhs = other.gradient();
        if lhs.is_null() || rhs.is_null() {
            return lhs == rhs;
        }
        let lhs = unsafe { &*lhs };
        let rhs = unsafe { &*rhs };
        lhs.colors_slice() == rhs.colors_slice()
            && lhs.stops_slice().len() == rhs.stops_slice().len()
            && lhs
                .stops_slice()
                .iter()
                .zip(rhs.stops_slice().iter())
                .all(|(a, b)| a.to_bits() == b.to_bits())
    }
}

impl Eq for GradientContentKey {}

impl Hash for GradientContentKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let gradient = self.gradient();
        if gradient.is_null() {
            0u8.hash(state);
            return;
        }
        1u8.hash(state);
        let gradient = unsafe { &*gradient };
        for stop in gradient.stops_slice().iter() {
            stop.to_bits().hash(state);
        }
        gradient.colors_slice().hash(state);
    }
}

impl GradientContentKey {
    // inline GradientContentKey(rcp<const Gradient> gradient);
    pub unsafe fn new(gradient: rcp<Gradient>) -> Self {
        debug_assert!(!gradient.get().is_null());
        Self {
            m_gradient: gradient,
        }
    }

    // inline GradientContentKey(GradientContentKey&& other);
    pub fn move_from(other: &mut Self) -> Self {
        // Source move construction transfers the intrusive owner without a
        // retain and leaves the moved-from owner empty.
        let moved = core::mem::replace(&mut other.m_gradient, rcp::new());
        Self { m_gradient: moved }
    }

    // const Gradient* gradient() const { return m_gradient.get(); }
    pub fn gradient(&self) -> *mut Gradient {
        self.m_gradient.get()
    }
}

pub struct DeepHashGradient;

pub trait DeepHashGradientContract {
    // size_t operator()(const GradientContentKey&) const;
    fn hash(&self, key: &GradientContentKey) -> usize;
}

pub struct DrawReleaseRefs;

pub trait DrawReleaseRefsContract {
    // void operator()(Draw* draw);
    unsafe fn call(&self, draw: *mut Draw);
}

// using DrawUniquePtr = std::unique_ptr<Draw, DrawReleaseRefs>;
#[repr(C)]
pub struct DrawUniquePtr(pub *mut Draw, Option<Box<dyn core::any::Any>>);

impl DrawUniquePtr {
    /// # Safety
    /// `draw` must be null or point to a live complete Draw allocation that
    /// remains valid until this pointer is dropped.
    pub unsafe fn from_raw(draw: *mut Draw) -> Self {
        Self(draw, None)
    }

    pub fn null() -> Self {
        Self(core::ptr::null_mut(), None)
    }

    /// Retain the complete most-derived source allocation alongside its
    /// offset-zero Draw pointer. The source unique_ptr owns a block-allocated
    /// draw; this token preserves that ownership boundary for Rust owners.
    /// # Safety
    /// `draw` must point to the live offset-zero Draw base within `owner`.
    pub unsafe fn from_owner<T: 'static>(draw: *mut Draw, owner: T) -> Self {
        Self(draw, Some(Box::new(owner)))
    }
}

impl Drop for DrawUniquePtr {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // The custom source deleter intentionally releases intrusive
            // references but does not free the arena-backed Draw allocation.
            unsafe { (&mut *self.0).releaseRefs() };
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RenderContextFrameDescriptor {
    // uint32_t renderTargetWidth = 0;
    pub renderTargetWidth: u32,
    // uint32_t renderTargetHeight = 0;
    pub renderTargetHeight: u32,
    // LoadAction loadAction = LoadAction::clear;
    pub loadAction: LoadAction,
    // ColorInt clearColor = 0;
    pub clearColor: ColorInt,
    // uint32_t msaaSampleCount = 0;
    pub msaaSampleCount: u32,
    // bool disableRasterOrdering = false;
    pub disableRasterOrdering: bool,
    // DitherMode ditherMode = DitherMode::interleavedGradientNoise;
    pub ditherMode: DitherMode,
    // uint32_t virtualTileWidth = 0;
    pub virtualTileWidth: u32,
    // uint32_t virtualTileHeight = 0;
    pub virtualTileHeight: u32,
    // bool wireframe = false;
    pub wireframe: bool,
    // bool fillsDisabled = false;
    pub fillsDisabled: bool,
    // bool strokesDisabled = false;
    pub strokesDisabled: bool,
    // bool clockwiseFillOverride = false;
    pub clockwiseFillOverride: bool,
    #[cfg(feature = "with-rive-tools")]
    // gpu::SynthesizedFailureType synthesizedFailureType =
    //     gpu::SynthesizedFailureType::none;
    pub synthesizedFailureType: gpu::SynthesizedFailureType,
}

impl Default for RenderContextFrameDescriptor {
    fn default() -> Self {
        Self {
            renderTargetWidth: 0,
            renderTargetHeight: 0,
            loadAction: LoadAction::clear,
            clearColor: 0,
            msaaSampleCount: 0,
            disableRasterOrdering: false,
            ditherMode: DitherMode::interleavedGradientNoise,
            virtualTileWidth: 0,
            virtualTileHeight: 0,
            wireframe: false,
            fillsDisabled: false,
            strokesDisabled: false,
            clockwiseFillOverride: false,
            #[cfg(feature = "with-rive-tools")]
            synthesizedFailureType: gpu::SynthesizedFailureType::none,
        }
    }
}

// using FrameDescriptor inside RenderContext.
pub type FrameDescriptor = RenderContextFrameDescriptor;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderContextFlushResources {
    // RenderTarget* renderTarget = nullptr;
    pub renderTarget: *mut RenderTarget,
    // void* externalCommandBuffer = nullptr;
    pub externalCommandBuffer: *mut c_void,
    // uint64_t currentFrameNumber = 0;
    pub currentFrameNumber: u64,
    // uint64_t safeFrameNumber = 0;
    pub safeFrameNumber: u64,
}

pub type FlushResources = RenderContextFlushResources;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ResourceAllocationCounts {
    pub flushUniformBufferCount: usize,
    pub pathBufferCount: usize,
    pub paintBufferCount: usize,
    pub paintAuxBufferCount: usize,
    pub contourBufferCount: usize,
    pub gradSpanBufferCount: usize,
    pub tessSpanBufferCount: usize,
    pub triangleVertexBufferCount: usize,
    pub imageDrawInstanceBufferCount: usize,
    pub gradTextureHeight: usize,
    pub tessTextureHeight: usize,
    pub featherAtlasTextureWidth: usize,
    pub featherAtlasTextureHeight: usize,
    pub plsTransientBackingWidth: usize,
    pub plsTransientBackingHeight: usize,
    pub plsTransientBackingPlaneCount: usize,
    pub plsAtomicCoverageBackingWidth: usize,
    pub plsAtomicCoverageBackingHeight: usize,
    pub coverageBufferLength: usize,
}

impl ResourceAllocationCounts {
    pub const NUM_ELEMENTS: usize = 19;

    // VecType toVec() const;
    pub fn toVec(&self) -> [usize; Self::NUM_ELEMENTS] {
        [
            self.flushUniformBufferCount,
            self.pathBufferCount,
            self.paintBufferCount,
            self.paintAuxBufferCount,
            self.contourBufferCount,
            self.gradSpanBufferCount,
            self.tessSpanBufferCount,
            self.triangleVertexBufferCount,
            self.imageDrawInstanceBufferCount,
            self.gradTextureHeight,
            self.tessTextureHeight,
            self.featherAtlasTextureWidth,
            self.featherAtlasTextureHeight,
            self.plsTransientBackingWidth,
            self.plsTransientBackingHeight,
            self.plsTransientBackingPlaneCount,
            self.plsAtomicCoverageBackingWidth,
            self.plsAtomicCoverageBackingHeight,
            self.coverageBufferLength,
        ]
    }

    // static ResourceAllocationCounts FromVec(const VecType& vec);
    pub fn FromVec(vec: &[usize; Self::NUM_ELEMENTS]) -> Self {
        Self {
            flushUniformBufferCount: vec[0],
            pathBufferCount: vec[1],
            paintBufferCount: vec[2],
            paintAuxBufferCount: vec[3],
            contourBufferCount: vec[4],
            gradSpanBufferCount: vec[5],
            tessSpanBufferCount: vec[6],
            triangleVertexBufferCount: vec[7],
            imageDrawInstanceBufferCount: vec[8],
            gradTextureHeight: vec[9],
            tessTextureHeight: vec[10],
            featherAtlasTextureWidth: vec[11],
            featherAtlasTextureHeight: vec[12],
            plsTransientBackingWidth: vec[13],
            plsTransientBackingHeight: vec[14],
            plsTransientBackingPlaneCount: vec[15],
            plsAtomicCoverageBackingWidth: vec[16],
            plsAtomicCoverageBackingHeight: vec[17],
            coverageBufferLength: vec[18],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawSortEntry {
    pub sortKey: i64,
    pub drawIndex: i16,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ScissorAABBHasher;

impl ScissorAABBHasher {
    // size_t operator()(AABBu16 aabb) const;
    pub fn call(&self, aabb: AABBu16) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        aabb.hash(&mut hasher);
        hasher.finish() as usize
    }
}

pub struct RenderContextImplOwner {
    base: NonNull<RenderContextImpl>,
    owner: Box<dyn crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImplContract>,
}

impl RenderContextImplOwner {
    /// The complete implementation must embed its RenderContextImpl base at
    /// offset zero, exactly like the C++ base subobject used by static_cast.
    pub fn from_box<T>(owner: Box<T>) -> Self
    where
        T: crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImplContract + 'static,
    {
        let mut owner: Box<dyn crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImplContract> = owner;
        let base = NonNull::from(owner.renderContextImplMut());
        Self { base, owner }
    }

    pub fn as_ptr(&self) -> *mut RenderContextImpl {
        self.base.as_ptr()
    }
    pub fn contract(&self) -> &dyn crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImplContract{
        &*self.owner
    }
    pub fn contract_mut(&mut self) -> &mut dyn crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImplContract{
        &mut *self.owner
    }
}

/// The exact C++ member sequence after the offset-zero `RiveRenderFactory`
/// base. Keeping the source fields in this `repr(C)` aggregate prevents Rust
/// destruction policy from changing their physical order.
#[repr(C)]
#[doc(hidden)]
pub struct RenderContextMembers {
    pub(crate) m_impl: RenderContextImplOwner,
    pub(crate) m_max_path_id: usize,
    #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
    pub(crate) m_ore_context: Option<Box<OreContext>>,
    pub(crate) m_current_resource_allocations: ResourceAllocationCounts,
    pub(crate) m_max_recent_resource_requirements: ResourceAllocationCounts,
    pub(crate) m_last_resource_trim_time_in_seconds: f64,
    pub(crate) m_frame_descriptor: FrameDescriptor,
    pub(crate) m_frame_interlock_mode: gpu::InterlockMode,
    pub(crate) m_frame_shader_features_mask: gpu::ShaderFeatures,
    #[cfg(debug_assertions)]
    pub(crate) m_did_begin_frame: bool,
    pub(crate) m_clip_content_id: u32,
    pub(crate) m_coverage_buffer_prefix: u32,
    pub(crate) m_indirect_draw_list: Vec<DrawSortEntry>,
    pub(crate) m_intersection_board: Option<Box<IntersectionBoard>>,
    pub(crate) m_scissor_id_lookup: HashMap<AABBu16, i16>,
    pub(crate) m_prev_scissor_id: i16,
    pub(crate) m_flush_uniform_data: WriteOnlyMappedMemory<gpu::FlushUniforms>,
    pub(crate) m_path_data: WriteOnlyMappedMemory<gpu::PathData>,
    pub(crate) m_paint_data: WriteOnlyMappedMemory<gpu::PaintData>,
    pub(crate) m_paint_aux_data: WriteOnlyMappedMemory<gpu::PaintAuxData>,
    pub(crate) m_contour_data: WriteOnlyMappedMemory<gpu::ContourData>,
    pub(crate) m_grad_span_data: WriteOnlyMappedMemory<gpu::GradientSpan>,
    pub(crate) m_tess_span_data: WriteOnlyMappedMemory<gpu::TessVertexSpan>,
    pub(crate) m_triangle_vertex_data: WriteOnlyMappedMemory<gpu::TriangleVertex>,
    pub(crate) m_image_draw_instance_data: WriteOnlyMappedMemory<gpu::ImageDrawInstance>,
    pub(crate) m_per_frame_allocator: TrivialBlockAllocator,
    pub(crate) m_num_chops_allocator: TrivialArrayAllocator<u8>,
    pub(crate) m_chop_vertices_allocator: TrivialArrayAllocator<Vec2D>,
    pub(crate) m_tangent_pairs_allocator: TrivialArrayAllocator<[Vec2D; 2]>,
    pub(crate) m_polar_segment_counts_allocator: TrivialArrayAllocator<u32, 16>,
    pub(crate) m_parametric_segment_counts_allocator: TrivialArrayAllocator<u32, 16>,
    pub(crate) m_logical_flushes: Vec<Box<LogicalFlush>>,
}

#[repr(C)]
pub struct RenderContext {
    // Source base chain: RenderContext -> RiveRenderFactory -> Factory. The
    // base remains offset zero and is destroyed explicitly after all members.
    pub(crate) base: ManuallyDrop<RiveRenderFactory>,
    pub(crate) members: ManuallyDrop<RenderContextMembers>,
    // Decoder injection is a product-only extension, not a pinned source
    // member. It stays outside the source-order aggregate.
    #[cfg(feature = "rive-ktx2")]
    pub(crate) m_ktx2_decoder: Option<Box<dyn Ktx2DecoderContract>>,
    #[cfg(feature = "rive-decoders")]
    pub(crate) m_bitmap_decoder: Option<Box<dyn BitmapDecoderContract>>,
    pub(crate) _pin: PhantomPinned,
}

impl Deref for RenderContext {
    type Target = RenderContextMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for RenderContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

#[cfg(test)]
mod render_context_layout_tests {
    use super::*;
    use core::mem::{align_of, offset_of, size_of};

    #[test]
    fn source_members_keep_authored_physical_order_after_offset_zero_base() {
        assert_eq!(offset_of!(RenderContext, base), 0);
        let expected_members_offset = size_of::<RiveRenderFactory>()
            .next_multiple_of(align_of::<RenderContextMembers>());
        assert_eq!(offset_of!(RenderContext, members), expected_members_offset);
        assert_eq!(offset_of!(RenderContextMembers, m_impl), 0);

        let mut offsets = vec![
            offset_of!(RenderContextMembers, m_impl),
            offset_of!(RenderContextMembers, m_max_path_id),
        ];
        #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
        offsets.push(offset_of!(RenderContextMembers, m_ore_context));
        offsets.extend([
            offset_of!(RenderContextMembers, m_current_resource_allocations),
            offset_of!(RenderContextMembers, m_max_recent_resource_requirements),
            offset_of!(RenderContextMembers, m_last_resource_trim_time_in_seconds),
            offset_of!(RenderContextMembers, m_frame_descriptor),
            offset_of!(RenderContextMembers, m_frame_interlock_mode),
            offset_of!(RenderContextMembers, m_frame_shader_features_mask),
        ]);
        #[cfg(debug_assertions)]
        offsets.push(offset_of!(RenderContextMembers, m_did_begin_frame));
        offsets.extend([
            offset_of!(RenderContextMembers, m_clip_content_id),
            offset_of!(RenderContextMembers, m_coverage_buffer_prefix),
            offset_of!(RenderContextMembers, m_indirect_draw_list),
            offset_of!(RenderContextMembers, m_intersection_board),
            offset_of!(RenderContextMembers, m_scissor_id_lookup),
            offset_of!(RenderContextMembers, m_prev_scissor_id),
            offset_of!(RenderContextMembers, m_flush_uniform_data),
            offset_of!(RenderContextMembers, m_path_data),
            offset_of!(RenderContextMembers, m_paint_data),
            offset_of!(RenderContextMembers, m_paint_aux_data),
            offset_of!(RenderContextMembers, m_contour_data),
            offset_of!(RenderContextMembers, m_grad_span_data),
            offset_of!(RenderContextMembers, m_tess_span_data),
            offset_of!(RenderContextMembers, m_triangle_vertex_data),
            offset_of!(RenderContextMembers, m_image_draw_instance_data),
            offset_of!(RenderContextMembers, m_per_frame_allocator),
            offset_of!(RenderContextMembers, m_num_chops_allocator),
            offset_of!(RenderContextMembers, m_chop_vertices_allocator),
            offset_of!(RenderContextMembers, m_tangent_pairs_allocator),
            offset_of!(RenderContextMembers, m_polar_segment_counts_allocator),
            offset_of!(RenderContextMembers, m_parametric_segment_counts_allocator),
            offset_of!(RenderContextMembers, m_logical_flushes),
        ]);
        assert!(
            offsets.windows(2).all(|pair| pair[0] < pair[1]),
            "RenderContext members must remain in pinned C++ declaration order: {offsets:?}"
        );
    }
}

impl RenderContext {
    // RenderContextImpl* impl() { return m_impl.get(); }
    pub fn impl_ptr(&self) -> *mut RenderContextImpl {
        self.m_impl.as_ptr()
    }

    // const FrameDescriptor& frameDescriptor() const;
    pub fn frameDescriptor(&self) -> &FrameDescriptor {
        #[cfg(debug_assertions)]
        assert!(self.m_did_begin_frame);
        &self.m_frame_descriptor
    }

    // const gpu::InterlockMode frameInterlockMode() const;
    pub fn frameInterlockMode(&self) -> gpu::InterlockMode {
        self.m_frame_interlock_mode
    }

    pub fn platformFeatures(&self) -> &gpu::PlatformFeatures {
        unsafe { (&*self.impl_ptr()).platformFeatures() }
    }

    pub fn getClipContentBounds(&self, clip_id: u32) -> &IAABB {
        #[cfg(debug_assertions)]
        {
            assert!(self.m_did_begin_frame);
            assert!(!self.m_logical_flushes.is_empty());
        }
        &self
            .m_logical_flushes
            .last()
            .unwrap()
            .getClipInfo(clip_id)
            .contentBounds
    }

    pub fn getTightenedClipBounds(&self, clip_id: u32) -> &AABBu16 {
        #[cfg(debug_assertions)]
        {
            assert!(self.m_did_begin_frame);
            assert!(!self.m_logical_flushes.is_empty());
        }
        &self
            .m_logical_flushes
            .last()
            .unwrap()
            .getClipInfo(clip_id)
            .tightenedBounds
    }

    // void setClipContentID(uint32_t clipID);
    pub fn setClipContentID(&mut self, clip_id: u32) {
        #[cfg(debug_assertions)]
        assert!(self.m_did_begin_frame);
        self.m_clip_content_id = clip_id;
    }

    // uint32_t getClipContentID() const;
    pub fn getClipContentID(&self) -> u32 {
        #[cfg(debug_assertions)]
        assert!(self.m_did_begin_frame);
        self.m_clip_content_id
    }

    // TrivialBlockAllocator& perFrameAllocator();
    pub fn perFrameAllocator(&mut self) -> &mut TrivialBlockAllocator {
        #[cfg(debug_assertions)]
        assert!(self.m_did_begin_frame);
        &mut self.m_per_frame_allocator
    }

    // TrivialArrayAllocator<T>& allocator accessors.
    pub fn numChopsAllocator(&mut self) -> &mut TrivialArrayAllocator<u8> {
        &mut self.m_num_chops_allocator
    }
    pub fn chopVerticesAllocator(&mut self) -> &mut TrivialArrayAllocator<Vec2D> {
        &mut self.m_chop_vertices_allocator
    }
    pub fn tangentPairsAllocator(&mut self) -> &mut TrivialArrayAllocator<[Vec2D; 2]> {
        &mut self.m_tangent_pairs_allocator
    }
    pub fn polarSegmentCountsAllocator(&mut self) -> &mut TrivialArrayAllocator<u32, 16> {
        &mut self.m_polar_segment_counts_allocator
    }
    pub fn parametricSegmentCountsAllocator(&mut self) -> &mut TrivialArrayAllocator<u32, 16> {
        &mut self.m_parametric_segment_counts_allocator
    }

    pub fn make<T>(&mut self, value: T) -> *mut T {
        #[cfg(debug_assertions)]
        assert!(self.m_did_begin_frame);
        self.m_per_frame_allocator.make(value)
    }

    pub fn featherAtlasMaxSize(&self) -> u32 {
        self.platformFeatures().maxTextureSize.min(4096)
    }
}

pub trait RenderContextContract: RiveRenderFactoryContract {
    // RenderContext(std::unique_ptr<RenderContextImpl>);
    // ~RenderContext();
    fn new<T>(implementation: Box<T>) -> Pin<Box<Self>>
    where
        Self: Sized,
        T: crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImplContract + 'static;
    fn impl_ptr(&self) -> *mut RenderContextImpl;
    fn static_impl_cast<T>(&self) -> *mut T
    where
        Self: Sized;
    fn platformFeatures(&self) -> &gpu::PlatformFeatures;
    fn frameDescriptor(&self) -> &FrameDescriptor;
    fn beginFrame(&mut self, descriptor: &FrameDescriptor);
    fn isOutsideCurrentFrame(&self, pixel_bounds: &IAABB) -> bool;
    fn frameSupportsClipRects(&self) -> bool;
    fn frameSupportsImagePaintForPaths(&self) -> bool;
    fn frameInterlockMode(&self) -> gpu::InterlockMode;
    fn generateClipID(
        &mut self,
        content_bounds: IAABB,
        parent_clip_id: u32,
        tightened_bounds: AABBu16,
    ) -> u32;
    fn pushDraws(&mut self, draws: &mut [DrawUniquePtr], draw_count: usize) -> bool;
    fn logicalFlush(&mut self);
    unsafe fn flush(&mut self, resources: &FlushResources);
    fn releaseResources(&mut self);
    fn perFrameAllocator(&mut self) -> &mut TrivialBlockAllocator;
    fn numChopsAllocator(&mut self) -> &mut TrivialArrayAllocator<u8>;
    fn chopVerticesAllocator(&mut self) -> &mut TrivialArrayAllocator<Vec2D>;
    fn tangentPairsAllocator(&mut self) -> &mut TrivialArrayAllocator<[Vec2D; 2]>;
    fn polarSegmentCountsAllocator(&mut self) -> &mut TrivialArrayAllocator<u32, 16>;
    fn parametricSegmentCountsAllocator(&mut self) -> &mut TrivialArrayAllocator<u32, 16>;
    #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
    fn makeRenderCanvas(&mut self, width: u32, height: u32) -> rcp<RenderCanvas>;
    #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
    fn getOreContext(&mut self) -> *mut OreContext;
    fn resetContainers(&mut self);
    fn setResourceSizes(&mut self, counts: ResourceAllocationCounts, force_realloc: bool);
    fn mapResourceBuffers(&mut self, counts: &ResourceAllocationCounts) -> bool;
    fn unmapResourceBuffers(&mut self, counts: &ResourceAllocationCounts);
    fn incrementCoverageBufferPrefix(&mut self, clear: &mut bool) -> u32;
}

pub struct LogicalFlush {
    // RenderContext* const m_ctx;
    pub(crate) m_ctx: NonNull<RenderContext>,
    pub(crate) m_resource_counts: ResourceCounters,
    pub(crate) m_draw_pass_count: i32,
    pub(crate) m_simple_gradients: HashMap<u64, u32>,
    pub(crate) m_pending_simple_grad_draws: Vec<TwoTexelRamp>,
    pub(crate) m_complex_gradients: HashMap<GradientContentKey, u16>,
    pub(crate) m_pending_complex_grad_draws: Vec<*const Gradient>,
    pub(crate) m_pending_grad_span_count: usize,
    pub(crate) m_clips: Vec<ClipInfo>,
    pub(crate) m_draws: Vec<DrawUniquePtr>,
    pub(crate) m_combined_draw_bounds: IAABB,
    pub(crate) m_combined_draw_contents: gpu::DrawContents,

    pub(crate) m_path_padding_count: u32,
    pub(crate) m_paint_padding_count: u32,
    pub(crate) m_paint_aux_padding_count: u32,
    pub(crate) m_contour_padding_count: u32,
    pub(crate) m_grad_span_padding_count: u32,
    pub(crate) m_midpoint_fan_tess_end_location: u32,
    pub(crate) m_outer_cubic_tess_end_location: u32,
    pub(crate) m_outer_cubic_tess_vertex_idx: u32,
    pub(crate) m_midpoint_fan_tess_vertex_idx: u32,
    pub(crate) m_grad_texture_layout: gpu::GradTextureLayout,
    pub(crate) m_baseline_shader_misc_flags: gpu::ShaderMiscFlags,
    pub(crate) m_flush_desc: gpu::FlushDescriptor,
    pub(crate) m_draw_list: BlockAllocatedLinkedList<DrawBatch>,
    pub(crate) m_first_dst_blend_barrier: *const DrawBatch,
    pub(crate) m_dst_blend_barrier_list_tail: *mut *const DrawBatch,
    pub(crate) m_combined_shader_features: gpu::ShaderFeatures,
    pub(crate) m_current_path_id: u32,
    pub(crate) m_current_contour_id: u32,
    pub(crate) m_feather_atlas_rectanizer: Option<Box<RectanizerSkyline>>,
    pub(crate) m_feather_atlas_max_x: u32,
    pub(crate) m_feather_atlas_max_y: u32,
    pub(crate) m_pending_feather_atlas_draws: Vec<*mut PathDraw>,
    pub(crate) m_coverage_buffer_length: u32,
    pub(crate) m_pending_barriers: gpu::BarrierFlags,
    pub(crate) m_current_z_index: u32,
    #[cfg(debug_assertions)]
    pub(crate) m_has_done_layout: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ClipInfo {
    // const uint32_t parentClipID = 0;
    pub parentClipID: u32,
    // const IAABB contentBounds;
    pub contentBounds: IAABB,
    // AABBu16 tightenedBounds;
    pub tightenedBounds: AABBu16,
    // AABBu16 readBounds = AABBu16::makeMaximallyNegative();
    pub readBounds: AABBu16,
}

impl ClipInfo {
    pub fn new(content_bounds: IAABB, parent_clip_id: u32, tightened_bounds: AABBu16) -> Self {
        Self {
            parentClipID: parent_clip_id,
            contentBounds: content_bounds,
            tightenedBounds: tightened_bounds,
            readBounds: AABBu16::makeMaximallyNegative(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ResourceCounters {
    pub midpointFanTessVertexCount: usize,
    pub outerCubicTessVertexCount: usize,
    pub pathCount: usize,
    pub contourCount: usize,
    pub maxTessellatedSegmentCount: usize,
    pub maxTriangleVertexCount: usize,
    pub imageDrawCount: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutCounters {
    pub pathPaddingCount: u32,
    pub paintPaddingCount: u32,
    pub paintAuxPaddingCount: u32,
    pub contourPaddingCount: u32,
    pub gradSpanCount: u32,
    pub gradSpanPaddingCount: u32,
    pub maxGradTextureHeight: u32,
    pub maxTessTextureHeight: u32,
    pub maxFeatherAtlasWidth: u32,
    pub maxFeatherAtlasHeight: u32,
    pub maxPLSTransientBackingPlaneCount: u32,
    pub maxCoverageBufferLength: usize,
}

pub type LogicalFlushClipInfo = ClipInfo;
pub type LogicalFlushResourceCounters = ResourceCounters;
pub type LogicalFlushLayoutCounters = LayoutCounters;

impl LogicalFlush {
    // LogicalFlush(RenderContext* parent);
    // const FrameDescriptor& frameDescriptor() const;
    pub fn frameDescriptor(&self) -> &FrameDescriptor {
        unsafe { self.m_ctx.as_ref().frameDescriptor() }
    }

    // gpu::InterlockMode interlockMode() const;
    pub fn interlockMode(&self) -> gpu::InterlockMode {
        unsafe { self.m_ctx.as_ref().frameInterlockMode() }
    }

    pub fn platformFeatures(&self) -> &gpu::PlatformFeatures {
        unsafe { self.m_ctx.as_ref().platformFeatures() }
    }

    // const gpu::FlushDescriptor& desc();
    pub fn desc(&self) -> &gpu::FlushDescriptor {
        #[cfg(debug_assertions)]
        assert!(self.m_has_done_layout);
        &self.m_flush_desc
    }

    // const ClipInfo& getClipInfo(uint32_t clipID);
    pub fn getClipInfo(&self, clip_id: u32) -> &ClipInfo {
        debug_assert!(clip_id > 0 && clip_id as usize <= self.m_clips.len());
        &self.m_clips[clip_id as usize - 1]
    }
}

pub trait LogicalFlushContract {
    /// # Safety
    /// The pinned parent must outlive the returned source-shaped flush; that
    /// lifetime is represented by its stored raw pointer rather than the type.
    unsafe fn new(parent: Pin<&mut RenderContext>) -> Self
    where
        Self: Sized;
    fn rewind(&mut self);
    fn resetContainers(&mut self);
    fn frameDescriptor(&self) -> &FrameDescriptor;
    fn interlockMode(&self) -> gpu::InterlockMode;
    fn platformFeatures(&self) -> &gpu::PlatformFeatures;
    fn desc(&self) -> &gpu::FlushDescriptor;
    fn generateClipID(
        &mut self,
        content_bounds: IAABB,
        parent_clip_id: u32,
        tightened_bounds: AABBu16,
    ) -> u32;
    fn pushDraws(&mut self, draws: &mut [DrawUniquePtr], draw_count: usize) -> bool;
    unsafe fn allocateGradient(
        &mut self,
        gradient: *const Gradient,
        location: *mut ColorRampLocation,
    ) -> bool;
    unsafe fn allocateFeatherAtlasDraw(
        &mut self,
        draw: *mut PathDraw,
        draw_width: u16,
        draw_height: u16,
        desired_padding: u16,
        x: *mut u16,
        y: *mut u16,
        padded_region: *mut AABBu16,
    ) -> bool;
    fn allocateCoverageBufferRange(&mut self, length: usize) -> usize;
    unsafe fn layoutResources(
        &mut self,
        resources: &FlushResources,
        logical_flush_idx: usize,
        running_resource_counts: *mut ResourceCounters,
        running_layout_counts: *mut LayoutCounters,
    );
    fn writeResources(&mut self);
    fn allocateMidpointFanTessVertices(&mut self, count: u32) -> u32;
    fn allocateOuterCubicTessVertices(&mut self, count: u32) -> u32;
    unsafe fn pushPath(&mut self, draw: *const PathDraw) -> u32;
    fn pushContour(
        &mut self,
        path_id: u32,
        midpoint: Vec2D,
        is_stroke: bool,
        closed: bool,
        vertex_index_0: u32,
    ) -> u32;
    fn pushPaddingVertices(&mut self, count: u32, tess_location: u32);
    fn pushBarriers(&mut self, barriers: gpu::BarrierFlags);
    unsafe fn pushMidpointFanDraw(
        &mut self,
        draw: *const PathDraw,
        draw_type: gpu::DrawType,
        tess_vertex_count: u32,
        tess_location: u32,
        misc: gpu::ShaderMiscFlags,
    ) -> *mut gpu::DrawBatch;
    unsafe fn pushOuterCubicsDraw(
        &mut self,
        draw: *const PathDraw,
        draw_type: gpu::DrawType,
        tess_vertex_count: u32,
        tess_location: u32,
        misc: gpu::ShaderMiscFlags,
    ) -> *mut gpu::DrawBatch;
    unsafe fn pushInteriorTriangulationDraw(
        &mut self,
        draw: *const PathDraw,
        path_id: u32,
        winding_faces: gpu::WindingFaces,
        #[cfg(debug_assertions)] vertex_counter: *mut usize,
    ) -> *mut gpu::DrawBatch;
    unsafe fn pushFeatherAtlasBlit(
        &mut self,
        draw: *mut PathDraw,
        path_id: u32,
    ) -> *mut gpu::DrawBatch;
    unsafe fn pushImageRectDraw(&mut self, draw: *mut ImageRectDraw) -> *mut gpu::DrawBatch;
    unsafe fn pushImageMeshDraw(&mut self, draw: *mut ImageMeshDraw) -> *mut gpu::DrawBatch;
    unsafe fn pushClipResetDraw(&mut self, draw: *mut ClipReset) -> *mut gpu::DrawBatch;
    fn getWritableClipInfo(&mut self, clip_id: u32) -> &mut ClipInfo;
    unsafe fn pushPathDraw(
        &mut self,
        draw: *const PathDraw,
        draw_type: gpu::DrawType,
        misc: gpu::ShaderMiscFlags,
        vertex_count: u32,
        base_vertex: u32,
    ) -> *mut gpu::DrawBatch;
    unsafe fn pushDraw(
        &mut self,
        draw: *const Draw,
        draw_type: gpu::DrawType,
        misc: gpu::ShaderMiscFlags,
        paint_type: gpu::PaintType,
        element_count: u32,
        base_element: u32,
    ) -> *mut gpu::DrawBatch;
    fn tightenClipBounds(&mut self);
    unsafe fn addBatchToDstBarrierList(&mut self, batch: *mut gpu::DrawBatch);
}

pub struct TessellationWriter<'a> {
    // LogicalFlush* const m_flush;
    pub(crate) m_flush: &'a mut LogicalFlush,
    // WriteOnlyMappedMemory<gpu::TessVertexSpan>& m_tessSpanData;
    pub(crate) m_tess_span_data: &'a mut WriteOnlyMappedMemory<gpu::TessVertexSpan>,
    // const uint32_t m_pathID;
    pub(crate) m_path_id: u32,
    // const gpu::ContourDirections m_contourDirections;
    pub(crate) m_contour_directions: gpu::ContourDirections,
    pub(crate) m_path_tess_location: u32,
    pub(crate) m_path_mirrored_tess_location: u32,
    pub(crate) m_next_cubic_padding_vertex_count: u32,
    #[cfg(debug_assertions)]
    pub(crate) m_expected_path_tess_end_location: u32,
    #[cfg(debug_assertions)]
    pub(crate) m_expected_path_mirrored_tess_end_location: u32,
}

impl<'a> TessellationWriter<'a> {
    // TessellationWriter(LogicalFlush* flush, ...);
    pub fn nextVertexIndex(&self) -> u32 {
        if self.m_contour_directions != gpu::ContourDirections::reverse {
            self.m_path_tess_location
        } else {
            self.m_path_mirrored_tess_location - 1
        }
    }
}

pub trait TessellationWriterContract<'a> {
    fn new(
        flush: &'a mut LogicalFlush,
        path_id: u32,
        contour_directions: gpu::ContourDirections,
        forward_tess_vertex_count: u32,
        forward_tess_location: u32,
        mirrored_tess_vertex_count: u32,
        mirrored_tess_location: u32,
    ) -> Self
    where
        Self: Sized;
    fn pushContour(
        &mut self,
        midpoint: Vec2D,
        is_stroke: bool,
        closed: bool,
        padding_vertex_count: u32,
    ) -> u32;
    fn pushCubic(
        &mut self,
        pts: &[Vec2D; 4],
        contour_directions: gpu::ContourDirections,
        join_tangent: Vec2D,
        parametric_segment_count: u32,
        polar_segment_count: u32,
        join_segment_count: u32,
        contour_id_with_flags: u32,
    );
    fn pushTessellationSpans(
        &mut self,
        pts: &[Vec2D; 4],
        join_tangent: Vec2D,
        total_vertex_count: u32,
        parametric_segment_count: u32,
        polar_segment_count: u32,
        join_segment_count: u32,
        contour_id_with_flags: u32,
    );
    fn pushMirroredTessellationSpans(
        &mut self,
        pts: &[Vec2D; 4],
        join_tangent: Vec2D,
        total_vertex_count: u32,
        parametric_segment_count: u32,
        polar_segment_count: u32,
        join_segment_count: u32,
        contour_id_with_flags: u32,
    );
    fn pushDoubleSidedTessellationSpans(
        &mut self,
        pts: &[Vec2D; 4],
        join_tangent: Vec2D,
        total_vertex_count: u32,
        parametric_segment_count: u32,
        polar_segment_count: u32,
        join_segment_count: u32,
        contour_id_with_flags: u32,
    );
}
