//! Opaque Metal presentation mechanics for Apple platform adapters.
//!
//! The adapter owns drawable validation, lifecycle, presentation scheduling,
//! completion policy, and error mapping. This module keeps the renderer's raw
//! wgpu device, queue, texture wrapper, and final-blit pipeline private.

use super::present_pipeline::{PresentPipeline, PresentTargetAlpha};
use super::{
    Context, RendererError, WgpuDeviceFailure, WgpuDeviceFailureKind, WgpuFactory, WgpuFrame,
    WgpuFrameMetrics,
};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLCommandQueue, MTLDevice, MTLPixelFormat, MTLResource, MTLTexture, MTLTextureType,
};
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::Arc;

/// Device-health state observable by an external presentation adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WgpuDeviceHealth {
    Healthy,
    DeviceLost,
    OutOfMemory,
    Failed(String),
}

/// Failure classes that an external platform command buffer can report back
/// into the renderer's shared device-health state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuExternalDeviceFailureKind {
    DeviceLost,
    OutOfMemory,
    Internal,
}

/// Metal-capable final-blit mechanics with all raw wgpu state hidden.
#[derive(Clone)]
pub struct WgpuMetalPresenter {
    context: Arc<Context>,
    presenter: Arc<PresentPipeline>,
}

impl WgpuFactory {
    /// Creates an opaque Metal presenter for this renderer domain.
    pub fn create_metal_presenter(&self) -> Result<WgpuMetalPresenter, RendererError> {
        WgpuMetalPresenter::new(Arc::clone(&self.context))
    }
}

impl WgpuMetalPresenter {
    fn new(context: Arc<Context>) -> Result<Self, RendererError> {
        metal_device(&context)?;
        metal_queue(&context)?;
        let presenter = Arc::new(PresentPipeline::new(
            &context.device,
            wgpu::TextureFormat::Bgra8Unorm,
            PresentTargetAlpha::Straight,
        ));
        Ok(Self { context, presenter })
    }

    /// Copies the renderer's `MTLDevice` with Objective-C +1 ownership.
    /// The caller must transfer that ownership to ARC or release it.
    pub fn copy_device(&self) -> Result<*mut c_void, RendererError> {
        Ok(Retained::into_raw(metal_device(&self.context)?).cast())
    }

    /// Copies the renderer's ordered `MTLCommandQueue` with Objective-C +1
    /// ownership. Presentation work submitted to this queue follows the
    /// renderer's final blit.
    pub fn copy_command_queue(&self) -> Result<*mut c_void, RendererError> {
        Ok(Retained::into_raw(metal_queue(&self.context)?).cast())
    }

    /// Returns the renderer domain's current device-health state.
    pub fn device_health(&self) -> WgpuDeviceHealth {
        match self.context.device_health.current() {
            None => WgpuDeviceHealth::Healthy,
            Some(failure) => match failure.kind {
                WgpuDeviceFailureKind::DeviceLost => WgpuDeviceHealth::DeviceLost,
                WgpuDeviceFailureKind::OutOfMemory => WgpuDeviceHealth::OutOfMemory,
                WgpuDeviceFailureKind::Validation | WgpuDeviceFailureKind::Internal => {
                    WgpuDeviceHealth::Failed(failure.message)
                }
            },
        }
    }

    /// Records a failure observed by platform presentation work submitted on
    /// the renderer's Metal queue.
    pub fn record_external_failure(&self, kind: WgpuExternalDeviceFailureKind, message: String) {
        self.context.device_health.record(WgpuDeviceFailure {
            kind: match kind {
                WgpuExternalDeviceFailureKind::DeviceLost => WgpuDeviceFailureKind::DeviceLost,
                WgpuExternalDeviceFailureKind::OutOfMemory => WgpuDeviceFailureKind::OutOfMemory,
                WgpuExternalDeviceFailureKind::Internal => WgpuDeviceFailureKind::Internal,
            },
            message,
        });
    }

    /// Finishes a renderer frame into one validated `id<MTLTexture>`.
    ///
    /// # Safety
    ///
    /// `texture` must be a live `id<MTLTexture>` retained by the caller until
    /// this synchronous method returns.
    pub unsafe fn render_to_texture(
        &self,
        frame: WgpuFrame,
        texture: *mut c_void,
        width: u32,
        height: u32,
    ) -> Result<WgpuFrameMetrics, RendererError> {
        if !Arc::ptr_eq(&self.context, &frame.context) {
            return Err(RendererError::Device(
                "Metal presenter belongs to a different WebGPU renderer".into(),
            ));
        }
        let texture = NonNull::new(texture).ok_or_else(|| {
            RendererError::Device("Metal presentation texture pointer is null".into())
        })?;
        let raw_texture = unsafe { texture.cast::<ProtocolObject<dyn MTLTexture>>().as_ref() };
        let actual_width = u32::try_from(raw_texture.width()).map_err(|_| {
            RendererError::Device("Metal presentation texture width exceeds UInt32".into())
        })?;
        let actual_height = u32::try_from(raw_texture.height()).map_err(|_| {
            RendererError::Device("Metal presentation texture height exceeds UInt32".into())
        })?;
        if (actual_width, actual_height) != (width, height) {
            return Err(RendererError::Device(format!(
                "Metal presentation texture is {actual_width}x{actual_height}, expected {width}x{height}"
            )));
        }
        if raw_texture.pixelFormat() != MTLPixelFormat::BGRA8Unorm {
            return Err(RendererError::Device(
                "Metal presentation texture format is not BGRA8Unorm".into(),
            ));
        }
        let renderer_device = metal_device(&self.context)?;
        if Retained::as_ptr(&raw_texture.device()) != Retained::as_ptr(&renderer_device) {
            return Err(RendererError::Device(
                "Metal presentation texture belongs to a different MTLDevice".into(),
            ));
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let hal_texture = unsafe {
            wgpu::hal::metal::Device::texture_from_raw(
                raw_texture.into(),
                wgpu::TextureFormat::Bgra8Unorm,
                MTLTextureType::Type2D,
                1,
                1,
                size.into(),
                None,
            )
        };
        let descriptor = wgpu::TextureDescriptor {
            label: Some("nuxie-metal-presentation-target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let texture = unsafe {
            self.context
                .device
                .create_texture_from_hal::<wgpu::hal::api::Metal>(
                    hal_texture,
                    &descriptor,
                    wgpu::TextureUses::UNINITIALIZED,
                )
        };
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        frame.finish_to_texture_view(&view, &self.presenter)
    }
}

fn metal_device(
    context: &Context,
) -> Result<Retained<ProtocolObject<dyn MTLDevice>>, RendererError> {
    let device = unsafe { context.device.as_hal::<wgpu::hal::api::Metal>() }
        .ok_or_else(|| RendererError::Unsupported("renderer is not using Metal"))?;
    Ok(device.raw_device().clone())
}

fn metal_queue(
    context: &Context,
) -> Result<Retained<ProtocolObject<dyn MTLCommandQueue>>, RendererError> {
    let queue = unsafe { context.queue.as_hal::<wgpu::hal::api::Metal>() }
        .ok_or_else(|| RendererError::Unsupported("renderer is not using Metal"))?;
    let pointer = std::ptr::from_ref(queue.as_raw()).cast_mut();
    unsafe { Retained::retain(pointer) }
        .ok_or_else(|| RendererError::Device("Metal command queue pointer is null".into()))
}
