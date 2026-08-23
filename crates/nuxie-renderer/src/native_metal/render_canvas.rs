//! Guarded product owner for the exact mechanically translated RenderCanvas.

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_metal::MTLTexture;
#[cfg(test)]
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage,
};

#[cfg(test)]
use super::capabilities::MetalCapabilitySelection;
#[cfg(test)]
use super::image_texture::NativeMetalImageTexture;
use super::mechanical_render_context::MechanicalRenderContext;
#[cfg(test)]
use super::render_target::RenderTargetMetal;
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RenderResourceDomain;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImageHandle;
#[cfg(test)]
use crate::RendererError;
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_ore_metal::metal::context::ContextMetal as OreContextMetal;
use nuxie_render_api::RenderImage;

/// One exact source RenderCanvas plus its mechanical execution domain.
///
/// The source canvas owns the paired intrusive image and target allocations.
/// The opaque guard additionally keeps their Metal registry, texture table,
/// and deferred-retirement queue alive. No raw source target or image escapes
/// this product handle.
pub struct NativeMetalRenderCanvas {
    inner: NativeMetalRenderCanvasInner,
}

enum NativeMetalRenderCanvasInner {
    /// Private allocation staging used while the mechanical source factory
    /// constructs its exact intrusive image and target owners.
    #[cfg(test)]
    Allocated {
        image: NativeMetalImageTexture,
        target: RenderTargetMetal,
    },
    Guarded {
        // Rust drops variant fields in declaration order. Release the source
        // canvas and enqueue target/image retirements before the backend.
        source: rcp<RenderCanvas>,
        image_texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
        target_texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
        resource_domain: RenderResourceDomain,
        execution_guard: Rc<RefCell<MechanicalRenderContext>>,
    },
}

impl NativeMetalRenderCanvas {
    #[cfg(test)]
    pub(crate) fn new(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        capabilities: MetalCapabilitySelection,
        width: u32,
        height: u32,
    ) -> Result<Self, RendererError> {
        let descriptor = MTLTextureDescriptor::new();
        descriptor.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
        descriptor.setTextureType(MTLTextureType::Type2D);
        descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
        descriptor.setStorageMode(MTLStorageMode::Private);
        unsafe {
            descriptor.setWidth(width as usize);
            descriptor.setHeight(height as usize);
            descriptor.setMipmapLevelCount(1);
        }
        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| RendererError::NativeMetal("failed to allocate render canvas".into()))?;
        let mut target = RenderTargetMetal::new(
            device,
            MTLPixelFormat::RGBA8Unorm,
            width,
            height,
            capabilities,
        )?;
        target.set_target_texture(Some(texture.clone()))?;
        let image =
            NativeMetalImageTexture::adopt(Some(texture), width, height).ok_or_else(|| {
                RendererError::NativeMetal("failed to adopt render canvas image".into())
            })?;
        Ok(Self {
            inner: NativeMetalRenderCanvasInner::Allocated { image, target },
        })
    }

    pub(super) fn from_source(
        source: rcp<RenderCanvas>,
        execution_guard: Rc<RefCell<MechanicalRenderContext>>,
        resource_domain: RenderResourceDomain,
    ) -> Option<Self> {
        if source.get().is_null() {
            drop(source);
            drop(execution_guard);
            return None;
        }
        // SAFETY: the nonnull source rcp owns the complete RenderCanvas for
        // this borrow. Its image owns the complete source TextureMetal whose
        // nativeHandle override projects its directly owned MTLTexture.
        let canvas = unsafe { &*source.get() };
        let texture_owner = canvas.render_image_ref().refTexture();
        if texture_owner.get().is_null() {
            drop(texture_owner);
            drop(source);
            drop(execution_guard);
            return None;
        }
        let native = unsafe { (&*texture_owner.get()).nativeHandle() };
        let texture = unsafe { Retained::<AnyObject>::retain(native.cast()) }.map(|native| {
            // SAFETY: TextureMetal's nonnull nativeHandle dispatch returns its
            // retained `ProtocolObject<dyn MTLTexture>` and no other object.
            // Null is the pinned allocation-failure state.
            unsafe { Retained::cast_unchecked::<ProtocolObject<dyn MTLTexture>>(native) }
        });
        let target_texture =
            super::mechanical_render_context::retained_canvas_target_texture(canvas);
        let same_texture = match (&texture, &target_texture) {
            (None, None) => true,
            (Some(image), Some(target)) => Retained::as_ptr(image) == Retained::as_ptr(target),
            _ => false,
        };
        if !same_texture {
            drop(texture_owner);
            drop(source);
            drop(execution_guard);
            return None;
        }
        drop(texture_owner);
        Some(Self {
            inner: NativeMetalRenderCanvasInner::Guarded {
                source,
                image_texture: texture,
                target_texture,
                resource_domain,
                execution_guard,
            },
        })
    }

    pub fn width(&self) -> u32 {
        match &self.inner {
            #[cfg(test)]
            NativeMetalRenderCanvasInner::Allocated { target, .. } => target.width(),
            NativeMetalRenderCanvasInner::Guarded { source, .. } => {
                Self::source_ref(source).width()
            }
        }
    }

    pub fn height(&self) -> u32 {
        match &self.inner {
            #[cfg(test)]
            NativeMetalRenderCanvasInner::Allocated { target, .. } => target.height(),
            NativeMetalRenderCanvasInner::Guarded { source, .. } => {
                Self::source_ref(source).height()
            }
        }
    }

    /// Confirms the source image still carries the texture captured from the
    /// source factory's same-texture image/target construction.
    pub fn render_target_and_image_share_texture(&self) -> bool {
        match &self.inner {
            #[cfg(test)]
            NativeMetalRenderCanvasInner::Allocated { image, target } => target
                .target_texture()
                .is_some_and(|texture| std::ptr::eq(texture, image.texture())),
            NativeMetalRenderCanvasInner::Guarded {
                source,
                image_texture,
                target_texture,
                ..
            } => {
                let texture_owner = Self::source_ref(source).render_image_ref().refTexture();
                if texture_owner.get().is_null() {
                    return false;
                }
                let native = unsafe { (&*texture_owner.get()).nativeHandle() };
                match (image_texture, target_texture) {
                    (None, None) => native.is_null(),
                    (Some(image), Some(target)) => {
                        native == Retained::as_ptr(image).cast_mut().cast()
                            && Retained::as_ptr(image) == Retained::as_ptr(target)
                    }
                    _ => false,
                }
            }
        }
    }

    /// Retains the canvas texture for a same-device native consumer such as
    /// the opt-in ORE context.
    pub fn retained_metal_texture(&self) -> Option<Retained<ProtocolObject<dyn MTLTexture>>> {
        match &self.inner {
            #[cfg(test)]
            NativeMetalRenderCanvasInner::Allocated { target, .. } => {
                target.retained_target_texture()
            }
            NativeMetalRenderCanvasInner::Guarded { image_texture, .. } => image_texture.clone(),
        }
    }

    /// Retains the exact source `m_renderImage` member and attaches the same
    /// execution-domain identity and lifetime guard as this canvas.
    ///
    /// This never constructs a second image owner from the native texture.
    pub fn render_image(&self) -> Box<dyn RenderImage> {
        let image = match &self.inner {
            #[cfg(test)]
            NativeMetalRenderCanvasInner::Allocated { .. } => {
                unreachable!("private canvas allocation staging has no source image owner")
            }
            NativeMetalRenderCanvasInner::Guarded {
                source,
                resource_domain,
                execution_guard,
                ..
            } => RiveRenderImageHandle::from_exact(Self::source_ref(source).ref_render_image())
                .expect("a source RenderCanvas always owns a nonnull render image")
                .with_execution_domain(
                    resource_domain.clone(),
                    Rc::clone(execution_guard) as Rc<dyn Any>,
                ),
        };
        Box::new(image)
    }

    /// Creates an ORE texture view from this canvas's retained shared texture.
    /// The returned ORE resource owns its native retain; no ORE-context-specific
    /// view is cached in the canvas.
    pub fn wrap_ore_texture(&self, context: &OreContextMetal) -> Option<AnyResourceHandle> {
        context.wrap_native_texture(
            self.retained_metal_texture()?,
            self.width(),
            self.height(),
            true,
        )
    }

    fn source_ref(source: &rcp<RenderCanvas>) -> &RenderCanvas {
        // SAFETY: from_source rejects null and this handle owns the rcp retain.
        unsafe { &*source.get() }
    }
}
