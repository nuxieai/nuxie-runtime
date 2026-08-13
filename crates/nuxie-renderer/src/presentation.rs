//! Opaque WebGPU presentation mechanics for platform adapters.
//!
//! Platform packages own their window/canvas, resize ordering, acquisition
//! recovery, and frame lifecycle. This module keeps the renderer's raw wgpu
//! device, queue, surface, acquired target, and any platform-specific final
//! blit pipeline private.

#[cfg(not(target_arch = "wasm32"))]
use super::present_pipeline::{PresentPipeline, PresentTargetAlpha};
use super::{Context, RendererError, WgpuFrame, WgpuFrameMetrics};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

/// Alpha representation expected by a platform presentation target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuPresentationAlpha {
    Straight,
    Premultiplied,
}

/// Typed result of one platform-surface acquisition attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuPresentationAcquireError {
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
}

/// Configured platform presentation surface with all raw wgpu state hidden.
pub struct WgpuPresentationSurface {
    context: Arc<Context>,
    surface: wgpu::Surface<'static>,
    configuration: wgpu::SurfaceConfiguration,
    #[cfg(not(target_arch = "wasm32"))]
    presenter: Arc<PresentPipeline>,
}

/// A single acquired platform frame. It is linear and cannot be cloned or
/// presented more than once.
pub struct WgpuPresentationFrame {
    context: Arc<Context>,
    texture: wgpu::SurfaceTexture,
    #[cfg(not(target_arch = "wasm32"))]
    presenter: Arc<PresentPipeline>,
}

impl super::WgpuFactory {
    /// Attaches an owned platform target to this renderer without exposing
    /// any wgpu state to the platform adapter.
    pub fn create_presentation_surface<T>(
        &self,
        target: T,
        width: u32,
        height: u32,
        alpha: WgpuPresentationAlpha,
    ) -> Result<WgpuPresentationSurface, RendererError>
    where
        T: HasWindowHandle + HasDisplayHandle + 'static,
    {
        WgpuPresentationSurface::new(Arc::clone(&self.context), target, width, height, alpha)
    }
}

impl WgpuPresentationSurface {
    fn new<T>(
        context: Arc<Context>,
        target: T,
        width: u32,
        height: u32,
        alpha: WgpuPresentationAlpha,
    ) -> Result<Self, RendererError>
    where
        T: HasWindowHandle + HasDisplayHandle + 'static,
    {
        let surface = create_surface(&context, target, "creation")?;
        let mut configuration = surface
            .get_default_config(&context.adapter, width, height)
            .ok_or_else(|| {
                RendererError::Adapter(
                    "selected WebGPU adapter cannot present to the platform target".into(),
                )
            })?;
        configuration.alpha_mode = match alpha {
            WgpuPresentationAlpha::Straight => wgpu::CompositeAlphaMode::Auto,
            WgpuPresentationAlpha::Premultiplied => wgpu::CompositeAlphaMode::PreMultiplied,
        };
        #[cfg(target_arch = "wasm32")]
        {
            let capabilities = surface.get_capabilities(&context.adapter);
            if !capabilities
                .formats
                .contains(&wgpu::TextureFormat::Rgba8Unorm)
            {
                return Err(RendererError::Adapter(
                    "browser WebGPU surface does not support Rgba8Unorm".into(),
                ));
            }
            // Upstream Rive's WebGPU host binds the acquired RGBA8 texture as
            // the renderer target. CopySrc is needed by advanced blending;
            // TextureBinding retains the upstream surface contract exactly.
            configuration.format = wgpu::TextureFormat::Rgba8Unorm;
            configuration.usage = wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING;
        }
        surface.configure(&context.device, &configuration);
        #[cfg(not(target_arch = "wasm32"))]
        let presenter = Arc::new(PresentPipeline::new(
            &context.device,
            configuration.format,
            match alpha {
                WgpuPresentationAlpha::Straight => PresentTargetAlpha::Straight,
                WgpuPresentationAlpha::Premultiplied => PresentTargetAlpha::Premultiplied,
            },
        ));
        Ok(Self {
            context,
            surface,
            configuration,
            #[cfg(not(target_arch = "wasm32"))]
            presenter,
        })
    }

    /// Reconfigures the target for a new physical extent.
    pub fn configure(&mut self, width: u32, height: u32) {
        self.configuration.width = width;
        self.configuration.height = height;
        self.surface
            .configure(&self.context.device, &self.configuration);
    }

    /// Recreates the raw surface from an owned platform target while
    /// preserving the selected format, extent, and alpha contract.
    pub fn recreate<T>(&mut self, target: T) -> Result<(), RendererError>
    where
        T: HasWindowHandle + HasDisplayHandle + 'static,
    {
        let surface = create_surface(&self.context, target, "recreation")?;
        surface.configure(&self.context.device, &self.configuration);
        self.surface = surface;
        Ok(())
    }

    /// Attempts to acquire one platform frame without applying recovery
    /// policy. The owning adapter decides whether and how often to retry.
    pub fn acquire(&self) -> Result<WgpuPresentationFrame, WgpuPresentationAcquireError> {
        let texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout => {
                return Err(WgpuPresentationAcquireError::Timeout)
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Err(WgpuPresentationAcquireError::Occluded)
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                return Err(WgpuPresentationAcquireError::Outdated)
            }
            wgpu::CurrentSurfaceTexture::Lost => return Err(WgpuPresentationAcquireError::Lost),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(WgpuPresentationAcquireError::Validation)
            }
        };
        Ok(WgpuPresentationFrame {
            context: Arc::clone(&self.context),
            texture,
            #[cfg(not(target_arch = "wasm32"))]
            presenter: Arc::clone(&self.presenter),
        })
    }
}

impl WgpuPresentationFrame {
    /// Finishes a renderer frame into this acquired target and presents it.
    pub async fn present(self, frame: WgpuFrame) -> Result<WgpuFrameMetrics, RendererError> {
        if !Arc::ptr_eq(&self.context, &frame.context) {
            return Err(RendererError::Device(
                "presentation frame belongs to a different WebGPU renderer".into(),
            ));
        }
        let view = self
            .texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        #[cfg(target_arch = "wasm32")]
        let metrics = frame
            .finish_to_surface_texture_async(&self.texture.texture, &view)
            .await?;
        #[cfg(not(target_arch = "wasm32"))]
        let metrics = frame
            .finish_to_texture_view_async(&view, &self.presenter)
            .await?;
        self.context.queue.present(self.texture);
        Ok(metrics)
    }
}

fn create_surface<T>(
    context: &Context,
    target: T,
    operation: &'static str,
) -> Result<wgpu::Surface<'static>, RendererError>
where
    T: HasWindowHandle + HasDisplayHandle + 'static,
{
    context.instance.create_surface(target).map_err(|error| {
        RendererError::Device(format!(
            "WebGPU presentation surface {operation} failed: {error}"
        ))
    })
}
