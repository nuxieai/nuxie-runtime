//! A device that is nothing but its limits, so WebGPU's read runs on a host with
//! no GPU and can report numbers a real default device never would. Only a
//! browser or a wasm host provides this C API, so a native test binary has to
//! define all of it to link. Every handle in these tests is null, so everything
//! past `newContext`'s own calls exists for the linker and never runs.

use super::render_context_webgpu_decl::ContextOptions;
use super::render_context_webgpu_impl::newContext;
use super::webgpu_cpp_decl::{Adapter, Device, Queue};
use super::webgpu_decl::*;
use std::cell::Cell;
use std::ffi::c_void;

thread_local! {
    static MAX_TEXTURE_DIMENSION_2D: Cell<u32> = const { Cell::new(0) };
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceGetLimits(_: WGPUDevice, limits: *mut WGPULimits) -> WGPUStatus {
    let limits = unsafe { &mut *limits };
    limits.maxTextureDimension2D = MAX_TEXTURE_DIMENSION_2D.get();
    limits.maxStorageBuffersPerShaderStage = 8;
    WGPUStatus_Success
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
    wgpuDeviceRelease(WGPUDevice);
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

#[test]
fn the_render_context_reports_its_devices_canvas_limit() {
    // Past the 2048 default and either side of the 8192 Ore's table says, so
    // neither a fixed number nor the default can pass.
    for limit in [4096, 8192, 16384] {
        MAX_TEXTURE_DIMENSION_2D.set(limit);
        let context = std::mem::ManuallyDrop::new(newContext(
            Adapter::default(),
            Device::default(),
            Queue::default(),
            ContextOptions::default(),
        ));
        assert_eq!(context.platformFeatures().maxTextureSize, limit);
    }
}
