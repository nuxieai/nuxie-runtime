/*
 * Mechanical translation of the complete pinned source file.
 * Upstream source revision: ee60b7014f1a28fa6bb5f2588cb274c273080f32
 */

// /*
//  * Copyright 2022 Rive
//  */
//
// #include "rive/renderer/rive_render_image.hpp"
// #include "rive_render_paint.hpp"
// #include "gradient.hpp"
//
// namespace rive
// {
// RiveRenderPaint::RiveRenderPaint() {}
//
// RiveRenderPaint::~RiveRenderPaint() {}
//
// rcp<RiveRenderPaint> RiveRenderPaint::clone() const
// {
//     auto r = make_rcp<RiveRenderPaint>();
//     r->m_data = m_data;
//     return r;
// }
//
// void RiveRenderPaint::color(ColorInt color)
// {
//     m_data.m_paintType = gpu::PaintType::solidColor;
//     m_data.m_simpleValue.color = color;
//     m_data.m_gradient.reset();
// }
//
// void RiveRenderPaint::shader(rcp<RenderShader> shader)
// {
//     m_data.m_gradient = static_rcp_cast<gpu::Gradient>(std::move(shader));
//     m_data.m_paintType = m_data.m_gradient ? m_data.m_gradient->paintType()
//                                            : gpu::PaintType::solidColor;
//     // m_simpleValue.colorRampLocation is unused at this level. A new location
//     // for a this gradient's color ramp will decided by the render context every
//     // frame.
//     m_data.m_simpleValue.color = 0xff000000;
// }
//
// rcp<gpu::Gradient> RiveRenderPaint::getGradientWithOpacity(float opacity) const
// {
//     if (m_data.m_gradient)
//     {
//         return m_data.m_gradient->getModulated(opacity);
//     }
//     return nullptr;
// }
//
// void RiveRenderPaint::modulatedImage(const RenderImage* renderImage,
//                                      ImageSampler sampler,
//                                      const Mat2D& matrix)
// {
//     if (renderImage == nullptr)
//     {
//         m_data.m_imageTexture = nullptr;
//         return;
//     }
//
//     m_data.m_imageSampler = sampler;
//     m_data.m_imageTransform = matrix;
//     LITE_RTTI_CAST_OR_RETURN(riveImage, const RiveRenderImage*, renderImage);
//     m_data.m_imageTexture = riveImage->refTexture();
// }
//
// void RiveRenderPaint::image(rcp<gpu::Texture> imageTexture, float opacity)
// {
//     m_data.m_paintType = gpu::PaintType::solidColor;
//     m_data.m_simpleValue.color = colorModulateOpacity(0xFFFFFFFF, opacity);
//     m_data.m_gradient.reset();
//     m_data.m_imageTexture = std::move(imageTexture);
// }
//
// void RiveRenderPaint::clipUpdate(uint32_t outerClipID)
// {
//     m_data.m_paintType = gpu::PaintType::clipUpdate;
//     m_data.m_simpleValue.outerClipID = outerClipID;
//     m_data.m_gradient.reset();
//     m_data.m_imageTexture.reset();
// }
//
// bool RiveRenderPaint::getIsOpaque() const
// {
//     if (m_data.m_feather != 0)
//     {
//         return false;
//     }
//     if (m_data.m_blendMode != BlendMode::srcOver)
//     {
//         return false;
//     }
//     if (m_data.m_additiveness != 0)
//     {
//         return false;
//     }
//     if (m_data.m_imageTexture != nullptr)
//     {
//         // We can't assume opacity with an image (as it might have non-1.0
//         // alpha)
//         return false;
//     }
//
//     switch (m_data.m_paintType)
//     {
//         case gpu::PaintType::solidColor:
//             return colorAlpha(m_data.m_simpleValue.color) == 0xff;
//         case gpu::PaintType::linearGradient:
//         case gpu::PaintType::radialGradient:
//             return m_data.m_gradient->isOpaque();
//         case gpu::PaintType::clipUpdate:
//             return false;
//     }
//     RIVE_UNREACHABLE();
// }
// } // namespace rive

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use super::rive_render_paint_hpp::RiveRenderPaint;
use crate::mechanical_port::source::renderer::src::gradient_hpp::Gradient;
impl RiveRenderPaint {
    pub fn implementation_source_identity() -> &'static str {
        "renderer/src/rive_render_paint.cpp@ee60b7014f1a28fa6bb5f2588cb274c273080f32"
    }
    pub fn getModulatedGradient(
        &self,
        opacity: f32,
        color: u32,
    ) -> crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<Gradient> {
        if self.m_data.m_gradient.get().is_null() {
            crate::mechanical_port::source::include::rive::refcnt_hpp::rcp::new()
        } else {
            unsafe { (&*self.m_data.m_gradient.get()).getModulated(opacity, color) }
        }
    }
}

impl crate::mechanical_port::source::renderer::include::rive::renderer::draw_hpp::RiveRenderPaintContract for RiveRenderPaint {
    fn getIsLayerMask(&self) -> bool { self.getIsLayerMask() }
    fn getLayerMaskMode(&self) -> nuxie_render_api::LayerMaskMode { self.getLayerMaskMode() }
    fn getAdditiveness(&self)->f32 { self.getAdditiveness() }
    fn getBlendMode(&self)->nuxie_render_api::BlendMode { self.getBlendMode() }
    fn getImageTexture(&self)->crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::Texture> { unsafe { crate::mechanical_port::source::include::rive::refcnt_hpp::ref_rcp(self.m_data.m_imageTexture.get()) } }
    fn getImageSampler(&self)->crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler { self.getImageSampler() }
    fn getImageTransform(&self)->nuxie_render_api::Mat2D { *self.getImageTransform() }
    fn getInverseGradientTransform(&self)->nuxie_render_api::Mat2D { self.getInverseGradientTransform() }
    fn getModulatedGradient(&self, opacity:f32, color:u32)->crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<Gradient> { self.getModulatedGradient(opacity, color) }
    fn getType(&self)->crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::PaintType { self.getType() }
    fn getSimpleValue(&self)->crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SimplePaintValue { self.getSimpleValue() }
    fn getIsOpaque(&self)->bool { self.getIsOpaque() }
    fn getFeather(&self)->f32 { self.getFeather() }
    fn getIsStroked(&self)->bool { self.getIsStroked() }
    fn getThickness(&self)->f32 { self.getThickness() }
    fn getJoin(&self)->nuxie_render_api::StrokeJoin { self.getJoin() }
    fn getCap(&self)->nuxie_render_api::StrokeCap { self.getCap() }
    fn getStrokePosition(&self)->nuxie_render_api::StrokePosition { self.getStrokePosition() }
    fn getForceClosed(&self)->bool { self.getForceClosed() }
}
