/*
 * Mechanical translation of the complete pinned source file.
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 * The literal source is retained below in declaration/order form.
 */

// /*
//  * Copyright 2022 Rive
//  */
//
// #pragma once
//
// #include "rive/math/raw_path.hpp"
// #include "rive/renderer.hpp"
// #include "rive/renderer/gpu.hpp"
// #include "rive/renderer/draw.hpp"
// #include "rive/renderer/render_context.hpp"
// #include <vector>
//
// namespace rive::gpu
// {
// class RenderContext;
// } // namespace rive::gpu
//
// namespace rive
// {
// class GrInnerFanTriangulator;
// class RiveRenderPath;
// class RiveRenderPaint;
//
// // Renderer implementation for Rive's pixel local storage renderer.
// class RiveRenderer : public Renderer
// {
// public:
//     RiveRenderer(gpu::RenderContext*);
//     ~RiveRenderer() override;
//
//     void save() override;
//     void restore() override;
//     void transform(const Mat2D& matrix) override;
//     void drawPath(RenderPath*, RenderPaint*) override;
//     void clipPath(RenderPath*) override;
//     void drawImage(const RenderImage*,
//                    ImageSampler,
//                    BlendMode,
//                    float opacity) override;
//     void drawImageMesh(const RenderImage*,
//                        ImageSampler,
//                        rcp<RenderBuffer> vertices_f32,
//                        rcp<RenderBuffer> uvCoords_f32,
//                        rcp<RenderBuffer> indices_u16,
//                        uint32_t vertexCount,
//                        uint32_t indexCount,
//                        BlendMode,
//                        float opacity) override;
//     void modulateOpacity(float opacity) override;
//
//     // Determines if a path is an axis-aligned rectangle that can be represented
//     // by rive::AABB.
//     static bool IsAABB(const RawPath&, AABB* result);
//
// #ifdef TESTING
//     bool hasClipRect() const
//     {
//         return m_renderStateStack.back().clipRectInverseMatrix != nullptr;
//     }
//     const AABB& getClipRect() const
//     {
//         return m_renderStateStack.back().clipRect;
//     }
//     const Mat2D& getClipRectMatrix() const
//     {
//         return m_renderStateStack.back().clipRectMatrix;
//     }
//     float currentModulatedOpacity() const
//     {
//         return m_renderStateStack.back().modulatedOpacity;
//     }
// #endif
//
// private:
//     void clipRectImpl(AABB, const RiveRenderPath* originalPath);
//     void clipPathImpl(const RiveRenderPath*);
//
//     // Clips and pushes the given draw to m_context. If the clipped draw is too
//     // complex to be supported by the GPU buffers, even after a logical flush,
//     // then nothing is drawn.
//     void clipAndPushDraw(gpu::DrawUniquePtr);
//
//     // Pushes any necessary clip updates to m_internalDrawBatch and sets the
//     // Draw's clipID and clipRectInverseMatrix, if any. Returns failure if the
//     // operation failed, at which point the caller should issue a logical flush
//     // and try again.
//     enum class ApplyClipResult
//     {
//         success,
//         failure,
//         fullyClipped,
//     };
//     [[nodiscard]] ApplyClipResult applyClip(gpu::Draw*);
//
//     struct RenderState
//     {
//         Mat2D matrix;
//         size_t clipStackHeight = 0;
//         AABB clipRect;
//         Mat2D clipRectMatrix;
//         IAABB clipRectPixelBounds;
//         const gpu::ClipRectInverseMatrix* clipRectInverseMatrix = nullptr;
//         float modulatedOpacity = 1.0f;
//
//         // The pixel bounds for all clipping (clip rects *and* clip paths),
//         // which defaults to a maximally-large rectangle
//         IAABB overallClipPixelBounds = IAABB::makeMaximal();
//     };
//     std::vector<RenderState> m_renderStateStack{1};
//
//     struct ClipElement
//     {
//         ClipElement() = default;
//         ClipElement(const Mat2D&,
//                     const RiveRenderPath*,
//                     FillRule,
//                     IAABB pixelBounds);
//         ~ClipElement();
//
//         void reset(const Mat2D&,
//                    const RiveRenderPath*,
//                    FillRule,
//                    IAABB pixelBounds);
//         bool isEquivalent(const Mat2D&, const RiveRenderPath*) const;
//
//         Mat2D matrix;
//         uint64_t rawPathMutationID;
//         AABB pathBounds;
//         IAABB pixelBounds;
//         rcp<const RiveRenderPath> path;
//         FillRule fillRule; // Bc RiveRenderPath fillRule can mutate during the
//                            // artboard draw process.
//         uint32_t clipID;
//     };
//     std::vector<ClipElement> m_clipStack;
//
//     gpu::RenderContext* const m_context;
//
//     std::vector<gpu::DrawUniquePtr> m_internalDrawBatch;
//
//     // Path of the rectangle [0, 0, 1, 1]. Used to draw images.
//     rcp<RiveRenderPath> m_unitRectPath;
// };
// } // namespace rive
//

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use crate::mechanical_port::source::include::rive::refcnt_hpp::{rcp, RefCntTarget};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::DrawUniquePtr;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::RenderContext;
use crate::mechanical_port::source::renderer::src::rive_render_path_hpp::RiveRenderPath;
use nuxie_render_api::{Aabb, FillRule, Mat2D, RawPath};
use std::mem::ManuallyDrop;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RenderState {
    pub matrix: Mat2D,
    pub clipStackHeight: usize,
    pub clipRect: Aabb,
    pub clipRectMatrix: Mat2D,
    pub clipRectPixelBounds: gpu::IAABB,
    pub clipRectInverseMatrix: *const gpu::ClipRectInverseMatrix,
    pub modulatedOpacity: f32,
    pub overallClipPixelBounds: gpu::IAABB,
}
impl Default for RenderState {
    fn default() -> Self {
        Self {
            matrix: Mat2D::IDENTITY,
            clipStackHeight: 0,
            clipRect: Aabb::new(0.0, 0.0, 0.0, 0.0),
            clipRectMatrix: Mat2D::IDENTITY,
            clipRectPixelBounds: gpu::IAABB::default(),
            clipRectInverseMatrix: core::ptr::null(),
            modulatedOpacity: 1.0,
            overallClipPixelBounds: gpu::IAABB::makeMaximal(),
        }
    }
}

pub struct ClipElement {
    pub matrix: Mat2D,
    pub rawPathMutationID: u64,
    pub pathBounds: Aabb,
    pub pixelBounds: gpu::IAABB,
    pub path: Option<rcp<RiveRenderPath>>,
    pub fillRule: FillRule,
    pub clipID: u32,
}
impl ClipElement {
    pub unsafe fn new(
        matrix: Mat2D,
        path: &RiveRenderPath,
        fill_rule: FillRule,
        pixel_bounds: gpu::IAABB,
    ) -> Self {
        let mut value = Self {
            matrix,
            rawPathMutationID: 0,
            pathBounds: path.getBounds(),
            pixelBounds: pixel_bounds,
            path: unsafe {
                Some(
                    crate::mechanical_port::source::include::rive::refcnt_hpp::ref_rcp(
                        path as *const _ as *mut _,
                    ),
                )
            },
            fillRule: fill_rule,
            clipID: 0,
        };
        unsafe { value.reset(matrix, path, fill_rule, pixel_bounds) };
        value
    }
    pub unsafe fn reset(
        &mut self,
        matrix: Mat2D,
        path: &RiveRenderPath,
        fill_rule: FillRule,
        pixel_bounds: gpu::IAABB,
    ) {
        self.matrix = matrix;
        self.rawPathMutationID = path.getRawPathMutationID();
        self.pathBounds = path.getBounds();
        self.path = unsafe {
            Some(
                crate::mechanical_port::source::include::rive::refcnt_hpp::ref_rcp(
                    path as *const _ as *mut _,
                ),
            )
        };
        self.fillRule = fill_rule;
        self.pixelBounds = pixel_bounds;
        self.clipID = 0;
    }
    pub fn isEquivalent(&self, matrix: Mat2D, path: &RiveRenderPath) -> bool {
        self.matrix == matrix
            && self.rawPathMutationID == path.getRawPathMutationID()
            && self.fillRule == path.getFillRule()
    }
}

pub struct RiveRenderer {
    pub m_renderStateStack: ManuallyDrop<Vec<RenderState>>,
    pub m_clipStack: ManuallyDrop<Vec<ClipElement>>,
    pub m_context: *mut RenderContext,
    pub m_internalDrawBatch: ManuallyDrop<Vec<DrawUniquePtr>>,
    pub m_unitRectPath: ManuallyDrop<Option<rcp<RiveRenderPath>>>,
}
impl Drop for RiveRenderer {
    fn drop(&mut self) {
        // Preserve C++ reverse member destruction. The context is a raw
        // nonowner and has no destructor work.
        unsafe {
            ManuallyDrop::drop(&mut self.m_unitRectPath);
            ManuallyDrop::drop(&mut self.m_internalDrawBatch);
            ManuallyDrop::drop(&mut self.m_clipStack);
            ManuallyDrop::drop(&mut self.m_renderStateStack);
        }
    }
}
impl RiveRenderer {
    /// # Safety
    /// `context` must remain live, pinned, and exclusively frame-managed for
    /// the entire lifetime of the returned renderer.
    pub unsafe fn new(context: *mut RenderContext) -> Self {
        Self {
            m_renderStateStack: ManuallyDrop::new(vec![RenderState::default()]),
            m_clipStack: ManuallyDrop::new(Vec::new()),
            m_context: context,
            m_internalDrawBatch: ManuallyDrop::new(Vec::new()),
            m_unitRectPath: ManuallyDrop::new(None),
        }
    }
    /// # Safety
    /// The caller must uphold the same pinned lifetime invariant because the
    /// returned source-shaped owner stores a raw Context pointer.
    pub unsafe fn new_from_context(context: &mut RenderContext) -> Self {
        unsafe { Self::new(context as *mut RenderContext) }
    }
    pub fn IsAABB(path: &RawPath, result: &mut Aabb) -> bool {
        use nuxie_render_api::PathVerb;
        let verbs = path.verbs();
        let points = path.points();
        if verbs.len() < 4
            || verbs[0..4]
                != [
                    PathVerb::Move,
                    PathVerb::Line,
                    PathVerb::Line,
                    PathVerb::Line,
                ]
            || points.len() < 4
        {
            return false;
        }
        if points.iter().skip(4).any(|p| *p != points[0]) {
            return false;
        }
        let p0 = points[0];
        let p1 = points[1];
        let p2 = points[2];
        let p3 = points[3];
        if (p0.x == p1.x && p2.x == p3.x && p0.y == p3.y && p1.y == p2.y)
            || (p0.x == p3.x && p1.x == p2.x && p0.y == p1.y && p2.y == p3.y)
        {
            *result = Aabb::new(
                p0.x.min(p2.x),
                p0.y.min(p2.y),
                p0.x.max(p2.x),
                p0.y.max(p2.y),
            );
            return true;
        }
        false
    }
    pub fn current_state(&self) -> &RenderState {
        self.m_renderStateStack.last().unwrap()
    }
    pub fn current_state_mut(&mut self) -> &mut RenderState {
        self.m_renderStateStack.last_mut().unwrap()
    }
    pub fn hasClipRect(&self) -> bool {
        !self.current_state().clipRectInverseMatrix.is_null()
    }
    pub fn getClipRect(&self) -> &Aabb {
        &self.current_state().clipRect
    }
    pub fn getClipRectMatrix(&self) -> &Mat2D {
        &self.current_state().clipRectMatrix
    }
    pub fn currentModulatedOpacity(&self) -> f32 {
        self.current_state().modulatedOpacity
    }
}
