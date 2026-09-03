/*
 * Mechanical translation of the complete pinned source file.
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 * The literal source is retained below in declaration/order form.
 */

// /*
//  * Copyright 2022 Rive
//  */
//
// #include "rive/renderer/rive_renderer.hpp"
//
// #include "rive_render_paint.hpp"
// #include "rive_render_path.hpp"
// #include "rive/math/math_types.hpp"
// #include "rive/math/simd.hpp"
// #include "rive/renderer/rive_render_image.hpp"
// #include "rive/profiler/profiler_macros.h"
//
// namespace rive
// {
// bool RiveRenderer::IsAABB(const RawPath& path, AABB* result)
// {
//     RIVE_PROF_SCOPE_L(3)
//     // Any quadrilateral begins with a move plus 3 lines.
//     constexpr static size_t kAABBVerbCount = 4;
//     constexpr static PathVerb aabbVerbs[kAABBVerbCount] = {PathVerb::move,
//                                                            PathVerb::line,
//                                                            PathVerb::line,
//                                                            PathVerb::line};
//     Span<const PathVerb> verbs = path.verbs();
//     if (verbs.count() < kAABBVerbCount ||
//         memcmp(verbs.data(), aabbVerbs, sizeof(aabbVerbs)) != 0)
//     {
//         return false;
//     }
//
//     // Only accept extra verbs and points if every point after the quadrilateral
//     // is equal to p0.
//     Span<const Vec2D> pts = path.points();
//     for (size_t i = 4; i < pts.count(); ++i)
//     {
//         if (pts[i] != pts[0])
//         {
//             return false;
//         }
//     }
//
//     // We have a quadrilateral! Now check if it is an axis-aligned rectangle.
//     float4 corners = {pts[0].x, pts[0].y, pts[2].x, pts[2].y};
//     float4 oppositeCorners = {pts[1].x, pts[1].y, pts[3].x, pts[3].y};
//     if (simd::all(corners == oppositeCorners.zyxw) ||
//         simd::all(corners == oppositeCorners.xwzy))
//     {
//         float4 r = simd::join(simd::min(corners.xy, corners.zw),
//                               simd::max(corners.xy, corners.zw));
//         simd::store(result, r);
//         return true;
//     }
//     return false;
// }
//
// RiveRenderer::ClipElement::ClipElement(const Mat2D& matrix_,
//                                        const RiveRenderPath* path_,
//                                        FillRule fillRule_,
//                                        IAABB pixelBounds_)
// {
//     reset(matrix_, path_, fillRule_, pixelBounds_);
// }
//
// RiveRenderer::ClipElement::~ClipElement() {}
//
// void RiveRenderer::ClipElement::reset(const Mat2D& matrix_,
//                                       const RiveRenderPath* path_,
//                                       FillRule fillRule_,
//                                       IAABB pixelBounds_)
// {
//     matrix = matrix_;
//     rawPathMutationID = path_->getRawPathMutationID();
//     pathBounds = path_->getBounds();
//     path = ref_rcp(path_);
//     fillRule = fillRule_;
//     clipID = 0; // This gets initialized lazily.
//     pixelBounds = pixelBounds_;
// }
//
// bool RiveRenderer::ClipElement::isEquivalent(const Mat2D& matrix_,
//                                              const RiveRenderPath* path_) const
// {
//     return matrix_ == matrix &&
//            path_->getRawPathMutationID() == rawPathMutationID &&
//            path_->getFillRule() == fillRule;
// }
//
// RiveRenderer::RiveRenderer(gpu::RenderContext* context) : m_context(context) {}
//
// RiveRenderer::~RiveRenderer() {}
//
// void RiveRenderer::save()
// {
//     // Copy the back of the stack before pushing, in case the vector grows and
//     // invalidates the reference.
//     RenderState copy = m_renderStateStack.back();
//     m_renderStateStack.push_back(copy);
// }
//
// void RiveRenderer::restore()
// {
//     assert(m_renderStateStack.size() > 1);
//     assert(m_renderStateStack.back().clipStackHeight >=
//            m_renderStateStack[m_renderStateStack.size() - 2].clipStackHeight);
//     m_renderStateStack.pop_back();
// }
//
// void RiveRenderer::transform(const Mat2D& matrix)
// {
//     m_renderStateStack.back().matrix =
//         m_renderStateStack.back().matrix * matrix;
// }
//
// void RiveRenderer::modulateOpacity(float opacity)
// {
//     m_renderStateStack.back().modulatedOpacity =
//         std::max(0.0f, m_renderStateStack.back().modulatedOpacity * opacity);
// }
//
// void RiveRenderer::drawPath(RenderPath* renderPath, RenderPaint* renderPaint)
// {
//     RIVE_PROF_SCOPE_L(2)
//     LITE_RTTI_CAST_OR_RETURN(path, RiveRenderPath*, renderPath);
//     LITE_RTTI_CAST_OR_RETURN(paint, RiveRenderPaint*, renderPaint);
//
//     if (path->getRawPath().empty())
//     {
//         return;
//     }
//
//     if (paint->getIsStroked() && m_context->frameDescriptor().strokesDisabled)
//     {
//         return;
//     }
//     if (!paint->getIsStroked() && m_context->frameDescriptor().fillsDisabled)
//     {
//         return;
//     }
//     if (paint->getIsStroked() &&
//         // Use inverse logic to ensure we abort when stroke thickness is NaN.
//         !(paint->getThickness() > 0))
//     {
//         return;
//     }
//     // Use inverse logic to ensure we abort when stroke thickness is NaN.
//     if (!(paint->getFeather() >= 0))
//     {
//         return;
//     }
//     if (m_renderStateStack.back().overallClipPixelBounds.empty())
//     {
//         return;
//     }
//
//     Mat2D imageMatrix;
//     Mat2D* imageMatrixPtr = nullptr;
//     if (paint->getImageTexture() != nullptr)
//     {
//         imageMatrix =
//             m_renderStateStack.back().matrix * paint->getImageTransform();
//         imageMatrixPtr = &imageMatrix;
//     }
//
//     if (paint->getFeather() != 0 && !paint->getIsStroked())
//     {
//         if (path->getFillRule() != FillRule::clockwise &&
//             !m_context->frameDescriptor().clockwiseFillOverride)
//         {
//             // Don't draw feathered fills that aren't clockwise.
//             return;
//         }
//         float matrixMaxScale = m_renderStateStack.back().matrix.findMaxScale();
//         if (paint->getFeather() * matrixMaxScale > 1)
//         {
//             clipAndPushDraw(gpu::PathDraw::Make(
//                 m_context,
//                 m_renderStateStack.back().matrix,
//                 imageMatrixPtr,
//                 path->makeSoftenedCopyForFeathering(paint->getFeather(),
//                                                     matrixMaxScale),
//                 path->getFillRule(),
//                 paint,
//                 m_renderStateStack.back().modulatedOpacity));
//             return;
//         }
//     }
//
//     clipAndPushDraw(
//         gpu::PathDraw::Make(m_context,
//                             m_renderStateStack.back().matrix,
//                             imageMatrixPtr,
//                             ref_rcp(path),
//                             path->getFillRule(),
//                             paint,
//                             m_renderStateStack.back().modulatedOpacity));
// }
//
// void RiveRenderer::clipPath(RenderPath* renderPath)
// {
//     RIVE_PROF_SCOPE_L(2)
//     LITE_RTTI_CAST_OR_RETURN(path, RiveRenderPath*, renderPath);
//
//     if (m_renderStateStack.back().overallClipPixelBounds.empty())
//     {
//         return;
//     }
//
//     if (path->getRawPath().empty())
//     {
//         m_renderStateStack.back().overallClipPixelBounds = {};
//         return;
//     }
//
//     // First try to handle axis-aligned rectangles using the "ENABLE_CLIP_RECT"
//     // shader feature. Multiple axis-aligned rectangles can be intersected into
//     // a single rectangle if their matrices are compatible.
//     AABB clipRectCandidate;
//     if (m_context->frameSupportsClipRects() &&
//         IsAABB(path->getRawPath(), &clipRectCandidate))
//     {
//         clipRectImpl(clipRectCandidate, path);
//     }
//     else
//     {
//         clipPathImpl(path);
//     }
// }
//
// // Finds a new rect, if such a rect exists, such that:
// //
// //     currentMatrix * rect == newMatrix * newRect
// //
// // Returns true if *rect was replaced with newRect.
// static bool transform_rect_to_new_space(AABB* rect,
//                                         const Mat2D& currentMatrix,
//                                         const Mat2D& newMatrix)
// {
//     if (currentMatrix == newMatrix)
//     {
//         return true;
//     }
//     Mat2D currentToNew;
//     if (!newMatrix.invert(&currentToNew))
//     {
//         return false;
//     }
//     currentToNew = currentToNew * currentMatrix;
//     float maxSkew = fmaxf(fabsf(currentToNew.xy()), fabsf(currentToNew.yx()));
//     float maxScale = fmaxf(fabsf(currentToNew.xx()), fabsf(currentToNew.yy()));
//     if (maxSkew > math::EPSILON && maxScale > math::EPSILON)
//     {
//         // Transforming this rect to the new view matrix would turn it into
//         // something that isn't a rect.
//         return false;
//     }
//     Vec2D pts[2] = {{rect->left(), rect->top()},
//                     {rect->right(), rect->bottom()}};
//     currentToNew.mapPoints(pts, pts, 2);
//     float4 p = simd::load4f(pts);
//     float2 topLeft = simd::min(p.xy, p.zw);
//     float2 botRight = simd::max(p.xy, p.zw);
//     *rect = {topLeft.x, topLeft.y, botRight.x, botRight.y};
//     return true;
// }
//
// void RiveRenderer::clipRectImpl(AABB rect, const RiveRenderPath* originalPath)
// {
//     RIVE_PROF_SCOPE_L(3)
//
//     auto& renderState = m_renderStateStack.back();
//     bool hasClipRect = renderState.clipRectInverseMatrix != nullptr;
//     if (rect.isEmptyOrNaN())
//     {
//         renderState.overallClipPixelBounds = {};
//         return;
//     }
//
//     // If there already is a clipRect, we can only accept another one by
//     // intersecting it with the existing one. This means the new rect must be
//     // axis-aligned with the existing clipRect.
//     if (hasClipRect && !transform_rect_to_new_space(&rect,
//                                                     renderState.matrix,
//                                                     renderState.clipRectMatrix))
//     {
//         // 'rect' is not axis-aligned with the existing clipRect. Fall back to
//         // clipPath.
//         clipPathImpl(originalPath);
//         return;
//     }
//
//     if (!hasClipRect)
//     {
//         // There wasn't an existing clipRect. This is the one!
//         renderState.clipRect = rect;
//         renderState.clipRectMatrix = renderState.matrix;
//     }
//     else
//     {
//         // Both rects are in the same space now. Intersect the two
//         // geometrically.
//         float4 a = simd::load4f(&renderState.clipRect);
//         float4 b = simd::load4f(&rect);
//         float4 intersection =
//             simd::join(simd::max(a.xy, b.xy), simd::min(a.zw, b.zw));
//         simd::store(&renderState.clipRect, intersection);
//     }
//
//     // Grab the pixel bounds of the new combined (intersected) clip rect
//     renderState.clipRectPixelBounds =
//         renderState.clipRectMatrix.mapBoundingBox(renderState.clipRect)
//             .roundOut();
//
//     renderState.overallClipPixelBounds =
//         renderState.overallClipPixelBounds.intersect(
//             renderState.clipRectPixelBounds);
//
//     renderState.clipRectInverseMatrix =
//         m_context->make<gpu::ClipRectInverseMatrix>(renderState.clipRectMatrix,
//                                                     renderState.clipRect);
// }
//
// void RiveRenderer::clipPathImpl(const RiveRenderPath* path)
// {
//     RIVE_PROF_SCOPE_L(3)
//     auto& renderState = m_renderStateStack.back();
//     if (path->getBounds().isEmptyOrNaN())
//     {
//         renderState.overallClipPixelBounds = {};
//         return;
//     }
//     // Only write a new clip element if this path isn't already on the stack
//     // from before. e.g.:
//     //
//     //     clipPath(samePath);
//     //     restore();
//     //     save();
//     //     clipPath(samePath); // <-- reuse the ClipElement (and clipID!)
//     //     already in m_clipStack.
//     //
//     const size_t clipStackHeight = renderState.clipStackHeight;
//     assert(m_clipStack.size() >= clipStackHeight);
//     if (m_clipStack.size() == clipStackHeight ||
//         !m_clipStack[clipStackHeight].isEquivalent(renderState.matrix, path))
//     {
//         // Calculate the pixel bounds for this clip path before we push it into
//         // the stack to ensure that we even need to do so
//         const auto pixelBounds =
//             renderState.matrix.mapBoundingBox(path->getRawPath().points())
//                 .roundOut();
//         renderState.overallClipPixelBounds =
//             renderState.overallClipPixelBounds.intersect(pixelBounds);
//         if (renderState.overallClipPixelBounds.empty())
//         {
//             // Nothing can draw under this, so no need to add to the stack.
//             return;
//         }
//
//         m_clipStack.resize(clipStackHeight);
//         m_clipStack.emplace_back(renderState.matrix,
//                                  path,
//                                  path->getFillRule(),
//                                  pixelBounds);
//     }
//     else
//     {
//         // We are going to reuse the element that is already in the clip stack,
//         // but need to re-update the overall clip pixel bounds.
//         renderState.overallClipPixelBounds =
//             renderState.overallClipPixelBounds.intersect(
//                 m_clipStack[clipStackHeight].pixelBounds);
//         if (renderState.overallClipPixelBounds.empty())
//         {
//             // Nothing can draw under this, so no need to increment the stack
//             // height.
//             return;
//         }
//     }
//
//     renderState.clipStackHeight = clipStackHeight + 1;
// }
//
// void RiveRenderer::drawImage(const RenderImage* renderImage,
//                              ImageSampler imageSampler,
//                              BlendMode blendMode,
//                              float opacity)
// {
//     RIVE_PROF_SCOPE_L(2)
//     LITE_RTTI_CAST_OR_RETURN(image, const RiveRenderImage*, renderImage);
//
//     rcp<gpu::Texture> imageTexture = image->refTexture();
//     if (imageTexture == nullptr)
//     {
//         // imageTexture may be null if the backend uses a custom factory and/or
//         // updates out-of-band assets asynchronously. If there's no texture yet,
//         // just don't draw anything.
//         return;
//     }
//
//     // Apply modulated opacity (clamp to prevent negative values)
//     float finalOpacity =
//         std::max(0.0f, opacity * m_renderStateStack.back().modulatedOpacity);
//
//     // Scale the view matrix so we can draw this image as the rect [0, 0, 1, 1].
//     save();
//     scale(image->width(), image->height());
//
//     if (!m_context->frameSupportsImagePaintForPaths())
//     {
//         // Fall back on ImageRectDraw if the current frame doesn't support
//         // drawing paths with image paints.
//         if (!m_renderStateStack.back().overallClipPixelBounds.empty())
//         {
//             const Mat2D& m = m_renderStateStack.back().matrix;
//             clipAndPushDraw(
//                 gpu::DrawUniquePtr(m_context->make<gpu::ImageRectDraw>(
//                     m_context,
//                     m.mapBoundingBox(AABB{0, 0, 1, 1}).roundOut(),
//                     m,
//                     blendMode,
//                     std::move(imageTexture),
//                     imageSampler,
//                     finalOpacity)));
//         }
//     }
//     else
//     {
//         // Implement drawImage() as drawPath() with a rectangular path and an
//         // image paint.
//         if (m_unitRectPath == nullptr)
//         {
//             m_unitRectPath = make_rcp<RiveRenderPath>();
//             m_unitRectPath->line({1, 0});
//             m_unitRectPath->line({1, 1});
//             m_unitRectPath->line({0, 1});
//         }
//
//         RiveRenderPaint paint;
//         paint.image(std::move(imageTexture), finalOpacity);
//         paint.blendMode(blendMode);
//         paint.imageSampler(imageSampler);
//         drawPath(m_unitRectPath.get(), &paint);
//     }
//
//     restore();
// }
//
// void RiveRenderer::drawImageMesh(const RenderImage* renderImage,
//                                  ImageSampler imageSampler,
//                                  rcp<RenderBuffer> vertices_f32,
//                                  rcp<RenderBuffer> uvCoords_f32,
//                                  rcp<RenderBuffer> indices_u16,
//                                  uint32_t vertexCount,
//                                  uint32_t indexCount,
//                                  BlendMode blendMode,
//                                  float opacity)
// {
//     RIVE_PROF_SCOPE_L(2)
//     LITE_RTTI_CAST_OR_RETURN(image, const RiveRenderImage*, renderImage);
//
//     rcp<gpu::Texture> imageTexture = image->refTexture();
//     if (imageTexture == nullptr)
//     {
//         // imageTexture may be null if the backend uses a custom factory and/or
//         // updates out-of-band assets asynchronously. If there's no texture yet,
//         // just don't draw anything.
//         return;
//     }
//
//     assert(vertices_f32);
//     assert(uvCoords_f32);
//     assert(indices_u16);
//
//     if (m_renderStateStack.back().overallClipPixelBounds.empty())
//     {
//         return;
//     }
//
//     // Apply modulated opacity (clamp to prevent negative values)
//     float finalOpacity =
//         std::max(0.0f, opacity * m_renderStateStack.back().modulatedOpacity);
//
//     clipAndPushDraw(gpu::DrawUniquePtr(
//         m_context->make<gpu::ImageMeshDraw>(gpu::Draw::FULLSCREEN_PIXEL_BOUNDS,
//                                             m_renderStateStack.back().matrix,
//                                             blendMode,
//                                             std::move(imageTexture),
//                                             imageSampler,
//                                             std::move(vertices_f32),
//                                             std::move(uvCoords_f32),
//                                             std::move(indices_u16),
//                                             indexCount,
//                                             finalOpacity)));
// }
//
// void RiveRenderer::clipAndPushDraw(gpu::DrawUniquePtr draw)
// {
//     RIVE_PROF_SCOPE_L(3)
//     assert(!m_renderStateStack.back().overallClipPixelBounds.empty());
//     if (draw.get() == nullptr)
//     {
//         return;
//     }
//     if (m_context->isOutsideCurrentFrame(draw->pixelBounds()))
//     {
//         return;
//     }
//
//     // Make two attempts to issue the draw: once on the context as-is and once
//     // with a clean flush.
//     for (int i = 0; i < 2; ++i)
//     {
//         // Always make sure we begin this loop with the internal draw batch
//         // empty, and clear it when we're done.
//         struct AutoResetInternalDrawBatch
//         {
//         public:
//             AutoResetInternalDrawBatch(RiveRenderer* renderer) :
//                 m_renderer(renderer)
//             {
//                 assert(m_renderer->m_internalDrawBatch.empty());
//             }
//             ~AutoResetInternalDrawBatch()
//             {
//                 m_renderer->m_internalDrawBatch.clear();
//             }
//
//         private:
//             RiveRenderer* m_renderer;
//         };
//
//         AutoResetInternalDrawBatch aridb(this);
//
//         auto applyClipResult = applyClip(draw.get());
//         if (applyClipResult == ApplyClipResult::failure)
//         {
//             // There wasn't room in the GPU buffers for this path draw. Flush
//             // and try again.
//             m_context->logicalFlush();
//             continue;
//         }
//         else if (applyClipResult == ApplyClipResult::fullyClipped)
//         {
//             return;
//         }
//
//         m_internalDrawBatch.push_back(std::move(draw));
//         if (!m_context->pushDraws(m_internalDrawBatch.data(),
//                                   m_internalDrawBatch.size()))
//         {
//             // There wasn't room in the GPU buffers for this path draw. Flush
//             // and try again.
//             m_context->logicalFlush();
//             // Reclaim "draw" because we will use it again on the next
//             // iteration.
//             draw = std::move(m_internalDrawBatch.back());
//             assert(draw != nullptr);
//             m_internalDrawBatch.pop_back();
//             continue;
//         }
//
//         // Success!
//         return;
//     }
//
//     // We failed to process the draw. Release its refs.
//     fprintf(stderr,
//             "RiveRenderer::clipAndPushDraw failed. The draw and/or clip stack "
//             "are too complex.\n");
// }
//
// // Used by clipping in clockwiseAtomic mode.
// //
// // Returns the inverse of a path, meaning, regions that were filled in the old
// // path are now empty, and empty regions in the old path are now filled.
// //
// // NOTE: A true inverse path would expand infinitely in all directions, but this
// // function limits it by the provided "bounds".
// //
// // NOTE: The returned path is always clockwise, even if the given path was not.
// // If the given path is not clockwise, we attempt to convert it to clockwise
// // based on its dominant winding direction. This may or may not be accurate.
// static rcp<RiveRenderPath> invertClockwisePath(const RiveRenderPath* path,
//                                                FillRule pathFillRule,
//                                                const Mat2D& viewMatrix,
//                                                IAABB bounds)
// {
//     auto inversePath = make_rcp<RiveRenderPath>();
//     inversePath->fillRule(FillRule::clockwise);
//     Mat2D viewInverseMatrix;
//     if (viewMatrix.invert(&viewInverseMatrix))
//     {
//         // Add the pre-viewMatrix "bounds" rect to the new path.
//         std::array<Vec2D, 4> boundsVertices = {
//             Vec2D(bounds.left, bounds.top),
//             Vec2D(bounds.right, bounds.top),
//             Vec2D(bounds.right, bounds.bottom),
//             Vec2D(bounds.left, bounds.bottom),
//         };
//         viewInverseMatrix.mapPoints(boundsVertices.data(),
//                                     boundsVertices.data(),
//                                     4);
//         inversePath->move(boundsVertices[0]);
//         if (const float viewMatrixDeterminant =
//                 viewMatrix[0] * viewMatrix[3] - viewMatrix[2] * viewMatrix[1];
//             viewMatrixDeterminant >= 0)
//         {
//             inversePath->line(boundsVertices[1]);
//             inversePath->line(boundsVertices[2]);
//             inversePath->line(boundsVertices[3]);
//         }
//         else
//         {
//             inversePath->line(boundsVertices[3]);
//             inversePath->line(boundsVertices[2]);
//             inversePath->line(boundsVertices[1]);
//         }
//         // Subtract the given path out of the bounds rect.
//         if (pathFillRule == FillRule::clockwise || path->getCoarseArea() >= 0)
//         {
//             inversePath->addRenderPathBackwards(path, Mat2D());
//         }
//         else
//         {
//             inversePath->addRenderPath(path, Mat2D());
//         }
//     }
//     return inversePath;
// }
//
// RiveRenderer::ApplyClipResult RiveRenderer::applyClip(gpu::Draw* draw)
// {
//     RIVE_PROF_SCOPE_L(3)
//     auto& renderState = m_renderStateStack.back();
//
//     draw->setClipRect(renderState.clipRectInverseMatrix,
//                       renderState.overallClipPixelBounds);
//
//     if (draw->clippedPixelBounds().empty() ||
//         m_context->isOutsideCurrentFrame(draw->clippedPixelBounds()))
//     {
//         // Either this draw is outside of the current frame (in which case it's
//         // not visible) or it was completely clipped by the current overall clip
//         // bounds.
//         return ApplyClipResult::fullyClipped;
//     }
//
//     const size_t clipStackHeight = renderState.clipStackHeight;
//     if (clipStackHeight == 0)
//     {
//         assert(draw->clipID() == 0);
//         return ApplyClipResult::success;
//     }
//
//     // Find which clip element in the stack (if any) is currently rendered to
//     // the clip buffer.
//     size_t clipIdxCurrentlyInClipBuffer = -1; // i.e., "none".
//     if (m_context->getClipContentID() != 0)
//     {
//         for (size_t i = clipStackHeight - 1; i != -1; --i)
//         {
//             if (m_clipStack[i].clipID == m_context->getClipContentID())
//             {
//                 clipIdxCurrentlyInClipBuffer = i;
//                 break;
//             }
//         }
//     }
//
//     // Draw the necessary updates to the clip buffer (i.e., draw every clip
//     // element after clipIdxCurrentlyInClipBuffer).
//     uint32_t parentClipID =
//         clipIdxCurrentlyInClipBuffer == -1
//             ? 0 // The next clip to be drawn is not nested.
//             : m_clipStack[clipIdxCurrentlyInClipBuffer].clipID;
//     if (m_context->frameInterlockMode() ==
//             gpu::InterlockMode::clockwiseAtomic ||
//         m_context->frameInterlockMode() == gpu::InterlockMode::msaa)
//     {
//         if (parentClipID == 0 && m_context->getClipContentID() != 0)
//         {
//             // Time for a new stencil clip! Erase the clip currently in the
//             // stencil buffer before we draw the new one.
//             auto stencilClipClear =
//                 gpu::DrawUniquePtr(m_context->make<gpu::ClipReset>(
//                     m_context,
//                     m_context->getClipContentID(),
//                     gpu::DrawContents::none,
//                     gpu::ClipReset::ResetAction::clearPreviousClip));
//             if (!m_context->isOutsideCurrentFrame(
//                     stencilClipClear->pixelBounds()))
//             {
//                 m_internalDrawBatch.push_back(std::move(stencilClipClear));
//             }
//         }
//     }
//
//     for (size_t i = clipIdxCurrentlyInClipBuffer + 1; i < clipStackHeight; ++i)
//     {
//         ClipElement& clip = m_clipStack[i];
//         assert(clip.pathBounds == clip.path->getBounds());
//
//         IAABB clipDrawBounds;
//         RiveRenderPaint clipUpdatePaint;
//         clipUpdatePaint.clipUpdate(
//             /*clip THIS clipDraw against:*/ parentClipID);
//
//         rcp clipPath = clip.path;
//         FillRule clipFillRule = clip.fillRule;
//         std::optional pixelBounds = clip.pixelBounds;
//
//         if (m_context->frameInterlockMode() ==
//                 gpu::InterlockMode::clockwiseAtomic &&
//             parentClipID != 0)
//         {
//             // clockwiseAtomic implements nested clips by erasing the inverse
//             // of the inner path from the outer clip.
//             clipPath = invertClockwisePath(
//                 clipPath.get(),
//                 clipFillRule,
//                 clip.matrix,
//                 m_context->getClipContentBounds(parentClipID));
//             clipFillRule = FillRule::clockwise;
//
//             // Clear this because the inverted path has different pixel bounds
//             // (as it now contains the clip content bounds as a box around the
//             // path)
//             pixelBounds = std::nullopt;
//         }
//
//         gpu::DrawUniquePtr clipDraw =
//             gpu::PathDraw::Make(m_context,
//                                 clip.matrix,
//                                 nullptr, // imageMatrix, unneeded for clips
//                                 std::move(clipPath),
//                                 clipFillRule,
//                                 &clipUpdatePaint,
//                                 1.0f, // no opacity modulation for clips
//                                 pixelBounds);
//
//         // We have already validated that the clip path is within the screen
//         //  bounds, so this should never return null (which is the "this is
//         //  offscreen" result).
//         assert(clipDraw != nullptr);
//
//         clipDrawBounds = clipDraw->pixelBounds();
//
//         // Generate a new clipID every time we (re-)render an element to the
//         // clip buffer. (Each embodiment of the element needs its own
//         // separate readBounds.)
//         {
//             // if we have a parent, use its current adjusted write bounds as its
//             // outer bounds, otherwise limit it to the screen area.
//             // TODO: This should take into account any clip rect that might be
//             // applied.
//             const auto outerBounds =
//                 (parentClipID != 0)
//                     ? m_context->getTightenedClipBounds(parentClipID)
//                     : AABBu16::MakeWH(
//                           m_context->frameDescriptor().renderTargetWidth,
//                           m_context->frameDescriptor().renderTargetHeight);
//
//             // Trim the draw bounds to the outer bounds as the initial minimal
//             // clip bounds (we shouldn't need to write to or read from anywhere
//             // that is outside of the screen or a parent clip's box, if one
//             // exists).
//             const auto tightenedBounds = outerBounds.intersect(clipDrawBounds);
//
//             // If there is a parent clip, the next element up the clip stack
//             // should have its ID.
//             assert(parentClipID == 0 ||
//                    (i != 0 && m_clipStack[i - 1].clipID == parentClipID));
//
//             clip.clipID = m_context->generateClipID(clipDrawBounds,
//                                                     parentClipID,
//                                                     tightenedBounds);
//         }
//         assert(clip.clipID != m_context->getClipContentID());
//         if (clip.clipID == 0)
//         {
//             return ApplyClipResult::failure; // The context is out of
//                                              // clipIDs. We will flush and
//                                              // try again.
//         }
//         clipDraw->setClipID(clip.clipID);
//
//         gpu::DrawContents clipDrawContents = clipDraw->drawContents();
//         if (!m_context->isOutsideCurrentFrame(clipDrawBounds))
//         {
//             m_internalDrawBatch.push_back(std::move(clipDraw));
//         }
//
//         if (parentClipID != 0)
//         {
//             if (m_context->frameInterlockMode() == gpu::InterlockMode::msaa)
//             {
//                 // When drawing nested stencil clips, we need to intersect them,
//                 // which involves erasing the region of the current clip in the
//                 // stencil buffer that is outside the the one we just drew.
//                 auto stencilClipIntersect =
//                     gpu::DrawUniquePtr(m_context->make<gpu::ClipReset>(
//                         m_context,
//                         parentClipID,
//                         clipDrawContents,
//                         gpu::ClipReset::ResetAction::intersectPreviousClip));
//                 if (!m_context->isOutsideCurrentFrame(
//                         stencilClipIntersect->pixelBounds()))
//                 {
//                     m_internalDrawBatch.push_back(
//                         std::move(stencilClipIntersect));
//                 }
//             }
//         }
//
//         parentClipID = clip.clipID; // Nest the next clip (if any) inside the
//                                     // one we just rendered.
//     }
//
//     assert(parentClipID == m_clipStack[clipStackHeight - 1].clipID);
//     draw->setClipID(parentClipID);
//     m_context->setClipContentID(parentClipID);
//
//     return ApplyClipResult::success;
// }
// } // namespace rive
//

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use super::rive_render_paint_hpp::RiveRenderPaint;
use super::rive_render_path_hpp::RiveRenderPath;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{make_rcp, rcp, ref_rcp};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderImage, RenderPaint, RenderPath, RendererContract,
};
use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler;
use crate::mechanical_port::source::renderer::include::rive::renderer::draw_hpp::RiveRenderPaintContract;
use crate::mechanical_port::source::renderer::include::rive::renderer::draw_hpp::FULLSCREEN_PIXEL_BOUNDS;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    AABBu16, DrawUniquePtr, RenderContext,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImage;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_renderer_hpp::{
    ClipElement, RenderState, RiveRenderer,
};
use crate::mechanical_port::source::renderer::src::draw_cpp::{
    make_clip_reset, make_image_mesh_draw, make_image_rect_draw, make_path_draw_from_source,
};
use nuxie_render_api::{
    Aabb, BlendMode, FillRule, Mat2D, RawPath, RenderPath as ApiRenderPath, Vec2D,
};

mod gpu {
    pub use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::*;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyClipResult {
    success,
    failure,
    fullyClipped,
}

fn mul(a: Mat2D, b: Mat2D) -> Mat2D {
    let [a0, a1, a2, a3, a4, a5] = a.0;
    let [b0, b1, b2, b3, b4, b5] = b.0;
    Mat2D([
        a0.mul_add(b0, a2 * b1),
        a1.mul_add(b0, a3 * b1),
        a0.mul_add(b2, a2 * b3),
        a1.mul_add(b2, a3 * b3),
        a0.mul_add(b4, a2 * b5) + a4,
        a1.mul_add(b4, a3 * b5) + a5,
    ])
}

fn determinant(m: Mat2D) -> f32 {
    let [a, b, c, d, _, _] = m.0;
    a.mul_add(d, -(c * b))
}

fn invert(m: Mat2D) -> Option<Mat2D> {
    let [a, b, c, d, tx, ty] = m.0;
    let det = determinant(m);
    if det == 0.0 {
        return None;
    }
    let inv = 1.0 / det;
    Some(Mat2D([
        d * inv,
        -b * inv,
        -c * inv,
        a * inv,
        c.mul_add(ty, -(d * tx)) * inv,
        b.mul_add(tx, -(a * ty)) * inv,
    ]))
}

#[cfg(test)]
mod renderer_mat2d_owner_tests {
    use super::{Mat2D, RendererContract, RiveRenderer, determinant, invert, max_scale, mul};

    fn from_bits(bits: [u32; 6]) -> Mat2D {
        Mat2D(bits.map(f32::from_bits))
    }

    fn bits(matrix: Mat2D) -> [u32; 6] {
        matrix.0.map(f32::to_bits)
    }

    #[test]
    fn renderer_inverse_preserves_pinned_finite_cancellation_and_nonfinite_determinants() {
        let cancellation = from_bits([
            0x26cd_29b3,
            0x2533_fdc2,
            0xd01a_d4bb,
            0xce87_d5a9,
            0,
            0,
        ]);
        assert_eq!(determinant(cancellation).to_bits(), 0xa7ee_c560);
        assert_eq!(
            bits(invert(cancellation).expect("pinned finite determinant is nonzero")),
            [
                0x6611_a2d3,
                0x3cc0_fa97,
                0xe7a6_00cd,
                0xbe5b_f782,
                0x8000_0000,
                0x8000_0000,
            ],
        );

        let max_diagonal = Mat2D([f32::MAX, 0.0, 0.0, f32::MAX, 0.0, 0.0]);
        assert_eq!(determinant(max_diagonal), f32::INFINITY);
        assert_eq!(
            bits(invert(max_diagonal).expect("pinned invert accepts an infinite determinant")),
            [
                0x0000_0000,
                0x8000_0000,
                0x8000_0000,
                0x0000_0000,
                0x0000_0000,
                0x0000_0000,
            ],
        );

        let nan_determinant = Mat2D([f32::NAN, 0.0, 0.0, 1.0, 0.0, 0.0]);
        assert!(invert(nan_determinant).is_some());
    }

    #[test]
    fn renderer_transform_concatenates_with_pinned_mat2d_multiply() {
        let current = from_bits([
            0x9422_bf8a,
            0x9788_280a,
            0xd2ec_7e6e,
            0x4d52_6674,
            0xe887_c79b,
            0x4bce_95e3,
        ]);
        let next = from_bits([
            0xb12b_6d28,
            0x2f8c_b036,
            0xdeb1_8044,
            0x4f30_2db7,
            0x155f_c859,
            0x4858_db48,
        ]);
        let expected = [
            0xc301_f7ed,
            0x3d67_41b5,
            0xe2a2_c127,
            0x5d10_cc02,
            0xe887_c79b,
            0x5632_3ab1,
        ];
        assert_eq!(bits(mul(current, next)), expected);

        // This consumer is the mechanically translated
        // `RiveRenderer::transform`, not a parallel test-only calculation.
        let mut renderer = unsafe { RiveRenderer::new(core::ptr::null_mut()) };
        renderer.current_state_mut().matrix = current;
        RendererContract::transform(&mut renderer, &next);
        assert_eq!(bits(renderer.current_state().matrix), expected);
    }

    #[test]
    fn feather_path_uses_pinned_find_max_scale_bits() {
        let matrix = from_bits([
            0xc32d_8148,
            0xc2d1_a0c5,
            0x42d9_3be7,
            0x4345_c7ae,
            0,
            0,
        ]);
        let matrix_max_scale = max_scale(matrix);
        assert_eq!(matrix_max_scale.to_bits(), 0x4392_8724);
        assert!(100.0 * matrix_max_scale > 1.0);
    }
}
fn max_scale(m: Mat2D) -> f32 {
    let [xx, xy, yx, yy, _, _] = m.0;
    if xy == 0.0 && yx == 0.0 {
        let x = xx.abs();
        let y = yy.abs();
        return if x < y { y } else { x };
    }
    let a = xx.mul_add(xx, xy * xy);
    let b = xx.mul_add(yx, yy * xy);
    let c = yx.mul_add(yx, yy * yy);
    let b_squared = b * b;
    let mut result = if b_squared <= MATH_EPSILON * MATH_EPSILON {
        a.max(c)
    } else {
        let a_minus_c = a - c;
        (a + c) * 0.5
            + a_minus_c
                .mul_add(a_minus_c, 4.0 * b_squared)
                .sqrt()
                * 0.5
    };
    if !result.is_finite() {
        result = 0.0;
    }
    result.max(0.0).sqrt()
}
const MATH_EPSILON: f32 = 1.0 / 4096.0;
fn own_path(
    mut owner: Box<crate::mechanical_port::source::renderer::src::draw_cpp::PathDrawAllocation>,
) -> DrawUniquePtr {
    let draw = owner.draw_ptr();
    unsafe { DrawUniquePtr::from_owner(draw, owner) }
}
fn own_image_rect(
    mut owner: Box<
        crate::mechanical_port::source::renderer::src::draw_cpp::ImageRectDrawAllocation,
    >,
) -> DrawUniquePtr {
    let draw = owner.draw_ptr();
    unsafe { DrawUniquePtr::from_owner(draw, owner) }
}
fn own_image_mesh(
    mut owner: Box<
        crate::mechanical_port::source::renderer::src::draw_cpp::ImageMeshDrawAllocation,
    >,
) -> DrawUniquePtr {
    let draw = owner.draw_ptr();
    unsafe { DrawUniquePtr::from_owner(draw, owner) }
}
fn own_clip(
    mut owner: Box<crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::ClipReset>,
) -> DrawUniquePtr {
    let draw = core::ptr::addr_of_mut!(owner.base);
    unsafe { DrawUniquePtr::from_owner(draw, owner) }
}
fn invert_clockwise_path(
    path: &RiveRenderPath,
    fill_rule: FillRule,
    matrix: Mat2D,
    bounds: gpu::IAABB,
) -> rcp<RiveRenderPath> {
    let inverse = make_rcp(RiveRenderPath::default);
    let owner = unsafe { &mut *inverse.get() };
    owner.m_fillRule = FillRule::Clockwise;
    if let Some(inv) = invert(matrix) {
        let mut corners = [
            Vec2D::new(bounds.left as f32, bounds.top as f32),
            Vec2D::new(bounds.right as f32, bounds.top as f32),
            Vec2D::new(bounds.right as f32, bounds.bottom as f32),
            Vec2D::new(bounds.left as f32, bounds.bottom as f32),
        ];
        inv.map_points_in_place(&mut corners);
        owner.move_to(corners[0].x, corners[0].y);
        let det = determinant(matrix);
        let order = if det >= 0.0 { [1, 2, 3] } else { [3, 2, 1] };
        for i in order {
            owner.line_to(corners[i].x, corners[i].y);
        }
        if fill_rule == FillRule::Clockwise || path.getCoarseArea() >= 0.0 {
            owner.addRenderPathBackwardsSource(path, Mat2D::IDENTITY)
        } else {
            owner.addRenderPathSource(path, Mat2D::IDENTITY)
        }
    }
    inverse
}

#[cfg(test)]
mod map_points_caller_tests {
    use super::{
        FillRule, Mat2D, RiveRenderPath, determinant, gpu, invert_clockwise_path,
    };

    #[test]
    fn inverse_clockwise_path_uses_pinned_four_point_in_place_batch() {
        let view_matrix = std::hint::black_box(Mat2D([
            f32::from_bits(0xbf80_0000),
            f32::from_bits(0x8000_0000),
            f32::from_bits(0x0000_0000),
            f32::from_bits(0x337f_fffe),
            f32::from_bits(0x3f00_0000),
            f32::from_bits(0x3f80_0001),
        ]));
        let path = RiveRenderPath::default();
        let inverse = invert_clockwise_path(
            &path,
            FillRule::Clockwise,
            view_matrix,
            gpu::IAABB::new(-1, 1, 16_777_217, 16_777_215),
        );
        let points = unsafe { (&*inverse.get()).getRawPath().points() };

        // The negative determinant orders corner 2 at path point 2. Pinned
        // mapPoints' scale/translation FMLA rounds its y lane to 0x57800000;
        // the former scalar transform_point substitute produced 0x577fffff.
        assert_eq!(points[2].y.to_bits(), 0x5780_0000);
    }

    #[test]
    fn inverse_clockwise_path_uses_pinned_contracted_winding_determinant() {
        let view_matrix = Mat2D(
            [
                0x26cd_29b3,
                0x2533_fdc2,
                0xd01a_d4bb,
                0xce87_d5a9,
                0,
                0,
            ]
            .map(f32::from_bits),
        );
        assert_eq!(determinant(view_matrix).to_bits(), 0xa7ee_c560);

        let path = RiveRenderPath::default();
        let inverse = invert_clockwise_path(
            &path,
            FillRule::Clockwise,
            view_matrix,
            gpu::IAABB::new(-1, -2, 3, 4),
        );
        let points = unsafe { (&*inverse.get()).getRawPath().points() };

        // Pinned determinant is negative, so corner 3 follows corner 0.
        // The former uncontracted determinant rounded to +0 and selected
        // corner 1 (and its inverse rejected this matrix outright).
        assert_eq!(points[1].x.to_bits(), 0xe8aa_8de4);
        assert_eq!(points[1].y.to_bits(), 0xbf61_ff57);
    }
}

fn simd_min(first: f32, second: f32) -> f32 {
    if first.is_nan() {
        second
    } else if second.is_nan() {
        first
    } else if first == 0.0 && second == 0.0 {
        f32::from_bits(first.to_bits() | second.to_bits())
    } else if second < first {
        second
    } else {
        first
    }
}

fn simd_max(first: f32, second: f32) -> f32 {
    if first.is_nan() {
        second
    } else if second.is_nan() {
        first
    } else if first == 0.0 && second == 0.0 {
        f32::from_bits(first.to_bits() & second.to_bits())
    } else if first < second {
        second
    } else {
        first
    }
}

fn transform_rect_to_new_space(
    rect: &mut Aabb,
    current_matrix: Mat2D,
    new_matrix: Mat2D,
) -> bool {
    if current_matrix == new_matrix {
        return true;
    }
    let Some(mut current_to_new) = invert(new_matrix) else {
        return false;
    };
    current_to_new = mul(current_to_new, current_matrix);
    let max_skew = current_to_new.0[2]
        .abs()
        .max(current_to_new.0[1].abs());
    let max_scale = current_to_new.0[0]
        .abs()
        .max(current_to_new.0[3].abs());
    if max_skew > MATH_EPSILON && max_scale > MATH_EPSILON {
        return false;
    }
    let mut points = [
        Vec2D::new(rect.min_x, rect.min_y),
        Vec2D::new(rect.max_x, rect.max_y),
    ];
    current_to_new.map_points_in_place(&mut points);
    *rect = Aabb::new(
        simd_min(points[0].x, points[1].x),
        simd_min(points[0].y, points[1].y),
        simd_max(points[0].x, points[1].x),
        simd_max(points[0].y, points[1].y),
    );
    true
}

#[cfg(test)]
mod transform_rect_to_new_space_tests {
    use super::{Aabb, Mat2D, transform_rect_to_new_space};

    #[test]
    fn tiny_skew_maps_only_the_pinned_diagonal_points() {
        let mut rect = Aabb::new(0.0, 0.0, 1.0, 1.0);
        let admitted_tiny_skew = Mat2D([1.0, 0.0, -0.00001, 1.0, 0.0, 0.0]);
        assert!(transform_rect_to_new_space(
            &mut rect,
            admitted_tiny_skew,
            Mat2D::IDENTITY,
        ));
        assert_eq!(
            [
                rect.min_x.to_bits(),
                rect.min_y.to_bits(),
                rect.max_x.to_bits(),
                rect.max_y.to_bits(),
            ],
            [0x0000_0000, 0x0000_0000, 0x3f7f_ff58, 0x3f80_0000],
        );

        let four_corner_substitute =
            admitted_tiny_skew.map_bounds(Aabb::new(0.0, 0.0, 1.0, 1.0));
        assert_eq!(four_corner_substitute.min_x.to_bits(), 0xb727_c5ac);
        assert_ne!(rect, four_corner_substitute);
    }

    #[test]
    fn finite_clip_rect_uses_pinned_inverse_and_composition_bits() {
        let current_matrix = Mat2D(
            [
                0x9422_bf8a,
                0x9788_280a,
                0xd2ec_7e6e,
                0x4d52_6674,
                0xe887_c79b,
                0x4bce_95e3,
            ]
            .map(f32::from_bits),
        );
        let new_matrix = Mat2D(
            [
                0xb12b_6d28,
                0x2f8c_b036,
                0xdeb1_8044,
                0x4f30_2db7,
                0x155f_c859,
                0x4858_db48,
            ]
            .map(f32::from_bits),
        );
        let mut rect = Aabb::new(
            f32::from_bits(0xdaea_f96f),
            f32::from_bits(0xa4ee_fdb1),
            f32::from_bits(0x1c3a_27c6),
            f32::from_bits(0x2b86_6340),
        );
        assert!(transform_rect_to_new_space(
            &mut rect,
            current_matrix,
            new_matrix,
        ));
        assert_eq!(
            [
                rect.min_x.to_bits(),
                rect.min_y.to_bits(),
                rect.max_x.to_bits(),
                rect.max_y.to_bits(),
            ],
            [0xe8f5_3a36, 0x4943_d3df, 0xe8f5_3a36, 0x4943_d3df],
        );
    }
}

impl RiveRenderer {
    pub fn implementation_source_identity() -> &'static str {
        "renderer/src/rive_renderer.cpp@4ac7b32798da0482e441ef09304dc3b480ed3ee5"
    }
    pub unsafe fn clipRectImplSource(&mut self, mut rect: Aabb, original_path: &RiveRenderPath) {
        let state = self.current_state().clone();
        if rect.is_empty_or_nan() {
            self.current_state_mut().overallClipPixelBounds = gpu::IAABB::default();
            return;
        }
        if !state.clipRectInverseMatrix.is_null() {
            if !transform_rect_to_new_space(&mut rect, state.matrix, state.clipRectMatrix) {
                unsafe { self.clipPathImplSource(original_path) };
                return;
            }
        }
        let matrix = if state.clipRectInverseMatrix.is_null() {
            state.matrix
        } else {
            state.clipRectMatrix
        };
        let combined = if state.clipRectInverseMatrix.is_null() {
            rect
        } else {
            Aabb::new(
                state.clipRect.min_x.max(rect.min_x),
                state.clipRect.min_y.max(rect.min_y),
                state.clipRect.max_x.min(rect.max_x),
                state.clipRect.max_y.min(rect.max_y),
            )
        };
        let pixel = matrix.map_bounds(combined).round_out();
        let inverse_ptr = unsafe {
            (&mut *self.m_context).make(gpu::ClipRectInverseMatrix::default())
                as *const gpu::ClipRectInverseMatrix
        };
        unsafe {
            (&mut *(inverse_ptr as *mut gpu::ClipRectInverseMatrix)).reset(matrix, combined);
        }
        let current = self.current_state_mut();
        current.clipRect = combined;
        current.clipRectMatrix = matrix;
        current.clipRectPixelBounds = pixel;
        current.overallClipPixelBounds = current.overallClipPixelBounds.intersect(pixel);
        current.clipRectInverseMatrix = inverse_ptr;
    }
    pub unsafe fn clipPathImplSource(&mut self, path: &RiveRenderPath) {
        if path.getBounds().is_empty_or_nan() {
            self.current_state_mut().overallClipPixelBounds = gpu::IAABB::default();
            return;
        }
        let state = self.current_state().clone();
        let mapped = state
            .matrix
            .map_bounding_box(path.getRawPath().points());
        let pixel = mapped.round_out();
        let combined = state.overallClipPixelBounds.intersect(pixel);
        if combined.empty() {
            self.current_state_mut().overallClipPixelBounds = combined;
            return;
        }
        let height = state.clipStackHeight;
        if self.m_clipStack.len() == height
            || !self.m_clipStack[height].isEquivalent(state.matrix, path)
        {
            self.m_clipStack.truncate(height);
            self.m_clipStack
                .push(unsafe { ClipElement::new(state.matrix, path, path.getFillRule(), pixel) });
        }
        self.current_state_mut().overallClipPixelBounds = combined;
        self.current_state_mut().clipStackHeight = height + 1;
    }
    pub fn clipAndPushDrawSource(&mut self, mut draw: DrawUniquePtr) {
        if draw.0.is_null() {
            return;
        }
        if unsafe { self.m_context.as_ref() }.map_or(true, |c| {
            c.isOutsideCurrentFrameExecutable(unsafe { &*draw.0 }.pixelBounds())
        }) {
            return;
        }
        for _attempt in 0..2 {
            // Source AutoResetInternalDrawBatch starts every attempt empty and
            // clears clip updates and partial target pushes on scope exit.
            self.m_internalDrawBatch.clear();
            match unsafe { self.applyClipSource(draw.0) } {
                ApplyClipResult::fullyClipped => {
                    self.m_internalDrawBatch.clear();
                    return;
                }
                ApplyClipResult::failure => {
                    unsafe { &mut *self.m_context }.logicalFlushExecutable();
                }
                ApplyClipResult::success => {
                    self.m_internalDrawBatch.push(draw);
                    let draw_count = self.m_internalDrawBatch.len();
                    let ok = unsafe {
                        (&mut *self.m_context)
                            .pushDrawsExecutable(&mut self.m_internalDrawBatch, draw_count)
                    };
                    if ok {
                        self.m_internalDrawBatch.clear();
                        return;
                    }
                    unsafe { &mut *self.m_context }.logicalFlushExecutable();
                    draw = self.m_internalDrawBatch.pop().unwrap();
                }
            }
            // Source AutoResetInternalDrawBatch clears only after the flush
            // has consumed any clip-update owners from this attempt.
            self.m_internalDrawBatch.clear();
        }
        eprintln!(
            "RiveRenderer::clipAndPushDraw failed. The draw and/or clip stack are too complex."
        );
    }
    pub unsafe fn applyClipSource(&mut self, draw: *mut gpu::Draw) -> ApplyClipResult {
        if draw.is_null() {
            return ApplyClipResult::fullyClipped;
        }
        let state = self.current_state().clone();
        unsafe {
            (&mut *draw).setClipRect(state.clipRectInverseMatrix, state.overallClipPixelBounds);
        }
        if unsafe { (&*draw).clippedPixelBounds().empty() }
            || unsafe {
                (&*self.m_context)
                    .isOutsideCurrentFrameExecutable(unsafe { (&*draw).clippedPixelBounds() })
            }
        {
            return ApplyClipResult::fullyClipped;
        }
        let height = state.clipStackHeight;
        if height == 0 {
            return ApplyClipResult::success;
        }
        let current_id = unsafe { (&*self.m_context).getClipContentID() };
        let (mut current, start) = if current_id == 0 {
            (0, 0)
        } else {
            (0..height)
                .rev()
                .find(|i| self.m_clipStack[*i].clipID == current_id)
                .map_or((0, 0), |i| (current_id, i + 1))
        };
        let interlock = unsafe { (&*self.m_context).frameInterlockMode() };
        if (interlock == gpu::InterlockMode::clockwiseAtomic
            || interlock == gpu::InterlockMode::msaa)
            && current == 0
            && current_id != 0
        {
            let reset=make_clip_reset(*unsafe{(&*self.m_context).getClipContentBounds(current_id)},current_id,gpu::DrawContents::none,crate::mechanical_port::source::renderer::include::rive::renderer::draw_hpp::ClipResetAction::clearPreviousClip);
            if !unsafe { (&*self.m_context).isOutsideCurrentFrameExecutable(reset.pixelBounds()) } {
                self.m_internalDrawBatch.push(own_clip(reset));
            }
        }
        for i in start..height {
            let clip = &mut self.m_clipStack[i];
            let mut paint = RiveRenderPaint::new();
            paint.clipUpdate(current);
            let Some(mut clip_path) = clip.path.clone() else {
                return ApplyClipResult::failure;
            };
            let mut rule = clip.fillRule;
            let mut bounds = Some(clip.pixelBounds);
            if interlock == gpu::InterlockMode::clockwiseAtomic && current != 0 {
                clip_path = invert_clockwise_path(
                    unsafe { &*clip_path.get() },
                    rule,
                    clip.matrix,
                    *unsafe { (&*self.m_context).getClipContentBounds(current) },
                );
                rule = FillRule::Clockwise;
                bounds = None;
            }
            let owner = unsafe {
                make_path_draw_from_source(
                    &mut *self.m_context,
                    clip.matrix,
                    None,
                    clip_path.clone(),
                    rule,
                    &paint,
                    1.0,
                    bounds,
                )
            };
            let Some(owner) = owner else {
                return ApplyClipResult::failure;
            };
            let draw_bounds = *owner.draw.pixelBounds();
            let outer = if current != 0 {
                *unsafe { (&*self.m_context).getTightenedClipBounds(current) }
            } else {
                AABBu16 {
                    left: 0,
                    top: 0,
                    right: unsafe { (&*self.m_context).frameDescriptor().renderTargetWidth as u16 },
                    bottom: unsafe {
                        (&*self.m_context).frameDescriptor().renderTargetHeight as u16
                    },
                }
            };
            let tightened = outer.intersect(draw_bounds);
            let id = unsafe {
                (&mut *self.m_context).generateClipIDExecutable(draw_bounds, current, tightened)
            };
            if id == 0 {
                return ApplyClipResult::failure;
            }
            clip.clipID = id;
            let contents = owner.draw.drawContents();
            let mut ptr = own_path(owner);
            unsafe {
                (&mut *ptr.0).setClipID(id);
                if !(&*self.m_context).isOutsideCurrentFrameExecutable(&draw_bounds) {
                    self.m_internalDrawBatch.push(ptr);
                }
            }
            if current != 0 && interlock == gpu::InterlockMode::msaa {
                let reset_bounds = *unsafe { (&*self.m_context).getClipContentBounds(current) };
                let reset=make_clip_reset(reset_bounds,current,contents,crate::mechanical_port::source::renderer::include::rive::renderer::draw_hpp::ClipResetAction::intersectPreviousClip);
                if !unsafe {
                    (&*self.m_context).isOutsideCurrentFrameExecutable(reset.pixelBounds())
                } {
                    self.m_internalDrawBatch.push(own_clip(reset));
                }
            }
            current = id;
        }
        unsafe {
            (&mut *draw).setClipID(current);
            (&mut *self.m_context).setClipContentID(current)
        };
        ApplyClipResult::success
    }
}

impl RendererContract for RiveRenderer {
    fn save(&mut self) {
        let copy = *self.current_state();
        self.m_renderStateStack.push(copy);
    }
    fn restore(&mut self) {
        debug_assert!(self.m_renderStateStack.len() > 1);
        debug_assert!(
            self.current_state().clipStackHeight
                >= self.m_renderStateStack[self.m_renderStateStack.len() - 2].clipStackHeight
        );
        self.m_renderStateStack.pop();
    }
    fn transform(&mut self, matrix: &Mat2D) {
        let current = self.current_state().matrix;
        self.current_state_mut().matrix = mul(current, *matrix);
    }
    unsafe fn drawPath(&mut self, path: *mut RenderPath, paint: *mut RenderPaint) {
        if path.is_null() || paint.is_null() {
            return;
        }
        let p = unsafe { &*(path.cast::<RiveRenderPath>()) };
        let q = unsafe { &*(paint.cast::<RiveRenderPaint>()) };
        if p.getRawPath().points().is_empty() {
            return;
        }
        let frame = unsafe { (&*self.m_context).frameDescriptor() };
        if q.getIsStroked() && frame.strokesDisabled {
            return;
        }
        if !q.getIsStroked() && frame.fillsDisabled {
            return;
        }
        if q.getIsStroked() && !(q.getThickness() > 0.0)
            || !(q.getFeather() >= 0.0)
            || self.current_state().overallClipPixelBounds.empty()
        {
            return;
        }
        let image_matrix = (!q.getImageTexture().is_null()).then(|| {
            mul(self.current_state().matrix, *q.getImageTransform())
        });
        if q.getFeather() != 0.0 && !q.getIsStroked() {
            if p.getFillRule() != FillRule::Clockwise && !frame.clockwiseFillOverride {
                return;
            }
            let matrix_max_scale = max_scale(self.current_state().matrix);
            if q.getFeather() * matrix_max_scale > 1.0 {
                let softened = p.makeSoftenedCopyForFeathering(q.getFeather(), matrix_max_scale);
                let owner = unsafe {
                    make_path_draw_from_source(
                        &mut *self.m_context,
                        self.current_state().matrix,
                        image_matrix,
                        softened,
                        p.getFillRule(),
                        q,
                        self.current_state().modulatedOpacity,
                        None,
                    )
                };
                if let Some(owner) = owner {
                    self.clipAndPushDrawSource(own_path(owner));
                }
                return;
            }
        }
        let path_owner = unsafe { ref_rcp(path.cast::<RiveRenderPath>()) };
        let owner = unsafe {
            make_path_draw_from_source(
                &mut *self.m_context,
                self.current_state().matrix,
                image_matrix,
                path_owner,
                p.getFillRule(),
                q,
                self.current_state().modulatedOpacity,
                None,
            )
        };
        if let Some(owner) = owner {
            self.clipAndPushDrawSource(own_path(owner));
        }
    }
    unsafe fn clipPath(&mut self, path: *mut RenderPath) {
        if path.is_null() {
            return;
        }
        let p = unsafe { &*(path.cast::<RiveRenderPath>()) };
        if self.current_state().overallClipPixelBounds.empty() {
            return;
        }
        if p.getRawPath().points().is_empty() {
            self.current_state_mut().overallClipPixelBounds = gpu::IAABB::default();
            return;
        }
        let mut candidate = Aabb::new(0.0, 0.0, 0.0, 0.0);
        if unsafe { (&*self.m_context).frameSupportsClipRectsExecutable() }
            && RiveRenderer::IsAABB(p.getRawPath(), &mut candidate)
        {
            unsafe { self.clipRectImplSource(candidate, p) }
        } else {
            unsafe { self.clipPathImplSource(p) }
        }
    }
    unsafe fn drawImage(
        &mut self,
        image: *const crate::mechanical_port::source::include::rive::renderer_hpp::RenderImage,
        sampler: ImageSampler,
        blend: BlendMode,
        opacity: f32,
    ) {
        if image.is_null() {
            return;
        }
        let image = unsafe { &*(image.cast::<RiveRenderImage>()) };
        let texture = image.refTexture();
        if texture.get().is_null() {
            return;
        }
        let final_opacity = (opacity * self.current_state().modulatedOpacity).max(0.0);
        self.save();
        self.transform(&Mat2D([
            image.width() as f32,
            0.0,
            0.0,
            image.height() as f32,
            0.0,
            0.0,
        ]));
        if !unsafe { (&*self.m_context).frameSupportsImagePaintForPathsExecutable() } {
            let clip_bounds = self.current_state().overallClipPixelBounds;
            if clip_bounds.empty() {
                self.restore();
                return;
            }
            let b = self
                .current_state()
                .matrix
                .map_bounds(Aabb::new(0.0, 0.0, 1.0, 1.0))
                .round_out();
            self.clipAndPushDrawSource(own_image_rect(unsafe {
                make_image_rect_draw(
                    b,
                    self.current_state().matrix,
                    blend,
                    final_opacity,
                    texture,
                    sampler,
                    gpu::DrawContents::none,
                    0,
                    None,
                )
            }));
        } else {
            if self.m_unitRectPath.is_none() {
                *self.m_unitRectPath = Some(make_rcp(RiveRenderPath::default));
                let p = unsafe { &mut *self.m_unitRectPath.as_mut().unwrap().get() };
                p.line_to(1.0, 0.0);
                p.line_to(1.0, 1.0);
                p.line_to(0.0, 1.0);
            }
            let p = unsafe { &*self.m_unitRectPath.as_ref().unwrap().get() };
            let mut paint = RiveRenderPaint::new();
            paint.image(texture, final_opacity);
            paint.blendMode(blend);
            paint.imageSampler(sampler);
            unsafe {
                <Self as RendererContract>::drawPath(
                    self,
                    p.base.base.renderPath_const() as *mut _,
                    paint.base_ptr(),
                );
            }
        }
        self.restore();
    }
    unsafe fn drawImageMesh(
        &mut self,
        image: *const crate::mechanical_port::source::include::rive::renderer_hpp::RenderImage,
        sampler: ImageSampler,
        vertices: rcp<RenderBuffer>,
        uv: rcp<RenderBuffer>,
        indices: rcp<RenderBuffer>,
        _vertex_count: u32,
        index_count: u32,
        blend: BlendMode,
        opacity: f32,
    ) {
        if image.is_null()
            || vertices.get().is_null()
            || uv.get().is_null()
            || indices.get().is_null()
            || self.current_state().overallClipPixelBounds.empty()
        {
            return;
        }
        let image = unsafe { &*(image.cast::<RiveRenderImage>()) };
        let texture = image.refTexture();
        if texture.get().is_null() {
            return;
        }
        let final_opacity = (opacity * self.current_state().modulatedOpacity).max(0.0);
        self.clipAndPushDrawSource(own_image_mesh(unsafe {
            make_image_mesh_draw(
                FULLSCREEN_PIXEL_BOUNDS,
                self.current_state().matrix,
                blend,
                final_opacity,
                texture,
                sampler,
                gpu::DrawContents::none,
                0,
                None,
                crate::mechanical_port::source::include::rive::refcnt_hpp::static_rcp_cast(
                    vertices,
                ),
                crate::mechanical_port::source::include::rive::refcnt_hpp::static_rcp_cast(uv),
                crate::mechanical_port::source::include::rive::refcnt_hpp::static_rcp_cast(indices),
                index_count,
            )
        }));
    }
    fn modulateOpacity(&mut self, opacity: f32) {
        let current = self.current_state().modulatedOpacity;
        self.current_state_mut().modulatedOpacity = (current * opacity).max(0.0);
    }
}
