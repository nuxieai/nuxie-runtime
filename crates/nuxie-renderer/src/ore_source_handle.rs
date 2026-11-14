//! The handles this crate's source objects issue into a
//! [`CanvasTextureInfo`], and how its GL, Vulkan and WebGPU contexts resolve
//! them.
//!
//! A source `RenderCanvas` or `RiveRenderImage` hands its retained source
//! object across the ORE seam as an opaque `Rc<dyn Any>`. The types behind it
//! are private to this module, so nothing else can issue one, and a context
//! resolves a handle only by downcasting it back. A handle from another
//! backend (Metal's bridges) or one a caller made up does not downcast and
//! wraps nothing.

use std::{any::Any, rc::Rc};

use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
#[cfg(any(
    feature = "native-vulkan-experimental",
    feature = "native-webgpu-experimental",
    feature = "renderer-vulkan",
    feature = "renderer-webgpu",
    feature = "renderer-webgl2",
    feature = "renderer-metal",
    feature = "ore-gl"
))]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;

/// A retained source `RenderCanvas`, as its context renders into it.
#[cfg(any(
    feature = "native-vulkan-experimental",
    feature = "native-webgpu-experimental",
    feature = "renderer-vulkan",
    feature = "renderer-webgpu",
    feature = "renderer-webgl2",
    feature = "renderer-metal",
    feature = "ore-gl"
))]
struct SourceCanvas(rcp<RenderCanvas>);

/// A retained source `gpu::Texture`, as a context samples it.
struct SourceTexture(rcp<Texture>);

/// The handle for rendering into `canvas`, or `None` for a null canvas.
#[cfg(any(
    feature = "native-vulkan-experimental",
    feature = "native-webgpu-experimental",
    feature = "renderer-vulkan",
    feature = "renderer-webgpu",
    feature = "renderer-webgl2",
    feature = "renderer-metal",
    feature = "ore-gl"
))]
pub(crate) fn source_canvas_handle(canvas: &rcp<RenderCanvas>) -> Option<Rc<dyn Any>> {
    if canvas.get().is_null() {
        return None;
    }
    Some(Rc::new(SourceCanvas(rcp::copy_ctor(canvas))))
}

/// The handle for sampling `texture`, or `None` for a null texture.
pub(crate) fn source_texture_handle(texture: rcp<Texture>) -> Option<Rc<dyn Any>> {
    if texture.get().is_null() {
        return None;
    }
    Some(Rc::new(SourceTexture(texture)))
}

#[cfg(any(
    feature = "ore-gl",
    feature = "native-vulkan-experimental",
    feature = "native-webgpu-experimental"
))]
pub(crate) use resolve::{wrap_source_canvas, wrap_source_texture};

#[cfg(any(
    feature = "ore-gl",
    feature = "native-vulkan-experimental",
    feature = "native-webgpu-experimental"
))]
mod resolve {
    use super::{SourceCanvas, SourceTexture};
    use nuxie_ore_metal::{
        context::{CanvasTextureInfo, ContextApi},
        gpu_resource::AnyResourceHandle,
    };

    /// Wraps the canvas `info` renders into, if this crate issued it.
    pub(crate) fn wrap_source_canvas<C: ContextApi + ?Sized>(
        context: &mut C,
        info: &CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        let canvas = info.canvas.as_deref()?.downcast_ref::<SourceCanvas>()?;
        // SAFETY: `SourceCanvas` is only built around a non-null retained
        // source RenderCanvas, which `info` keeps alive across this call, and
        // a source RenderCanvas is what every backend's raw wrap reads.
        unsafe { context.wrapCanvasTexture(canvas.0.get().cast()) }
    }

    /// Wraps the texture `info` samples, if this crate issued it.
    pub(crate) fn wrap_source_texture<C: ContextApi + ?Sized>(
        context: &mut C,
        info: &CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        let texture = info.texture.as_deref()?.downcast_ref::<SourceTexture>()?;
        // SAFETY: `SourceTexture` is only built around a non-null retained
        // source gpu::Texture, which `info` keeps alive across this call, and
        // a source gpu::Texture is what every backend's raw wrap reads.
        unsafe { context.wrapRiveTexture(texture.0.get().cast(), info.width, info.height) }
    }
}
