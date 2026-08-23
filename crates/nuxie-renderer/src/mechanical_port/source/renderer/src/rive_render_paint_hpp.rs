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
// #include "rive/renderer.hpp"
// #include "rive/renderer/gpu.hpp"
// #include "rive/renderer/texture.hpp"
//
// namespace rive::gpu
// {
// class Gradient;
// }
//
// namespace rive
// {
// // RenderPaint implementation for Rive's pixel local storage renderer.
// class RiveRenderPaint : public LITE_RTTI_OVERRIDE(RenderPaint, RiveRenderPaint)
// {
// public:
//     RiveRenderPaint();
//     ~RiveRenderPaint();
//
//     void style(RenderPaintStyle style) override
//     {
//         m_stroked = style == RenderPaintStyle::stroke;
//     }
//     void color(ColorInt color) override;
//     void thickness(float thickness) override { m_thickness = fabsf(thickness); }
//     void join(StrokeJoin join) override { m_join = join; }
//     void cap(StrokeCap cap) override { m_cap = cap; }
//     void feather(float feather) override { m_feather = fabsf(feather); }
//     void blendMode(BlendMode mode) override { m_blendMode = mode; }
//     void shader(rcp<RenderShader> shader) override;
//     void image(rcp<gpu::Texture>, float opacity);
//     void imageSampler(ImageSampler imageSampler)
//     {
//         m_imageSampler = imageSampler;
//     }
//     void clipUpdate(uint32_t outerClipID);
//     void invalidateStroke() override {}
//
//     gpu::PaintType getType() const { return m_paintType; }
//     bool getIsStroked() const { return m_stroked; }
//     ColorInt getColor() const { return m_simpleValue.color; }
//     const gpu::Gradient* getGradient() const { return m_gradient.get(); }
//     rcp<gpu::Gradient> getGradientWithOpacity(float opacity) const;
//     gpu::Texture* getImageTexture() const { return m_imageTexture.get(); }
//     ImageSampler getImageSampler() const { return m_imageSampler; }
//     float getImageOpacity() const { return m_simpleValue.imageOpacity; }
//     float getOuterClipID() const { return m_simpleValue.outerClipID; }
//     float getThickness() const { return m_thickness; }
//     StrokeJoin getJoin() const
//     {
//         // Feathers ignore the join and always use round.
//         return m_feather != 0 ? StrokeJoin::round : m_join;
//     }
//     StrokeCap getCap() const
//     {
//         // Feathers ignore the cap and always use round.
//         return m_feather != .0 ? StrokeCap::round : m_cap;
//     }
//     float getFeather() const { return m_feather; }
//     BlendMode getBlendMode() const { return m_blendMode; }
//     gpu::SimplePaintValue getSimpleValue() const { return m_simpleValue; }
//     bool getIsOpaque() const;
//
// private:
//     gpu::PaintType m_paintType = gpu::PaintType::solidColor;
//     gpu::SimplePaintValue m_simpleValue;
//     rcp<const gpu::Gradient> m_gradient;
//     rcp<gpu::Texture> m_imageTexture;
//     ImageSampler m_imageSampler = ImageSampler::LinearClamp();
//     float m_thickness = 1;
//     StrokeJoin m_join = StrokeJoin::miter;
//     StrokeCap m_cap = StrokeCap::butt;
//     float m_feather = 0;
//     BlendMode m_blendMode = BlendMode::srcOver;
//     bool m_stroked = false;
// };
// } // namespace rive
//

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use crate::mechanical_port::source::include::rive::refcnt_hpp::{
    rcp, static_rcp_cast, RefCntTarget,
};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderPaint as SourceRenderPaint, RenderPaintContract, RenderShader as SourceRenderShader,
};
use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler;
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{
    LiteRttiBase, LiteRttiCastFrom, LiteRttiTypeId, CONST_ID,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;
use crate::mechanical_port::source::renderer::src::gradient_hpp::{Gradient, GradientShader};
use nuxie_render_api::{
    BlendMode, ColorInt, RenderPaint, RenderPaintStyle, RenderShader, StrokeCap, StrokeJoin,
};
use std::any::Any;
use std::mem::ManuallyDrop;

#[repr(C)]
pub struct RiveRenderPaint {
    pub(crate) base: SourceRenderPaint,
    pub m_paintType: gpu::PaintType,
    pub m_simpleValue: gpu::SimplePaintValue,
    pub(crate) m_gradient: ManuallyDrop<rcp<Gradient>>,
    pub m_imageTexture: ManuallyDrop<rcp<Texture>>,
    pub m_imageSampler: ImageSampler,
    pub m_thickness: f32,
    pub m_join: StrokeJoin,
    pub m_cap: StrokeCap,
    pub m_feather: f32,
    pub m_blendMode: BlendMode,
    pub m_stroked: bool,
}
impl Default for RiveRenderPaint {
    fn default() -> Self {
        Self {
            base: unsafe { SourceRenderPaint::new_for_owner::<Self>() },
            m_paintType: gpu::PaintType::solidColor,
            m_simpleValue: gpu::SimplePaintValue::default(),
            m_gradient: ManuallyDrop::new(rcp::new()),
            m_imageTexture: ManuallyDrop::new(rcp::new()),
            m_imageSampler: ImageSampler::LinearClamp(),
            m_thickness: 1.0,
            m_join: StrokeJoin::Miter,
            m_cap: StrokeCap::Butt,
            m_feather: 0.0,
            m_blendMode: BlendMode::SrcOver,
            m_stroked: false,
        }
    }
}
impl Drop for RiveRenderPaint {
    fn drop(&mut self) {
        // C++ destroys members in reverse declaration order, then the base.
        // ManuallyDrop preserves the physical field order while making that
        // intrusive-release order explicit.
        unsafe {
            ManuallyDrop::drop(&mut self.m_imageTexture);
            ManuallyDrop::drop(&mut self.m_gradient);
        }
    }
}
impl RiveRenderPaint {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn base_ptr(&self) -> *mut SourceRenderPaint {
        self as *const Self as *mut SourceRenderPaint
    }
    pub fn style(&mut self, style: RenderPaintStyle) {
        self.m_stroked = style == RenderPaintStyle::Stroke;
    }
    pub fn color(&mut self, color: ColorInt) {
        self.m_paintType = gpu::PaintType::solidColor;
        self.m_simpleValue = gpu::SimplePaintValue { color };
        *self.m_gradient = rcp::new();
        *self.m_imageTexture = rcp::new();
    }
    pub fn thickness(&mut self, v: f32) {
        self.m_thickness = v.abs();
    }
    pub fn join(&mut self, v: StrokeJoin) {
        self.m_join = v;
    }
    pub fn cap(&mut self, v: StrokeCap) {
        self.m_cap = v;
    }
    pub fn feather(&mut self, v: f32) {
        self.m_feather = v.abs();
    }
    pub fn blendMode(&mut self, v: BlendMode) {
        self.m_blendMode = v;
    }
    pub fn imageSampler(&mut self, v: ImageSampler) {
        self.m_imageSampler = v;
    }
    pub fn image(&mut self, texture: rcp<Texture>, opacity: f32) {
        self.m_paintType = gpu::PaintType::image;
        self.m_simpleValue = gpu::SimplePaintValue {
            imageOpacity: opacity,
        };
        *self.m_gradient = rcp::new();
        *self.m_imageTexture = texture;
    }
    pub fn clipUpdate(&mut self, id: u32) {
        self.m_paintType = gpu::PaintType::clipUpdate;
        self.m_simpleValue = gpu::SimplePaintValue { outerClipID: id };
        *self.m_gradient = rcp::new();
        *self.m_imageTexture = rcp::new();
    }
    /// Installs the source gradient shader slot.
    ///
    /// # Safety
    /// A non-null `shader` must be an intrusive `Gradient` allocation stored
    /// through its offset-zero `RenderShader` base. The source operation uses
    /// a static cast rather than RTTI validation; callers with a product
    /// `RenderShader` must perform that exact type proof before entering this
    /// boundary.
    pub unsafe fn shader_source(
        &mut self,
        shader: crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<SourceRenderShader>,
    ) {
        *self.m_gradient = unsafe {
            crate::mechanical_port::source::include::rive::refcnt_hpp::static_rcp_cast(shader)
        };
        self.m_paintType = if self.m_gradient.get().is_null() {
            gpu::PaintType::solidColor
        } else {
            unsafe { (&*self.m_gradient.get()).paintType() }
        };
        self.m_simpleValue.color = 0xff000000;
        *self.m_imageTexture = rcp::new();
    }
    pub fn shader_api(&mut self, shader: Option<&dyn RenderShader>) {
        let Some(shader) = shader else {
            unsafe { self.shader_source(rcp::new()) };
            return;
        };
        let Some(gradient) = shader.as_any().downcast_ref::<GradientShader>() else {
            return;
        };
        let owned = gradient.gradient();
        let owned = unsafe {
            crate::mechanical_port::source::include::rive::refcnt_hpp::static_rcp_cast(owned)
        };
        unsafe { self.shader_source(owned) };
    }
    pub fn getType(&self) -> gpu::PaintType {
        self.m_paintType
    }
    pub fn getIsStroked(&self) -> bool {
        self.m_stroked
    }
    pub fn getColor(&self) -> ColorInt {
        unsafe { self.m_simpleValue.color }
    }
    pub fn getGradient(&self) -> *const Gradient {
        self.m_gradient.get() as *const _
    }
    pub fn getImageTexture(&self) -> *mut Texture {
        self.m_imageTexture.get()
    }
    pub fn getImageSampler(&self) -> ImageSampler {
        self.m_imageSampler
    }
    pub fn getImageOpacity(&self) -> f32 {
        unsafe { self.m_simpleValue.imageOpacity }
    }
    pub fn getOuterClipID(&self) -> f32 {
        unsafe { self.m_simpleValue.outerClipID as f32 }
    }
    pub fn getThickness(&self) -> f32 {
        self.m_thickness
    }
    pub fn getJoin(&self) -> StrokeJoin {
        if self.m_feather != 0.0 {
            StrokeJoin::Round
        } else {
            self.m_join
        }
    }
    pub fn getCap(&self) -> StrokeCap {
        if self.m_feather != 0.0 {
            StrokeCap::Round
        } else {
            self.m_cap
        }
    }
    pub fn getFeather(&self) -> f32 {
        self.m_feather
    }
    pub fn getBlendMode(&self) -> BlendMode {
        self.m_blendMode
    }
    pub fn getSimpleValue(&self) -> gpu::SimplePaintValue {
        self.m_simpleValue
    }
    pub fn getIsOpaque(&self) -> bool {
        if self.m_feather != 0.0 || self.m_blendMode != BlendMode::SrcOver {
            return false;
        }
        match self.m_paintType {
            gpu::PaintType::solidColor => (unsafe { self.m_simpleValue.color } >> 24) == 0xff,
            gpu::PaintType::linearGradient | gpu::PaintType::radialGradient => {
                !self.m_gradient.get().is_null() && unsafe { (&*self.m_gradient.get()).isOpaque() }
            }
            gpu::PaintType::image | gpu::PaintType::clipUpdate => false,
        }
    }
}
impl LiteRttiBase for RiveRenderPaint {
    fn liteTypeID(&self) -> u32 {
        self.base.liteTypeID()
    }
    fn setLiteTypeID(&mut self, id: u32) {
        self.base.setLiteTypeID(id);
    }
}
impl LiteRttiTypeId for RiveRenderPaint {
    const LITE_RTTI_TYPE_ID: u32 = CONST_ID("RiveRenderPaint");
}
impl LiteRttiCastFrom<SourceRenderPaint> for RiveRenderPaint {
    unsafe fn from_base(base: *mut SourceRenderPaint) -> *mut Self {
        base.cast()
    }
}
unsafe impl RefCntTarget for RiveRenderPaint {
    fn r#ref(&self) {
        RefCntTarget::r#ref(&self.base);
    }
    unsafe fn unref(&self) {
        unsafe { RefCntTarget::unref(&self.base) };
    }
}
impl RenderPaintContract for RiveRenderPaint {
    fn style(
        &mut self,
        style: crate::mechanical_port::source::include::rive::renderer_hpp::RenderPaintStyle,
    ) {
        self.style(if style == crate::mechanical_port::source::include::rive::renderer_hpp::RenderPaintStyle::stroke { RenderPaintStyle::Stroke } else { RenderPaintStyle::Fill });
    }
    fn color(&mut self, value: ColorInt) {
        self.color(value);
    }
    fn thickness(&mut self, value: f32) {
        self.thickness(value);
    }
    fn join(&mut self, value: StrokeJoin) {
        self.join(value);
    }
    fn cap(&mut self, value: StrokeCap) {
        self.cap(value);
    }
    fn feather(&mut self, value: f32) {
        self.feather(value);
    }
    fn blendMode(&mut self, value: BlendMode) {
        self.blendMode(value);
    }
    unsafe fn shader(
        &mut self,
        shader: crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<
            crate::mechanical_port::source::include::rive::renderer_hpp::RenderShader,
        >,
    ) {
        unsafe { self.shader_source(shader) };
    }
    fn invalidateStroke(&mut self) {}
}
impl RenderPaint for RiveRenderPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn style(&mut self, s: RenderPaintStyle) {
        self.style(s)
    }
    fn color(&mut self, v: ColorInt) {
        self.color(v)
    }
    fn thickness(&mut self, v: f32) {
        self.thickness(v)
    }
    fn join(&mut self, v: StrokeJoin) {
        self.join(v)
    }
    fn cap(&mut self, v: StrokeCap) {
        self.cap(v)
    }
    fn feather(&mut self, v: f32) {
        self.feather(v)
    }
    fn blend_mode(&mut self, v: BlendMode) {
        self.blendMode(v)
    }
    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        self.shader_api(shader)
    }
    fn invalidate_stroke(&mut self) {}
}

/// Product-facing owner for the exact intrusive paint allocation produced by
/// RiveRenderFactory. The wrapper supplies the `nuxie_render_api` trait object;
/// the complete source paint stays in its original `rcp` allocation.
pub struct RiveRenderPaintHandle {
    source: rcp<RiveRenderPaint>,
}

impl RiveRenderPaintHandle {
    /// # Safety
    /// `source` must be a fresh RiveRenderFactory result whose complete owner
    /// is RiveRenderPaint, with no second safe product-wrapper authority.
    pub(crate) unsafe fn from_source(source: rcp<SourceRenderPaint>) -> Option<Self> {
        if source.get().is_null() {
            return None;
        }
        // SAFETY: the backend-agnostic source factory always allocates
        // RiveRenderPaint with its RenderPaint base at offset zero.
        let source = unsafe { static_rcp_cast(source) };
        Some(Self { source })
    }

    pub fn source(&self) -> &RiveRenderPaint {
        // SAFETY: construction rejected null and the handle owns this retain.
        unsafe { &*self.source.get() }
    }

    pub fn source_mut(&mut self) -> &mut RiveRenderPaint {
        // SAFETY: this mutable product handle is non-Clone and never exposes a
        // second safe mutable owner for the underlying source allocation.
        unsafe { &mut *self.source.get() }
    }

    pub fn source_base(&self) -> &SourceRenderPaint {
        &self.source().base
    }

    pub fn source_base_mut(&mut self) -> &mut SourceRenderPaint {
        &mut self.source_mut().base
    }

    pub fn into_source(self) -> rcp<SourceRenderPaint> {
        let mut source = self.source;
        // SAFETY: move the exact derived owner into its offset-zero base type.
        unsafe { rcp::converting_move_ctor(&mut source) }
    }
}

impl RenderPaint for RiveRenderPaintHandle {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn style(&mut self, value: RenderPaintStyle) {
        self.source_mut().style(value);
    }
    fn color(&mut self, value: ColorInt) {
        self.source_mut().color(value);
    }
    fn thickness(&mut self, value: f32) {
        self.source_mut().thickness(value);
    }
    fn join(&mut self, value: StrokeJoin) {
        self.source_mut().join(value);
    }
    fn cap(&mut self, value: StrokeCap) {
        self.source_mut().cap(value);
    }
    fn feather(&mut self, value: f32) {
        self.source_mut().feather(value);
    }
    fn blend_mode(&mut self, value: BlendMode) {
        self.source_mut().blendMode(value);
    }
    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        self.source_mut().shader_api(shader);
    }
    fn invalidate_stroke(&mut self) {}
}
