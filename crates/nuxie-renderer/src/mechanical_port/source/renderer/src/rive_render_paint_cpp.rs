/*
 * Mechanical translation of the complete pinned source file.
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 * The literal source is retained below in declaration/order form.
 */

// /*
//  * Copyright 2022 Rive
//  */
//
// #include "rive_render_paint.hpp"
// #include "gradient.hpp"
//
// namespace rive
// {
// RiveRenderPaint::RiveRenderPaint() {}
//
// RiveRenderPaint::~RiveRenderPaint() {}
//
// void RiveRenderPaint::color(ColorInt color)
// {
//     m_paintType = gpu::PaintType::solidColor;
//     m_simpleValue.color = color;
//     m_gradient.reset();
//     m_imageTexture.reset();
// }
//
// void RiveRenderPaint::shader(rcp<RenderShader> shader)
// {
//     m_gradient = static_rcp_cast<gpu::Gradient>(std::move(shader));
//     m_paintType =
//         m_gradient ? m_gradient->paintType() : gpu::PaintType::solidColor;
//     // m_simpleValue.colorRampLocation is unused at this level. A new location
//     // for a this gradient's color ramp will decided by the render context every
//     // frame.
//     m_simpleValue.color = 0xff000000;
//     m_imageTexture.reset();
// }
//
// rcp<gpu::Gradient> RiveRenderPaint::getGradientWithOpacity(float opacity) const
// {
//     if (m_gradient)
//     {
//         return m_gradient->getModulated(opacity);
//     }
//     return nullptr;
// }
//
// void RiveRenderPaint::image(rcp<gpu::Texture> imageTexture, float opacity)
// {
//     m_paintType = gpu::PaintType::image;
//     m_simpleValue.imageOpacity = opacity;
//     m_gradient.reset();
//     m_imageTexture = std::move(imageTexture);
// }
//
// void RiveRenderPaint::clipUpdate(uint32_t outerClipID)
// {
//     m_paintType = gpu::PaintType::clipUpdate;
//     m_simpleValue.outerClipID = outerClipID;
//     m_gradient.reset();
//     m_imageTexture.reset();
// }
//
// bool RiveRenderPaint::getIsOpaque() const
// {
//     if (m_feather != 0)
//     {
//         return false;
//     }
//     if (m_blendMode != BlendMode::srcOver)
//     {
//         return false;
//     }
//     switch (m_paintType)
//     {
//         case gpu::PaintType::solidColor:
//             return colorAlpha(m_simpleValue.color) == 0xff;
//         case gpu::PaintType::linearGradient:
//         case gpu::PaintType::radialGradient:
//             return m_gradient->isOpaque();
//         case gpu::PaintType::image:
//         case gpu::PaintType::clipUpdate:
//             return false;
//     }
//     RIVE_UNREACHABLE();
// }
// } // namespace rive
//

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use super::rive_render_paint_hpp::RiveRenderPaint;
use crate::mechanical_port::source::renderer::src::gradient_hpp::Gradient;
impl RiveRenderPaint {
    pub fn implementation_source_identity() -> &'static str {
        "renderer/src/rive_render_paint.cpp@4ac7b32798da0482e441ef09304dc3b480ed3ee5"
    }
    pub fn getGradientWithOpacity(
        &self,
        opacity: f32,
    ) -> crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<Gradient> {
        if self.m_gradient.get().is_null() {
            crate::mechanical_port::source::include::rive::refcnt_hpp::rcp::new()
        } else {
            unsafe { (&*self.m_gradient.get()).getModulated(opacity) }
        }
    }
}

impl crate::mechanical_port::source::renderer::include::rive::renderer::draw_hpp::RiveRenderPaintContract for RiveRenderPaint {
    fn getBlendMode(&self)->nuxie_render_api::BlendMode { self.getBlendMode() }
    fn getImageTexture(&self)->crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::Texture> { unsafe { crate::mechanical_port::source::include::rive::refcnt_hpp::ref_rcp(self.m_imageTexture.get()) } }
    fn getImageSampler(&self)->crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler { self.getImageSampler() }
    fn getGradientWithOpacity(&self, opacity:f32)->crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<Gradient> { self.getGradientWithOpacity(opacity) }
    fn getType(&self)->crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::PaintType { self.getType() }
    fn getSimpleValue(&self)->crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SimplePaintValue { self.getSimpleValue() }
    fn getIsOpaque(&self)->bool { self.getIsOpaque() }
    fn getFeather(&self)->f32 { self.getFeather() }
    fn getIsStroked(&self)->bool { self.getIsStroked() }
    fn getThickness(&self)->f32 { self.getThickness() }
    fn getJoin(&self)->nuxie_render_api::StrokeJoin { self.getJoin() }
    fn getCap(&self)->nuxie_render_api::StrokeCap { self.getCap() }
}
