#include "rive_renderer_ffi_private.hpp"

#include "rive/renderer/metal/render_context_metal_impl.h"
#include "rive/renderer/rive_renderer.hpp"

#import <Metal/Metal.h>

#include <stdio.h>

namespace
{
class rive_ffi_metal_context : public rive_ffi_context
{
public:
    bool init(uint32_t initialWidth, uint32_t initialHeight)
    {
        @autoreleasepool
        {
            gpu = MTLCreateSystemDefaultDevice();
            if (gpu == nil)
            {
                return false;
            }
            adapterNameStorage = gpu.name.UTF8String;
            queue = [gpu newCommandQueue];
            if (queue == nil)
            {
                return false;
            }

            context = rive::gpu::RenderContextMetalImpl::MakeContext(gpu);
            if (context == nullptr)
            {
                return false;
            }
            context->static_impl_cast<rive::gpu::RenderContextMetalImpl>()
                ->setCommandQueue(queue);
            return ensureTarget(initialWidth, initialHeight);
        }
    }

    bool ensureTarget(uint32_t nextWidth, uint32_t nextHeight) override
    {
        @autoreleasepool
        {
            if (context == nullptr || gpu == nil || nextWidth == 0 ||
                nextHeight == 0)
            {
                return false;
            }
            if (target != nullptr && width == nextWidth && height == nextHeight)
            {
                return true;
            }

            auto metalTarget =
                context->static_impl_cast<rive::gpu::RenderContextMetalImpl>()
                    ->makeRenderTarget(
                        MTLPixelFormatBGRA8Unorm,
                        nextWidth,
                        nextHeight);
            if (metalTarget == nullptr)
            {
                return false;
            }

            MTLTextureDescriptor* desc = [[MTLTextureDescriptor alloc] init];
            desc.pixelFormat = MTLPixelFormatBGRA8Unorm;
            desc.width = nextWidth;
            desc.height = nextHeight;
            desc.usage = MTLTextureUsageRenderTarget;
            desc.textureType = MTLTextureType2D;
            desc.mipmapLevelCount = 1;
            desc.storageMode = MTLStorageModePrivate;

            id<MTLTexture> texture = [gpu newTextureWithDescriptor:desc];
            if (texture == nil)
            {
                return false;
            }

            metalTarget->setTargetTexture(texture);
            target = metalTarget;
            width = nextWidth;
            height = nextHeight;
            return true;
        }
    }

    void beforeFlush(rive::gpu::RenderContext::FlushResources& resources) override
    {
        @autoreleasepool
        {
            commandBuffer = [queue commandBuffer];
            resources.externalCommandBuffer = (__bridge void*)commandBuffer;
        }
    }

    bool afterFlush() override
    {
        @autoreleasepool
        {
            if (commandBuffer == nil)
            {
                return false;
            }
            [commandBuffer commit];
            [commandBuffer waitUntilCompleted];
            const bool succeeded =
                commandBuffer.status == MTLCommandBufferStatusCompleted;
            commandBuffer = nil;
            return succeeded;
        }
    }

    const char* adapterName() const override
    {
        return adapterNameStorage.c_str();
    }

    size_t readPixels(uint8_t* out, size_t len) override
    {
        @autoreleasepool
        {
            const size_t expected = static_cast<size_t>(width) * height * 4;
            if (out == nullptr || len < expected || target == nullptr ||
                gpu == nil || queue == nil)
            {
                return 0;
            }

            auto* metalTarget =
                static_cast<rive::gpu::RenderTargetMetal*>(target.get());
            id<MTLTexture> texture = metalTarget->targetTexture();
            if (texture == nil)
            {
                return 0;
            }

            id<MTLBuffer> pixelReadBuff =
                [gpu newBufferWithLength:expected
                                 options:MTLResourceStorageModeShared];
            id<MTLCommandBuffer> readCommandBuffer = [queue commandBuffer];
            id<MTLBlitCommandEncoder> blitEncoder =
                [readCommandBuffer blitCommandEncoder];

            [blitEncoder copyFromTexture:texture
                             sourceSlice:0
                             sourceLevel:0
                            sourceOrigin:MTLOriginMake(0, 0, 0)
                              sourceSize:MTLSizeMake(width, height, 1)
                                toBuffer:pixelReadBuff
                       destinationOffset:0
                  destinationBytesPerRow:width * 4
                destinationBytesPerImage:expected];
            [blitEncoder endEncoding];
            [readCommandBuffer commit];
            [readCommandBuffer waitUntilCompleted];

            if (readCommandBuffer.status == MTLCommandBufferStatusError ||
                pixelReadBuff.contents == nullptr)
            {
                return 0;
            }

            const uint8_t* contents =
                reinterpret_cast<const uint8_t*>(pixelReadBuff.contents);
            const size_t rowBytes = static_cast<size_t>(width) * 4;
            for (size_t y = 0; y < height; ++y)
            {
                const uint8_t* src =
                    &contents[(static_cast<size_t>(height) - y - 1) * rowBytes];
                uint8_t* dst = &out[y * rowBytes];
                for (size_t x = 0; x < rowBytes; x += 4)
                {
                    dst[x + 0] = src[x + 2];
                    dst[x + 1] = src[x + 1];
                    dst[x + 2] = src[x + 0];
                    dst[x + 3] = src[x + 3];
                }
            }
            return expected;
        }
    }

private:
    __strong id<MTLDevice> gpu = nil;
    __strong id<MTLCommandQueue> queue = nil;
    __strong id<MTLCommandBuffer> commandBuffer = nil;
    std::string adapterNameStorage;
};
} // namespace

extern "C" rive_ffi_context* rive_ffi_context_make_metal(uint32_t width,
                                                         uint32_t height)
{
    auto* ctx = new rive_ffi_metal_context;
    if (!ctx->init(width, height))
    {
        delete ctx;
        return nullptr;
    }
    return ctx;
}

extern "C" int
rive_ffi_metal_adapter_identity(rive_ffi_adapter_identity* out)
{
    if (out == nullptr)
    {
        return 0;
    }
    @autoreleasepool
    {
        id<MTLDevice> gpu = MTLCreateSystemDefaultDevice();
        if (gpu == nil || gpu.name == nil)
        {
            return 0;
        }
        *out = {};
        NSString* name = gpu.name;
        NSString* vendor = @"unknown";
        for (NSString* candidate in @[@"Apple", @"AMD", @"Intel", @"NVIDIA"])
        {
            if ([name hasPrefix:candidate])
            {
                vendor = candidate;
                break;
            }
        }
        snprintf(out->name, sizeof(out->name), "%s", name.UTF8String);
        snprintf(out->vendor, sizeof(out->vendor), "%s", vendor.UTF8String);
        snprintf(out->device,
                 sizeof(out->device),
                 "registry-id:0x%llx",
                 static_cast<unsigned long long>(gpu.registryID));
        snprintf(out->driver,
                 sizeof(out->driver),
                 "%s",
                 NSProcessInfo.processInfo.operatingSystemVersionString.UTF8String);
        return 1;
    }
}
