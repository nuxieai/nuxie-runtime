#include "rive_renderer_ffi_private.hpp"

#include "rive/renderer/webgpu/render_context_webgpu_impl.hpp"

#include <webgpu/webgpu.h>
#include <webgpu/webgpu_cpp.h>

#include <cstdio>
#include <string>

namespace
{
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

    bool afterFlush() override
    {
        if (commandEncoder == nullptr)
        {
            return false;
        }
        wgpu::CommandBuffer commandBuffer = commandEncoder.Finish();
        commandEncoder = nullptr;
        if (commandBuffer == nullptr)
        {
            return false;
        }
        queue.Submit(1, &commandBuffer);

        WorkDoneResult result;
        WGPUQueueWorkDoneCallbackInfo callback = {};
        callback.mode = WGPUCallbackMode_WaitAnyOnly;
        callback.callback = onWorkDone;
        callback.userdata1 = &result;
        const WGPUFuture future =
            wgpuQueueOnSubmittedWorkDone(queue.Get(), callback);
        return await(instance.Get(), future) && result.succeeded;
    }

    const char* adapterName() const override
    {
        return adapterNameStorage.c_str();
    }

private:
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
