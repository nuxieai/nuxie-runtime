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
//
// namespace rive
// {
// // RenderPath implementation for Rive's pixel local storage renderer.
// class RiveRenderPath : public LITE_RTTI_OVERRIDE(RenderPath, RiveRenderPath)
// {
// public:
//     RiveRenderPath() = default;
//     RiveRenderPath(FillRule fillRule, RawPath& rawPath);
//
//     void rewind() override;
//     void fillRule(FillRule rule) override { m_fillRule = rule; }
//
//     void moveTo(float x, float y) override;
//     void lineTo(float x, float y) override;
//     void cubicTo(float ox, float oy, float ix, float iy, float x, float y)
//         override;
//     void close() override;
//
//     void addPath(CommandPath* p, const Mat2D& m) override
//     {
//         addRenderPath(p->renderPath(), m);
//     }
//     void addRenderPath(const RenderPath* path, const Mat2D& matrix) override;
//     void addRenderPathBackwards(const RenderPath* path,
//                                 const Mat2D& transform) override;
//     void addRawPath(const RawPath& path) override;
//     const RawPath& getRawPath() const { return m_rawPath; }
//     FillRule getFillRule() const { return m_fillRule; }
//
//     const AABB& getBounds() const;
//     // Approximates the area of the path by linearizing it with a coarse
//     // tolerance of 8px in artboard space.
//     constexpr static float kCoarseAreaTolerance = 8;
//     float getCoarseArea() const;
//     // Determine if the path's signed, post-transform area is positive.
//     bool isClockwiseDominant(const Mat2D& viewMatrix) const;
//     uint64_t getRawPathMutationID() const;
//
//     // 1-dimensional feathering along the normal vector quits looking like a
//     // blur when there is strong curvature. This method returns a copy of the
//     // path with shorter, flatter curves that will more accurately depict a
//     // gaussian blur when drawn with the given feather.
//     //
//     // TODO: Move this work to the GPU.
//     rcp<RiveRenderPath> makeSoftenedCopyForFeathering(float feather,
//                                                       float matrixMaxScale);
//
// #ifdef DEBUG
//     // Allows ref holders to guarantee the rawPath doesn't mutate during a
//     // specific time.
//     void lockRawPathMutations() const { ++m_rawPathMutationLockCount; }
//     void unlockRawPathMutations() const
//     {
//         assert(m_rawPathMutationLockCount > 0);
//         --m_rawPathMutationLockCount;
//     }
// #endif
//
// private:
//     FillRule m_fillRule = FillRule::nonZero;
//     RawPath m_rawPath;
//     mutable AABB m_bounds;
//     mutable float m_coarseArea;
//     mutable uint64_t m_rawPathMutationID;
//
//     enum Dirt
//     {
//         kPathBoundsDirt = 1 << 0,
//         kRawPathMutationIDDirt = 1 << 1,
//         kPathCoarseAreaDirt = 1 << 2,
//         kAllDirt = ~0,
//     };
//
//     mutable uint32_t m_dirt = kAllDirt;
//     RIVE_DEBUG_CODE(mutable int m_rawPathMutationLockCount = 0;)
// };
// } // namespace rive
//

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use crate::mechanical_port::source::include::rive::refcnt_hpp::{
    rcp, static_rcp_cast, RefCntTarget,
};
use crate::mechanical_port::source::include::rive::renderer_hpp::{RenderPath, RenderPathContract};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{
    LiteRttiBase, LiteRttiCastFrom, LiteRttiTypeId, CONST_ID,
};
use nuxie_render_api::{Aabb, FillRule, Mat2D, RawPath, RenderPath as ApiRenderPath, Vec2D};
use std::any::Any;
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

#[repr(C)]
pub struct RiveRenderPath {
    pub(crate) base: RenderPath,
    pub m_fillRule: FillRule,
    pub m_rawPath: RawPath,
    pub m_bounds: Cell<Aabb>,
    pub m_coarseArea: Cell<f32>,
    pub m_rawPathMutationID: Cell<u64>,
    pub m_dirt: Cell<u32>,
    #[cfg(debug_assertions)]
    pub m_rawPathMutationLockCount: Cell<i32>,
}
impl Default for RiveRenderPath {
    fn default() -> Self {
        Self {
            base: unsafe { RenderPath::new_for_owner::<Self>() },
            m_fillRule: FillRule::NonZero,
            m_rawPath: RawPath::new(),
            m_bounds: Cell::new(Aabb::new(0.0, 0.0, 0.0, 0.0)),
            m_coarseArea: Cell::new(0.0),
            m_rawPathMutationID: Cell::new(0),
            m_dirt: Cell::new(u32::MAX),
            #[cfg(debug_assertions)]
            m_rawPathMutationLockCount: Cell::new(0),
        }
    }
}
impl RiveRenderPath {
    pub const kCoarseAreaTolerance: f32 = 8.0;
    pub fn new_with_raw_path(fill_rule: FillRule, raw_path: &mut RawPath) -> Self {
        let mut path = Self::default();
        path.m_fillRule = fill_rule;
        path.m_rawPath = core::mem::replace(raw_path, RawPath::new());
        path.m_rawPath.prune_empty_segments();
        path
    }
    pub fn getRawPath(&self) -> &RawPath {
        &self.m_rawPath
    }
    pub fn getFillRule(&self) -> FillRule {
        self.m_fillRule
    }
    pub fn getBounds(&self) -> Aabb {
        if self.m_dirt.get() & Self::K_PATH_BOUNDS_DIRT != 0 {
            // The source caches RawPath::bounds() until the next owner mutation.
            self.m_bounds.set(
                self.m_rawPath
                    .bounds()
                    .unwrap_or(Aabb::new(0.0, 0.0, 0.0, 0.0)),
            );
            self.m_dirt
                .set(self.m_dirt.get() & !Self::K_PATH_BOUNDS_DIRT);
        }
        self.m_bounds.get()
    }
    pub fn getCoarseArea(&self) -> f32 {
        if self.m_dirt.get() & Self::K_PATH_COARSE_AREA_DIRT != 0 {
            self.m_coarseArea.set(
                crate::mechanical_port::source::renderer::src::rive_render_path_cpp::coarse_area(
                    &self.m_rawPath,
                ),
            );
            self.m_dirt
                .set(self.m_dirt.get() & !Self::K_PATH_COARSE_AREA_DIRT);
        }
        self.m_coarseArea.get()
    }
    pub fn isClockwiseDominant(&self, view: Mat2D) -> bool {
        let a = self.getCoarseArea();
        let m = view.0;
        a * (m[0] * m[3] - m[2] * m[1]) >= 0.0
    }
    pub fn getRawPathMutationID(&self) -> u64 {
        static UNIQUE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
        if self.m_dirt.get() & Self::K_RAW_PATH_MUTATION_ID_DIRT != 0 {
            self.m_rawPathMutationID.set(
                UNIQUE_ID_COUNTER
                    .fetch_add(1, Ordering::SeqCst)
                    .wrapping_add(1),
            );
            self.m_dirt
                .set(self.m_dirt.get() & !Self::K_RAW_PATH_MUTATION_ID_DIRT);
        }
        self.m_rawPathMutationID.get()
    }
    pub fn makeSoftenedCopyForFeathering(&self, feather: f32, matrix_max_scale: f32) -> rcp<Self> {
        crate::mechanical_port::source::include::rive::refcnt_hpp::make_rcp(|| {
            crate::mechanical_port::source::renderer::src::rive_render_path_cpp::softened_copy(
                self,
                feather,
                matrix_max_scale,
            )
        })
    }
    pub fn rewind(&mut self) {
        self.assertRawPathMutationsUnlocked();
        self.m_rawPath.rewind();
        self.m_dirt.set(u32::MAX);
    }
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.assertRawPathMutationsUnlocked();
        self.m_rawPath.move_to(x, y);
        self.m_dirt.set(u32::MAX);
    }
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.assertRawPathMutationsUnlocked();
        self.m_rawPath.inject_implicit_move_if_needed_for_owner();
        let point = Vec2D::new(x, y);
        if self.m_rawPath.points().last().copied() != Some(point) {
            self.m_rawPath.line(point);
        }
        self.m_dirt.set(u32::MAX);
    }
    pub fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.assertRawPathMutationsUnlocked();
        self.m_rawPath.inject_implicit_move_if_needed_for_owner();
        let points = [Vec2D::new(ox, oy), Vec2D::new(ix, iy), Vec2D::new(x, y)];
        if self.m_rawPath.points().last().copied() != Some(points[0])
            || points[0] != points[1]
            || points[1] != points[2]
        {
            self.m_rawPath.cubic(points[0], points[1], points[2]);
        }
        self.m_dirt.set(u32::MAX);
    }
    pub fn close(&mut self) {
        self.assertRawPathMutationsUnlocked();
        self.m_rawPath.close();
        self.m_dirt.set(u32::MAX);
    }
    #[cfg(debug_assertions)]
    fn assertRawPathMutationsUnlocked(&self) {
        assert_eq!(self.m_rawPathMutationLockCount.get(), 0);
    }
    #[cfg(not(debug_assertions))]
    fn assertRawPathMutationsUnlocked(&self) {}
    #[cfg(debug_assertions)]
    pub fn lockRawPathMutations(&self) {
        self.m_rawPathMutationLockCount
            .set(self.m_rawPathMutationLockCount.get() + 1);
    }
    #[cfg(not(debug_assertions))]
    pub fn lockRawPathMutations(&self) {}
    #[cfg(debug_assertions)]
    pub fn unlockRawPathMutations(&self) {
        assert!(self.m_rawPathMutationLockCount.get() > 0);
        self.m_rawPathMutationLockCount
            .set(self.m_rawPathMutationLockCount.get() - 1);
    }
    #[cfg(not(debug_assertions))]
    pub fn unlockRawPathMutations(&self) {}
}
impl RiveRenderPath {
    pub(crate) const K_PATH_BOUNDS_DIRT: u32 = 1 << 0;
    pub(crate) const K_RAW_PATH_MUTATION_ID_DIRT: u32 = 1 << 1;
    pub(crate) const K_PATH_COARSE_AREA_DIRT: u32 = 1 << 2;
}

impl LiteRttiBase for RiveRenderPath {
    fn liteTypeID(&self) -> u32 {
        self.base.liteTypeID()
    }
    fn setLiteTypeID(&mut self, id: u32) {
        self.base.setLiteTypeID(id);
    }
}
impl LiteRttiTypeId for RiveRenderPath {
    const LITE_RTTI_TYPE_ID: u32 = CONST_ID("RiveRenderPath");
}
impl LiteRttiCastFrom<RenderPath> for RiveRenderPath {
    unsafe fn from_base(base: *mut RenderPath) -> *mut Self {
        base.cast()
    }
}
unsafe impl RefCntTarget for RiveRenderPath {
    fn r#ref(&self) {
        RefCntTarget::r#ref(&self.base);
    }
    unsafe fn unref(&self) {
        unsafe { RefCntTarget::unref(&self.base) };
    }
}
unsafe impl RenderPathContract for RiveRenderPath {
    fn rewind(&mut self) {
        self.rewind();
    }
    fn fillRule(&mut self, rule: FillRule) {
        self.m_fillRule = rule;
    }
    fn moveTo(&mut self, x: f32, y: f32) {
        self.move_to(x, y);
    }
    fn lineTo(&mut self, x: f32, y: f32) {
        self.line_to(x, y);
    }
    fn cubicTo(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.cubic_to(ox, oy, ix, iy, x, y);
    }
    fn close(&mut self) {
        self.close();
    }
    unsafe fn addRenderPath(&mut self, path: *const RenderPath, matrix: &Mat2D) {
        self.assertRawPathMutationsUnlocked();
        let other = unsafe { &*(path.cast::<RiveRenderPath>()) };
        let verb_start = self.m_rawPath.verbs().len();
        let point_start = self.m_rawPath.points().len();
        self.m_rawPath.add_path(&other.m_rawPath, *matrix);
        if *matrix != Mat2D::IDENTITY {
            self.m_rawPath
                .prune_empty_segments_from_offsets(verb_start, point_start);
        }
        self.m_dirt.set(u32::MAX);
    }
    unsafe fn addRenderPathBackwards(&mut self, path: *const RenderPath, matrix: &Mat2D) {
        let other = unsafe { &*(path.cast::<RiveRenderPath>()) };
        let verb_start = self.m_rawPath.verbs().len();
        let point_start = self.m_rawPath.points().len();
        self.m_rawPath.add_path_backwards(&other.m_rawPath, *matrix);
        if *matrix != Mat2D::IDENTITY {
            self.m_rawPath
                .prune_empty_segments_from_offsets(verb_start, point_start);
        }
        self.m_dirt.set(u32::MAX);
    }
    fn addRawPath(&mut self, path: &RawPath) {
        self.m_rawPath.add_path(path, Mat2D::IDENTITY);
    }
}
impl ApiRenderPath for RiveRenderPath {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn rewind(&mut self) {
        self.rewind();
    }
    fn fill_rule(&mut self, rule: FillRule) {
        self.m_fillRule = rule;
    }
    fn add_render_path(&mut self, path: &dyn ApiRenderPath, matrix: Mat2D) {
        if let Some(other) = path.as_any().downcast_ref::<Self>() {
            unsafe {
                self.addRenderPath(other.base.base.renderPath_const(), &matrix);
            }
        }
    }
    fn add_render_path_backwards(&mut self, path: &dyn ApiRenderPath, matrix: Mat2D) {
        if let Some(other) = path.as_any().downcast_ref::<Self>() {
            unsafe {
                self.addRenderPathBackwards(other.base.base.renderPath_const(), &matrix);
            }
        }
    }
    fn add_raw_path(&mut self, path: &RawPath) {
        self.addRawPath(path);
    }
    fn move_to(&mut self, x: f32, y: f32) {
        self.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.line_to(x, y);
    }
    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.cubic_to(ox, oy, ix, iy, x, y);
    }
    fn close(&mut self) {
        self.close();
    }
}

/// Product-facing owner for the source `rcp<RenderPath>` returned by Factory.
///
/// The handle is intentionally not `Clone`: the product `RenderPath` API is
/// mutable, so one Rust wrapper retains the one safe mutable-borrow authority.
/// The allocation itself remains the exact intrusive `RiveRenderPath` source
/// owner and is never copied into or replaced by a Rust `Box`.
pub struct RiveRenderPathHandle {
    source: rcp<RiveRenderPath>,
}

impl RiveRenderPathHandle {
    /// # Safety
    /// `source` must be a fresh RiveRenderFactory result whose complete owner
    /// is RiveRenderPath, with no second safe product-wrapper authority.
    pub(crate) unsafe fn from_source(source: rcp<RenderPath>) -> Option<Self> {
        if source.get().is_null() {
            return None;
        }
        // SAFETY: RiveRenderFactory is the only constructor feeding this
        // adapter and always returns an offset-zero RiveRenderPath allocation.
        let source = unsafe { static_rcp_cast(source) };
        Some(Self { source })
    }

    pub fn source(&self) -> &RiveRenderPath {
        // SAFETY: from_source rejects null and this handle owns the retain for
        // the complete allocation throughout the returned borrow.
        unsafe { &*self.source.get() }
    }

    pub fn source_mut(&mut self) -> &mut RiveRenderPath {
        // SAFETY: the handle is non-Clone and does not expose its owning rcp;
        // &mut self therefore carries the wrapper's sole safe mutation right.
        unsafe { &mut *self.source.get() }
    }

    pub fn source_base(&self) -> &RenderPath {
        &self.source().base
    }

    pub fn source_base_mut(&mut self) -> &mut RenderPath {
        &mut self.source_mut().base
    }

    pub fn into_source(self) -> rcp<RenderPath> {
        let mut source = self.source;
        // SAFETY: the offset-zero derived-to-base conversion moves the one
        // logical retain out of the product handle without an extra ref().
        unsafe { rcp::converting_move_ctor(&mut source) }
    }
}

impl ApiRenderPath for RiveRenderPathHandle {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn rewind(&mut self) {
        self.source_mut().rewind();
    }
    fn fill_rule(&mut self, rule: FillRule) {
        self.source_mut().m_fillRule = rule;
    }
    fn add_render_path(&mut self, path: &dyn ApiRenderPath, matrix: Mat2D) {
        let Some(other) = path.as_any().downcast_ref::<Self>() else {
            return;
        };
        let other = other.source_base().base.renderPath_const();
        unsafe {
            self.source_mut().addRenderPath(other, &matrix);
        }
    }
    fn add_render_path_backwards(&mut self, path: &dyn ApiRenderPath, matrix: Mat2D) {
        let Some(other) = path.as_any().downcast_ref::<Self>() else {
            return;
        };
        let other = other.source_base().base.renderPath_const();
        unsafe {
            self.source_mut().addRenderPathBackwards(other, &matrix);
        }
    }
    fn add_raw_path(&mut self, path: &RawPath) {
        self.source_mut().addRawPath(path);
    }
    fn move_to(&mut self, x: f32, y: f32) {
        self.source_mut().move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.source_mut().line_to(x, y);
    }
    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.source_mut().cubic_to(ox, oy, ix, iy, x, y);
    }
    fn close(&mut self) {
        self.source_mut().close();
    }
}
