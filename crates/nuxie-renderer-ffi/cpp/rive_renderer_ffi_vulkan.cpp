#include "rive_renderer_ffi_private.hpp"

#include "rive/renderer/vulkan/render_context_vulkan_impl.hpp"
#include "rive/renderer/vulkan/render_target_vulkan.hpp"
#include "rive_vk_bootstrap/vulkan_device.hpp"
#include "rive_vk_bootstrap/vulkan_headless_frame_synchronizer.hpp"
#include "rive_vk_bootstrap/vulkan_instance.hpp"

#include <cstring>
#include <memory>
#include <string>
#include <utility>
#include <vector>

namespace
{
class rive_ffi_vulkan_context final : public rive_ffi_context
{
public:
    ~rive_ffi_vulkan_context() override
    {
        frameSynchronizer.reset();
        target.reset();
        targetImpl = nullptr;
        context.reset();
        device.reset();
        instance.reset();
    }

    bool initialize(uint32_t initialWidth, uint32_t initialHeight)
    {
        instance = rive_vkb::VulkanInstance::Create({
            .appName = "Nuxie exact Vulkan source oracle",
            .idealAPIVersion = VK_API_VERSION_1_3,
            .desiredValidationType = rive_vkb::VulkanValidationType::none,
            .wantDebugCallbacks = false,
        });
        if (instance == nullptr)
        {
            return false;
        }

        device = rive_vkb::VulkanDevice::Create(
            *instance,
            {
                .headless = true,
                .printInitializationMessage = true,
            });
        if (device == nullptr)
        {
            return false;
        }

        context = rive::gpu::RenderContextVulkanImpl::MakeContext(
            instance->vkInstance(),
            device->vkPhysicalDevice(),
            device->vkDevice(),
            device->vulkanFeatures(),
            instance->getVkGetInstanceProcAddrPtr(),
            {});
        if (context == nullptr)
        {
            return false;
        }

        auto* impl =
            context->static_impl_cast<rive::gpu::RenderContextVulkanImpl>();
        VkQueue graphicsQueue = VK_NULL_HANDLE;
        auto getDeviceQueue = reinterpret_cast<PFN_vkGetDeviceQueue>(
            impl->vulkanContext()->GetDeviceProcAddr(device->vkDevice(),
                                                     "vkGetDeviceQueue"));
        if (getDeviceQueue == nullptr)
        {
            return false;
        }
        getDeviceQueue(device->vkDevice(),
                       device->graphicsQueueFamilyIndex(),
                       0,
                       &graphicsQueue);
        impl->setCanvasQueue(graphicsQueue,
                             device->graphicsQueueFamilyIndex());
        adapterNameStorage = device->name();
        return !adapterNameStorage.empty() &&
               ensureTarget(initialWidth, initialHeight);
    }

    bool ensureTarget(uint32_t nextWidth, uint32_t nextHeight) override
    {
        if (context == nullptr || instance == nullptr || device == nullptr ||
            nextWidth == 0 || nextHeight == 0)
        {
            return false;
        }
        if (target != nullptr && frameSynchronizer != nullptr &&
            width == nextWidth && height == nextHeight)
        {
            return true;
        }

        target.reset();
        targetImpl = nullptr;
        frameSynchronizer.reset();
        pixels.clear();

        auto* impl =
            context->static_impl_cast<rive::gpu::RenderContextVulkanImpl>();
        constexpr VkFormat imageFormat = VK_FORMAT_B8G8R8A8_UNORM;
        constexpr VkImageUsageFlags usageFlags =
            VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT |
            VK_IMAGE_USAGE_TRANSFER_SRC_BIT |
            VK_IMAGE_USAGE_TRANSFER_DST_BIT;
        frameSynchronizer =
            rive_vkb::VulkanHeadlessFrameSynchronizer::Create(
                *instance,
                *device,
                ref_rcp(impl->vulkanContext()),
                {
                    .width = nextWidth,
                    .height = nextHeight,
                    .imageFormat = imageFormat,
                    .imageUsageFlags = usageFlags,
                });
        if (frameSynchronizer == nullptr)
        {
            return false;
        }

        auto nextTarget = impl->makeRenderTarget(nextWidth,
                                                 nextHeight,
                                                 imageFormat,
                                                 usageFlags);
        if (nextTarget == nullptr)
        {
            frameSynchronizer.reset();
            return false;
        }
        targetImpl = nextTarget.get();
        target = std::move(nextTarget);
        width = nextWidth;
        height = nextHeight;
        return true;
    }

    void beforeFlush(
        rive::gpu::RenderContext::FlushResources& resources) override
    {
        frameReady = false;
        if (frameSynchronizer == nullptr || targetImpl == nullptr)
        {
            return;
        }
        if (!frameSynchronizer->isFrameStarted() &&
            frameSynchronizer->beginFrame() != VK_SUCCESS)
        {
            return;
        }

        targetImpl->setTargetImageView(frameSynchronizer->vkImageView(),
                                       frameSynchronizer->vkImage(),
                                       frameSynchronizer->lastAccess());
        resources.externalCommandBuffer =
            frameSynchronizer->currentCommandBuffer();
        resources.currentFrameNumber =
            frameSynchronizer->currentFrameNumber();
        resources.safeFrameNumber = frameSynchronizer->safeFrameNumber();
        frameReady = true;
    }

    bool afterFlush() override
    {
        if (!frameReady || frameSynchronizer == nullptr || targetImpl == nullptr)
        {
            return false;
        }
        auto lastAccess = targetImpl->targetLastAccess();
        frameSynchronizer->queueImageCopy(&lastAccess);
        if (frameSynchronizer->endFrame(lastAccess) != VK_SUCCESS)
        {
            return false;
        }
        pixels.clear();
        return frameSynchronizer->getPixelsFromLastImageCopy(&pixels) ==
               VK_SUCCESS;
    }

    const char* adapterName() const override
    {
        return adapterNameStorage.c_str();
    }

    size_t readPixels(uint8_t* out, size_t len) override
    {
        const size_t expected = static_cast<size_t>(width) * height * 4;
        if (out == nullptr || len < expected || pixels.size() != expected)
        {
            return 0;
        }
        std::memcpy(out, pixels.data(), expected);
        return expected;
    }

private:
    std::unique_ptr<rive_vkb::VulkanInstance> instance;
    std::unique_ptr<rive_vkb::VulkanDevice> device;
    std::unique_ptr<rive_vkb::VulkanHeadlessFrameSynchronizer>
        frameSynchronizer;
    rive::gpu::RenderTargetVulkanImpl* targetImpl = nullptr;
    std::vector<uint8_t> pixels;
    std::string adapterNameStorage;
    bool frameReady = false;
};
} // namespace

extern "C" rive_ffi_context* rive_ffi_context_make_vulkan(uint32_t width,
                                                          uint32_t height)
{
    auto context = std::make_unique<rive_ffi_vulkan_context>();
    if (!context->initialize(width, height))
    {
        return nullptr;
    }
    return context.release();
}
