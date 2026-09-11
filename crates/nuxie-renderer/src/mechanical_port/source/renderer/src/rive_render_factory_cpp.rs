/*
 * Copyright 2022 Rive
 */

// Mechanical translation of the complete pinned source implementation
// renderer/src/rive_render_factory.cpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// /*
//  * Copyright 2022 Rive
//  */
//
// #include "rive/renderer/rive_render_factory.hpp"
// #include "gradient.hpp"
// #include "rive_render_paint.hpp"
// #include "rive_render_path.hpp"
// #include "rive/renderer/rive_renderer.hpp"
//
// namespace rive
// {
// rcp<RenderShader> RiveRenderFactory::makeLinearGradient(
//     float sx,
//     float sy,
//     float ex,
//     float ey,
//     const ColorInt colors[], // [count]
//     const float stops[],     // [count]
//     size_t count)
// {
//     return gpu::Gradient::MakeLinear(sx, sy, ex, ey, colors, stops, count);
// }
//
// rcp<RenderShader> RiveRenderFactory::makeRadialGradient(
//     float cx,
//     float cy,
//     float radius,
//     const ColorInt colors[], // [count]
//     const float stops[],     // [count]
//     size_t count)
// {
//     return gpu::Gradient::MakeRadial(cx, cy, radius, colors, stops, count);
// }
//
// rcp<RenderPath> RiveRenderFactory::makeRenderPath(RawPath& rawPath,
//                                                   FillRule fillRule)
// {
//     return make_rcp<RiveRenderPath>(fillRule, rawPath);
// }
//
// rcp<RenderPath> RiveRenderFactory::makeEmptyRenderPath()
// {
//     return make_rcp<RiveRenderPath>();
// }
//
// rcp<RenderPaint> RiveRenderFactory::makeRenderPaint()
// {
//     return make_rcp<RiveRenderPaint>();
// }
// } // namespace rive

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::super::include::rive::renderer::rive_render_factory_hpp::RiveRenderFactory;
use super::gradient_hpp::{Gradient, GradientShader};
use super::rive_render_paint_hpp::{RiveRenderPaint, RiveRenderPaintHandle};
use super::rive_render_path_hpp::{RiveRenderPath, RiveRenderPathHandle};
use crate::mechanical_port::source::include::rive::refcnt_hpp::{make_rcp, rcp};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderPaint, RenderPath, RenderShader,
};
use nuxie_render_api::{ColorInt, FillRule, RawPath};

impl RiveRenderFactory {
    pub fn implementation_source_identity() -> &'static str {
        "renderer/src/rive_render_factory.cpp@4ac7b32798da0482e441ef09304dc3b480ed3ee5"
    }

    /// Pointer/count spelling of the source virtual slot.
    pub unsafe fn makeLinearGradientSource(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: *const ColorInt,
        stops: *const f32,
        count: usize,
    ) -> rcp<RenderShader> {
        let mut gradient = unsafe { Gradient::MakeLinear(sx, sy, ex, ey, colors, stops, count) };
        // The source returns the derived rcp as the base RenderShader owner,
        // transferring the single intrusive retain without an extra ref().
        unsafe { rcp::converting_move_ctor(&mut gradient) }
    }

    pub unsafe fn makeRadialGradientSource(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: *const ColorInt,
        stops: *const f32,
        count: usize,
    ) -> rcp<RenderShader> {
        let mut gradient = unsafe { Gradient::MakeRadial(cx, cy, radius, colors, stops, count) };
        unsafe { rcp::converting_move_ctor(&mut gradient) }
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
        if colors.len() != stops.len() {
            return rcp::new();
        }
        // SAFETY: slices provide valid source `[count]` arrays for the whole
        // call and Gradient copies them before returning.
        unsafe {
            self.makeLinearGradientSource(
                sx,
                sy,
                ex,
                ey,
                colors.as_ptr(),
                stops.as_ptr(),
                colors.len(),
            )
        }
    }

    pub fn makeRadialGradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> rcp<RenderShader> {
        if colors.len() != stops.len() {
            return rcp::new();
        }
        unsafe {
            self.makeRadialGradientSource(
                cx,
                cy,
                radius,
                colors.as_ptr(),
                stops.as_ptr(),
                colors.len(),
            )
        }
    }

    /// Product-facing adapter: the public shader owns the exact source
    /// intrusive gradient allocation rather than exposing a stack/boxed
    /// `Gradient` value.
    pub fn makeLinearGradientShader(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Option<GradientShader> {
        if colors.len() != stops.len() {
            return None;
        }
        let gradient = unsafe {
            Gradient::MakeLinear(
                sx,
                sy,
                ex,
                ey,
                colors.as_ptr(),
                stops.as_ptr(),
                colors.len(),
            )
        };
        GradientShader::new(gradient)
    }

    pub fn makeRadialGradientShader(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Option<GradientShader> {
        if colors.len() != stops.len() {
            return None;
        }
        let gradient = unsafe {
            Gradient::MakeRadial(
                cx,
                cy,
                radius,
                colors.as_ptr(),
                stops.as_ptr(),
                colors.len(),
            )
        };
        GradientShader::new(gradient)
    }

    pub fn makeLinearGradientHandle(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Option<GradientShader> {
        self.makeLinearGradientShader(sx, sy, ex, ey, colors, stops)
    }

    pub fn makeRadialGradientHandle(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Option<GradientShader> {
        self.makeRadialGradientShader(cx, cy, radius, colors, stops)
    }

    pub fn makeRenderPath(&mut self, rawPath: &mut RawPath, fillRule: FillRule) -> rcp<RenderPath> {
        let mut path = make_rcp(|| RiveRenderPath::new_with_raw_path(fillRule, rawPath));
        // Source `return make_rcp<RiveRenderPath>(...)` converts to the
        // inherited RenderPath owner without retaining a second allocation.
        unsafe { rcp::converting_move_ctor(&mut path) }
    }

    pub fn makeEmptyRenderPath(&mut self) -> rcp<RenderPath> {
        let mut path = make_rcp(RiveRenderPath::default);
        unsafe { rcp::converting_move_ctor(&mut path) }
    }

    pub fn makeRenderPaint(&mut self) -> rcp<RenderPaint> {
        let mut paint = make_rcp(RiveRenderPaint::new);
        unsafe { rcp::converting_move_ctor(&mut paint) }
    }

    pub fn makeRenderPathHandle(
        &mut self,
        rawPath: &mut RawPath,
        fillRule: FillRule,
    ) -> Option<RiveRenderPathHandle> {
        let source = self.makeRenderPath(rawPath, fillRule);
        // SAFETY: makeRenderPath just constructed the fresh exact
        // RiveRenderPath allocation and moved its sole retain into source.
        unsafe { RiveRenderPathHandle::from_source(source) }
    }

    pub fn makeEmptyRenderPathHandle(&mut self) -> Option<RiveRenderPathHandle> {
        let source = self.makeEmptyRenderPath();
        // SAFETY: the source factory returns a fresh RiveRenderPath owner.
        unsafe { RiveRenderPathHandle::from_source(source) }
    }

    pub fn makeRenderPaintHandle(&mut self) -> Option<RiveRenderPaintHandle> {
        let source = self.makeRenderPaint();
        // SAFETY: the source factory returns a fresh RiveRenderPaint owner.
        unsafe { RiveRenderPaintHandle::from_source(source) }
    }
}
