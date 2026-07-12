// Copyright 2026 Rive
// Temporary source copied into RIVE_RUNTIME_DIR by build.sh.

#include "rive/renderer/rive_renderer.hpp"
#include "rive/renderer/webgpu/render_context_webgpu_impl.hpp"

#include <webgpu/webgpu.h>
#include <webgpu/webgpu_cpp.h>

#include <algorithm>
#include <array>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

namespace
{
constexpr uint32_t kFrameWidth = 64;
constexpr uint32_t kFrameHeight = 64;
constexpr uint32_t kHeaderBytes = 20;
constexpr uint32_t kBytesPerTexel = 2;
constexpr uint32_t kCopyRowAlignment = 256;

void fail(const char* message)
{
    std::fprintf(stderr, "cpp-atlas-mask-oracle: %s\\n", message);
    std::exit(1);
}

void await(WGPUInstance instance, WGPUFuture future)
{
    WGPUFutureWaitInfo futureWait = {future};
    if (wgpuInstanceWaitAny(instance, 1, &futureWait, -1) !=
        WGPUWaitStatus_Success)
    {
        fail("wgpuInstanceWaitAny failed");
    }
}

void onAdapterRequest(WGPURequestAdapterStatus status,
                      WGPUAdapter adapter,
                      WGPUStringView message,
                      void* userdata1,
                      void*)
{
    if (status != WGPURequestAdapterStatus_Success)
    {
        std::fprintf(stderr,
                     "cpp-atlas-mask-oracle: adapter request failed: %.*s\\n",
                     static_cast<int>(message.length),
                     message.data);
        return;
    }
    *static_cast<WGPUAdapter*>(userdata1) = adapter;
}

void onDeviceError(WGPUDevice const*,
                   WGPUErrorType,
                   WGPUStringView message,
                   void*,
                   void*)
{
    std::fprintf(stderr,
                 "cpp-atlas-mask-oracle: WebGPU error: %.*s\\n",
                 static_cast<int>(message.length),
                 message.data);
}

struct MapResult
{
    bool succeeded = false;
};

void onMap(WGPUMapAsyncStatus status,
           WGPUStringView message,
           void* userdata1,
           void*)
{
    MapResult* result = static_cast<MapResult*>(userdata1);
    result->succeeded = status == WGPUMapAsyncStatus_Success;
    if (!result->succeeded)
    {
        std::fprintf(stderr,
                     "cpp-atlas-mask-oracle: MapAsync failed: %.*s\\n",
                     static_cast<int>(message.length),
                     message.data);
    }
}

uint32_t alignCopyRow(uint32_t bytes)
{
    return (bytes + kCopyRowAlignment - 1) & ~(kCopyRowAlignment - 1);
}

void writeU32(std::array<uint8_t, kHeaderBytes>& header,
              size_t offset,
              uint32_t value)
{
    for (size_t i = 0; i != 4; ++i)
    {
        header[offset + i] = static_cast<uint8_t>(value >> (i * 8));
    }
}

void writeMask(const char* output,
               uint32_t width,
               uint32_t height,
               const uint8_t* paddedRows,
               uint32_t paddedRowBytes)
{
    const uint32_t packedRowBytes = width * kBytesPerTexel;
    std::array<uint8_t, kHeaderBytes> header{};
    constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'M', 'S', 'K', '\0'};
    std::memcpy(header.data(), kMagic, sizeof(kMagic));
    // Exact AtlasMask::serialize() contract from 10a64ec.
    writeU32(header, 8, 1);
    writeU32(header, 12, width);
    writeU32(header, 16, height);

    std::ofstream file(output, std::ios::binary | std::ios::trunc);
    if (!file)
    {
        fail("could not open output file");
    }
    file.write(reinterpret_cast<const char*>(header.data()), header.size());
    for (uint32_t y = 0; y != height; ++y)
    {
        file.write(reinterpret_cast<const char*>(paddedRows + y * paddedRowBytes),
                   packedRowBytes);
    }
    if (!file)
    {
        fail("could not write output file");
    }
}
} // namespace

int main(int argc, char** argv)
{
    const char* output = argc == 2 ? argv[1] : "atlas-mask.r16f";

    constexpr WGPUInstanceFeatureName kTimedWaitAny =
        WGPUInstanceFeatureName_TimedWaitAny;
    WGPUInstanceDescriptor instanceDesc = {};
    instanceDesc.requiredFeatureCount = 1;
    instanceDesc.requiredFeatures = &kTimedWaitAny;
    wgpu::Instance instance =
        wgpu::Instance::Acquire(wgpuCreateInstance(&instanceDesc));
    if (!instance)
    {
        fail("could not create Dawn WebGPU instance");
    }

    WGPUAdapter adapterHandle = nullptr;
    WGPURequestAdapterCallbackInfo adapterCallback = {};
    adapterCallback.mode = WGPUCallbackMode_WaitAnyOnly;
    adapterCallback.callback = onAdapterRequest;
    adapterCallback.userdata1 = &adapterHandle;
    await(instance.Get(),
          wgpuInstanceRequestAdapter(instance.Get(), nullptr, adapterCallback));
    if (adapterHandle == nullptr)
    {
        fail("could not acquire Dawn WebGPU adapter");
    }
    wgpu::Adapter adapter = wgpu::Adapter::Acquire(adapterHandle);

    WGPUDeviceDescriptor deviceDesc = {};
    deviceDesc.uncapturedErrorCallbackInfo.callback = onDeviceError;
    wgpu::Device device =
        wgpu::Device::Acquire(wgpuAdapterCreateDevice(adapter.Get(), &deviceDesc));
    if (!device)
    {
        fail("could not create Dawn WebGPU device");
    }
    wgpu::Queue queue = device.GetQueue();
    auto context = rive::gpu::RenderContextWebGPUImpl::MakeContext(
        adapter,
        device,
        queue,
        rive::gpu::RenderContextWebGPUImpl::ContextOptions());
    if (!context)
    {
        fail("could not create Rive WebGPU render context");
    }
    auto* webgpuContext =
        context->static_impl_cast<rive::gpu::RenderContextWebGPUImpl>();

    wgpu::TextureDescriptor targetDesc = {};
    targetDesc.usage = wgpu::TextureUsage::RenderAttachment;
    targetDesc.dimension = wgpu::TextureDimension::e2D;
    targetDesc.size = {kFrameWidth, kFrameHeight, 1};
    targetDesc.format = wgpu::TextureFormat::RGBA8Unorm;
    wgpu::Texture targetTexture = device.CreateTexture(&targetDesc);
    auto target = webgpuContext->makeRenderTarget(
        wgpu::TextureFormat::RGBA8Unorm,
        kFrameWidth,
        kFrameHeight);
    target->setTargetTextureView(targetTexture.CreateView(), targetTexture);

    context->beginFrame({.renderTargetWidth = kFrameWidth,
                         .renderTargetHeight = kFrameHeight,
                         .loadAction = rive::gpu::LoadAction::clear,
                         .clearColor = 0,
                         .msaaSampleCount = 4});
    rive::RiveRenderer renderer(context.get());
    auto path = context->makeEmptyRenderPath();
    // Exact fixed_feather_atlas_mask() geometry from Rust's env test.
    path->moveTo(16, 16);
    path->lineTo(48, 16);
    path->lineTo(48, 48);
    path->lineTo(16, 48);
    path->close();
    auto paint = context->makeRenderPaint();
    paint->style(rive::RenderPaintStyle::stroke);
    paint->thickness(8);
    paint->join(rive::StrokeJoin::miter);
    paint->cap(rive::StrokeCap::butt);
    paint->feather(20);
    paint->color(0xffffffff);
    renderer.drawPath(path.get(), paint.get());

    wgpu::CommandEncoder renderEncoder = device.CreateCommandEncoder();
    context->flush({.renderTarget = target.get(),
                    .externalCommandBuffer = renderEncoder.Get()});
    wgpu::CommandBuffer renderCommands = renderEncoder.Finish();
    queue.Submit(1, &renderCommands);

    const wgpu::Texture atlas = webgpuContext->atlasMaskTextureForOracle();
    const uint32_t width = kFrameWidth;
    const uint32_t height = kFrameHeight;
    const uint32_t copyWidth = std::min(width, atlas.GetWidth());
    const uint32_t copyHeight = std::min(height, atlas.GetHeight());
    const uint32_t packedRowBytes = width * kBytesPerTexel;
    const uint32_t paddedRowBytes = alignCopyRow(packedRowBytes);
    wgpu::BufferDescriptor readbackDesc = {};
    readbackDesc.usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::MapRead;
    readbackDesc.size = static_cast<uint64_t>(paddedRowBytes) * height;
    wgpu::Buffer readback = device.CreateBuffer(&readbackDesc);

    wgpu::CommandEncoder readEncoder = device.CreateCommandEncoder();
    wgpu::TexelCopyTextureInfo source = {.texture = atlas};
    wgpu::TexelCopyBufferInfo destination = {
        .layout = {.offset = 0, .bytesPerRow = paddedRowBytes, .rowsPerImage = height},
        .buffer = readback,
    };
    wgpu::Extent3D copySize = {
        .width = copyWidth,
        .height = copyHeight,
        .depthOrArrayLayers = 1,
    };
    readEncoder.CopyTextureToBuffer(&source, &destination, &copySize);
    wgpu::CommandBuffer readCommands = readEncoder.Finish();
    queue.Submit(1, &readCommands);

    MapResult mapResult;
    WGPUBufferMapCallbackInfo mapCallback = {};
    mapCallback.mode = WGPUCallbackMode_WaitAnyOnly;
    mapCallback.callback = onMap;
    mapCallback.userdata1 = &mapResult;
    await(instance.Get(),
          wgpuBufferMapAsync(readback.Get(),
                             WGPUMapMode_Read,
                             0,
                             readbackDesc.size,
                             mapCallback));
    if (!mapResult.succeeded)
    {
        fail("could not map atlas readback buffer");
    }
    const uint8_t* data = static_cast<const uint8_t*>(
        wgpuBufferGetConstMappedRange(readback.Get(), 0, readbackDesc.size));
    if (data == nullptr)
    {
        fail("mapped atlas readback buffer has no range");
    }
    writeMask(output, width, height, data, paddedRowBytes);
    wgpuBufferUnmap(readback.Get());
    std::printf("wrote %s: %ux%u R16Float row-packed atlas mask\\n",
                output,
                width,
                height);
    return 0;
}
