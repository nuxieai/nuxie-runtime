//! Mechanical owner surface for pinned `renderer/include/rive/renderer/draw.hpp`.
//!
//! The executable geometry implementation is the already-rooted, source-line
//! corresponding `crate::draw` owner. The scheduling records live beside
//! `RenderContext`, matching the C++ mutual dependency between `draw.hpp` and
//! `render_context.hpp`; this module gives that owner its literal source path
//! and exports the concrete records used by the paired `draw.cpp` bridge.

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use crate::mechanical_port::source::include::rive::renderer_hpp::{RenderBuffer, RenderPath};
pub use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    ContourDirections, PaintType, SimplePaintValue,
};
pub use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    ClipReset, Draw, DrawObjectType, DrawUniquePtr, Gradient, ImageMeshDraw, ImageRectDraw,
    PathDraw, RenderContext,
};
pub use nuxie_render_api::{BlendMode, FillRule, Mat2D, RawPath, StrokeCap, StrokeJoin, Vec2D};

pub(crate) use crate::draw::{
    build_feather_tessellation_with_direction, build_fill_tessellation,
    build_interior_tessellation, build_stroke_tessellation_with_layout,
    clockwise_atomic_negate_coverage, feather_atlas_fill_direction, feather_atlas_scale,
    feather_pixel_bounds, feather_requires_atlas, path_coarse_area, path_pixel_bounds,
    should_use_interior_tessellation, FeatherFillDirection, FillTessellation, InteriorTessellation,
    StrokePreparationScratch, StrokeTessellation,
};

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathCoverageType {
    pixelLocalStorage,
    clockwise,
    clockwiseAtomic,
    msaa,
    featherAtlas,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipResetAction {
    clearPreviousClip,
    intersectPreviousClip,
}

/// Source `Draw::FULLSCREEN_PIXEL_BOUNDS`.
pub const FULLSCREEN_PIXEL_BOUNDS:
    crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB =
    crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB {
        left: 0,
        top: 0,
        right: 1 << 24,
        bottom: 1 << 24,
    };

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangulatorAxis {
    horizontal,
    vertical,
    dontCare,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteriorTriangulationOp {
    countDataAndTriangulate = 0,
    pushOuterCubicTessellationData = 1,
}

/// Executable getter surface consumed by the pinned `PathDraw::Make` and
/// `PathDraw` constructor. Backends may keep their authored paint owner; this
/// contract preserves every source getter, including the already-opacity-
/// modulated gradient owner returned by `getGradientWithOpacity()`.
pub trait RiveRenderPaintContract {
    fn getBlendMode(&self) -> BlendMode;
    fn getImageTexture(
        &self,
    ) -> crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<
        crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::Texture,
    >;
    fn getImageSampler(
        &self,
    ) -> crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler;
    fn getGradientWithOpacity(
        &self,
        opacity: f32,
    ) -> crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<Gradient>;
    fn getType(&self) -> PaintType;
    fn getSimpleValue(&self) -> SimplePaintValue;
    fn getIsOpaque(&self) -> bool;
    fn getFeather(&self) -> f32;
    fn getIsStroked(&self) -> bool;
    fn getThickness(&self) -> f32;
    fn getJoin(&self) -> StrokeJoin;
    fn getCap(&self) -> StrokeCap;
}

/// Exact data-backed paint implementation for mechanical callers that do not
/// have a more-derived authored `RiveRenderPaint` vtable owner.
pub struct RiveRenderPaintData {
    pub blend_mode: BlendMode,
    pub image_texture: crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<
        crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::Texture,
    >,
    pub image_sampler:
        crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler,
    pub gradient_with_opacity:
        crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<Gradient>,
    pub paint_type: PaintType,
    pub simple_value: SimplePaintValue,
    pub is_opaque: bool,
    pub feather: f32,
    pub is_stroked: bool,
    pub thickness: f32,
    pub join: StrokeJoin,
    pub cap: StrokeCap,
}

impl RiveRenderPaintContract for RiveRenderPaintData {
    fn getBlendMode(&self) -> BlendMode {
        self.blend_mode
    }
    fn getImageTexture(
        &self,
    ) -> crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<
        crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::Texture,
    > {
        self.image_texture.clone()
    }
    fn getImageSampler(
        &self,
    ) -> crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler
    {
        self.image_sampler
    }
    fn getGradientWithOpacity(
        &self,
        _: f32,
    ) -> crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<Gradient> {
        self.gradient_with_opacity.clone()
    }
    fn getType(&self) -> PaintType {
        self.paint_type
    }
    fn getSimpleValue(&self) -> SimplePaintValue {
        self.simple_value
    }
    fn getIsOpaque(&self) -> bool {
        self.is_opaque
    }
    fn getFeather(&self) -> f32 {
        self.feather
    }
    fn getIsStroked(&self) -> bool {
        self.is_stroked
    }
    fn getThickness(&self) -> f32 {
        self.thickness
    }
    fn getJoin(&self) -> StrokeJoin {
        self.join
    }
    fn getCap(&self) -> StrokeCap {
        self.cap
    }
}

/// Inline declarations from `ImageMeshDraw` that are not owned by
/// `crate::draw`'s CPU geometry module.
impl ImageMeshDraw {
    pub fn vertexBuffer(&self) -> *mut RenderBuffer {
        self.vertex_buffer
    }
    pub fn uvBuffer(&self) -> *mut RenderBuffer {
        self.uv_buffer
    }
    pub fn indexBuffer(&self) -> *mut RenderBuffer {
        self.index_buffer
    }
}

/// One source `DrawUniquePtr` plus its complete Rust allocation owner.
///
/// C++ block-allocates draws and stores a custom unique pointer which only
/// releases intrusive references. Rust retains the complete allocation here
/// and hands the exact offset-zero `Draw` address to `RenderContext`.
pub struct DrawAllocation<T> {
    owner: Box<T>,
    draw: *mut Draw,
}

impl<T> DrawAllocation<T> {
    /// # Safety
    /// `draw` must point at the offset-zero `Draw` base within `owner`.
    pub unsafe fn new(owner: Box<T>, draw: *mut Draw) -> Self {
        Self { owner, draw }
    }

    pub fn draw_ptr(&mut self) -> *mut Draw {
        self.draw
    }

    pub fn owner(&self) -> &T {
        &self.owner
    }
}
