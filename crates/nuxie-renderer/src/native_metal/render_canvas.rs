//! Guarded product owner for the exact mechanically translated RenderCanvas.

use std::any::Any;
use std::cell::RefCell;
use std::pin::Pin;
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
#[cfg(test)]
use crate::RendererError;
use crate::exact_source_adapter::ExactSourceRendererAdapter;
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::FrameDescriptor;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RenderResourceDomain;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImageHandle;
#[cfg(feature = "native-ore-metal-experimental")]
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
#[cfg(feature = "native-ore-metal-experimental")]
use nuxie_ore_metal::metal::context::ContextMetal as OreContextMetal;
use nuxie_render_api::{
    BlendMode, ColorInt, ImageSampler, Mat2D, RenderBuffer, RenderCanvas as RenderCanvasContract,
    RenderCanvasError, RenderCanvasFrame, RenderImage, RenderPaint, RenderPath, Renderer,
};

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
    #[cfg(feature = "native-ore-metal-experimental")]
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

impl RenderCanvasContract for NativeMetalRenderCanvas {
    fn width(&self) -> u32 {
        NativeMetalRenderCanvas::width(self)
    }

    fn height(&self) -> u32 {
        NativeMetalRenderCanvas::height(self)
    }

    fn render_image(&self) -> Rc<dyn RenderImage> {
        Rc::from(NativeMetalRenderCanvas::render_image(self))
    }

    fn begin_frame(
        &mut self,
        clear_color: ColorInt,
    ) -> Result<Box<dyn RenderCanvasFrame>, RenderCanvasError> {
        let (width, height) = (self.width(), self.height());
        let (source, resource_domain, execution_guard) = match &self.inner {
            NativeMetalRenderCanvasInner::Guarded {
                source,
                resource_domain,
                execution_guard,
                ..
            } => (source, resource_domain, execution_guard),
            #[cfg(test)]
            NativeMetalRenderCanvasInner::Allocated { .. } => {
                return Err(RenderCanvasError::new(
                    "private RenderCanvas allocation staging cannot begin a frame",
                ));
            }
        };
        let renderer = {
            let mut mechanical = execution_guard.borrow_mut();
            let context = unsafe { Pin::get_unchecked_mut(mechanical.render_context_mut()) };
            context.beginFrameExecutable(&FrameDescriptor {
                renderTargetWidth: width,
                renderTargetHeight: height,
                clearColor: clear_color,
                ..FrameDescriptor::default()
            });
            unsafe { ExactSourceRendererAdapter::new(context, resource_domain.clone()) }
        };
        Ok(Box::new(NativeMetalRenderCanvasFrame {
            renderer,
            source: rcp::copy_ctor(source),
            execution_guard: Rc::clone(execution_guard),
        }))
    }
}

struct NativeMetalRenderCanvasFrame {
    renderer: ExactSourceRendererAdapter,
    source: rcp<RenderCanvas>,
    execution_guard: Rc<RefCell<MechanicalRenderContext>>,
}

impl RenderCanvasFrame for NativeMetalRenderCanvasFrame {
    fn renderer(&mut self) -> &mut dyn Renderer {
        self
    }

    fn finish(self: Box<Self>) -> Result<(), RenderCanvasError> {
        let mut mechanical = self.execution_guard.borrow_mut();
        let context = unsafe { Pin::get_unchecked_mut(mechanical.render_context_mut()) };
        // SAFETY: the canvas frame retains this nonnull exact source owner and
        // is the sole owner allowed to finish its begun frame.
        context.finishRenderCanvasExecutable(unsafe { &mut *self.source.get() });
        Ok(())
    }
}

impl Renderer for NativeMetalRenderCanvasFrame {
    fn save(&mut self) {
        self.renderer.save();
    }

    fn restore(&mut self) {
        self.renderer.restore();
    }

    fn transform(&mut self, transform: Mat2D) {
        self.renderer.transform(transform);
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        self.renderer.draw_path(path, paint);
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        self.renderer.clip_path(path);
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        self.renderer
            .draw_image(image, sampler, blend_mode, opacity);
    }

    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        self.renderer.draw_image_mesh(
            image,
            sampler,
            vertices,
            uv_coords,
            indices,
            vertex_count,
            index_count,
            blend_mode,
            opacity,
        );
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.renderer.modulate_opacity(opacity);
    }
}
