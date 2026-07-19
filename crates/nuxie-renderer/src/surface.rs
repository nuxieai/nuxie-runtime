//! Apple drawable presentation for the retained WebGPU renderer.

use super::{RenderMode, RendererError, WgpuFactory, WgpuFrame, WgpuFrameMetrics};
use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLCommandBuffer, MTLCommandBufferError, MTLCommandBufferStatus, MTLCommandQueue, MTLDevice,
    MTLDrawable, MTLPixelFormat, MTLResource, MTLTexture, MTLTextureType,
};
use objc2_quartz_core::CAMetalDrawable;
#[cfg(test)]
use objc2_quartz_core::CAMetalLayer;
use std::ffi::c_void;
use std::fmt;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

type PresentationCallback = Box<dyn FnOnce() + Send + 'static>;

struct PresentationCompletionState {
    callback: Mutex<Option<PresentationCallback>>,
}

impl PresentationCompletionState {
    fn complete(&self) {
        let callback = {
            let mut callback = match self.callback.lock() {
                Ok(callback) => callback,
                Err(poisoned) => poisoned.into_inner(),
            };
            callback.take()
        };
        if let Some(callback) = callback {
            callback();
        }
    }
}

/// One completion that fires exactly once after Metal finishes using a
/// drawable, or immediately when presentation cannot be scheduled.
pub struct ApplePresentationCompletion {
    state: Arc<PresentationCompletionState>,
    armed: bool,
}

impl ApplePresentationCompletion {
    pub fn new(callback: impl FnOnce() + Send + 'static) -> Self {
        Self {
            state: Arc::new(PresentationCompletionState {
                callback: Mutex::new(Some(Box::new(callback))),
            }),
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for ApplePresentationCompletion {
    fn drop(&mut self) {
        if self.armed {
            self.state.complete();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SurfaceDisposition {
    None = 0,
    Presented = 1,
    SkippedZeroSize = 2,
    SkippedTimeout = 3,
    SkippedOccluded = 4,
    Reconfigured = 5,
    Recreated = 6,
    DeviceLost = 7,
    OutOfMemory = 8,
    Fatal = 9,
}

#[derive(Debug)]
pub enum SurfaceError {
    NullDrawable,
    InvalidDrawable(String),
    Unsupported(&'static str),
    Presentation(&'static str),
    Renderer(RendererError),
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullDrawable => formatter.write_str("CAMetalDrawable pointer is null"),
            Self::InvalidDrawable(message) => {
                write!(formatter, "invalid Apple drawable: {message}")
            }
            Self::Unsupported(message) => write!(formatter, "unsupported Apple surface: {message}"),
            Self::Presentation(message) => {
                write!(formatter, "failed to present Apple drawable: {message}")
            }
            Self::Renderer(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for SurfaceError {}

impl From<RendererError> for SurfaceError {
    fn from(error: RendererError) -> Self {
        Self::Renderer(error)
    }
}

pub struct AppleSurface {
    presenter: PresentPipeline,
    width: u32,
    height: u32,
    attached: bool,
}

impl AppleSurface {
    /// Creates the first renderer domain without touching UIKit-owned state.
    /// Swift configures its `CAMetalLayer` with [`Self::copy_metal_device`] and
    /// acquires each drawable on the main actor.
    pub fn attach_with_factory(
        width: u32,
        height: u32,
        mode: RenderMode,
    ) -> Result<(WgpuFactory, Self), SurfaceError> {
        let mut factory = WgpuFactory::new_with_mode(width.max(1), height.max(1), mode)?;
        let surface = Self::attach(&mut factory, width, height)?;
        Ok((factory, surface))
    }

    /// Creates logical presentation state for a shared renderer domain.
    pub fn attach(
        factory: &mut WgpuFactory,
        width: u32,
        height: u32,
    ) -> Result<Self, SurfaceError> {
        if width != 0 && height != 0 {
            factory.resize(width, height)?;
        }
        Ok(Self {
            presenter: PresentPipeline::new(
                &factory.context.device,
                wgpu::TextureFormat::Bgra8Unorm,
            ),
            width,
            height,
            attached: true,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn is_attached(&self) -> bool {
        self.attached
    }

    pub fn resize(
        &mut self,
        factory: &mut WgpuFactory,
        width: u32,
        height: u32,
    ) -> Result<SurfaceDisposition, SurfaceError> {
        self.configure_dimensions(factory, width, height)?;
        Ok(if width == 0 || height == 0 {
            SurfaceDisposition::SkippedZeroSize
        } else {
            SurfaceDisposition::Reconfigured
        })
    }

    pub fn detach(&mut self) {
        self.attached = false;
    }

    pub fn reattach(
        &mut self,
        factory: &mut WgpuFactory,
        width: u32,
        height: u32,
    ) -> Result<SurfaceDisposition, SurfaceError> {
        self.configure_dimensions(factory, width, height)?;
        self.attached = true;
        Ok(if width == 0 || height == 0 {
            SurfaceDisposition::SkippedZeroSize
        } else {
            SurfaceDisposition::Recreated
        })
    }

    /// Copies the renderer's `MTLDevice` with Objective-C +1 ownership.
    /// The caller must transfer that ownership to ARC or release it.
    pub fn copy_metal_device(&self, factory: &WgpuFactory) -> Result<*mut c_void, SurfaceError> {
        let device = metal_device(factory)?;
        Ok(Retained::into_raw(device).cast())
    }

    /// Renders and schedules presentation into a main-actor-acquired drawable.
    /// A null drawable is the bounded no-drawable outcome, not an error.
    ///
    /// # Safety
    ///
    /// A non-null pointer must be a live `id<CAMetalDrawable>` retained by the
    /// caller until this synchronous method returns.
    pub unsafe fn present(
        &mut self,
        factory: &mut WgpuFactory,
        frame: WgpuFrame,
        drawable: *mut c_void,
        completion: Option<ApplePresentationCompletion>,
    ) -> Result<(SurfaceDisposition, WgpuFrameMetrics), SurfaceError> {
        let mut completion = completion;
        if let Some(disposition) = device_failure_disposition(factory)? {
            return Ok((disposition, frame.metrics()));
        }
        if !self.attached {
            return Err(SurfaceError::Unsupported("surface is not attached"));
        }
        if self.width == 0 || self.height == 0 {
            return Ok((SurfaceDisposition::SkippedZeroSize, frame.metrics()));
        }
        let Some(drawable) = NonNull::new(drawable) else {
            return Ok((SurfaceDisposition::SkippedTimeout, frame.metrics()));
        };
        let drawable = unsafe {
            drawable
                .cast::<ProtocolObject<dyn CAMetalDrawable>>()
                .as_ref()
        };
        let texture = wrap_drawable_texture(factory, drawable, self.width, self.height)?;
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let metrics = frame.finish_to_texture_view(&view, &self.presenter)?;
        schedule_drawable_presentation(factory, drawable, completion.as_mut())?;
        if let Some(disposition) = device_failure_disposition(factory)? {
            return Ok((disposition, metrics));
        }
        Ok((SurfaceDisposition::Presented, metrics))
    }

    fn configure_dimensions(
        &mut self,
        factory: &mut WgpuFactory,
        width: u32,
        height: u32,
    ) -> Result<(), SurfaceError> {
        if width != 0 && height != 0 {
            factory.resize(width, height)?;
        }
        self.width = width;
        self.height = height;
        Ok(())
    }
}

fn device_failure_disposition(
    factory: &WgpuFactory,
) -> Result<Option<SurfaceDisposition>, SurfaceError> {
    let Some(failure) = factory.context.device_health.current() else {
        return Ok(None);
    };
    Ok(match failure.kind {
        super::WgpuDeviceFailureKind::DeviceLost => Some(SurfaceDisposition::DeviceLost),
        super::WgpuDeviceFailureKind::OutOfMemory => Some(SurfaceDisposition::OutOfMemory),
        super::WgpuDeviceFailureKind::Validation | super::WgpuDeviceFailureKind::Internal => {
            return Err(SurfaceError::Renderer(RendererError::Device(
                failure.message,
            )));
        }
    })
}

fn metal_device(
    factory: &WgpuFactory,
) -> Result<Retained<ProtocolObject<dyn MTLDevice>>, SurfaceError> {
    let device = unsafe { factory.context.device.as_hal::<wgpu::hal::api::Metal>() }
        .ok_or(SurfaceError::Unsupported("renderer is not using Metal"))?;
    Ok(device.raw_device().clone())
}

fn wrap_drawable_texture(
    factory: &WgpuFactory,
    drawable: &ProtocolObject<dyn CAMetalDrawable>,
    expected_width: u32,
    expected_height: u32,
) -> Result<wgpu::Texture, SurfaceError> {
    let raw_texture = drawable.texture();
    let width = u32::try_from(raw_texture.width())
        .map_err(|_| SurfaceError::InvalidDrawable("width exceeds UInt32".to_owned()))?;
    let height = u32::try_from(raw_texture.height())
        .map_err(|_| SurfaceError::InvalidDrawable("height exceeds UInt32".to_owned()))?;
    if (width, height) != (expected_width, expected_height) {
        return Err(SurfaceError::InvalidDrawable(format!(
            "texture is {width}x{height}, expected {expected_width}x{expected_height}"
        )));
    }
    if raw_texture.pixelFormat() != MTLPixelFormat::BGRA8Unorm {
        return Err(SurfaceError::InvalidDrawable(
            "texture format is not BGRA8Unorm".to_owned(),
        ));
    }
    let drawable_device = raw_texture.device();
    let renderer_device = metal_device(factory)?;
    if Retained::as_ptr(&drawable_device) != Retained::as_ptr(&renderer_device) {
        return Err(SurfaceError::InvalidDrawable(
            "texture belongs to a different MTLDevice".to_owned(),
        ));
    }

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let hal_texture = unsafe {
        wgpu::hal::metal::Device::texture_from_raw(
            raw_texture,
            wgpu::TextureFormat::Bgra8Unorm,
            MTLTextureType::Type2D,
            1,
            1,
            size.into(),
            None,
        )
    };
    let descriptor = wgpu::TextureDescriptor {
        label: Some("nuxie-apple-drawable"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    Ok(unsafe {
        factory
            .context
            .device
            .create_texture_from_hal::<wgpu::hal::api::Metal>(
                hal_texture,
                &descriptor,
                wgpu::TextureUses::UNINITIALIZED,
            )
    })
}

fn schedule_drawable_presentation(
    factory: &WgpuFactory,
    drawable: &ProtocolObject<dyn CAMetalDrawable>,
    completion: Option<&mut ApplePresentationCompletion>,
) -> Result<(), SurfaceError> {
    let queue = unsafe { factory.context.queue.as_hal::<wgpu::hal::api::Metal>() }
        .ok_or(SurfaceError::Unsupported("renderer is not using Metal"))?;
    let command_buffer = queue
        .as_raw()
        .commandBuffer()
        .ok_or(SurfaceError::Presentation(
            "MTLCommandQueue returned no command buffer",
        ))?;
    let drawable: &ProtocolObject<dyn MTLDrawable> = drawable.as_ref();
    command_buffer.presentDrawable(drawable);
    let completion_state = completion
        .as_ref()
        .map(|completion| Arc::clone(&completion.state));
    let device_health = Arc::clone(&factory.context.device_health);
    let completed_handler = RcBlock::new(
        move |command_buffer: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
            let command_buffer = unsafe { command_buffer.as_ref() };
            if command_buffer.status() == MTLCommandBufferStatus::Error {
                let error_code = command_buffer.error().map(|error| error.code());
                device_health.record(super::WgpuDeviceFailure {
                    kind: metal_failure_kind(error_code),
                    message: match error_code {
                        Some(code) => {
                            format!("Metal presentation command buffer failed with code {code}")
                        }
                        None => {
                            "Metal presentation command buffer failed without an NSError".to_owned()
                        }
                    },
                });
            }
            if let Some(completion_state) = &completion_state {
                completion_state.complete();
            }
        },
    );
    unsafe {
        command_buffer.addCompletedHandler(RcBlock::as_ptr(&completed_handler));
    }
    command_buffer.commit();
    if let Some(completion) = completion {
        completion.disarm();
    }
    Ok(())
}

fn metal_failure_kind(error_code: Option<isize>) -> super::WgpuDeviceFailureKind {
    match error_code.and_then(|code| usize::try_from(code).ok()) {
        Some(code) if code == MTLCommandBufferError::OutOfMemory.0 => {
            super::WgpuDeviceFailureKind::OutOfMemory
        }
        Some(code) if code == MTLCommandBufferError::DeviceRemoved.0 => {
            super::WgpuDeviceFailureKind::DeviceLost
        }
        _ => super::WgpuDeviceFailureKind::Internal,
    }
}

pub(crate) struct PresentPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl PresentPipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-surface-present-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("surface_present.wgsl").into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-surface-present-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-surface-present-pipeline-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-surface-present-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nuxie-surface-present-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            pipeline,
            layout,
            sampler,
        }
    }

    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        source: &wgpu::TextureView,
    ) {
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-surface-present-group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("nuxie-surface-present-pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2::rc::{autoreleasepool, Retained};
    use objc2_core_foundation::CGSize;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn configured_layer(
        surface: &AppleSurface,
        factory: &WgpuFactory,
        width: u32,
        height: u32,
    ) -> Retained<CAMetalLayer> {
        let device_pointer = surface
            .copy_metal_device(factory)
            .expect("Metal renderer must expose its device");
        let device: Retained<ProtocolObject<dyn MTLDevice>> = unsafe {
            Retained::from_raw(device_pointer.cast()).expect("copied Metal device must be non-null")
        };
        let layer = CAMetalLayer::new();
        layer.setDevice(Some(&device));
        layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        layer.setFramebufferOnly(true);
        layer.setDrawableSize(CGSize::new(width.into(), height.into()));
        layer.setMaximumDrawableCount(2);
        layer.setAllowsNextDrawableTimeout(true);
        layer
    }

    fn wait_for_metal_queue(factory: &WgpuFactory) {
        let queue = unsafe { factory.context.queue.as_hal::<wgpu::hal::api::Metal>() }
            .expect("renderer must use Metal");
        let command_buffer = queue
            .as_raw()
            .commandBuffer()
            .expect("Metal queue must return a command buffer");
        command_buffer.commit();
        command_buffer.waitUntilCompleted();
    }

    #[test]
    fn surface_dispositions_are_stable_fixed_width_values() {
        assert_eq!(SurfaceDisposition::Presented as u8, 1);
        assert_eq!(SurfaceDisposition::SkippedTimeout as u8, 3);
        assert_eq!(SurfaceDisposition::Recreated as u8, 6);
        assert_eq!(SurfaceDisposition::Fatal as u8, 9);
    }

    #[test]
    fn logical_surface_lifecycle_and_resize_are_transactional() {
        let (mut factory, mut surface) =
            AppleSurface::attach_with_factory(8, 8, RenderMode::Msaa).unwrap();
        assert!(surface.is_attached());
        assert_eq!(surface.dimensions(), (8, 8));
        assert_eq!(factory.dimensions(), (8, 8));

        assert_eq!(
            surface.resize(&mut factory, 0, 0).unwrap(),
            SurfaceDisposition::SkippedZeroSize
        );
        assert_eq!(surface.dimensions(), (0, 0));
        assert_eq!(factory.dimensions(), (8, 8));

        assert_eq!(
            surface.resize(&mut factory, 16, 12).unwrap(),
            SurfaceDisposition::Reconfigured
        );
        assert_eq!(surface.dimensions(), (16, 12));
        assert_eq!(factory.dimensions(), (16, 12));

        assert!(matches!(
            surface.resize(&mut factory, u32::MAX, 12),
            Err(SurfaceError::Renderer(
                RendererError::InvalidTextureExtent { .. }
            ))
        ));
        assert_eq!(surface.dimensions(), (16, 12));
        assert_eq!(factory.dimensions(), (16, 12));

        surface.detach();
        assert!(!surface.is_attached());
        assert!(matches!(
            surface.reattach(&mut factory, u32::MAX, 12),
            Err(SurfaceError::Renderer(
                RendererError::InvalidTextureExtent { .. }
            ))
        ));
        assert!(!surface.is_attached());
        assert_eq!(surface.dimensions(), (16, 12));
        assert_eq!(factory.dimensions(), (16, 12));

        assert_eq!(
            surface.reattach(&mut factory, 10, 6).unwrap(),
            SurfaceDisposition::Recreated
        );
        assert!(surface.is_attached());
        assert_eq!(surface.dimensions(), (10, 6));
        assert_eq!(factory.dimensions(), (10, 6));
    }

    #[test]
    fn null_drawable_is_a_bounded_timeout_outcome() {
        let (mut factory, mut surface) =
            AppleSurface::attach_with_factory(2, 2, RenderMode::Msaa).unwrap();
        let frame = factory.begin_frame(0x0000_0000);

        let (disposition, _) = unsafe {
            surface
                .present(&mut factory, frame, std::ptr::null_mut(), None)
                .unwrap()
        };

        assert_eq!(disposition, SurfaceDisposition::SkippedTimeout);
    }

    #[test]
    fn detached_surface_rejects_present_before_inspecting_the_drawable() {
        let (mut factory, mut surface) =
            AppleSurface::attach_with_factory(2, 2, RenderMode::Msaa).unwrap();
        surface.detach();
        let frame = factory.begin_frame(0x0000_0000);

        assert!(matches!(
            unsafe { surface.present(&mut factory, frame, std::ptr::null_mut(), None) },
            Err(SurfaceError::Unsupported("surface is not attached"))
        ));
    }

    #[test]
    fn recorded_device_failures_become_structured_surface_outcomes() {
        for (kind, expected) in [
            (
                super::super::WgpuDeviceFailureKind::DeviceLost,
                SurfaceDisposition::DeviceLost,
            ),
            (
                super::super::WgpuDeviceFailureKind::OutOfMemory,
                SurfaceDisposition::OutOfMemory,
            ),
        ] {
            let (mut factory, mut surface) =
                AppleSurface::attach_with_factory(2, 2, RenderMode::Msaa).unwrap();
            let frame = factory.begin_frame(0x0000_0000);
            factory
                .context
                .device_health
                .record(super::super::WgpuDeviceFailure {
                    kind,
                    message: "injected device failure".to_owned(),
                });

            let (disposition, _) = unsafe {
                surface
                    .present(&mut factory, frame, std::ptr::null_mut(), None)
                    .unwrap()
            };
            assert_eq!(disposition, expected);
        }
    }

    #[test]
    fn recorded_validation_error_is_returned_instead_of_panicking() {
        let (mut factory, mut surface) =
            AppleSurface::attach_with_factory(2, 2, RenderMode::Msaa).unwrap();
        let frame = factory.begin_frame(0x0000_0000);
        factory
            .context
            .device_health
            .record(super::super::WgpuDeviceFailure {
                kind: super::super::WgpuDeviceFailureKind::Validation,
                message: "injected validation error".to_owned(),
            });

        assert!(matches!(
            unsafe { surface.present(&mut factory, frame, std::ptr::null_mut(), None) },
            Err(SurfaceError::Renderer(RendererError::Device(message)))
                if message == "injected validation error"
        ));
    }

    #[test]
    fn configured_cametal_layer_drawable_is_rendered_and_scheduled_for_presentation() {
        autoreleasepool(|_| {
            let (mut factory, mut surface) =
                AppleSurface::attach_with_factory(4, 3, RenderMode::Msaa).unwrap();
            let layer = configured_layer(&surface, &factory, 4, 3);
            let drawable = layer
                .nextDrawable()
                .expect("configured CAMetalLayer must vend a drawable");
            assert_eq!(drawable.texture().width(), 4);
            assert_eq!(drawable.texture().height(), 3);
            assert_eq!(drawable.texture().pixelFormat(), MTLPixelFormat::BGRA8Unorm);
            let drawable_pointer = Retained::as_ptr(&drawable).cast_mut().cast::<c_void>();
            let completed = Arc::new(AtomicBool::new(false));
            let completed_for_callback = Arc::clone(&completed);

            let frame = factory.begin_frame(0xff11_2233);
            let (disposition, _) = unsafe {
                surface
                    .present(
                        &mut factory,
                        frame,
                        drawable_pointer,
                        Some(ApplePresentationCompletion::new(move || {
                            completed_for_callback.store(true, Ordering::Release);
                        })),
                    )
                    .unwrap()
            };
            wait_for_metal_queue(&factory);

            assert_eq!(disposition, SurfaceDisposition::Presented);
            assert!(completed.load(Ordering::Acquire));
        });
    }

    #[test]
    fn completion_fires_when_presentation_is_skipped_before_submission() {
        let (mut factory, mut surface) =
            AppleSurface::attach_with_factory(2, 2, RenderMode::Msaa).unwrap();
        let completed = Arc::new(AtomicBool::new(false));
        let completed_for_callback = Arc::clone(&completed);
        let frame = factory.begin_frame(0x0000_0000);

        let (disposition, _) = unsafe {
            surface
                .present(
                    &mut factory,
                    frame,
                    std::ptr::null_mut(),
                    Some(ApplePresentationCompletion::new(move || {
                        completed_for_callback.store(true, Ordering::Release);
                    })),
                )
                .unwrap()
        };

        assert_eq!(disposition, SurfaceDisposition::SkippedTimeout);
        assert!(completed.load(Ordering::Acquire));
    }

    #[test]
    fn metal_completion_errors_map_to_structured_device_health() {
        assert_eq!(
            metal_failure_kind(Some(MTLCommandBufferError::OutOfMemory.0 as isize)),
            super::super::WgpuDeviceFailureKind::OutOfMemory
        );
        assert_eq!(
            metal_failure_kind(Some(MTLCommandBufferError::DeviceRemoved.0 as isize)),
            super::super::WgpuDeviceFailureKind::DeviceLost
        );
        assert_eq!(
            metal_failure_kind(Some(MTLCommandBufferError::Internal.0 as isize)),
            super::super::WgpuDeviceFailureKind::Internal
        );
    }

    #[test]
    fn present_pipeline_blits_rgba_frames_into_bgra_targets_without_cpu_staging() {
        let factory = WgpuFactory::new_with_mode(2, 2, RenderMode::Msaa).unwrap();
        let target = factory
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("nuxie-test-bgra-present-target"),
                size: wgpu::Extent3d {
                    width: 2,
                    height: 2,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let presenter =
            PresentPipeline::new(&factory.context.device, wgpu::TextureFormat::Bgra8Unorm);

        factory
            .begin_frame(0xff11_2233)
            .finish_to_texture_view(&view, &presenter)
            .unwrap();

        let readback = factory
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-test-bgra-present-readback"),
                size: wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64 * 2,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-test-bgra-present-copy"),
                });
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
                    rows_per_image: Some(2),
                },
            },
            target.size(),
        );
        factory.context.queue.submit(Some(encoder.finish()));

        let slice = readback.slice(..);
        pollster::block_on(super::super::map_buffer(&factory.context, &slice)).unwrap();
        let mapped = slice.get_mapped_range().unwrap();
        assert_eq!(&mapped[..4], &[0x33, 0x22, 0x11, 0xff]);
    }

    #[test]
    fn present_pipeline_converts_premultiplied_frames_to_straight_surface_alpha() {
        let factory = WgpuFactory::new_with_mode(1, 1, RenderMode::Msaa).unwrap();
        let target = factory
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("nuxie-test-transparent-bgra-present-target"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let presenter =
            PresentPipeline::new(&factory.context.device, wgpu::TextureFormat::Bgra8Unorm);

        factory
            .begin_frame(0x80ff_0000)
            .finish_to_texture_view(&view, &presenter)
            .unwrap();

        let readback = factory
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-test-transparent-bgra-present-readback"),
                size: wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-test-transparent-bgra-present-copy"),
                });
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
                    rows_per_image: Some(1),
                },
            },
            target.size(),
        );
        factory.context.queue.submit(Some(encoder.finish()));

        let slice = readback.slice(..);
        pollster::block_on(super::super::map_buffer(&factory.context, &slice)).unwrap();
        let mapped = slice.get_mapped_range().unwrap();
        assert_eq!(&mapped[..4], &[0x00, 0x00, 0xff, 0x80]);
    }
}
