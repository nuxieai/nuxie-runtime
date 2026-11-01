//! A device that is nothing but its limits, so WebGPU's read runs on a host with
//! no GPU and can report numbers a real default device never would. Only a
//! browser or a wasm host provides this C API, so a native test binary has to
//! define all of it to link. Every handle in these tests is null, so everything
//! past the limits read and `newContext`'s own calls exists for the linker and
//! never runs.

use super::render_context_webgpu_decl::ContextOptions;
use super::render_context_webgpu_impl::newContext;
use super::webgpu_cpp_decl::{Adapter, Device, Queue};
use super::webgpu_decl::*;
use super::WebGpuProductBackend;
use crate::NativeWebGpuFactory;
use nuxie_render_api::Factory;
use std::cell::Cell;
use std::ffi::c_void;
use std::mem::ManuallyDrop;

thread_local! {
    static MAX_TEXTURE_DIMENSION_2D: Cell<u32> = const { Cell::new(0) };
    static LIMITS_STATUS: Cell<WGPUStatus> = const { Cell::new(WGPUStatus_Success) };
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceGetLimits(_: WGPUDevice, limits: *mut WGPULimits) -> WGPUStatus {
    let limits = unsafe { &mut *limits };
    limits.maxTextureDimension2D = MAX_TEXTURE_DIMENSION_2D.get();
    limits.maxStorageBuffersPerShaderStage = 8;
    LIMITS_STATUS.get()
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceHasFeature(_: WGPUDevice, _: WGPUFeatureName) -> WGPUBool {
    0
}

#[no_mangle]
unsafe extern "C" fn wgpuAdapterGetInfo(_: WGPUAdapter, _: *mut WGPUAdapterInfo) -> WGPUStatus {
    WGPUStatus_Error
}

#[no_mangle]
unsafe extern "C" fn wgpuAdapterInfoFreeMembers(_: WGPUAdapterInfo) {}

macro_rules! never_called {
    ($($name:ident($($arg:ty),*) $(-> $ret:ty)?;)*) => {
        $(#[no_mangle]
        unsafe extern "C" fn $name($(_: $arg),*) $(-> $ret)? {
            unreachable!("every handle in these tests is null");
        })*
    };
}

never_called! {
    wgpuAdapterRelease(WGPUAdapter);
    wgpuBindGroupLayoutAddRef(WGPUBindGroupLayout);
    wgpuBindGroupLayoutRelease(WGPUBindGroupLayout);
    wgpuBindGroupRelease(WGPUBindGroup);
    wgpuBufferAddRef(WGPUBuffer);
    wgpuBufferRelease(WGPUBuffer);
    wgpuCommandEncoderBeginRenderPass(
        WGPUCommandEncoder,
        *const WGPURenderPassDescriptor
    ) -> WGPURenderPassEncoder;
    wgpuCommandEncoderRelease(WGPUCommandEncoder);
    wgpuDeviceAddRef(WGPUDevice);
    wgpuDeviceCreateBindGroup(WGPUDevice, *const WGPUBindGroupDescriptor) -> WGPUBindGroup;
    wgpuDeviceCreateBindGroupLayout(
        WGPUDevice,
        *const WGPUBindGroupLayoutDescriptor
    ) -> WGPUBindGroupLayout;
    wgpuDeviceCreateBuffer(WGPUDevice, *const WGPUBufferDescriptor) -> WGPUBuffer;
    wgpuDeviceCreatePipelineLayout(
        WGPUDevice,
        *const WGPUPipelineLayoutDescriptor
    ) -> WGPUPipelineLayout;
    wgpuDeviceCreateRenderPipeline(
        WGPUDevice,
        *const WGPURenderPipelineDescriptor
    ) -> WGPURenderPipeline;
    wgpuDeviceCreateSampler(WGPUDevice, *const WGPUSamplerDescriptor) -> WGPUSampler;
    wgpuDeviceCreateShaderModule(
        WGPUDevice,
        *const WGPUShaderModuleDescriptor
    ) -> WGPUShaderModule;
    wgpuDeviceCreateTexture(WGPUDevice, *const WGPUTextureDescriptor) -> WGPUTexture;
    wgpuDeviceDestroy(WGPUDevice);
    wgpuDeviceRelease(WGPUDevice);
    wgpuInstanceRelease(WGPUInstance);
    wgpuPipelineLayoutRelease(WGPUPipelineLayout);
    wgpuQueueAddRef(WGPUQueue);
    wgpuQueueRelease(WGPUQueue);
    wgpuQueueWriteBuffer(WGPUQueue, WGPUBuffer, u64, *const c_void, usize);
    wgpuQueueWriteTexture(
        WGPUQueue,
        *const WGPUTexelCopyTextureInfo,
        *const c_void,
        usize,
        *const WGPUTexelCopyBufferLayout,
        *const WGPUExtent3D
    );
    wgpuRenderPassEncoderDraw(WGPURenderPassEncoder, u32, u32, u32, u32);
    wgpuRenderPassEncoderDrawIndexed(WGPURenderPassEncoder, u32, u32, u32, i32, u32);
    wgpuRenderPassEncoderEnd(WGPURenderPassEncoder);
    wgpuRenderPassEncoderRelease(WGPURenderPassEncoder);
    wgpuRenderPassEncoderSetBindGroup(
        WGPURenderPassEncoder,
        u32,
        WGPUBindGroup,
        usize,
        *const u32
    );
    wgpuRenderPassEncoderSetBlendConstant(WGPURenderPassEncoder, *const WGPUColor);
    wgpuRenderPassEncoderSetIndexBuffer(
        WGPURenderPassEncoder,
        WGPUBuffer,
        WGPUIndexFormat,
        u64,
        u64
    );
    wgpuRenderPassEncoderSetPipeline(WGPURenderPassEncoder, WGPURenderPipeline);
    wgpuRenderPassEncoderSetScissorRect(WGPURenderPassEncoder, u32, u32, u32, u32);
    wgpuRenderPassEncoderSetStencilReference(WGPURenderPassEncoder, u32);
    wgpuRenderPassEncoderSetVertexBuffer(WGPURenderPassEncoder, u32, WGPUBuffer, u64, u64);
    wgpuRenderPassEncoderSetViewport(WGPURenderPassEncoder, f32, f32, f32, f32, f32, f32);
    wgpuRenderPipelineRelease(WGPURenderPipeline);
    wgpuSamplerAddRef(WGPUSampler);
    wgpuSamplerRelease(WGPUSampler);
    wgpuShaderModuleRelease(WGPUShaderModule);
    wgpuTextureAddRef(WGPUTexture);
    wgpuTextureCreateView(WGPUTexture, *const WGPUTextureViewDescriptor) -> WGPUTextureView;
    wgpuTextureRelease(WGPUTexture);
    wgpuTextureViewAddRef(WGPUTextureView);
    wgpuTextureViewRelease(WGPUTextureView);
}

// The product factory on the fake device, as `new` builds it on a real one.
fn factory_on_fake_device() -> ManuallyDrop<NativeWebGpuFactory> {
    ManuallyDrop::new(NativeWebGpuFactory::with_backend(
        WebGpuProductBackend::with_null_device_for_test(),
    ))
}

#[test]
fn the_factory_answers_with_its_devices_limit() {
    // Past the 2048 default and either side of the 8192 Ore's table says, so
    // neither a fixed number nor the default can pass.
    for limit in [4096, 8192, 16384] {
        MAX_TEXTURE_DIMENSION_2D.set(limit);
        assert_eq!(factory_on_fake_device().max_texture_size(), limit);
    }
}

#[test]
fn the_factory_answers_the_default_when_the_limits_are_unusable() {
    let answers = [
        (WGPUStatus_Error, 4096),
        (WGPUStatus_Success, WGPU_LIMIT_U32_UNDEFINED),
        (WGPUStatus_Success, 0),
    ]
    .map(|(status, limit)| {
        LIMITS_STATUS.set(status);
        MAX_TEXTURE_DIMENSION_2D.set(limit);
        factory_on_fake_device().max_texture_size()
    });
    assert_eq!(answers, [2048; 3]);
}

// Upstream's WebGPU never writes this field. It also sets the feather atlas's
// ceiling, so the device's limit must not reach it.
#[test]
fn the_render_context_keeps_upstreams_texture_size() {
    MAX_TEXTURE_DIMENSION_2D.set(4096);
    let context = ManuallyDrop::new(newContext(
        Adapter::default(),
        Device::default(),
        Queue::default(),
        ContextOptions::default(),
    ));
    assert_eq!(context.platformFeatures().maxTextureSize, 2048);
}
