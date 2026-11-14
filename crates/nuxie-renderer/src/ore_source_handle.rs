//! The handles this crate's source objects issue into a
//! [`CanvasTextureInfo`], and how its GL, Vulkan and WebGPU contexts resolve
//! them.
//!
//! A source `RenderCanvas` or `RiveRenderImage` hands its retained source
//! object across the ORE seam as an opaque `Rc<dyn Any>`. The handle's type
//! names the backend that issued it: a [`SourceCanvas<I>`] or
//! [`SourceTexture<I>`] for the issuer `I`. A context resolves a handle only
//! by downcasting it to its own issuer's type, so a handle another backend
//! issued (another source backend, or Metal's bridges), one in the wrong
//! slot, or one a caller made up does not downcast and wraps nothing. The
//! types are private to this module, so nothing else can issue one.
//!
//! The issuer matters because each backend's raw wrap reads the backend's own
//! derived texture or render target behind the source base: Vulkan's casts
//! the texture to its `Texture2D` without any check of its own.

use std::{any::Any, marker::PhantomData, rc::Rc};

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

/// The backend whose context resolves a handle: the render context that made
/// the source object. Each source backend names its own uninhabited issuer.
pub(crate) trait SourceIssuer: 'static {}

/// Handles the GL context resolves.
#[cfg(feature = "ore-gl")]
pub(crate) enum GlIssued {}
#[cfg(feature = "ore-gl")]
impl SourceIssuer for GlIssued {}

/// Handles the Vulkan context resolves.
#[cfg(any(feature = "native-vulkan-experimental", all(test, feature = "ore-gl")))]
pub(crate) enum VulkanIssued {}
#[cfg(any(feature = "native-vulkan-experimental", all(test, feature = "ore-gl")))]
impl SourceIssuer for VulkanIssued {}

/// Handles the WebGPU context resolves.
#[cfg(any(feature = "native-webgpu-experimental", all(test, feature = "ore-gl")))]
pub(crate) enum WebGpuIssued {}
#[cfg(any(feature = "native-webgpu-experimental", all(test, feature = "ore-gl")))]
impl SourceIssuer for WebGpuIssued {}

/// Issues the texture handle of one backend; an image records the one its
/// execution domain uses when it joins that domain.
pub(crate) type TextureIssuer = fn(rcp<Texture>) -> Option<Rc<dyn Any>>;

/// A retained source `RenderCanvas`, as the context of `I` renders into it.
#[cfg(any(
    feature = "native-vulkan-experimental",
    feature = "native-webgpu-experimental",
    feature = "renderer-vulkan",
    feature = "renderer-webgpu",
    feature = "renderer-webgl2",
    feature = "renderer-metal",
    feature = "ore-gl"
))]
struct SourceCanvas<I>(rcp<RenderCanvas>, PhantomData<fn() -> I>);

/// A retained source `gpu::Texture`, as the context of `I` samples it.
struct SourceTexture<I>(rcp<Texture>, PhantomData<fn() -> I>);

/// The handle for rendering into `canvas` in the context of `I`, or `None`
/// for a null canvas.
#[cfg(any(
    feature = "native-vulkan-experimental",
    feature = "native-webgpu-experimental",
    feature = "renderer-vulkan",
    feature = "renderer-webgpu",
    feature = "renderer-webgl2",
    feature = "renderer-metal",
    feature = "ore-gl"
))]
pub(crate) fn source_canvas_handle<I: SourceIssuer>(
    canvas: &rcp<RenderCanvas>,
) -> Option<Rc<dyn Any>> {
    if canvas.get().is_null() {
        return None;
    }
    Some(Rc::new(SourceCanvas::<I>(rcp::copy_ctor(canvas), PhantomData)))
}

/// The handle for sampling `texture` in the context of `I`, or `None` for a
/// null texture.
pub(crate) fn source_texture_handle<I: SourceIssuer>(texture: rcp<Texture>) -> Option<Rc<dyn Any>> {
    if texture.get().is_null() {
        return None;
    }
    Some(Rc::new(SourceTexture::<I>(texture, PhantomData)))
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
    use super::{SourceCanvas, SourceIssuer, SourceTexture};
    use nuxie_ore_metal::{
        context::{CanvasTextureInfo, ContextApi},
        gpu_resource::AnyResourceHandle,
    };

    /// Wraps the canvas `info` renders into, if the backend `I` of `context`
    /// issued it.
    pub(crate) fn wrap_source_canvas<I: SourceIssuer, C: ContextApi + ?Sized>(
        context: &mut C,
        info: &CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        let canvas = info.canvas.as_deref()?.downcast_ref::<SourceCanvas<I>>()?;
        // SAFETY: `SourceCanvas<I>` is only built around a non-null retained
        // source RenderCanvas that the render context of `I` made, which
        // `info` keeps alive across this call; that is what the raw wrap of
        // `I`'s context reads.
        unsafe { context.wrapCanvasTexture(canvas.0.get().cast()) }
    }

    /// Wraps the texture `info` samples, if the backend `I` of `context`
    /// issued it.
    pub(crate) fn wrap_source_texture<I: SourceIssuer, C: ContextApi + ?Sized>(
        context: &mut C,
        info: &CanvasTextureInfo,
    ) -> Option<AnyResourceHandle> {
        let texture = info.texture.as_deref()?.downcast_ref::<SourceTexture<I>>()?;
        // SAFETY: `SourceTexture<I>` is only built around a non-null retained
        // source gpu::Texture that the render context of `I` made, which
        // `info` keeps alive across this call; that is what the raw wrap of
        // `I`'s context reads.
        unsafe { context.wrapRiveTexture(texture.0.get().cast(), info.width, info.height) }
    }
}
