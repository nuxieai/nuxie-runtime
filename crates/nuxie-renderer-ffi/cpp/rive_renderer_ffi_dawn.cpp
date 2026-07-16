#include "rive_renderer_ffi_private.hpp"

#include "rive/renderer/webgpu/render_context_webgpu_impl.hpp"

#include <webgpu/webgpu.h>
#include <webgpu/webgpu_cpp.h>

#ifdef RIVE_FFI_PERF_COUNTERS
#include <dawn/dawn_proc.h>
#include <dawn/native/DawnNative.h>
#endif

#include <cstdio>
#include <cstring>
#include <string>
#include <string_view>
#ifdef RIVE_FFI_PERF_COUNTERS
#include <unordered_map>
#endif

namespace
{
#ifdef RIVE_FFI_PERF_COUNTERS
enum class PipelineKind
{
    other,
    tessellation,
    pathPatches,
};

DawnProcTable g_nativeProcs = {};
DawnProcTable g_countingProcs = {};
thread_local rive_ffi_backend_work_metrics* g_activeMetrics = nullptr;
thread_local std::unordered_map<WGPURenderPipeline, PipelineKind>
    g_pipelineKinds;
thread_local std::unordered_map<WGPURenderPassEncoder, PipelineKind>
    g_passKinds;

std::string_view stringView(WGPUStringView value)
{
    if (value.data == nullptr)
    {
        return {};
    }
    const size_t length = value.length == WGPU_STRLEN
                              ? std::strlen(value.data)
                              : value.length;
    return {value.data, length};
}

PipelineKind pipelineKind(const WGPURenderPipelineDescriptor* descriptor)
{
    if (descriptor == nullptr)
    {
        return PipelineKind::other;
    }
    const std::string_view label = stringView(descriptor->label);
    if (label.find("Tessellate") != std::string_view::npos)
    {
        return PipelineKind::tessellation;
    }
    if (label.find("AtlasPipeline") != std::string_view::npos)
    {
        return PipelineKind::pathPatches;
    }
    constexpr std::string_view prefix = "RIVE_Draw{drawType=";
    const size_t drawTypeStart = label.find(prefix);
    if (drawTypeStart == std::string_view::npos)
    {
        return PipelineKind::other;
    }
    const size_t valueStart = drawTypeStart + prefix.size();
    const size_t valueEnd = label.find(',', valueStart);
    if (valueEnd == std::string_view::npos)
    {
        return PipelineKind::other;
    }
    unsigned drawType = 0;
    for (char c : label.substr(valueStart, valueEnd - valueStart))
    {
        if (c < '0' || c > '9')
        {
            return PipelineKind::other;
        }
        drawType = drawType * 10 + static_cast<unsigned>(c - '0');
    }
    switch (drawType)
    {
        case 0:
        case 1:
        case 2:
        case 7:
        case 8:
        case 9:
        case 10:
        case 11:
        case 12:
        case 13:
            return PipelineKind::pathPatches;
        default:
            return PipelineKind::other;
    }
}

WGPUCommandEncoder countedDeviceCreateCommandEncoder(
    WGPUDevice device,
    const WGPUCommandEncoderDescriptor* descriptor)
{
    if (g_activeMetrics != nullptr)
    {
        ++g_activeMetrics->command_encoders;
    }
    return g_nativeProcs.deviceCreateCommandEncoder(device, descriptor);
}

WGPURenderPassEncoder countedCommandEncoderBeginRenderPass(
    WGPUCommandEncoder encoder,
    const WGPURenderPassDescriptor* descriptor)
{
    WGPURenderPassEncoder pass =
        g_nativeProcs.commandEncoderBeginRenderPass(encoder, descriptor);
    g_passKinds[pass] = PipelineKind::other;
    if (g_activeMetrics != nullptr)
    {
        ++g_activeMetrics->render_passes;
    }
    return pass;
}

WGPUBindGroup countedDeviceCreateBindGroup(
    WGPUDevice device,
    const WGPUBindGroupDescriptor* descriptor)
{
    if (g_activeMetrics != nullptr)
    {
        ++g_activeMetrics->bind_groups_created;
        if (descriptor != nullptr)
        {
            for (size_t i = 0; i < descriptor->entryCount; ++i)
            {
                g_activeMetrics->texture_bindings +=
                    descriptor->entries[i].textureView != nullptr ? 1 : 0;
            }
        }
    }
    return g_nativeProcs.deviceCreateBindGroup(device, descriptor);
}

WGPURenderPipeline countedDeviceCreateRenderPipeline(
    WGPUDevice device,
    const WGPURenderPipelineDescriptor* descriptor)
{
    WGPURenderPipeline pipeline =
        g_nativeProcs.deviceCreateRenderPipeline(device, descriptor);
    g_pipelineKinds[pipeline] = pipelineKind(descriptor);
    return pipeline;
}

void countedRenderPassEncoderSetPipeline(WGPURenderPassEncoder pass,
                                         WGPURenderPipeline pipeline)
{
    auto found = g_pipelineKinds.find(pipeline);
    g_passKinds[pass] = found == g_pipelineKinds.end()
                            ? PipelineKind::other
                            : found->second;
    g_nativeProcs.renderPassEncoderSetPipeline(pass, pipeline);
}

void countedRenderPassEncoderSetBindGroup(WGPURenderPassEncoder pass,
                                          uint32_t groupIndex,
                                          WGPUBindGroup group,
                                          size_t dynamicOffsetCount,
                                          const uint32_t* dynamicOffsets)
{
    if (g_activeMetrics != nullptr)
    {
        ++g_activeMetrics->bind_group_sets;
    }
    g_nativeProcs.renderPassEncoderSetBindGroup(
        pass, groupIndex, group, dynamicOffsetCount, dynamicOffsets);
}

void recordDraw(WGPURenderPassEncoder pass, uint32_t instanceCount)
{
    if (g_activeMetrics == nullptr)
    {
        return;
    }
    ++g_activeMetrics->gpu_draw_calls;
    g_activeMetrics->gpu_draw_instances += instanceCount;
    const auto found = g_passKinds.find(pass);
    const PipelineKind kind = found == g_passKinds.end()
                                  ? PipelineKind::other
                                  : found->second;
    if (kind == PipelineKind::tessellation)
    {
        g_activeMetrics->tessellation_spans += instanceCount;
    }
    else if (kind == PipelineKind::pathPatches)
    {
        g_activeMetrics->path_patches += instanceCount;
    }
}

void countedRenderPassEncoderDraw(WGPURenderPassEncoder pass,
                                  uint32_t vertexCount,
                                  uint32_t instanceCount,
                                  uint32_t firstVertex,
                                  uint32_t firstInstance)
{
    recordDraw(pass, instanceCount);
    g_nativeProcs.renderPassEncoderDraw(
        pass, vertexCount, instanceCount, firstVertex, firstInstance);
}

void countedRenderPassEncoderDrawIndexed(WGPURenderPassEncoder pass,
                                         uint32_t indexCount,
                                         uint32_t instanceCount,
                                         uint32_t firstIndex,
                                         int32_t baseVertex,
                                         uint32_t firstInstance)
{
    recordDraw(pass, instanceCount);
    g_nativeProcs.renderPassEncoderDrawIndexed(pass,
                                               indexCount,
                                               instanceCount,
                                               firstIndex,
                                               baseVertex,
                                               firstInstance);
}

void countedQueueWriteBuffer(WGPUQueue queue,
                             WGPUBuffer buffer,
                             uint64_t offset,
                             const void* data,
                             size_t size)
{
    if (g_activeMetrics != nullptr)
    {
        ++g_activeMetrics->buffer_upload_calls;
        g_activeMetrics->buffer_upload_bytes += size;
    }
    g_nativeProcs.queueWriteBuffer(queue, buffer, offset, data, size);
}

void countedQueueWriteTexture(WGPUQueue queue,
                              const WGPUTexelCopyTextureInfo* destination,
                              const void* data,
                              size_t dataSize,
                              const WGPUTexelCopyBufferLayout* dataLayout,
                              const WGPUExtent3D* writeSize)
{
    if (g_activeMetrics != nullptr)
    {
        ++g_activeMetrics->texture_upload_calls;
        g_activeMetrics->texture_upload_bytes += dataSize;
    }
    g_nativeProcs.queueWriteTexture(
        queue, destination, data, dataSize, dataLayout, writeSize);
}

void countedQueueSubmit(WGPUQueue queue,
                        size_t commandCount,
                        const WGPUCommandBuffer* commands)
{
    if (g_activeMetrics != nullptr)
    {
        ++g_activeMetrics->queue_submissions;
    }
    g_nativeProcs.queueSubmit(queue, commandCount, commands);
}

void installCountingProcs()
{
    static const bool installed = []() {
        g_nativeProcs = dawn::native::GetProcs();
        g_countingProcs = g_nativeProcs;
        g_countingProcs.deviceCreateCommandEncoder =
            countedDeviceCreateCommandEncoder;
        g_countingProcs.commandEncoderBeginRenderPass =
            countedCommandEncoderBeginRenderPass;
        g_countingProcs.deviceCreateBindGroup = countedDeviceCreateBindGroup;
        g_countingProcs.deviceCreateRenderPipeline =
            countedDeviceCreateRenderPipeline;
        g_countingProcs.renderPassEncoderSetPipeline =
            countedRenderPassEncoderSetPipeline;
        g_countingProcs.renderPassEncoderSetBindGroup =
            countedRenderPassEncoderSetBindGroup;
        g_countingProcs.renderPassEncoderDraw = countedRenderPassEncoderDraw;
        g_countingProcs.renderPassEncoderDrawIndexed =
            countedRenderPassEncoderDrawIndexed;
        g_countingProcs.queueWriteBuffer = countedQueueWriteBuffer;
        g_countingProcs.queueWriteTexture = countedQueueWriteTexture;
        g_countingProcs.queueSubmit = countedQueueSubmit;
        dawnProcSetProcs(&g_countingProcs);
        return true;
    }();
    (void)installed;
}
#endif

bool await(WGPUInstance instance, WGPUFuture future)
{
    WGPUFutureWaitInfo futureWait = {};
    futureWait.future = future;
    return wgpuInstanceWaitAny(instance, 1, &futureWait, -1) ==
           WGPUWaitStatus_Success;
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
                     "rive-renderer-ffi: Dawn adapter request failed: %.*s\n",
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
                 "rive-renderer-ffi: Dawn device error: %.*s\n",
                 static_cast<int>(message.length),
                 message.data);
}

struct WorkDoneResult
{
    bool succeeded = false;
};

struct MapResult
{
    bool succeeded = false;
};

void onMap(WGPUMapAsyncStatus status,
           WGPUStringView message,
           void* userdata1,
           void*)
{
    auto* result = static_cast<MapResult*>(userdata1);
    result->succeeded = status == WGPUMapAsyncStatus_Success;
    if (!result->succeeded)
    {
        std::fprintf(stderr,
                     "rive-renderer-ffi: Dawn map failed: %.*s\n",
                     static_cast<int>(message.length),
                     message.data);
    }
}

void onWorkDone(WGPUQueueWorkDoneStatus status,
                WGPUStringView message,
                void* userdata1,
                void*)
{
    auto* result = static_cast<WorkDoneResult*>(userdata1);
    result->succeeded = status == WGPUQueueWorkDoneStatus_Success;
    if (!result->succeeded)
    {
        std::fprintf(stderr,
                     "rive-renderer-ffi: Dawn queue wait failed: %.*s\n",
                     static_cast<int>(message.length),
                     message.data);
    }
}

std::string copyString(WGPUStringView value)
{
    return value.data == nullptr ? std::string()
                                 : std::string(value.data, value.length);
}

class rive_ffi_dawn_context : public rive_ffi_context
{
public:
    ~rive_ffi_dawn_context() override
    {
        renderer.reset();
        target.reset();
        context.reset();
    }

    bool init(uint32_t initialWidth, uint32_t initialHeight)
    {
#ifdef RIVE_FFI_PERF_COUNTERS
        installCountingProcs();
#endif
        constexpr WGPUInstanceFeatureName kTimedWaitAny =
            WGPUInstanceFeatureName_TimedWaitAny;
        WGPUInstanceDescriptor instanceDesc = {};
        instanceDesc.requiredFeatureCount = 1;
        instanceDesc.requiredFeatures = &kTimedWaitAny;
        instance = wgpu::Instance::Acquire(wgpuCreateInstance(&instanceDesc));
        if (!instance)
        {
            return false;
        }

        WGPUAdapter adapterHandle = nullptr;
        WGPURequestAdapterOptions adapterOptions = {};
        adapterOptions.backendType = WGPUBackendType_Metal;
        adapterOptions.powerPreference = WGPUPowerPreference_HighPerformance;
        WGPURequestAdapterCallbackInfo adapterCallback = {};
        adapterCallback.mode = WGPUCallbackMode_WaitAnyOnly;
        adapterCallback.callback = onAdapterRequest;
        adapterCallback.userdata1 = &adapterHandle;
        if (!await(instance.Get(),
                   wgpuInstanceRequestAdapter(instance.Get(),
                                              &adapterOptions,
                                              adapterCallback)) ||
            adapterHandle == nullptr)
        {
            return false;
        }
        adapter = wgpu::Adapter::Acquire(adapterHandle);

        WGPUAdapterInfo adapterInfo = {};
        if (wgpuAdapterGetInfo(adapter.Get(), &adapterInfo) !=
                WGPUStatus_Success ||
            adapterInfo.backendType != WGPUBackendType_Metal)
        {
            wgpuAdapterInfoFreeMembers(adapterInfo);
            return false;
        }
        adapterNameStorage = copyString(adapterInfo.device);
        wgpuAdapterInfoFreeMembers(adapterInfo);
        if (adapterNameStorage.empty())
        {
            return false;
        }

        WGPUDeviceDescriptor deviceDesc = {};
        deviceDesc.uncapturedErrorCallbackInfo.callback = onDeviceError;
        constexpr WGPUFeatureName kClipDistances =
            WGPUFeatureName_ClipDistances;
        const bool supportsClipDistances =
            wgpuAdapterHasFeature(adapter.Get(), kClipDistances);
        deviceDesc.requiredFeatureCount = supportsClipDistances ? 1 : 0;
        deviceDesc.requiredFeatures =
            supportsClipDistances ? &kClipDistances : nullptr;
        device = wgpu::Device::Acquire(
            wgpuAdapterCreateDevice(adapter.Get(), &deviceDesc));
        if (!device)
        {
            return false;
        }
        queue = device.GetQueue();
        context = rive::gpu::RenderContextWebGPUImpl::MakeContext(
            adapter,
            device,
            queue,
            rive::gpu::RenderContextWebGPUImpl::ContextOptions());
        return context != nullptr && ensureTarget(initialWidth, initialHeight);
    }

    bool ensureTarget(uint32_t nextWidth, uint32_t nextHeight) override
    {
        if (context == nullptr || device == nullptr || nextWidth == 0 ||
            nextHeight == 0)
        {
            return false;
        }
        if (target != nullptr && width == nextWidth && height == nextHeight)
        {
            return true;
        }

        wgpu::TextureDescriptor targetDesc = {};
        targetDesc.usage = wgpu::TextureUsage::RenderAttachment |
                           wgpu::TextureUsage::CopySrc |
                           wgpu::TextureUsage::TextureBinding;
        targetDesc.dimension = wgpu::TextureDimension::e2D;
        targetDesc.size = {nextWidth, nextHeight, 1};
        targetDesc.format = wgpu::TextureFormat::RGBA8Unorm;
        targetTexture = device.CreateTexture(&targetDesc);
        if (!targetTexture)
        {
            return false;
        }

        auto* webgpuContext =
            context->static_impl_cast<rive::gpu::RenderContextWebGPUImpl>();
        auto nextTarget = webgpuContext->makeRenderTarget(
            wgpu::TextureFormat::RGBA8Unorm, nextWidth, nextHeight);
        if (nextTarget == nullptr)
        {
            return false;
        }
        nextTarget->setTargetTextureView(targetTexture.CreateView(),
                                         targetTexture);
        target = std::move(nextTarget);
        width = nextWidth;
        height = nextHeight;
        return true;
    }

    void beforeFlush(
        rive::gpu::RenderContext::FlushResources& resources) override
    {
        commandEncoder = device.CreateCommandEncoder();
        resources.externalCommandBuffer = commandEncoder.Get();
    }

    void beginBackendWorkMetrics(bool enabled) override
    {
        lastBackendWorkMetrics = {};
#ifdef RIVE_FFI_PERF_COUNTERS
        currentBackendWorkMetrics = {};
        collectBackendWorkMetrics = enabled;
        g_activeMetrics = enabled ? &currentBackendWorkMetrics : nullptr;
#else
        (void)enabled;
#endif
    }

    bool afterFlush() override
    {
        if (commandEncoder == nullptr)
        {
            return finishBackendWorkMetrics(false);
        }
        wgpu::CommandBuffer commandBuffer = commandEncoder.Finish();
        commandEncoder = nullptr;
        if (commandBuffer == nullptr)
        {
            return finishBackendWorkMetrics(false);
        }
        queue.Submit(1, &commandBuffer);

        WorkDoneResult result;
        WGPUQueueWorkDoneCallbackInfo callback = {};
        callback.mode = WGPUCallbackMode_WaitAnyOnly;
        callback.callback = onWorkDone;
        callback.userdata1 = &result;
        const WGPUFuture future =
            wgpuQueueOnSubmittedWorkDone(queue.Get(), callback);
        return finishBackendWorkMetrics(await(instance.Get(), future) &&
                                        result.succeeded);
    }

    const char* adapterName() const override
    {
        return adapterNameStorage.c_str();
    }

    size_t readPixels(uint8_t* out, size_t len) override
    {
        const size_t packedRowBytes = static_cast<size_t>(width) * 4;
        const size_t expected = packedRowBytes * height;
        if (out == nullptr || len < expected || targetTexture == nullptr ||
            device == nullptr || queue == nullptr)
        {
            return 0;
        }

        constexpr size_t kCopyRowAlignment = 256;
        const uint32_t paddedRowBytes = static_cast<uint32_t>(
            (packedRowBytes + kCopyRowAlignment - 1) &
            ~(kCopyRowAlignment - 1));
        wgpu::BufferDescriptor readbackDesc = {};
        readbackDesc.usage =
            wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::MapRead;
        readbackDesc.size =
            static_cast<uint64_t>(paddedRowBytes) * height;
        wgpu::Buffer readback = device.CreateBuffer(&readbackDesc);
        wgpu::CommandEncoder encoder = device.CreateCommandEncoder();
        if (readback == nullptr || encoder == nullptr)
        {
            return 0;
        }

        wgpu::TexelCopyTextureInfo source = {.texture = targetTexture};
        wgpu::TexelCopyBufferInfo destination = {
            .layout = {.offset = 0,
                       .bytesPerRow = paddedRowBytes,
                       .rowsPerImage = height},
            .buffer = readback,
        };
        wgpu::Extent3D copySize = {
            .width = width, .height = height, .depthOrArrayLayers = 1};
        encoder.CopyTextureToBuffer(&source, &destination, &copySize);
        wgpu::CommandBuffer commandBuffer = encoder.Finish();
        if (commandBuffer == nullptr)
        {
            return 0;
        }
        queue.Submit(1, &commandBuffer);

        MapResult mapResult;
        WGPUBufferMapCallbackInfo callback = {};
        callback.mode = WGPUCallbackMode_WaitAnyOnly;
        callback.callback = onMap;
        callback.userdata1 = &mapResult;
        if (!await(instance.Get(),
                   wgpuBufferMapAsync(readback.Get(),
                                      WGPUMapMode_Read,
                                      0,
                                      readbackDesc.size,
                                      callback)) ||
            !mapResult.succeeded)
        {
            return 0;
        }
        const auto* mapped = static_cast<const uint8_t*>(
            wgpuBufferGetConstMappedRange(readback.Get(),
                                          0,
                                          readbackDesc.size));
        if (mapped == nullptr)
        {
            wgpuBufferUnmap(readback.Get());
            return 0;
        }
        for (uint32_t y = 0; y != height; ++y)
        {
            std::memcpy(out + y * packedRowBytes,
                        mapped + y * paddedRowBytes,
                        packedRowBytes);
        }
        wgpuBufferUnmap(readback.Get());
        return expected;
    }

private:
#ifdef RIVE_FFI_PERF_COUNTERS
    bool finishBackendWorkMetrics(bool succeeded)
    {
        g_activeMetrics = nullptr;
        if (collectBackendWorkMetrics)
        {
            lastBackendWorkMetrics = currentBackendWorkMetrics;
        }
        collectBackendWorkMetrics = false;
        return succeeded;
    }

    rive_ffi_backend_work_metrics currentBackendWorkMetrics = {};
    bool collectBackendWorkMetrics = false;
#else
    bool finishBackendWorkMetrics(bool succeeded) { return succeeded; }
#endif

    wgpu::Instance instance;
    wgpu::Adapter adapter;
    wgpu::Device device;
    wgpu::Queue queue;
    wgpu::Texture targetTexture;
    wgpu::CommandEncoder commandEncoder;
    std::string adapterNameStorage;
};
} // namespace

extern "C" rive_ffi_context* rive_ffi_context_make_dawn(uint32_t width,
                                                         uint32_t height)
{
    auto* ctx = new rive_ffi_dawn_context;
    if (!ctx->init(width, height))
    {
        delete ctx;
        return nullptr;
    }
    return ctx;
}
