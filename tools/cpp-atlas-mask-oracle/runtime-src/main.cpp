// Copyright 2026 Rive
// Temporary source copied into RIVE_RUNTIME_DIR by build.sh.

#include "rive/renderer/rive_renderer.hpp"
#include "rive/renderer/webgpu/render_context_webgpu_impl.hpp"

#include <webgpu/webgpu.h>
#include <webgpu/webgpu_cpp.h>

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
constexpr float kSquareMin = 16;
constexpr float kSquareMax = 48;
constexpr float kStrokeThickness = 8;
constexpr float kFeather = 20;
constexpr uint32_t kAtlasPadding = 2;
constexpr uint32_t kExpectedLogicalAtlasSize = 39;
constexpr uint32_t kExpectedPhysicalAtlasSize = 48;
constexpr uint32_t kMaskHeaderBytes = 20;
constexpr uint32_t kBlitHeaderBytes = 20;
constexpr uint32_t kInputsHeaderBytes = 40;
constexpr uint32_t kBytesPerTexel = 2;
constexpr uint32_t kTessBytesPerTexel = 16;
constexpr uint32_t kCopyRowAlignment = 256;
constexpr uint32_t kExpectedTessWidth = 2048;

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

template <size_t N>
void writeU32(std::array<uint8_t, N>& header, size_t offset, uint32_t value)
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
    std::array<uint8_t, kMaskHeaderBytes> header{};
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

void writeBlit(const char* output,
               uint32_t width,
               uint32_t height,
               const uint8_t* packedRows)
{
    std::array<uint8_t, kBlitHeaderBytes> header{};
    constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'A', 'B', 'L', '\0'};
    std::memcpy(header.data(), kMagic, sizeof(kMagic));
    writeU32(header, 8, 1);
    writeU32(header, 12, width);
    writeU32(header, 16, height);

    std::ofstream file(output, std::ios::binary | std::ios::trunc);
    if (!file)
    {
        fail("could not open atlas-blit output file");
    }
    file.write(reinterpret_cast<const char*>(header.data()), header.size());
    file.write(reinterpret_cast<const char*>(packedRows),
               static_cast<std::streamsize>(width) * height * 4);
    if (!file)
    {
        fail("could not write atlas-blit output file");
    }
}

uint32_t floatBits(float value)
{
    uint32_t bits;
    std::memcpy(&bits, &value, sizeof(bits));
    return bits;
}

void writeInputs(
    const char* output,
    const rive::gpu::RenderContextWebGPUImpl::AtlasMaskOracleFacts& facts,
    uint32_t tessWidth,
    uint32_t tessHeight,
    const uint8_t* paddedRows,
    uint32_t paddedRowBytes)
{
    if (facts.contours.size() != 1)
    {
        fail("expected exactly one production contour record");
    }
    const uint64_t packedRowBytes64 =
        static_cast<uint64_t>(tessWidth) * kTessBytesPerTexel;
    if (packedRowBytes64 > UINT32_MAX)
    {
        fail("tessellation row is too wide to serialize");
    }
    const uint32_t packedRowBytes = static_cast<uint32_t>(packedRowBytes64);
    std::array<uint8_t, kInputsHeaderBytes> header{};
    constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'A', 'T', 'I', '\0'};
    std::memcpy(header.data(), kMagic, sizeof(kMagic));
    writeU32(header, 8, 1);
    writeU32(header, 12, facts.strokeBasePatch);
    writeU32(header, 16, facts.strokePatchCount);
    writeU32(header, 20, static_cast<uint32_t>(facts.contours.size()));
    writeU32(header, 24, tessWidth);
    writeU32(header, 28, tessHeight);
    writeU32(header, 32, 16);
    writeU32(header, 36, kTessBytesPerTexel);

    std::ofstream file(output, std::ios::binary | std::ios::trunc);
    if (!file)
    {
        fail("could not open atlas-input output file");
    }
    file.write(reinterpret_cast<const char*>(header.data()), header.size());
    for (const auto& contour : facts.contours)
    {
        std::array<uint8_t, 16> record{};
        writeU32(record, 0, floatBits(contour.midpointX));
        writeU32(record, 4, floatBits(contour.midpointY));
        writeU32(record, 8, contour.pathID);
        writeU32(record, 12, contour.vertexIndex0);
        file.write(reinterpret_cast<const char*>(record.data()), record.size());
    }
    for (uint32_t y = 0; y != tessHeight; ++y)
    {
        file.write(reinterpret_cast<const char*>(paddedRows + y * paddedRowBytes),
                   packedRowBytes);
    }
    if (!file)
    {
        fail("could not write atlas-input output file");
    }
}

std::vector<uint8_t> readTexture(wgpu::Instance instance,
                                 wgpu::Device device,
                                 wgpu::Queue queue,
                                 wgpu::Texture texture,
                                 uint32_t bytesPerTexel)
{
    const uint32_t width = texture.GetWidth();
    const uint32_t height = texture.GetHeight();
    const uint32_t packedRowBytes = width * bytesPerTexel;
    const uint32_t paddedRowBytes = alignCopyRow(packedRowBytes);
    wgpu::BufferDescriptor readbackDesc = {};
    readbackDesc.usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::MapRead;
    readbackDesc.size = static_cast<uint64_t>(paddedRowBytes) * height;
    wgpu::Buffer readback = device.CreateBuffer(&readbackDesc);
    wgpu::CommandEncoder encoder = device.CreateCommandEncoder();
    wgpu::TexelCopyTextureInfo source = {.texture = texture};
    wgpu::TexelCopyBufferInfo destination = {
        .layout = {.offset = 0, .bytesPerRow = paddedRowBytes, .rowsPerImage = height},
        .buffer = readback,
    };
    wgpu::Extent3D copySize = {.width = width, .height = height, .depthOrArrayLayers = 1};
    encoder.CopyTextureToBuffer(&source, &destination, &copySize);
    wgpu::CommandBuffer commands = encoder.Finish();
    queue.Submit(1, &commands);

    MapResult mapResult;
    WGPUBufferMapCallbackInfo mapCallback = {};
    mapCallback.mode = WGPUCallbackMode_WaitAnyOnly;
    mapCallback.callback = onMap;
    mapCallback.userdata1 = &mapResult;
    await(instance.Get(), wgpuBufferMapAsync(readback.Get(), WGPUMapMode_Read, 0,
                                              readbackDesc.size, mapCallback));
    if (!mapResult.succeeded)
    {
        fail("could not map texture readback buffer");
    }
    const uint8_t* mapped = static_cast<const uint8_t*>(
        wgpuBufferGetConstMappedRange(readback.Get(), 0, readbackDesc.size));
    if (mapped == nullptr)
    {
        fail("mapped texture readback buffer has no range");
    }
    std::vector<uint8_t> packed(static_cast<size_t>(packedRowBytes) * height);
    for (uint32_t y = 0; y != height; ++y)
    {
        std::memcpy(packed.data() + y * packedRowBytes,
                    mapped + y * paddedRowBytes,
                    packedRowBytes);
    }
    wgpuBufferUnmap(readback.Get());
    return packed;
}
} // namespace

int main(int argc, char** argv)
{
    const char* output = argc > 1 ? argv[1] : "atlas-mask.r16f";
    const char* inputsOutput = argc > 2 ? argv[2] : "atlas-inputs.bin";
    const char* blitOutput = argc > 3 ? argv[3] : "atlas-blit.rgba";
    const bool fillCase = argc > 4 && std::strcmp(argv[4], "fill") == 0;
    if (argc > 5 || (argc > 4 && !fillCase))
    {
        fail("usage: rive_atlas_mask_oracle [mask-output] [inputs-output] [blit-output] [fill]");
    }

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
    targetDesc.usage =
        wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::CopySrc;
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
    if (fillCase)
    {
        path->fillRule(rive::FillRule::clockwise);
        constexpr float kControlOffset = 8.83064f;
        constexpr float kCenter = (kSquareMin + kSquareMax) * .5f;
        path->moveTo(kSquareMax, kCenter);
        path->cubicTo(kSquareMax,
                      kCenter + kControlOffset,
                      kCenter + kControlOffset,
                      kSquareMax,
                      kCenter,
                      kSquareMax);
        path->cubicTo(kCenter - kControlOffset,
                      kSquareMax,
                      kSquareMin,
                      kCenter + kControlOffset,
                      kSquareMin,
                      kCenter);
        path->cubicTo(kSquareMin,
                      kCenter - kControlOffset,
                      kCenter - kControlOffset,
                      kSquareMin,
                      kCenter,
                      kSquareMin);
        path->cubicTo(kCenter + kControlOffset,
                      kSquareMin,
                      kSquareMax,
                      kCenter - kControlOffset,
                      kSquareMax,
                      kCenter);
    }
    else
    {
        // Exact fixed_feather_atlas_mask() geometry from Rust's env test.
        path->moveTo(kSquareMin, kSquareMin);
        path->lineTo(kSquareMax, kSquareMin);
        path->lineTo(kSquareMax, kSquareMax);
        path->lineTo(kSquareMin, kSquareMax);
    }
    path->close();
    auto paint = context->makeRenderPaint();
    paint->style(fillCase ? rive::RenderPaintStyle::fill
                          : rive::RenderPaintStyle::stroke);
    if (!fillCase)
    {
        paint->thickness(kStrokeThickness);
        paint->join(rive::StrokeJoin::miter);
        paint->cap(rive::StrokeCap::butt);
    }
    paint->feather(kFeather);
    paint->color(0xffffffff);
    renderer.drawPath(path.get(), paint.get());

    wgpu::CommandEncoder renderEncoder = device.CreateCommandEncoder();
    context->flush({.renderTarget = target.get(),
                    .externalCommandBuffer = renderEncoder.Get()});
    wgpu::CommandBuffer renderCommands = renderEncoder.Finish();
    queue.Submit(1, &renderCommands);

    const auto& facts = webgpuContext->atlasMaskFactsForOracle();
    std::printf("draw schedule: interlock=%u fixedFunctionColorOutput=%d batches=%zu\n",
                facts.interlockMode,
                facts.fixedFunctionColorOutput,
                facts.drawBatches.size());
    for (const auto& batch : facts.drawBatches)
    {
        std::printf("  drawType=%u shaderFeatures=0x%x shaderMiscFlags=0x%x\n",
                    batch.drawType,
                    batch.shaderFeatures,
                    batch.shaderMiscFlags);
    }
    if (facts.interlockMode !=
            static_cast<uint32_t>(rive::gpu::InterlockMode::msaa) ||
        !facts.fixedFunctionColorOutput || facts.drawBatches.size() != 1 ||
        facts.drawBatches.front().drawType !=
            static_cast<uint32_t>(rive::gpu::DrawType::atlasBlit) ||
        facts.drawBatches.front().shaderFeatures != static_cast<uint32_t>(
            rive::gpu::ShaderFeatures::ENABLE_DITHER) ||
        facts.drawBatches.front().shaderMiscFlags != static_cast<uint32_t>(
            rive::gpu::ShaderMiscFlags::fixedFunctionColorOutput))
    {
        fail("final atlas-blit oracle must execute one fixed-function MSAA atlas batch");
    }
    if (facts.contentWidth != kExpectedLogicalAtlasSize ||
        facts.contentHeight != kExpectedLogicalAtlasSize ||
        !facts.pathTransformValid || facts.pathTranslateX != kAtlasPadding ||
        facts.pathTranslateY != kAtlasPadding || facts.strokeBatchCount != 1 ||
        facts.atlasBatchIsStroke == fillCase ||
        facts.strokeScissor.left != 0 || facts.strokeScissor.top != 0 ||
        facts.strokeScissor.right != kExpectedLogicalAtlasSize ||
        facts.strokeScissor.bottom != kExpectedLogicalAtlasSize)
    {
        std::fprintf(
            stderr,
            "cpp-atlas-mask-oracle: production flush contract drift: content=%ux%u transformValid=%d translation=(%g,%g) strokeBatches=%zu scissor=[%u,%u,%u,%u]\n",
            facts.contentWidth,
            facts.contentHeight,
            facts.pathTransformValid,
            facts.pathTranslateX,
            facts.pathTranslateY,
            facts.strokeBatchCount,
            facts.strokeScissor.left,
            facts.strokeScissor.top,
            facts.strokeScissor.right,
            facts.strokeScissor.bottom);
        return 1;
    }
    const wgpu::Texture atlas = webgpuContext->atlasMaskTextureForOracle();
    const uint32_t atlasWidth = atlas.GetWidth();
    const uint32_t atlasHeight = atlas.GetHeight();
    if (atlasWidth != kExpectedPhysicalAtlasSize ||
        atlasHeight != kExpectedPhysicalAtlasSize)
    {
        std::fprintf(stderr,
                     "cpp-atlas-mask-oracle: expected physical=48x48 logical=39x39 placement=(2,2) frame=64x64, got physical=%ux%u\n",
                     atlasWidth,
                     atlasHeight);
        return 1;
    }
    const std::vector<uint8_t> atlasBytes =
        readTexture(instance, device, queue, atlas, kBytesPerTexel);
    writeMask(output,
              atlasWidth,
              atlasHeight,
              atlasBytes.data(),
              atlasWidth * kBytesPerTexel);

    const wgpu::Texture tessellation =
        webgpuContext->tessellationTextureForOracle();
    const uint32_t tessWidth = tessellation.GetWidth();
    const uint32_t tessHeight = tessellation.GetHeight();
    if (tessWidth != kExpectedTessWidth || tessHeight == 0)
    {
        std::fprintf(stderr,
                     "cpp-atlas-mask-oracle: unexpected tessellation texture dimensions %ux%u\n",
                     tessWidth,
                     tessHeight);
        return 1;
    }
    const std::vector<uint8_t> tessellationBytes =
        readTexture(instance, device, queue, tessellation, kTessBytesPerTexel);
    writeInputs(inputsOutput,
                facts,
                tessWidth,
                tessHeight,
                tessellationBytes.data(),
                tessWidth * kTessBytesPerTexel);
    const std::vector<uint8_t> targetBytes =
        readTexture(instance, device, queue, targetTexture, 4);
    writeBlit(blitOutput,
              kFrameWidth,
              kFrameHeight,
              targetBytes.data());
    std::printf("wrote %s: %ux%u R16Float row-packed atlas mask\\n",
                output,
                atlasWidth,
                atlasHeight);
    std::printf("wrote %s: batch=%u+%u contours=%zu tessellation=%ux%u RGBA32Uint\\n",
                inputsOutput,
                facts.strokeBasePatch,
                facts.strokePatchCount,
                facts.contours.size(),
                tessWidth,
                tessHeight);
    std::printf("wrote %s: %ux%u tightly packed RGBA8 atlas blit\n",
                blitOutput,
                kFrameWidth,
                kFrameHeight);
    return 0;
}
