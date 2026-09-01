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
//
//     void modulatedImage(const RenderImage*,
//                         ImageSampler,
//                         const Mat2D&) override;
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
//     float getOuterClipID() const { return m_simpleValue.outerClipID; }
//     float getThickness() const { return m_thickness; }
//     const Mat2D& getImageTransform() const { return m_imageTransform; }
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
//     Mat2D m_imageTransform;
// };
// } // namespace rive
//

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use crate::mechanical_port::source::include::rive::refcnt_hpp::{
    RefCntTarget, rcp, static_rcp_cast,
};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderImage as SourceRenderImage, RenderPaint as SourceRenderPaint, RenderPaintContract,
    RenderShader as SourceRenderShader,
};
use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler;
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{
    CONST_ID, LiteRttiBase, LiteRttiCastFrom, LiteRttiTypeId, lite_rtti_cast,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RenderResourceDomain;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::{
    RiveRenderImage, RiveRenderImageHandle,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;
use crate::mechanical_port::source::renderer::src::gradient_hpp::{Gradient, GradientShader};
use nuxie_render_api::{
    BlendMode, ColorInt, ImageFilter as ApiImageFilter, ImageSampler as ApiImageSampler,
    ImageWrap as ApiImageWrap, Mat2D, RenderImage as ApiRenderImage, RenderPaint, RenderPaintStyle,
    RenderShader, StrokeCap, StrokeJoin,
};
use std::any::Any;
use std::mem::ManuallyDrop;
use std::rc::Rc;

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
    pub m_imageTransform: Mat2D,
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
            m_imageTransform: Mat2D::IDENTITY,
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
        self.m_paintType = gpu::PaintType::solidColor;
        self.m_simpleValue = gpu::SimplePaintValue {
            color: color_modulate_opacity(0xffff_ffff, opacity),
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
    pub fn getOuterClipID(&self) -> f32 {
        unsafe { self.m_simpleValue.outerClipID as f32 }
    }
    pub fn getThickness(&self) -> f32 {
        self.m_thickness
    }
    pub fn getImageTransform(&self) -> &Mat2D {
        &self.m_imageTransform
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
        if !self.m_imageTexture.get().is_null() {
            return false;
        }
        match self.m_paintType {
            gpu::PaintType::solidColor => (unsafe { self.m_simpleValue.color } >> 24) == 0xff,
            gpu::PaintType::linearGradient | gpu::PaintType::radialGradient => {
                !self.m_gradient.get().is_null() && unsafe { (&*self.m_gradient.get()).isOpaque() }
            }
            gpu::PaintType::clipUpdate => false,
        }
    }

    pub unsafe fn modulatedImage(
        &mut self,
        render_image: *const SourceRenderImage,
        sampler: ImageSampler,
        matrix: &Mat2D,
    ) {
        if render_image.is_null() {
            *self.m_imageTexture = rcp::new();
            return;
        }
        self.m_imageSampler = sampler;
        self.m_imageTransform = *matrix;
        let rive_image = unsafe { lite_rtti_cast::<RiveRenderImage, _>(render_image.cast_mut()) };
        if rive_image.is_null() {
            return;
        }
        *self.m_imageTexture = unsafe { (&*rive_image).refTexture() };
    }

    fn modulated_image_api(
        &mut self,
        image: Option<&dyn ApiRenderImage>,
        sampler: ApiImageSampler,
        transform: Mat2D,
    ) {
        let sampler = source_image_sampler(sampler);
        let Some(image) = image else {
            unsafe { self.modulatedImage(core::ptr::null(), sampler, &transform) };
            return;
        };
        let Some(image) = image.as_any().downcast_ref::<RiveRenderImageHandle>() else {
            self.m_imageSampler = sampler;
            self.m_imageTransform = transform;
            return;
        };
        let source = &image.source().base as *const SourceRenderImage;
        unsafe { self.modulatedImage(source, sampler, &transform) };
    }
}

fn color_modulate_opacity(value: ColorInt, opacity: f32) -> ColorInt {
    let source_opacity = ((value >> 24) & 0xff) as f32 / 255.0;
    let product = source_opacity * opacity;
    let clamped = product.min(1.0).max(0.0);
    let alpha = (255.0 * clamped).round() as u32;
    (value & 0x00ff_ffff) | (alpha << 24)
}

fn source_image_sampler(value: ApiImageSampler) -> ImageSampler {
    use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::{
        ImageFilter, ImageWrap,
    };
    ImageSampler {
        wrapX: match value.wrap_x {
            ApiImageWrap::Clamp => ImageWrap::clamp,
            ApiImageWrap::Repeat => ImageWrap::repeat,
            ApiImageWrap::Mirror => ImageWrap::mirror,
        },
        wrapY: match value.wrap_y {
            ApiImageWrap::Clamp => ImageWrap::clamp,
            ApiImageWrap::Repeat => ImageWrap::repeat,
            ApiImageWrap::Mirror => ImageWrap::mirror,
        },
        filter: match value.filter {
            ApiImageFilter::Bilinear => ImageFilter::bilinear,
            ApiImageFilter::Nearest => ImageFilter::nearest,
        },
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
    fn modulatedImage(
        &mut self,
        image: *const SourceRenderImage,
        sampler: ImageSampler,
        transform: &Mat2D,
    ) {
        unsafe { self.modulatedImage(image, sampler, transform) }
    }
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
    fn modulated_image(
        &mut self,
        image: Option<&dyn ApiRenderImage>,
        sampler: ApiImageSampler,
        transform: Mat2D,
    ) {
        self.modulated_image_api(image, sampler, transform)
    }
}

/// Product-facing owner for the exact intrusive paint allocation produced by
/// RiveRenderFactory. The wrapper supplies the `nuxie_render_api` trait object;
/// the complete source paint stays in its original `rcp` allocation.
struct AttachedPaintExecutionDomain {
    resource_domain: RenderResourceDomain,
    // Retains the actual backend execution owner until after source teardown.
    _domain_guard: Rc<dyn Any>,
}

pub struct RiveRenderPaintHandle {
    source: rcp<RiveRenderPaint>,
    // Declared after source so paint/texture release completes before the
    // backend owner drops. Identity and lifetime are one indivisible edge.
    execution_domain: Option<AttachedPaintExecutionDomain>,
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
        Some(Self {
            source,
            execution_domain: None,
        })
    }

    /// Attach the opaque identity and owner of this paint's execution domain
    /// together. The consuming builder permits this attachment exactly once.
    pub(crate) fn with_execution_domain(
        mut self,
        resource_domain: RenderResourceDomain,
        domain_guard: Rc<dyn Any>,
    ) -> Self {
        assert!(
            self.execution_domain.is_none(),
            "paint execution domain already attached"
        );
        self.execution_domain = Some(AttachedPaintExecutionDomain {
            resource_domain,
            _domain_guard: domain_guard,
        });
        self
    }

    /// Returns whether this resource was created by the queried execution
    /// domain. An unattached source handle belongs to no product domain.
    pub(crate) fn belongs_to(&self, resource_domain: &RenderResourceDomain) -> bool {
        self.execution_domain
            .as_ref()
            .is_some_and(|attached| attached.resource_domain.same_domain(resource_domain))
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

    /// Borrows the exact source base only after validating the execution
    /// domain. The borrow cannot outlive this handle and therefore cannot
    /// outlive the bundled lifetime guard.
    pub(crate) fn source_base_for(
        &self,
        resource_domain: &RenderResourceDomain,
    ) -> Option<&SourceRenderPaint> {
        self.belongs_to(resource_domain).then(|| self.source_base())
    }

    pub fn source_base_mut(&mut self) -> &mut SourceRenderPaint {
        &mut self.source_mut().base
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
    fn modulated_image(
        &mut self,
        image: Option<&dyn ApiRenderImage>,
        sampler: ApiImageSampler,
        transform: Mat2D,
    ) {
        let sampler = source_image_sampler(sampler);
        let Some(image) = image else {
            unsafe {
                self.source_mut()
                    .modulatedImage(core::ptr::null(), sampler, &transform)
            };
            return;
        };
        let Some(resource_domain) = self
            .execution_domain
            .as_ref()
            .map(|attached| attached.resource_domain.clone())
        else {
            return;
        };
        let Some(image) = image.as_any().downcast_ref::<RiveRenderImageHandle>() else {
            let source = self.source_mut();
            source.m_imageSampler = sampler;
            source.m_imageTransform = transform;
            return;
        };
        let Some(image) = image.source_base_for(&resource_domain) else {
            return;
        };
        unsafe {
            self.source_mut()
                .modulatedImage(image as *const _, sampler, &transform)
        };
    }
}

#[cfg(test)]
mod handle_tests {
    use super::*;
    use crate::mechanical_port::source::include::rive::refcnt_hpp::make_rcp;

    struct ForeignImage;

    impl ApiRenderImage for ForeignImage {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn retain_image(&self) -> Rc<dyn ApiRenderImage> {
            Rc::new(Self)
        }

        fn image_identity(&self) -> usize {
            0
        }

        fn width(&self) -> u32 {
            1
        }

        fn height(&self) -> u32 {
            1
        }
    }

    fn paint_handle() -> RiveRenderPaintHandle {
        RiveRenderPaintHandle {
            source: make_rcp(RiveRenderPaint::new),
            execution_domain: None,
        }
    }

    fn image_handle(texture: rcp<Texture>) -> RiveRenderImageHandle {
        RiveRenderImageHandle::from_exact(make_rcp(|| unsafe { RiveRenderImage::new(texture) }))
            .expect("source image")
    }

    #[test]
    fn matching_image_domain_installs_the_exact_source_texture() {
        let resource_domain = RenderResourceDomain::new();
        let guard: Rc<dyn Any> = Rc::new(());
        let texture = make_rcp(|| Texture::new(9, 7));
        let expected = texture.get();
        let image =
            image_handle(texture).with_execution_domain(resource_domain.clone(), Rc::clone(&guard));
        let mut paint = paint_handle().with_execution_domain(resource_domain, guard);

        RenderPaint::modulated_image(
            &mut paint,
            Some(&image),
            ApiImageSampler::default(),
            Mat2D::IDENTITY,
        );

        assert_eq!(paint.source().getImageTexture(), expected);

        RenderPaint::modulated_image(
            &mut paint,
            None,
            ApiImageSampler::default(),
            Mat2D::IDENTITY,
        );
        assert!(paint.source().getImageTexture().is_null());
    }

    #[test]
    fn foreign_and_unattached_images_leave_the_paint_unchanged() {
        let resource_domain = RenderResourceDomain::new();
        let guard: Rc<dyn Any> = Rc::new(());
        let seeded_texture = make_rcp(|| Texture::new(3, 5));
        let seeded_texture_ptr = seeded_texture.get();
        let mut paint = paint_handle().with_execution_domain(resource_domain, guard);
        paint.source_mut().image(seeded_texture, 1.0);
        let seeded_sampler = paint.source().getImageSampler();
        let seeded_transform = *paint.source().getImageTransform();

        let unattached = image_handle(make_rcp(|| Texture::new(7, 11)));
        let foreign_guard: Rc<dyn Any> = Rc::new(());
        let foreign = image_handle(make_rcp(|| Texture::new(13, 17)))
            .with_execution_domain(RenderResourceDomain::new(), foreign_guard);
        for image in [&unattached, &foreign] {
            RenderPaint::modulated_image(
                &mut paint,
                Some(image),
                ApiImageSampler {
                    wrap_x: ApiImageWrap::Repeat,
                    wrap_y: ApiImageWrap::Mirror,
                    filter: ApiImageFilter::Nearest,
                },
                Mat2D([2.0, 0.0, 0.0, 3.0, 4.0, 5.0]),
            );
            assert_eq!(paint.source().getImageTexture(), seeded_texture_ptr);
            assert_eq!(paint.source().getImageSampler(), seeded_sampler);
            assert_eq!(*paint.source().getImageTransform(), seeded_transform);
        }
    }

    #[test]
    fn wrong_source_image_type_preserves_texture_but_updates_sampler_and_transform() {
        let resource_domain = RenderResourceDomain::new();
        let guard: Rc<dyn Any> = Rc::new(());
        let seeded_texture = make_rcp(|| Texture::new(3, 5));
        let seeded_texture_ptr = seeded_texture.get();
        let mut paint = paint_handle().with_execution_domain(resource_domain, guard);
        paint.source_mut().image(seeded_texture, 1.0);
        let sampler = ApiImageSampler {
            wrap_x: ApiImageWrap::Repeat,
            wrap_y: ApiImageWrap::Mirror,
            filter: ApiImageFilter::Nearest,
        };
        let transform = Mat2D([2.0, 0.0, 0.0, 3.0, 4.0, 5.0]);

        RenderPaint::modulated_image(
            &mut paint,
            Some(&ForeignImage),
            sampler,
            transform,
        );

        assert_eq!(paint.source().getImageTexture(), seeded_texture_ptr);
        assert_eq!(paint.source().getImageSampler(), source_image_sampler(sampler));
        assert_eq!(*paint.source().getImageTransform(), transform);
    }

    #[test]
    fn paint_domain_gate_and_guard_lifetime_are_bundled() {
        let matching_domain = RenderResourceDomain::new();
        let foreign_domain = RenderResourceDomain::new();
        let guard: Rc<dyn Any> = Rc::new(());
        let paint =
            paint_handle().with_execution_domain(matching_domain.clone(), Rc::clone(&guard));

        assert!(paint.source_base_for(&matching_domain).is_some());
        assert!(paint.source_base_for(&foreign_domain).is_none());
        assert_eq!(Rc::strong_count(&guard), 2);
        drop(paint);
        assert_eq!(Rc::strong_count(&guard), 1);
    }
}
