/*
 * Copyright 2022 Rive
 */

// Mechanical translation of the complete pinned source implementation
// renderer/src/render_context.cpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// The complete pinned C++ source is retained below in source order. Every
// declaration, nested implementation, branch, preprocessor path, assertion,
// side effect, and source quirk remains visible in this source-shaped Rust
// owner. Executable correspondence follows the retained source in the same
// function order and is wired through the source-shaped contracts below.

// /*
//  * Copyright 2022 Rive
//  */
//
// #include "rive/renderer/render_context.hpp"
//
// #include "gr_inner_fan_triangulator.hpp"
// #include "intersection_board.hpp"
// #include "gradient.hpp"
// #include "rive_render_paint.hpp"
// #include "rive/renderer/draw.hpp"
// #ifdef RIVE_CANVAS
// #include "rive/renderer/render_canvas.hpp"
// #endif
// #include "rive/renderer/rive_render_image.hpp"
// #include "rive/renderer/render_context_impl.hpp"
// #include "rive/gpu_texture_format.hpp"
// #include "rive/renderer/stack_vector.hpp"
// #include "rive/profiler/profiler_macros.h"
//
// #include "shaders/constants.glsl"
//
// #include <string_view>
//
// #ifdef RIVE_DECODERS
// #include "rive/decoders/bitmap_decoder.hpp"
// #endif
//
// #ifdef RIVE_KTX2
// #include "rive/decoders/decode_ktx2.hpp"
// #endif
//
// #include "sort_key_builder.hpp"
//
// namespace rive::gpu
// {
// constexpr size_t kDefaultSimpleGradientCapacity = 512;
// constexpr size_t kDefaultComplexGradientCapacity = 1024;
// constexpr size_t kDefaultDrawCapacity = 2048;
//
// // TODO: Move this variable to PlatformFeatures.
// constexpr uint32_t kMaxTextureHeight = 2048;
// constexpr size_t kMaxTessellationVertexCount =
//     kMaxTextureHeight * kTessTextureWidth;
// constexpr size_t kMaxTessellationPaddingVertexCount =
//     gpu::kMidpointFanPatchSegmentSpan + // Padding at the beginning of the tess
//                                         // texture
//     (gpu::kOuterCurvePatchSegmentSpan -
//      1) + // Max padding between patch types in the tess texture
//     1;    // Padding at the end of the tessellation texture
// constexpr size_t kMaxTessellationVertexCountBeforePadding =
//     kMaxTessellationVertexCount - kMaxTessellationPaddingVertexCount;
//
// // Metal requires vertex buffers to be 256-byte aligned.
// constexpr size_t kMaxTessellationAlignmentVertices =
//     gpu::kTessVertexBufferAlignmentInElements - 1;
//
// // We can only reorder 32767 draws at a time since the one-based groupIndex
// // returned by IntersectionBoard is a signed 16-bit integer.
// constexpr size_t kMaxReorderedDrawPassCount =
//     std::numeric_limits<int16_t>::max();
//
// // How tall to make a resource texture in order to support the given number of
// // items.
// template <size_t WidthInItems>
// constexpr static size_t resource_texture_height(size_t itemCount)
// {
//     return (itemCount + WidthInItems - 1) / WidthInItems;
// }
//
// constexpr static size_t gradient_data_height(size_t simpleRampCount,
//                                              size_t complexRampCount)
// {
//     return resource_texture_height<gpu::kGradTextureWidthInSimpleRamps>(
//                simpleRampCount) +
//            complexRampCount;
// }
//
// // Returns true if the current bounds poke outside of the containing bounds (and
// // thus would need to be clipped against them)
// inline bool needsScissor(IAABB currentBounds,
//                          IAABB containingBounds,
//                          uint32_t renderTargetWidth,
//                          uint32_t renderTargetHeight)
// {
//     // intersect the current bounds with the screen dimensions before testing so
//     // that if we end up outside the containing bounds on a side that is also a
//     // screen edge it doesn't matter.
//     return !containingBounds.contains(currentBounds.intersect(
//         IAABB::MakeWH(renderTargetWidth, renderTargetHeight)));
// }
//
// inline GradientContentKey::GradientContentKey(rcp<const Gradient> gradient) :
//     m_gradient(std::move(gradient))
// {}
//
// inline GradientContentKey::GradientContentKey(GradientContentKey&& other) :
//     m_gradient(std::move(other.m_gradient))
// {}
//
// bool GradientContentKey::operator==(const GradientContentKey& other) const
// {
//     if (m_gradient.get() == other.m_gradient.get())
//     {
//         return true;
//     }
//     else
//     {
//         return m_gradient->count() == other.m_gradient->count() &&
//                !memcmp(m_gradient->stops(),
//                        other.m_gradient->stops(),
//                        m_gradient->count() * sizeof(float)) &&
//                !memcmp(m_gradient->colors(),
//                        other.m_gradient->colors(),
//                        m_gradient->count() * sizeof(ColorInt));
//     }
// }
//
// size_t DeepHashGradient::operator()(const GradientContentKey& key) const
// {
//     const Gradient* grad = key.gradient();
//     std::hash<std::string_view> hash;
//     size_t x =
//         hash(std::string_view(reinterpret_cast<const char*>(grad->stops()),
//                               grad->count() * sizeof(float)));
//     size_t y =
//         hash(std::string_view(reinterpret_cast<const char*>(grad->colors()),
//                               grad->count() * sizeof(ColorInt)));
//     return x ^ y;
// }
//
// RenderContext::RenderContext(std::unique_ptr<RenderContextImpl> impl) :
//     m_impl(std::move(impl)),
//     // -1 from m_maxPathID so we reserve a path record for the clearColor paint
//     // (for atomic mode). This also allows us to index the storage buffers
//     // directly by pathID.
//     m_maxPathID(MaxPathID(m_impl->platformFeatures().pathIDGranularity) - 1)
// {
//     // Validate platformFeatures: if supportsBlendAdvancedCoherentKHR is set,
//     // supportsBlendAdvancedKHR must also be.
//     assert(!m_impl->platformFeatures().supportsBlendAdvancedCoherentKHR ||
//            m_impl->platformFeatures().supportsBlendAdvancedKHR);
//
// #ifdef RIVE_GENERATE_FEATHER_LUT
//     float table[GAUSSIAN_TABLE_SIZE];
//     generate_gausian_integral_table(table);
//     generate_inverse_gausian_integral_table(table);
// #endif
//
//     setResourceSizes(ResourceAllocationCounts(), /*forceRealloc =*/true);
//     releaseResources();
// }
//
// RenderContext::~RenderContext()
// {
//     // Always call flush() to avoid deadlock.
//     assert(!m_didBeginFrame);
//     // Delete the logical flushes before the block allocators let go of their
//     // allocations.
//     m_logicalFlushes.clear();
// #ifdef RIVE_CANVAS
//     m_oreContext.reset();
// #endif
// }
//
// const gpu::PlatformFeatures& RenderContext::platformFeatures() const
// {
//     return m_impl->platformFeatures();
// }
//
// rcp<RenderBuffer> RenderContext::makeRenderBuffer(RenderBufferType type,
//                                                   RenderBufferFlags flags,
//                                                   size_t sizeInBytes)
// {
//     return m_impl->makeRenderBuffer(type, flags, sizeInBytes);
// }
//
// #ifdef RIVE_CANVAS
// rcp<RenderCanvas> RenderContext::makeRenderCanvas(uint32_t width,
//                                                   uint32_t height)
// {
//     return m_impl->makeRenderCanvas(width, height);
// }
// rive::ore::Context* RenderContext::ore()
// {
//     if (m_oreContext == nullptr)
//         m_oreContext = m_impl->makeOreContext();
//     return m_oreContext.get();
// }
// #endif
//
// rcp<RenderImage> RenderContext::decodeImage(Span<const uint8_t> encodedBytes)
// {
//     RIVE_PROF_SCOPE_L(1)
//     rcp<Texture> texture = m_impl->platformDecodeImageTexture(encodedBytes);
//
// #ifdef RIVE_KTX2
//     // KTX2 magic = «KTX 20»\r\n\x1A\n. Match the first 4 bytes for the cheap
//     // dispatch; full magic is re-checked inside DecodeKtx2.
//     if (texture == nullptr && encodedBytes.size() >= 12 &&
//         encodedBytes[0] == 0xAB && encodedBytes[1] == 0x4B &&
//         encodedBytes[2] == 0x54 && encodedBytes[3] == 0x58)
//     {
//         const Ktx2HwSupport hwSupport = {
//             platformFeatures().supportsTextureCompressionBC,
//             platformFeatures().supportsTextureCompressionASTC,
//             platformFeatures().supportsTextureCompressionETC2,
//         };
//         Ktx2DecodeResult ktx2;
//         if (DecodeKtx2(encodedBytes.data(),
//                        encodedBytes.size(),
//                        ktx2,
//                        hwSupport))
//         {
//             // KTX2 provides the full level chain (or just level 0). The
//             // backends never auto-generate; whatever the file ships with is
//             // exactly what gets uploaded.
//             texture = m_impl->makeImageTexture(ktx2.pixelWidth,
//                                                ktx2.pixelHeight,
//                                                ktx2.levelCount,
//                                                ktx2.format,
//                                                ktx2.blocks.data(),
//                                                ktx2.blockWidth,
//                                                ktx2.blockHeight,
//                                                ktx2.srgb);
//         }
//     }
// #endif
//
// #ifdef RIVE_DECODERS
//     if (texture == nullptr)
//     {
//         auto bitmap = Bitmap::decode(encodedBytes.data(), encodedBytes.size());
//         if (bitmap)
//         {
//             // Bitmap::decode always produces RGBA — convert if needed.
//             if (bitmap->pixelFormat() != Bitmap::PixelFormat::RGBAPremul)
//             {
//                 bitmap->pixelFormat(Bitmap::PixelFormat::RGBAPremul);
//             }
//             uint32_t width = bitmap->width();
//             uint32_t height = bitmap->height();
//             uint32_t mipLevelCount = math::msb(height | width);
//             texture = m_impl->makeImageTexture(width,
//                                                height,
//                                                mipLevelCount,
//                                                GPUTextureFormat::rgba32,
//                                                bitmap->bytes(),
//                                                /*blockWidth=*/1,
//                                                /*blockHeight=*/1,
//                                                /*srgb=*/false,
//                                                /*generateRemainingMips=*/true);
//         }
//     }
// #endif
//     return texture != nullptr ? make_rcp<RiveRenderImage>(std::move(texture))
//                               : nullptr;
// }
//
// void RenderContext::releaseResources()
// {
//     assert(!m_didBeginFrame);
//     resetContainers();
//     setResourceSizes(ResourceAllocationCounts());
//     m_maxRecentResourceRequirements = ResourceAllocationCounts();
//     m_lastResourceTrimTimeInSeconds = m_impl->secondsNow();
// }
//
// void RenderContext::resetContainers()
// {
//     assert(!m_didBeginFrame);
//
//     if (!m_logicalFlushes.empty())
//     {
//         // Should get reset to 1 after flush().
//         assert(m_logicalFlushes.size() == 1);
//         m_logicalFlushes.resize(1);
//         m_logicalFlushes.front()->resetContainers();
//     }
//
//     m_indirectDrawList.clear();
//     m_indirectDrawList.shrink_to_fit();
//
//     m_intersectionBoard = nullptr;
// }
//
// RenderContext::LogicalFlush::LogicalFlush(RenderContext* parent) : m_ctx(parent)
// {
//     rewind();
// }
//
// void RenderContext::LogicalFlush::rewind()
// {
//     RIVE_PROF_SCOPE_L(1)
//     m_resourceCounts = Draw::ResourceCounters();
//     m_drawPassCount = 0;
//     m_simpleGradients.clear();
//     m_pendingSimpleGradDraws.clear();
//     m_complexGradients.clear();
//     m_pendingComplexGradDraws.clear();
//     m_pendingGradSpanCount = 0;
//     m_clips.clear();
//     m_draws.clear();
//     m_combinedDrawBounds = IAABB::makeMaximallyNegative();
//     m_combinedDrawContents = gpu::DrawContents::none;
//
//     m_pathPaddingCount = 0;
//     m_paintPaddingCount = 0;
//     m_paintAuxPaddingCount = 0;
//     m_contourPaddingCount = 0;
//     m_gradSpanPaddingCount = 0;
//     m_midpointFanTessEndLocation = 0;
//     m_outerCubicTessEndLocation = 0;
//     m_outerCubicTessVertexIdx = 0;
//     m_midpointFanTessVertexIdx = 0;
//     m_baselineShaderMiscFlags = gpu::ShaderMiscFlags::none;
//
//     m_flushDesc = FlushDescriptor();
//
//     m_drawList.reset();
//     m_firstDstBlendBarrier = nullptr;
//     m_dstBlendBarrierListTail = &m_firstDstBlendBarrier;
//     m_combinedShaderFeatures = gpu::ShaderFeatures::NONE;
//
//     m_currentPathID = 0;
//     m_currentContourID = 0;
//
//     if (m_featherAtlasRectanizer != nullptr)
//     {
//         m_featherAtlasRectanizer->reset();
//     }
//     m_featherAtlasMaxX = 0;
//     m_featherAtlasMaxY = 0;
//     m_pendingFeatherAtlasDraws.clear();
//
//     m_coverageBufferLength = 0;
//
//     m_pendingBarriers = BarrierFlags::none;
//
//     m_currentZIndex = 0;
//
//     RIVE_DEBUG_CODE(m_hasDoneLayout = false;)
// }
//
// void RenderContext::LogicalFlush::resetContainers()
// {
//     m_clips.clear();
//     m_clips.shrink_to_fit();
//     m_draws.clear();
//     m_draws.shrink_to_fit();
//     m_draws.reserve(kDefaultDrawCapacity);
//
//     m_simpleGradients.rehash(0);
//     m_simpleGradients.reserve(kDefaultSimpleGradientCapacity);
//
//     m_pendingSimpleGradDraws.clear();
//     m_pendingSimpleGradDraws.shrink_to_fit();
//     m_pendingSimpleGradDraws.reserve(kDefaultSimpleGradientCapacity);
//
//     m_complexGradients.rehash(0);
//     m_complexGradients.reserve(kDefaultComplexGradientCapacity);
//
//     m_pendingComplexGradDraws.clear();
//     m_pendingComplexGradDraws.shrink_to_fit();
//     m_pendingComplexGradDraws.reserve(kDefaultComplexGradientCapacity);
//
//     m_pendingFeatherAtlasDraws.clear();
//     m_pendingFeatherAtlasDraws.shrink_to_fit();
//     // Don't reserve any space in m_pendingFeatherAtlasDraws since there are
//     // many usecases where it isn't used at all.
// }
//
// static gpu::InterlockMode select_interlock_mode(
//     const RenderContext::FrameDescriptor& frameDescriptor,
//     const gpu::PlatformFeatures& platformFeatures)
// {
//     if (frameDescriptor.msaaSampleCount != 0)
//     {
//         return gpu::InterlockMode::msaa;
//     }
//     if (frameDescriptor.clockwiseFillOverride)
//     {
//         if (platformFeatures.supportsClockwiseMode &&
//             !frameDescriptor.disableRasterOrdering)
//         {
//             return gpu::InterlockMode::clockwise;
//         }
//         if (platformFeatures.supportsClockwiseAtomicMode)
//         {
//             return gpu::InterlockMode::clockwiseAtomic;
//         }
//     }
//     if (platformFeatures.supportsRasterOrderingMode &&
//         (!frameDescriptor.disableRasterOrdering ||
//          // Only respect "disableRasterOrdering" if we have atomic mode to fall
//          // back on.
//          // FIXME: This API can be improved.
//          !platformFeatures.supportsAtomicMode))
//     {
//         return gpu::InterlockMode::rasterOrdering;
//     }
//     if (platformFeatures.supportsAtomicMode)
//     {
//         return gpu::InterlockMode::atomics;
//     }
//     return gpu::InterlockMode::msaa;
// }
//
// void RenderContext::beginFrame(const FrameDescriptor& frameDescriptor)
// {
//     RIVE_PROF_SCOPE_L(0)
//
//     m_impl->preBeginFrame(this);
//     assert(!m_didBeginFrame);
//     assert(frameDescriptor.renderTargetWidth > 0);
//     assert(frameDescriptor.renderTargetHeight > 0);
//     m_frameDescriptor = frameDescriptor;
//     m_frameInterlockMode =
//         select_interlock_mode(m_frameDescriptor, platformFeatures());
//     if (m_frameInterlockMode == gpu::InterlockMode::msaa &&
//         m_frameDescriptor.msaaSampleCount == 0)
//     {
//         // Use 4x MSAA if msaaSampleCount wasn't already specified.
//         m_frameDescriptor.msaaSampleCount = 4;
//     }
//     m_frameShaderFeaturesMask =
//         gpu::ShaderFeaturesMaskFor(m_frameInterlockMode);
//     if (m_logicalFlushes.empty())
//     {
//         m_logicalFlushes.emplace_back(new LogicalFlush(this));
//     }
//     RIVE_DEBUG_CODE(m_didBeginFrame = true);
// }
//
// bool RenderContext::isOutsideCurrentFrame(const IAABB& pixelBounds)
// {
//     assert(m_didBeginFrame);
//     int4 bounds = simd::load4i(&pixelBounds);
//     auto renderTargetSize =
//         simd::cast<int32_t>(uint2{m_frameDescriptor.renderTargetWidth,
//                                   m_frameDescriptor.renderTargetHeight});
//     return simd::any(bounds.xy >= renderTargetSize || bounds.zw <= 0 ||
//                      bounds.xy >= bounds.zw);
// }
//
// bool RenderContext::frameSupportsClipRects() const
// {
//     assert(m_didBeginFrame);
//     return m_frameInterlockMode != gpu::InterlockMode::msaa ||
//            platformFeatures().supportsClipPlanes;
// }
//
// bool RenderContext::frameSupportsImagePaintForPaths() const
// {
//     assert(m_didBeginFrame);
//     return m_frameInterlockMode != gpu::InterlockMode::atomics;
// }
//
// uint32_t RenderContext::generateClipID(IAABB contentBounds,
//                                        uint32_t parentClipID,
//                                        AABBu16 tightenedBounds)
// {
//     assert(m_didBeginFrame);
//     assert(!m_logicalFlushes.empty());
//     return m_logicalFlushes.back()->generateClipID(contentBounds,
//                                                    parentClipID,
//                                                    tightenedBounds);
// }
//
// uint32_t RenderContext::LogicalFlush::generateClipID(IAABB contentBounds,
//                                                      uint32_t parentClipID,
//                                                      AABBu16 tightenedBounds)
// {
//     if (m_clips.size() < m_ctx->m_maxPathID) // maxClipID == maxPathID.
//     {
//         m_clips.emplace_back(contentBounds, parentClipID, tightenedBounds);
//         assert(m_ctx->m_clipContentID != m_clips.size());
//         return math::lossless_numeric_cast<uint32_t>(m_clips.size());
//     }
//     return 0; // There are no available clip IDs. The caller should flush and
//               // try again.
// }
//
// RenderContext::LogicalFlush::ClipInfo& RenderContext::LogicalFlush::
//     getWritableClipInfo(uint32_t clipID)
// {
//     assert(clipID > 0);
//     assert(clipID <= m_clips.size());
//     return m_clips[clipID - 1];
// }
//
// bool RenderContext::pushDraws(DrawUniquePtr draws[], size_t drawCount)
// {
//     assert(m_didBeginFrame);
//     assert(!m_logicalFlushes.empty());
//     return m_logicalFlushes.back()->pushDraws(draws, drawCount);
// }
//
// bool RenderContext::LogicalFlush::pushDraws(DrawUniquePtr draws[],
//                                             size_t drawCount)
// {
//     RIVE_PROF_SCOPE_L(1)
//     assert(!m_hasDoneLayout);
//
//     PUSH_DISABLE_CLANG_SIMD_ABI_WARNING()
//     auto countsVector = m_resourceCounts.toVec();
//     for (size_t i = 0; i < drawCount; ++i)
//     {
//         assert(!draws[i]->pixelBounds().empty());
//         assert(m_ctx->frameSupportsClipRects() ||
//                draws[i]->clipRectInverseMatrix() == nullptr);
//         countsVector += draws[i]->resourceCounts().toVec();
//     }
//     POP_DISABLE_CLANG_SIMD_ABI_WARNING()
//     Draw::ResourceCounters countsWithNewBatch = countsVector;
//
//     // Textures and buffers have hard size limits. If the new batch doesn't fit
//     // within our constraints, the caller needs to flush and try again.
//     if (countsWithNewBatch.pathCount > m_ctx->m_maxPathID ||
//         countsWithNewBatch.contourCount > kMaxContourID ||
//         countsWithNewBatch.midpointFanTessVertexCount +
//                 countsWithNewBatch.outerCubicTessVertexCount >
//             kMaxTessellationVertexCountBeforePadding)
//     {
//         return false;
//     }
//
//     // Allocate subpasses.
//     int passCountInBatch = 0;
//     for (size_t i = 0; i < drawCount; ++i)
//     {
//         draws[i]->countSubpasses(platformFeatures());
//         assert(draws[i]->prepassCount() >= 0);
//         assert(draws[i]->subpassCount() >= 0);
//         assert(draws[i]->prepassCount() + draws[i]->subpassCount() >= 1);
//         passCountInBatch += draws[i]->prepassCount() + draws[i]->subpassCount();
//     }
//
//     // We can only reorder 32k draws at a time in atomic and msaa modes since
//     // the sort key addresses them with a signed 16-bit index. Make sure we
//     // don't exceed that limit.
//     if (m_ctx->frameInterlockMode() != gpu::InterlockMode::rasterOrdering &&
//         m_drawPassCount + passCountInBatch > kMaxReorderedDrawPassCount)
//     {
//         return false;
//     }
//
//     // Allocate final resources.
//     for (size_t i = 0; i < drawCount; ++i)
//     {
//         if (!draws[i]->allocateResources(this))
//         {
//             // The draw failed to allocate resources. Give up and let the caller
//             // flush and try again.
//             //
//             // FIXME: This works today, but the surrounding code could be
//             // modified to inadvertently leave a stale dangling reference to one
//             // of these draws in m_pendingFeatherAtlasDraws. This needs to be
//             // revisited.
//             return false;
//         }
//     }
//
//     for (size_t i = 0; i < drawCount; ++i)
//     {
//         m_draws.push_back(std::move(draws[i]));
//         // Note: not updating m_combinedDrawBounds here because it will get done
//         // in tightenClipBounds later, after we've determined the minimal write
//         // sizes for any clips.
//         m_combinedDrawContents |= m_draws.back()->drawContents();
//     }
//
//     m_resourceCounts = countsWithNewBatch;
//     m_drawPassCount += passCountInBatch;
//     return true;
// }
//
// bool RenderContext::LogicalFlush::allocateGradient(
//     const Gradient* gradient,
//     gpu::ColorRampLocation* colorRampLocation)
// {
//     RIVE_PROF_SCOPE_L(2)
//     assert(!m_hasDoneLayout);
//
//     const float* stops = gradient->stops();
//     size_t stopCount = gradient->count();
//     assert(stopCount > 0); // RiveRenderFactory guarantees this.
//
//     if (stopCount == 1 || (stopCount == 2 && stops[0] == 0 && stops[1] == 1))
//     {
//         // This is a simple gradient that can be implemented by a two-texel
//         // color ramp.
//         const ColorInt* colors = gradient->colors();
//         TwoTexelRamp colorRamp = {colors[0],
//                                   // Handle ramps with a single stop.
//                                   colors[std::min<size_t>(1, stopCount - 1)]};
//         uint64_t simpleKey;
//         static_assert(sizeof(simpleKey) == sizeof(ColorInt) * 2);
//         RIVE_INLINE_MEMCPY(&simpleKey, &colorRamp, sizeof(ColorInt) * 2);
//         uint32_t rampTexelsIdx;
//         auto iter = m_simpleGradients.find(simpleKey);
//         if (iter != m_simpleGradients.end())
//         {
//             // This gradient is already in the texture.
//             rampTexelsIdx = iter->second;
//         }
//         else
//         {
//             if (gradient_data_height(m_simpleGradients.size() + 1,
//                                      m_complexGradients.size()) >
//                 kMaxTextureHeight)
//             {
//                 // We ran out of rows in the gradient texture. Caller has to
//                 // flush and try again.
//                 return false;
//             }
//             rampTexelsIdx = math::lossless_numeric_cast<uint32_t>(
//                 m_simpleGradients.size() * 2);
//             m_simpleGradients.insert({simpleKey, rampTexelsIdx});
//             m_pendingSimpleGradDraws.push_back(colorRamp);
//             // Simple gradients get uploaded to the GPU as a single GradientSpan
//             // instance.
//             ++m_pendingGradSpanCount;
//         }
//         colorRampLocation->row = rampTexelsIdx / kGradTextureWidth;
//         colorRampLocation->col = rampTexelsIdx % kGradTextureWidth;
//     }
//     else
//     {
//         // This is a complex gradient. Render it to an entire row of the
//         // gradient texture.
//         GradientContentKey key(ref_rcp(gradient));
//         auto iter = m_complexGradients.find(key);
//         uint16_t row;
//         if (iter != m_complexGradients.end())
//         {
//             row = iter->second; // This gradient is already in the texture.
//         }
//         else
//         {
//             if (gradient_data_height(m_simpleGradients.size(),
//                                      m_complexGradients.size() + 1) >
//                 kMaxTextureHeight)
//             {
//                 // We ran out of rows in the gradient texture. Caller has to
//                 // flush and try again.
//                 return false;
//             }
//
//             row = static_cast<uint32_t>(m_complexGradients.size());
//             m_complexGradients.emplace(std::move(key), row);
//             m_pendingComplexGradDraws.push_back(gradient);
//
//             size_t spanCount = stopCount - 1;
//             m_pendingGradSpanCount += spanCount;
//         }
//         // Store the row relative to the first complex gradient for now.
//         // PaintData::set() will offset this value by the number of simple
//         // gradient rows once its final value is known.
//         colorRampLocation->row = row;
//         colorRampLocation->col = ColorRampLocation::kComplexGradientMarker;
//     }
//     return true;
// }
//
// bool RenderContext::LogicalFlush::allocateFeatherAtlasDraw(
//     PathDraw* pathDraw,
//     uint16_t drawWidth,
//     uint16_t drawHeight,
//     uint16_t desiredPadding,
//     uint16_t* x,
//     uint16_t* y,
//     AABBu16* paddedRegion)
// {
//     RIVE_PROF_SCOPE_L(2)
//
//     if (m_featherAtlasRectanizer == nullptr)
//     {
//         uint16_t atlasMaxSize = m_ctx->featherAtlasMaxSize();
//         // Use an atlas larger than featherAtlasMaxSize if it's too small for
//         // the request (meaning the render target is larger than
//         // featherAtlasMaxSize).
//         m_featherAtlasRectanizer = std::make_unique<rive::RectanizerSkyline>(
//             std::max(atlasMaxSize, drawWidth),
//             std::max(atlasMaxSize, drawHeight));
//     }
//
//     const uint16_t atlasMaxWidth = m_featherAtlasRectanizer->width();
//     const uint16_t atlasMaxHeight = m_featherAtlasRectanizer->height();
//     uint16_t paddedWidth =
//         std::min<uint16_t>(drawWidth + desiredPadding * 2, atlasMaxWidth);
//     uint16_t paddedHeight =
//         std::min<uint16_t>(drawHeight + desiredPadding * 2, atlasMaxHeight);
//     int16_t ix, iy;
//     if (!m_featherAtlasRectanizer->addRect(paddedWidth, paddedHeight, &ix, &iy))
//     {
//         // Delete the rectanizer of it wasn't big enough for this path. It will
//         // be reallocated to a large enough size on the next call.
//         if (drawWidth > atlasMaxWidth || drawHeight > atlasMaxHeight)
//         {
//             m_featherAtlasRectanizer = nullptr;
//         }
//         m_featherAtlasRectanizer = nullptr;
//         return false;
//     }
//
//     assert(ix >= 0);
//     assert(iy >= 0);
//     assert(ix + paddedWidth <= atlasMaxWidth);
//     assert(iy + paddedHeight <= atlasMaxHeight);
//
//     *x = ix + (paddedWidth - drawWidth) / 2;
//     *y = iy + (paddedHeight - drawHeight) / 2;
//     *paddedRegion = {math::lossless_numeric_cast<uint16_t>(ix),
//                      math::lossless_numeric_cast<uint16_t>(iy),
//                      math::lossless_numeric_cast<uint16_t>(ix + paddedWidth),
//                      math::lossless_numeric_cast<uint16_t>(iy + paddedHeight)};
//     assert(
//         (AABBu16{0, 0, atlasMaxWidth, atlasMaxHeight}).contains(*paddedRegion));
//
//     m_featherAtlasMaxX =
//         std::max<uint32_t>(m_featherAtlasMaxX, paddedRegion->right);
//     m_featherAtlasMaxY =
//         std::max<uint32_t>(m_featherAtlasMaxY, paddedRegion->bottom);
//     assert(m_featherAtlasMaxX <= atlasMaxWidth);
//     assert(m_featherAtlasMaxY <= atlasMaxHeight);
//
//     m_pendingFeatherAtlasDraws.push_back(pathDraw);
//     return true;
// }
//
// size_t RenderContext::LogicalFlush::allocateCoverageBufferRange(size_t length)
// {
//     RIVE_PROF_SCOPE_L(2)
//     assert(m_ctx->frameInterlockMode() == gpu::InterlockMode::clockwiseAtomic);
//     // Allocations must be aligned to the tile size.
//     assert(length % (BUFFER_IMAGE_TILE_SIZE * BUFFER_IMAGE_TILE_SIZE) == 0u);
//     uint32_t offset = m_coverageBufferLength;
//     if (offset + length > m_ctx->platformFeatures().maxCoverageBufferLength)
//     {
//         return -1;
//     }
//     m_coverageBufferLength += length;
//     return offset;
// }
//
// void RenderContext::logicalFlush()
// {
//     assert(m_didBeginFrame);
//
//     // Reset clipping state after every logical flush because the clip buffer is
//     // not preserved between render passes.
//     m_clipContentID = 0;
//
//     // Don't issue any GPU commands between logical flushes. Instead, build up a
//     // list of flushes that we will submit all at once at the end of the frame.
//     m_logicalFlushes.emplace_back(new LogicalFlush(this));
// }
//
// void RenderContext::flush(const FlushResources& flushResources)
// {
//     RIVE_PROF_SCOPE_L(0)
//     assert(m_didBeginFrame);
//     assert(flushResources.renderTarget->width() ==
//            m_frameDescriptor.renderTargetWidth);
//     assert(flushResources.renderTarget->height() ==
//            m_frameDescriptor.renderTargetHeight);
//
//     m_clipContentID = 0;
//
//     // Layout this frame's resource buffers and textures.
//     LogicalFlush::ResourceCounters totalFrameResourceCounts;
//     LogicalFlush::LayoutCounters layoutCounts;
//     for (size_t i = 0; i < m_logicalFlushes.size(); ++i)
//     {
//         m_logicalFlushes[i]->layoutResources(flushResources,
//                                              i,
//                                              &totalFrameResourceCounts,
//                                              &layoutCounts);
//     }
//
//     // Determine the minimum required resource allocation sizes to service this
//     // flush.
//     const ResourceAllocationCounts resourceRequirements = {
//         .flushUniformBufferCount = m_logicalFlushes.size(),
//         .pathBufferCount =
//             totalFrameResourceCounts.pathCount + layoutCounts.pathPaddingCount,
//         .paintBufferCount =
//             totalFrameResourceCounts.pathCount + layoutCounts.paintPaddingCount,
//         .paintAuxBufferCount = totalFrameResourceCounts.pathCount +
//                                layoutCounts.paintAuxPaddingCount,
//         .contourBufferCount = totalFrameResourceCounts.contourCount +
//                               layoutCounts.contourPaddingCount,
//         .gradSpanBufferCount =
//             layoutCounts.gradSpanCount + layoutCounts.gradSpanPaddingCount,
//         .tessSpanBufferCount =
//             totalFrameResourceCounts.maxTessellatedSegmentCount,
//         .triangleVertexBufferCount =
//             totalFrameResourceCounts.maxTriangleVertexCount,
//         .imageDrawInstanceBufferCount = totalFrameResourceCounts.imageDrawCount,
//         .gradTextureHeight = layoutCounts.maxGradTextureHeight,
//         .tessTextureHeight = layoutCounts.maxTessTextureHeight,
//         .featherAtlasTextureWidth = layoutCounts.maxFeatherAtlasWidth,
//         .featherAtlasTextureHeight = layoutCounts.maxFeatherAtlasHeight,
//         .plsTransientBackingWidth =
//             (layoutCounts.maxPLSTransientBackingPlaneCount > 0)
//                 ? static_cast<size_t>(m_frameDescriptor.renderTargetWidth)
//                 : 0,
//         .plsTransientBackingHeight =
//             (layoutCounts.maxPLSTransientBackingPlaneCount > 0)
//                 ? static_cast<size_t>(m_frameDescriptor.renderTargetHeight)
//                 : 0,
//         .plsTransientBackingPlaneCount =
//             layoutCounts.maxPLSTransientBackingPlaneCount,
//         .plsAtomicCoverageBackingWidth =
//             (frameInterlockMode() == gpu::InterlockMode::atomics)
//                 ? static_cast<size_t>(m_frameDescriptor.renderTargetWidth)
//                 : 0,
//         .plsAtomicCoverageBackingHeight =
//             (frameInterlockMode() == gpu::InterlockMode::atomics)
//                 ? static_cast<size_t>(m_frameDescriptor.renderTargetHeight)
//                 : 0,
//         .coverageBufferLength = layoutCounts.maxCoverageBufferLength,
//     };
//
//     // Ensure we're within hardware limits.
//     assert(resourceRequirements.gradTextureHeight <= kMaxTextureHeight);
//     assert(resourceRequirements.tessTextureHeight <= kMaxTextureHeight);
//     assert(resourceRequirements.featherAtlasTextureWidth <=
//                featherAtlasMaxSize() ||
//            resourceRequirements.featherAtlasTextureWidth <=
//                frameDescriptor().renderTargetWidth);
//     assert(resourceRequirements.featherAtlasTextureHeight <=
//                featherAtlasMaxSize() ||
//            resourceRequirements.featherAtlasTextureHeight <=
//                frameDescriptor().renderTargetHeight);
//     assert(resourceRequirements.plsTransientBackingWidth <=
//            m_frameDescriptor.renderTargetWidth);
//     assert(resourceRequirements.plsTransientBackingHeight <=
//            m_frameDescriptor.renderTargetHeight);
//     assert(resourceRequirements.coverageBufferLength <=
//            platformFeatures().maxCoverageBufferLength);
//
//     PUSH_DISABLE_CLANG_SIMD_ABI_WARNING()
//
//     // Track m_maxRecentResourceRequirements so we can trim GPU allocations when
//     // steady-state usage goes down.
//     m_maxRecentResourceRequirements = ResourceAllocationCounts::FromVec(
//         simd::max(resourceRequirements.toVec(),
//                   m_maxRecentResourceRequirements.toVec()));
//
//     // Grow resources enough to handle this flush.
//     // If "allocs" already fits in our current allocations, then don't change
//     // them.
//     // If they don't fit, overallocate by the specified amount in order to
//     // create some slack for growth.
//     constexpr static ResourceAllocationCounts OVERALLOC_x4 = {
//         .flushUniformBufferCount = 5,        // 125%
//         .pathBufferCount = 5,                // 125%
//         .paintBufferCount = 5,               // 125%
//         .paintAuxBufferCount = 5,            // 125%
//         .contourBufferCount = 5,             // 125%
//         .gradSpanBufferCount = 5,            // 125%
//         .tessSpanBufferCount = 5,            // 125%
//         .triangleVertexBufferCount = 5,      // 125%
//         .imageDrawInstanceBufferCount = 5,   // 125%
//         .gradTextureHeight = 5,              // 125%
//         .tessTextureHeight = 5,              // 125%
//         .featherAtlasTextureWidth = 5,       // 125%
//         .featherAtlasTextureHeight = 5,      // 125%
//         .plsTransientBackingWidth = 4,       // 100% (i.e., don't overallocate)
//         .plsTransientBackingHeight = 4,      // 100% (i.e., don't overallocate)
//         .plsTransientBackingPlaneCount = 4,  // 100% (i.e., don't overallocate)
//         .plsAtomicCoverageBackingWidth = 4,  // 100% (i.e., don't overallocate)
//         .plsAtomicCoverageBackingHeight = 4, // 100% (i.e., don't overallocate)
//         .coverageBufferLength = 5,           // 125%
//     };
//     ResourceAllocationCounts allocs =
//         ResourceAllocationCounts::FromVec(simd::if_then_else(
//             resourceRequirements.toVec() <=
//                 m_currentResourceAllocations.toVec(),
//             m_currentResourceAllocations.toVec(),
//             (resourceRequirements.toVec() * OVERALLOC_x4.toVec()) >> 2));
//
//     // In case the 25% growth pushed us above limits.
//     allocs.gradTextureHeight =
//         std::min<size_t>(allocs.gradTextureHeight, kMaxTextureHeight);
//     allocs.tessTextureHeight =
//         std::min<size_t>(allocs.tessTextureHeight, kMaxTextureHeight);
//     allocs.featherAtlasTextureWidth = std::min<size_t>(
//         allocs.featherAtlasTextureWidth,
//         std::max(featherAtlasMaxSize(), frameDescriptor().renderTargetWidth));
//     allocs.featherAtlasTextureHeight = std::min<size_t>(
//         allocs.featherAtlasTextureHeight,
//         std::max(featherAtlasMaxSize(), frameDescriptor().renderTargetHeight));
//     allocs.coverageBufferLength =
//         std::min(allocs.coverageBufferLength,
//                  platformFeatures().maxCoverageBufferLength);
//
//     // Additionally, every 5 seconds, trim resources down to the most recent
//     // steady-state usage.
//     double flushTime = m_impl->secondsNow();
//     bool needsResourceTrim = flushTime - m_lastResourceTrimTimeInSeconds >= 5;
//     if (needsResourceTrim)
//     {
//         // Trim GPU resource allocations to their maximum recent usage, plus
//         // overallocation, and only if the recent usage is below a certain
//         // threshold.
//         constexpr static ResourceAllocationCounts SHRINK_THRESHOLD_x3 = {
//             .flushUniformBufferCount = 2,        // 66.7%
//             .pathBufferCount = 2,                // 66.7%
//             .paintBufferCount = 2,               // 66.7%
//             .paintAuxBufferCount = 2,            // 66.7%
//             .contourBufferCount = 2,             // 66.7%
//             .gradSpanBufferCount = 2,            // 66.7%
//             .tessSpanBufferCount = 2,            // 66.7%
//             .triangleVertexBufferCount = 2,      // 66.7%
//             .imageDrawInstanceBufferCount = 2,   // 66.7%
//             .gradTextureHeight = 2,              // 66.7%
//             .tessTextureHeight = 2,              // 66.7%
//             .featherAtlasTextureWidth = 2,       // 66.7%
//             .featherAtlasTextureHeight = 2,      // 66.7%
//             .plsTransientBackingWidth = 3,       // 100% (i.e., always shrink)
//             .plsTransientBackingHeight = 3,      // 100% (i.e., always shrink)
//             .plsTransientBackingPlaneCount = 3,  // 100% (i.e., always shrink)
//             .plsAtomicCoverageBackingWidth = 3,  // 100% (i.e., always shrink)
//             .plsAtomicCoverageBackingHeight = 3, // 100% (i.e., always shrink)
//             .coverageBufferLength = 2,           // 66.7%
//         };
//         allocs = ResourceAllocationCounts::FromVec(simd::if_then_else(
//             m_maxRecentResourceRequirements.toVec() <=
//                 (allocs.toVec() * SHRINK_THRESHOLD_x3.toVec()) / size_t(3),
//             // TODO: Do we actually need overallocation here?? Or should we just
//             // trust the past 5 seconds of steady usage?
//             (m_maxRecentResourceRequirements.toVec() * OVERALLOC_x4.toVec()) >>
//                 2,
//             allocs.toVec()));
//
//         // Ensure we stayed within limits.
//         assert(allocs.gradTextureHeight <= kMaxTextureHeight);
//         assert(allocs.tessTextureHeight <= kMaxTextureHeight);
//         assert(allocs.featherAtlasTextureWidth <= featherAtlasMaxSize() ||
//                allocs.featherAtlasTextureWidth <=
//                    frameDescriptor().renderTargetWidth);
//         assert(allocs.featherAtlasTextureHeight <= featherAtlasMaxSize() ||
//                allocs.featherAtlasTextureHeight <=
//                    frameDescriptor().renderTargetHeight);
//         assert(allocs.coverageBufferLength <=
//                platformFeatures().maxCoverageBufferLength);
//
//         // Zero out m_maxRecentResourceRequirements for the next interval.
//         m_maxRecentResourceRequirements = ResourceAllocationCounts();
//         m_lastResourceTrimTimeInSeconds = flushTime;
//     }
//
//     assert(simd::all(allocs.toVec() >= resourceRequirements.toVec()));
//     POP_DISABLE_CLANG_SIMD_ABI_WARNING()
//
//     setResourceSizes(allocs);
//
//     m_impl->prepareToFlush(flushResources.currentFrameNumber,
//                            flushResources.safeFrameNumber);
//     if (mapResourceBuffers(resourceRequirements))
//     {
//         for (const auto& flush : m_logicalFlushes)
//         {
//             flush->writeResources();
//         }
//
//         assert(m_flushUniformData.elementsWritten() == m_logicalFlushes.size());
//         assert(m_imageDrawInstanceData.elementsWritten() ==
//                totalFrameResourceCounts.imageDrawCount);
//         assert(m_pathData.elementsWritten() ==
//                totalFrameResourceCounts.pathCount +
//                    layoutCounts.pathPaddingCount);
//         assert(m_paintData.elementsWritten() ==
//                totalFrameResourceCounts.pathCount +
//                    layoutCounts.paintPaddingCount);
//         assert(m_paintAuxData.elementsWritten() ==
//                totalFrameResourceCounts.pathCount +
//                    layoutCounts.paintAuxPaddingCount);
//         assert(m_contourData.elementsWritten() ==
//                totalFrameResourceCounts.contourCount +
//                    layoutCounts.contourPaddingCount);
//         assert(m_gradSpanData.elementsWritten() ==
//                layoutCounts.gradSpanCount + layoutCounts.gradSpanPaddingCount);
//         assert(m_tessSpanData.elementsWritten() <=
//                totalFrameResourceCounts.maxTessellatedSegmentCount);
//         assert(m_triangleVertexData.elementsWritten() <=
//                totalFrameResourceCounts.maxTriangleVertexCount);
//
//         unmapResourceBuffers(resourceRequirements);
//
//         // Issue logical flushes to the backend.
//         for (const auto& flush : m_logicalFlushes)
//         {
//             m_impl->flush(flush->desc());
//         }
//     }
//     else
//     {
//         fprintf(stderr, "Buffer mapping failed, cannot render.\n");
//         unmapResourceBuffers(resourceRequirements);
//     }
//
//     m_impl->postFlush(flushResources);
//
//     if (!m_logicalFlushes.empty())
//     {
//         m_logicalFlushes.resize(1);
//         m_logicalFlushes.front()->rewind();
//     }
//
//     // Drop all memory that was allocated for this frame using
//     // TrivialBlockAllocator.
//     m_perFrameAllocator.reset();
//     m_numChopsAllocator.reset();
//     m_chopVerticesAllocator.reset();
//     m_tangentPairsAllocator.reset();
//     m_polarSegmentCountsAllocator.reset();
//     m_parametricSegmentCountsAllocator.reset();
//
//     m_frameDescriptor = FrameDescriptor();
//
//     RIVE_DEBUG_CODE(m_didBeginFrame = false;)
//
//     // Wait to reset CPU-side containers until after the flush has finished.
//     if (needsResourceTrim)
//     {
//         resetContainers();
//     }
// }
//
// static uint32_t pls_transient_backing_plane_count(
//     gpu::InterlockMode interlockMode,
//     gpu::DrawContents combinedDrawContents)
// {
//     switch (interlockMode)
//     {
//         case gpu::InterlockMode::rasterOrdering:
//             return 3; // clip, scratch, coverage
//         case gpu::InterlockMode::atomics:
//         case gpu::InterlockMode::clockwiseAtomic:
//             return 1; // only clip (coverage is atomic)
//         case gpu::InterlockMode::clockwise:
//         {
//             uint32_t n = 1; // coverage
//             if (enums::any_flag_set(combinedDrawContents,
//                                     gpu::DrawContents::activeClip |
//                                         gpu::DrawContents::clipUpdate))
//             {
//                 ++n; // clip
//             }
//             if (enums::is_flag_set(combinedDrawContents,
//                                    gpu::DrawContents::advancedBlend))
//             {
//                 ++n; // scratch color
//             }
//             return n;
//         }
//         case gpu::InterlockMode::msaa:
//             return 0; // N/A
//     }
//     RIVE_UNREACHABLE();
// }
//
// static bool wants_fixed_function_color_output(
//     const gpu::PlatformFeatures& platformFeatures,
//     gpu::InterlockMode interlockMode,
//     gpu::DrawContents combinedDrawContents,
//     bool manuallyResolved)
// {
//     switch (interlockMode)
//     {
//         case gpu::InterlockMode::rasterOrdering:
//             // rasterOrdering shaders always read the framebuffer, even with
//             // srcOver blend.
//             return false;
//
//         case gpu::InterlockMode::atomics:
//         case gpu::InterlockMode::clockwiseAtomic:
//             return !enums::is_flag_set(combinedDrawContents,
//                                        gpu::DrawContents::advancedBlend);
//
//         case gpu::InterlockMode::clockwise:
//             assert(enums::no_flags_set(combinedDrawContents,
//                                        gpu::DrawContents::nonZeroFill |
//                                            gpu::DrawContents::evenOddFill));
//             return platformFeatures.supportsClockwiseFixedFunctionMode &&
//                    !enums::is_flag_set(combinedDrawContents,
//                                        gpu::DrawContents::advancedBlend);
//
//         case gpu::InterlockMode::msaa:
//             // Manual MSAA resolves read the framebuffer, so they can't use
//             // fixedFunctionColorOutput.
//             return !manuallyResolved &&
//                    !enums::is_flag_set(combinedDrawContents,
//                                        gpu::DrawContents::advancedBlend);
//     }
//
//     RIVE_UNREACHABLE();
// }
//
// void RenderContext::LogicalFlush::layoutResources(
//     const FlushResources& flushResources,
//     size_t logicalFlushIdx,
//     ResourceCounters* runningFrameResourceCounts,
//     LayoutCounters* runningFrameLayoutCounts)
// {
//     RIVE_PROF_SCOPE_L(1)
//     assert(!m_hasDoneLayout);
//
//     const FrameDescriptor& frameDescriptor = m_ctx->frameDescriptor();
//
//     // Reserve a path record for the clearColor paint (used by atomic mode).
//     // This also allows us to index the storage buffers directly by pathID.
//     ++m_resourceCounts.pathCount;
//
//     // Storage buffer offsets are required to be aligned on multiples of 256.
//     m_pathPaddingCount =
//         math::padding_to_align_up<gpu::kPathBufferAlignmentInElements>(
//             m_resourceCounts.pathCount);
//     m_paintPaddingCount =
//         math::padding_to_align_up<gpu::kPaintBufferAlignmentInElements>(
//             m_resourceCounts.pathCount);
//     m_paintAuxPaddingCount =
//         math::padding_to_align_up<gpu::kPaintAuxBufferAlignmentInElements>(
//             m_resourceCounts.pathCount);
//     m_contourPaddingCount =
//         math::padding_to_align_up<gpu::kContourBufferAlignmentInElements>(
//             m_resourceCounts.contourCount);
//
//     // Metal requires vertex buffers to be 256-byte aligned.
//     m_gradSpanPaddingCount =
//         math::padding_to_align_up<gpu::kGradSpanBufferAlignmentInElements>(
//             m_pendingGradSpanCount);
//
//     size_t totalTessVertexCountWithPadding = 0;
//     if ((m_resourceCounts.midpointFanTessVertexCount |
//          m_resourceCounts.outerCubicTessVertexCount) != 0)
//     {
//         // midpointFan tessellation vertices reside at the beginning of the
//         // tessellation texture, after 1 patch of padding vertices.
//         constexpr uint32_t kPrePadding = gpu::kMidpointFanPatchSegmentSpan;
//         m_midpointFanTessVertexIdx = kPrePadding;
//         m_midpointFanTessEndLocation =
//             m_midpointFanTessVertexIdx +
//             math::lossless_numeric_cast<uint32_t>(
//                 m_resourceCounts.midpointFanTessVertexCount);
//
//         // outerCubic tessellation vertices reside after the midpointFan
//         // vertices, aligned on a multiple of the outerCubic patch size.
//         uint32_t interiorPadding =
//             math::padding_to_align_up<gpu::kOuterCurvePatchSegmentSpan>(
//                 m_midpointFanTessEndLocation);
//         m_outerCubicTessVertexIdx =
//             m_midpointFanTessEndLocation + interiorPadding;
//         m_outerCubicTessEndLocation =
//             m_outerCubicTessVertexIdx +
//             math::lossless_numeric_cast<uint32_t>(
//                 m_resourceCounts.outerCubicTessVertexCount);
//
//         // We need one more padding vertex after all the tessellation vertices.
//         constexpr uint32_t kPostPadding = 1;
//         totalTessVertexCountWithPadding =
//             m_outerCubicTessEndLocation + kPostPadding;
//
//         assert(kPrePadding + interiorPadding + kPostPadding <=
//                kMaxTessellationPaddingVertexCount);
//         assert(totalTessVertexCountWithPadding <= kMaxTessellationVertexCount);
//     }
//
//     uint32_t tessDataHeight = math::lossless_numeric_cast<uint32_t>(
//         resource_texture_height<kTessTextureWidth>(
//             totalTessVertexCountWithPadding));
//     if (m_resourceCounts.maxTessellatedSegmentCount != 0)
//     {
//         // Conservatively account for line breaks and padding in the
//         // tessellation span count. Line breaks potentially introduce a new
//         // span. Count the maximum number of line breaks we might encounter,
//         // which is at most TWO for every line in the tessellation texture (one
//         // for a forward span, and one for its reflection.)
//         size_t maxSpanBreakCount = tessDataHeight * 2;
//         // The tessellation texture requires 3 separate spans of padding
//         // vertices (see above and below).
//         constexpr size_t kPaddingSpanCount = 3;
//         m_resourceCounts.maxTessellatedSegmentCount +=
//             maxSpanBreakCount + kPaddingSpanCount +
//             kMaxTessellationAlignmentVertices;
//     }
//
//     // Complex gradients begin on the first row immediately after the simple
//     // gradients.
//     m_gradTextureLayout.complexOffsetY = math::lossless_numeric_cast<uint32_t>(
//         resource_texture_height<gpu::kGradTextureWidthInSimpleRamps>(
//             m_simpleGradients.size()));
//
//     m_flushDesc.renderTarget = flushResources.renderTarget;
//     m_flushDesc.interlockMode = m_ctx->frameInterlockMode();
//     m_flushDesc.msaaSampleCount = frameDescriptor.msaaSampleCount;
//
//     // In atomic mode, we may be able to skip the explicit clear of the color
//     // buffer and fold it into the atomic "resolve" operation instead.
//     bool doClearDuringAtomicResolve = false;
//
//     if (logicalFlushIdx != 0)
//     {
//         // We always have to preserve the renderTarget between logical flushes.
//         m_flushDesc.colorLoadAction = gpu::LoadAction::preserveRenderTarget;
//     }
//     else if (frameDescriptor.loadAction == gpu::LoadAction::clear)
//     {
//         // In atomic mode, we can clear during the resolve operation if the
//         // clearColor is opaque (because we don't want or have a "source only"
//         // blend mode).
//         doClearDuringAtomicResolve =
//             m_ctx->frameInterlockMode() == gpu::InterlockMode::atomics &&
//             colorAlpha(frameDescriptor.clearColor) == 255;
//         m_flushDesc.colorLoadAction = doClearDuringAtomicResolve
//                                           ? gpu::LoadAction::dontCare
//                                           : gpu::LoadAction::clear;
//     }
//     else
//     {
//         m_flushDesc.colorLoadAction = frameDescriptor.loadAction;
//     }
//     m_flushDesc.colorClearValue = frameDescriptor.clearColor;
//
//     if (doClearDuringAtomicResolve)
//     {
//         // In atomic mode we can accomplish a clear of the color buffer while
//         // the shader resolves coverage, instead of actually clearing.
//         // writeResources() will configure the fill for pathID=0 to be a solid
//         // fill matching the clearColor, so if we just initialize coverage
//         // buffer to solid coverage with pathID=0, the resolve step will write
//         // out the correct clear color.
//         assert(m_flushDesc.interlockMode == gpu::InterlockMode::atomics);
//         m_flushDesc.coverageClearValue =
//             static_cast<uint32_t>(FIXED_COVERAGE_ONE);
//     }
//     else if (m_flushDesc.interlockMode == gpu::InterlockMode::atomics)
//     {
//         // When we don't skip the initial clear in atomic mode, clear the
//         // coverage buffer to pathID=0 and a transparent coverage value.
//         // pathID=0 meets the requirement that pathID is always monotonically
//         // increasing. Transparent coverage makes sure the clearColor doesn't
//         // get written out while resolving.
//         m_flushDesc.coverageClearValue =
//             static_cast<uint32_t>(FIXED_COVERAGE_ZERO);
//     }
//     else
//     {
//         // In non-atomic mode, the coverage buffer just needs to be initialized
//         // with "pathID=0" to avoid collisions with any pathIDs being rendered.
//         m_flushDesc.coverageClearValue = 0;
//     }
//
//     // Adjust the clip bounds so that they are as tight on the writes/reads as
//     // possible, to enable minimal scissor rectangle sizes.
//     // Note: This is done here so that m_combinedDrawBounds are updated before
//     // we try to use them, to ensure they're also tightened in on the clipping
//     // (when scissor is supported).
//     tightenClipBounds();
//
//     if (doClearDuringAtomicResolve ||
//         m_flushDesc.colorLoadAction == gpu::LoadAction::clear)
//     {
//         // If we're clearing then we always update the entire render target.
//         m_flushDesc.renderTargetUpdateBounds =
//             m_flushDesc.renderTarget->bounds();
//     }
//     else
//     {
//         // When we don't clear, we only update the draw bounds.
//         m_flushDesc.renderTargetUpdateBounds =
//             m_flushDesc.renderTarget->bounds().intersect(m_combinedDrawBounds);
//     }
//     if (m_flushDesc.renderTargetUpdateBounds.empty())
//     {
//         // If this is empty it means there are no draws and no clear.
//         m_flushDesc.renderTargetUpdateBounds = {0, 0, 0, 0};
//     }
//
//     m_flushDesc.virtualTileWidth = frameDescriptor.virtualTileWidth;
//     m_flushDesc.virtualTileHeight = frameDescriptor.virtualTileHeight;
//
//     m_flushDesc.manuallyResolved = m_ctx->m_impl->wantsManualRenderPassResolve(
//         m_flushDesc.interlockMode,
//         m_flushDesc.renderTarget,
//         m_flushDesc.renderTargetUpdateBounds,
//         m_flushDesc.virtualTileWidth,
//         m_flushDesc.virtualTileHeight,
//         m_combinedDrawContents);
//
//     m_flushDesc.fixedFunctionColorOutput =
//         wants_fixed_function_color_output(m_ctx->platformFeatures(),
//                                           m_ctx->frameInterlockMode(),
//                                           m_combinedDrawContents,
//                                           m_flushDesc.manuallyResolved);
//     if (m_flushDesc.fixedFunctionColorOutput)
//     {
//         m_baselineShaderMiscFlags |=
//             gpu::ShaderMiscFlags::fixedFunctionColorOutput;
//     }
//     m_flushDesc.featherAtlasContentWidth = m_featherAtlasMaxX;
//     m_flushDesc.featherAtlasContentHeight = m_featherAtlasMaxY;
//
//     m_flushDesc.flushUniformDataOffsetInBytes =
//         logicalFlushIdx * sizeof(gpu::FlushUniforms);
//     m_flushDesc.pathCount =
//         math::lossless_numeric_cast<uint32_t>(m_resourceCounts.pathCount);
//     m_flushDesc.firstPath = runningFrameResourceCounts->pathCount +
//                             runningFrameLayoutCounts->pathPaddingCount;
//     m_flushDesc.firstPaint = runningFrameResourceCounts->pathCount +
//                              runningFrameLayoutCounts->paintPaddingCount;
//     m_flushDesc.firstPaintAux = runningFrameResourceCounts->pathCount +
//                                 runningFrameLayoutCounts->paintAuxPaddingCount;
//     m_flushDesc.contourCount =
//         math::lossless_numeric_cast<uint32_t>(m_resourceCounts.contourCount);
//     m_flushDesc.firstContour = runningFrameResourceCounts->contourCount +
//                                runningFrameLayoutCounts->contourPaddingCount;
//     m_flushDesc.gradSpanCount =
//         math::lossless_numeric_cast<uint32_t>(m_pendingGradSpanCount);
//     m_flushDesc.firstGradSpan = runningFrameLayoutCounts->gradSpanCount +
//                                 runningFrameLayoutCounts->gradSpanPaddingCount;
//     m_flushDesc.gradDataHeight = math::lossless_numeric_cast<uint32_t>(
//         m_gradTextureLayout.complexOffsetY + m_complexGradients.size());
//     m_flushDesc.tessDataHeight = tessDataHeight;
//     m_flushDesc.clockwiseFillOverride = frameDescriptor.clockwiseFillOverride;
//     m_flushDesc.wireframe = frameDescriptor.wireframe;
//     m_flushDesc.ditherMode = m_ctx->frameDescriptor().ditherMode;
// #ifdef WITH_RIVE_TOOLS
//     m_flushDesc.synthesizedFailureType = frameDescriptor.synthesizedFailureType;
// #endif
//
//     m_flushDesc.externalCommandBuffer = flushResources.externalCommandBuffer;
//
//     PUSH_DISABLE_CLANG_SIMD_ABI_WARNING()
//     *runningFrameResourceCounts =
//         runningFrameResourceCounts->toVec() + m_resourceCounts.toVec();
//     POP_DISABLE_CLANG_SIMD_ABI_WARNING()
//
//     runningFrameLayoutCounts->pathPaddingCount += m_pathPaddingCount;
//     runningFrameLayoutCounts->paintPaddingCount += m_paintPaddingCount;
//     runningFrameLayoutCounts->paintAuxPaddingCount += m_paintAuxPaddingCount;
//     runningFrameLayoutCounts->contourPaddingCount += m_contourPaddingCount;
//     runningFrameLayoutCounts->gradSpanCount += m_pendingGradSpanCount;
//     runningFrameLayoutCounts->gradSpanPaddingCount += m_gradSpanPaddingCount;
//     runningFrameLayoutCounts->maxGradTextureHeight =
//         std::max(m_flushDesc.gradDataHeight,
//                  runningFrameLayoutCounts->maxGradTextureHeight);
//     runningFrameLayoutCounts->maxTessTextureHeight =
//         std::max(m_flushDesc.tessDataHeight,
//                  runningFrameLayoutCounts->maxTessTextureHeight);
//     runningFrameLayoutCounts->maxFeatherAtlasWidth =
//         std::max(m_featherAtlasMaxX,
//                  runningFrameLayoutCounts->maxFeatherAtlasWidth);
//     runningFrameLayoutCounts->maxFeatherAtlasHeight =
//         std::max(m_featherAtlasMaxY,
//                  runningFrameLayoutCounts->maxFeatherAtlasHeight);
//     runningFrameLayoutCounts->maxPLSTransientBackingPlaneCount =
//         std::max(pls_transient_backing_plane_count(m_flushDesc.interlockMode,
//                                                    m_combinedDrawContents),
//                  runningFrameLayoutCounts->maxPLSTransientBackingPlaneCount);
//     runningFrameLayoutCounts->maxCoverageBufferLength =
//         std::max<size_t>(m_coverageBufferLength,
//                          runningFrameLayoutCounts->maxCoverageBufferLength);
//
//     assert(m_flushDesc.firstPath % gpu::kPathBufferAlignmentInElements == 0);
//     assert(m_flushDesc.firstPaint % gpu::kPaintBufferAlignmentInElements == 0);
//     assert(m_flushDesc.firstPaintAux %
//                gpu::kPaintAuxBufferAlignmentInElements ==
//            0);
//     assert(m_flushDesc.firstContour % gpu::kContourBufferAlignmentInElements ==
//            0);
//     assert(m_flushDesc.firstGradSpan %
//                gpu::kGradSpanBufferAlignmentInElements ==
//            0);
//     RIVE_DEBUG_CODE(m_hasDoneLayout = true;)
// }
//
// void RenderContext::LogicalFlush::pushBarriers(BarrierFlags barrierFlags)
// {
//     if (m_ctx->platformFeatures()
//             .clockwiseAtomicBorrowedCoverageBarrierNeedsRenderPassInit &&
//         enums::is_flag_set(barrierFlags,
//                            gpu::BarrierFlags::clockwiseBorrowedCoverage))
//     {
//         // We need a workaround in order for input attachments to work.
//         BarrierFlags workaroundBarriers =
//             BarrierFlags::clockwiseBorrowedCoverage | BarrierFlags::plsAtomic;
//         if (enums::is_flag_set(m_combinedDrawContents,
//                                gpu::DrawContents::advancedBlend))
//         {
//             workaroundBarriers |= BarrierFlags::dstBlend;
//         }
//         m_drawList.emplace_back(m_ctx->perFrameAllocator(),
//                                 gpu::DrawType::renderPassInitialize,
//                                 m_baselineShaderMiscFlags,
//                                 gpu::DrawContents::none,
//                                 1,
//                                 0,
//                                 BlendMode::overlay,
//                                 ImageSampler::LinearClamp(),
//                                 workaroundBarriers);
//         barrierFlags &= ~gpu::BarrierFlags::clockwiseBorrowedCoverage;
//     }
//
//     m_pendingBarriers |= barrierFlags;
// }
//
// void RenderContext::LogicalFlush::writeResources()
// {
//     RIVE_PROF_SCOPE_L(1)
//     const gpu::PlatformFeatures& platformFeatures = m_ctx->platformFeatures();
//     assert(m_hasDoneLayout);
//     assert(m_flushDesc.firstPath == m_ctx->m_pathData.elementsWritten());
//     assert(m_flushDesc.firstPaint == m_ctx->m_paintData.elementsWritten());
//     assert(m_flushDesc.firstPaintAux ==
//            m_ctx->m_paintAuxData.elementsWritten());
//
//     // Wait until here before we record these texture sizes; they aren't decided
//     // until after all LogicalFlushes have run layoutResources().
//     m_flushDesc.featherAtlasTextureWidth =
//         math::lossless_numeric_cast<uint32_t>(
//             m_ctx->m_currentResourceAllocations.featherAtlasTextureWidth);
//     m_flushDesc.featherAtlasTextureHeight =
//         math::lossless_numeric_cast<uint32_t>(
//             m_ctx->m_currentResourceAllocations.featherAtlasTextureHeight);
//     m_gradTextureLayout.inverseHeight =
//         1.f / m_ctx->m_currentResourceAllocations.gradTextureHeight;
//
//     // Exact tessSpan/triangleVertex counts aren't known until after their data
//     // is written out.
//     size_t firstTessVertexSpan = m_ctx->m_tessSpanData.elementsWritten();
//     size_t initialTriangleVertexDataSize =
//         m_ctx->m_triangleVertexData.bytesWritten();
//
//     // Metal requires vertex buffers to be 256-byte aligned.
//     size_t tessAlignmentPadding =
//         math::padding_to_align_up<gpu::kTessVertexBufferAlignmentInElements>(
//             firstTessVertexSpan);
//     assert(tessAlignmentPadding <= kMaxTessellationAlignmentVertices);
//     m_ctx->m_tessSpanData.push_back_n(nullptr, tessAlignmentPadding);
//     m_flushDesc.firstTessVertexSpan =
//         firstTessVertexSpan + tessAlignmentPadding;
//     assert(m_flushDesc.firstTessVertexSpan ==
//            m_ctx->m_tessSpanData.elementsWritten());
//
//     // Write out the simple gradient data.
//     constexpr static uint32_t ONE_TEXEL_FIXED = 65536 / gpu::kGradTextureWidth;
//     assert(m_simpleGradients.size() == m_pendingSimpleGradDraws.size());
//     if (!m_pendingSimpleGradDraws.empty())
//     {
//         for (size_t i = 0; i < m_pendingSimpleGradDraws.size(); ++i)
//         {
//             // Render each simple gradient as a single, empty GradientSpan with
//             // 1px borders to the left and right.
//             auto [color0, color1] = m_pendingSimpleGradDraws[i];
//             uint32_t y = math::lossless_numeric_cast<uint32_t>(
//                 i / gpu::kGradTextureWidthInSimpleRamps);
//             size_t centerX = (i % gpu::kGradTextureWidthInSimpleRamps) * 2 + 1;
//             uint32_t centerXFixed = math::lossless_numeric_cast<uint32_t>(
//                 centerX * ONE_TEXEL_FIXED);
//             m_ctx->m_gradSpanData.set_back(centerXFixed,
//                                            centerXFixed,
//                                            y,
//                                            GRAD_SPAN_FLAG_LEFT_BORDER |
//                                                GRAD_SPAN_FLAG_RIGHT_BORDER,
//                                            color0,
//                                            color1);
//         }
//     }
//
//     // Write out the vertex data for rendering complex gradients.
//     assert(m_complexGradients.size() == m_pendingComplexGradDraws.size());
//     if (!m_pendingComplexGradDraws.empty())
//     {
//         // The viewport will start at simpleGradDataHeight when rendering color
//         // ramps.
//         for (uint32_t i = 0; i < m_pendingComplexGradDraws.size(); ++i)
//         {
//             // Push "GradientSpan" instances that will render each section of
//             // this color ramp's gradient.
//             const Gradient* gradient = m_pendingComplexGradDraws[i];
//             const float* stops = gradient->stops();
//             const ColorInt* colors = gradient->colors();
//             size_t stopCount = gradient->count();
//             uint32_t y = i + m_gradTextureLayout.complexOffsetY;
//
//             // "stop * m + a" converts a stop position to a fixed-point x
//             // coordinate in the gradient texture. (In an ideal world, stops
//             // would all be aligned on pixel centers for the texture sampling to
//             // be identical to the gradient, but here we just stretch it across
//             // kGradTextureWidth pixels and hope everything looks ok.)
//             float m = (kGradTextureWidth - 1.f) * ONE_TEXEL_FIXED;
//             float a = .5f * ONE_TEXEL_FIXED;
//             uint32_t lastXFixed = static_cast<uint32_t>(stops[0] * m + a);
//             ColorInt lastColor = colors[0];
//             assert(stopCount >= 2);
//             for (size_t i = 1; i < stopCount; ++i)
//             {
//                 uint32_t xFixed = static_cast<uint32_t>(stops[i] * m + a);
//                 // stops[] must be ordered.
//                 assert(lastXFixed <= xFixed && xFixed < 65536);
//                 uint32_t flags = GRAD_SPAN_FLAG_COMPLEX_BORDER;
//                 if (i == 1)
//                     flags |= GRAD_SPAN_FLAG_LEFT_BORDER;
//                 if (i == stopCount - 1)
//                     flags |= GRAD_SPAN_FLAG_RIGHT_BORDER;
//                 m_ctx->m_gradSpanData.set_back(lastXFixed,
//                                                xFixed,
//                                                y,
//                                                flags,
//                                                lastColor,
//                                                colors[i]);
//                 lastColor = colors[i];
//                 lastXFixed = xFixed;
//             }
//         }
//     }
//
//     // Write a path record for the clearColor paint (used by atomic mode).
//     // This also allows us to index the storage buffers directly by pathID.
//     gpu::SimplePaintValue clearColorValue;
//     clearColorValue.color = m_ctx->frameDescriptor().clearColor;
//     m_ctx->m_pathData.skip_back();
//     m_ctx->m_paintData.set_back(gpu::DrawContents::none,
//                                 PaintType::solidColor,
//                                 clearColorValue,
//                                 GradTextureLayout(),
//                                 /*clipID =*/0,
//                                 /*hasClipRect =*/false,
//                                 BlendMode::srcOver);
//     m_ctx->m_paintAuxData.skip_back();
//
//     // Render padding vertices in the tessellation texture.
//     if (m_flushDesc.tessDataHeight > 0)
//     {
//         // Padding at the beginning of the tessellation texture.
//         pushPaddingVertices(gpu::kMidpointFanPatchSegmentSpan, 0);
//         // Padding between patch types in the tessellation texture.
//         if (m_outerCubicTessVertexIdx > m_midpointFanTessEndLocation)
//         {
//             pushPaddingVertices(m_outerCubicTessVertexIdx -
//                                     m_midpointFanTessEndLocation,
//                                 m_midpointFanTessEndLocation);
//         }
//         // The final vertex of the final patch of each contour crosses over into
//         // the next contour. (This is how we wrap around back to the beginning.)
//         // Therefore, the final contour of the flush needs an out-of-contour
//         // vertex to cross into as well, so we emit a padding vertex here at the
//         // end.
//         pushPaddingVertices(1, m_outerCubicTessEndLocation);
//     }
//
//     // Write out all the data for our high level draws, and build up a low-level
//     // draw list.
//     if (!platformFeatures.supportsClipScissor &&
//         m_ctx->frameInterlockMode() == gpu::InterlockMode::rasterOrdering)
//     {
//         for (const DrawUniquePtr& draw : m_draws)
//         {
//             // TODO: We don't currently support a front-to-back prepass in
//             // rasterOrdering mode. If we decide to support this, we will either
//             // need to walk the draws backwards here, or, more likely, start
//             // sorting and re-ordering in rasterOrdering mode as well.
//             assert(draw->prepassCount() == 0);
//             assert(draw->subpassCount() > 0);
//             for (int i = 0; i < draw->subpassCount(); ++i)
//             {
//                 draw->pushToRenderContext(this, i);
//             }
//         }
//     }
//     else
//     {
//         assert(m_drawPassCount <= kMaxReorderedDrawPassCount);
//
//         // Sort the draw list to optimize batching, since we can only batch
//         // non-overlapping draws.
//         auto& indirectDrawList = m_ctx->m_indirectDrawList;
//         indirectDrawList.clear();
//         indirectDrawList.reserve(m_drawPassCount);
//
//         // TODO: For clockwiseAtomic, these next values aren't constant (they're
//         // constants now just to have stand-in values representing the default
//         // case). Instead:
//         //  - There would be (at least) three relevant "overlap bits":
//         //    - color buffer write
//         //    - clip buffer read
//         //    - clip buffer write
//         //  - groupingType should be GroupingType::overlapAllowed (unless there
//         //    is some reason the current draw could *never* overlap anything
//         //    else)
//         //  - Any draws that write to the color buffer (which may include draws
//         //    that also use the *clip* buffer) would set the "color buffer
//         //    write" bit in its overlap bits
//         //  - Draws that are using advanced blending would set the "color buffer
//         //    write" bit in its disallow mask, so that they are not allowed to
//         //    overlap things that write to the color buffer (there is nothing
//         //    extra for advanced blending that goes into the overlap bits -
//         //    advanced blending has no bearing on whether or not things can
//         //    overlap on top of it!)
//         //  - Any draws that read from the clip buffer:
//         //    - set the "clip buffer read" bit in `overlapBits` - this gets
//         //      stored with the rectangle and signifies that the rectangle is
//         //      involved in a clip buffer read
//         //    - sets the "clip buffer write" bit in `disallowOverlapMask` - this
//         //      tells the intersection board that if this draw overlaps a clip
//         //      buffer write, it needs to go in the next draw group (there needs
//         //      to be a barrier)
//         //  - Any draws that write to the clip buffer:
//         //    - set the "clip buffer write" bit in `overlapBits`
//         //    - sets *both* the "clip buffer read/write" bits in
//         //      `disallowOverlapMask` - this means that these draws would need a
//         //      barrier between any previous overlapping clip buffer reads or
//         //      writes.
//         //  - Similarly, the ordering of the bits in the sort key would likely
//         //    want to change for this mode to ensure that the sorting preserves
//         //    proper ordering within a given draw group, since now there are
//         //    overlaps and thus draw ordering can matter.
//         //    (it also might be worth double checking that there aren't other
//         //    modes where a different sort ordering could be more efficient, to
//         //    perhaps better group like things together that don't cause
//         //    barriers)
//         constexpr static uint16_t kOverlapBits = 0;
//         constexpr static uint16_t kDisallowOverlapMask = 0;
//         constexpr static GroupingType kGroupingType = GroupingType::disjoint;
//
//         if (m_ctx->m_intersectionBoard == nullptr)
//         {
//             m_ctx->m_intersectionBoard =
//                 std::make_unique<IntersectionBoard>(kGroupingType);
//         }
//
//         assert(m_ctx->m_intersectionBoard->groupingType() == kGroupingType);
//         IntersectionBoard* intersectionBoard = m_ctx->m_intersectionBoard.get();
//         intersectionBoard->resizeAndReset(m_flushDesc.renderTarget->width(),
//                                           m_flushDesc.renderTarget->height());
//
//         static constexpr SortKeyBuilder keyBuilder{
//             // Our top priority in re-ordering is to group non-overlapping draws
//             // together, in order to maximize batching while preserving
//             // correctness.
//             {.entry = SortEntry::drawGroup, .bitCount = 15},
//
//             // Within sub-groups of non-overlapping draws, sort similar draw
//             // types together.
//             {.entry = SortEntry::drawType, .bitCount = 3},
//
//             // Within sub-groups of matching draw type, sort by texture binding.
//             {.entry = SortEntry::textureHash, .bitCount = 14},
//
//             // It's less expensive to change the scissorID than texture, but
//             // more expensive than the blend mode, so here's where it lives.
//             {.entry = SortEntry::scissorID, .bitCount = 15},
//
//             // If using KHR_blend_equation_advanced, we need a batching barrier
//             // between draws with different blend modes. If not using
//             // KHR_blend_equation_advanced, sorting by blend mode may still give
//             // us better branching on the GPU.
//             {.entry = SortEntry::blendMode, .bitCount = 4},
//
//             // msaa mode draws strokes, fills, and even/odd with different
//             // stencil settings.
//             {.entry = SortEntry::drawContents, .bitCount = 9},
//
//             // Finally, we need sorting by subpass. Without this, the MSAA
//             // subpasses (and maybe others) won't run in the correct order when
//             // allSubpassesInSameDrawGroup was true.
//             {.entry = SortEntry::subpassIndex, .bitCount = 3},
//         };
//
//         m_ctx->m_scissorIDLookup.clear();
//
//         // Set this to 0, any actual scissor IDs used will then start at 1.
//         m_ctx->m_prevScissorID = 0;
//
//         for (int16_t drawIndex = 0; drawIndex < int16_t(m_draws.size());
//              ++drawIndex)
//         {
//             Draw* draw = m_draws[drawIndex].get();
//
//             int16_t scissorID = 0;
//             auto drawPixelBoundRect = draw->pixelBounds();
//
//             if (platformFeatures.supportsClipScissor &&
//                 (draw->clipID() != 0 ||
//                  draw->clippingPixelBounds().has_value()))
//             {
//                 const auto drawClipID = draw->clipID();
//
//                 // Start with either the clipping pixel bounds (if they exist)
//                 // or a maximally-large rectangle.
//                 auto clipBounds =
//                     draw->clippingPixelBounds().value_or(IAABB::makeMaximal());
//
//                 if (drawClipID != 0)
//                 {
//                     // Intersect with the tightened clip bounds if there was a
//                     // clip in the stack (which may be tighter than it was when
//                     // originally rendered - but also there may have been a clip
//                     // rect that happened after this clip path, which is why the
//                     // intersect still needs to happen)
//                     clipBounds = clipBounds.intersect(
//                         getClipInfo(drawClipID).tightenedBounds);
//                 }
//
//                 const auto drawBounds = draw->pixelBounds();
//
//                 if (needsScissor(drawBounds,
//                                  clipBounds,
//                                  frameDescriptor().renderTargetWidth,
//                                  frameDescriptor().renderTargetHeight))
//                 {
//                     drawPixelBoundRect = clipBounds;
//
//                     const auto clipBoundsU16 =
//                         clipBounds.clamp_cast<uint16_t>();
//
//                     // If the value is already in the map, get it, otherwise
//                     // we'll add the next new ID (which is 1 + the size of
//                     // the array, since we're using "0" as "no scissor")
//                     auto result = m_ctx->m_scissorIDLookup.try_emplace(
//                         clipBoundsU16,
//                         m_ctx->m_prevScissorID + 1);
//                     scissorID = result.first->second;
//                     assert(scissorID > 0);
//                     if (scissorID > m_ctx->m_prevScissorID)
//                     {
//                         ++m_ctx->m_prevScissorID;
//                     }
//
//                     // Update the scissor rect for this draw so we can
//                     // ensure it doesn't batch with draws with different
//                     // scissor rects.
//                     draw->setScissorRect(clipBoundsU16);
//                 }
//             }
//
//             int4 drawBounds = simd::load4i(&drawPixelBoundRect);
//
//             // Add one extra pixel of padding to the draw bounds to make
//             // absolutely certain we get no overlapping pixels, which destroy
//             // the atomic shader.
//             constexpr int32_t Max32i = std::numeric_limits<int32_t>::max();
//             constexpr int32_t Min32i = std::numeric_limits<int32_t>::min();
//             drawBounds = simd::if_then_else(
//                 drawBounds != int4{Min32i, Min32i, Max32i, Max32i},
//                 drawBounds + int4{-1, -1, 1, 1},
//                 drawBounds);
//             if (m_ctx->frameInterlockMode() ==
//                     gpu::InterlockMode::clockwiseAtomic &&
//                 enums::is_flag_set(draw->drawContents(),
//                                    gpu::DrawContents::clipUpdate))
//             {
//                 // ***FIXME***: until we implement scissors for clipping,
//                 // clockwiseAtomic clip updates can't be reordered. Expand their
//                 // pixel bounds to block reordering.
//                 drawBounds = {
//                     0,
//                     0,
//                     static_cast<int32_t>(
//                         m_ctx->frameDescriptor().renderTargetWidth),
//                     static_cast<int32_t>(
//                         m_ctx->frameDescriptor().renderTargetHeight),
//                 };
//             }
//
//             // When the dstBlend barrier has no other option than to copy out a
//             // texture, this copy destroys MSAA information and we can no longer
//             // put subpasses in different drawGroups.
//             // Otherwise, we put subpasses into different draw groups because it
//             // yields better reordering.
//             const bool allSubpassesInSameDrawGroup =
//                 m_ctx->frameInterlockMode() == gpu::InterlockMode::msaa &&
//                 !platformFeatures.supportsBlendAdvancedKHR &&
//                 enums::is_flag_set(m_combinedDrawContents,
//                                    gpu::DrawContents::advancedBlend);
//
//             // Our top priority in re-ordering is to group non-overlapping draws
//             // together, in order to maximize batching while preserving
//             // correctness.
//             const int8_t maxSubpasses = math::lossless_numeric_cast<int8_t>(
//                 std::max(draw->prepassCount(), draw->subpassCount()));
//             int16_t drawGroupIdx = intersectionBoard->addRectangle(
//                 drawBounds,
//                 kOverlapBits,
//                 kDisallowOverlapMask,
//                 allSubpassesInSameDrawGroup ? 1 : maxSubpasses);
//             assert(drawGroupIdx > 0);
//             const auto textureHash =
//                 (draw->imageTexture() != nullptr)
//                     ? draw->imageTexture()->textureResourceHash()
//                     : 0;
//             int64_t key = keyBuilder.buildKey({
//                 {SortEntry::blendMode,
//                  gpu::ConvertBlendModeToPLSBlendMode(draw->blendMode())},
//                 {SortEntry::drawContents, draw->drawContents()},
//                 {SortEntry::drawGroup, drawGroupIdx},
//                 {SortEntry::drawType, draw->type()},
//                 {SortEntry::scissorID, scissorID},
//                 {SortEntry::subpassIndex, 0}, // This gets added later
//
//                 // The hash may lose bits in the key
//                 {SortEntry::textureHash, textureHash, ValidateKeyEntry::no},
//             });
//
//             // Add the first prepass and subpass, if any.
//             if (draw->prepassCount() > 0)
//             {
//                 // Negating the key is an easy way to sort the prepasses
//                 // front-to-back, and before the subpasses.
//                 indirectDrawList.push_back({
//                     .sortKey = -key,
//                     .drawIndex = drawIndex,
//                 });
//             }
//             if (draw->subpassCount() > 0)
//             {
//                 indirectDrawList.push_back({
//                     .sortKey = key,
//                     .drawIndex = drawIndex,
//                 });
//             }
//
//             // Add any additional passes.
//             if (maxSubpasses > 1)
//             {
//                 const auto subpassKeyIncrement =
//                     allSubpassesInSameDrawGroup
//                         // Special case: All subpasses belong to the same
//                         // drawGroup, so only increment subpassIndex.
//                         ? keyBuilder.buildPartialKey({
//                               {SortEntry::subpassIndex, 1},
//                           })
//                         // Usual case: Increment the drawGroup and subpassIndex
//                         // both at once. (The intersectionBoard already reserved
//                         // "maxPasses" layers of drawGroupIndices for us.)
//                         : keyBuilder.buildPartialKey({
//                               {SortEntry::drawGroup, 1},
//                               {SortEntry::subpassIndex, 1},
//                           });
//
//                 for (int8_t subpassIndex = 1; subpassIndex < maxSubpasses;
//                      ++subpassIndex)
//                 {
//                     key += subpassKeyIncrement;
//
//                     assert(keyBuilder.extract<int16_t>(SortEntry::drawGroup,
//                                                        key) ==
//                            int16_t(allSubpassesInSameDrawGroup
//                                        ? drawGroupIdx
//                                        : drawGroupIdx + subpassIndex));
//
//                     if (subpassIndex < draw->prepassCount())
//                     {
//                         // Negating the key is an easy way to sort the prepasses
//                         // front-to-back, and before the subpasses.
//                         indirectDrawList.push_back({
//                             .sortKey = -key,
//                             .drawIndex = drawIndex,
//                         });
//                     }
//                     if (subpassIndex < draw->subpassCount())
//                     {
//                         indirectDrawList.push_back({
//                             .sortKey = key,
//                             .drawIndex = drawIndex,
//                         });
//                     }
//                 }
//             }
//         }
//         assert(indirectDrawList.size() == m_drawPassCount);
//
//         // Re-order the draws
//         // TODO: If we have any overlappable draws, then we will actually need
//         // to sort using the draw index as well (negatively to go front-to-back
//         // for pre-passes and positively for standard passes). Otherwise, the
//         // order of draws *within* a given sorted key does not matter at all.
//         std::sort(
//             indirectDrawList.begin(),
//             indirectDrawList.end(),
//             [](const auto& a, const auto& b) { return a.sortKey < b.sortKey; });
//
//         assert(m_pendingBarriers == BarrierFlags::none);
//         if (m_ctx->frameInterlockMode() == gpu::InterlockMode::atomics &&
//             platformFeatures.atomicPLSInitNeedsDraw)
//         {
//             // Atomic mode sometimes needs to initialize PLS with a draw when
//             // the backend can't do it with typical clear/load APIs.
//             // So far only Metal needs this, and its implementation doesn't
//             // require a barrier before or after.
//             m_drawList.emplace_back(m_ctx->perFrameAllocator(),
//                                     gpu::DrawType::renderPassInitialize,
//                                     m_baselineShaderMiscFlags,
//                                     gpu::DrawContents::none,
//                                     1,
//                                     0,
//                                     BlendMode::srcOver,
//                                     ImageSampler::LinearClamp(),
//                                     BarrierFlags::none);
//         }
//         else if (m_ctx->frameInterlockMode() == gpu::InterlockMode::msaa &&
//                  m_flushDesc.colorLoadAction ==
//                      gpu::LoadAction::preserveRenderTarget &&
//                  platformFeatures.msaaColorPreserveNeedsDraw)
//         {
//             // When implemented with a transient attachment, MSAA needs us to
//             // draw the old renderTarget contents into the framebuffer at the
//             // beginning of the render pass when
//             // LoadAction::preserveRenderTarget is specified.
//             m_drawList.emplace_back(
//                 m_ctx->perFrameAllocator(),
//                 gpu::DrawType::renderPassInitialize,
//                 m_baselineShaderMiscFlags,
//                 gpu::DrawContents::opaquePaint,
//                 1,
//                 0,
//                 // A more realistic value here would be "BlendMode::none" (which
//                 // is what actually happens), but since that isn't a Rive blend
//                 // mode, we just need any value here. It will get ignored by the
//                 // draw.
//                 BlendMode::srcOver,
//                 ImageSampler{.filter = ImageFilter::bilinear},
//                 // The MSAA init reads the framebuffer, so it needs the
//                 // equivalent of a "dstBlend" barrier.
//                 BarrierFlags::dstBlend);
//             m_combinedDrawContents |= m_drawList.tail()->drawContents;
//             // The draw that follows the this init will need a special
//             // "msaaPostInit" barrier.
//             pushBarriers(BarrierFlags::msaaPostInit);
//             assert(m_dstBlendBarrierListTail == &m_firstDstBlendBarrier);
//             assert(m_firstDstBlendBarrier == nullptr);
//             m_firstDstBlendBarrier = m_drawList.tail();
//             m_dstBlendBarrierListTail = &m_drawList.tail()->nextDstBlendBarrier;
//         }
//
//         // Indicates required barriers between draws whose keys differ on the
//         // given mask.
//         struct BarriersForKeyDiff
//         {
//             int64_t mask;
//             BarrierFlags barrier;
//         };
//         StackVector<BarriersForKeyDiff, 3> barriersForKeyDiffs;
//
//         // Find a mask that tells us when to insert barriers, and which barriers
//         // are needed. When the keys of two adjacent draws differ within this
//         // bitmask, we insert a barrier between them.
//         switch (m_flushDesc.interlockMode)
//         {
//             case gpu::InterlockMode::atomics:
//             {
//                 // In atomic mode, we need barriers any time draws overlap.
//                 // Insert a barrier every time the drawGroupIdx changes.
//                 barriersForKeyDiffs.push_back(
//                     {keyBuilder.mask(SortEntry::drawGroup),
//                      BarrierFlags::plsAtomic | BarrierFlags::drawBatchBreak});
//                 // We need a plsAtomic barrier after the initial clears, loads,
//                 // etc.
//                 pushBarriers(BarrierFlags::plsAtomic |
//                              BarrierFlags::drawBatchBreak);
//                 break;
//             }
//
//             case gpu::InterlockMode::rasterOrdering:
//             case gpu::InterlockMode::clockwise:
//                 // clockwise and rasterOrdering modes don't need barriers, but
//                 // we still reorder in order to improve batching.
//                 break;
//
//             case gpu::InterlockMode::clockwiseAtomic:
//             {
//                 // In clockwiseAtomic mode, we only need a barrier between the
//                 // borrowedCoverage prepasses and the main rendering. Prepasses
//                 // have a negative key, so just insert a barrier when the sign
//                 // changes.
//                 constexpr static int64_t SIGN_BIT = (1ll << 63);
//                 barriersForKeyDiffs.push_back(
//                     {SIGN_BIT,
//                      BarrierFlags::clockwiseBorrowedCoverage |
//                          BarrierFlags::drawBatchBreak});
//                 // Just break batching between draw groups. If we also need a
//                 // dstBlend or "clip read" (plsAtomic) barrier, that will be
//                 // handled with more sophisticated logic later on.
//                 barriersForKeyDiffs.push_back(
//                     {keyBuilder.mask(SortEntry::drawGroup),
//                      BarrierFlags::drawBatchBreak});
//                 if (indirectDrawList.empty() ||
//                     indirectDrawList[0].sortKey >= 0)
//                 {
//                     // There are no borrowed coverage passes. Initiate the
//                     // transition to the main subpass immediately.
//                     pushBarriers(BarrierFlags::clockwiseBorrowedCoverage);
//                 }
//                 break;
//             }
//
//             case gpu::InterlockMode::msaa:
//             {
//                 // MSAA mode can't batch draws that overlap because they both
//                 // rely on the stencil buffer across subpasses. Stop batching
//                 // every time the drawGroupIdx changes.
//                 int64_t needsBreakMask = keyBuilder.mask(SortEntry::drawGroup);
//                 // MSAA mode draws clips, strokes, fills, and even/odd with
//                 // different stencil settings, so these can't be batched.
//                 needsBreakMask |= keyBuilder.mask(SortEntry::drawContents);
//                 if (platformFeatures.supportsBlendAdvancedKHR)
//                 {
//                     // If using KHR_blend_equation_advanced, we also need to
//                     // stop batching between blend modes in order to change the
//                     // blend equation.
//                     needsBreakMask |= keyBuilder.mask(SortEntry::blendMode);
//                 }
//                 // MSAA barriers only need to prevent batching of draws for now.
//                 // If we also need a dstBlend barrier, that will be decided
//                 // later.
//                 barriersForKeyDiffs.push_back(
//                     {needsBreakMask, BarrierFlags::drawBatchBreak});
//                 break;
//             }
//         }
//
//         // Write out the draw data from the sorted draw list, and build up a
//         // condensed/batched list of low-level draws.
//         constexpr int64_t BEGIN_KEY = std::numeric_limits<int64_t>::min();
//         int64_t priorSignedKey = BEGIN_KEY;
//         int16_t currentDrawGroup = -1;
//         DrawBatch* firstBatchInCurrentDrawGroup = nullptr;
//
//         // clockwiseAtomic (CWA) needs more sophisticated barrier logic for
//         // clips.
//         bool hasCWAClipReadBarrier = false;
//         bool currentDrawGroupHasCWAClipUpdate = false;
//
//         for (const auto& sortEntry : indirectDrawList)
//         {
//             auto signedKey = sortEntry.sortKey;
//             assert(signedKey >= priorSignedKey);
//             // The first draw never gets simple barriers. If barriers are
//             // required before the first draw, those get scheduled outside this
//             // loop.
//             if (priorSignedKey != BEGIN_KEY)
//             {
//                 for (auto [mask, barriers] : barriersForKeyDiffs)
//                 {
//                     if ((priorSignedKey & mask) != (signedKey & mask))
//                     {
//                         pushBarriers(barriers);
//                     }
//                 }
//             }
//
//             auto key = abs(signedKey);
//             auto drawIndex = sortEntry.drawIndex;
//             auto subpassIndex =
//                 keyBuilder.extract<int8_t>(SortEntry::subpassIndex, key);
//             if (signedKey < 0)
//             {
//                 // Negative keys are a prepass. Update the subpassIndex to be
//                 // negative.
//                 subpassIndex = -1 - subpassIndex;
//             }
//             // FIXME: m_currentZIndex shouldn't be a stateful variable; it
//             // should be passed to pushToRenderContext() instead.
//             const int16_t drawGroup =
//                 keyBuilder.extract<int16_t>(SortEntry::drawGroup, key);
//             assert(drawGroup > 0);
//             m_currentZIndex = drawGroup;
//
//             Draw* draw = m_draws[drawIndex].get();
//
//             assert(
//                 draw->drawContents() ==
//                 keyBuilder.extract<gpu::DrawContents>(SortEntry::drawContents,
//                                                       key));
//             assert(draw->blendMode() != BlendMode::srcOver ==
//                    draw->hasAdvancedBlend());
//
//             DrawBatch* batch = draw->pushToRenderContext(this, subpassIndex);
//
//             if (batch != nullptr && platformFeatures.supportsClipScissor)
//             {
//                 batch->scissorRect = draw->scissorRect();
//             }
//
//             // Some barriers need more sophisticated logic than "do my keys
//             // differ".
//             if ((m_ctx->frameInterlockMode() ==
//                      gpu::InterlockMode::clockwiseAtomic ||
//                  m_ctx->frameInterlockMode() == gpu::InterlockMode::msaa) &&
//                 subpassIndex == 0 && batch != nullptr)
//             {
//                 // Barriers at this level have to go on the first batch in the
//                 // current drawGroup. Otherwise we might see something get
//                 // reordered like this:
//                 //
//                 //   - drawA, subpass0
//                 //   - dstBlendBarrier (because drawB has a dstBlend)
//                 //   - drawB, subpass0
//                 //   - drawA, subpass1
//                 //   - drawB, subpass1
//                 //
//                 // In this scenario, drawA gets a dstBlend barrier between
//                 // subpasses. When dstBlend is implemented as a texture copy, it
//                 // interrupts the render pass and resolves MSAA, causing the
//                 // MSAA data to be lost between supbasses of drawA.
//                 //
//                 // Since drawA and drawB don't overlap, the correct solution is
//                 // to only apply barriers on the first batch of a drawGroup:
//                 //
//                 //   - dstBlendBarrier (because drawB has a dstBlend)
//                 //   - drawA, subpass0
//                 //   - drawB, subpass0
//                 //   - drawA, subpass1
//                 //   - drawB, subpass1
//                 //
//                 // (This also leads to fewer barriers overall.)
//                 if (currentDrawGroup != drawGroup)
//                 {
//                     if (currentDrawGroupHasCWAClipUpdate)
//                     {
//                         // Now that we're moving on to a new drawGroup, reset
//                         // hasCWAClipReadBarrier if the clip got written, since
//                         // any future clip reads may overlap what was written.
//                         hasCWAClipReadBarrier = false;
//                         currentDrawGroupHasCWAClipUpdate = false;
//                     }
//                     firstBatchInCurrentDrawGroup = batch;
//                     currentDrawGroup = drawGroup;
//                 }
//                 assert(firstBatchInCurrentDrawGroup != nullptr);
//
//                 if (draw->hasAdvancedBlend() &&
//                     (m_ctx->frameInterlockMode() != gpu::InterlockMode::msaa ||
//                      !m_ctx->platformFeatures()
//                           .supportsBlendAdvancedCoherentKHR))
//                 {
//                     // An implementation-dependent barrier is required between
//                     // overlapping draws. Add a "dstBlend" barrier and build up
//                     // a list of "dstReads" for the batch. The dstRead list will
//                     // be required in the event that the implementation has to
//                     // handle dstReads by copying out a texture.
//                     assert(draw->nextDstRead() == nullptr);
//                     firstBatchInCurrentDrawGroup->dstReadList =
//                         draw->addToDstReadList(
//                             firstBatchInCurrentDrawGroup->dstReadList);
//                     if (!enums::is_flag_set(
//                             firstBatchInCurrentDrawGroup->barriers,
//                             BarrierFlags::dstBlend))
//                     {
//                         firstBatchInCurrentDrawGroup->barriers |=
//                             BarrierFlags::dstBlend;
//                         addBatchToDstBarrierList(firstBatchInCurrentDrawGroup);
//                     }
//                     // We either added ourselves to the dstBlendBarrier
//                     // list or merged into a batch that was already part
//                     // of it.
//                     assert(m_dstBlendBarrierListTail ==
//                            &firstBatchInCurrentDrawGroup->nextDstBlendBarrier);
//                 }
//
//                 // clockwiseAtomic (CWA) needs more sophisticated barrier logic
//                 // for clips.
//                 if (m_ctx->frameInterlockMode() ==
//                     gpu::InterlockMode::clockwiseAtomic)
//                 {
//                     if (draw->isClipUpdate())
//                     {
//                         // Once the clip gets written, it needs a barrier before
//                         // it can be read again from fragment shaders.
//                         //
//                         // NOTE: This won't immediately reset
//                         // "hasCWAClipReadBarrier" because clip reads and writes
//                         // in the same drawGroup don't overlap. Instead, we
//                         // defer resetting hasCWAClipReadBarrier until we begin
//                         // the next drawGroup.
//                         //
//                         // NOTE2: It's ok of activeClip is also set here
//                         // (i.e., nested clip updates, or "clipUpdate |
//                         // activeClip"). Those don't need a barrier. Nested
//                         // clips use hardware blend to apply the existing clip,
//                         // rather than reading it in the fragment shader.
//                         currentDrawGroupHasCWAClipUpdate = true;
//                     }
//                     else if (draw->hasActiveClip() && !hasCWAClipReadBarrier)
//                     {
//                         // Clipped path draws need a barrier because they access
//                         // the clip buffer via input attachment in the fragment
//                         // shader.
//                         firstBatchInCurrentDrawGroup->barriers |=
//                             gpu::BarrierFlags::plsAtomic;
//                         hasCWAClipReadBarrier = true;
//                     }
//                 }
//                 else
//                 {
//                     assert(m_ctx->frameInterlockMode() ==
//                            gpu::InterlockMode::msaa);
//
//                     // msaa doesn't mix srcOver draws with advanced blend draws.
//                     assert(enums::is_flag_set(
//                                batch->shaderFeatures,
//                                gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND) ==
//                            (draw->blendMode() != BlendMode::srcOver));
//
//                     // If using KHR_blend_equation_advanced, we can't mix blend
//                     // modes in a batch.
//                     assert(
//                         !m_ctx->platformFeatures().supportsBlendAdvancedKHR ||
//                         batch->firstBlendMode == draw->blendMode());
//                 }
//             }
//
//             priorSignedKey = signedKey;
//         }
//     }
//
//     // Some modes need one more draw to resolve all the pixels.
//     if (m_ctx->frameInterlockMode() == gpu::InterlockMode::atomics ||
//         m_flushDesc.manuallyResolved)
//     {
//         m_drawList.emplace_back(
//             m_ctx->perFrameAllocator(),
//             gpu::DrawType::renderPassResolve,
//             m_baselineShaderMiscFlags,
//             (m_ctx->frameInterlockMode() == gpu::InterlockMode::atomics)
//                 ? gpu::DrawContents::none
//                 : gpu::DrawContents::opaquePaint,
//             1,
//             0,
//             BlendMode::srcOver,
//             ImageSampler::LinearClamp(),
//             (m_ctx->frameInterlockMode() == gpu::InterlockMode::atomics)
//                 ? BarrierFlags::plsAtomicPreResolve
//                 : BarrierFlags::preManualResolve);
//         m_combinedDrawContents |= m_drawList.tail()->drawContents;
//     }
//
//     // Write out the draws to the feather atlas. Do this after the main draws
//     // (even though the atlas ones execute first) so that our path info and Z
//     // index are decided and available to pushFeatherAtlasTessellation().
//     if (!m_pendingFeatherAtlasDraws.empty())
//     {
//         AABBu16 fullAtlasViewport = {0,
//                                      0,
//                                      m_flushDesc.featherAtlasContentWidth,
//                                      m_flushDesc.featherAtlasContentHeight};
//         gpu::AtlasDrawBatch* currentBatch =
//             m_ctx->m_perFrameAllocator.makePODArray<gpu::AtlasDrawBatch>(
//                 m_pendingFeatherAtlasDraws.size());
//         // Iterate the atlas draws 4 times so we can sort by fill / stroke /
//         // scissored / not, and batch together the draws that don't have
//         // scissor.
//         for (bool stroked : {false, true})
//         {
//             if (stroked)
//             {
//                 m_flushDesc.featherAtlasStrokeBatches = currentBatch;
//             }
//             else
//             {
//                 m_flushDesc.featherAtlasFillBatches = currentBatch;
//             }
//             for (bool scissored : {false, true})
//             {
//                 gpu::AtlasDrawBatch* lastBatch = nullptr;
//                 for (PathDraw* draw : m_pendingFeatherAtlasDraws)
//                 {
//                     if (draw->isStroke() != stroked ||
//                         draw->featherAtlasScissorEnabled() != scissored)
//                     {
//                         continue;
//                     }
//                     uint32_t tessVertexCount, tessBaseVertex;
//                     draw->pushFeatherAtlasTessellation(this,
//                                                        &tessVertexCount,
//                                                        &tessBaseVertex);
//                     if (tessVertexCount == 0)
//                     {
//                         continue;
//                     }
//                     uint32_t patchCount =
//                         tessVertexCount / gpu::kMidpointFanPatchSegmentSpan;
//                     uint32_t basePatch =
//                         tessBaseVertex / gpu::kMidpointFanPatchSegmentSpan;
//                     assert(patchCount * gpu::kMidpointFanPatchSegmentSpan ==
//                            tessVertexCount);
//                     assert(basePatch * gpu::kMidpointFanPatchSegmentSpan ==
//                            tessBaseVertex);
//                     if (lastBatch == nullptr || scissored)
//                     {
//                         lastBatch = currentBatch++;
//                         *lastBatch = {
//                             lastBatch->scissor =
//                                 scissored ? draw->featherAtlasScissor()
//                                           : fullAtlasViewport,
//                             lastBatch->patchCount = patchCount,
//                             lastBatch->basePatch = basePatch,
//                         };
//                     }
//                     else
//                     {
//                         assert(lastBatch->basePatch + lastBatch->patchCount ==
//                                basePatch);
//                         lastBatch->patchCount += patchCount;
//                     }
//                 }
//             }
//             if (stroked)
//             {
//                 m_flushDesc.featherAtlasStrokeBatchCount =
//                     currentBatch - m_flushDesc.featherAtlasStrokeBatches;
//             }
//             else
//             {
//                 m_flushDesc.featherAtlasFillBatchCount =
//                     currentBatch - m_flushDesc.featherAtlasFillBatches;
//             }
//         }
//         assert(m_flushDesc.featherAtlasFillBatchCount +
//                    m_flushDesc.featherAtlasStrokeBatchCount ==
//                currentBatch - m_flushDesc.featherAtlasFillBatches);
//         assert(m_flushDesc.featherAtlasFillBatchCount +
//                    m_flushDesc.featherAtlasStrokeBatchCount <=
//                m_pendingFeatherAtlasDraws.size());
//     }
//
//     // Pad our buffers to 256-byte alignment.
//     m_ctx->m_pathData.push_back_n(nullptr, m_pathPaddingCount);
//     m_ctx->m_paintData.push_back_n(nullptr, m_paintPaddingCount);
//     m_ctx->m_paintAuxData.push_back_n(nullptr, m_paintAuxPaddingCount);
//     m_ctx->m_contourData.push_back_n(nullptr, m_contourPaddingCount);
//     m_ctx->m_gradSpanData.push_back_n(nullptr, m_gradSpanPaddingCount);
//
//     assert(m_ctx->m_pathData.elementsWritten() ==
//            m_flushDesc.firstPath + m_resourceCounts.pathCount +
//                m_pathPaddingCount);
//     assert(m_ctx->m_paintData.elementsWritten() ==
//            m_flushDesc.firstPaint + m_resourceCounts.pathCount +
//                m_paintPaddingCount);
//     assert(m_ctx->m_paintAuxData.elementsWritten() ==
//            m_flushDesc.firstPaintAux + m_resourceCounts.pathCount +
//                m_paintAuxPaddingCount);
//     assert(m_ctx->m_contourData.elementsWritten() ==
//            m_flushDesc.firstContour + m_resourceCounts.contourCount +
//                m_contourPaddingCount);
//     assert(m_ctx->m_gradSpanData.elementsWritten() ==
//            m_flushDesc.firstGradSpan + m_pendingGradSpanCount +
//                m_gradSpanPaddingCount);
//     assert(m_midpointFanTessVertexIdx == m_midpointFanTessEndLocation);
//     assert(m_outerCubicTessVertexIdx == m_outerCubicTessEndLocation);
//
//     // Some of the flushDescriptor's data isn't known until after
//     // writeResources(). Update it now that it's known.
//     m_flushDesc.combinedShaderFeatures = m_combinedShaderFeatures;
//
//     if (m_coverageBufferLength > 0)
//     {
//         assert(m_flushDesc.interlockMode ==
//                gpu::InterlockMode::clockwiseAtomic);
//         // The coverage buffer prefix gets reset to zero when the buffer is
//         // reallocated, so wait until here to get the prefix.
//         m_flushDesc.coverageBufferPrefix = m_ctx->incrementCoverageBufferPrefix(
//             &m_flushDesc.needsCoverageBufferClear);
//     }
//
//     m_flushDesc.tessVertexSpanCount = math::lossless_numeric_cast<uint32_t>(
//         m_ctx->m_tessSpanData.elementsWritten() -
//         m_flushDesc.firstTessVertexSpan);
//
//     m_flushDesc.hasTriangleVertices =
//         m_ctx->m_triangleVertexData.bytesWritten() !=
//         initialTriangleVertexDataSize;
//
//     m_flushDesc.drawList = &m_drawList;
//     m_flushDesc.firstDstBlendBarrier = m_firstDstBlendBarrier;
//     m_flushDesc.unresolvedBarriers = m_pendingBarriers;
//     // Write out the uniforms for this flush now that the flushDescriptor is
//     // complete.
//     m_ctx->m_flushUniformData.emplace_back(m_flushDesc, platformFeatures);
//
// #ifndef NDEBUG
//     for (const DrawBatch& batch : *m_flushDesc.drawList)
//     {
//         assert((batch.drawContents & m_combinedDrawContents) ==
//                batch.drawContents);
//         assert((batch.shaderFeatures & m_flushDesc.combinedShaderFeatures) ==
//                batch.shaderFeatures);
//     }
// #endif
// }
//
// void RenderContext::LogicalFlush::tightenClipBounds()
// {
//     assert(m_combinedDrawBounds == IAABB::makeMaximallyNegative() &&
//            "m_combinedDrawBounds should not have been updated yet");
//
//     // Iterate through the draws in reverse - this ensures that all paths
//     // clipped by a given clip update will update read bounds first, then any
//     // nested clips will update, and all bounds state should bubble nicely to
//     // the top.
//     for (size_t i = m_draws.size() - 1; i != size_t(-1); i--)
//     {
//         const auto& draw = m_draws[i];
//
//         // Depending on whether the platform supports clip scissor or not, use
//         // the clipped bounds or the pixel bounds as the default bounds for
//         // calculating the combined bounds.
//         IAABB drawBoundsForCombinedBounds =
//             m_ctx->platformFeatures().supportsClipScissor
//                 ? draw->clippedPixelBounds()
//                 : draw->pixelBounds();
//
//         if (draw->clipID() == 0)
//         {
//             // Do nothing here, but we'll update the combined draw bounds after
//             // the `else`s.
//         }
//         else if (draw->isClipUpdate())
//         {
//             auto& clipInfo = getWritableClipInfo(draw->clipID());
//
//             // Ensure the clip's write bounds are as tight on both the clip
//             // shape and all reads as possible.
//             clipInfo.tightenedBounds =
//                 clipInfo.tightenedBounds.intersect(clipInfo.readBounds);
//
//             if (m_ctx->platformFeatures().supportsClipScissor)
//             {
//                 // Bring in the draw bounds for combining based on the
//                 // newly-tightened bounds.
//                 assert(drawBoundsForCombinedBounds.contains(
//                     clipInfo.tightenedBounds));
//                 drawBoundsForCombinedBounds =
//                     clipInfo.tightenedBounds.lossless_numeric_cast<int32_t>();
//             }
//
//             if (draw->hasActiveClip())
//             {
//                 assert(clipInfo.parentClipID != 0);
//
//                 // This is a nested clip so we need to additionally add its
//                 // adjusted bounds to its parent clip as its read bounds.
//                 auto& parentClipInfo =
//                     getWritableClipInfo(clipInfo.parentClipID);
//                 parentClipInfo.readBounds =
//                     parentClipInfo.readBounds.join(clipInfo.tightenedBounds);
//             }
//             else
//             {
//                 assert(clipInfo.parentClipID == 0);
//             }
//         }
//         else if (draw->hasActiveClip())
//         {
//             // Anything else with a clip should add itself to the clip's read
//             // bounds.
//             auto& clipInfo = getWritableClipInfo(draw->clipID());
//             clipInfo.readBounds = clipInfo.readBounds.join(
//                 draw->clippedPixelBounds().clamp_cast<uint16_t>());
//         }
//
//         m_combinedDrawBounds =
//             m_combinedDrawBounds.join(drawBoundsForCombinedBounds);
//     }
// }
//
// void RenderContext::setResourceSizes(ResourceAllocationCounts allocs,
//                                      bool forceRealloc)
// {
//     RIVE_PROF_SCOPE_L(1)
// #if 0
//     class Logger
//     {
//     public:
//         void logSize(const char* name,
//                      size_t oldSize,
//                      size_t newSize,
//                      size_t newSizeInBytes)
//         {
//             m_totalSizeInBytes += newSizeInBytes;
//             if (oldSize == newSize)
//             {
//                 return;
//             }
//             if (!m_hasChanged)
//             {
//                 printf("RenderContext::setResourceSizes():\n");
//                 m_hasChanged = true;
//             }
//             printf("  resize %s: %zu -> %zu (%zu KiB)\n",
//                    name,
//                    oldSize,
//                    newSize,
//                    newSizeInBytes >> 10);
//         }
//
//         void logTextureSize(const char* widthName,
//                             const char* heightName,
//                             size_t oldWidth,
//                             size_t oldHeight,
//                             size_t newWidth,
//                             size_t newHeight,
//                             size_t bytesPerPixel)
//         {
//             m_totalSizeInBytes += newHeight * newWidth * bytesPerPixel;
//             if (oldWidth == newWidth && oldHeight == newHeight)
//             {
//                 return;
//             }
//             if (!m_hasChanged)
//             {
//                 printf("RenderContext::setResourceSizes():\n");
//                 m_hasChanged = true;
//             }
//             printf("  resize %s x %s: %zu x %zu -> %zu x %zu (%zu KiB)\n",
//                    widthName,
//                    heightName,
//                    oldWidth,
//                    oldHeight,
//                    newWidth,
//                    newHeight,
//                    (newHeight * newWidth * bytesPerPixel) >> 10);
//         }
//
//         void logTexture3dSize(const char* name,
//                               size_t oldWidth,
//                               size_t oldHeight,
//                               size_t oldDepth,
//                               size_t newWidth,
//                               size_t newHeight,
//                               size_t newDepth,
//                               size_t bytesPerPixel)
//         {
//             m_totalSizeInBytes += newHeight * newWidth * bytesPerPixel;
//             if (oldWidth == newWidth && oldHeight == newHeight &&
//                 oldDepth == newDepth)
//             {
//                 return;
//             }
//             if (!m_hasChanged)
//             {
//                 printf("RenderContext::setResourceSizes():\n");
//                 m_hasChanged = true;
//             }
//             printf("  resize %s: [%zu x %zu x %zu] -> [%zu x %zu x %zu] "
//                    "(%zu KiB)\n",
//                    name,
//                    oldWidth,
//                    oldHeight,
//                    oldDepth,
//                    newWidth,
//                    newHeight,
//                    newDepth,
//                    (newHeight * newWidth * newDepth * bytesPerPixel) >> 10);
//         }
//
//         ~Logger()
//         {
//             if (!m_hasChanged)
//             {
//                 return;
//             }
//             printf("  TOTAL GPU resource usage: %zu KiB\n",
//                    m_totalSizeInBytes >> 10);
//         }
//
//     private:
//         size_t m_totalSizeInBytes = 0;
//         bool m_hasChanged = false;
//     } logger;
// #define LOG_BUFFER_RING_SIZE(NAME, ITEM_SIZE_IN_BYTES)                         \
//     logger.logSize(#NAME,                                                      \
//                    m_currentResourceAllocations.NAME,                          \
//                    allocs.NAME,                                                \
//                    allocs.NAME* ITEM_SIZE_IN_BYTES* gpu::kBufferRingSize)
// #define LOG_TEXTURE_SIZE(NAME, BYTES_PER_VALUE)                                \
//     logger.logSize(#NAME,                                                      \
//                    m_currentResourceAllocations.NAME,                          \
//                    allocs.NAME,                                                \
//                    allocs.NAME*(BYTES_PER_VALUE))
// #define LOG_TEXTURE_2D_SIZE(NAME, WIDTH_NAME, HEIGHT_NAME, BYTES_PER_PIXEL)    \
//     logger.logTexture3dSize(NAME,                                              \
//                             m_currentResourceAllocations.WIDTH_NAME,           \
//                             m_currentResourceAllocations.HEIGHT_NAME,          \
//                             1,                                                 \
//                             allocs.WIDTH_NAME,                                 \
//                             allocs.HEIGHT_NAME,                                \
//                             1,                                                 \
//                             BYTES_PER_PIXEL)
// #define LOG_TEXTURE_3D_SIZE(NAME,                                              \
//                             WIDTH_NAME,                                        \
//                             HEIGHT_NAME,                                       \
//                             DEPTH_NAME,                                        \
//                             BYTES_PER_PIXEL)                                   \
//     logger.logTexture3dSize(NAME,                                              \
//                             m_currentResourceAllocations.WIDTH_NAME,           \
//                             m_currentResourceAllocations.HEIGHT_NAME,          \
//                             m_currentResourceAllocations.DEPTH_NAME,           \
//                             allocs.WIDTH_NAME,                                 \
//                             allocs.HEIGHT_NAME,                                \
//                             allocs.DEPTH_NAME,                                 \
//                             BYTES_PER_PIXEL)
// #define LOG_BUFFER_SIZE(NAME, BYTES_PER_ELEMENT)                               \
//     logger.logSize(#NAME,                                                      \
//                    m_currentResourceAllocations.NAME,                          \
//                    allocs.NAME,                                                \
//                    allocs.NAME* BYTES_PER_ELEMENT)
// #else
// #define LOG_BUFFER_RING_SIZE(NAME, ITEM_SIZE_IN_BYTES)
// #define LOG_TEXTURE_SIZE(NAME, BYTES_PER_ROW)
// #define LOG_TEXTURE_2D_SIZE(NAME, WIDTH_NAME, HEIGHT_NAME, BYTES_PER_PIXEL)
// #define LOG_TEXTURE_3D_SIZE(NAME,                                              \
//                             WIDTH_NAME,                                        \
//                             HEIGHT_NAME,                                       \
//                             DEPTH_NAME,                                        \
//                             BYTES_PER_PIXEL)
// #define LOG_BUFFER_SIZE(NAME, BYTES_PER_ELEMENT)
// #endif
//
//     LOG_BUFFER_RING_SIZE(flushUniformBufferCount, sizeof(gpu::FlushUniforms));
//     if (allocs.flushUniformBufferCount !=
//             m_currentResourceAllocations.flushUniformBufferCount ||
//         forceRealloc)
//     {
//         m_impl->resizeFlushUniformBuffer(allocs.flushUniformBufferCount *
//                                          sizeof(gpu::FlushUniforms));
//     }
//
//     LOG_BUFFER_RING_SIZE(pathBufferCount, sizeof(gpu::PathData));
//     if (allocs.pathBufferCount !=
//             m_currentResourceAllocations.pathBufferCount ||
//         forceRealloc)
//     {
//         m_impl->resizePathBuffer(allocs.pathBufferCount * sizeof(gpu::PathData),
//                                  gpu::PathData::kBufferStructure);
//     }
//
//     LOG_BUFFER_RING_SIZE(paintBufferCount, sizeof(gpu::PaintData));
//     if (allocs.paintBufferCount !=
//             m_currentResourceAllocations.paintBufferCount ||
//         forceRealloc)
//     {
//         m_impl->resizePaintBuffer(allocs.paintBufferCount *
//                                       sizeof(gpu::PaintData),
//                                   gpu::PaintData::kBufferStructure);
//     }
//
//     LOG_BUFFER_RING_SIZE(paintAuxBufferCount, sizeof(gpu::PaintAuxData));
//     if (allocs.paintAuxBufferCount !=
//             m_currentResourceAllocations.paintAuxBufferCount ||
//         forceRealloc)
//     {
//         m_impl->resizePaintAuxBuffer(allocs.paintAuxBufferCount *
//                                          sizeof(gpu::PaintAuxData),
//                                      gpu::PaintAuxData::kBufferStructure);
//     }
//
//     LOG_BUFFER_RING_SIZE(contourBufferCount, sizeof(gpu::ContourData));
//     if (allocs.contourBufferCount !=
//             m_currentResourceAllocations.contourBufferCount ||
//         forceRealloc)
//     {
//         m_impl->resizeContourBuffer(allocs.contourBufferCount *
//                                         sizeof(gpu::ContourData),
//                                     gpu::ContourData::kBufferStructure);
//     }
//
//     LOG_BUFFER_RING_SIZE(gradSpanBufferCount, sizeof(gpu::GradientSpan));
//     if (allocs.gradSpanBufferCount !=
//             m_currentResourceAllocations.gradSpanBufferCount ||
//         forceRealloc)
//     {
//         m_impl->resizeGradSpanBuffer(allocs.gradSpanBufferCount *
//                                      sizeof(gpu::GradientSpan));
//     }
//
//     LOG_BUFFER_RING_SIZE(tessSpanBufferCount, sizeof(gpu::TessVertexSpan));
//     if (allocs.tessSpanBufferCount !=
//             m_currentResourceAllocations.tessSpanBufferCount ||
//         forceRealloc)
//     {
//         m_impl->resizeTessVertexSpanBuffer(allocs.tessSpanBufferCount *
//                                            sizeof(gpu::TessVertexSpan));
//     }
//
//     LOG_BUFFER_RING_SIZE(triangleVertexBufferCount,
//                          sizeof(gpu::TriangleVertex));
//     if (allocs.triangleVertexBufferCount !=
//             m_currentResourceAllocations.triangleVertexBufferCount ||
//         forceRealloc)
//     {
//         m_impl->resizeTriangleVertexBuffer(allocs.triangleVertexBufferCount *
//                                            sizeof(gpu::TriangleVertex));
//     }
//
//     LOG_BUFFER_RING_SIZE(imageDrawInstanceBufferCount,
//                          sizeof(gpu::ImageDrawInstance));
//     if (allocs.imageDrawInstanceBufferCount !=
//             m_currentResourceAllocations.imageDrawInstanceBufferCount ||
//         forceRealloc)
//     {
//         m_impl->resizeImageDrawInstanceBuffer(
//             allocs.imageDrawInstanceBufferCount *
//             sizeof(gpu::ImageDrawInstance));
//     }
//
//     assert(allocs.gradTextureHeight <= kMaxTextureHeight);
//     LOG_TEXTURE_SIZE(gradTextureHeight, gpu::kGradTextureWidth * 4);
//     if (allocs.gradTextureHeight !=
//             m_currentResourceAllocations.gradTextureHeight ||
//         forceRealloc)
//     {
//         m_impl->resizeGradientTexture(
//             gpu::kGradTextureWidth,
//             math::lossless_numeric_cast<uint32_t>(allocs.gradTextureHeight));
//     }
//
//     assert(allocs.tessTextureHeight <= kMaxTextureHeight);
//     LOG_TEXTURE_SIZE(tessTextureHeight, gpu::kTessTextureWidth * 4 * 4);
//     if (allocs.tessTextureHeight !=
//             m_currentResourceAllocations.tessTextureHeight ||
//         forceRealloc)
//     {
//         m_impl->resizeTessellationTexture(
//             gpu::kTessTextureWidth,
//             math::lossless_numeric_cast<uint32_t>(allocs.tessTextureHeight));
//     }
//
//     assert(allocs.featherAtlasTextureWidth <= featherAtlasMaxSize() ||
//            allocs.featherAtlasTextureWidth <=
//                frameDescriptor().renderTargetWidth);
//     assert(allocs.featherAtlasTextureHeight <= featherAtlasMaxSize() ||
//            allocs.featherAtlasTextureHeight <=
//                frameDescriptor().renderTargetHeight);
//     LOG_TEXTURE_2D_SIZE("featherAtlasTexture",
//                         featherAtlasTextureWidth,
//                         featherAtlasTextureHeight,
//                         sizeof(uint16_t));
//     if (allocs.featherAtlasTextureWidth !=
//             m_currentResourceAllocations.featherAtlasTextureWidth ||
//         allocs.featherAtlasTextureHeight !=
//             m_currentResourceAllocations.featherAtlasTextureHeight ||
//         forceRealloc)
//     {
//         m_impl->resizeFeatherAtlasTexture(
//             math::lossless_numeric_cast<uint32_t>(
//                 allocs.featherAtlasTextureWidth),
//             math::lossless_numeric_cast<uint32_t>(
//                 allocs.featherAtlasTextureHeight));
//     }
//
//     assert(allocs.plsTransientBackingPlaneCount <=
//            RenderContextImpl::PLS_TRANSIENT_BACKING_MAX_PLANE_COUNT);
//     LOG_TEXTURE_3D_SIZE("plsTransientBacking",
//                         plsTransientBackingWidth,
//                         plsTransientBackingHeight,
//                         plsTransientBackingPlaneCount,
//                         sizeof(uint32_t));
//     if (allocs.plsTransientBackingWidth !=
//             m_currentResourceAllocations.plsTransientBackingWidth ||
//         allocs.plsTransientBackingHeight !=
//             m_currentResourceAllocations.plsTransientBackingHeight ||
//         allocs.plsTransientBackingPlaneCount !=
//             m_currentResourceAllocations.plsTransientBackingPlaneCount ||
//         forceRealloc)
//     {
//         m_impl->resizeTransientPLSBacking(
//             math::lossless_numeric_cast<uint32_t>(
//                 allocs.plsTransientBackingWidth),
//             math::lossless_numeric_cast<uint32_t>(
//                 allocs.plsTransientBackingHeight),
//             math::lossless_numeric_cast<uint32_t>(
//                 allocs.plsTransientBackingPlaneCount));
//     }
//
//     assert(allocs.plsAtomicCoverageBackingWidth <=
//            allocs.plsTransientBackingWidth);
//     assert(allocs.plsAtomicCoverageBackingHeight <=
//            allocs.plsTransientBackingHeight);
//     LOG_TEXTURE_2D_SIZE("plsAtomicCoverageBacking",
//                         plsAtomicCoverageBackingWidth,
//                         plsAtomicCoverageBackingHeight,
//                         sizeof(uint32_t));
//     if (allocs.plsAtomicCoverageBackingWidth !=
//             m_currentResourceAllocations.plsAtomicCoverageBackingWidth ||
//         allocs.plsAtomicCoverageBackingHeight !=
//             m_currentResourceAllocations.plsAtomicCoverageBackingHeight ||
//         forceRealloc)
//     {
//         m_impl->resizeAtomicCoverageBacking(
//             math::lossless_numeric_cast<uint32_t>(
//                 allocs.plsAtomicCoverageBackingWidth),
//             math::lossless_numeric_cast<uint32_t>(
//                 allocs.plsAtomicCoverageBackingHeight));
//     }
//
//     assert(allocs.coverageBufferLength <=
//            platformFeatures().maxCoverageBufferLength);
//     LOG_BUFFER_SIZE(coverageBufferLength, sizeof(uint32_t));
//     if (allocs.coverageBufferLength !=
//             m_currentResourceAllocations.coverageBufferLength ||
//         forceRealloc)
//     {
//         m_impl->resizeCoverageBuffer(allocs.coverageBufferLength *
//                                      sizeof(uint32_t));
//         // Start the coverageBufferPrefix over at zero. This ensure the new
//         // buffer gets cleared because the only criteria for clearing it is when
//         // the prefix wraps around to 0.
//         m_coverageBufferPrefix = 0;
//     }
//
//     m_currentResourceAllocations = allocs;
// }
//
// bool RenderContext::mapResourceBuffers(
//     const ResourceAllocationCounts& mapCounts)
// {
//     RIVE_PROF_SCOPE_L(1)
//
// #define HANDLE_MAP_FAILURE(...)                                                \
//     do                                                                         \
//     {                                                                          \
//         if (!(__VA_ARGS__))                                                    \
//         {                                                                      \
//             return false;                                                      \
//         }                                                                      \
//     } while (false)
//
//     if (mapCounts.flushUniformBufferCount > 0)
//     {
//         HANDLE_MAP_FAILURE(m_flushUniformData.mapElements(
//             m_impl.get(),
//             &RenderContextImpl::mapFlushUniformBuffer,
//             mapCounts.flushUniformBufferCount));
//     }
//     assert(m_flushUniformData.hasRoomFor(mapCounts.flushUniformBufferCount));
//
//     if (mapCounts.pathBufferCount > 0)
//     {
//         HANDLE_MAP_FAILURE(
//             m_pathData.mapElements(m_impl.get(),
//                                    &RenderContextImpl::mapPathBuffer,
//                                    mapCounts.pathBufferCount));
//     }
//     assert(m_pathData.hasRoomFor(mapCounts.pathBufferCount));
//
//     if (mapCounts.paintBufferCount > 0)
//     {
//         HANDLE_MAP_FAILURE(
//             m_paintData.mapElements(m_impl.get(),
//                                     &RenderContextImpl::mapPaintBuffer,
//                                     mapCounts.paintBufferCount));
//     }
//     assert(m_paintData.hasRoomFor(mapCounts.paintBufferCount));
//
//     if (mapCounts.paintAuxBufferCount > 0)
//     {
//         HANDLE_MAP_FAILURE(
//             m_paintAuxData.mapElements(m_impl.get(),
//                                        &RenderContextImpl::mapPaintAuxBuffer,
//                                        mapCounts.paintAuxBufferCount));
//     }
//     assert(m_paintAuxData.hasRoomFor(mapCounts.paintAuxBufferCount));
//
//     if (mapCounts.contourBufferCount > 0)
//     {
//         HANDLE_MAP_FAILURE(
//             m_contourData.mapElements(m_impl.get(),
//                                       &RenderContextImpl::mapContourBuffer,
//                                       mapCounts.contourBufferCount));
//     }
//     assert(m_contourData.hasRoomFor(mapCounts.contourBufferCount));
//
//     if (mapCounts.gradSpanBufferCount > 0)
//     {
//         HANDLE_MAP_FAILURE(
//             m_gradSpanData.mapElements(m_impl.get(),
//                                        &RenderContextImpl::mapGradSpanBuffer,
//                                        mapCounts.gradSpanBufferCount));
//     }
//     assert(m_gradSpanData.hasRoomFor(mapCounts.gradSpanBufferCount));
//
//     if (mapCounts.tessSpanBufferCount > 0)
//     {
//         HANDLE_MAP_FAILURE(m_tessSpanData.mapElements(
//             m_impl.get(),
//             &RenderContextImpl::mapTessVertexSpanBuffer,
//             mapCounts.tessSpanBufferCount));
//     }
//     assert(m_tessSpanData.hasRoomFor(mapCounts.tessSpanBufferCount));
//
//     if (mapCounts.triangleVertexBufferCount > 0)
//     {
//         HANDLE_MAP_FAILURE(m_triangleVertexData.mapElements(
//             m_impl.get(),
//             &RenderContextImpl::mapTriangleVertexBuffer,
//             mapCounts.triangleVertexBufferCount));
//     }
//     assert(
//         m_triangleVertexData.hasRoomFor(mapCounts.triangleVertexBufferCount));
//
//     if (mapCounts.imageDrawInstanceBufferCount > 0)
//     {
//         HANDLE_MAP_FAILURE(m_imageDrawInstanceData.mapElements(
//             m_impl.get(),
//             &RenderContextImpl::mapImageDrawInstanceBuffer,
//             mapCounts.imageDrawInstanceBufferCount));
//     }
//     assert(m_imageDrawInstanceData.hasRoomFor(
//         mapCounts.imageDrawInstanceBufferCount > 0));
//
// #undef HANDLE_MAP_FAILURE
//     return true;
// }
//
// void RenderContext::unmapResourceBuffers(
//     const ResourceAllocationCounts& mapCounts)
// {
//     RIVE_PROF_SCOPE_L(1)
//     if (m_flushUniformData)
//     {
//         m_flushUniformData.unmapElements(
//             m_impl.get(),
//             &RenderContextImpl::unmapFlushUniformBuffer,
//             mapCounts.flushUniformBufferCount);
//     }
//     if (m_pathData)
//     {
//         m_pathData.unmapElements(m_impl.get(),
//                                  &RenderContextImpl::unmapPathBuffer,
//                                  mapCounts.pathBufferCount);
//     }
//     if (m_paintData)
//     {
//         m_paintData.unmapElements(m_impl.get(),
//                                   &RenderContextImpl::unmapPaintBuffer,
//                                   mapCounts.paintBufferCount);
//     }
//     if (m_paintAuxData)
//     {
//         m_paintAuxData.unmapElements(m_impl.get(),
//                                      &RenderContextImpl::unmapPaintAuxBuffer,
//                                      mapCounts.paintAuxBufferCount);
//     }
//     if (m_contourData)
//     {
//         m_contourData.unmapElements(m_impl.get(),
//                                     &RenderContextImpl::unmapContourBuffer,
//                                     mapCounts.contourBufferCount);
//     }
//     if (m_gradSpanData)
//     {
//         m_gradSpanData.unmapElements(m_impl.get(),
//                                      &RenderContextImpl::unmapGradSpanBuffer,
//                                      mapCounts.gradSpanBufferCount);
//     }
//     if (m_tessSpanData)
//     {
//         m_tessSpanData.unmapElements(
//             m_impl.get(),
//             &RenderContextImpl::unmapTessVertexSpanBuffer,
//             mapCounts.tessSpanBufferCount);
//     }
//     if (m_triangleVertexData)
//     {
//         m_triangleVertexData.unmapElements(
//             m_impl.get(),
//             &RenderContextImpl::unmapTriangleVertexBuffer,
//             mapCounts.triangleVertexBufferCount);
//     }
//     if (m_imageDrawInstanceData)
//     {
//         m_imageDrawInstanceData.unmapElements(
//             m_impl.get(),
//             &RenderContextImpl::unmapImageDrawInstanceBuffer,
//             mapCounts.imageDrawInstanceBufferCount);
//     }
// }
//
// uint32_t RenderContext::incrementCoverageBufferPrefix(
//     bool* needsCoverageBufferClear)
// {
//     RIVE_PROF_SCOPE_L(1)
//     assert(m_didBeginFrame);
//     assert(frameInterlockMode() == gpu::InterlockMode::clockwiseAtomic);
//     do
//     {
//         if (m_coverageBufferPrefix == 0)
//         {
//             // When the prefix wraps around to 0, we need to clear the coverage
//             // buffer because our shaders require coverageBufferPrefix to be
//             // monotonically increasing.
//             *needsCoverageBufferClear = true;
//         }
//         m_coverageBufferPrefix += CLOCKWISE_COVERAGE_PREFIX_ONE_VALUE;
//     } while (m_coverageBufferPrefix == 0);
//
//     return m_coverageBufferPrefix;
// }
//
// uint32_t RenderContext::LogicalFlush::allocateMidpointFanTessVertices(
//     uint32_t count)
// {
//     uint32_t location = m_midpointFanTessVertexIdx;
//     m_midpointFanTessVertexIdx += count;
//     assert(m_midpointFanTessVertexIdx <= m_midpointFanTessEndLocation);
//     return location;
// }
//
// uint32_t RenderContext::LogicalFlush::allocateOuterCubicTessVertices(
//     uint32_t count)
// {
//     uint32_t location = m_outerCubicTessVertexIdx;
//     m_outerCubicTessVertexIdx += count;
//     assert(m_outerCubicTessVertexIdx <= m_outerCubicTessEndLocation);
//     return location;
// }
//
// uint32_t RenderContext::LogicalFlush::pushPath(const PathDraw* draw)
// {
//     RIVE_PROF_SCOPE_L(2)
//     assert(m_hasDoneLayout);
//
//     ++m_currentPathID;
//     assert(0 < m_currentPathID && m_currentPathID <= m_ctx->m_maxPathID);
//
//     m_ctx->m_pathData.set_back(draw->matrix(),
//                                draw->strokeRadius(),
//                                draw->featherRadius(),
//                                m_currentZIndex,
//                                draw->featherAtlasTransform(),
//                                draw->coverageBufferRange());
//     m_ctx->m_paintData.set_back(draw->drawContents(),
//                                 draw->paintType(),
//                                 draw->simplePaintValue(),
//                                 m_gradTextureLayout,
//                                 draw->clipID(),
//                                 draw->hasClipRect(),
//                                 draw->blendMode());
//     m_ctx->m_paintAuxData.set_back(draw->matrix(),
//                                    draw->paintType(),
//                                    draw->simplePaintValue(),
//                                    draw->gradient(),
//                                    draw->imageTexture(),
//                                    draw->clipRectInverseMatrix(),
//                                    m_flushDesc.renderTarget,
//                                    m_ctx->platformFeatures());
//
//     assert(m_flushDesc.firstPath + m_currentPathID + 1 ==
//            m_ctx->m_pathData.elementsWritten());
//     assert(m_flushDesc.firstPaint + m_currentPathID + 1 ==
//            m_ctx->m_paintData.elementsWritten());
//     assert(m_flushDesc.firstPaintAux + m_currentPathID + 1 ==
//            m_ctx->m_paintAuxData.elementsWritten());
//
//     return m_currentPathID;
// }
//
// RenderContext::TessellationWriter::TessellationWriter(
//     LogicalFlush* flush,
//     uint32_t pathID,
//     gpu::ContourDirections contourDirections,
//     uint32_t forwardTessVertexCount,
//     uint32_t forwardTessLocation,
//     uint32_t mirroredTessVertexCount,
//     uint32_t mirroredTessLocation) :
//     m_flush(flush),
//     m_tessSpanData(m_flush->m_ctx->m_tessSpanData),
//     m_pathID(pathID),
//     m_contourDirections(contourDirections),
//     m_pathTessLocation(forwardTessLocation),
//     m_pathMirroredTessLocation(mirroredTessLocation)
// {
//     RIVE_PROF_SCOPE_L(2)
//     RIVE_DEBUG_CODE(m_expectedPathTessEndLocation =
//                         m_pathTessLocation + forwardTessVertexCount;)
//     RIVE_DEBUG_CODE(m_expectedPathMirroredTessEndLocation =
//                         m_pathMirroredTessLocation - mirroredTessVertexCount;)
//     assert(m_flush->m_hasDoneLayout);
//     assert(m_flush->m_ctx->m_pathData.elementsWritten() > 0);
//     assert(forwardTessVertexCount == 0 || mirroredTessVertexCount == 0 ||
//            forwardTessVertexCount == mirroredTessVertexCount);
//     assert(!gpu::ContourDirectionsAreDoubleSided(m_contourDirections) ||
//            forwardTessVertexCount == mirroredTessVertexCount);
//     assert(m_pathTessLocation >= 0);
//     assert(m_pathMirroredTessLocation <= kMaxTessellationVertexCount);
//     assert(m_expectedPathTessEndLocation <= kMaxTessellationVertexCount);
//     assert(m_expectedPathMirroredTessEndLocation >= 0);
// }
//
// RenderContext::TessellationWriter::~TessellationWriter()
// {
//     assert(m_pathTessLocation == m_expectedPathTessEndLocation);
//     assert(m_pathMirroredTessLocation == m_expectedPathMirroredTessEndLocation);
// }
//
// uint32_t RenderContext::LogicalFlush::pushContour(uint32_t pathID,
//                                                   Vec2D midpoint,
//                                                   bool isStroke,
//                                                   bool closed,
//                                                   uint32_t vertexIndex0)
// {
//     RIVE_PROF_SCOPE_L(2)
//     assert(pathID != 0);
//     assert(isStroke || closed);
//
//     if (isStroke)
//     {
//         midpoint.x = closed ? 1 : 0;
//     }
//     m_ctx->m_contourData.emplace_back(midpoint, pathID, vertexIndex0);
//
//     ++m_currentContourID;
//     assert(0 < m_currentContourID && m_currentContourID <= gpu::kMaxContourID);
//     assert(m_flushDesc.firstContour + m_currentContourID ==
//            m_ctx->m_contourData.elementsWritten());
//     return m_currentContourID;
// }
//
// uint32_t RenderContext::TessellationWriter::pushContour(
//     Vec2D midpoint,
//     bool isStroke,
//     bool closed,
//     uint32_t paddingVertexCount)
// {
//     RIVE_PROF_SCOPE_L(2)
//     // The first curve of the contour will be pre-padded with
//     // 'paddingVertexCount' tessellation vertices, colocated at T=0. The caller
//     // must use this argument align the end of the contour on a boundary of the
//     // patch size. (See math::padding_to_align_up().)
//     m_nextCubicPaddingVertexCount = paddingVertexCount;
//
//     return m_flush->pushContour(m_pathID,
//                                 midpoint,
//                                 isStroke,
//                                 closed,
//                                 nextVertexIndex());
// }
//
// void RenderContext::TessellationWriter::pushCubic(
//     const Vec2D pts[4],
//     gpu::ContourDirections contourDirections,
//     Vec2D joinTangent,
//     uint32_t parametricSegmentCount,
//     uint32_t polarSegmentCount,
//     uint32_t joinSegmentCount,
//     uint32_t contourIDWithFlags)
// {
//     RIVE_PROF_SCOPE_L(3)
//     assert(0 <= parametricSegmentCount &&
//            parametricSegmentCount <= kMaxParametricSegments);
//     assert(0 <= polarSegmentCount && polarSegmentCount <= kMaxPolarSegments);
//     assert(joinSegmentCount > 0);
//     assert((contourIDWithFlags & CONTOUR_ID_MASK) ==
//            (m_flush->m_currentContourID & CONTOUR_ID_MASK));
//     // contourID can't be zero.
//     assert((contourIDWithFlags & CONTOUR_ID_MASK) != 0);
//     // contourID can't be out of range in the contour buffer. (Contour buffer
//     // indices are 1-based.)
//     assert((contourIDWithFlags & CONTOUR_ID_MASK) <=
//            m_flush->desc().contourCount);
//
//     // Polar and parametric segments share the same beginning and ending
//     // vertices, so the merged *vertex* count is equal to the sum of polar and
//     // parametric *segment* counts.
//     uint32_t curveMergedVertexCount =
//         parametricSegmentCount + polarSegmentCount;
//     // -1 because the curve and join share an ending/beginning vertex.
//     uint32_t totalVertexCount = m_nextCubicPaddingVertexCount +
//                                 curveMergedVertexCount + joinSegmentCount - 1;
//
//     // Only the first curve of a contour gets padding vertices.
//     m_nextCubicPaddingVertexCount = 0;
//
//     switch (contourDirections)
//     {
//         case gpu::ContourDirections::forward:
//             pushTessellationSpans(pts,
//                                   joinTangent,
//                                   totalVertexCount,
//                                   parametricSegmentCount,
//                                   polarSegmentCount,
//                                   joinSegmentCount,
//                                   contourIDWithFlags);
//             break;
//         case gpu::ContourDirections::reverse:
//             pushMirroredTessellationSpans(pts,
//                                           joinTangent,
//                                           totalVertexCount,
//                                           parametricSegmentCount,
//                                           polarSegmentCount,
//                                           joinSegmentCount,
//                                           contourIDWithFlags);
//             break;
//         case gpu::ContourDirections::reverseThenForward:
//         case gpu::ContourDirections::forwardThenReverse:
//             // m_pathTessLocation and m_pathMirroredTessLocation are already
//             // configured, so at ths point we don't need to handle
//             // reverseThenForward or forwardThenReverse differently.
//             pushDoubleSidedTessellationSpans(pts,
//                                              joinTangent,
//                                              totalVertexCount,
//                                              parametricSegmentCount,
//                                              polarSegmentCount,
//                                              joinSegmentCount,
//                                              contourIDWithFlags);
//             break;
//     }
// }
//
// RIVE_ALWAYS_INLINE void RenderContext::TessellationWriter::
//     pushTessellationSpans(const Vec2D pts[4],
//                           Vec2D joinTangent,
//                           uint32_t totalVertexCount,
//                           uint32_t parametricSegmentCount,
//                           uint32_t polarSegmentCount,
//                           uint32_t joinSegmentCount,
//                           uint32_t contourIDWithFlags)
// {
//     RIVE_PROF_SCOPE_L(3)
//     assert(totalVertexCount > 0);
//
//     uint32_t y = m_pathTessLocation / kTessTextureWidth;
//     int32_t x0 = m_pathTessLocation % kTessTextureWidth;
//     int32_t x1 = x0 + totalVertexCount;
//     for (;;)
//     {
//         m_tessSpanData.set_back(pts,
//                                 joinTangent,
//                                 static_cast<float>(y),
//                                 x0,
//                                 x1,
//                                 parametricSegmentCount,
//                                 polarSegmentCount,
//                                 joinSegmentCount,
//                                 contourIDWithFlags);
//         if (x1 > static_cast<int32_t>(kTessTextureWidth))
//         {
//             // The span was too long to fit on the current line. Wrap and draw
//             // it again, this time behind the left edge of the texture so we
//             // capture what got clipped off last time.
//             ++y;
//             x0 -= kTessTextureWidth;
//             x1 -= kTessTextureWidth;
//             continue;
//         }
//         break;
//     }
//     assert(y ==
//            (m_pathTessLocation + totalVertexCount - 1) / kTessTextureWidth);
//
//     m_pathTessLocation += totalVertexCount;
//     assert(m_pathTessLocation <= m_expectedPathTessEndLocation);
// }
//
// RIVE_ALWAYS_INLINE void RenderContext::TessellationWriter::
//     pushMirroredTessellationSpans(const Vec2D pts[4],
//                                   Vec2D joinTangent,
//                                   uint32_t totalVertexCount,
//                                   uint32_t parametricSegmentCount,
//                                   uint32_t polarSegmentCount,
//                                   uint32_t joinSegmentCount,
//                                   uint32_t contourIDWithFlags)
// {
//     assert(totalVertexCount > 0);
//
//     uint32_t reflectionY = (m_pathMirroredTessLocation - 1) / kTessTextureWidth;
//     int32_t reflectionX0 =
//         (m_pathMirroredTessLocation - 1) % kTessTextureWidth + 1;
//     int32_t reflectionX1 = reflectionX0 - totalVertexCount;
//
//     for (;;)
//     {
//         m_tessSpanData.set_back(pts,
//                                 joinTangent,
//                                 static_cast<float>(reflectionY),
//                                 reflectionX0,
//                                 reflectionX1,
//                                 parametricSegmentCount,
//                                 polarSegmentCount,
//                                 joinSegmentCount,
//                                 contourIDWithFlags);
//         if (reflectionX1 < 0)
//         {
//             --reflectionY;
//             reflectionX0 += kTessTextureWidth;
//             reflectionX1 += kTessTextureWidth;
//             continue;
//         }
//         break;
//     }
//
//     m_pathMirroredTessLocation -= totalVertexCount;
//     assert(m_pathMirroredTessLocation >= m_expectedPathMirroredTessEndLocation);
// }
//
// RIVE_ALWAYS_INLINE void RenderContext::TessellationWriter::
//     pushDoubleSidedTessellationSpans(const Vec2D pts[4],
//                                      Vec2D joinTangent,
//                                      uint32_t totalVertexCount,
//                                      uint32_t parametricSegmentCount,
//                                      uint32_t polarSegmentCount,
//                                      uint32_t joinSegmentCount,
//                                      uint32_t contourIDWithFlags)
// {
//     assert(totalVertexCount > 0);
//
//     int32_t y = m_pathTessLocation / kTessTextureWidth;
//     int32_t x0 = m_pathTessLocation % kTessTextureWidth;
//     int32_t x1 = x0 + totalVertexCount;
//
//     uint32_t reflectionY = (m_pathMirroredTessLocation - 1) / kTessTextureWidth;
//     int32_t reflectionX0 =
//         (m_pathMirroredTessLocation - 1) % kTessTextureWidth + 1;
//     int32_t reflectionX1 = reflectionX0 - totalVertexCount;
//
//     for (;;)
//     {
//         m_tessSpanData.set_back(pts,
//                                 joinTangent,
//                                 static_cast<float>(y),
//                                 x0,
//                                 x1,
//                                 static_cast<float>(reflectionY),
//                                 reflectionX0,
//                                 reflectionX1,
//                                 parametricSegmentCount,
//                                 polarSegmentCount,
//                                 joinSegmentCount,
//                                 contourIDWithFlags);
//         if (x1 > static_cast<int32_t>(kTessTextureWidth) || reflectionX1 < 0)
//         {
//             // Either the span or its reflection was too long to fit on the
//             // current line. Wrap and draw both of them again, this time beyond
//             // the opposite edge of the texture so we capture what got clipped
//             // off last time.
//             ++y;
//             x0 -= kTessTextureWidth;
//             x1 -= kTessTextureWidth;
//
//             --reflectionY;
//             reflectionX0 += kTessTextureWidth;
//             reflectionX1 += kTessTextureWidth;
//             continue;
//         }
//         break;
//     }
//
//     m_pathTessLocation += totalVertexCount;
//     assert(m_pathTessLocation <= m_expectedPathTessEndLocation);
//
//     m_pathMirroredTessLocation -= totalVertexCount;
//     assert(m_pathMirroredTessLocation >= m_expectedPathMirroredTessEndLocation);
// }
//
// void RenderContext::LogicalFlush::pushPaddingVertices(uint32_t count,
//                                                       uint32_t tessLocation)
// {
//     RIVE_PROF_SCOPE_L(3)
//     assert(m_hasDoneLayout);
//     assert(count > 0);
//
//     constexpr static Vec2D kEmptyCubic[4]{};
//     TessellationWriter(this,
//                        /*pathID=*/0,
//                        gpu::ContourDirections::forward,
//                        count,
//                        tessLocation)
//         .pushTessellationSpans(kEmptyCubic,
//                                {0, 0},
//                                count,
//                                0,
//                                0,
//                                1,
//                                INVALID_CONTOUR_ID_WITH_FLAGS);
// }
//
// gpu::DrawBatch& RenderContext::LogicalFlush::pushMidpointFanDraw(
//     const PathDraw* draw,
//     gpu::DrawType drawType,
//     uint32_t tessVertexCount,
//     uint32_t tessLocation,
//     gpu::ShaderMiscFlags shaderMiscFlags)
// {
//     RIVE_PROF_SCOPE_L(2)
//     assert(m_hasDoneLayout);
//
//     uint32_t baseInstance = math::lossless_numeric_cast<uint32_t>(
//         tessLocation / kMidpointFanPatchSegmentSpan);
//     // flush() is responsible for alignment.
//     assert(baseInstance * kMidpointFanPatchSegmentSpan == tessLocation);
//
//     uint32_t instanceCount = tessVertexCount / kMidpointFanPatchSegmentSpan;
//     // flush() is responsible for alignment.
//     assert(instanceCount * kMidpointFanPatchSegmentSpan == tessVertexCount);
//
//     return pushPathDraw(draw,
//                         drawType,
//                         shaderMiscFlags,
//                         instanceCount,
//                         baseInstance);
// }
//
// gpu::DrawBatch& RenderContext::LogicalFlush::pushOuterCubicsDraw(
//     const PathDraw* draw,
//     gpu::DrawType drawType,
//     uint32_t tessVertexCount,
//     uint32_t tessLocation,
//     gpu::ShaderMiscFlags shaderMiscFlags)
// {
//     RIVE_PROF_SCOPE_L(2)
//     assert(m_hasDoneLayout);
//
//     uint32_t baseInstance = math::lossless_numeric_cast<uint32_t>(
//         tessLocation / kOuterCurvePatchSegmentSpan);
//     // flush() is responsible for alignment.
//     assert(baseInstance * kOuterCurvePatchSegmentSpan == tessLocation);
//
//     uint32_t instanceCount = tessVertexCount / kOuterCurvePatchSegmentSpan;
//     // flush() is responsible for alignment.
//     assert(instanceCount * kOuterCurvePatchSegmentSpan == tessVertexCount);
//
//     return pushPathDraw(draw,
//                         drawType,
//                         shaderMiscFlags,
//                         instanceCount,
//                         baseInstance);
// }
//
// gpu::DrawBatch* RenderContext::LogicalFlush::pushInteriorTriangulationDraw(
//     const PathDraw* draw,
//     uint32_t pathID,
//     gpu::WindingFaces windingFaces,
//     gpu::ShaderMiscFlags shaderMiscFlags RIVE_DEBUG_CODE(,
//                                                          size_t* vertexCounter))
// {
//     RIVE_PROF_SCOPE_L(2)
//     assert(m_hasDoneLayout);
//     assert(pathID != 0);
//
//     uint32_t baseVertex = math::lossless_numeric_cast<uint32_t>(
//         m_ctx->m_triangleVertexData.elementsWritten());
//     size_t actualVertexCount = draw->triangulator()->polysToTriangles(
//         pathID,
//         draw->triangulatorFillRule(),
//         draw->triangulatorReverseTriangles(),
//         draw->triangulatorNegateWinding(),
//         windingFaces,
//         &m_ctx->m_triangleVertexData);
//     assert(baseVertex + actualVertexCount ==
//            m_ctx->m_triangleVertexData.elementsWritten());
//     RIVE_DEBUG_CODE(*vertexCounter += actualVertexCount;)
//     if (actualVertexCount > 0)
//     {
//         return &pushPathDraw(
//             draw,
//             DrawType::interiorTriangulation,
//             shaderMiscFlags,
//             math::lossless_numeric_cast<uint32_t>(actualVertexCount),
//             baseVertex);
//     }
//     return nullptr;
// }
//
// gpu::DrawBatch& RenderContext::LogicalFlush::pushFeatherAtlasBlit(
//     PathDraw* draw,
//     uint32_t pathID)
// {
//     RIVE_PROF_SCOPE_L(2)
//     auto baseVertex = math::lossless_numeric_cast<uint32_t>(
//         m_ctx->m_triangleVertexData.elementsWritten());
//     auto [l, t, r, b] = AABB(draw->pixelBounds());
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{l, b}, 1, pathID);
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{l, t}, 1, pathID);
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{r, b}, 1, pathID);
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{r, b}, 1, pathID);
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{l, t}, 1, pathID);
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{r, t}, 1, pathID);
//     return pushPathDraw(draw,
//                         DrawType::featherAtlasBlit,
//                         m_baselineShaderMiscFlags,
//                         6,
//                         baseVertex);
// }
//
// gpu::DrawBatch& RenderContext::LogicalFlush::pushImageRectDraw(
//     ImageRectDraw* draw)
// {
//     RIVE_PROF_SCOPE_L(2)
//     assert(m_hasDoneLayout);
//
//     // If we support image paints for paths, the client should use pushPath()
//     // with an image paint instead of calling this method.
//     assert(!m_ctx->frameSupportsImagePaintForPaths());
//
//     const uint32_t imageDrawBaseInstance =
//         math::lossless_numeric_cast<uint32_t>(
//             m_ctx->m_imageDrawInstanceData.elementsWritten());
//     m_ctx->m_imageDrawInstanceData.emplace_back(draw->matrix(),
//                                                 draw->opacity(),
//                                                 draw->clipRectInverseMatrix(),
//                                                 draw->clipID(),
//                                                 draw->blendMode(),
//                                                 m_currentZIndex);
//
//     DrawBatch& batch = pushDraw(draw,
//                                 DrawType::imageRect,
//                                 m_baselineShaderMiscFlags,
//                                 PaintType::image,
//                                 1,
//                                 imageDrawBaseInstance);
//     return batch;
// }
//
// gpu::DrawBatch& RenderContext::LogicalFlush::pushImageMeshDraw(
//     ImageMeshDraw* draw)
// {
//     RIVE_PROF_SCOPE_L(2)
//     assert(m_hasDoneLayout);
//
//     const uint32_t imageDrawBaseInstance =
//         math::lossless_numeric_cast<uint32_t>(
//             m_ctx->m_imageDrawInstanceData.elementsWritten());
//     m_ctx->m_imageDrawInstanceData.emplace_back(draw->matrix(),
//                                                 draw->opacity(),
//                                                 draw->clipRectInverseMatrix(),
//                                                 draw->clipID(),
//                                                 draw->blendMode(),
//                                                 m_currentZIndex);
//
//     DrawBatch& batch = pushDraw(draw,
//                                 DrawType::imageMesh,
//                                 m_baselineShaderMiscFlags,
//                                 PaintType::image,
//                                 1, // one instance (the mesh)
//                                 imageDrawBaseInstance);
//     batch.indexCountPerInstance = draw->indexCount();
//     batch.vertexBuffer = draw->vertexBuffer();
//     batch.uvBuffer = draw->uvBuffer();
//     batch.indexBuffer = draw->indexBuffer();
//     return batch;
// }
//
// gpu::DrawBatch& RenderContext::LogicalFlush::pushClipResetDraw(ClipReset* draw)
// {
//     RIVE_PROF_SCOPE_L(2)
//     assert(m_hasDoneLayout);
//
//     uint32_t baseVertex = math::lossless_numeric_cast<uint32_t>(
//         m_ctx->m_triangleVertexData.elementsWritten());
//     auto [l, t, r, b] = AABB(getClipInfo(draw->previousClipID()).contentBounds);
//     uint32_t z = m_currentZIndex;
//     assert(AABB(l, t, r, b).round() == draw->pixelBounds());
//     assert(draw->resourceCounts().maxTriangleVertexCount == 6);
//     assert(m_ctx->m_triangleVertexData.hasRoomFor(6));
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{l, b}, 0, z);
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{l, t}, 0, z);
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{r, b}, 0, z);
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{r, b}, 0, z);
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{l, t}, 0, z);
//     m_ctx->m_triangleVertexData.emplace_back(Vec2D{r, t}, 0, z);
//     return pushDraw(draw,
//                     DrawType::clipReset,
//                     gpu::ShaderMiscFlags::none,
//                     PaintType::clipUpdate,
//                     6,
//                     baseVertex);
// }
//
// gpu::DrawBatch& RenderContext::LogicalFlush::pushPathDraw(
//     const PathDraw* draw,
//     DrawType drawType,
//     gpu::ShaderMiscFlags shaderMiscFlags,
//     uint32_t vertexCount,
//     uint32_t baseVertex)
// {
//     RIVE_PROF_SCOPE_L(3)
//     assert(m_hasDoneLayout);
//
//     // Clockwise fills get their own shaders in rasterOrdering mode.
//     // TODO: eventually we will use draw_clockwise_path.frag for these
//     // draws in rasterOrdering mode, instead of just making a variant of
//     // draw_raster_order_path.frag.
//     if (m_ctx->frameInterlockMode() == gpu::InterlockMode::rasterOrdering &&
//         enums::is_flag_set(draw->drawContents(),
//                            gpu::DrawContents::clockwiseFill))
//     {
//         shaderMiscFlags |= gpu::ShaderMiscFlags::clockwiseFill;
//     }
//
//     DrawBatch& batch = pushDraw(draw,
//                                 drawType,
//                                 shaderMiscFlags,
//                                 draw->paintType(),
//                                 vertexCount,
//                                 baseVertex);
//
//     auto pathShaderFeatures = gpu::ShaderFeatures::NONE;
//     if (draw->featherRadius() != 0 &&
//         drawType != gpu::DrawType::interiorTriangulation &&
//         drawType != gpu::DrawType::featherAtlasBlit)
//     {
//         pathShaderFeatures |= ShaderFeatures::ENABLE_FEATHER;
//     }
//     if (enums::is_flag_set(draw->drawContents(),
//                            gpu::DrawContents::evenOddFill))
//     {
//         assert(!enums::is_flag_set(batch.shaderMiscFlags,
//                                    gpu::ShaderMiscFlags::clockwiseFill));
//         pathShaderFeatures |= ShaderFeatures::ENABLE_EVEN_ODD;
//     }
//     constexpr static gpu::DrawContents NESTED_CLIP_FLAGS =
//         gpu::DrawContents::clipUpdate | gpu::DrawContents::activeClip;
//     if ((draw->drawContents() & NESTED_CLIP_FLAGS) == NESTED_CLIP_FLAGS)
//     {
//         pathShaderFeatures |= ShaderFeatures::ENABLE_NESTED_CLIPPING;
//     }
//     batch.shaderFeatures |=
//         pathShaderFeatures & m_ctx->m_frameShaderFeaturesMask;
//     assert(
//         (batch.shaderFeatures &
//          gpu::ShaderFeaturesMaskFor(drawType, m_ctx->frameInterlockMode())) ==
//         batch.shaderFeatures);
//     m_combinedShaderFeatures |= batch.shaderFeatures;
//     return batch;
// }
//
// RIVE_ALWAYS_INLINE static bool can_combine_shader_misc_flags(
//     const gpu::DrawBatch* batch,
//     const Draw* draw,
//     gpu::ShaderMiscFlags shaderMiscFlags)
// {
//     // If a path doesn't have ANY_PATH_FILL bits, it means it's a stroke.
//     constexpr static auto ANY_PATH_FILL = gpu::DrawContents::clockwiseFill |
//                                           gpu::DrawContents::evenOddFill |
//                                           gpu::DrawContents::nonZeroFill;
//
//     gpu::ShaderMiscFlags compareMask = ~gpu::ShaderMiscFlags::none;
//
//     // Strokes draw identically in the clockwise and legacy shaders, so strokes
//     // can be combined with paths of any fill type.
//     if ((enums::no_flags_set(batch->drawContents, ANY_PATH_FILL) ||
//          enums::no_flags_set(draw->drawContents(), ANY_PATH_FILL)))
//     {
//         compareMask &= ~gpu::ShaderMiscFlags::clockwiseFill;
//     }
//
//     return (batch->shaderMiscFlags & compareMask) ==
//            (shaderMiscFlags & compareMask);
// }
//
// RIVE_ALWAYS_INLINE static bool can_combine_draw_images(
//     const Texture* currentDrawTexture,
//     const Texture* nextDrawTexture,
//     const ImageSampler currentImageSamplerKey,
//     const ImageSampler nextImageSamplerKey)
// {
//     if (currentDrawTexture == nullptr || nextDrawTexture == nullptr)
//     {
//         // We can always combine two draws if one or both do not use an image
//         // paint.
//         return true;
//     }
//     // Since the image paint's texture must be bound to a specific slot, we
//     // can't combine draws that use different textures.
//     return (currentDrawTexture == nextDrawTexture) &&
//            (currentImageSamplerKey == nextImageSamplerKey);
// }
//
// constexpr uint32_t patchIndexCount(DrawType drawType)
// {
//     switch (drawType)
//     {
//         case DrawType::midpointFanPatches:
//             return kMidpointFanPatchIndexCount;
//         case DrawType::midpointFanCenterAAPatches:
//             return kMidpointFanCenterAAPatchIndexCount;
//         case DrawType::outerCurvePatches:
//             return kOuterCurvePatchIndexCount;
//         case DrawType::msaaStrokes:
//             return kMidpointFanPatchBorderIndexCount;
//         case DrawType::msaaMidpointFanBorrowedCoverage:
//         case DrawType::msaaDynamicMidpointFans:
//         case DrawType::msaaMidpointFans:
//         case DrawType::msaaMidpointFanStencilReset:
//         case DrawType::msaaMidpointFanPathsStencil:
//         case DrawType::msaaMidpointFanPathsCover:
//             return kMidpointFanPatchIndexCount -
//                    kMidpointFanPatchBorderIndexCount;
//         case DrawType::msaaOuterCubics:
//             return kOuterCurvePatchIndexCount -
//                    kOuterCurvePatchBorderIndexCount;
//         case DrawType::interiorTriangulation:
//         case DrawType::featherAtlasBlit:
//         case DrawType::imageRect:
//         case DrawType::imageMesh:
//         case DrawType::clipReset:
//         case DrawType::renderPassInitialize:
//         case DrawType::renderPassResolve:
//             RIVE_UNREACHABLE();
//     }
//     RIVE_UNREACHABLE();
// }
//
// constexpr uint32_t patchBaseIndex(DrawType drawType)
// {
//     switch (drawType)
//     {
//         case DrawType::midpointFanPatches:
//         case DrawType::msaaStrokes:
//             return kMidpointFanPatchBaseIndex;
//         case DrawType::midpointFanCenterAAPatches:
//             return kMidpointFanCenterAAPatchBaseIndex;
//         case DrawType::outerCurvePatches:
//             return kOuterCurvePatchBaseIndex;
//         case DrawType::msaaMidpointFanBorrowedCoverage:
//         case DrawType::msaaDynamicMidpointFans:
//         case DrawType::msaaMidpointFans:
//         case DrawType::msaaMidpointFanStencilReset:
//         case DrawType::msaaMidpointFanPathsStencil:
//         case DrawType::msaaMidpointFanPathsCover:
//             return kMidpointFanPatchBaseIndex +
//                    kMidpointFanPatchBorderIndexCount;
//         case DrawType::msaaOuterCubics:
//             return kOuterCurvePatchBaseIndex + kOuterCurvePatchBorderIndexCount;
//         case DrawType::interiorTriangulation:
//         case DrawType::featherAtlasBlit:
//         case DrawType::imageRect:
//         case DrawType::imageMesh:
//         case DrawType::clipReset:
//         case DrawType::renderPassInitialize:
//         case DrawType::renderPassResolve:
//             RIVE_UNREACHABLE();
//     }
//     RIVE_UNREACHABLE();
// }
//
// static void assignDrawIndices(DrawType drawType, gpu::DrawBatch* batch)
// {
//     switch (drawType)
//     {
//         case DrawType::midpointFanPatches:
//         case DrawType::midpointFanCenterAAPatches:
//         case DrawType::outerCurvePatches:
//         case DrawType::msaaStrokes:
//         case DrawType::msaaMidpointFanBorrowedCoverage:
//         case DrawType::msaaDynamicMidpointFans:
//         case DrawType::msaaMidpointFans:
//         case DrawType::msaaMidpointFanStencilReset:
//         case DrawType::msaaMidpointFanPathsStencil:
//         case DrawType::msaaMidpointFanPathsCover:
//         case DrawType::msaaOuterCubics:
//             batch->indexCountPerInstance = patchIndexCount(drawType);
//             batch->baseIndex = patchBaseIndex(drawType);
//             break;
//         case DrawType::imageRect:
//             batch->indexCountPerInstance = std::size(kImageRectIndices);
//             batch->baseIndex = 0;
//             break;
//         case DrawType::imageMesh:
//         case DrawType::interiorTriangulation:
//         case DrawType::featherAtlasBlit:
//         case DrawType::clipReset:
//         case DrawType::renderPassInitialize:
//         case DrawType::renderPassResolve:
//             batch->indexCountPerInstance = 0;
//             batch->baseIndex = 0;
//             break;
//         default:
//             RIVE_UNREACHABLE();
//     }
// }
//
// gpu::DrawBatch& RenderContext::LogicalFlush::pushDraw(
//     const Draw* draw,
//     DrawType drawType,
//     gpu::ShaderMiscFlags shaderMiscFlags,
//     gpu::PaintType paintType,
//     uint32_t elementCount,
//     uint32_t baseElement)
// {
//     RIVE_PROF_SCOPE_L(3)
//     assert(m_hasDoneLayout);
//     assert(elementCount > 0);
//
//     shaderMiscFlags |= m_baselineShaderMiscFlags;
//
//     if ((m_ctx->frameInterlockMode() == gpu::InterlockMode::clockwise ||
//          (m_ctx->frameInterlockMode() == gpu::InterlockMode::clockwiseAtomic &&
//           !enums::is_flag_set(shaderMiscFlags,
//                               gpu::ShaderMiscFlags::borrowedCoveragePass))) &&
//         enums::is_flag_set(draw->drawContents(), gpu::DrawContents::clipUpdate))
//     {
//         // Clockwise modes give clip updates a dedicated draw by setting
//         // gpu::ShaderMiscFlags::clipUpdateOnly.
//         shaderMiscFlags |= gpu::ShaderMiscFlags::clipUpdateOnly;
//         if (m_ctx->frameInterlockMode() ==
//                 gpu::InterlockMode::clockwiseAtomic &&
//             enums::is_flag_set(draw->drawContents(),
//                                gpu::DrawContents::activeClip))
//         {
//             // clockwiseAtomic takes it a step futher and separates out nested
//             // clip updates into their own draw type.
//             shaderMiscFlags |= gpu::ShaderMiscFlags::nestedClipUpdateOnly;
//         }
//     }
//
//     // In clockwiseAtomic and msaa modes, individual draws can use
//     // fixedFunctionColorOutput even if the render pass as a whole does not.
//     if (m_ctx->frameInterlockMode() == gpu::InterlockMode::clockwiseAtomic)
//     {
//         if (enums::is_flag_set(shaderMiscFlags,
//                                gpu::ShaderMiscFlags::borrowedCoveragePass) ||
//             draw->blendMode() == BlendMode::srcOver)
//         {
//             shaderMiscFlags |= gpu::ShaderMiscFlags::fixedFunctionColorOutput;
//         }
//     }
//     else if (m_ctx->frameInterlockMode() == gpu::InterlockMode::msaa &&
//              draw->blendMode() == BlendMode::srcOver)
//     {
//         shaderMiscFlags |= gpu::ShaderMiscFlags::fixedFunctionColorOutput;
//     }
//
//     bool canMergeWithPreviousBatch;
//     switch (drawType)
//     {
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
//             if (!m_drawList.empty() &&
//                 !enums::is_flag_set(m_pendingBarriers,
//                                     gpu::BarrierFlags::drawBatchBreak))
//             {
//                 const DrawBatch* currentBatch = m_drawList.tail();
//                 canMergeWithPreviousBatch =
//                     currentBatch->drawType == drawType &&
//                     can_combine_shader_misc_flags(currentBatch,
//                                                   draw,
//                                                   shaderMiscFlags) &&
//                     can_combine_draw_images(currentBatch->imageTexture,
//                                             draw->imageTexture(),
//                                             currentBatch->imageSampler,
//                                             draw->imageSampler());
//                 if (canMergeWithPreviousBatch &&
//                     currentBatch->baseElement + currentBatch->elementCount !=
//                         baseElement)
//                 {
//                     // In MSAA mode, multiple subpasses reference the same
//                     // tessellation data. Although rare, this breaks the
//                     // guarantee we have in other modes that mergeable batches
//                     // will always have contiguous patches.
//                     assert(m_ctx->frameInterlockMode() ==
//                            gpu::InterlockMode::msaa);
//                     canMergeWithPreviousBatch = false;
//                 }
//
//                 // Also break if there is a scissor rect mismatch
//                 if (m_ctx->platformFeatures().supportsClipScissor)
//                 {
//                     if (currentBatch->scissorRect != draw->scissorRect())
//                     {
//                         canMergeWithPreviousBatch = false;
//                     }
//                 }
//                 break;
//             }
//             [[fallthrough]];
//
//         // Image draws can't be combined for now because they each have their
//         // own unique uniforms.
//         case DrawType::imageRect:
//         case DrawType::imageMesh:
//         case DrawType::renderPassInitialize:
//         case DrawType::renderPassResolve:
//             canMergeWithPreviousBatch = false;
//             break;
//     }
//
//     DrawBatch* batch;
//     if (!canMergeWithPreviousBatch)
//     {
//         batch = m_drawList.emplace_back(m_ctx->perFrameAllocator(),
//                                         drawType,
//                                         shaderMiscFlags,
//                                         draw->drawContents(),
//                                         elementCount,
//                                         baseElement,
//                                         draw->blendMode(),
//                                         draw->imageSampler(),
//                                         m_pendingBarriers);
//         assignDrawIndices(drawType, batch);
//     }
//     else
//     {
//         batch = m_drawList.tail();
//         assert(batch->drawType == drawType);
//         assert(batch->baseElement + batch->elementCount == baseElement);
//
//         batch->elementCount += elementCount;
//
//         // clockwise doesn't mix regular draws and clip updates.
//         assert(m_ctx->frameInterlockMode() != gpu::InterlockMode::clockwise ||
//                (batch->drawContents & gpu::DrawContents::clipUpdate) ==
//                    (draw->drawContents() & gpu::DrawContents::clipUpdate));
//
//         // Feathered fills should never combine with fills, strokes, or
//         // feathered strokes because they use a different DrawType.
//         assert((batch->drawContents & gpu::DrawContents::featheredFill) ==
//                (draw->drawContents() & gpu::DrawContents::featheredFill));
//
//         // msaa can't mix drawContents in a batch.
//         assert(m_ctx->frameInterlockMode() != gpu::InterlockMode::msaa ||
//                batch->drawContents == draw->drawContents());
//
//         batch->shaderMiscFlags |= shaderMiscFlags;
//         batch->drawContents |= draw->drawContents();
//         batch->barriers |= m_pendingBarriers;
//     }
//     m_pendingBarriers = gpu::BarrierFlags::none;
//
//     // If the batch was merged into a previous one, this ensures it was a valid
//     // merge.
//     assert(batch->drawType == drawType);
//     assert(can_combine_draw_images(batch->imageTexture,
//                                    draw->imageTexture(),
//                                    batch->imageSampler,
//                                    draw->imageSampler()));
//     assert(m_pendingBarriers == BarrierFlags::none);
//
//     auto shaderFeatures = ShaderFeatures::NONE;
//     if (draw->clipID() != 0)
//     {
//         shaderFeatures |= ShaderFeatures::ENABLE_CLIPPING;
//     }
//     if (draw->hasClipRect() && paintType != PaintType::clipUpdate)
//     {
//         shaderFeatures |= ShaderFeatures::ENABLE_CLIP_RECT;
//     }
//
//     if (frameDescriptor().ditherMode ==
//         gpu::DitherMode::interleavedGradientNoise)
//     {
//         shaderFeatures |= ShaderFeatures::ENABLE_DITHER;
//     }
//
//     if (paintType != PaintType::clipUpdate &&
//         !enums::is_flag_set(shaderMiscFlags,
//                             gpu::ShaderMiscFlags::borrowedCoveragePass))
//     {
//         assert(!enums::is_flag_set(shaderMiscFlags,
//                                    gpu::ShaderMiscFlags::clipUpdateOnly));
//         switch (draw->blendMode())
//         {
//             case BlendMode::hue:
//             case BlendMode::saturation:
//             case BlendMode::color:
//             case BlendMode::luminosity:
//                 shaderFeatures |= ShaderFeatures::ENABLE_HSL_BLEND_MODES;
//                 [[fallthrough]];
//             case BlendMode::screen:
//             case BlendMode::overlay:
//             case BlendMode::darken:
//             case BlendMode::lighten:
//             case BlendMode::colorDodge:
//             case BlendMode::colorBurn:
//             case BlendMode::hardLight:
//             case BlendMode::softLight:
//             case BlendMode::difference:
//             case BlendMode::exclusion:
//             case BlendMode::multiply:
//                 shaderFeatures |= ShaderFeatures::ENABLE_ADVANCED_BLEND;
//                 break;
//             case BlendMode::srcOver:
//                 break;
//         }
//     }
//     batch->shaderFeatures |= shaderFeatures & m_ctx->m_frameShaderFeaturesMask;
//     assert(
//         (batch->shaderFeatures &
//          gpu::ShaderFeaturesMaskFor(drawType, m_ctx->frameInterlockMode())) ==
//         batch->shaderFeatures);
//
//     if (paintType == PaintType::image)
//     {
//         assert(draw->imageTexture() != nullptr);
//         if (batch->imageTexture == nullptr)
//         {
//             batch->imageTexture = draw->imageTexture();
//         }
//         assert(batch->imageTexture == draw->imageTexture());
//     }
//
//     m_combinedShaderFeatures |= batch->shaderFeatures;
//     return *batch;
// }
// } // namespace rive::gpu

// Executable correspondence for the pinned implementation. Keep this section
// in the same function order as the retained source above.
use core::marker::PhantomPinned;
use core::mem::ManuallyDrop;
use core::pin::Pin;
use core::ptr::NonNull;
use std::collections::HashMap;

use crate::mechanical_port::source::include::rive::factory_hpp::{
    Factory, FactoryAccess, FactoryContract, OreContext,
};
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderImage, RenderPaint, RenderPath, RenderShader,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::*;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImplContract;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RiveRenderBufferHandle;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_factory_hpp::{
    RiveRenderFactory, RiveRenderFactoryAccess, RiveRenderFactoryContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::{
    RiveRenderImage, RiveRenderImageHandle,
};
use nuxie_render_api::{FillRule, RawPath};

const K_DEFAULT_DRAW_CAPACITY: usize = 2048;
const K_DEFAULT_SIMPLE_GRADIENT_CAPACITY: usize = 512;
const K_DEFAULT_COMPLEX_GRADIENT_CAPACITY: usize = 1024;
const K_MAX_TEXTURE_HEIGHT: usize = 2048;
const K_MAX_TESSELLATION_VERTEX_COUNT: usize = K_MAX_TEXTURE_HEIGHT * gpu::kTessTextureWidth;
const K_MAX_TESSELLATION_PADDING_VERTEX_COUNT: usize = gpu::kMidpointFanPatchSegmentSpan as usize
    + (gpu::kOuterCurvePatchSegmentSpan as usize - 1)
    + 1;
const K_MAX_TESSELLATION_VERTEX_COUNT_BEFORE_PADDING: usize =
    K_MAX_TESSELLATION_VERTEX_COUNT - K_MAX_TESSELLATION_PADDING_VERTEX_COUNT;
const K_MAX_REORDERED_DRAW_PASS_COUNT: i32 = i16::MAX as i32;
const GRAD_SPAN_FLAG_LEFT_BORDER: u32 = 0x8000_0000;
const GRAD_SPAN_FLAG_RIGHT_BORDER: u32 = 0x4000_0000;
const GRAD_SPAN_FLAG_COMPLEX_BORDER: u32 = 0x2000_0000;
const SORT_SUBPASS_SHIFT: u32 = 0;
const SORT_CONTENTS_SHIFT: u32 = 3;
const SORT_BLEND_SHIFT: u32 = 12;
const SORT_SCISSOR_SHIFT: u32 = 16;
const SORT_TEXTURE_SHIFT: u32 = 31;
const SORT_DRAW_TYPE_SHIFT: u32 = 45;
const SORT_GROUP_SHIFT: u32 = 48;
const SORT_GROUP_MASK: i64 = 0x7fff_i64 << SORT_GROUP_SHIFT;
const SORT_CONTENTS_MASK: i64 = 0x1ff_i64 << SORT_CONTENTS_SHIFT;
const SORT_BLEND_MASK: i64 = 0xf_i64 << SORT_BLEND_SHIFT;

fn pls_blend_mode(mode: nuxie_render_api::BlendMode) -> u64 {
    if mode == nuxie_render_api::BlendMode::SrcOver {
        0
    } else {
        mode as u64 - 13
    }
}

fn make_sort_key(draw: &Draw, group: i16, scissor: i16, subpass: i8) -> i64 {
    let texture_hash = if draw.imageTexture().is_null() {
        0
    } else {
        unsafe { (&*draw.imageTexture()).textureResourceHash() }
    };
    (((group as u64) & 0x7fff) << SORT_GROUP_SHIFT
        | ((draw.r#type() as u64) & 7) << SORT_DRAW_TYPE_SHIFT
        | ((texture_hash as u64) & 0x3fff) << SORT_TEXTURE_SHIFT
        | ((scissor as u64) & 0x7fff) << SORT_SCISSOR_SHIFT
        | (pls_blend_mode(draw.blendMode()) & 0xf) << SORT_BLEND_SHIFT
        | ((draw.drawContents().0 as u64) & 0x1ff) << SORT_CONTENTS_SHIFT
        | ((subpass as u64) & 7) << SORT_SUBPASS_SHIFT) as i64
}

fn patch_indices(draw_type: gpu::DrawType) -> (u32, u32) {
    use gpu::DrawType::*;
    match draw_type {
        midpointFanPatches => (
            gpu::kMidpointFanPatchIndexCount,
            gpu::kMidpointFanPatchBaseIndex,
        ),
        midpointFanCenterAAPatches => (
            gpu::kMidpointFanCenterAAPatchIndexCount,
            gpu::kMidpointFanCenterAAPatchBaseIndex,
        ),
        outerCurvePatches => (
            gpu::kOuterCurvePatchIndexCount,
            gpu::kOuterCurvePatchBaseIndex,
        ),
        msaaStrokes => (
            gpu::kMidpointFanPatchBorderIndexCount,
            gpu::kMidpointFanPatchBaseIndex,
        ),
        msaaMidpointFanBorrowedCoverage
        | msaaDynamicMidpointFans
        | msaaMidpointFans
        | msaaMidpointFanStencilReset
        | msaaMidpointFanPathsStencil
        | msaaMidpointFanPathsCover => (
            gpu::kMidpointFanPatchIndexCount - gpu::kMidpointFanPatchBorderIndexCount,
            gpu::kMidpointFanPatchBaseIndex + gpu::kMidpointFanPatchBorderIndexCount,
        ),
        msaaOuterCubics => (
            gpu::kOuterCurvePatchIndexCount - gpu::kOuterCurvePatchBorderIndexCount,
            gpu::kOuterCurvePatchBaseIndex + gpu::kOuterCurvePatchBorderIndexCount,
        ),
        imageRect => (gpu::kImageRectIndices.len() as u32, 0),
        imageMesh
        | interiorTriangulation
        | featherAtlasBlit
        | clipReset
        | renderPassInitialize
        | renderPassResolve => (0, 0),
    }
}

fn resource_texture_height(item_count: usize, width_in_items: usize) -> usize {
    (item_count + width_in_items - 1) / width_in_items
}

fn padding_to_align_up(value: usize, alignment: usize) -> usize {
    debug_assert_ne!(alignment, 0);
    (alignment - value % alignment) % alignment
}

fn intersect_gpu_bounds(a: gpu::IAABB, b: IAABB) -> gpu::IAABB {
    gpu::IAABB {
        left: a.left.max(b.left),
        top: a.top.max(b.top),
        right: a.right.min(b.right),
        bottom: a.bottom.min(b.bottom),
    }
}
fn intersect_u16(a: AABBu16, b: AABBu16) -> AABBu16 {
    AABBu16 {
        left: a.left.max(b.left),
        top: a.top.max(b.top),
        right: a.right.min(b.right),
        bottom: a.bottom.min(b.bottom),
    }
}
fn join_u16(a: AABBu16, b: AABBu16) -> AABBu16 {
    AABBu16 {
        left: a.left.min(b.left),
        top: a.top.min(b.top),
        right: a.right.max(b.right),
        bottom: a.bottom.max(b.bottom),
    }
}
fn clamp_bounds_u16(a: IAABB) -> AABBu16 {
    AABBu16 {
        left: a.left.clamp(0, u16::MAX as i32) as u16,
        top: a.top.clamp(0, u16::MAX as i32) as u16,
        right: a.right.clamp(0, u16::MAX as i32) as u16,
        bottom: a.bottom.clamp(0, u16::MAX as i32) as u16,
    }
}
fn join_i32(a: IAABB, b: IAABB) -> IAABB {
    IAABB {
        left: a.left.min(b.left),
        top: a.top.min(b.top),
        right: a.right.max(b.right),
        bottom: a.bottom.max(b.bottom),
    }
}

fn gradient_data_height(simple_ramp_count: usize, complex_ramp_count: usize) -> usize {
    resource_texture_height(
        simple_ramp_count,
        gpu::kGradTextureWidthInSimpleRamps as usize,
    ) + complex_ramp_count
}

fn pls_transient_backing_plane_count(
    interlock: gpu::InterlockMode,
    contents: gpu::DrawContents,
) -> u32 {
    match interlock {
        gpu::InterlockMode::rasterOrdering => 3,
        gpu::InterlockMode::atomics | gpu::InterlockMode::clockwiseAtomic => 1,
        gpu::InterlockMode::clockwise => {
            let mut count = 1;
            if (contents.0 & (gpu::DrawContents::activeClip | gpu::DrawContents::clipUpdate).0) != 0
            {
                count += 1;
            }
            if (contents.0 & gpu::DrawContents::advancedBlend.0) != 0 {
                count += 1;
            }
            count
        }
        gpu::InterlockMode::msaa => 0,
    }
}

fn wants_fixed_function_color_output(
    features: &gpu::PlatformFeatures,
    interlock: gpu::InterlockMode,
    contents: gpu::DrawContents,
    manually_resolved: bool,
) -> bool {
    let advanced = (contents.0 & gpu::DrawContents::advancedBlend.0) != 0;
    match interlock {
        gpu::InterlockMode::rasterOrdering => false,
        gpu::InterlockMode::atomics | gpu::InterlockMode::clockwiseAtomic => !advanced,
        gpu::InterlockMode::clockwise => {
            debug_assert_eq!(
                contents.0 & (gpu::DrawContents::nonZeroFill | gpu::DrawContents::evenOddFill).0,
                0
            );
            features.supportsClockwiseFixedFunctionMode && !advanced
        }
        gpu::InterlockMode::msaa => !manually_resolved && !advanced,
    }
}

fn empty_flush_descriptor() -> gpu::FlushDescriptor {
    gpu::FlushDescriptor {
        renderTarget: None,
        combinedShaderFeatures: gpu::ShaderFeatures::NONE,
        interlockMode: gpu::InterlockMode::rasterOrdering,
        msaaSampleCount: 0,
        colorLoadAction: gpu::LoadAction::clear,
        colorClearValue: 0,
        coverageClearValue: 0,
        depthClearValue: gpu::DEPTH_MAX,
        stencilClearValue: gpu::STENCIL_CLEAR,
        renderTargetUpdateBounds: gpu::IAABB::default(),
        virtualTileWidth: 0,
        virtualTileHeight: 0,
        manuallyResolved: false,
        fixedFunctionColorOutput: false,
        featherAtlasTextureWidth: 0,
        featherAtlasTextureHeight: 0,
        featherAtlasContentWidth: 0,
        featherAtlasContentHeight: 0,
        coverageBufferPrefix: 0,
        needsCoverageBufferClear: false,
        flushUniformDataOffsetInBytes: 0,
        pathCount: 0,
        firstPath: 0,
        firstPaint: 0,
        firstPaintAux: 0,
        contourCount: 0,
        firstContour: 0,
        gradSpanCount: 0,
        firstGradSpan: 0,
        tessVertexSpanCount: 0,
        firstTessVertexSpan: 0,
        gradDataHeight: 0,
        tessDataHeight: 0,
        clockwiseFillOverride: false,
        hasTriangleVertices: false,
        wireframe: false,
        ditherMode: DitherMode::interleavedGradientNoise,
        #[cfg(feature = "with-rive-tools")]
        synthesizedFailureType: gpu::SynthesizedFailureType::none,
        externalCommandBuffer: None,
        featherAtlasFillBatches: None,
        featherAtlasFillBatchCount: 0,
        featherAtlasStrokeBatches: None,
        featherAtlasStrokeBatchCount: 0,
        drawList: None,
        firstDstBlendBarrier: None,
        unresolvedBarriers: gpu::BarrierFlags::none,
    }
}

fn maximally_negative_i32() -> IAABB {
    IAABB {
        left: i32::MAX,
        top: i32::MAX,
        right: i32::MIN,
        bottom: i32::MIN,
    }
}

fn select_interlock_mode(
    frame_descriptor: &FrameDescriptor,
    platform_features: &gpu::PlatformFeatures,
) -> gpu::InterlockMode {
    if frame_descriptor.msaaSampleCount != 0 {
        return gpu::InterlockMode::msaa;
    }
    if frame_descriptor.clockwiseFillOverride {
        if platform_features.supportsClockwiseMode && !frame_descriptor.disableRasterOrdering {
            return gpu::InterlockMode::clockwise;
        }
        if platform_features.supportsClockwiseAtomicMode {
            return gpu::InterlockMode::clockwiseAtomic;
        }
    }
    if platform_features.supportsRasterOrderingMode
        && (!frame_descriptor.disableRasterOrdering || !platform_features.supportsAtomicMode)
    {
        return gpu::InterlockMode::rasterOrdering;
    }
    if platform_features.supportsAtomicMode {
        return gpu::InterlockMode::atomics;
    }
    gpu::InterlockMode::msaa
}

impl RenderContext {
    pub fn riveRenderFactory(&self) -> &RiveRenderFactory {
        &self.base
    }
    pub fn riveRenderFactoryMut(&mut self) -> &mut RiveRenderFactory {
        &mut self.base
    }

    pub fn makeLinearGradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> rcp<RenderShader> {
        self.base.makeLinearGradient(sx, sy, ex, ey, colors, stops)
    }

    pub fn makeRadialGradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> rcp<RenderShader> {
        self.base.makeRadialGradient(cx, cy, radius, colors, stops)
    }

    pub fn makeRenderPath(
        &mut self,
        raw_path: &mut RawPath,
        fill_rule: FillRule,
    ) -> rcp<RenderPath> {
        self.base.makeRenderPath(raw_path, fill_rule)
    }

    pub fn makeEmptyRenderPath(&mut self) -> rcp<RenderPath> {
        self.base.makeEmptyRenderPath()
    }

    pub fn makeRenderPaint(&mut self) -> rcp<RenderPaint> {
        self.base.makeRenderPaint()
    }

    pub fn from_impl<T>(implementation: Box<T>) -> Pin<Box<Self>>
    where
        T: RenderContextImplContract + 'static,
    {
        let owner = RenderContextImplOwner::from_box(implementation);
        let features = owner.contract().renderContextImpl().platformFeatures();
        debug_assert!(
            !features.supportsBlendAdvancedCoherentKHR || features.supportsBlendAdvancedKHR
        );
        #[cfg(feature = "rive-generate-feather-lut")]
        {
            let mut table = [0.0f32; gpu::GAUSSIAN_TABLE_SIZE as usize];
            unsafe {
                gpu::generate_gausian_integral_table(table.as_mut_ptr());
                gpu::generate_inverse_gausian_integral_table(table.as_mut_ptr());
            }
        }
        let max_path_id = gpu::MaxPathID(features.pathIDGranularity as i32) as usize - 1;
        let mut context = Box::pin(Self {
            base: ManuallyDrop::new(RiveRenderFactory::default()),
            members: ManuallyDrop::new(RenderContextMembers {
                m_impl: owner,
                m_max_path_id: max_path_id,
                #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
                m_ore_context: None,
                m_current_resource_allocations: ResourceAllocationCounts::default(),
                m_max_recent_resource_requirements: ResourceAllocationCounts::default(),
                m_last_resource_trim_time_in_seconds: 0.0,
                m_frame_descriptor: FrameDescriptor::default(),
                m_frame_interlock_mode: gpu::InterlockMode::msaa,
                m_frame_shader_features_mask: gpu::ShaderFeatures::NONE,
                #[cfg(debug_assertions)]
                m_did_begin_frame: false,
                m_clip_content_id: 0,
                m_coverage_buffer_prefix: 0,
                m_indirect_draw_list: Vec::new(),
                m_intersection_board: None,
                m_scissor_id_lookup: HashMap::new(),
                m_prev_scissor_id: -1,
                m_flush_uniform_data: gpu::WriteOnlyMappedMemory::default(),
                m_path_data: gpu::WriteOnlyMappedMemory::default(),
                m_paint_data: gpu::WriteOnlyMappedMemory::default(),
                m_paint_aux_data: gpu::WriteOnlyMappedMemory::default(),
                m_contour_data: gpu::WriteOnlyMappedMemory::default(),
                m_grad_span_data: gpu::WriteOnlyMappedMemory::default(),
                m_tess_span_data: gpu::WriteOnlyMappedMemory::default(),
                m_triangle_vertex_data: gpu::WriteOnlyMappedMemory::default(),
                m_image_draw_instance_data: gpu::WriteOnlyMappedMemory::default(),
                m_per_frame_allocator: TrivialBlockAllocator::default(),
                m_num_chops_allocator: TrivialArrayAllocator::default(),
                m_chop_vertices_allocator: TrivialArrayAllocator::default(),
                m_tangent_pairs_allocator: TrivialArrayAllocator::default(),
                m_polar_segment_counts_allocator: TrivialArrayAllocator::default(),
                m_parametric_segment_counts_allocator: TrivialArrayAllocator::default(),
                m_logical_flushes: Vec::new(),
            }),
            #[cfg(feature = "rive-ktx2")]
            m_ktx2_decoder: None,
            #[cfg(feature = "rive-decoders")]
            m_bitmap_decoder: None,
            _pin: PhantomPinned,
        });
        unsafe {
            let this = Pin::get_unchecked_mut(context.as_mut());
            this.setResourceSizes(ResourceAllocationCounts::default(), true);
            this.releaseResources();
        }
        context
    }

    pub fn makeRenderBuffer(
        &mut self,
        buffer_type: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType,
        flags: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags,
        size_in_bytes: usize,
    ) -> crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<
        crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer,
    > {
        self.m_impl
            .contract_mut()
            .makeRenderBuffer(buffer_type, flags, size_in_bytes)
    }

    pub fn makeRenderBufferHandle(
        &mut self,
        buffer_type: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType,
        flags: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Option<RiveRenderBufferHandle> {
        let source = self.makeRenderBuffer(buffer_type, flags, size_in_bytes);
        // SAFETY: RenderContext returns a fresh backend intrusive allocation;
        // this call transfers its sole product mutation authority to the handle.
        unsafe { RiveRenderBufferHandle::from_source(source) }
    }

    #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
    pub fn makeRenderCanvasExecutable(
        &mut self,
        width: u32,
        height: u32,
    ) -> crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas>{
        self.m_impl.contract_mut().makeRenderCanvas(width, height)
    }

    #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
    pub fn oreExecutable(&mut self) -> *mut OreContext {
        if self.m_ore_context.is_none() {
            self.m_ore_context = self.m_impl.contract_mut().makeOreContext();
        }
        self.m_ore_context
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |value| value)
    }

    pub unsafe fn decodeImageExecutable(
        &mut self,
        encoded_bytes: Span<u8>,
    ) -> crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<
        crate::mechanical_port::source::include::rive::renderer_hpp::RenderImage,
    > {
        let bytes = if encoded_bytes.size == 0 {
            // A default/empty source Span may carry a null data pointer. Rust
            // slices require a non-null pointer even at length zero.
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(encoded_bytes.data, encoded_bytes.size) }
        };
        let mut texture = self.m_impl.contract_mut().platformDecodeImageTexture(bytes);

        // The KTX2 and software-decoder source branches remain independently
        // dispatched. Backends that enable them install the corresponding
        // decoder before falling through to the common image owner below.
        #[cfg(feature = "rive-ktx2")]
        if texture.get().is_null() && bytes.len() >= 12 && bytes[0..4] == [0xab, 0x4b, 0x54, 0x58] {
            let features = *self.platformFeatures();
            let support = Ktx2HwSupport {
                supports_bc: features.supportsTextureCompressionBC,
                supports_astc: features.supportsTextureCompressionASTC,
                supports_etc2: features.supportsTextureCompressionETC2,
            };
            if let Some(decoded) = self
                .m_ktx2_decoder
                .as_mut()
                .and_then(|decoder| decoder.decodeKtx2(bytes, support))
            {
                texture = self.m_impl.contract_mut().makeImageTexture(
                    decoded.pixel_width,
                    decoded.pixel_height,
                    decoded.level_count,
                    decoded.format,
                    &decoded.blocks,
                    decoded.block_width,
                    decoded.block_height,
                    decoded.srgb,
                    false,
                );
            }
        }

        #[cfg(feature = "rive-decoders")]
        if texture.get().is_null() {
            if let Some(mut bitmap) = self
                .m_bitmap_decoder
                .as_mut()
                .and_then(|decoder| decoder.decodeBitmap(bytes))
            {
                if bitmap.pixel_format != BitmapPixelFormat::rgbaPremul {
                    self.m_bitmap_decoder
                        .as_mut()
                        .unwrap()
                        .convertToRGBAPremul(&mut bitmap);
                    bitmap.pixel_format = BitmapPixelFormat::rgbaPremul;
                }
                let dimensions = bitmap.width | bitmap.height;
                let mip_count = if dimensions == 0 {
                    0
                } else {
                    u32::BITS - dimensions.leading_zeros()
                };
                texture = self.m_impl.contract_mut().makeImageTexture(
                    bitmap.width,
                    bitmap.height,
                    mip_count,
                    crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat::rgba32,
                    &bitmap.bytes,
                    1,
                    1,
                    false,
                    true,
                );
            }
        }

        if texture.get().is_null() {
            return crate::mechanical_port::source::include::rive::refcnt_hpp::rcp::new();
        }
        let mut derived =
            crate::mechanical_port::source::include::rive::refcnt_hpp::make_rcp(|| unsafe {
                RiveRenderImage::new(texture)
            });
        unsafe {
            crate::mechanical_port::source::include::rive::refcnt_hpp::rcp::converting_move_ctor(
                &mut derived,
            )
        }
    }

    pub fn decodeImageHandle(&mut self, encoded_bytes: &[u8]) -> Option<RiveRenderImageHandle> {
        let source = unsafe {
            self.decodeImageExecutable(Span {
                data: encoded_bytes.as_ptr(),
                size: encoded_bytes.len(),
            })
        };
        // SAFETY: decodeImageExecutable either returns null or the fresh exact
        // RiveRenderImage allocation built immediately above.
        unsafe { RiveRenderImageHandle::from_source(source) }
    }

    #[cfg(feature = "rive-ktx2")]
    pub fn installKtx2Decoder(&mut self, decoder: Box<dyn Ktx2DecoderContract>) {
        self.m_ktx2_decoder = Some(decoder);
    }

    #[cfg(feature = "rive-decoders")]
    pub fn installBitmapDecoder(&mut self, decoder: Box<dyn BitmapDecoderContract>) {
        self.m_bitmap_decoder = Some(decoder);
    }

    pub fn releaseResources(&mut self) {
        #[cfg(debug_assertions)]
        debug_assert!(!self.m_did_begin_frame);
        self.resetContainers();
        self.setResourceSizes(ResourceAllocationCounts::default(), false);
        self.m_max_recent_resource_requirements = ResourceAllocationCounts::default();
        self.m_last_resource_trim_time_in_seconds = self.m_impl.contract().secondsNow();
    }

    pub fn resetContainers(&mut self) {
        #[cfg(debug_assertions)]
        debug_assert!(!self.m_did_begin_frame);
        if !self.m_logical_flushes.is_empty() {
            debug_assert_eq!(self.m_logical_flushes.len(), 1);
            self.m_logical_flushes.truncate(1);
            self.m_logical_flushes[0].resetContainers();
        }
        self.m_indirect_draw_list.clear();
        self.m_indirect_draw_list.shrink_to_fit();
        self.m_intersection_board = None;
    }

    pub fn beginFrameExecutable(&mut self, frame_descriptor: &FrameDescriptor) {
        let this = self as *mut RenderContext;
        unsafe { self.m_impl.contract_mut().preBeginFrame(this) };
        #[cfg(debug_assertions)]
        debug_assert!(!self.m_did_begin_frame);
        debug_assert!(frame_descriptor.renderTargetWidth > 0);
        debug_assert!(frame_descriptor.renderTargetHeight > 0);
        self.m_frame_descriptor = *frame_descriptor;
        self.m_frame_interlock_mode =
            select_interlock_mode(&self.m_frame_descriptor, self.platformFeatures());
        if self.m_frame_interlock_mode == gpu::InterlockMode::msaa
            && self.m_frame_descriptor.msaaSampleCount == 0
        {
            self.m_frame_descriptor.msaaSampleCount = 4;
        }
        self.m_frame_shader_features_mask = gpu::ShaderFeaturesMaskFor(self.m_frame_interlock_mode);
        if self.m_logical_flushes.is_empty() {
            self.m_logical_flushes
                .push(unsafe { LogicalFlush::new_box(this) });
        }
        #[cfg(debug_assertions)]
        {
            self.m_did_begin_frame = true;
        }
    }

    /// Source-shaped abandonment inverse for a begun frame which never
    /// reached flush. This releases pending logical draw owners and rewinds
    /// every per-frame arena without creating or committing a command buffer.
    pub fn abortFrameExecutable(&mut self) {
        if !self.m_logical_flushes.is_empty() {
            self.m_logical_flushes.truncate(1);
            self.m_logical_flushes[0].rewindExecutable();
        }
        self.m_per_frame_allocator.reset();
        self.m_num_chops_allocator.reset();
        self.m_chop_vertices_allocator.reset();
        self.m_tangent_pairs_allocator.reset();
        self.m_polar_segment_counts_allocator.reset();
        self.m_parametric_segment_counts_allocator.reset();
        self.m_clip_content_id = 0;
        self.m_prev_scissor_id = -1;
        self.m_scissor_id_lookup.clear();
        self.m_coverage_buffer_prefix = 0;
        self.m_indirect_draw_list.clear();
        self.m_intersection_board = None;
        self.m_frame_descriptor = FrameDescriptor::default();
        #[cfg(debug_assertions)]
        {
            self.m_did_begin_frame = false;
        }
    }

    pub fn isOutsideCurrentFrameExecutable(&self, bounds: &IAABB) -> bool {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_did_begin_frame);
        bounds.left >= self.m_frame_descriptor.renderTargetWidth as i32
            || bounds.top >= self.m_frame_descriptor.renderTargetHeight as i32
            || bounds.right <= 0
            || bounds.bottom <= 0
            || bounds.left >= bounds.right
            || bounds.top >= bounds.bottom
    }

    pub fn frameSupportsClipRectsExecutable(&self) -> bool {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_did_begin_frame);
        self.m_frame_interlock_mode != gpu::InterlockMode::msaa
            || self.platformFeatures().supportsClipPlanes
    }

    pub fn frameSupportsImagePaintForPathsExecutable(&self) -> bool {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_did_begin_frame);
        self.m_frame_interlock_mode != gpu::InterlockMode::atomics
    }

    pub fn generateClipIDExecutable(
        &mut self,
        content_bounds: IAABB,
        parent_clip_id: u32,
        tightened_bounds: AABBu16,
    ) -> u32 {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_did_begin_frame);
        debug_assert!(!self.m_logical_flushes.is_empty());
        self.m_logical_flushes
            .last_mut()
            .unwrap()
            .generateClipIDExecutable(content_bounds, parent_clip_id, tightened_bounds)
    }

    pub unsafe fn pushDrawsExecutable(
        &mut self,
        draws: &mut [DrawUniquePtr],
        draw_count: usize,
    ) -> bool {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_did_begin_frame);
        debug_assert!(!self.m_logical_flushes.is_empty());
        debug_assert!(draw_count <= draws.len());
        unsafe {
            self.m_logical_flushes
                .last_mut()
                .unwrap()
                .pushDrawsExecutable(&mut draws[..draw_count])
        }
    }

    pub fn logicalFlushExecutable(&mut self) {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_did_begin_frame);
        self.m_clip_content_id = 0;
        let flush = unsafe { LogicalFlush::new_box(self) };
        self.m_logical_flushes.push(flush);
    }

    pub unsafe fn flushExecutable(&mut self, resources: &FlushResources) {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_did_begin_frame);
        debug_assert!(!resources.renderTarget.is_null());
        debug_assert_eq!(
            unsafe { (&*resources.renderTarget).width() },
            self.m_frame_descriptor.renderTargetWidth
        );
        debug_assert_eq!(
            unsafe { (&*resources.renderTarget).height() },
            self.m_frame_descriptor.renderTargetHeight
        );
        self.m_clip_content_id = 0;

        let mut total = ResourceCounters::default();
        let mut layout = LayoutCounters::default();
        for index in 0..self.m_logical_flushes.len() {
            unsafe {
                self.m_logical_flushes[index].layoutResourcesExecutable(
                    resources,
                    index,
                    &mut total,
                    &mut layout,
                );
            }
        }
        let transient = layout.maxPLSTransientBackingPlaneCount > 0;
        let atomic = self.frameInterlockMode() == gpu::InterlockMode::atomics;
        let requirements = ResourceAllocationCounts {
            flushUniformBufferCount: self.m_logical_flushes.len(),
            pathBufferCount: total.pathCount + layout.pathPaddingCount as usize,
            paintBufferCount: total.pathCount + layout.paintPaddingCount as usize,
            paintAuxBufferCount: total.pathCount + layout.paintAuxPaddingCount as usize,
            contourBufferCount: total.contourCount + layout.contourPaddingCount as usize,
            gradSpanBufferCount: layout.gradSpanCount as usize
                + layout.gradSpanPaddingCount as usize,
            tessSpanBufferCount: total.maxTessellatedSegmentCount,
            triangleVertexBufferCount: total.maxTriangleVertexCount,
            imageDrawInstanceBufferCount: total.imageDrawCount,
            gradTextureHeight: layout.maxGradTextureHeight as usize,
            tessTextureHeight: layout.maxTessTextureHeight as usize,
            featherAtlasTextureWidth: layout.maxFeatherAtlasWidth as usize,
            featherAtlasTextureHeight: layout.maxFeatherAtlasHeight as usize,
            plsTransientBackingWidth: if transient {
                self.m_frame_descriptor.renderTargetWidth as usize
            } else {
                0
            },
            plsTransientBackingHeight: if transient {
                self.m_frame_descriptor.renderTargetHeight as usize
            } else {
                0
            },
            plsTransientBackingPlaneCount: layout.maxPLSTransientBackingPlaneCount as usize,
            plsAtomicCoverageBackingWidth: if atomic {
                self.m_frame_descriptor.renderTargetWidth as usize
            } else {
                0
            },
            plsAtomicCoverageBackingHeight: if atomic {
                self.m_frame_descriptor.renderTargetHeight as usize
            } else {
                0
            },
            coverageBufferLength: layout.maxCoverageBufferLength,
        };
        debug_assert!(requirements.gradTextureHeight <= K_MAX_TEXTURE_HEIGHT);
        debug_assert!(requirements.tessTextureHeight <= K_MAX_TEXTURE_HEIGHT);
        debug_assert!(
            requirements.coverageBufferLength <= self.platformFeatures().maxCoverageBufferLength
        );

        let req = requirements.toVec();
        let recent = self.m_max_recent_resource_requirements.toVec();
        let mut recent_max = [0usize; ResourceAllocationCounts::NUM_ELEMENTS];
        for i in 0..recent_max.len() {
            recent_max[i] = req[i].max(recent[i]);
        }
        self.m_max_recent_resource_requirements = ResourceAllocationCounts::FromVec(&recent_max);

        let current = self.m_current_resource_allocations.toVec();
        let overalloc = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 5];
        let mut allocated = [0usize; ResourceAllocationCounts::NUM_ELEMENTS];
        for i in 0..allocated.len() {
            allocated[i] = if req[i] <= current[i] {
                current[i]
            } else {
                req[i].saturating_mul(overalloc[i]) >> 2
            };
        }
        let mut allocs = ResourceAllocationCounts::FromVec(&allocated);
        allocs.gradTextureHeight = allocs.gradTextureHeight.min(K_MAX_TEXTURE_HEIGHT);
        allocs.tessTextureHeight = allocs.tessTextureHeight.min(K_MAX_TEXTURE_HEIGHT);
        allocs.featherAtlasTextureWidth = allocs.featherAtlasTextureWidth.min(
            self.featherAtlasMaxSize()
                .max(self.frameDescriptor().renderTargetWidth) as usize,
        );
        allocs.featherAtlasTextureHeight = allocs.featherAtlasTextureHeight.min(
            self.featherAtlasMaxSize()
                .max(self.frameDescriptor().renderTargetHeight) as usize,
        );
        allocs.coverageBufferLength = allocs
            .coverageBufferLength
            .min(self.platformFeatures().maxCoverageBufferLength);

        let flush_time = self.m_impl.contract().secondsNow();
        let needs_trim = flush_time - self.m_last_resource_trim_time_in_seconds >= 5.0;
        if needs_trim {
            let recent = self.m_max_recent_resource_requirements.toVec();
            let values = allocs.toVec();
            let shrink = [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 2];
            let mut trimmed = [0usize; ResourceAllocationCounts::NUM_ELEMENTS];
            for i in 0..trimmed.len() {
                trimmed[i] = if recent[i] <= values[i].saturating_mul(shrink[i]) / 3 {
                    recent[i].saturating_mul(overalloc[i]) >> 2
                } else {
                    values[i]
                };
            }
            allocs = ResourceAllocationCounts::FromVec(&trimmed);
            self.m_max_recent_resource_requirements = ResourceAllocationCounts::default();
            self.m_last_resource_trim_time_in_seconds = flush_time;
        }
        debug_assert!(allocs.toVec().iter().zip(req.iter()).all(|(a, r)| a >= r));
        self.setResourceSizes(allocs, false);
        self.m_impl
            .contract_mut()
            .prepareToFlush(resources.currentFrameNumber, resources.safeFrameNumber);
        if unsafe { self.mapResourceBuffersExecutable(&requirements) } {
            for flush in self.m_logical_flushes.iter_mut() {
                unsafe { flush.writeResourcesExecutable() };
            }
            debug_assert_eq!(
                self.m_flush_uniform_data.elementsWritten(),
                self.m_logical_flushes.len()
            );
            debug_assert_eq!(
                self.m_image_draw_instance_data.elementsWritten(),
                total.imageDrawCount
            );
            debug_assert_eq!(
                self.m_path_data.elementsWritten(),
                total.pathCount + layout.pathPaddingCount as usize
            );
            debug_assert_eq!(
                self.m_paint_data.elementsWritten(),
                total.pathCount + layout.paintPaddingCount as usize
            );
            debug_assert_eq!(
                self.m_paint_aux_data.elementsWritten(),
                total.pathCount + layout.paintAuxPaddingCount as usize
            );
            debug_assert_eq!(
                self.m_contour_data.elementsWritten(),
                total.contourCount + layout.contourPaddingCount as usize
            );
            debug_assert_eq!(
                self.m_grad_span_data.elementsWritten(),
                layout.gradSpanCount as usize + layout.gradSpanPaddingCount as usize
            );
            debug_assert!(
                self.m_tess_span_data.elementsWritten() <= total.maxTessellatedSegmentCount
            );
            debug_assert!(
                self.m_triangle_vertex_data.elementsWritten() <= total.maxTriangleVertexCount
            );
            unsafe { self.unmapResourceBuffersExecutable(&requirements) };
            let members = &mut *self.members;
            for flush in members.m_logical_flushes.iter() {
                unsafe { members.m_impl.contract_mut().flush(flush.desc()) };
            }
        } else {
            eprintln!("Buffer mapping failed, cannot render.");
            unsafe { self.unmapResourceBuffersExecutable(&requirements) };
        }
        unsafe { self.m_impl.contract_mut().postFlush(resources) };
        if !self.m_logical_flushes.is_empty() {
            self.m_logical_flushes.truncate(1);
            self.m_logical_flushes[0].rewindExecutable();
        }
        self.m_per_frame_allocator.reset();
        self.m_num_chops_allocator.reset();
        self.m_chop_vertices_allocator.reset();
        self.m_tangent_pairs_allocator.reset();
        self.m_polar_segment_counts_allocator.reset();
        self.m_parametric_segment_counts_allocator.reset();
        self.m_frame_descriptor = FrameDescriptor::default();
        #[cfg(debug_assertions)]
        {
            self.m_did_begin_frame = false;
        }
        if needs_trim {
            self.resetContainers();
        }
    }

    pub fn setResourceSizes(&mut self, allocs: ResourceAllocationCounts, force_realloc: bool) {
        let old = self.m_current_resource_allocations;
        let implementation = self.m_impl.contract_mut();
        if force_realloc || allocs.flushUniformBufferCount != old.flushUniformBufferCount {
            implementation.resizeFlushUniformBuffer(
                allocs.flushUniformBufferCount * core::mem::size_of::<gpu::FlushUniforms>(),
            );
        }
        if force_realloc || allocs.pathBufferCount != old.pathBufferCount {
            implementation.resizePathBuffer(
                allocs.pathBufferCount * core::mem::size_of::<gpu::PathData>(),
                gpu::PathData::kBufferStructure,
            );
        }
        if force_realloc || allocs.paintBufferCount != old.paintBufferCount {
            implementation.resizePaintBuffer(
                allocs.paintBufferCount * core::mem::size_of::<gpu::PaintData>(),
                gpu::PaintData::kBufferStructure,
            );
        }
        if force_realloc || allocs.paintAuxBufferCount != old.paintAuxBufferCount {
            implementation.resizePaintAuxBuffer(
                allocs.paintAuxBufferCount * core::mem::size_of::<gpu::PaintAuxData>(),
                gpu::PaintAuxData::kBufferStructure,
            );
        }
        if force_realloc || allocs.contourBufferCount != old.contourBufferCount {
            implementation.resizeContourBuffer(
                allocs.contourBufferCount * core::mem::size_of::<gpu::ContourData>(),
                gpu::ContourData::kBufferStructure,
            );
        }
        if force_realloc || allocs.gradSpanBufferCount != old.gradSpanBufferCount {
            implementation.resizeGradSpanBuffer(
                allocs.gradSpanBufferCount * core::mem::size_of::<gpu::GradientSpan>(),
            );
        }
        if force_realloc || allocs.tessSpanBufferCount != old.tessSpanBufferCount {
            implementation.resizeTessVertexSpanBuffer(
                allocs.tessSpanBufferCount * core::mem::size_of::<gpu::TessVertexSpan>(),
            );
        }
        if force_realloc || allocs.triangleVertexBufferCount != old.triangleVertexBufferCount {
            implementation.resizeTriangleVertexBuffer(
                allocs.triangleVertexBufferCount * core::mem::size_of::<gpu::TriangleVertex>(),
            );
        }
        if force_realloc || allocs.imageDrawInstanceBufferCount != old.imageDrawInstanceBufferCount
        {
            implementation.resizeImageDrawInstanceBuffer(
                allocs.imageDrawInstanceBufferCount
                    * core::mem::size_of::<gpu::ImageDrawInstance>(),
            );
        }
        debug_assert!(allocs.gradTextureHeight <= K_MAX_TEXTURE_HEIGHT);
        if force_realloc || allocs.gradTextureHeight != old.gradTextureHeight {
            implementation
                .resizeGradientTexture(gpu::kGradTextureWidth, allocs.gradTextureHeight as u32);
        }
        debug_assert!(allocs.tessTextureHeight <= K_MAX_TEXTURE_HEIGHT);
        if force_realloc || allocs.tessTextureHeight != old.tessTextureHeight {
            implementation.resizeTessellationTexture(
                gpu::kTessTextureWidth as u32,
                allocs.tessTextureHeight as u32,
            );
        }
        if force_realloc
            || allocs.featherAtlasTextureWidth != old.featherAtlasTextureWidth
            || allocs.featherAtlasTextureHeight != old.featherAtlasTextureHeight
        {
            implementation.resizeFeatherAtlasTexture(
                allocs.featherAtlasTextureWidth as u32,
                allocs.featherAtlasTextureHeight as u32,
            );
        }
        debug_assert!(
            allocs.plsTransientBackingPlaneCount
                <= RenderContextImpl::PLS_TRANSIENT_BACKING_MAX_PLANE_COUNT as usize
        );
        if force_realloc
            || allocs.plsTransientBackingWidth != old.plsTransientBackingWidth
            || allocs.plsTransientBackingHeight != old.plsTransientBackingHeight
            || allocs.plsTransientBackingPlaneCount != old.plsTransientBackingPlaneCount
        {
            implementation.resizeTransientPLSBacking(
                allocs.plsTransientBackingWidth as u32,
                allocs.plsTransientBackingHeight as u32,
                allocs.plsTransientBackingPlaneCount as u32,
            );
        }
        debug_assert!(allocs.plsAtomicCoverageBackingWidth <= allocs.plsTransientBackingWidth);
        debug_assert!(allocs.plsAtomicCoverageBackingHeight <= allocs.plsTransientBackingHeight);
        if force_realloc
            || allocs.plsAtomicCoverageBackingWidth != old.plsAtomicCoverageBackingWidth
            || allocs.plsAtomicCoverageBackingHeight != old.plsAtomicCoverageBackingHeight
        {
            implementation.resizeAtomicCoverageBacking(
                allocs.plsAtomicCoverageBackingWidth as u32,
                allocs.plsAtomicCoverageBackingHeight as u32,
            );
        }
        debug_assert!(
            allocs.coverageBufferLength
                <= implementation
                    .renderContextImpl()
                    .platformFeatures()
                    .maxCoverageBufferLength
        );
        if force_realloc || allocs.coverageBufferLength != old.coverageBufferLength {
            implementation
                .resizeCoverageBuffer(allocs.coverageBufferLength * core::mem::size_of::<u32>());
            self.m_coverage_buffer_prefix = 0;
        }
        self.m_current_resource_allocations = allocs;
    }

    pub fn incrementCoverageBufferPrefixExecutable(&mut self, needs_clear: &mut bool) -> u32 {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_did_begin_frame);
        debug_assert_eq!(
            self.frameInterlockMode(),
            gpu::InterlockMode::clockwiseAtomic
        );
        loop {
            if self.m_coverage_buffer_prefix == 0 {
                *needs_clear = true;
            }
            self.m_coverage_buffer_prefix = self.m_coverage_buffer_prefix.wrapping_add(1 << 20);
            if self.m_coverage_buffer_prefix != 0 {
                return self.m_coverage_buffer_prefix;
            }
        }
    }

    pub unsafe fn mapResourceBuffersExecutable(
        &mut self,
        counts: &ResourceAllocationCounts,
    ) -> bool {
        macro_rules! map {
            ($field:ident, $count:ident, $method:ident) => {{
                if counts.$count > 0 {
                    let members = &mut *self.members;
                    let implementation = members.m_impl.contract_mut();
                    if !unsafe {
                        members.$field
                            .mapElementsWith(counts.$count, |bytes| implementation.$method(bytes))
                    } {
                        return false;
                    }
                }
                debug_assert!(self.members.$field.hasRoomFor(counts.$count));
            }};
        }
        map!(
            m_flush_uniform_data,
            flushUniformBufferCount,
            mapFlushUniformBuffer
        );
        map!(m_path_data, pathBufferCount, mapPathBuffer);
        map!(m_paint_data, paintBufferCount, mapPaintBuffer);
        map!(m_paint_aux_data, paintAuxBufferCount, mapPaintAuxBuffer);
        map!(m_contour_data, contourBufferCount, mapContourBuffer);
        map!(m_grad_span_data, gradSpanBufferCount, mapGradSpanBuffer);
        map!(
            m_tess_span_data,
            tessSpanBufferCount,
            mapTessVertexSpanBuffer
        );
        map!(
            m_triangle_vertex_data,
            triangleVertexBufferCount,
            mapTriangleVertexBuffer
        );
        map!(
            m_image_draw_instance_data,
            imageDrawInstanceBufferCount,
            mapImageDrawInstanceBuffer
        );
        true
    }

    pub unsafe fn unmapResourceBuffersExecutable(&mut self, counts: &ResourceAllocationCounts) {
        macro_rules! unmap {
            ($field:ident, $count:ident, $method:ident) => {{
                if self.members.$field.is_mapped() {
                    let members = &mut *self.members;
                    let implementation = members.m_impl.contract_mut();
                    unsafe {
                        members.$field
                            .unmapElementsWith(counts.$count, |bytes| implementation.$method(bytes))
                    };
                }
            }};
        }
        unmap!(
            m_flush_uniform_data,
            flushUniformBufferCount,
            unmapFlushUniformBuffer
        );
        unmap!(m_path_data, pathBufferCount, unmapPathBuffer);
        unmap!(m_paint_data, paintBufferCount, unmapPaintBuffer);
        unmap!(m_paint_aux_data, paintAuxBufferCount, unmapPaintAuxBuffer);
        unmap!(m_contour_data, contourBufferCount, unmapContourBuffer);
        unmap!(m_grad_span_data, gradSpanBufferCount, unmapGradSpanBuffer);
        unmap!(
            m_tess_span_data,
            tessSpanBufferCount,
            unmapTessVertexSpanBuffer
        );
        unmap!(
            m_triangle_vertex_data,
            triangleVertexBufferCount,
            unmapTriangleVertexBuffer
        );
        unmap!(
            m_image_draw_instance_data,
            imageDrawInstanceBufferCount,
            unmapImageDrawInstanceBuffer
        );
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        macro_rules! trace_drop {
            ($stage:literal) => {
                #[cfg(test)]
                RENDER_CONTEXT_DROP_TRACE.with(|trace| trace.borrow_mut().push($stage));
            };
        }
        #[cfg(debug_assertions)]
        debug_assert!(!self.m_did_begin_frame);
        #[cfg(feature = "rive-ktx2")]
        {
            self.m_ktx2_decoder = None;
        }
        #[cfg(feature = "rive-decoders")]
        {
            self.m_bitmap_decoder = None;
        }
        self.m_logical_flushes.clear();
        #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
        {
            self.m_ore_context = None;
        }
        unsafe {
            // C++ reverse member destruction, without changing the source
            // declaration order in `RenderContextMembers`.
            trace_drop!("logicalFlushes");
            core::ptr::drop_in_place(&mut self.m_logical_flushes);
            trace_drop!("parametricAllocator");
            core::ptr::drop_in_place(&mut self.m_parametric_segment_counts_allocator);
            trace_drop!("polarAllocator");
            core::ptr::drop_in_place(&mut self.m_polar_segment_counts_allocator);
            trace_drop!("tangentAllocator");
            core::ptr::drop_in_place(&mut self.m_tangent_pairs_allocator);
            trace_drop!("chopAllocator");
            core::ptr::drop_in_place(&mut self.m_chop_vertices_allocator);
            trace_drop!("numChopsAllocator");
            core::ptr::drop_in_place(&mut self.m_num_chops_allocator);
            trace_drop!("perFrameAllocator");
            core::ptr::drop_in_place(&mut self.m_per_frame_allocator);
            trace_drop!("imageDrawData");
            core::ptr::drop_in_place(&mut self.m_image_draw_instance_data);
            trace_drop!("triangleData");
            core::ptr::drop_in_place(&mut self.m_triangle_vertex_data);
            trace_drop!("tessData");
            core::ptr::drop_in_place(&mut self.m_tess_span_data);
            trace_drop!("gradientData");
            core::ptr::drop_in_place(&mut self.m_grad_span_data);
            trace_drop!("contourData");
            core::ptr::drop_in_place(&mut self.m_contour_data);
            trace_drop!("paintAuxData");
            core::ptr::drop_in_place(&mut self.m_paint_aux_data);
            trace_drop!("paintData");
            core::ptr::drop_in_place(&mut self.m_paint_data);
            trace_drop!("pathData");
            core::ptr::drop_in_place(&mut self.m_path_data);
            trace_drop!("flushUniformData");
            core::ptr::drop_in_place(&mut self.m_flush_uniform_data);
            trace_drop!("scissorLookup");
            core::ptr::drop_in_place(&mut self.m_scissor_id_lookup);
            trace_drop!("intersectionBoard");
            core::ptr::drop_in_place(&mut self.m_intersection_board);
            trace_drop!("indirectDrawList");
            core::ptr::drop_in_place(&mut self.m_indirect_draw_list);
            #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
            trace_drop!("oreContext");
            #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
            core::ptr::drop_in_place(&mut self.m_ore_context);
            trace_drop!("implementation");
            core::ptr::drop_in_place(&mut self.m_impl);
            trace_drop!("base");
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

#[cfg(test)]
thread_local! {
    static RENDER_CONTEXT_DROP_TRACE: std::cell::RefCell<Vec<&'static str>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

#[cfg(test)]
pub(crate) fn takeRenderContextDropTrace() -> Vec<&'static str> {
    RENDER_CONTEXT_DROP_TRACE.with(|trace| core::mem::take(&mut *trace.borrow_mut()))
}

impl LogicalFlush {
    unsafe fn new_box(parent: *mut RenderContext) -> Box<Self> {
        let mut flush = Box::new(Self {
            m_ctx: NonNull::new(parent).expect("RenderContext::LogicalFlush parent"),
            m_resource_counts: ResourceCounters::default(),
            m_draw_pass_count: 0,
            m_simple_gradients: HashMap::new(),
            m_pending_simple_grad_draws: Vec::new(),
            m_complex_gradients: HashMap::new(),
            m_pending_complex_grad_draws: Vec::new(),
            m_pending_grad_span_count: 0,
            m_clips: Vec::new(),
            m_draws: Vec::new(),
            m_combined_draw_bounds: maximally_negative_i32(),
            m_combined_draw_contents: gpu::DrawContents::none,
            m_path_padding_count: 0,
            m_paint_padding_count: 0,
            m_paint_aux_padding_count: 0,
            m_contour_padding_count: 0,
            m_grad_span_padding_count: 0,
            m_midpoint_fan_tess_end_location: 0,
            m_outer_cubic_tess_end_location: 0,
            m_outer_cubic_tess_vertex_idx: 0,
            m_midpoint_fan_tess_vertex_idx: 0,
            m_grad_texture_layout: gpu::GradTextureLayout::default(),
            m_baseline_shader_misc_flags: gpu::ShaderMiscFlags::none,
            m_flush_desc: empty_flush_descriptor(),
            m_draw_list: BlockAllocatedLinkedList::default(),
            m_first_dst_blend_barrier: core::ptr::null(),
            m_dst_blend_barrier_list_tail: core::ptr::null_mut(),
            m_combined_shader_features: gpu::ShaderFeatures::NONE,
            m_current_path_id: 0,
            m_current_contour_id: 0,
            m_feather_atlas_rectanizer: None,
            m_feather_atlas_max_x: 0,
            m_feather_atlas_max_y: 0,
            m_pending_feather_atlas_draws: Vec::new(),
            m_coverage_buffer_length: 0,
            m_pending_barriers: gpu::BarrierFlags::none,
            m_current_z_index: 0,
            #[cfg(debug_assertions)]
            m_has_done_layout: false,
        });
        flush.rewindExecutable();
        flush
    }

    pub fn rewindExecutable(&mut self) {
        self.m_resource_counts = ResourceCounters::default();
        self.m_draw_pass_count = 0;
        self.m_simple_gradients.clear();
        self.m_pending_simple_grad_draws.clear();
        self.m_complex_gradients.clear();
        self.m_pending_complex_grad_draws.clear();
        self.m_pending_grad_span_count = 0;
        self.m_clips.clear();
        self.m_draws.clear();
        self.m_combined_draw_bounds = maximally_negative_i32();
        self.m_combined_draw_contents = gpu::DrawContents::none;
        self.m_path_padding_count = 0;
        self.m_paint_padding_count = 0;
        self.m_paint_aux_padding_count = 0;
        self.m_contour_padding_count = 0;
        self.m_grad_span_padding_count = 0;
        self.m_midpoint_fan_tess_end_location = 0;
        self.m_outer_cubic_tess_end_location = 0;
        self.m_outer_cubic_tess_vertex_idx = 0;
        self.m_midpoint_fan_tess_vertex_idx = 0;
        self.m_baseline_shader_misc_flags = gpu::ShaderMiscFlags::none;
        self.m_flush_desc = empty_flush_descriptor();
        self.m_draw_list.clear();
        self.m_first_dst_blend_barrier = core::ptr::null();
        self.m_dst_blend_barrier_list_tail = &mut self.m_first_dst_blend_barrier;
        self.m_combined_shader_features = gpu::ShaderFeatures::NONE;
        self.m_current_path_id = 0;
        self.m_current_contour_id = 0;
        if let Some(rectanizer) = self.m_feather_atlas_rectanizer.as_mut() {
            rectanizer.reset();
        }
        self.m_feather_atlas_max_x = 0;
        self.m_feather_atlas_max_y = 0;
        self.m_pending_feather_atlas_draws.clear();
        self.m_coverage_buffer_length = 0;
        self.m_pending_barriers = gpu::BarrierFlags::none;
        self.m_current_z_index = 0;
        #[cfg(debug_assertions)]
        {
            self.m_has_done_layout = false;
        }
    }

    pub fn resetContainers(&mut self) {
        self.m_clips.clear();
        self.m_clips.shrink_to_fit();
        self.m_draws.clear();
        self.m_draws.shrink_to_fit();
        self.m_draws.reserve(K_DEFAULT_DRAW_CAPACITY);
        self.m_simple_gradients = HashMap::with_capacity(K_DEFAULT_SIMPLE_GRADIENT_CAPACITY);
        self.m_pending_simple_grad_draws.clear();
        self.m_pending_simple_grad_draws.shrink_to_fit();
        self.m_pending_simple_grad_draws
            .reserve(K_DEFAULT_SIMPLE_GRADIENT_CAPACITY);
        self.m_complex_gradients = HashMap::with_capacity(K_DEFAULT_COMPLEX_GRADIENT_CAPACITY);
        self.m_pending_complex_grad_draws.clear();
        self.m_pending_complex_grad_draws.shrink_to_fit();
        self.m_pending_complex_grad_draws
            .reserve(K_DEFAULT_COMPLEX_GRADIENT_CAPACITY);
        self.m_pending_feather_atlas_draws.clear();
        self.m_pending_feather_atlas_draws.shrink_to_fit();
    }

    pub fn generateClipIDExecutable(
        &mut self,
        content_bounds: IAABB,
        parent_clip_id: u32,
        tightened_bounds: AABBu16,
    ) -> u32 {
        let context = unsafe { self.m_ctx.as_ref() };
        if self.m_clips.len() < context.m_max_path_id {
            self.m_clips.push(ClipInfo::new(
                content_bounds,
                parent_clip_id,
                tightened_bounds,
            ));
            debug_assert_ne!(context.m_clip_content_id, self.m_clips.len() as u32);
            self.m_clips.len() as u32
        } else {
            0
        }
    }

    pub fn getWritableClipInfoExecutable(&mut self, clip_id: u32) -> &mut ClipInfo {
        debug_assert!(clip_id > 0 && clip_id as usize <= self.m_clips.len());
        &mut self.m_clips[clip_id as usize - 1]
    }

    pub fn allocateCoverageBufferRangeExecutable(&mut self, length: usize) -> usize {
        debug_assert_eq!(
            unsafe { self.m_ctx.as_ref() }.frameInterlockMode(),
            gpu::InterlockMode::clockwiseAtomic
        );
        const BUFFER_IMAGE_TILE_SIZE: usize = 16;
        debug_assert_eq!(
            length % (BUFFER_IMAGE_TILE_SIZE * BUFFER_IMAGE_TILE_SIZE),
            0
        );
        let offset = self.m_coverage_buffer_length as usize;
        if offset.saturating_add(length) > self.platformFeatures().maxCoverageBufferLength {
            return usize::MAX;
        }
        self.m_coverage_buffer_length = self.m_coverage_buffer_length.wrapping_add(length as u32);
        offset
    }

    pub unsafe fn allocateGradientExecutable(
        &mut self,
        gradient: *const Gradient,
        color_ramp_location: *mut gpu::ColorRampLocation,
    ) -> bool {
        #[cfg(debug_assertions)]
        debug_assert!(!self.m_has_done_layout);
        let gradient = unsafe { &*gradient };
        let stops = gradient.stops_slice();
        let stop_count = gradient.count();
        debug_assert!(stop_count > 0);
        if stop_count == 1 || (stop_count == 2 && stops[0] == 0.0 && stops[1] == 1.0) {
            let colors = gradient.colors_slice();
            let color_ramp = gpu::TwoTexelRamp {
                color0: colors[0],
                color1: colors[usize::min(1, stop_count - 1)],
            };
            let simple_key = (color_ramp.color0 as u64) | ((color_ramp.color1 as u64) << 32);
            let ramp_texels_idx = if let Some(existing) = self.m_simple_gradients.get(&simple_key) {
                *existing
            } else {
                if gradient_data_height(
                    self.m_simple_gradients.len() + 1,
                    self.m_complex_gradients.len(),
                ) > K_MAX_TEXTURE_HEIGHT
                {
                    return false;
                }
                let index = (self.m_simple_gradients.len() * 2) as u32;
                self.m_simple_gradients.insert(simple_key, index);
                self.m_pending_simple_grad_draws.push(color_ramp);
                self.m_pending_grad_span_count += 1;
                index
            };
            unsafe {
                (*color_ramp_location).row = (ramp_texels_idx / gpu::kGradTextureWidth) as u16;
                (*color_ramp_location).col = (ramp_texels_idx % gpu::kGradTextureWidth) as u16;
            }
        } else {
            let mut key = unsafe {
                GradientContentKey::new(
                    crate::mechanical_port::source::include::rive::refcnt_hpp::ref_rcp(
                        gradient as *const Gradient as *mut Gradient,
                    ),
                )
            };
            let row = if let Some(existing) = self.m_complex_gradients.get(&key) {
                *existing
            } else {
                if gradient_data_height(
                    self.m_simple_gradients.len(),
                    self.m_complex_gradients.len() + 1,
                ) > K_MAX_TEXTURE_HEIGHT
                {
                    return false;
                }
                let row = self.m_complex_gradients.len() as u16;
                let owned_key = GradientContentKey::move_from(&mut key);
                self.m_complex_gradients.insert(owned_key, row);
                self.m_pending_complex_grad_draws.push(gradient);
                self.m_pending_grad_span_count += stop_count - 1;
                row
            };
            unsafe {
                (*color_ramp_location).row = row;
                (*color_ramp_location).col = gpu::ColorRampLocation::kComplexGradientMarker;
            }
        }
        true
    }

    pub unsafe fn allocateFeatherAtlasDrawExecutable(
        &mut self,
        path_draw: *mut PathDraw,
        draw_width: u16,
        draw_height: u16,
        desired_padding: u16,
        x: *mut u16,
        y: *mut u16,
        padded_region: *mut AABBu16,
    ) -> bool {
        if self.m_feather_atlas_rectanizer.is_none() {
            let atlas_max = unsafe { self.m_ctx.as_ref() }.featherAtlasMaxSize() as u16;
            self.m_feather_atlas_rectanizer = Some(Box::new(RectanizerSkyline::new(
                atlas_max.max(draw_width) as i32,
                atlas_max.max(draw_height) as i32,
            )));
        }
        let rectanizer = self.m_feather_atlas_rectanizer.as_mut().unwrap();
        let atlas_max_width = rectanizer.width() as u16;
        let atlas_max_height = rectanizer.height() as u16;
        // Explicit `std::min<uint16_t>` converts the arithmetic result back to
        // uint16_t before comparison, preserving modulo-2^16 truncation.
        let padded_width = draw_width
            .wrapping_add(desired_padding.wrapping_mul(2))
            .min(atlas_max_width);
        let padded_height = draw_height
            .wrapping_add(desired_padding.wrapping_mul(2))
            .min(atlas_max_height);
        let mut ix = 0i16;
        let mut iy = 0i16;
        if !rectanizer.addRect(padded_width as i32, padded_height as i32, &mut ix, &mut iy) {
            if draw_width > atlas_max_width || draw_height > atlas_max_height {
                self.m_feather_atlas_rectanizer = None;
            }
            // Preserve the source's unconditional second reset.
            self.m_feather_atlas_rectanizer = None;
            return false;
        }
        debug_assert!(ix >= 0 && iy >= 0);
        debug_assert!(ix as u32 + padded_width as u32 <= atlas_max_width as u32);
        debug_assert!(iy as u32 + padded_height as u32 <= atlas_max_height as u32);
        unsafe {
            *x = ix as u16 + (padded_width - draw_width) / 2;
            *y = iy as u16 + (padded_height - draw_height) / 2;
            *padded_region = AABBu16 {
                left: ix as u16,
                top: iy as u16,
                right: ix as u16 + padded_width,
                bottom: iy as u16 + padded_height,
            };
            self.m_feather_atlas_max_x = self
                .m_feather_atlas_max_x
                .max((*padded_region).right as u32);
            self.m_feather_atlas_max_y = self
                .m_feather_atlas_max_y
                .max((*padded_region).bottom as u32);
        }
        debug_assert!(self.m_feather_atlas_max_x <= atlas_max_width as u32);
        debug_assert!(self.m_feather_atlas_max_y <= atlas_max_height as u32);
        self.m_pending_feather_atlas_draws.push(path_draw);
        true
    }

    pub unsafe fn pushDrawsExecutable(&mut self, draws: &mut [DrawUniquePtr]) -> bool {
        #[cfg(debug_assertions)]
        debug_assert!(!self.m_has_done_layout);
        let mut counts = self.m_resource_counts;
        for owner in draws.iter_mut() {
            let draw = unsafe { &mut *owner.0 };
            let bounds = draw.pixelBounds();
            debug_assert!(bounds.left < bounds.right && bounds.top < bounds.bottom);
            debug_assert!(
                unsafe { self.m_ctx.as_ref() }.frameSupportsClipRectsExecutable()
                    || draw.clipRectInverseMatrix().is_null()
            );
            let add = *draw.resourceCounts();
            counts.midpointFanTessVertexCount += add.midpointFanTessVertexCount;
            counts.outerCubicTessVertexCount += add.outerCubicTessVertexCount;
            counts.pathCount += add.pathCount;
            counts.contourCount += add.contourCount;
            counts.maxTessellatedSegmentCount += add.maxTessellatedSegmentCount;
            counts.maxTriangleVertexCount += add.maxTriangleVertexCount;
            counts.imageDrawCount += add.imageDrawCount;
        }
        let context = unsafe { self.m_ctx.as_ref() };
        if counts.pathCount > context.m_max_path_id
            || counts.contourCount > gpu::kMaxContourID
            || counts.midpointFanTessVertexCount + counts.outerCubicTessVertexCount
                > K_MAX_TESSELLATION_VERTEX_COUNT_BEFORE_PADDING
        {
            return false;
        }
        let mut pass_count_in_batch = 0;
        for owner in draws.iter_mut() {
            let draw = unsafe { &mut *owner.0 };
            unsafe { draw.countSubpasses(self.platformFeatures()) };
            debug_assert!(draw.prepassCount() >= 0);
            debug_assert!(draw.subpassCount() >= 0);
            debug_assert!(draw.prepassCount() + draw.subpassCount() >= 1);
            pass_count_in_batch += draw.prepassCount() + draw.subpassCount();
        }
        if context.frameInterlockMode() != gpu::InterlockMode::rasterOrdering
            && self.m_draw_pass_count + pass_count_in_batch > K_MAX_REORDERED_DRAW_PASS_COUNT
        {
            return false;
        }
        for owner in draws.iter_mut() {
            if !unsafe { (&mut *owner.0).allocateResources(self) } {
                return false;
            }
        }
        for owner in draws.iter_mut() {
            let moved = core::mem::replace(owner, DrawUniquePtr::null());
            self.m_draws.push(moved);
            self.m_combined_draw_contents |=
                unsafe { (&*self.m_draws.last().unwrap().0).drawContents() };
        }
        self.m_resource_counts = counts;
        self.m_draw_pass_count += pass_count_in_batch;
        true
    }

    pub fn allocateMidpointFanTessVerticesExecutable(&mut self, count: u32) -> u32 {
        let location = self.m_midpoint_fan_tess_vertex_idx;
        self.m_midpoint_fan_tess_vertex_idx =
            self.m_midpoint_fan_tess_vertex_idx.wrapping_add(count);
        debug_assert!(self.m_midpoint_fan_tess_vertex_idx <= self.m_midpoint_fan_tess_end_location);
        location
    }

    pub fn allocateOuterCubicTessVerticesExecutable(&mut self, count: u32) -> u32 {
        let location = self.m_outer_cubic_tess_vertex_idx;
        self.m_outer_cubic_tess_vertex_idx = self.m_outer_cubic_tess_vertex_idx.wrapping_add(count);
        debug_assert!(self.m_outer_cubic_tess_vertex_idx <= self.m_outer_cubic_tess_end_location);
        location
    }

    pub unsafe fn pushPathExecutable(&mut self, draw: *const PathDraw) -> u32 {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_has_done_layout);
        self.m_current_path_id += 1;
        let context = unsafe { self.m_ctx.as_mut() };
        debug_assert!(
            self.m_current_path_id > 0 && self.m_current_path_id as usize <= context.m_max_path_id
        );
        let draw = unsafe { &*draw };
        let mut path: gpu::PathData = unsafe { core::mem::zeroed() };
        let atlas = draw.featherAtlasTransform();
        let coverage = draw.coverageBufferRange();
        path.set(
            *draw.matrix(),
            draw.strokeRadius(),
            draw.featherRadius(),
            self.m_current_z_index,
            &atlas,
            &coverage,
        );
        unsafe { context.m_path_data.emplace_back(path) };
        let mut paint: gpu::PaintData = unsafe { core::mem::zeroed() };
        paint.set(
            draw.drawContents(),
            draw.paintType(),
            draw.simplePaintValue(),
            self.m_grad_texture_layout,
            draw.clipID(),
            draw.hasClipRect(),
            draw.blendMode(),
        );
        unsafe { context.m_paint_data.emplace_back(paint) };
        let gradient_coeffs = if draw.gradient().is_null() {
            None
        } else {
            Some(unsafe { (*draw.gradient()).m_coeffs })
        };
        let image_size = if draw.imageTexture().is_null() {
            None
        } else {
            Some((unsafe { (*draw.imageTexture()).width() }, unsafe {
                (*draw.imageTexture()).height()
            }))
        };
        let clip_matrix = if draw.clipRectInverseMatrix().is_null() {
            None
        } else {
            Some(*unsafe { (*draw.clipRectInverseMatrix()).inverseMatrix() })
        };
        let target_height = unsafe { self.m_flush_desc.renderTarget.unwrap().as_ref().height() };
        let mut aux: gpu::PaintAuxData = unsafe { core::mem::zeroed() };
        aux.set(
            *draw.matrix(),
            draw.paintType(),
            draw.simplePaintValue(),
            gradient_coeffs,
            image_size,
            clip_matrix,
            context.platformFeatures().framebufferBottomUp,
            target_height,
        );
        unsafe { context.m_paint_aux_data.emplace_back(aux) };
        debug_assert_eq!(
            self.m_flush_desc.firstPath + self.m_current_path_id as usize + 1,
            context.m_path_data.elementsWritten()
        );
        debug_assert_eq!(
            self.m_flush_desc.firstPaint + self.m_current_path_id as usize + 1,
            context.m_paint_data.elementsWritten()
        );
        debug_assert_eq!(
            self.m_flush_desc.firstPaintAux + self.m_current_path_id as usize + 1,
            context.m_paint_aux_data.elementsWritten()
        );
        self.m_current_path_id
    }

    pub fn pushContourExecutable(
        &mut self,
        path_id: u32,
        mut midpoint: Vec2D,
        is_stroke: bool,
        closed: bool,
        vertex_index_0: u32,
    ) -> u32 {
        debug_assert_ne!(path_id, 0);
        debug_assert!(is_stroke || closed);
        if is_stroke {
            midpoint[0] = if closed { 1.0 } else { 0.0 };
        }
        let context = unsafe { self.m_ctx.as_mut() };
        unsafe {
            context.m_contour_data.emplace_back(gpu::ContourData::new(
                nuxie_render_api::Vec2D::new(midpoint[0], midpoint[1]),
                path_id,
                vertex_index_0,
            ))
        };
        self.m_current_contour_id += 1;
        debug_assert!(
            self.m_current_contour_id > 0
                && self.m_current_contour_id as usize <= gpu::kMaxContourID
        );
        debug_assert_eq!(
            self.m_flush_desc.firstContour + self.m_current_contour_id as usize,
            context.m_contour_data.elementsWritten()
        );
        self.m_current_contour_id
    }

    pub unsafe fn pushMidpointFanDrawExecutable(
        &mut self,
        draw: *const PathDraw,
        draw_type: gpu::DrawType,
        count: u32,
        location: u32,
        misc: gpu::ShaderMiscFlags,
    ) -> *mut gpu::DrawBatch {
        let base = location / gpu::kMidpointFanPatchSegmentSpan;
        debug_assert_eq!(base * gpu::kMidpointFanPatchSegmentSpan, location);
        let instances = count / gpu::kMidpointFanPatchSegmentSpan;
        debug_assert_eq!(instances * gpu::kMidpointFanPatchSegmentSpan, count);
        unsafe { self.pushPathDrawExecutable(draw, draw_type, misc, instances, base) }
    }

    pub unsafe fn pushOuterCubicsDrawExecutable(
        &mut self,
        draw: *const PathDraw,
        draw_type: gpu::DrawType,
        count: u32,
        location: u32,
        misc: gpu::ShaderMiscFlags,
    ) -> *mut gpu::DrawBatch {
        let base = location / gpu::kOuterCurvePatchSegmentSpan;
        debug_assert_eq!(base * gpu::kOuterCurvePatchSegmentSpan, location);
        let instances = count / gpu::kOuterCurvePatchSegmentSpan;
        debug_assert_eq!(instances * gpu::kOuterCurvePatchSegmentSpan, count);
        unsafe { self.pushPathDrawExecutable(draw, draw_type, misc, instances, base) }
    }

    pub unsafe fn pushInteriorTriangulationDrawExecutable(
        &mut self,
        draw: *const PathDraw,
        path_id: u32,
        winding: gpu::WindingFaces,
        misc: gpu::ShaderMiscFlags,
        #[cfg(debug_assertions)] counter: *mut usize,
    ) -> *mut gpu::DrawBatch {
        debug_assert_ne!(path_id, 0);
        let context = unsafe { self.m_ctx.as_mut() };
        let base = context.m_triangle_vertex_data.elementsWritten() as u32;
        let count = unsafe {
            (&*draw).pushInteriorTriangles(path_id, winding, &mut context.m_triangle_vertex_data)
        };
        debug_assert_eq!(
            base as usize + count,
            context.m_triangle_vertex_data.elementsWritten()
        );
        #[cfg(debug_assertions)]
        if !counter.is_null() {
            unsafe {
                *counter += count;
            }
        }
        if count == 0 {
            core::ptr::null_mut()
        } else {
            unsafe {
                self.pushPathDrawExecutable(
                    draw,
                    gpu::DrawType::interiorTriangulation,
                    misc,
                    count as u32,
                    base,
                )
            }
        }
    }

    pub unsafe fn pushFeatherAtlasBlitExecutable(
        &mut self,
        draw: *mut PathDraw,
        path_id: u32,
    ) -> *mut gpu::DrawBatch {
        let context = unsafe { self.m_ctx.as_mut() };
        let base = context.m_triangle_vertex_data.elementsWritten() as u32;
        let b = unsafe { (*draw).pixelBounds() };
        for (x, y) in [
            (b.left, b.bottom),
            (b.left, b.top),
            (b.right, b.bottom),
            (b.right, b.bottom),
            (b.left, b.top),
            (b.right, b.top),
        ] {
            unsafe {
                context
                    .m_triangle_vertex_data
                    .emplace_back(gpu::TriangleVertex::new(
                        nuxie_render_api::Vec2D::new(x as f32, y as f32),
                        1,
                        path_id as u16,
                    ))
            };
        }
        unsafe {
            self.pushPathDrawExecutable(
                draw,
                gpu::DrawType::featherAtlasBlit,
                self.m_baseline_shader_misc_flags,
                6,
                base,
            )
        }
    }

    pub unsafe fn pushImageRectDrawExecutable(
        &mut self,
        draw: *mut ImageRectDraw,
    ) -> *mut gpu::DrawBatch {
        debug_assert!(!unsafe { self.m_ctx.as_ref() }.frameSupportsImagePaintForPathsExecutable());
        let context = unsafe { self.m_ctx.as_mut() };
        let base = context.m_image_draw_instance_data.elementsWritten() as u32;
        let clip = if unsafe { (*draw).clipRectInverseMatrix().is_null() } {
            None
        } else {
            Some(*unsafe { (*(*draw).clipRectInverseMatrix()).inverseMatrix() })
        };
        let instance = gpu::ImageDrawInstance::new(
            *unsafe { (*draw).matrix() },
            unsafe { (*draw).opacity() },
            clip,
            unsafe { (*draw).clipID() },
            unsafe { (*draw).blendMode() },
            self.m_current_z_index,
        );
        unsafe { context.m_image_draw_instance_data.emplace_back(instance) };
        unsafe {
            self.pushDrawExecutable(
                &(*draw).base,
                gpu::DrawType::imageRect,
                self.m_baseline_shader_misc_flags,
                gpu::PaintType::image,
                1,
                base,
            )
        }
    }

    pub unsafe fn pushImageMeshDrawExecutable(
        &mut self,
        draw: *mut ImageMeshDraw,
    ) -> *mut gpu::DrawBatch {
        let context = unsafe { self.m_ctx.as_mut() };
        let base = context.m_image_draw_instance_data.elementsWritten() as u32;
        let clip = if unsafe { (*draw).clipRectInverseMatrix().is_null() } {
            None
        } else {
            Some(*unsafe { (*(*draw).clipRectInverseMatrix()).inverseMatrix() })
        };
        let instance = gpu::ImageDrawInstance::new(
            *unsafe { (*draw).matrix() },
            unsafe { (*draw).opacity() },
            clip,
            unsafe { (*draw).clipID() },
            unsafe { (*draw).blendMode() },
            self.m_current_z_index,
        );
        unsafe { context.m_image_draw_instance_data.emplace_back(instance) };
        let batch = unsafe {
            self.pushDrawExecutable(
                &(*draw).base,
                gpu::DrawType::imageMesh,
                self.m_baseline_shader_misc_flags,
                gpu::PaintType::image,
                1,
                base,
            )
        };
        unsafe {
            (*batch).indexCountPerInstance = (*draw).index_count;
            (*batch).vertexBuffer = NonNull::new((*draw).vertex_buffer);
            (*batch).uvBuffer = NonNull::new((*draw).uv_buffer);
            (*batch).indexBuffer = NonNull::new((*draw).index_buffer);
        }
        batch
    }

    pub unsafe fn pushClipResetDrawExecutable(
        &mut self,
        draw: *mut ClipReset,
    ) -> *mut gpu::DrawBatch {
        let context = unsafe { self.m_ctx.as_mut() };
        let base = context.m_triangle_vertex_data.elementsWritten() as u32;
        let bounds = self
            .getClipInfo(unsafe { (*draw).previousClipID() })
            .contentBounds;
        let z = self.m_current_z_index as u16;
        for (x, y) in [
            (bounds.left, bounds.bottom),
            (bounds.left, bounds.top),
            (bounds.right, bounds.bottom),
            (bounds.right, bounds.bottom),
            (bounds.left, bounds.top),
            (bounds.right, bounds.top),
        ] {
            unsafe {
                context
                    .m_triangle_vertex_data
                    .emplace_back(gpu::TriangleVertex::new(
                        nuxie_render_api::Vec2D::new(x as f32, y as f32),
                        0,
                        z,
                    ))
            };
        }
        unsafe {
            self.pushDrawExecutable(
                &(*draw).base,
                gpu::DrawType::clipReset,
                gpu::ShaderMiscFlags::none,
                gpu::PaintType::clipUpdate,
                6,
                base,
            )
        }
    }

    pub unsafe fn pushPathDrawExecutable(
        &mut self,
        draw: *const PathDraw,
        draw_type: gpu::DrawType,
        mut misc: gpu::ShaderMiscFlags,
        count: u32,
        base: u32,
    ) -> *mut gpu::DrawBatch {
        let context = unsafe { self.m_ctx.as_ref() };
        let draw_ref = unsafe { &*draw };
        if context.frameInterlockMode() == gpu::InterlockMode::rasterOrdering
            && (draw_ref.drawContents().0 & gpu::DrawContents::clockwiseFill.0) != 0
        {
            misc |= gpu::ShaderMiscFlags::clockwiseFill;
        }
        let batch = unsafe {
            self.pushDrawExecutable(
                &draw_ref.base,
                draw_type,
                misc,
                draw_ref.paintType(),
                count,
                base,
            )
        };
        let mut features = gpu::ShaderFeatures::NONE;
        if draw_ref.featherRadius() != 0.0
            && draw_type != gpu::DrawType::interiorTriangulation
            && draw_type != gpu::DrawType::featherAtlasBlit
        {
            features |= gpu::ShaderFeatures::ENABLE_FEATHER;
        }
        if (draw_ref.drawContents().0 & gpu::DrawContents::evenOddFill.0) != 0 {
            features |= gpu::ShaderFeatures::ENABLE_EVEN_ODD;
        }
        let nested = gpu::DrawContents::clipUpdate | gpu::DrawContents::activeClip;
        if (draw_ref.drawContents() & nested) == nested {
            features |= gpu::ShaderFeatures::ENABLE_NESTED_CLIPPING;
        }
        unsafe {
            (*batch).shaderFeatures |= features & context.m_frame_shader_features_mask;
        }
        self.m_combined_shader_features |= unsafe { (*batch).shaderFeatures };
        batch
    }

    pub unsafe fn pushDrawExecutable(
        &mut self,
        draw: *const Draw,
        draw_type: gpu::DrawType,
        mut misc: gpu::ShaderMiscFlags,
        paint_type: gpu::PaintType,
        count: u32,
        base: u32,
    ) -> *mut gpu::DrawBatch {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_has_done_layout);
        debug_assert!(count > 0);
        let draw = unsafe { &*draw };
        let context = unsafe { self.m_ctx.as_ref() };
        misc |= self.m_baseline_shader_misc_flags;
        if (context.frameInterlockMode() == gpu::InterlockMode::clockwise
            || (context.frameInterlockMode() == gpu::InterlockMode::clockwiseAtomic
                && (misc.0 & gpu::ShaderMiscFlags::borrowedCoveragePass.0) == 0))
            && (draw.drawContents().0 & gpu::DrawContents::clipUpdate.0) != 0
        {
            misc |= gpu::ShaderMiscFlags::clipUpdateOnly;
            if context.frameInterlockMode() == gpu::InterlockMode::clockwiseAtomic
                && draw.hasActiveClip()
            {
                misc |= gpu::ShaderMiscFlags::nestedClipUpdateOnly;
            }
        }
        if context.frameInterlockMode() == gpu::InterlockMode::clockwiseAtomic {
            if (misc.0 & gpu::ShaderMiscFlags::borrowedCoveragePass.0) != 0
                || draw.blendMode() == nuxie_render_api::BlendMode::SrcOver
            {
                misc |= gpu::ShaderMiscFlags::fixedFunctionColorOutput;
            }
        } else if context.frameInterlockMode() == gpu::InterlockMode::msaa
            && draw.blendMode() == nuxie_render_api::BlendMode::SrcOver
        {
            misc |= gpu::ShaderMiscFlags::fixedFunctionColorOutput;
        }
        let mergeable_type = !matches!(
            draw_type,
            gpu::DrawType::imageRect
                | gpu::DrawType::imageMesh
                | gpu::DrawType::renderPassInitialize
                | gpu::DrawType::renderPassResolve
        );
        let mut can_merge = false;
        let previous = self.m_draw_list.tail();
        if mergeable_type
            && !previous.is_null()
            && (self.m_pending_barriers.0 & gpu::BarrierFlags::drawBatchBreak.0) == 0
        {
            let current = unsafe { &*previous };
            let any_fill = gpu::DrawContents::clockwiseFill
                | gpu::DrawContents::evenOddFill
                | gpu::DrawContents::nonZeroFill;
            let mut compare_mask = !gpu::ShaderMiscFlags::none;
            if (current.drawContents.0 & any_fill.0) == 0
                || (draw.drawContents().0 & any_fill.0) == 0
            {
                compare_mask &= !gpu::ShaderMiscFlags::clockwiseFill;
            }
            let current_texture = current
                .imageTexture
                .map_or(core::ptr::null_mut(), |p| p.as_ptr());
            let images_combine = current_texture.is_null()
                || draw.imageTexture().is_null()
                || (current_texture == draw.imageTexture()
                    && current.imageSampler == draw.imageSampler());
            can_merge = current.drawType == draw_type
                && (current.shaderMiscFlags & compare_mask) == (misc & compare_mask)
                && images_combine;
            if can_merge && current.baseElement + current.elementCount != base {
                debug_assert_eq!(context.frameInterlockMode(), gpu::InterlockMode::msaa);
                can_merge = false;
            }
            if context.platformFeatures().supportsClipScissor {
                let draw_scissor = draw.scissorRect().map(|r| gpu::AABBu16 {
                    left: r.left,
                    top: r.top,
                    right: r.right,
                    bottom: r.bottom,
                });
                if current.scissorRect != draw_scissor {
                    can_merge = false;
                }
            }
        }
        let batch = if !can_merge {
            let batch = self.m_draw_list.push_back(gpu::DrawBatch::new(
                draw_type,
                misc,
                draw.drawContents(),
                count,
                base,
                draw.blendMode(),
                draw.imageSampler(),
                self.m_pending_barriers,
            ));
            let (index_count, base_index) = patch_indices(draw_type);
            unsafe {
                (*batch).indexCountPerInstance = index_count;
                (*batch).baseIndex = base_index;
            }
            batch
        } else {
            let batch = previous;
            unsafe {
                debug_assert_eq!((*batch).baseElement + (*batch).elementCount, base);
                (*batch).elementCount += count;
                (*batch).shaderMiscFlags |= misc;
                (*batch).drawContents |= draw.drawContents();
                (*batch).barriers |= self.m_pending_barriers;
            }
            batch
        };
        self.m_pending_barriers = gpu::BarrierFlags::none;
        let mut shader_features = gpu::ShaderFeatures::NONE;
        if draw.clipID() != 0 {
            shader_features |= gpu::ShaderFeatures::ENABLE_CLIPPING;
        }
        if draw.hasClipRect() && paint_type != gpu::PaintType::clipUpdate {
            shader_features |= gpu::ShaderFeatures::ENABLE_CLIP_RECT;
        }
        if self.frameDescriptor().ditherMode == DitherMode::interleavedGradientNoise {
            shader_features |= gpu::ShaderFeatures::ENABLE_DITHER;
        }
        if paint_type != gpu::PaintType::clipUpdate
            && (misc.0 & gpu::ShaderMiscFlags::borrowedCoveragePass.0) == 0
        {
            debug_assert_eq!(misc.0 & gpu::ShaderMiscFlags::clipUpdateOnly.0, 0);
            match draw.blendMode() {
                nuxie_render_api::BlendMode::Hue
                | nuxie_render_api::BlendMode::Saturation
                | nuxie_render_api::BlendMode::Color
                | nuxie_render_api::BlendMode::Luminosity => {
                    shader_features |= gpu::ShaderFeatures::ENABLE_HSL_BLEND_MODES
                        | gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND;
                }
                nuxie_render_api::BlendMode::Screen
                | nuxie_render_api::BlendMode::Overlay
                | nuxie_render_api::BlendMode::Darken
                | nuxie_render_api::BlendMode::Lighten
                | nuxie_render_api::BlendMode::ColorDodge
                | nuxie_render_api::BlendMode::ColorBurn
                | nuxie_render_api::BlendMode::HardLight
                | nuxie_render_api::BlendMode::SoftLight
                | nuxie_render_api::BlendMode::Difference
                | nuxie_render_api::BlendMode::Exclusion
                | nuxie_render_api::BlendMode::Multiply => {
                    shader_features |= gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND
                }
                nuxie_render_api::BlendMode::SrcOver => {}
            }
        }
        unsafe {
            (*batch).shaderFeatures |= shader_features & context.m_frame_shader_features_mask;
        }
        if paint_type == gpu::PaintType::image {
            debug_assert!(!draw.imageTexture().is_null());
            unsafe {
                if (*batch).imageTexture.is_none() {
                    (*batch).imageTexture = NonNull::new(draw.imageTexture());
                }
                debug_assert_eq!((*batch).imageTexture.unwrap().as_ptr(), draw.imageTexture());
            }
        }
        self.m_combined_shader_features |= unsafe { (*batch).shaderFeatures };
        batch
    }

    pub fn pushBarriersExecutable(&mut self, mut barriers: gpu::BarrierFlags) {
        if self
            .platformFeatures()
            .clockwiseAtomicBorrowedCoverageBarrierNeedsRenderPassInit
            && (barriers.0 & gpu::BarrierFlags::clockwiseBorrowedCoverage.0) != 0
        {
            let mut workaround =
                gpu::BarrierFlags::clockwiseBorrowedCoverage | gpu::BarrierFlags::plsAtomic;
            if (self.m_combined_draw_contents.0 & gpu::DrawContents::advancedBlend.0) != 0 {
                workaround |= gpu::BarrierFlags::dstBlend;
            }
            self.m_draw_list.push_back(gpu::DrawBatch::new(
                gpu::DrawType::renderPassInitialize,
                self.m_baseline_shader_misc_flags,
                gpu::DrawContents::none,
                1,
                0,
                nuxie_render_api::BlendMode::Overlay,
                crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler::LinearClamp(),
                workaround,
            ));
            barriers.0 &= !gpu::BarrierFlags::clockwiseBorrowedCoverage.0;
        }
        self.m_pending_barriers |= barriers;
    }

    pub unsafe fn writeResourcesExecutable(&mut self) {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_has_done_layout);
        let context = unsafe { self.m_ctx.as_mut() };
        debug_assert_eq!(
            self.m_flush_desc.firstPath,
            context.m_path_data.elementsWritten()
        );
        debug_assert_eq!(
            self.m_flush_desc.firstPaint,
            context.m_paint_data.elementsWritten()
        );
        debug_assert_eq!(
            self.m_flush_desc.firstPaintAux,
            context.m_paint_aux_data.elementsWritten()
        );
        self.m_flush_desc.featherAtlasTextureWidth = context
            .m_current_resource_allocations
            .featherAtlasTextureWidth as u16;
        self.m_flush_desc.featherAtlasTextureHeight = context
            .m_current_resource_allocations
            .featherAtlasTextureHeight as u16;
        self.m_grad_texture_layout.inverseHeight =
            1.0 / context.m_current_resource_allocations.gradTextureHeight as f32;
        let first_tess_span = context.m_tess_span_data.elementsWritten();
        let initial_triangle_bytes = context.m_triangle_vertex_data.bytesWritten();
        let tess_alignment_padding =
            padding_to_align_up(first_tess_span, gpu::kTessVertexBufferAlignmentInElements);
        debug_assert!(tess_alignment_padding <= gpu::kTessVertexBufferAlignmentInElements - 1);
        unsafe {
            context
                .m_tess_span_data
                .push_back_n(core::ptr::null(), tess_alignment_padding)
        };
        self.m_flush_desc.firstTessVertexSpan = first_tess_span + tess_alignment_padding;

        const ONE_TEXEL_FIXED: u32 = 65536 / gpu::kGradTextureWidth;
        debug_assert_eq!(
            self.m_simple_gradients.len(),
            self.m_pending_simple_grad_draws.len()
        );
        for (index, ramp) in self.m_pending_simple_grad_draws.iter().enumerate() {
            let y = (index / gpu::kGradTextureWidthInSimpleRamps as usize) as u32;
            let center_x = (index % gpu::kGradTextureWidthInSimpleRamps as usize) * 2 + 1;
            let fixed = center_x as u32 * ONE_TEXEL_FIXED;
            let mut span = gpu::GradientSpan::default();
            span.set(
                fixed,
                fixed,
                y,
                GRAD_SPAN_FLAG_LEFT_BORDER | GRAD_SPAN_FLAG_RIGHT_BORDER,
                ramp.color0,
                ramp.color1,
            );
            unsafe { context.m_grad_span_data.emplace_back(span) };
        }
        debug_assert_eq!(
            self.m_complex_gradients.len(),
            self.m_pending_complex_grad_draws.len()
        );
        for (index, gradient_ptr) in self.m_pending_complex_grad_draws.iter().enumerate() {
            let gradient = unsafe { &**gradient_ptr };
            let stops = gradient.stops_slice();
            let colors = gradient.colors_slice();
            let y = index as u32 + self.m_grad_texture_layout.complexOffsetY;
            let m = (gpu::kGradTextureWidth as f32 - 1.0) * ONE_TEXEL_FIXED as f32;
            let a = 0.5 * ONE_TEXEL_FIXED as f32;
            let mut last_x = (stops[0] * m + a) as u32;
            let mut last_color = colors[0];
            debug_assert!(stops.len() >= 2);
            for stop_index in 1..stops.len() {
                let x = (stops[stop_index] * m + a) as u32;
                debug_assert!(last_x <= x && x < 65536);
                let mut flags = GRAD_SPAN_FLAG_COMPLEX_BORDER;
                if stop_index == 1 {
                    flags |= GRAD_SPAN_FLAG_LEFT_BORDER;
                }
                if stop_index == stops.len() - 1 {
                    flags |= GRAD_SPAN_FLAG_RIGHT_BORDER;
                }
                let mut span = gpu::GradientSpan::default();
                span.set(last_x, x, y, flags, last_color, colors[stop_index]);
                unsafe { context.m_grad_span_data.emplace_back(span) };
                last_x = x;
                last_color = colors[stop_index];
            }
        }

        let clear_value = gpu::SimplePaintValue {
            color: context.frameDescriptor().clearColor,
        };
        unsafe { context.m_path_data.skip_back() };
        let mut clear_paint: gpu::PaintData = unsafe { core::mem::zeroed() };
        clear_paint.set(
            gpu::DrawContents::none,
            gpu::PaintType::solidColor,
            clear_value,
            gpu::GradTextureLayout::default(),
            0,
            false,
            nuxie_render_api::BlendMode::SrcOver,
        );
        unsafe {
            context.m_paint_data.emplace_back(clear_paint);
            context.m_paint_aux_data.skip_back();
        }
        if self.m_flush_desc.tessDataHeight > 0 {
            self.pushPaddingVerticesExecutable(gpu::kMidpointFanPatchSegmentSpan, 0);
            if self.m_outer_cubic_tess_vertex_idx > self.m_midpoint_fan_tess_end_location {
                self.pushPaddingVerticesExecutable(
                    self.m_outer_cubic_tess_vertex_idx - self.m_midpoint_fan_tess_end_location,
                    self.m_midpoint_fan_tess_end_location,
                );
            }
            self.pushPaddingVerticesExecutable(1, self.m_outer_cubic_tess_end_location);
        }

        let features = *context.platformFeatures();
        if !features.supportsClipScissor
            && context.frameInterlockMode() == gpu::InterlockMode::rasterOrdering
        {
            for draw_index in 0..self.m_draws.len() {
                let draw = unsafe { &mut *self.m_draws[draw_index].0 };
                debug_assert_eq!(draw.prepassCount(), 0);
                debug_assert!(draw.subpassCount() > 0);
                for subpass in 0..draw.subpassCount() {
                    unsafe { draw.pushToRenderContext(self, subpass) };
                }
            }
        } else {
            debug_assert!(self.m_draw_pass_count <= K_MAX_REORDERED_DRAW_PASS_COUNT);
            context.m_indirect_draw_list.clear();
            context
                .m_indirect_draw_list
                .reserve(self.m_draw_pass_count as usize);
            if context.m_intersection_board.is_none() {
                context.m_intersection_board = Some(Box::new(IntersectionBoard::new()));
            }
            let target = self.m_flush_desc.renderTarget.unwrap();
            context
                .m_intersection_board
                .as_mut()
                .unwrap()
                .resizeAndReset(unsafe { target.as_ref().width() }, unsafe {
                    target.as_ref().height()
                });
            context.m_scissor_id_lookup.clear();
            context.m_prev_scissor_id = 0;
            for draw_index in 0..self.m_draws.len() {
                let draw = unsafe { &mut *self.m_draws[draw_index].0 };
                let mut scissor_id = 0i16;
                let mut draw_bounds = *draw.pixelBounds();
                if features.supportsClipScissor
                    && (draw.clipID() != 0 || draw.clippingPixelBounds().is_some())
                {
                    let mut clip = draw.clippingPixelBounds().unwrap_or(IAABB {
                        left: i32::MIN,
                        top: i32::MIN,
                        right: i32::MAX,
                        bottom: i32::MAX,
                    });
                    if draw.clipID() != 0 {
                        let tight = self.getClipInfo(draw.clipID()).tightenedBounds;
                        clip = IAABB {
                            left: clip.left.max(tight.left as i32),
                            top: clip.top.max(tight.top as i32),
                            right: clip.right.min(tight.right as i32),
                            bottom: clip.bottom.min(tight.bottom as i32),
                        };
                    }
                    let target_w = self.frameDescriptor().renderTargetWidth as i32;
                    let target_h = self.frameDescriptor().renderTargetHeight as i32;
                    let visible_draw_bounds = IAABB {
                        left: draw_bounds.left.max(0),
                        top: draw_bounds.top.max(0),
                        right: draw_bounds.right.min(target_w),
                        bottom: draw_bounds.bottom.min(target_h),
                    };
                    let needs_scissor = !(clip.left <= visible_draw_bounds.left
                        && clip.top <= visible_draw_bounds.top
                        && clip.right >= visible_draw_bounds.right
                        && clip.bottom >= visible_draw_bounds.bottom);
                    if needs_scissor {
                        draw_bounds = clip;
                        let clip_u16 = clamp_bounds_u16(clip);
                        if let Some(existing) = context.m_scissor_id_lookup.get(&clip_u16) {
                            scissor_id = *existing;
                        } else {
                            scissor_id = context.m_prev_scissor_id + 1;
                            context.m_scissor_id_lookup.insert(clip_u16, scissor_id);
                            context.m_prev_scissor_id += 1;
                        }
                        draw.setScissorRect(clip_u16);
                    }
                }
                if draw_bounds
                    != (IAABB {
                        left: i32::MIN,
                        top: i32::MIN,
                        right: i32::MAX,
                        bottom: i32::MAX,
                    })
                {
                    draw_bounds = IAABB {
                        left: draw_bounds.left.saturating_sub(1),
                        top: draw_bounds.top.saturating_sub(1),
                        right: draw_bounds.right.saturating_add(1),
                        bottom: draw_bounds.bottom.saturating_add(1),
                    };
                }
                if context.frameInterlockMode() == gpu::InterlockMode::clockwiseAtomic
                    && draw.isClipUpdate()
                {
                    draw_bounds = IAABB {
                        left: 0,
                        top: 0,
                        right: context.frameDescriptor().renderTargetWidth as i32,
                        bottom: context.frameDescriptor().renderTargetHeight as i32,
                    };
                }
                let all_same_group = context.frameInterlockMode() == gpu::InterlockMode::msaa
                    && !features.supportsBlendAdvancedKHR
                    && (self.m_combined_draw_contents.0 & gpu::DrawContents::advancedBlend.0) != 0;
                let max_subpasses = draw.prepassCount().max(draw.subpassCount()) as i8;
                let group = context
                    .m_intersection_board
                    .as_mut()
                    .unwrap()
                    .addRectangle(draw_bounds, if all_same_group { 1 } else { max_subpasses });
                let mut key = make_sort_key(draw, group, scissor_id, 0);
                if draw.prepassCount() > 0 {
                    context.m_indirect_draw_list.push(DrawSortEntry {
                        sortKey: -key,
                        drawIndex: draw_index as i16,
                    });
                }
                if draw.subpassCount() > 0 {
                    context.m_indirect_draw_list.push(DrawSortEntry {
                        sortKey: key,
                        drawIndex: draw_index as i16,
                    });
                }
                for subpass in 1..max_subpasses {
                    let group_increment = if all_same_group {
                        0
                    } else {
                        1i64 << SORT_GROUP_SHIFT
                    };
                    key += group_increment + (1i64 << SORT_SUBPASS_SHIFT);
                    if subpass < draw.prepassCount() as i8 {
                        context.m_indirect_draw_list.push(DrawSortEntry {
                            sortKey: -key,
                            drawIndex: draw_index as i16,
                        });
                    }
                    if subpass < draw.subpassCount() as i8 {
                        context.m_indirect_draw_list.push(DrawSortEntry {
                            sortKey: key,
                            drawIndex: draw_index as i16,
                        });
                    }
                }
            }
            debug_assert_eq!(
                context.m_indirect_draw_list.len(),
                self.m_draw_pass_count as usize
            );
            context
                .m_indirect_draw_list
                .sort_by_key(|entry| entry.sortKey);
            self.writeSortedDrawsExecutable(&features);
        }

        if context.frameInterlockMode() == gpu::InterlockMode::atomics
            || self.m_flush_desc.manuallyResolved
        {
            let barriers = if context.frameInterlockMode() == gpu::InterlockMode::atomics {
                gpu::BarrierFlags::plsAtomicPreResolve
            } else {
                gpu::BarrierFlags::preManualResolve
            };
            let contents = if context.frameInterlockMode() == gpu::InterlockMode::atomics {
                gpu::DrawContents::none
            } else {
                gpu::DrawContents::opaquePaint
            };
            self.m_draw_list.push_back(gpu::DrawBatch::new(
                gpu::DrawType::renderPassResolve, self.m_baseline_shader_misc_flags, contents, 1, 0,
                nuxie_render_api::BlendMode::SrcOver,
                crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler::LinearClamp(), barriers,
            ));
            self.m_combined_draw_contents |= contents;
        }
        self.writeFeatherAtlasBatchesExecutable();
        unsafe {
            context
                .m_path_data
                .push_back_n(core::ptr::null(), self.m_path_padding_count as usize);
            context
                .m_paint_data
                .push_back_n(core::ptr::null(), self.m_paint_padding_count as usize);
            context
                .m_paint_aux_data
                .push_back_n(core::ptr::null(), self.m_paint_aux_padding_count as usize);
            context
                .m_contour_data
                .push_back_n(core::ptr::null(), self.m_contour_padding_count as usize);
            context
                .m_grad_span_data
                .push_back_n(core::ptr::null(), self.m_grad_span_padding_count as usize);
        }
        debug_assert_eq!(
            self.m_midpoint_fan_tess_vertex_idx,
            self.m_midpoint_fan_tess_end_location
        );
        debug_assert_eq!(
            self.m_outer_cubic_tess_vertex_idx,
            self.m_outer_cubic_tess_end_location
        );
        self.m_flush_desc.combinedShaderFeatures = self.m_combined_shader_features;
        if self.m_coverage_buffer_length > 0 {
            debug_assert_eq!(
                self.m_flush_desc.interlockMode,
                gpu::InterlockMode::clockwiseAtomic
            );
            self.m_flush_desc.coverageBufferPrefix = context
                .incrementCoverageBufferPrefixExecutable(
                    &mut self.m_flush_desc.needsCoverageBufferClear,
                );
        }
        self.m_flush_desc.tessVertexSpanCount = (context.m_tess_span_data.elementsWritten()
            - self.m_flush_desc.firstTessVertexSpan)
            as u32;
        self.m_flush_desc.hasTriangleVertices =
            context.m_triangle_vertex_data.bytesWritten() != initial_triangle_bytes;
        self.m_flush_desc.drawList = NonNull::new(&mut self.m_draw_list);
        self.m_flush_desc.firstDstBlendBarrier =
            NonNull::new(self.m_first_dst_blend_barrier as *mut gpu::DrawBatch);
        self.m_flush_desc.unresolvedBarriers = self.m_pending_barriers;
        let uniforms =
            unsafe { gpu::FlushUniforms::new(&self.m_flush_desc, context.platformFeatures()) };
        unsafe { context.m_flush_uniform_data.emplace_back(uniforms) };
        #[cfg(debug_assertions)]
        for batch in self.m_draw_list.iter() {
            debug_assert_eq!(
                (batch.drawContents & self.m_combined_draw_contents),
                batch.drawContents
            );
            debug_assert_eq!(
                (batch.shaderFeatures & self.m_flush_desc.combinedShaderFeatures),
                batch.shaderFeatures
            );
        }
    }

    fn pushPaddingVerticesExecutable(&mut self, count: u32, location: u32) {
        #[cfg(debug_assertions)]
        debug_assert!(self.m_has_done_layout);
        debug_assert!(count > 0);
        let context = unsafe { self.m_ctx.as_mut() };
        let mut remaining = count;
        let mut current = location;
        while remaining > 0 {
            let y = current / gpu::kTessTextureWidth as u32;
            let x0 = (current % gpu::kTessTextureWidth as u32) as i32;
            let x1 = x0 + remaining as i32;
            let mut span = gpu::TessVertexSpan::default();
            span.set_without_reflection(
                [nuxie_render_api::Vec2D::new(0.0, 0.0); 4],
                nuxie_render_api::Vec2D::new(0.0, 0.0),
                y as f32,
                x0,
                x1,
                0,
                0,
                1,
                0,
            );
            unsafe { context.m_tess_span_data.emplace_back(span) };
            if x1 > gpu::kTessTextureWidth as i32 {
                let consumed =
                    gpu::kTessTextureWidth as u32 - current % gpu::kTessTextureWidth as u32;
                current += consumed;
                remaining -= consumed;
            } else {
                break;
            }
        }
    }

    unsafe fn writeSortedDrawsExecutable(&mut self, features: &gpu::PlatformFeatures) {
        let context = unsafe { self.m_ctx.as_mut() };
        debug_assert_eq!(self.m_pending_barriers, gpu::BarrierFlags::none);
        if context.frameInterlockMode() == gpu::InterlockMode::atomics
            && features.atomicPLSInitNeedsDraw
        {
            self.m_draw_list.push_back(gpu::DrawBatch::new(
                gpu::DrawType::renderPassInitialize, self.m_baseline_shader_misc_flags,
                gpu::DrawContents::none, 1, 0, nuxie_render_api::BlendMode::SrcOver,
                crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler::LinearClamp(),
                gpu::BarrierFlags::none,
            ));
        } else if context.frameInterlockMode() == gpu::InterlockMode::msaa
            && self.m_flush_desc.colorLoadAction == gpu::LoadAction::preserveRenderTarget
            && features.msaaColorPreserveNeedsDraw
        {
            let batch = self.m_draw_list.push_back(gpu::DrawBatch::new(
                gpu::DrawType::renderPassInitialize, self.m_baseline_shader_misc_flags,
                gpu::DrawContents::opaquePaint, 1, 0, nuxie_render_api::BlendMode::SrcOver,
                crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler::LinearClamp(),
                gpu::BarrierFlags::dstBlend,
            ));
            self.m_combined_draw_contents |= gpu::DrawContents::opaquePaint;
            self.pushBarriersExecutable(gpu::BarrierFlags::msaaPostInit);
            debug_assert!(self.m_first_dst_blend_barrier.is_null());
            self.m_first_dst_blend_barrier = batch;
            self.m_dst_blend_barrier_list_tail = unsafe {
                &mut (*batch).nextDstBlendBarrier as *mut Option<NonNull<gpu::DrawBatch>>
                    as *mut *const gpu::DrawBatch
            };
        }
        let interlock = self.m_flush_desc.interlockMode;
        match interlock {
            gpu::InterlockMode::atomics => self.pushBarriersExecutable(
                gpu::BarrierFlags::plsAtomic | gpu::BarrierFlags::drawBatchBreak,
            ),
            gpu::InterlockMode::clockwiseAtomic => {
                if context
                    .m_indirect_draw_list
                    .first()
                    .map_or(true, |entry| entry.sortKey >= 0)
                {
                    self.pushBarriersExecutable(gpu::BarrierFlags::clockwiseBorrowedCoverage);
                }
            }
            _ => {}
        }
        let entries = context.m_indirect_draw_list.clone();
        let mut prior: Option<i64> = None;
        let mut current_group = -1i16;
        let mut first_batch: *mut gpu::DrawBatch = core::ptr::null_mut();
        let mut has_cwa_clip_read_barrier = false;
        let mut current_group_has_cwa_clip_update = false;
        for entry in entries {
            let signed_key = entry.sortKey;
            if let Some(previous) = prior {
                match interlock {
                    gpu::InterlockMode::atomics => {
                        if (previous & SORT_GROUP_MASK) != (signed_key & SORT_GROUP_MASK) {
                            self.pushBarriersExecutable(
                                gpu::BarrierFlags::plsAtomic | gpu::BarrierFlags::drawBatchBreak,
                            );
                        }
                    }
                    gpu::InterlockMode::clockwiseAtomic => {
                        if (previous ^ signed_key) < 0 {
                            self.pushBarriersExecutable(
                                gpu::BarrierFlags::clockwiseBorrowedCoverage
                                    | gpu::BarrierFlags::drawBatchBreak,
                            );
                        }
                        if (previous & SORT_GROUP_MASK) != (signed_key & SORT_GROUP_MASK) {
                            self.pushBarriersExecutable(gpu::BarrierFlags::drawBatchBreak);
                        }
                    }
                    gpu::InterlockMode::msaa => {
                        let mut mask = SORT_GROUP_MASK | SORT_CONTENTS_MASK;
                        if features.supportsBlendAdvancedKHR {
                            mask |= SORT_BLEND_MASK;
                        }
                        if (previous & mask) != (signed_key & mask) {
                            self.pushBarriersExecutable(gpu::BarrierFlags::drawBatchBreak);
                        }
                    }
                    gpu::InterlockMode::rasterOrdering | gpu::InterlockMode::clockwise => {}
                }
            }
            let key = signed_key.unsigned_abs();
            let mut subpass = ((key >> SORT_SUBPASS_SHIFT) & 7) as i32;
            if signed_key < 0 {
                subpass = -1 - subpass;
            }
            let group = ((key >> SORT_GROUP_SHIFT) & 0x7fff) as i16;
            debug_assert!(group > 0);
            self.m_current_z_index = group as u32;
            let draw = unsafe { &mut *self.m_draws[entry.drawIndex as usize].0 };
            let batch = unsafe { draw.pushToRenderContext(self, subpass) };
            if !batch.is_null() && features.supportsClipScissor {
                unsafe {
                    (*batch).scissorRect = draw.scissorRect().map(|r| gpu::AABBu16 {
                        left: r.left,
                        top: r.top,
                        right: r.right,
                        bottom: r.bottom,
                    })
                };
            }
            if matches!(
                context.frameInterlockMode(),
                gpu::InterlockMode::clockwiseAtomic | gpu::InterlockMode::msaa
            ) && subpass == 0
                && !batch.is_null()
            {
                if current_group != group {
                    if current_group_has_cwa_clip_update {
                        has_cwa_clip_read_barrier = false;
                        current_group_has_cwa_clip_update = false;
                    }
                    first_batch = batch;
                    current_group = group;
                }
                if draw.hasAdvancedBlend()
                    && (context.frameInterlockMode() != gpu::InterlockMode::msaa
                        || !features.supportsBlendAdvancedCoherentKHR)
                {
                    debug_assert!(draw.nextDstRead().is_null());
                    let old_head = unsafe {
                        (*first_batch)
                            .dstReadList
                            .map_or(core::ptr::null(), |p| p.as_ptr())
                    };
                    let new_head = unsafe { draw.addToDstReadList(old_head) as *mut Draw };
                    unsafe { (*first_batch).dstReadList = NonNull::new(new_head) };
                    if unsafe { ((*first_batch).barriers.0 & gpu::BarrierFlags::dstBlend.0) == 0 } {
                        unsafe { (*first_batch).barriers |= gpu::BarrierFlags::dstBlend };
                        unsafe { self.addBatchToDstBarrierListExecutable(first_batch) };
                    }
                }
                if context.frameInterlockMode() == gpu::InterlockMode::clockwiseAtomic {
                    if draw.isClipUpdate() {
                        current_group_has_cwa_clip_update = true;
                    } else if draw.hasActiveClip() && !has_cwa_clip_read_barrier {
                        unsafe {
                            (*first_batch).barriers |= gpu::BarrierFlags::plsAtomic;
                        }
                        has_cwa_clip_read_barrier = true;
                    }
                } else {
                    debug_assert_eq!(context.frameInterlockMode(), gpu::InterlockMode::msaa);
                }
            }
            prior = Some(signed_key);
        }
    }

    unsafe fn addBatchToDstBarrierListExecutable(&mut self, batch: *mut gpu::DrawBatch) {
        unsafe {
            if self.m_first_dst_blend_barrier.is_null() {
                self.m_first_dst_blend_barrier = batch;
            } else {
                let mut cursor = self.m_first_dst_blend_barrier as *mut gpu::DrawBatch;
                while let Some(next) = (*cursor).nextDstBlendBarrier {
                    cursor = next.as_ptr();
                }
                (*cursor).nextDstBlendBarrier = NonNull::new(batch);
            }
            (*batch).nextDstBlendBarrier = None;
        }
    }

    unsafe fn writeFeatherAtlasBatchesExecutable(&mut self) {
        if self.m_pending_feather_atlas_draws.is_empty() {
            return;
        }
        let context = unsafe { self.m_ctx.as_mut() };
        let full = gpu::AABBu16 {
            left: 0,
            top: 0,
            right: self.m_flush_desc.featherAtlasContentWidth,
            bottom: self.m_flush_desc.featherAtlasContentHeight,
        };
        let start = unsafe {
            context
                .m_per_frame_allocator
                .makePODArray::<gpu::AtlasDrawBatch>(self.m_pending_feather_atlas_draws.len())
        };
        let mut current = start;
        for stroked in [false, true] {
            let category_start = current;
            if stroked {
                self.m_flush_desc.featherAtlasStrokeBatches = NonNull::new(current);
            } else {
                self.m_flush_desc.featherAtlasFillBatches = NonNull::new(current);
            }
            for scissored in [false, true] {
                let mut last: *mut gpu::AtlasDrawBatch = core::ptr::null_mut();
                for draw_index in 0..self.m_pending_feather_atlas_draws.len() {
                    let draw_ptr = self.m_pending_feather_atlas_draws[draw_index];
                    let draw = unsafe { &mut *draw_ptr };
                    if draw.isStroke() != stroked || draw.featherAtlasScissorEnabled() != scissored
                    {
                        continue;
                    }
                    let mut vertex_count = 0;
                    let mut base_vertex = 0;
                    unsafe {
                        draw.pushFeatherAtlasTessellation(self, &mut vertex_count, &mut base_vertex)
                    };
                    if vertex_count == 0 {
                        continue;
                    }
                    let patch_count = vertex_count / gpu::kMidpointFanPatchSegmentSpan;
                    let base_patch = base_vertex / gpu::kMidpointFanPatchSegmentSpan;
                    if last.is_null() || scissored {
                        last = current;
                        let scissor = if scissored {
                            let r = draw.featherAtlasScissor();
                            gpu::AABBu16 {
                                left: r.left,
                                top: r.top,
                                right: r.right,
                                bottom: r.bottom,
                            }
                        } else {
                            full
                        };
                        unsafe {
                            core::ptr::write(
                                current,
                                gpu::AtlasDrawBatch {
                                    scissor,
                                    patchCount: patch_count,
                                    basePatch: base_patch,
                                },
                            );
                            current = current.add(1);
                        }
                    } else {
                        debug_assert_eq!(
                            unsafe { (*last).basePatch + (*last).patchCount },
                            base_patch
                        );
                        unsafe {
                            (*last).patchCount += patch_count;
                        }
                    }
                }
            }
            let count = unsafe { current.offset_from(category_start) as usize };
            if stroked {
                self.m_flush_desc.featherAtlasStrokeBatchCount = count;
            } else {
                self.m_flush_desc.featherAtlasFillBatchCount = count;
            }
        }
        debug_assert!(
            self.m_flush_desc.featherAtlasFillBatchCount
                + self.m_flush_desc.featherAtlasStrokeBatchCount
                <= self.m_pending_feather_atlas_draws.len()
        );
    }

    pub unsafe fn layoutResourcesExecutable(
        &mut self,
        flush_resources: &FlushResources,
        logical_flush_index: usize,
        running_resources: &mut ResourceCounters,
        running_layout: &mut LayoutCounters,
    ) {
        #[cfg(debug_assertions)]
        debug_assert!(!self.m_has_done_layout);
        let context = unsafe { self.m_ctx.as_mut() };
        let frame = context.frameDescriptor();
        self.m_resource_counts.pathCount += 1;
        self.m_path_padding_count = padding_to_align_up(
            self.m_resource_counts.pathCount,
            gpu::kPathBufferAlignmentInElements,
        ) as u32;
        self.m_paint_padding_count = padding_to_align_up(
            self.m_resource_counts.pathCount,
            gpu::kPaintBufferAlignmentInElements,
        ) as u32;
        self.m_paint_aux_padding_count = padding_to_align_up(
            self.m_resource_counts.pathCount,
            gpu::kPaintAuxBufferAlignmentInElements,
        ) as u32;
        self.m_contour_padding_count = padding_to_align_up(
            self.m_resource_counts.contourCount,
            gpu::kContourBufferAlignmentInElements,
        ) as u32;
        self.m_grad_span_padding_count = padding_to_align_up(
            self.m_pending_grad_span_count,
            gpu::kGradSpanBufferAlignmentInElements,
        ) as u32;

        let mut total_tess_count = 0usize;
        if (self.m_resource_counts.midpointFanTessVertexCount
            | self.m_resource_counts.outerCubicTessVertexCount)
            != 0
        {
            let pre_padding = gpu::kMidpointFanPatchSegmentSpan;
            self.m_midpoint_fan_tess_vertex_idx = pre_padding;
            self.m_midpoint_fan_tess_end_location =
                pre_padding + self.m_resource_counts.midpointFanTessVertexCount as u32;
            let interior_padding = padding_to_align_up(
                self.m_midpoint_fan_tess_end_location as usize,
                gpu::kOuterCurvePatchSegmentSpan as usize,
            ) as u32;
            self.m_outer_cubic_tess_vertex_idx =
                self.m_midpoint_fan_tess_end_location + interior_padding;
            self.m_outer_cubic_tess_end_location = self.m_outer_cubic_tess_vertex_idx
                + self.m_resource_counts.outerCubicTessVertexCount as u32;
            total_tess_count = self.m_outer_cubic_tess_end_location as usize + 1;
            debug_assert!(
                pre_padding as usize + interior_padding as usize + 1
                    <= K_MAX_TESSELLATION_PADDING_VERTEX_COUNT
            );
            debug_assert!(total_tess_count <= K_MAX_TESSELLATION_VERTEX_COUNT);
        }
        let tess_height = resource_texture_height(total_tess_count, gpu::kTessTextureWidth) as u32;
        if self.m_resource_counts.maxTessellatedSegmentCount != 0 {
            self.m_resource_counts.maxTessellatedSegmentCount +=
                tess_height as usize * 2 + 3 + (gpu::kTessVertexBufferAlignmentInElements - 1);
        }
        self.m_grad_texture_layout.complexOffsetY = resource_texture_height(
            self.m_simple_gradients.len(),
            gpu::kGradTextureWidthInSimpleRamps as usize,
        ) as u32;
        self.m_flush_desc.renderTarget = NonNull::new(flush_resources.renderTarget);
        self.m_flush_desc.interlockMode = context.frameInterlockMode();
        self.m_flush_desc.msaaSampleCount = frame.msaaSampleCount as i32;
        let mut clear_during_atomic_resolve = false;
        if logical_flush_index != 0 {
            self.m_flush_desc.colorLoadAction = gpu::LoadAction::preserveRenderTarget;
        } else if frame.loadAction == gpu::LoadAction::clear {
            clear_during_atomic_resolve = context.frameInterlockMode()
                == gpu::InterlockMode::atomics
                && ((frame.clearColor >> 24) & 0xff) == 255;
            self.m_flush_desc.colorLoadAction = if clear_during_atomic_resolve {
                gpu::LoadAction::dontCare
            } else {
                gpu::LoadAction::clear
            };
        } else {
            self.m_flush_desc.colorLoadAction = frame.loadAction;
        }
        self.m_flush_desc.colorClearValue = frame.clearColor;
        self.m_flush_desc.coverageClearValue = if clear_during_atomic_resolve {
            debug_assert_eq!(self.m_flush_desc.interlockMode, gpu::InterlockMode::atomics);
            (1 << 16) + 2048
        } else if self.m_flush_desc.interlockMode == gpu::InterlockMode::atomics {
            1 << 16
        } else {
            0
        };
        self.tightenClipBoundsExecutable();
        let source_target_bounds = unsafe { (&*flush_resources.renderTarget).bounds() };
        let target_bounds = gpu::IAABB {
            left: source_target_bounds.left,
            top: source_target_bounds.top,
            right: source_target_bounds.right,
            bottom: source_target_bounds.bottom,
        };
        self.m_flush_desc.renderTargetUpdateBounds = if clear_during_atomic_resolve
            || self.m_flush_desc.colorLoadAction == gpu::LoadAction::clear
        {
            target_bounds
        } else {
            intersect_gpu_bounds(target_bounds, self.m_combined_draw_bounds)
        };
        if self.m_flush_desc.renderTargetUpdateBounds.left
            >= self.m_flush_desc.renderTargetUpdateBounds.right
            || self.m_flush_desc.renderTargetUpdateBounds.top
                >= self.m_flush_desc.renderTargetUpdateBounds.bottom
        {
            self.m_flush_desc.renderTargetUpdateBounds = gpu::IAABB::default();
        }
        self.m_flush_desc.virtualTileWidth = frame.virtualTileWidth;
        self.m_flush_desc.virtualTileHeight = frame.virtualTileHeight;
        self.m_flush_desc.manuallyResolved = unsafe {
            context.m_impl.contract().wantsManualRenderPassResolve(
                self.m_flush_desc.interlockMode,
                flush_resources.renderTarget,
                &self.m_flush_desc.renderTargetUpdateBounds,
                self.m_flush_desc.virtualTileWidth,
                self.m_flush_desc.virtualTileHeight,
                self.m_combined_draw_contents,
            )
        };
        self.m_flush_desc.fixedFunctionColorOutput = wants_fixed_function_color_output(
            context.platformFeatures(),
            context.frameInterlockMode(),
            self.m_combined_draw_contents,
            self.m_flush_desc.manuallyResolved,
        );
        if self.m_flush_desc.fixedFunctionColorOutput {
            self.m_baseline_shader_misc_flags |= gpu::ShaderMiscFlags::fixedFunctionColorOutput;
        }
        self.m_flush_desc.featherAtlasContentWidth = self.m_feather_atlas_max_x as u16;
        self.m_flush_desc.featherAtlasContentHeight = self.m_feather_atlas_max_y as u16;
        self.m_flush_desc.flushUniformDataOffsetInBytes =
            logical_flush_index * core::mem::size_of::<gpu::FlushUniforms>();
        self.m_flush_desc.pathCount = self.m_resource_counts.pathCount as u32;
        self.m_flush_desc.firstPath =
            running_resources.pathCount + running_layout.pathPaddingCount as usize;
        self.m_flush_desc.firstPaint =
            running_resources.pathCount + running_layout.paintPaddingCount as usize;
        self.m_flush_desc.firstPaintAux =
            running_resources.pathCount + running_layout.paintAuxPaddingCount as usize;
        self.m_flush_desc.contourCount = self.m_resource_counts.contourCount as u32;
        self.m_flush_desc.firstContour =
            running_resources.contourCount + running_layout.contourPaddingCount as usize;
        self.m_flush_desc.gradSpanCount = self.m_pending_grad_span_count as u32;
        self.m_flush_desc.firstGradSpan =
            running_layout.gradSpanCount as usize + running_layout.gradSpanPaddingCount as usize;
        self.m_flush_desc.gradDataHeight =
            self.m_grad_texture_layout.complexOffsetY + self.m_complex_gradients.len() as u32;
        self.m_flush_desc.tessDataHeight = tess_height;
        self.m_flush_desc.clockwiseFillOverride = frame.clockwiseFillOverride;
        self.m_flush_desc.wireframe = frame.wireframe;
        self.m_flush_desc.ditherMode = frame.ditherMode;
        #[cfg(feature = "with-rive-tools")]
        {
            self.m_flush_desc.synthesizedFailureType = frame.synthesizedFailureType;
        }
        self.m_flush_desc.externalCommandBuffer =
            NonNull::new(flush_resources.externalCommandBuffer);

        running_resources.midpointFanTessVertexCount +=
            self.m_resource_counts.midpointFanTessVertexCount;
        running_resources.outerCubicTessVertexCount +=
            self.m_resource_counts.outerCubicTessVertexCount;
        running_resources.pathCount += self.m_resource_counts.pathCount;
        running_resources.contourCount += self.m_resource_counts.contourCount;
        running_resources.maxTessellatedSegmentCount +=
            self.m_resource_counts.maxTessellatedSegmentCount;
        running_resources.maxTriangleVertexCount += self.m_resource_counts.maxTriangleVertexCount;
        running_resources.imageDrawCount += self.m_resource_counts.imageDrawCount;
        running_layout.pathPaddingCount += self.m_path_padding_count;
        running_layout.paintPaddingCount += self.m_paint_padding_count;
        running_layout.paintAuxPaddingCount += self.m_paint_aux_padding_count;
        running_layout.contourPaddingCount += self.m_contour_padding_count;
        running_layout.gradSpanCount += self.m_pending_grad_span_count as u32;
        running_layout.gradSpanPaddingCount += self.m_grad_span_padding_count;
        running_layout.maxGradTextureHeight = running_layout
            .maxGradTextureHeight
            .max(self.m_flush_desc.gradDataHeight);
        running_layout.maxTessTextureHeight = running_layout
            .maxTessTextureHeight
            .max(self.m_flush_desc.tessDataHeight);
        running_layout.maxFeatherAtlasWidth = running_layout
            .maxFeatherAtlasWidth
            .max(self.m_feather_atlas_max_x);
        running_layout.maxFeatherAtlasHeight = running_layout
            .maxFeatherAtlasHeight
            .max(self.m_feather_atlas_max_y);
        running_layout.maxPLSTransientBackingPlaneCount = running_layout
            .maxPLSTransientBackingPlaneCount
            .max(pls_transient_backing_plane_count(
                self.m_flush_desc.interlockMode,
                self.m_combined_draw_contents,
            ));
        running_layout.maxCoverageBufferLength = running_layout
            .maxCoverageBufferLength
            .max(self.m_coverage_buffer_length as usize);
        debug_assert_eq!(
            self.m_flush_desc.firstPath % gpu::kPathBufferAlignmentInElements,
            0
        );
        debug_assert_eq!(
            self.m_flush_desc.firstPaint % gpu::kPaintBufferAlignmentInElements,
            0
        );
        debug_assert_eq!(
            self.m_flush_desc.firstPaintAux % gpu::kPaintAuxBufferAlignmentInElements,
            0
        );
        debug_assert_eq!(
            self.m_flush_desc.firstContour % gpu::kContourBufferAlignmentInElements,
            0
        );
        debug_assert_eq!(
            self.m_flush_desc.firstGradSpan % gpu::kGradSpanBufferAlignmentInElements,
            0
        );
        #[cfg(debug_assertions)]
        {
            self.m_has_done_layout = true;
        }
    }

    pub fn tightenClipBoundsExecutable(&mut self) {
        debug_assert_eq!(self.m_combined_draw_bounds, maximally_negative_i32());
        let supports_scissor = self.platformFeatures().supportsClipScissor;
        for index in (0..self.m_draws.len()).rev() {
            let draw = unsafe { &*self.m_draws[index].0 };
            let mut combined_bounds = if supports_scissor {
                *draw.clippedPixelBounds()
            } else {
                *draw.pixelBounds()
            };
            if draw.clipID() == 0 {
            } else if draw.isClipUpdate() {
                let clip_id = draw.clipID();
                let active = draw.hasActiveClip();
                let clip = self.getWritableClipInfoExecutable(clip_id);
                clip.tightenedBounds = intersect_u16(clip.tightenedBounds, clip.readBounds);
                let tightened = clip.tightenedBounds;
                let parent_id = clip.parentClipID;
                if supports_scissor {
                    debug_assert!(
                        combined_bounds.left <= tightened.left as i32
                            && combined_bounds.top <= tightened.top as i32
                            && combined_bounds.right >= tightened.right as i32
                            && combined_bounds.bottom >= tightened.bottom as i32
                    );
                    combined_bounds = IAABB {
                        left: tightened.left as i32,
                        top: tightened.top as i32,
                        right: tightened.right as i32,
                        bottom: tightened.bottom as i32,
                    };
                }
                if active {
                    debug_assert_ne!(parent_id, 0);
                    let parent = self.getWritableClipInfoExecutable(parent_id);
                    parent.readBounds = join_u16(parent.readBounds, tightened);
                } else {
                    debug_assert_eq!(parent_id, 0);
                }
            } else if draw.hasActiveClip() {
                let clip = self.getWritableClipInfoExecutable(draw.clipID());
                clip.readBounds = join_u16(
                    clip.readBounds,
                    clamp_bounds_u16(*draw.clippedPixelBounds()),
                );
            }
            self.m_combined_draw_bounds = join_i32(self.m_combined_draw_bounds, combined_bounds);
        }
    }
}

impl<'a> TessellationWriter<'a> {
    pub unsafe fn newExecutable(
        flush: &'a mut LogicalFlush,
        path_id: u32,
        directions: gpu::ContourDirections,
        forward_count: u32,
        forward_location: u32,
        mirrored_count: u32,
        mirrored_location: u32,
    ) -> Self {
        let context = unsafe { flush.m_ctx.as_mut() };
        #[cfg(debug_assertions)]
        debug_assert!(flush.m_has_done_layout);
        debug_assert!(context.m_path_data.elementsWritten() > 0);
        debug_assert!(forward_count == 0 || mirrored_count == 0 || forward_count == mirrored_count);
        debug_assert!(
            !gpu::ContourDirectionsAreDoubleSided(directions) || forward_count == mirrored_count
        );
        Self {
            m_flush: flush,
            m_tess_span_data: unsafe { &mut *(&mut context.m_tess_span_data as *mut _) },
            m_path_id: path_id,
            m_contour_directions: directions,
            m_path_tess_location: forward_location,
            m_path_mirrored_tess_location: mirrored_location,
            m_next_cubic_padding_vertex_count: 0,
            #[cfg(debug_assertions)]
            m_expected_path_tess_end_location: forward_location + forward_count,
            #[cfg(debug_assertions)]
            m_expected_path_mirrored_tess_end_location: mirrored_location - mirrored_count,
        }
    }

    pub fn pushContourExecutable(
        &mut self,
        midpoint: Vec2D,
        is_stroke: bool,
        closed: bool,
        padding: u32,
    ) -> u32 {
        self.m_next_cubic_padding_vertex_count = padding;
        self.m_flush.pushContourExecutable(
            self.m_path_id,
            midpoint,
            is_stroke,
            closed,
            self.nextVertexIndex(),
        )
    }

    pub fn pushCubicExecutable(
        &mut self,
        pts: &[Vec2D; 4],
        directions: gpu::ContourDirections,
        join_tangent: Vec2D,
        parametric_count: u32,
        polar_count: u32,
        join_count: u32,
        contour_id_flags: u32,
    ) {
        debug_assert!(parametric_count <= gpu::kMaxParametricSegments);
        debug_assert!(polar_count <= gpu::kMaxPolarSegments);
        debug_assert!(join_count > 0);
        debug_assert_eq!(
            contour_id_flags & gpu::kContourIDMask,
            self.m_flush.m_current_contour_id & gpu::kContourIDMask
        );
        debug_assert_ne!(contour_id_flags & gpu::kContourIDMask, 0);
        debug_assert!((contour_id_flags & gpu::kContourIDMask) <= self.m_flush.desc().contourCount);
        let total =
            self.m_next_cubic_padding_vertex_count + parametric_count + polar_count + join_count
                - 1;
        self.m_next_cubic_padding_vertex_count = 0;
        match directions {
            gpu::ContourDirections::forward => self.pushTessellationSpansExecutable(
                pts,
                join_tangent,
                total,
                parametric_count,
                polar_count,
                join_count,
                contour_id_flags,
            ),
            gpu::ContourDirections::reverse => self.pushMirroredTessellationSpansExecutable(
                pts,
                join_tangent,
                total,
                parametric_count,
                polar_count,
                join_count,
                contour_id_flags,
            ),
            gpu::ContourDirections::reverseThenForward
            | gpu::ContourDirections::forwardThenReverse => self
                .pushDoubleSidedTessellationSpansExecutable(
                    pts,
                    join_tangent,
                    total,
                    parametric_count,
                    polar_count,
                    join_count,
                    contour_id_flags,
                ),
        }
    }

    fn converted_points(pts: &[Vec2D; 4]) -> [nuxie_render_api::Vec2D; 4] {
        [
            nuxie_render_api::Vec2D::new(pts[0][0], pts[0][1]),
            nuxie_render_api::Vec2D::new(pts[1][0], pts[1][1]),
            nuxie_render_api::Vec2D::new(pts[2][0], pts[2][1]),
            nuxie_render_api::Vec2D::new(pts[3][0], pts[3][1]),
        ]
    }

    pub fn pushTessellationSpansExecutable(
        &mut self,
        pts: &[Vec2D; 4],
        tangent: Vec2D,
        total: u32,
        parametric: u32,
        polar: u32,
        join: u32,
        contour: u32,
    ) {
        debug_assert!(total > 0);
        let mut y = self.m_path_tess_location / gpu::kTessTextureWidth as u32;
        let mut x0 = (self.m_path_tess_location % gpu::kTessTextureWidth as u32) as i32;
        let mut x1 = x0 + total as i32;
        loop {
            let mut span = gpu::TessVertexSpan::default();
            span.set_without_reflection(
                Self::converted_points(pts),
                nuxie_render_api::Vec2D::new(tangent[0], tangent[1]),
                y as f32,
                x0,
                x1,
                parametric,
                polar,
                join,
                contour,
            );
            unsafe { self.m_tess_span_data.emplace_back(span) };
            if x1 > gpu::kTessTextureWidth as i32 {
                y += 1;
                x0 -= gpu::kTessTextureWidth as i32;
                x1 -= gpu::kTessTextureWidth as i32;
            } else {
                break;
            }
        }
        debug_assert_eq!(
            y,
            (self.m_path_tess_location + total - 1) / gpu::kTessTextureWidth as u32,
        );
        self.m_path_tess_location += total;
        #[cfg(debug_assertions)]
        debug_assert!(self.m_path_tess_location <= self.m_expected_path_tess_end_location);
    }

    pub fn pushMirroredTessellationSpansExecutable(
        &mut self,
        pts: &[Vec2D; 4],
        tangent: Vec2D,
        total: u32,
        parametric: u32,
        polar: u32,
        join: u32,
        contour: u32,
    ) {
        debug_assert!(total > 0);
        let mut y = (self.m_path_mirrored_tess_location - 1) / gpu::kTessTextureWidth as u32;
        let mut x0 =
            ((self.m_path_mirrored_tess_location - 1) % gpu::kTessTextureWidth as u32 + 1) as i32;
        let mut x1 = x0 - total as i32;
        loop {
            let mut span = gpu::TessVertexSpan::default();
            // The reverse-only source calls the 9-argument overload: reverse
            // coordinates occupy the primary y/x fields and reflection stays
            // discarded, rather than using the double-sided overload.
            span.set_without_reflection(
                Self::converted_points(pts),
                nuxie_render_api::Vec2D::new(tangent[0], tangent[1]),
                y as f32,
                x0,
                x1,
                parametric,
                polar,
                join,
                contour,
            );
            unsafe { self.m_tess_span_data.emplace_back(span) };
            if x1 < 0 {
                y -= 1;
                x0 += gpu::kTessTextureWidth as i32;
                x1 += gpu::kTessTextureWidth as i32;
            } else {
                break;
            }
        }
        self.m_path_mirrored_tess_location -= total;
        #[cfg(debug_assertions)]
        debug_assert!(
            self.m_path_mirrored_tess_location >= self.m_expected_path_mirrored_tess_end_location
        );
    }

    pub fn pushDoubleSidedTessellationSpansExecutable(
        &mut self,
        pts: &[Vec2D; 4],
        tangent: Vec2D,
        total: u32,
        parametric: u32,
        polar: u32,
        join: u32,
        contour: u32,
    ) {
        debug_assert!(total > 0);
        let mut y = self.m_path_tess_location / gpu::kTessTextureWidth as u32;
        let mut x0 = (self.m_path_tess_location % gpu::kTessTextureWidth as u32) as i32;
        let mut x1 = x0 + total as i32;
        let mut ry = (self.m_path_mirrored_tess_location - 1) / gpu::kTessTextureWidth as u32;
        let mut rx0 =
            ((self.m_path_mirrored_tess_location - 1) % gpu::kTessTextureWidth as u32 + 1) as i32;
        let mut rx1 = rx0 - total as i32;
        loop {
            let mut span = gpu::TessVertexSpan::default();
            span.set(
                Self::converted_points(pts),
                nuxie_render_api::Vec2D::new(tangent[0], tangent[1]),
                y as f32,
                x0,
                x1,
                ry as f32,
                rx0,
                rx1,
                parametric,
                polar,
                join,
                contour,
            );
            unsafe { self.m_tess_span_data.emplace_back(span) };
            if x1 > gpu::kTessTextureWidth as i32 || rx1 < 0 {
                y += 1;
                x0 -= gpu::kTessTextureWidth as i32;
                x1 -= gpu::kTessTextureWidth as i32;
                ry -= 1;
                rx0 += gpu::kTessTextureWidth as i32;
                rx1 += gpu::kTessTextureWidth as i32;
            } else {
                break;
            }
        }
        self.m_path_tess_location += total;
        self.m_path_mirrored_tess_location -= total;
        #[cfg(debug_assertions)]
        {
            debug_assert!(self.m_path_tess_location <= self.m_expected_path_tess_end_location);
            debug_assert!(
                self.m_path_mirrored_tess_location
                    >= self.m_expected_path_mirrored_tess_end_location
            );
        }
    }
}

impl Drop for TessellationWriter<'_> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            debug_assert_eq!(
                self.m_path_tess_location,
                self.m_expected_path_tess_end_location
            );
            debug_assert_eq!(
                self.m_path_mirrored_tess_location,
                self.m_expected_path_mirrored_tess_end_location
            );
        }
    }
}

impl FactoryAccess for RenderContext {
    fn factory(&self) -> &Factory {
        &self.base.base
    }
    fn factoryMut(&mut self) -> &mut Factory {
        &mut self.base.base
    }
}

impl RiveRenderFactoryAccess for RenderContext {
    fn riveRenderFactory(&self) -> &RiveRenderFactory {
        &self.base
    }
    fn riveRenderFactoryMut(&mut self) -> &mut RiveRenderFactory {
        &mut self.base
    }
}

impl FactoryContract for RenderContext {
    fn makeRenderBuffer(
        &mut self,
        buffer_type: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType,
        flags: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags,
        size_in_bytes: usize,
    ) -> rcp<RenderBuffer> {
        RenderContext::makeRenderBuffer(self, buffer_type, flags, size_in_bytes)
    }

    unsafe fn makeLinearGradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: *const ColorInt,
        stops: *const f32,
        count: usize,
    ) -> rcp<RenderShader> {
        // SAFETY: the Factory virtual caller supplies valid `[count]` arrays
        // for the duration of this source-shaped copy operation.
        unsafe {
            self.base
                .makeLinearGradientSource(sx, sy, ex, ey, colors, stops, count)
        }
    }

    unsafe fn makeRadialGradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: *const ColorInt,
        stops: *const f32,
        count: usize,
    ) -> rcp<RenderShader> {
        // SAFETY: the Factory virtual caller supplies valid `[count]` arrays
        // for the duration of this source-shaped copy operation.
        unsafe {
            self.base
                .makeRadialGradientSource(cx, cy, radius, colors, stops, count)
        }
    }

    fn makeRenderPath(&mut self, raw_path: &mut RawPath, fill_rule: FillRule) -> rcp<RenderPath> {
        RenderContext::makeRenderPath(self, raw_path, fill_rule)
    }

    fn makeEmptyRenderPath(&mut self) -> rcp<RenderPath> {
        RenderContext::makeEmptyRenderPath(self)
    }

    fn makeRenderPaint(&mut self) -> rcp<RenderPaint> {
        RenderContext::makeRenderPaint(self)
    }

    unsafe fn decodeImage(&mut self, encoded: *const u8, size: usize) -> rcp<RenderImage> {
        // SAFETY: this is the source Span ABI; callers must provide a valid
        // byte range (or a null pointer only when size is zero).
        unsafe {
            self.decodeImageExecutable(Span {
                data: encoded,
                size,
            })
        }
    }

    unsafe fn ore(&mut self) -> *mut OreContext {
        #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
        {
            // SAFETY: the concrete RenderContext owns the configured ORE
            // context for the duration of this source virtual call.
            return self.oreExecutable();
        }
        #[cfg(not(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental")))]
        {
            core::ptr::null_mut()
        }
    }
}

impl RiveRenderFactoryContract for RenderContext {}

impl RenderContextContract for RenderContext {
    fn new<T>(implementation: Box<T>) -> Pin<Box<Self>>
    where
        T: RenderContextImplContract + 'static,
    {
        RenderContext::from_impl(implementation)
    }
    fn impl_ptr(&self) -> *mut RenderContextImpl {
        RenderContext::impl_ptr(self)
    }
    fn static_impl_cast<T>(&self) -> *mut T
    where
        Self: Sized,
    {
        self.impl_ptr().cast()
    }
    fn platformFeatures(&self) -> &gpu::PlatformFeatures {
        RenderContext::platformFeatures(self)
    }
    fn frameDescriptor(&self) -> &FrameDescriptor {
        RenderContext::frameDescriptor(self)
    }
    fn beginFrame(&mut self, descriptor: &FrameDescriptor) {
        self.beginFrameExecutable(descriptor)
    }
    fn isOutsideCurrentFrame(&self, bounds: &IAABB) -> bool {
        self.isOutsideCurrentFrameExecutable(bounds)
    }
    fn frameSupportsClipRects(&self) -> bool {
        self.frameSupportsClipRectsExecutable()
    }
    fn frameSupportsImagePaintForPaths(&self) -> bool {
        self.frameSupportsImagePaintForPathsExecutable()
    }
    fn frameInterlockMode(&self) -> gpu::InterlockMode {
        RenderContext::frameInterlockMode(self)
    }
    fn generateClipID(&mut self, bounds: IAABB, parent: u32, tightened: AABBu16) -> u32 {
        self.generateClipIDExecutable(bounds, parent, tightened)
    }
    fn pushDraws(&mut self, draws: &mut [DrawUniquePtr], count: usize) -> bool {
        unsafe { self.pushDrawsExecutable(draws, count) }
    }
    fn logicalFlush(&mut self) {
        self.logicalFlushExecutable()
    }
    unsafe fn flush(&mut self, resources: &FlushResources) {
        unsafe { self.flushExecutable(resources) }
    }
    fn releaseResources(&mut self) {
        RenderContext::releaseResources(self)
    }
    fn perFrameAllocator(&mut self) -> &mut TrivialBlockAllocator {
        RenderContext::perFrameAllocator(self)
    }
    fn numChopsAllocator(&mut self) -> &mut TrivialArrayAllocator<u8> {
        RenderContext::numChopsAllocator(self)
    }
    fn chopVerticesAllocator(&mut self) -> &mut TrivialArrayAllocator<Vec2D> {
        RenderContext::chopVerticesAllocator(self)
    }
    fn tangentPairsAllocator(&mut self) -> &mut TrivialArrayAllocator<[Vec2D; 2]> {
        RenderContext::tangentPairsAllocator(self)
    }
    fn polarSegmentCountsAllocator(&mut self) -> &mut TrivialArrayAllocator<u32, 16> {
        RenderContext::polarSegmentCountsAllocator(self)
    }
    fn parametricSegmentCountsAllocator(&mut self) -> &mut TrivialArrayAllocator<u32, 16> {
        RenderContext::parametricSegmentCountsAllocator(self)
    }
    #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
    fn makeRenderCanvas(&mut self,width:u32,height:u32)->crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas>{
        self.makeRenderCanvasExecutable(width, height)
    }
    #[cfg(any(feature = "native-ore-metal-experimental", feature = "native-ore-vulkan-experimental"))]
    fn getOreContext(&mut self) -> *mut OreContext {
        self.oreExecutable()
    }
    fn resetContainers(&mut self) {
        RenderContext::resetContainers(self)
    }
    fn setResourceSizes(&mut self, counts: ResourceAllocationCounts, force: bool) {
        RenderContext::setResourceSizes(self, counts, force)
    }
    fn mapResourceBuffers(&mut self, counts: &ResourceAllocationCounts) -> bool {
        unsafe { self.mapResourceBuffersExecutable(counts) }
    }
    fn unmapResourceBuffers(&mut self, counts: &ResourceAllocationCounts) {
        unsafe { self.unmapResourceBuffersExecutable(counts) }
    }
    fn incrementCoverageBufferPrefix(&mut self, clear: &mut bool) -> u32 {
        self.incrementCoverageBufferPrefixExecutable(clear)
    }
}

impl LogicalFlushContract for LogicalFlush {
    unsafe fn new(parent: Pin<&mut RenderContext>) -> Self {
        let ptr = unsafe { Pin::get_unchecked_mut(parent) as *mut RenderContext };
        *unsafe { LogicalFlush::new_box(ptr) }
    }
    fn rewind(&mut self) {
        self.rewindExecutable()
    }
    fn resetContainers(&mut self) {
        LogicalFlush::resetContainers(self)
    }
    fn frameDescriptor(&self) -> &FrameDescriptor {
        LogicalFlush::frameDescriptor(self)
    }
    fn interlockMode(&self) -> gpu::InterlockMode {
        LogicalFlush::interlockMode(self)
    }
    fn platformFeatures(&self) -> &gpu::PlatformFeatures {
        LogicalFlush::platformFeatures(self)
    }
    fn desc(&self) -> &gpu::FlushDescriptor {
        LogicalFlush::desc(self)
    }
    fn generateClipID(&mut self, b: IAABB, p: u32, t: AABBu16) -> u32 {
        self.generateClipIDExecutable(b, p, t)
    }
    fn pushDraws(&mut self, d: &mut [DrawUniquePtr], count: usize) -> bool {
        unsafe { self.pushDrawsExecutable(&mut d[..count]) }
    }
    unsafe fn allocateGradient(
        &mut self,
        g: *const Gradient,
        l: *mut gpu::ColorRampLocation,
    ) -> bool {
        unsafe { self.allocateGradientExecutable(g, l) }
    }
    unsafe fn allocateFeatherAtlasDraw(
        &mut self,
        d: *mut PathDraw,
        w: u16,
        h: u16,
        p: u16,
        x: *mut u16,
        y: *mut u16,
        r: *mut AABBu16,
    ) -> bool {
        unsafe { self.allocateFeatherAtlasDrawExecutable(d, w, h, p, x, y, r) }
    }
    fn allocateCoverageBufferRange(&mut self, length: usize) -> usize {
        self.allocateCoverageBufferRangeExecutable(length)
    }
    unsafe fn layoutResources(
        &mut self,
        r: &FlushResources,
        i: usize,
        rc: *mut ResourceCounters,
        lc: *mut LayoutCounters,
    ) {
        unsafe { self.layoutResourcesExecutable(r, i, &mut *rc, &mut *lc) }
    }
    fn writeResources(&mut self) {
        unsafe { self.writeResourcesExecutable() }
    }
    fn allocateMidpointFanTessVertices(&mut self, c: u32) -> u32 {
        self.allocateMidpointFanTessVerticesExecutable(c)
    }
    fn allocateOuterCubicTessVertices(&mut self, c: u32) -> u32 {
        self.allocateOuterCubicTessVerticesExecutable(c)
    }
    unsafe fn pushPath(&mut self, d: *const PathDraw) -> u32 {
        unsafe { self.pushPathExecutable(d) }
    }
    fn pushContour(&mut self, p: u32, m: Vec2D, s: bool, c: bool, v: u32) -> u32 {
        self.pushContourExecutable(p, m, s, c, v)
    }
    fn pushPaddingVertices(&mut self, c: u32, l: u32) {
        self.pushPaddingVerticesExecutable(c, l)
    }
    fn pushBarriers(&mut self, b: gpu::BarrierFlags) {
        self.pushBarriersExecutable(b)
    }
    unsafe fn pushMidpointFanDraw(
        &mut self,
        d: *const PathDraw,
        t: gpu::DrawType,
        c: u32,
        l: u32,
        m: gpu::ShaderMiscFlags,
    ) -> *mut gpu::DrawBatch {
        unsafe { self.pushMidpointFanDrawExecutable(d, t, c, l, m) }
    }
    unsafe fn pushOuterCubicsDraw(
        &mut self,
        d: *const PathDraw,
        t: gpu::DrawType,
        c: u32,
        l: u32,
        m: gpu::ShaderMiscFlags,
    ) -> *mut gpu::DrawBatch {
        unsafe { self.pushOuterCubicsDrawExecutable(d, t, c, l, m) }
    }
    unsafe fn pushInteriorTriangulationDraw(
        &mut self,
        d: *const PathDraw,
        p: u32,
        w: gpu::WindingFaces,
        #[cfg(debug_assertions)] counter: *mut usize,
    ) -> *mut gpu::DrawBatch {
        unsafe {
            self.pushInteriorTriangulationDrawExecutable(
                d,
                p,
                w,
                self.m_baseline_shader_misc_flags,
                #[cfg(debug_assertions)]
                counter,
            )
        }
    }
    unsafe fn pushFeatherAtlasBlit(&mut self, d: *mut PathDraw, p: u32) -> *mut gpu::DrawBatch {
        unsafe { self.pushFeatherAtlasBlitExecutable(d, p) }
    }
    unsafe fn pushImageRectDraw(&mut self, d: *mut ImageRectDraw) -> *mut gpu::DrawBatch {
        unsafe { self.pushImageRectDrawExecutable(d) }
    }
    unsafe fn pushImageMeshDraw(&mut self, d: *mut ImageMeshDraw) -> *mut gpu::DrawBatch {
        unsafe { self.pushImageMeshDrawExecutable(d) }
    }
    unsafe fn pushClipResetDraw(&mut self, d: *mut ClipReset) -> *mut gpu::DrawBatch {
        unsafe { self.pushClipResetDrawExecutable(d) }
    }
    fn getWritableClipInfo(&mut self, id: u32) -> &mut ClipInfo {
        self.getWritableClipInfoExecutable(id)
    }
    unsafe fn pushPathDraw(
        &mut self,
        d: *const PathDraw,
        t: gpu::DrawType,
        m: gpu::ShaderMiscFlags,
        c: u32,
        b: u32,
    ) -> *mut gpu::DrawBatch {
        unsafe { self.pushPathDrawExecutable(d, t, m, c, b) }
    }
    unsafe fn pushDraw(
        &mut self,
        d: *const Draw,
        t: gpu::DrawType,
        m: gpu::ShaderMiscFlags,
        p: gpu::PaintType,
        c: u32,
        b: u32,
    ) -> *mut gpu::DrawBatch {
        unsafe { self.pushDrawExecutable(d, t, m, p, c, b) }
    }
    fn tightenClipBounds(&mut self) {
        self.tightenClipBoundsExecutable()
    }
    unsafe fn addBatchToDstBarrierList(&mut self, b: *mut gpu::DrawBatch) {
        unsafe { self.addBatchToDstBarrierListExecutable(b) }
    }
}

impl<'a> TessellationWriterContract<'a> for TessellationWriter<'a> {
    fn new(
        flush: &'a mut LogicalFlush,
        path: u32,
        d: gpu::ContourDirections,
        fc: u32,
        fl: u32,
        mc: u32,
        ml: u32,
    ) -> Self {
        unsafe { TessellationWriter::newExecutable(flush, path, d, fc, fl, mc, ml) }
    }
    fn pushContour(&mut self, m: Vec2D, s: bool, c: bool, p: u32) -> u32 {
        self.pushContourExecutable(m, s, c, p)
    }
    fn pushCubic(
        &mut self,
        p: &[Vec2D; 4],
        d: gpu::ContourDirections,
        t: Vec2D,
        a: u32,
        b: u32,
        j: u32,
        c: u32,
    ) {
        self.pushCubicExecutable(p, d, t, a, b, j, c)
    }
    fn pushTessellationSpans(
        &mut self,
        p: &[Vec2D; 4],
        t: Vec2D,
        n: u32,
        a: u32,
        b: u32,
        j: u32,
        c: u32,
    ) {
        self.pushTessellationSpansExecutable(p, t, n, a, b, j, c)
    }
    fn pushMirroredTessellationSpans(
        &mut self,
        p: &[Vec2D; 4],
        t: Vec2D,
        n: u32,
        a: u32,
        b: u32,
        j: u32,
        c: u32,
    ) {
        self.pushMirroredTessellationSpansExecutable(p, t, n, a, b, j, c)
    }
    fn pushDoubleSidedTessellationSpans(
        &mut self,
        p: &[Vec2D; 4],
        t: Vec2D,
        n: u32,
        a: u32,
        b: u32,
        j: u32,
        c: u32,
    ) {
        self.pushDoubleSidedTessellationSpansExecutable(p, t, n, a, b, j, c)
    }
}
