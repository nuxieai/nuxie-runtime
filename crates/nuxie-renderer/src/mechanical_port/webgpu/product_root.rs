//! Headless native product root for the exact Dawn WebGPU translation.

use std::cell::RefCell;
use std::ffi::CStr;
use std::pin::Pin;
use std::rc::Rc;

use super::render_context_webgpu_decl::{
    ContextOptions, RenderContextWebGPUImpl, RenderTargetWebGPU,
};
use super::webgpu_cpp_decl::{
    Adapter, AdapterInfo, BackendType, Buffer, BufferUsage, CallbackMode, CommandEncoder,
    Device, ErrorType, FeatureName, Instance, InstanceFeatureName, MapAsyncStatus, MapMode,
    PowerPreference, Queue, QueueWorkDoneStatus, RequestAdapterStatus, RequestDeviceStatus,
    Texture, TextureDimension, TextureFormat, TextureUsage, WaitStatus,
};
use super::webgpu_decl::{
    WGPUFeatureLevel_Undefined, WGPUFuture, WGPUFutureWaitInfo, WGPUOrigin3D,
    WGPURequestAdapterOptions, WGPUStringView, WGPUTexelCopyBufferInfo,
    WGPUTexelCopyBufferLayout, WGPUTexelCopyTextureInfo, WGPUTextureAspect_All,
    WGPUTextureDescriptor, WGPUBufferDescriptor, WGPUExtent3D, WGPU_STRLEN,
};
use crate::exact_source_adapter::ExactSourceBackend;
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, FrameDescriptor, RenderContext, RenderContextContract,
};
use crate::{RenderMode, RendererError};

const WAIT_FOREVER: u64 = u64::MAX;
const COPY_ROW_ALIGNMENT: usize = 256;

fn report_uncaptured_error(_device: &Device, error_type: ErrorType, message: WGPUStringView) {
    eprintln!(
        "exact Dawn WebGPU uncaptured error type={}: {}",
        error_type.0,
        copy_string(&message)
    );
}

pub(crate) struct WebGpuProductBackend {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    context: Option<Pin<Box<RenderContext>>>,
    target: rcp<RenderTargetWebGPU>,
    target_texture: Texture,
    command_encoder: Option<CommandEncoder>,
    width: u32,
    height: u32,
    frame_number: u64,
    active_frame: bool,
    adapter_name: String,
}

impl WebGpuProductBackend {
    pub(crate) fn new(width: u32, height: u32) -> Result<Self, RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::InvalidTextureExtent {
                label: "WebGPU target",
                width,
                height,
                max_dimension: u32::MAX,
            });
        }

        let timed_wait_any = InstanceFeatureName::TimedWaitAny.into();
        let instance_descriptor = super::webgpu_cpp_decl::InstanceDescriptor {
            requiredFeatureCount: 1,
            requiredFeatures: &timed_wait_any,
            ..Default::default()
        };
        let instance = unsafe { super::webgpu_cpp_decl::CreateInstance(Some(&instance_descriptor)) };
        if instance.Get().is_null() {
            return Err(RendererError::Adapter(
                "create exact Dawn WebGPU instance".into(),
            ));
        }

        let adapter_result = Rc::new(RefCell::new(None));
        let callback_result = Rc::clone(&adapter_result);
        let adapter_options = WGPURequestAdapterOptions {
            nextInChain: std::ptr::null_mut(),
            featureLevel: WGPUFeatureLevel_Undefined,
            powerPreference: PowerPreference::HighPerformance.into(),
            forceFallbackAdapter: 0,
            backendType: BackendType::Metal.into(),
            compatibleSurface: std::ptr::null_mut(),
        };
        let future = unsafe {
            instance.RequestAdapter(
                &adapter_options,
                CallbackMode::WaitAnyOnly,
                move |status, adapter, message| {
                    *callback_result.borrow_mut() = Some((status, adapter, copy_string(&message)));
                },
            )
        };
        await_future(&instance, future, "request Dawn Metal adapter")?;
        let (status, adapter, message) = adapter_result
            .borrow_mut()
            .take()
            .ok_or_else(|| RendererError::Adapter("Dawn adapter callback did not run".into()))?;
        if status != RequestAdapterStatus::Success || adapter.Get().is_null() {
            return Err(RendererError::Adapter(format!(
                "request Dawn Metal adapter: {message}"
            )));
        }

        let mut adapter_info = AdapterInfo::default();
        if !unsafe { adapter.GetInfo(&mut adapter_info) }.asBool() {
            return Err(RendererError::Adapter(
                "query Dawn Metal adapter information".into(),
            ));
        }
        if adapter_info.asRaw().backendType != BackendType::Metal.into() {
            return Err(RendererError::Adapter(
                "Dawn selected a non-Metal adapter".into(),
            ));
        }
        let adapter_name = copy_string(&adapter_info.asRaw().device);
        if adapter_name.is_empty() {
            return Err(RendererError::Adapter(
                "Dawn Metal adapter did not report a device name".into(),
            ));
        }

        let clip_distances = FeatureName::ClipDistances.into();
        let supports_clip_distances = unsafe { adapter.HasFeature(clip_distances) }.asBool();
        let mut device_descriptor = super::webgpu_cpp_decl::DeviceDescriptor::default();
        device_descriptor.SetUncapturedErrorCallback(report_uncaptured_error);
        device_descriptor.requiredFeatureCount = usize::from(supports_clip_distances);
        device_descriptor.requiredFeatures = if supports_clip_distances {
            &clip_distances
        } else {
            std::ptr::null()
        };
        let device_result = Rc::new(RefCell::new(None));
        let callback_result = Rc::clone(&device_result);
        let future = unsafe {
            adapter.RequestDevice(
                &device_descriptor,
                CallbackMode::WaitAnyOnly,
                move |status, device, message| {
                    *callback_result.borrow_mut() = Some((status, device, copy_string(&message)));
                },
            )
        };
        await_future(&instance, future, "request Dawn WebGPU device")?;
        let (status, device, message) = device_result
            .borrow_mut()
            .take()
            .ok_or_else(|| RendererError::Device("Dawn device callback did not run".into()))?;
        if status != RequestDeviceStatus::Success || device.Get().is_null() {
            return Err(RendererError::Device(format!(
                "request exact Dawn WebGPU device: {message}"
            )));
        }
        let queue = unsafe { device.GetQueue() };
        if queue.Get().is_null() {
            return Err(RendererError::Device(
                "obtain exact Dawn WebGPU queue".into(),
            ));
        }

        let mut context = super::render_context_webgpu_decl::MakeContext(
            adapter.clone(),
            device.clone(),
            queue.clone(),
            ContextOptions::default(),
        );
        #[cfg(feature = "rive-decoders")]
        crate::exact_source_adapter::install_bitmap_decoder(context.as_mut());
        let implementation = unsafe {
            &mut *Pin::get_unchecked_mut(context.as_mut())
                .static_impl_cast::<RenderContextWebGPUImpl>()
        };

        let texture_descriptor = WGPUTextureDescriptor {
            usage: (TextureUsage::RenderAttachment
                | TextureUsage::CopySrc
                | TextureUsage::TextureBinding)
                .intoBitmask()
                .into(),
            dimension: TextureDimension::e2D.into(),
            size: WGPUExtent3D {
                width,
                height,
                depthOrArrayLayers: 1,
            },
            format: TextureFormat::RGBA8Unorm.into(),
            ..Default::default()
        };
        let target_texture = unsafe { device.CreateTexture(&texture_descriptor) };
        if target_texture.Get().is_null() {
            return Err(RendererError::Device(
                "create exact Dawn WebGPU target texture".into(),
            ));
        }
        let mut target = implementation.makeRenderTarget(TextureFormat::RGBA8Unorm, width, height);
        if !target.operator_bool() {
            return Err(RendererError::Device(
                "create exact Dawn WebGPU render target".into(),
            ));
        }
        let target_view = unsafe { target_texture.CreateView(std::ptr::null()) };
        if target_view.Get().is_null() {
            return Err(RendererError::Device(
                "create exact Dawn WebGPU target view".into(),
            ));
        }
        unsafe { &mut *target.get() }.setTargetTextureView(target_view, target_texture.clone());

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            context: Some(context),
            target,
            target_texture,
            command_encoder: None,
            width,
            height,
            frame_number: 0,
            active_frame: false,
            adapter_name,
        })
    }

    pub(crate) fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    fn context_pin(&mut self) -> Pin<&mut RenderContext> {
        self.context.as_mut().expect("live WebGPU context").as_mut()
    }

    fn submit_and_wait(&self, command_buffer: &super::webgpu_cpp_decl::CommandBuffer) -> Result<(), RendererError> {
        let raw = command_buffer.Get();
        unsafe { self.queue.Submit(1, &raw) };
        let completion = Rc::new(RefCell::new(None));
        let callback_completion = Rc::clone(&completion);
        let future = unsafe {
            self.queue.OnSubmittedWorkDone(
                CallbackMode::WaitAnyOnly,
                move |status, message| {
                    *callback_completion.borrow_mut() = Some((status, copy_string(&message)));
                },
            )
        };
        await_future(&self.instance, future, "wait for Dawn queue")?;
        let (status, message) = completion
            .borrow_mut()
            .take()
            .ok_or_else(|| RendererError::Device("Dawn queue callback did not run".into()))?;
        if status != QueueWorkDoneStatus::Success {
            return Err(RendererError::Device(format!(
                "Dawn queue completion failed: {message}"
            )));
        }
        Ok(())
    }

    fn read_pixels(&self) -> Result<Vec<u8>, RendererError> {
        let packed_row_bytes = usize::try_from(u64::from(self.width) * 4)
            .map_err(|_| RendererError::Map("WebGPU row size overflow".into()))?;
        let padded_row_bytes = packed_row_bytes
            .checked_add(COPY_ROW_ALIGNMENT - 1)
            .map(|value| value & !(COPY_ROW_ALIGNMENT - 1))
            .ok_or_else(|| RendererError::Map("WebGPU padded row size overflow".into()))?;
        let readback_size = padded_row_bytes
            .checked_mul(self.height as usize)
            .ok_or_else(|| RendererError::Map("WebGPU readback size overflow".into()))?;
        let buffer_descriptor = WGPUBufferDescriptor {
            usage: (BufferUsage::CopyDst | BufferUsage::MapRead)
                .intoBitmask()
                .into(),
            size: readback_size as u64,
            ..Default::default()
        };
        let readback = unsafe { self.device.CreateBuffer(&buffer_descriptor) };
        let encoder = unsafe { self.device.CreateCommandEncoder(std::ptr::null()) };
        if readback.Get().is_null() || encoder.Get().is_null() {
            return Err(RendererError::Map(
                "create exact Dawn WebGPU readback resources".into(),
            ));
        }
        let source = WGPUTexelCopyTextureInfo {
            texture: self.target_texture.Get(),
            mipLevel: 0,
            origin: WGPUOrigin3D { x: 0, y: 0, z: 0 },
            aspect: WGPUTextureAspect_All,
        };
        let destination = WGPUTexelCopyBufferInfo {
            layout: WGPUTexelCopyBufferLayout {
                offset: 0,
                bytesPerRow: padded_row_bytes as u32,
                rowsPerImage: self.height,
            },
            buffer: readback.Get(),
        };
        let extent = WGPUExtent3D {
            width: self.width,
            height: self.height,
            depthOrArrayLayers: 1,
        };
        unsafe { encoder.CopyTextureToBuffer(&source, &destination, &extent) };
        let command_buffer = unsafe { encoder.Finish(std::ptr::null()) };
        if command_buffer.Get().is_null() {
            return Err(RendererError::Map(
                "finish exact Dawn WebGPU readback commands".into(),
            ));
        }
        let raw = command_buffer.Get();
        unsafe { self.queue.Submit(1, &raw) };

        let mapping = Rc::new(RefCell::new(None));
        let callback_mapping = Rc::clone(&mapping);
        let future = unsafe {
            readback.MapAsync(
                MapMode::Read.into(),
                0,
                readback_size,
                CallbackMode::WaitAnyOnly,
                move |status, message| {
                    *callback_mapping.borrow_mut() = Some((status, copy_string(&message)));
                },
            )
        };
        await_future(&self.instance, future, "map Dawn readback buffer")?;
        let (status, message) = mapping
            .borrow_mut()
            .take()
            .ok_or_else(|| RendererError::Map("Dawn map callback did not run".into()))?;
        if status != MapAsyncStatus::Success {
            return Err(RendererError::Map(format!(
                "Dawn readback mapping failed: {message}"
            )));
        }
        let mapped = unsafe { readback.GetConstMappedRange(0, readback_size) }.cast::<u8>();
        if mapped.is_null() {
            unsafe { readback.Unmap() };
            return Err(RendererError::Map(
                "Dawn readback returned a null mapped range".into(),
            ));
        }
        let source = unsafe { std::slice::from_raw_parts(mapped, readback_size) };
        let mut pixels = vec![0; packed_row_bytes * self.height as usize];
        for row in 0..self.height as usize {
            pixels[row * packed_row_bytes..][..packed_row_bytes]
                .copy_from_slice(&source[row * padded_row_bytes..][..packed_row_bytes]);
        }
        unsafe { readback.Unmap() };
        Ok(pixels)
    }
}

impl ExactSourceBackend for WebGpuProductBackend {
    fn context_mut(&mut self) -> Pin<&mut RenderContext> {
        self.context_pin()
    }

    fn begin_frame(&mut self, clear_color: u32, mode: RenderMode) -> Result<u64, RendererError> {
        if self.active_frame {
            return Err(RendererError::Device(
                "exact WebGPU context already has an active frame".into(),
            ));
        }
        let encoder = unsafe { self.device.CreateCommandEncoder(std::ptr::null()) };
        if encoder.Get().is_null() {
            return Err(RendererError::Device(
                "create exact Dawn WebGPU command encoder".into(),
            ));
        }
        let mut descriptor = FrameDescriptor {
            renderTargetWidth: self.width,
            renderTargetHeight: self.height,
            clearColor: clear_color,
            ..FrameDescriptor::default()
        };
        match mode {
            RenderMode::RasterOrdering => {}
            RenderMode::Msaa => descriptor.msaaSampleCount = 4,
            RenderMode::ClockwiseAtomic => {
                descriptor.disableRasterOrdering = true;
                descriptor.clockwiseFillOverride = true;
            }
        }
        unsafe { Pin::get_unchecked_mut(self.context_pin()) }.beginFrameExecutable(&descriptor);
        self.frame_number = self.frame_number.wrapping_add(1);
        self.command_encoder = Some(encoder);
        self.active_frame = true;
        Ok(self.frame_number)
    }

    fn finish_frame(&mut self, frame_number: u64) -> Result<Vec<u8>, RendererError> {
        if !self.active_frame || frame_number != self.frame_number {
            return Err(RendererError::Device(
                "exact WebGPU frame ownership mismatch".into(),
            ));
        }
        let encoder = self
            .command_encoder
            .as_ref()
            .ok_or_else(|| RendererError::Device("missing WebGPU command encoder".into()))?;
        let resources = FlushResources {
            renderTarget: self.target.get().cast(),
            externalCommandBuffer: encoder.Get().cast(),
            currentFrameNumber: frame_number,
            safeFrameNumber: frame_number.saturating_sub(1),
        };
        unsafe {
            Pin::get_unchecked_mut(self.context_pin()).flushExecutable(&resources);
        }
        let encoder = self.command_encoder.take().expect("live WebGPU encoder");
        let command_buffer = unsafe { encoder.Finish(std::ptr::null()) };
        if command_buffer.Get().is_null() {
            self.active_frame = false;
            return Err(RendererError::Device(
                "finish exact Dawn WebGPU commands".into(),
            ));
        }
        let completion = self.submit_and_wait(&command_buffer);
        self.active_frame = false;
        completion?;
        self.read_pixels()
    }

    fn abort_frame(&mut self) {
        if self.active_frame {
            unsafe { Pin::get_unchecked_mut(self.context_pin()) }.abortFrameExecutable();
            self.command_encoder.take();
            self.active_frame = false;
        }
    }
}

impl Drop for WebGpuProductBackend {
    fn drop(&mut self) {
        self.abort_frame();
        self.target.operator_assign_null();
        self.context.take();
        self.command_encoder.take();
        unsafe { self.device.Destroy() };
        let _ = (&self.adapter, &self.queue);
    }
}

fn await_future(
    instance: &Instance,
    future: WGPUFuture,
    operation: &'static str,
) -> Result<(), RendererError> {
    let mut wait = WGPUFutureWaitInfo {
        future,
        completed: 0,
    };
    let status = unsafe { instance.WaitAny(1, &mut wait, WAIT_FOREVER) };
    if status != WaitStatus::Success {
        return Err(RendererError::Device(format!(
            "{operation}: WaitAny returned {status:?}"
        )));
    }
    Ok(())
}

fn copy_string(value: &WGPUStringView) -> String {
    if value.data.is_null() {
        return String::new();
    }
    let length = if value.length == WGPU_STRLEN {
        unsafe { CStr::from_ptr(value.data) }.to_bytes().len()
    } else {
        value.length
    };
    let bytes = unsafe { std::slice::from_raw_parts(value.data.cast::<u8>(), length) };
    String::from_utf8_lossy(bytes).into_owned()
}
