/*
 * Copyright 2022 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/rive_render_factory.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// /*
//  * Copyright 2022 Rive
//  */
//
// #pragma once
//
// #include "rive/factory.hpp"
//
// namespace rive
// {
// // Partial rive::Factory implementation for the PLS objects that are
// // backend-agnostic.
// class RiveRenderFactory : public Factory
// {
// public:
//     rcp<RenderShader> makeLinearGradient(float sx,
//                                          float sy,
//                                          float ex,
//                                          float ey,
//                                          const ColorInt colors[], // [count]
//                                          const float stops[],     // [count]
//                                          size_t count) override;
//
//     rcp<RenderShader> makeRadialGradient(float cx,
//                                          float cy,
//                                          float radius,
//                                          const ColorInt colors[], // [count]
//                                          const float stops[],     // [count]
//                                          size_t count) override;
//
//     rcp<RenderPath> makeRenderPath(RawPath&, FillRule) override;
//
//     rcp<RenderPath> makeEmptyRenderPath() override;
//
//     rcp<RenderPaint> makeRenderPaint() override;
// };
// } // namespace rive

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::mechanical_port::source::include::rive::factory_hpp::{
    Factory, FactoryAccess, FactoryContract,
};
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderPaint, RenderPath, RenderShader,
};
use nuxie_render_api::{ColorInt, FillRule, RawPath};

/// Backend-neutral portion of the pinned Factory inheritance graph.  The
/// remaining pure virtual Factory slots stay in `FactoryContract`; this owner
/// contains every concrete PLS creation method declared by the source class.
#[repr(C)]
#[derive(Default)]
pub struct RiveRenderFactory {
    pub(crate) base: Factory,
}

impl FactoryAccess for RiveRenderFactory {
    fn factory(&self) -> &Factory {
        &self.base
    }
    fn factoryMut(&mut self) -> &mut Factory {
        &mut self.base
    }
}

pub trait RiveRenderFactoryAccess: FactoryAccess {
    fn riveRenderFactory(&self) -> &RiveRenderFactory;
    fn riveRenderFactoryMut(&mut self) -> &mut RiveRenderFactory;
}

/// Marker for the one inherited Factory virtual-slot set. RiveRenderFactory
/// adds only the concrete backend-agnostic implementations; it does not
/// redeclare the same virtual methods and therefore cannot create duplicate
/// Rust dispatch slots.
pub trait RiveRenderFactoryContract: FactoryContract + RiveRenderFactoryAccess {}
