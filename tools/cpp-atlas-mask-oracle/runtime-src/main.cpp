// Copyright 2026 Rive
// Temporary source copied into RIVE_RUNTIME_DIR by build.sh.

#include "rive/renderer/rive_renderer.hpp"
#include "rive/renderer/webgpu/render_context_webgpu_impl.hpp"
#include "rive_render_path.hpp"

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
constexpr uint32_t kTessSpanHeaderBytes = 28;
constexpr uint32_t kDirectGridHeaderBytes = 64;
constexpr uint32_t kBytesPerTexel = 2;
constexpr uint32_t kTessBytesPerTexel = 16;
constexpr uint32_t kCopyRowAlignment = 256;
constexpr uint32_t kExpectedTessWidth = 2048;
constexpr uint32_t kExpectedPolySharkPatchCount = 786;
constexpr uint32_t kExpectedPolySharkContourCount = 1;
constexpr uint32_t kExpectedPolySharkTessHeight = 5;
constexpr uint32_t kDirectGridFrameSize = 1000;
constexpr uint32_t kDirectGridContourCount = 100;
constexpr uint32_t kDirectFlowerContourCount = 2;
constexpr uint32_t kDirectBadSkinFrameWidth = 999;
constexpr uint32_t kDirectBadSkinFrameHeight = 720;
constexpr uint32_t kDirectBadSkinContourCount = 1;
constexpr uint32_t kDirectStrokesRoundFrameSize = 400;
constexpr float kDirectStrokesRoundThickness = 4.5f;
constexpr uint32_t kIntersectionGroupBatchCount = 9;
#include "generated_polyshark_path.inc"

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

void addIntersectionGroupRect(rive::RenderPath* path,
                              float left,
                              float top,
                              float right,
                              float bottom)
{
    path->moveTo(left, top);
    path->lineTo(right, top);
    path->lineTo(right, bottom);
    path->lineTo(left, bottom);
    path->close();
}

uint32_t floatBits(float value);

void writeSoftenedPath(const char* output, const rive::RawPath& path)
{
    std::array<uint8_t, 20> header{};
    constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'S', 'F', 'T', '\0'};
    std::memcpy(header.data(), kMagic, sizeof(kMagic));
    writeU32(header, 8, 1);
    writeU32(header, 12, static_cast<uint32_t>(path.verbs().size()));
    writeU32(header, 16, static_cast<uint32_t>(path.points().size()));
    std::ofstream file(output, std::ios::binary | std::ios::trunc);
    if (!file)
    {
        fail("could not open softened-path output file");
    }
    file.write(reinterpret_cast<const char*>(header.data()), header.size());
    for (rive::PathVerb verb : path.verbs())
    {
        std::array<uint8_t, 4> value{};
        writeU32(value, 0, static_cast<uint32_t>(verb));
        file.write(reinterpret_cast<const char*>(value.data()), value.size());
    }
    for (const rive::Vec2D& point : path.points())
    {
        std::array<uint8_t, 8> value{};
        writeU32(value, 0, floatBits(point.x));
        writeU32(value, 4, floatBits(point.y));
        file.write(reinterpret_cast<const char*>(value.data()), value.size());
    }
    if (!file)
    {
        fail("could not write softened-path output file");
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
    if (facts.contours.empty())
    {
        fail("expected at least one production contour record");
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

void writeTessVertexSpans(
    const char* output,
    const rive::gpu::RenderContextWebGPUImpl::AtlasMaskOracleFacts& facts)
{
    if (facts.firstTessVertexSpan > UINT32_MAX ||
        facts.tessVertexSpans.empty() ||
        facts.tessVertexSpans.size() > UINT32_MAX)
    {
        fail("tessellation-span facts are incomplete or too large");
    }
    static_assert(sizeof(rive::gpu::TessVertexSpan) == 64);
    std::array<uint8_t, kTessSpanHeaderBytes> header{};
    constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'A', 'T', 'S', '\0'};
    std::memcpy(header.data(), kMagic, sizeof(kMagic));
    writeU32(header, 8, 1);
    writeU32(header, 12, kTessSpanHeaderBytes);
    writeU32(header, 16, static_cast<uint32_t>(facts.firstTessVertexSpan));
    writeU32(header, 20, static_cast<uint32_t>(facts.tessVertexSpans.size()));
    writeU32(header, 24, sizeof(rive::gpu::TessVertexSpan));

    std::ofstream file(output, std::ios::binary | std::ios::trunc);
    if (!file)
    {
        fail("could not open tessellation-span output file");
    }
    file.write(reinterpret_cast<const char*>(header.data()), header.size());
    for (const auto& span : facts.tessVertexSpans)
    {
        std::array<uint8_t, 64> record{};
        for (size_t point = 0; point != 4; ++point)
        {
            writeU32(record, point * 8, floatBits(span.pts[point].x));
            writeU32(record, point * 8 + 4, floatBits(span.pts[point].y));
        }
        writeU32(record, 32, floatBits(span.joinTangent.x));
        writeU32(record, 36, floatBits(span.joinTangent.y));
        writeU32(record, 40, floatBits(span.y));
        writeU32(record, 44, floatBits(span.reflectionY));
        writeU32(record, 48, static_cast<uint32_t>(span.x0x1));
        writeU32(record, 52, static_cast<uint32_t>(span.reflectionX0X1));
        writeU32(record, 56, span.segmentCounts);
        writeU32(record, 60, span.contourIDWithFlags);
        file.write(reinterpret_cast<const char*>(record.data()), record.size());
    }
    if (!file)
    {
        fail("could not write tessellation-span output file");
    }
}

// RIVEDGI/RIVEDFI v1 is a little-endian, tightly packed direct preparation
// snapshot. Header: magic[8], version, headerBytes, flags, interlockMode,
// drawBatchCount, tessWidth, tessHeight, contourCount, triangleVertexCount,
// drawBatchStride, contourStride, triangleVertexStride, tessTexelStride, and
// reserved. It is followed by draw batches (5 u32), contours (XY float bits,
// path ID, vertex index), interior triangle vertices (XY float bits, packed
// weight/path ID), then the complete RGBA32Uint tessellation texture.
void writeDirectInputs(
    const char* output,
    const rive::gpu::RenderContextWebGPUImpl::AtlasMaskOracleFacts& facts,
    uint32_t tessWidth,
    uint32_t tessHeight,
    const uint8_t* paddedRows,
    uint32_t paddedRowBytes,
    const char magic[8],
    uint32_t expectedContourCount)
{
    if (facts.contours.size() != expectedContourCount ||
        facts.triangles.empty() || facts.triangles.size() % 3 != 0 ||
        facts.drawBatches.size() < 3)
    {
        fail("direct preparation facts are incomplete");
    }
    const uint64_t packedRowBytes64 =
        static_cast<uint64_t>(tessWidth) * kTessBytesPerTexel;
    if (packedRowBytes64 > UINT32_MAX ||
        facts.drawBatches.size() > UINT32_MAX ||
        facts.contours.size() > UINT32_MAX || facts.triangles.size() > UINT32_MAX)
    {
        fail("direct preparation data is too large to serialize");
    }
    const uint32_t packedRowBytes = static_cast<uint32_t>(packedRowBytes64);
    std::array<uint8_t, kDirectGridHeaderBytes> header{};
    std::memcpy(header.data(), magic, 8);
    writeU32(header, 8, 1);
    writeU32(header, 12, kDirectGridHeaderBytes);
    writeU32(header, 16, 1); // bit 0: clockwiseFillOverride was enabled.
    writeU32(header, 20, facts.interlockMode);
    writeU32(header, 24, static_cast<uint32_t>(facts.drawBatches.size()));
    writeU32(header, 28, tessWidth);
    writeU32(header, 32, tessHeight);
    writeU32(header, 36, static_cast<uint32_t>(facts.contours.size()));
    writeU32(header, 40, static_cast<uint32_t>(facts.triangles.size()));
    writeU32(header, 44, 20);
    writeU32(header, 48, 16);
    writeU32(header, 52, 12);
    writeU32(header, 56, kTessBytesPerTexel);
    writeU32(header, 60, 0);

    std::ofstream file(output, std::ios::binary | std::ios::trunc);
    if (!file)
    {
        fail("could not open direct input output file");
    }
    file.write(reinterpret_cast<const char*>(header.data()), header.size());
    for (const auto& batch : facts.drawBatches)
    {
        std::array<uint8_t, 20> record{};
        writeU32(record, 0, batch.drawType);
        writeU32(record, 4, batch.shaderFeatures);
        writeU32(record, 8, batch.shaderMiscFlags);
        writeU32(record, 12, batch.baseElement);
        writeU32(record, 16, batch.elementCount);
        file.write(reinterpret_cast<const char*>(record.data()), record.size());
    }
    for (const auto& contour : facts.contours)
    {
        std::array<uint8_t, 16> record{};
        writeU32(record, 0, floatBits(contour.midpointX));
        writeU32(record, 4, floatBits(contour.midpointY));
        writeU32(record, 8, contour.pathID);
        writeU32(record, 12, contour.vertexIndex0);
        file.write(reinterpret_cast<const char*>(record.data()), record.size());
    }
    for (const auto& triangle : facts.triangles)
    {
        std::array<uint8_t, 12> record{};
        writeU32(record, 0, floatBits(triangle.x));
        writeU32(record, 4, floatBits(triangle.y));
        writeU32(record, 8, triangle.weightPathID);
        file.write(reinterpret_cast<const char*>(record.data()), record.size());
    }
    for (uint32_t y = 0; y != tessHeight; ++y)
    {
        file.write(reinterpret_cast<const char*>(paddedRows + y * paddedRowBytes),
                   packedRowBytes);
    }
    if (!file)
    {
        fail("could not write direct input output file");
    }
}

void writeDirectGridInputs(
    const char* output,
    const rive::gpu::RenderContextWebGPUImpl::AtlasMaskOracleFacts& facts,
    uint32_t tessWidth,
    uint32_t tessHeight,
    const uint8_t* paddedRows,
    uint32_t paddedRowBytes)
{
    constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'D', 'G', 'I', '\0'};
    writeDirectInputs(output,
                      facts,
                      tessWidth,
                      tessHeight,
                      paddedRows,
                      paddedRowBytes,
                      kMagic,
                      kDirectGridContourCount);
}

void writeDirectFlowerInputs(
    const char* output,
    const rive::gpu::RenderContextWebGPUImpl::AtlasMaskOracleFacts& facts,
    uint32_t tessWidth,
    uint32_t tessHeight,
    const uint8_t* paddedRows,
    uint32_t paddedRowBytes)
{
    constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'D', 'F', 'I', '\0'};
    writeDirectInputs(output,
                      facts,
                      tessWidth,
                      tessHeight,
                      paddedRows,
                      paddedRowBytes,
                      kMagic,
                      kDirectFlowerContourCount);
}

void writeDirectBadSkinInputs(
    const char* output,
    const rive::gpu::RenderContextWebGPUImpl::AtlasMaskOracleFacts& facts,
    uint32_t tessWidth,
    uint32_t tessHeight,
    const uint8_t* paddedRows,
    uint32_t paddedRowBytes)
{
    constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'D', 'B', 'I', '\0'};
    writeDirectInputs(output,
                      facts,
                      tessWidth,
                      tessHeight,
                      paddedRows,
                      paddedRowBytes,
                      kMagic,
                      kDirectBadSkinContourCount);
}

void addClockwiseNestedGrid(rive::RenderPath* path)
{
    // Source citation: fixtures/renderer/streams/gm/
    // largeclippedpath_clockwise_nested.rive-stream:10 serializes these 100
    // 20px strips, 50 horizontal then 50 vertical, with alternating winding.
    for (uint32_t y = 0; y != kDirectGridFrameSize; y += 20)
    {
        if ((y / 20) % 2 == 0)
        {
            path->moveTo(0, static_cast<float>(y));
            path->lineTo(0, static_cast<float>(y + 20));
            path->lineTo(kDirectGridFrameSize, static_cast<float>(y + 20));
            path->lineTo(kDirectGridFrameSize, static_cast<float>(y));
        }
        else
        {
            path->moveTo(0, static_cast<float>(y));
            path->lineTo(kDirectGridFrameSize, static_cast<float>(y));
            path->lineTo(kDirectGridFrameSize, static_cast<float>(y + 20));
            path->lineTo(0, static_cast<float>(y + 20));
        }
        path->close();
    }
    for (uint32_t x = 0; x != kDirectGridFrameSize; x += 20)
    {
        if ((x / 20) % 2 == 0)
        {
            path->moveTo(static_cast<float>(x), 0);
            path->lineTo(static_cast<float>(x), kDirectGridFrameSize);
            path->lineTo(static_cast<float>(x + 20), kDirectGridFrameSize);
            path->lineTo(static_cast<float>(x + 20), 0);
        }
        else
        {
            path->moveTo(static_cast<float>(x), 0);
            path->lineTo(static_cast<float>(x + 20), 0);
            path->lineTo(static_cast<float>(x + 20), kDirectGridFrameSize);
            path->lineTo(static_cast<float>(x), kDirectGridFrameSize);
        }
        path->close();
    }
}

void addClockwiseNestedFlower(rive::RenderPath* path)
{
    // Source citation: fixtures/renderer/streams/gm/
    // largeclippedpath_clockwise_nested.rive-stream:7 serializes this exact
    // 9-cubic flower followed by its inner 4-cubic oval.
    path->moveTo(833.333374f, 500.f);
    path->cubicTo(1035.17468f,
                  626.838745f,
                  991.497986f,
                  746.839539f,
                  755.348145f,
                  714.262573f);
    path->cubicTo(828.437256f,
                  941.167725f,
                  717.843933f,
                  1005.01886f,
                  557.88269f,
                  828.269287f);
    path->cubicTo(468.020355f,
                  1049.06946f,
                  342.258209f,
                  1026.89429f,
                  333.333313f,
                  788.67511f);
    path->cubicTo(122.567162f,
                  900.055542f,
                  40.4816399f,
                  802.229858f,
                  186.769104f,
                  614.006653f);
    path->cubicTo(-46.2811317f,
                  563.851013f,
                  -46.2810974f,
                  436.148895f,
                  186.769135f,
                  385.993195f);
    path->cubicTo(40.4817047f,
                  197.77002f,
                  122.567123f,
                  99.9444427f,
                  333.333344f,
                  211.324844f);
    path->cubicTo(342.258423f,
                  -26.8943138f,
                  468.020172f,
                  -49.0694847f,
                  557.882874f,
                  171.730774f);
    path->cubicTo(717.843994f,
                  -5.0188098f,
                  828.437256f,
                  58.8322334f,
                  755.348206f,
                  285.737518f);
    path->cubicTo(991.497986f,
                  253.160507f,
                  1035.17468f,
                  373.161469f,
                  833.333374f,
                  500.000061f);
    path->close();

    path->moveTo(750.f, 500.f);
    path->cubicTo(750.f, 637.97876f, 637.97876f, 750.f, 500.f, 750.f);
    path->cubicTo(362.02124f, 750.f, 250.f, 637.97876f, 250.f, 500.f);
    path->cubicTo(250.f, 362.02124f, 362.02124f, 250.f, 500.f, 250.f);
    path->cubicTo(637.97876f, 250.f, 750.f, 362.02124f, 750.f, 500.f);
    path->close();
}

void addBadSkinHair(rive::RenderPath* path)
{
    // Exact fixture path: fixtures/renderer/streams/riv/bad_skin.rive-stream:330.
    path->moveTo(-81.5608521f, -65.8947601f);
    path->cubicTo(-81.5608521f, -65.8947601f, -105.346954f, -143.298767f,
                  -31.1603794f, -150.845825f);
    path->cubicTo(-10.2647314f, -146.784561f, -5.52482748f, -138.151382f,
                  3.95540905f, -135.285233f);
    path->cubicTo(13.435585f, -132.419128f, 45.7343864f, -139.608963f,
                  67.0147171f, -116.697594f);
    path->cubicTo(88.295105f, -93.7862701f, 101.926819f, -73.546402f,
                  114.122017f, -9.98956394f);
    path->cubicTo(127.370155f, 43.0014648f, 179.42717f, 98.3242035f,
                  219.327179f, 105.924187f);
    path->cubicTo(259.227203f, 113.524231f, 219.327179f, 296.024231f,
                  219.327179f, 296.024231f);
    path->cubicTo(219.327179f, 296.024231f, 14.4906435f, 346.763397f,
                  -28.0207996f, 347.893097f);
    path->cubicTo(-70.5313416f, 348.922821f, -254.072845f, 382.324249f,
                  -283.272827f, 345.224243f);
    path->cubicTo(-312.472809f, 308.024261f, -167.772812f, 149.824234f,
                  -167.772812f, 149.824234f);
    path->cubicTo(-167.772812f, 149.824234f, -173.372818f, 117.124199f,
                  -171.672791f, 98.2241821f);
    path->cubicTo(-169.270737f, 76.4746246f, -171.432159f, 54.370945f,
                  -137.79129f, 30.8296814f);
    path->cubicTo(-104.150368f, 7.28838682f, -136.469254f, -24.0374756f,
                  -120.125793f, -40.795002f);
    path->cubicTo(-98.6691818f, -62.6391525f, -81.5608521f, -65.8947601f,
                  -81.5608521f, -65.8947601f);
    path->close();
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
    const bool circleCase = argc > 4 && std::strcmp(argv[4], "fill") == 0;
    const bool cuspCase = argc > 4 && std::strcmp(argv[4], "cusp") == 0;
    const bool directCuspCase =
        argc > 4 && std::strcmp(argv[4], "direct-cusp") == 0;
    const bool directPolySharkCase =
        argc > 4 && std::strcmp(argv[4], "direct-polyshark") == 0;
    const bool directGridCase = argc > 4 && std::strcmp(argv[4], "direct-grid") == 0;
    const bool directFlowerCase =
        argc > 4 && std::strcmp(argv[4], "direct-flower") == 0;
    const bool directBadSkinCase =
        argc > 4 && std::strcmp(argv[4], "direct-bad-skin") == 0;
    const bool directStrokesRoundCase =
        argc > 4 && std::strcmp(argv[4], "direct-strokes-round") == 0;
    const bool clippedCase = argc > 4 && std::strcmp(argv[4], "clipped") == 0;
    const bool pathClippedCase =
        argc > 4 && std::strcmp(argv[4], "path-clipped") == 0;
    const bool changingPathClippedCase =
        argc > 4 && std::strcmp(argv[4], "changing-path-clipped") == 0;
    const bool nestedPathClippedCase =
        argc > 4 && std::strcmp(argv[4], "nested-path-clipped") == 0;
    const bool nestedEvenOddPathClippedCase =
        argc > 4 && std::strcmp(argv[4], "nested-evenodd-path-clipped") == 0;
    const bool nestedClockwisePathClippedCase =
        argc > 4 && std::strcmp(argv[4], "nested-clockwise-path-clipped") == 0;
    const bool advancedBlendCase =
        argc > 4 && std::strcmp(argv[4], "advanced-blend") == 0;
    const bool atomicAdvancedBlendCase =
        argc > 4 && std::strcmp(argv[4], "atomic-advanced-blend") == 0;
    const bool intersectionGroupsCase =
        argc > 4 && std::strcmp(argv[4], "msaa-intersection-groups") == 0;
    const bool anyAdvancedBlendCase =
        advancedBlendCase || atomicAdvancedBlendCase;
    const bool directTriangulatedCase =
        directGridCase || directFlowerCase || directBadSkinCase;
    const bool directCase =
        directCuspCase || directPolySharkCase || directTriangulatedCase ||
        directStrokesRoundCase;
    const bool directOutputCase = directCase || atomicAdvancedBlendCase;
    const bool fillCase = circleCase || cuspCase ||
                          (directCase && !directStrokesRoundCase) ||
                          anyAdvancedBlendCase ||
                          intersectionGroupsCase ||
                          nestedEvenOddPathClippedCase ||
                          nestedClockwisePathClippedCase;
    const char* auxiliaryOutput = argc > 5 ? argv[5] : nullptr;
    if (argc > 6 ||
        (argc > 4 && !fillCase && !directStrokesRoundCase && !clippedCase &&
         !pathClippedCase &&
         !changingPathClippedCase && !nestedPathClippedCase &&
         !nestedEvenOddPathClippedCase && !nestedClockwisePathClippedCase &&
         !advancedBlendCase && !atomicAdvancedBlendCase &&
         !intersectionGroupsCase) ||
        (auxiliaryOutput != nullptr && !cuspCase && !directStrokesRoundCase) ||
        (directStrokesRoundCase && auxiliaryOutput == nullptr))
    {
        fail("usage: rive_atlas_mask_oracle [mask-output] [inputs-output] [blit-output] [fill|cusp|clipped|path-clipped|changing-path-clipped|nested-path-clipped|nested-evenodd-path-clipped|nested-clockwise-path-clipped|advanced-blend|atomic-advanced-blend|msaa-intersection-groups|direct-cusp|direct-polyshark|direct-grid|direct-flower|direct-bad-skin|direct-strokes-round] [softened-or-tess-span-output]");
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
    constexpr WGPUFeatureName kClipDistances = WGPUFeatureName_ClipDistances;
    if (clippedCase && !wgpuAdapterHasFeature(adapter.Get(), kClipDistances))
    {
        fail("Dawn adapter does not support clip distances");
    }
    deviceDesc.requiredFeatureCount = clippedCase ? 1 : 0;
    deviceDesc.requiredFeatures = clippedCase ? &kClipDistances : nullptr;
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
    const uint32_t frameWidth = directStrokesRoundCase
                                    ? kDirectStrokesRoundFrameSize
                                : directBadSkinCase
                                    ? kDirectBadSkinFrameWidth
                                    : (directTriangulatedCase ? kDirectGridFrameSize
                                                               : kFrameWidth);
    const uint32_t frameHeight = directStrokesRoundCase
                                     ? kDirectStrokesRoundFrameSize
                                 : directBadSkinCase
                                     ? kDirectBadSkinFrameHeight
                                     : (directTriangulatedCase ? kDirectGridFrameSize
                                                                : kFrameHeight);
    targetDesc.size = {frameWidth, frameHeight, 1};
    targetDesc.format = wgpu::TextureFormat::RGBA8Unorm;
    wgpu::Texture targetTexture = device.CreateTexture(&targetDesc);
    auto target = webgpuContext->makeRenderTarget(
        wgpu::TextureFormat::RGBA8Unorm,
        frameWidth,
        frameHeight);
    target->setTargetTextureView(targetTexture.CreateView(), targetTexture);

    context->beginFrame({.renderTargetWidth = frameWidth,
                         .renderTargetHeight = frameHeight,
                         .loadAction = rive::gpu::LoadAction::clear,
                         .clearColor = directStrokesRoundCase
                                           ? 0xffffffff
                                           : (anyAdvancedBlendCase ? 0xff204080 : 0),
                         .msaaSampleCount = directOutputCase ? 0u : 4u,
                         .disableRasterOrdering = directOutputCase,
                         .clockwiseFillOverride = directOutputCase});
    rive::RiveRenderer renderer(context.get());
    auto path = context->makeEmptyRenderPath();
    // bad_skin preserves its stream-authored nonzero fill rule while the frame
    // still enables clockwiseFillOverride for atomic preparation.
    if (fillCase && !directBadSkinCase)
    {
        path->fillRule(rive::FillRule::clockwise);
    }
    if (directStrokesRoundCase)
    {
        // Source: strokes_round.rive-stream draw 38. The clip is intentionally
        // omitted because the isolated seam residual is unchanged without it.
        path->moveTo(25.5016327f, 70.300293f);
        path->lineTo(67.7646637f, 70.300293f);
        path->cubicTo(79.4274673f,
                      70.300293f,
                      88.8961792f,
                      80.9101868f,
                      88.8961792f,
                      89.5240784f);
        path->lineTo(88.8961792f, 127.971649f);
        path->cubicTo(88.8961792f,
                      138.581543f,
                      79.4274673f,
                      147.195435f,
                      67.7646637f,
                      147.195435f);
        path->lineTo(25.5016327f, 147.195435f);
        path->cubicTo(16.0329189f,
                      147.195435f,
                      4.37011719f,
                      138.581543f,
                      4.37011719f,
                      127.971649f);
        path->lineTo(4.37011719f, 89.5240784f);
        path->cubicTo(4.37011719f,
                      80.9101868f,
                      16.0329189f,
                      70.300293f,
                      25.5016327f,
                      70.300293f);
        path->close();
    }
    else if (directCuspCase)
    {
        renderer.transform(
            rive::Mat2D(1.46300006f, 0, 0, 1.46300006f, -40, -20));
        path->moveTo(0, 100);
        path->moveTo(0, 100);
        path->cubicTo(133.635864f, 0, -33.6358566f, 0, 100, 100);
    }
    else if (directPolySharkCase)
    {
        renderer.transform(rive::Mat2D(kPolySharkStreamScale,
                                       0,
                                       0,
                                       kPolySharkStreamScale,
                                       0,
                                       0));
        addFeatherPolyShapesShark(path.get());
    }
    else if (directGridCase)
    {
        addClockwiseNestedGrid(path.get());
    }
    else if (directFlowerCase)
    {
        addClockwiseNestedFlower(path.get());
    }
    else if (directBadSkinCase)
    {
        renderer.transform(rive::Mat2D(1.00501573f,
                                       0.116219193f,
                                       -0.11621917f,
                                       1.00501561f,
                                       550.433167f,
                                       361.510925f));
        addBadSkinHair(path.get());
    }
    else if (cuspCase)
    {
        path->moveTo(16, 48);
        path->cubicTo(51.2f, 16, 12.8f, 16, 48, 48);
        if (auxiliaryOutput != nullptr)
        {
            rive::RiveRenderPath source;
            source.fillRule(rive::FillRule::clockwise);
            source.moveTo(0, 100);
            source.moveTo(0, 100);
            source.cubicTo(110, 0, -10, 0, 100, 100);
            auto softened = source.makeSoftenedCopyForFeathering(1, 1.46300006f);
            writeSoftenedPath(auxiliaryOutput, softened->getRawPath());
        }
    }
    else if (circleCase)
    {
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
    else if (intersectionGroupsCase)
    {
        // This mode creates its three authored paths after the paint setup.
    }
    else
    {
        // Exact fixed_feather_atlas_mask() geometry from Rust's env test.
        path->moveTo(kSquareMin, kSquareMin);
        path->lineTo(kSquareMax, kSquareMin);
        path->lineTo(kSquareMax, kSquareMax);
        path->lineTo(kSquareMin, kSquareMax);
    }
    if (!directCase && !intersectionGroupsCase)
    {
        path->close();
    }
    auto paint = context->makeRenderPaint();
    paint->style(fillCase ? rive::RenderPaintStyle::fill
                          : rive::RenderPaintStyle::stroke);
    if (!fillCase)
    {
        paint->thickness(directStrokesRoundCase ? kDirectStrokesRoundThickness
                                                : kStrokeThickness);
        paint->join(rive::StrokeJoin::miter);
        paint->cap(rive::StrokeCap::butt);
    }
    paint->feather(directTriangulatedCase || directStrokesRoundCase ||
                           intersectionGroupsCase
                       ? 0.f
                       : (directCase ? 1.f : kFeather));
    paint->color(directStrokesRoundCase
                     ? 0x7a52bdb0
                     : (anyAdvancedBlendCase ? 0xc0e08040 : 0xffffffff));
    if (anyAdvancedBlendCase)
    {
        paint->blendMode(rive::BlendMode::colorDodge);
    }
    if (clippedCase)
    {
        auto clip = context->makeEmptyRenderPath();
        clip->moveTo(16, 8);
        clip->lineTo(32, 8);
        clip->lineTo(32, 56);
        clip->lineTo(16, 56);
        clip->close();
        renderer.clipPath(clip.get());
    }
    else if (pathClippedCase)
    {
        auto clip = context->makeEmptyRenderPath();
        clip->moveTo(32, 16);
        clip->lineTo(48, 48);
        clip->lineTo(16, 48);
        clip->close();
        renderer.clipPath(clip.get());
    }
    else if (changingPathClippedCase)
    {
        renderer.save();
        auto firstClip = context->makeEmptyRenderPath();
        firstClip->moveTo(16, 16);
        firstClip->lineTo(32, 48);
        firstClip->lineTo(8, 48);
        firstClip->close();
        renderer.clipPath(firstClip.get());
        renderer.drawPath(path.get(), paint.get());
        renderer.restore();

        auto secondClip = context->makeEmptyRenderPath();
        secondClip->moveTo(48, 16);
        secondClip->lineTo(56, 48);
        secondClip->lineTo(32, 48);
        secondClip->close();
        renderer.clipPath(secondClip.get());
    }
    else if (nestedPathClippedCase)
    {
        auto outerClip = context->makeEmptyRenderPath();
        outerClip->moveTo(32, 8);
        outerClip->lineTo(56, 56);
        outerClip->lineTo(8, 56);
        outerClip->close();
        renderer.clipPath(outerClip.get());

        auto innerClip = context->makeEmptyRenderPath();
        innerClip->moveTo(32, 20);
        innerClip->lineTo(44, 48);
        innerClip->lineTo(20, 48);
        innerClip->close();
        renderer.clipPath(innerClip.get());
    }
    else if (nestedEvenOddPathClippedCase)
    {
        auto outerClip = context->makeEmptyRenderPath();
        outerClip->fillRule(rive::FillRule::clockwise);
        outerClip->moveTo(8, 8);
        outerClip->lineTo(32, 8);
        outerClip->lineTo(32, 56);
        outerClip->lineTo(8, 56);
        outerClip->close();
        outerClip->moveTo(32, 8);
        outerClip->lineTo(32, 56);
        outerClip->lineTo(56, 56);
        outerClip->lineTo(56, 8);
        outerClip->close();
        renderer.clipPath(outerClip.get());

        auto innerClip = context->makeEmptyRenderPath();
        innerClip->fillRule(rive::FillRule::evenOdd);
        innerClip->moveTo(12, 12);
        innerClip->lineTo(52, 12);
        innerClip->lineTo(52, 52);
        innerClip->lineTo(12, 52);
        innerClip->close();
        innerClip->moveTo(20, 20);
        innerClip->lineTo(44, 20);
        innerClip->lineTo(44, 44);
        innerClip->lineTo(20, 44);
        innerClip->close();
        renderer.clipPath(innerClip.get());
    }
    else if (nestedClockwisePathClippedCase)
    {
        auto outerClip = context->makeEmptyRenderPath();
        outerClip->fillRule(rive::FillRule::evenOdd);
        outerClip->moveTo(8, 8);
        outerClip->lineTo(56, 8);
        outerClip->lineTo(56, 56);
        outerClip->lineTo(8, 56);
        outerClip->close();
        outerClip->moveTo(24, 24);
        outerClip->lineTo(40, 24);
        outerClip->lineTo(40, 40);
        outerClip->lineTo(24, 40);
        outerClip->close();
        renderer.clipPath(outerClip.get());

        auto innerClip = context->makeEmptyRenderPath();
        innerClip->fillRule(rive::FillRule::clockwise);
        innerClip->moveTo(8, 8);
        innerClip->lineTo(32, 8);
        innerClip->lineTo(32, 56);
        innerClip->lineTo(8, 56);
        innerClip->close();
        innerClip->moveTo(32, 8);
        innerClip->lineTo(32, 56);
        innerClip->lineTo(56, 56);
        innerClip->lineTo(56, 8);
        innerClip->close();
        renderer.clipPath(innerClip.get());
    }
    if (intersectionGroupsCase)
    {
        // Draw 0 is opaque MSAA fast-path coverage: prepassCount=3,
        // subpassCount=0. Draw 1 is its overlapping translucent counterpart.
        // Draw 2 is disjoint from both and translucent clockwise coverage,
        // which has prepassCount=0 and subpassCount=3. The distinct fill flags
        // keep their identities in AtlasMaskOracleFacts::drawBatches.
        auto draw0Path = context->makeEmptyRenderPath();
        draw0Path->fillRule(rive::FillRule::clockwise);
        addIntersectionGroupRect(draw0Path.get(), 8, 8, 28, 28);
        auto draw0Paint = context->makeRenderPaint();
        draw0Paint->style(rive::RenderPaintStyle::fill);
        draw0Paint->color(0xffffffff);
        renderer.drawPath(draw0Path.get(), draw0Paint.get());

        auto draw1Path = context->makeEmptyRenderPath();
        draw1Path->fillRule(rive::FillRule::nonZero);
        addIntersectionGroupRect(draw1Path.get(), 20, 20, 44, 44);
        auto draw1Paint = context->makeRenderPaint();
        draw1Paint->style(rive::RenderPaintStyle::fill);
        draw1Paint->color(0x80ffffff);
        renderer.drawPath(draw1Path.get(), draw1Paint.get());

        auto draw2Path = context->makeEmptyRenderPath();
        draw2Path->fillRule(rive::FillRule::clockwise);
        addIntersectionGroupRect(draw2Path.get(), 48, 8, 60, 20);
        auto draw2Paint = context->makeRenderPaint();
        draw2Paint->style(rive::RenderPaintStyle::fill);
        draw2Paint->color(0x80ffffff);
        renderer.drawPath(draw2Path.get(), draw2Paint.get());
    }
    else
    {
        renderer.drawPath(path.get(), paint.get());
    }

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
        std::printf("  drawType=%u shaderFeatures=0x%x shaderMiscFlags=0x%x drawContents=0x%x range=%u+%u\n",
                    batch.drawType,
                    batch.shaderFeatures,
                    batch.shaderMiscFlags,
                    batch.drawContents,
                    batch.baseElement,
                    batch.elementCount);
    }
    if (intersectionGroupsCase)
    {
        const uint32_t draw0Contents =
            static_cast<uint32_t>(rive::gpu::DrawContents::opaquePaint) |
            static_cast<uint32_t>(rive::gpu::DrawContents::clockwiseFill);
        const uint32_t draw1Contents =
            static_cast<uint32_t>(rive::gpu::DrawContents::nonZeroFill);
        const uint32_t draw2Contents =
            static_cast<uint32_t>(rive::gpu::DrawContents::clockwiseFill);
        const std::array<uint32_t, kIntersectionGroupBatchCount> expectedTypes = {
            static_cast<uint32_t>(
                rive::gpu::DrawType::msaaMidpointFanBorrowedCoverage),
            static_cast<uint32_t>(rive::gpu::DrawType::msaaMidpointFans),
            static_cast<uint32_t>(
                rive::gpu::DrawType::msaaMidpointFanStencilReset),
            static_cast<uint32_t>(
                rive::gpu::DrawType::msaaMidpointFanBorrowedCoverage),
            static_cast<uint32_t>(rive::gpu::DrawType::msaaMidpointFans),
            static_cast<uint32_t>(
                rive::gpu::DrawType::msaaMidpointFanStencilReset),
            static_cast<uint32_t>(
                rive::gpu::DrawType::msaaMidpointFanBorrowedCoverage),
            static_cast<uint32_t>(rive::gpu::DrawType::msaaMidpointFans),
            static_cast<uint32_t>(
                rive::gpu::DrawType::msaaMidpointFanStencilReset),
        };
        const std::array<uint32_t, kIntersectionGroupBatchCount>
            expectedContents = {
                draw0Contents,
                draw0Contents,
                draw0Contents,
                draw2Contents,
                draw2Contents,
                draw2Contents,
                draw1Contents,
                draw1Contents,
                draw1Contents,
            };
        const std::array<uint32_t, kIntersectionGroupBatchCount> expectedBases = {
            1, 1, 1, 2, 2, 2, 3, 3, 3,
        };
        const uint32_t expectedFeatures =
            static_cast<uint32_t>(rive::gpu::ShaderFeatures::ENABLE_DITHER);
        const uint32_t expectedMiscFlags = static_cast<uint32_t>(
            rive::gpu::ShaderMiscFlags::fixedFunctionColorOutput);
        bool scheduleMatches =
            facts.interlockMode ==
                static_cast<uint32_t>(rive::gpu::InterlockMode::msaa) &&
            facts.fixedFunctionColorOutput &&
            facts.drawBatches.size() == kIntersectionGroupBatchCount;
        for (size_t i = 0; scheduleMatches && i != facts.drawBatches.size(); ++i)
        {
            const auto& batch = facts.drawBatches[i];
            scheduleMatches = batch.drawType == expectedTypes[i] &&
                              batch.drawContents == expectedContents[i] &&
                              batch.baseElement == expectedBases[i] &&
                              batch.elementCount == 1 &&
                              batch.shaderFeatures == expectedFeatures &&
                              batch.shaderMiscFlags == expectedMiscFlags;
        }
        if (!scheduleMatches)
        {
            fail("msaa intersection-group oracle must schedule tagged draws 0 and 2 ahead of overlapping draw 1");
        }
        // Draws 1 and 2 are both positive-key three-pass fills. Draw 2's
        // group-3 type 10 sorts ahead of draw 1's lower type 8 only because
        // draw-group bits have higher priority and IntersectionBoard starts
        // draw 1 above all three layers reserved by draw 0.
        // This mode is schedule-only. It intentionally does not create an
        // atlas artifact because its paths use native MSAA coverage.
        return 0;
    }
    else if (advancedBlendCase)
    {
        const uint32_t expectedFeatures =
            static_cast<uint32_t>(
                rive::gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND) |
            static_cast<uint32_t>(rive::gpu::ShaderFeatures::ENABLE_DITHER);
        const uint32_t advancedBlend =
            static_cast<uint32_t>(rive::gpu::DrawContents::advancedBlend);
        if (facts.interlockMode !=
                static_cast<uint32_t>(rive::gpu::InterlockMode::msaa) ||
            facts.fixedFunctionColorOutput || facts.drawBatches.size() != 1 ||
            facts.drawBatches[0].drawType !=
                static_cast<uint32_t>(rive::gpu::DrawType::atlasBlit) ||
            facts.drawBatches[0].shaderFeatures != expectedFeatures ||
            facts.drawBatches[0].shaderMiscFlags != 0 ||
            facts.drawBatches[0].drawContents != advancedBlend ||
            facts.drawBatches[0].baseElement != 0 ||
            facts.drawBatches[0].elementCount != 6)
        {
            fail("advanced-blend oracle must execute one destination-reading MSAA atlas batch with shader color output");
        }
    }
    else if (atomicAdvancedBlendCase)
    {
        const uint32_t expectedFeatures =
            static_cast<uint32_t>(
                rive::gpu::ShaderFeatures::ENABLE_ADVANCED_BLEND) |
            static_cast<uint32_t>(rive::gpu::ShaderFeatures::ENABLE_FEATHER) |
            static_cast<uint32_t>(rive::gpu::ShaderFeatures::ENABLE_DITHER);
        const uint32_t expectedContents =
            static_cast<uint32_t>(rive::gpu::DrawContents::featheredFill) |
            static_cast<uint32_t>(rive::gpu::DrawContents::clockwiseFill) |
            static_cast<uint32_t>(rive::gpu::DrawContents::advancedBlend);
        if (facts.interlockMode !=
                static_cast<uint32_t>(rive::gpu::InterlockMode::atomics) ||
            facts.fixedFunctionColorOutput || facts.drawBatches.size() != 3 ||
            facts.drawBatches[0].drawType != static_cast<uint32_t>(
                                                  rive::gpu::DrawType::renderPassInitialize) ||
            facts.drawBatches[0].shaderFeatures != 0 ||
            facts.drawBatches[0].shaderMiscFlags != 0 ||
            facts.drawBatches[0].drawContents != 0 ||
            facts.drawBatches[0].baseElement != 0 ||
            facts.drawBatches[0].elementCount != 1 ||
            facts.drawBatches[1].drawType != static_cast<uint32_t>(
                                                  rive::gpu::DrawType::midpointFanCenterAAPatches) ||
            facts.drawBatches[1].shaderFeatures != expectedFeatures ||
            facts.drawBatches[1].shaderMiscFlags != 0 ||
            facts.drawBatches[1].drawContents != expectedContents ||
            facts.drawBatches[1].baseElement != 1 ||
            facts.drawBatches[1].elementCount != 22 ||
            facts.drawBatches[2].drawType != static_cast<uint32_t>(
                                                  rive::gpu::DrawType::renderPassResolve) ||
            facts.drawBatches[2].shaderFeatures != 0 ||
            facts.drawBatches[2].shaderMiscFlags != 0 ||
            facts.drawBatches[2].drawContents != 0 ||
            facts.drawBatches[2].baseElement != 0 ||
            facts.drawBatches[2].elementCount != 1)
        {
            fail("atomic advanced-blend oracle must execute initialize, destination-reading feather patches, and resolve");
        }
    }
    else if (directCase)
    {
        const uint32_t expectedPatchDrawType = static_cast<uint32_t>(
            directStrokesRoundCase
                ? rive::gpu::DrawType::midpointFanPatches
                : rive::gpu::DrawType::midpointFanCenterAAPatches);
        bool directScheduleValid =
            facts.drawBatches.size() == 3 &&
            facts.drawBatches[0].drawType == static_cast<uint32_t>(
                                                   rive::gpu::DrawType::renderPassInitialize) &&
            facts.drawBatches[1].drawType == expectedPatchDrawType &&
            facts.drawBatches[2].drawType == static_cast<uint32_t>(
                                                   rive::gpu::DrawType::renderPassResolve);
        if (directTriangulatedCase)
        {
            directScheduleValid = facts.drawBatches.size() == 4 &&
                                  facts.drawBatches.front().drawType == static_cast<uint32_t>(
                                      rive::gpu::DrawType::renderPassInitialize) &&
                                  facts.drawBatches[1].drawType == static_cast<uint32_t>(
                                      rive::gpu::DrawType::outerCurvePatches) &&
                                  facts.drawBatches[2].drawType == static_cast<uint32_t>(
                                      rive::gpu::DrawType::interiorTriangulation) &&
                                  facts.drawBatches.back().drawType == static_cast<uint32_t>(
                                      rive::gpu::DrawType::renderPassResolve);
        }
        const bool strokesRoundScheduleValid =
            !directStrokesRoundCase ||
            (directScheduleValid && facts.fixedFunctionColorOutput &&
             facts.drawBatches[1].shaderFeatures == static_cast<uint32_t>(
                 rive::gpu::ShaderFeatures::ENABLE_DITHER) &&
             facts.drawBatches[1].shaderMiscFlags == static_cast<uint32_t>(
                 rive::gpu::ShaderMiscFlags::fixedFunctionColorOutput) &&
             facts.drawBatches[1].drawContents ==
                 static_cast<uint32_t>(rive::gpu::DrawContents::stroke) &&
             facts.drawBatches[1].baseElement == 1 &&
             facts.drawBatches[1].elementCount == 10 &&
             facts.strokePatchCount == 10);
        if (facts.interlockMode !=
                static_cast<uint32_t>(rive::gpu::InterlockMode::atomics) ||
            !directScheduleValid || !strokesRoundScheduleValid ||
            (!directTriangulatedCase &&
             (facts.strokeBatchCount != 1 || facts.atlasBatchIsStroke ||
              facts.strokePatchCount == 0)) ||
            (directCuspCase && facts.contours.size() != 2) ||
            (directPolySharkCase &&
             (facts.strokePatchCount != kExpectedPolySharkPatchCount ||
              facts.contours.size() != kExpectedPolySharkContourCount)) ||
            (directStrokesRoundCase && facts.contours.size() != 1) ||
            (directGridCase &&
             (facts.contours.size() != kDirectGridContourCount ||
              facts.triangles.empty() || facts.triangles.size() % 3 != 0)) ||
            (directFlowerCase &&
             (facts.contours.size() != kDirectFlowerContourCount ||
              facts.triangles.empty() || facts.triangles.size() % 3 != 0)) ||
            (directBadSkinCase &&
             (facts.contours.size() != kDirectBadSkinContourCount ||
              facts.triangles.empty() || facts.triangles.size() % 3 != 0)))
        {
            fail("direct oracle must execute one atomic path-patch batch between initialize and resolve");
        }
        if (directStrokesRoundCase)
        {
            writeTessVertexSpans(auxiliaryOutput, facts);
        }
    }
    else if (nestedEvenOddPathClippedCase || nestedClockwisePathClippedCase)
    {
        const uint32_t clipUpdate =
            static_cast<uint32_t>(rive::gpu::DrawContents::clipUpdate);
        const uint32_t activeClip =
            static_cast<uint32_t>(rive::gpu::DrawContents::activeClip);
        const uint32_t evenOddFill =
            static_cast<uint32_t>(rive::gpu::DrawContents::evenOddFill);
        const uint32_t clockwiseFill =
            static_cast<uint32_t>(rive::gpu::DrawContents::clockwiseFill);
        const uint32_t dither =
            static_cast<uint32_t>(rive::gpu::ShaderFeatures::ENABLE_DITHER);
        const uint32_t fixedColor = static_cast<uint32_t>(
            rive::gpu::ShaderMiscFlags::fixedFunctionColorOutput);
        const std::vector<uint32_t> expectedTypes =
            nestedEvenOddPathClippedCase
                ? std::vector<uint32_t>{
                      static_cast<uint32_t>(rive::gpu::DrawType::msaaMidpointFanBorrowedCoverage),
                      static_cast<uint32_t>(rive::gpu::DrawType::msaaMidpointFans),
                      static_cast<uint32_t>(rive::gpu::DrawType::msaaMidpointFanStencilReset),
                      static_cast<uint32_t>(rive::gpu::DrawType::msaaMidpointFanPathsStencil),
                      static_cast<uint32_t>(rive::gpu::DrawType::clipReset),
                      static_cast<uint32_t>(rive::gpu::DrawType::atlasBlit),
                  }
                : std::vector<uint32_t>{
                      static_cast<uint32_t>(rive::gpu::DrawType::msaaMidpointFanPathsStencil),
                      static_cast<uint32_t>(rive::gpu::DrawType::msaaMidpointFanPathsCover),
                      static_cast<uint32_t>(rive::gpu::DrawType::msaaMidpointFanPathsStencil),
                      static_cast<uint32_t>(rive::gpu::DrawType::clipReset),
                      static_cast<uint32_t>(rive::gpu::DrawType::atlasBlit),
                  };
        const std::vector<uint32_t> expectedContents =
            nestedEvenOddPathClippedCase
                ? std::vector<uint32_t>{
                      clipUpdate | clockwiseFill,
                      clipUpdate | clockwiseFill,
                      clipUpdate | clockwiseFill,
                      clipUpdate | activeClip | evenOddFill,
                      clipUpdate | activeClip | evenOddFill,
                      activeClip,
                  }
                : std::vector<uint32_t>{
                      clipUpdate | evenOddFill,
                      clipUpdate | evenOddFill,
                      clipUpdate | activeClip | clockwiseFill,
                      clipUpdate | activeClip | clockwiseFill,
                      activeClip,
                  };
        const std::vector<uint32_t> expectedBases =
            nestedEvenOddPathClippedCase
                ? std::vector<uint32_t>{1, 1, 1, 3, 0, 6}
                : std::vector<uint32_t>{1, 1, 3, 0, 6};
        const std::vector<uint32_t> expectedCounts =
            nestedEvenOddPathClippedCase
                ? std::vector<uint32_t>{2, 2, 2, 2, 6, 6}
                : std::vector<uint32_t>{2, 2, 2, 6, 6};
        bool scheduleMatches =
            facts.interlockMode ==
                static_cast<uint32_t>(rive::gpu::InterlockMode::msaa) &&
            facts.fixedFunctionColorOutput &&
            facts.drawBatches.size() == expectedTypes.size();
        for (size_t i = 0; scheduleMatches && i != expectedTypes.size(); ++i)
        {
            const auto& batch = facts.drawBatches[i];
            scheduleMatches = batch.drawType == expectedTypes[i] &&
                              batch.drawContents == expectedContents[i] &&
                              batch.baseElement == expectedBases[i] &&
                              batch.elementCount == expectedCounts[i] &&
                              batch.shaderFeatures == dither &&
                              batch.shaderMiscFlags == fixedColor;
        }
        if (!scheduleMatches)
        {
            fail(nestedEvenOddPathClippedCase
                     ? "alternate clip oracle must execute clockwise outer and nested even-odd MSAA stencil transitions"
                     : "alternate clip oracle must execute even-odd outer and nested clockwise MSAA stencil transitions");
        }
    }
    else if (nestedPathClippedCase)
    {
        const uint32_t clipUpdate =
            static_cast<uint32_t>(rive::gpu::DrawContents::clipUpdate) |
            static_cast<uint32_t>(rive::gpu::DrawContents::nonZeroFill);
        const uint32_t nestedClipUpdate =
            clipUpdate |
            static_cast<uint32_t>(rive::gpu::DrawContents::activeClip);
        const uint32_t activeClip =
            static_cast<uint32_t>(rive::gpu::DrawContents::activeClip);
        if (facts.interlockMode !=
                static_cast<uint32_t>(rive::gpu::InterlockMode::msaa) ||
            !facts.fixedFunctionColorOutput || facts.drawBatches.size() != 6 ||
            facts.drawBatches[0].drawType != static_cast<uint32_t>(
                                                   rive::gpu::DrawType::msaaMidpointFanBorrowedCoverage) ||
            facts.drawBatches[1].drawType != static_cast<uint32_t>(
                                                   rive::gpu::DrawType::msaaMidpointFans) ||
            facts.drawBatches[2].drawType != static_cast<uint32_t>(
                                                   rive::gpu::DrawType::msaaMidpointFanStencilReset) ||
            facts.drawBatches[3].drawType != static_cast<uint32_t>(
                                                   rive::gpu::DrawType::msaaMidpointFanPathsStencil) ||
            facts.drawBatches[4].drawType !=
                static_cast<uint32_t>(rive::gpu::DrawType::clipReset) ||
            facts.drawBatches[5].drawType !=
                static_cast<uint32_t>(rive::gpu::DrawType::atlasBlit) ||
            facts.drawBatches[0].drawContents != clipUpdate ||
            facts.drawBatches[1].drawContents != clipUpdate ||
            facts.drawBatches[2].drawContents != clipUpdate ||
            facts.drawBatches[3].drawContents != nestedClipUpdate ||
            facts.drawBatches[4].drawContents != nestedClipUpdate ||
            facts.drawBatches[5].drawContents != activeClip ||
            facts.drawBatches[0].baseElement != 1 ||
            facts.drawBatches[1].baseElement != 1 ||
            facts.drawBatches[2].baseElement != 1 ||
            facts.drawBatches[0].elementCount != 1 ||
            facts.drawBatches[1].elementCount != 1 ||
            facts.drawBatches[2].elementCount != 1 ||
            facts.drawBatches[3].baseElement != 2 ||
            facts.drawBatches[3].elementCount != 1 ||
            facts.drawBatches[4].baseElement != 0 ||
            facts.drawBatches[4].elementCount != 6 ||
            facts.drawBatches[5].baseElement != 6 ||
            facts.drawBatches[5].elementCount != 6 ||
            facts.drawBatches[5].shaderFeatures != static_cast<uint32_t>(
                rive::gpu::ShaderFeatures::ENABLE_DITHER) ||
            facts.drawBatches[5].shaderMiscFlags != static_cast<uint32_t>(
                rive::gpu::ShaderMiscFlags::fixedFunctionColorOutput))
        {
            fail("nested path-clipped atlas oracle must stencil the inner non-zero path, intersect the parent clip, and draw one clipped atlas batch");
        }
    }
    else if (pathClippedCase || changingPathClippedCase)
    {
        const uint32_t clipUpdate =
            static_cast<uint32_t>(rive::gpu::DrawContents::clipUpdate) |
            static_cast<uint32_t>(rive::gpu::DrawContents::nonZeroFill);
        const uint32_t activeClip =
            static_cast<uint32_t>(rive::gpu::DrawContents::activeClip);
        const uint32_t clipReset =
            static_cast<uint32_t>(rive::gpu::DrawContents::clipUpdate);
        const size_t expectedBatchCount = changingPathClippedCase ? 9 : 4;
        if (facts.interlockMode !=
                static_cast<uint32_t>(rive::gpu::InterlockMode::msaa) ||
            !facts.fixedFunctionColorOutput ||
            facts.drawBatches.size() != expectedBatchCount ||
            facts.drawBatches[0].drawType != static_cast<uint32_t>(
                                                   rive::gpu::DrawType::msaaMidpointFanBorrowedCoverage) ||
            facts.drawBatches[1].drawType != static_cast<uint32_t>(
                                                   rive::gpu::DrawType::msaaMidpointFans) ||
            facts.drawBatches[2].drawType != static_cast<uint32_t>(
                                                   rive::gpu::DrawType::msaaMidpointFanStencilReset) ||
            facts.drawBatches[3].drawType !=
                static_cast<uint32_t>(rive::gpu::DrawType::atlasBlit) ||
            facts.drawBatches[0].drawContents != clipUpdate ||
            facts.drawBatches[1].drawContents != clipUpdate ||
            facts.drawBatches[2].drawContents != clipUpdate ||
            facts.drawBatches[3].drawContents != activeClip ||
            facts.drawBatches[0].baseElement != 1 ||
            facts.drawBatches[1].baseElement != 1 ||
            facts.drawBatches[2].baseElement != 1 ||
            facts.drawBatches[0].elementCount != 1 ||
            facts.drawBatches[1].elementCount != 1 ||
            facts.drawBatches[2].elementCount != 1 ||
            facts.drawBatches[3].baseElement != 0 ||
            facts.drawBatches[3].elementCount != 6 ||
            facts.drawBatches[3].shaderFeatures != static_cast<uint32_t>(
                rive::gpu::ShaderFeatures::ENABLE_DITHER) ||
            facts.drawBatches[3].shaderMiscFlags != static_cast<uint32_t>(
                rive::gpu::ShaderMiscFlags::fixedFunctionColorOutput) ||
            (changingPathClippedCase &&
             (facts.drawBatches[4].drawType !=
                  static_cast<uint32_t>(rive::gpu::DrawType::clipReset) ||
              facts.drawBatches[4].drawContents != clipReset ||
              facts.drawBatches[5].drawType != static_cast<uint32_t>(
                                                     rive::gpu::DrawType::msaaMidpointFanBorrowedCoverage) ||
              facts.drawBatches[6].drawType != static_cast<uint32_t>(
                                                     rive::gpu::DrawType::msaaMidpointFans) ||
              facts.drawBatches[7].drawType != static_cast<uint32_t>(
                                                     rive::gpu::DrawType::msaaMidpointFanStencilReset) ||
              facts.drawBatches[8].drawType !=
                  static_cast<uint32_t>(rive::gpu::DrawType::atlasBlit) ||
              facts.drawBatches[5].drawContents != clipUpdate ||
              facts.drawBatches[6].drawContents != clipUpdate ||
              facts.drawBatches[7].drawContents != clipUpdate ||
              facts.drawBatches[8].drawContents != activeClip ||
              facts.drawBatches[4].baseElement != 6 ||
              facts.drawBatches[4].elementCount != 6 ||
              facts.drawBatches[5].baseElement != 2 ||
              facts.drawBatches[6].baseElement != 2 ||
              facts.drawBatches[7].baseElement != 2 ||
              facts.drawBatches[5].elementCount != 1 ||
              facts.drawBatches[6].elementCount != 1 ||
              facts.drawBatches[7].elementCount != 1 ||
              facts.drawBatches[8].baseElement != 12 ||
              facts.drawBatches[8].elementCount != 6 ||
              facts.drawBatches[8].shaderFeatures != static_cast<uint32_t>(
                  rive::gpu::ShaderFeatures::ENABLE_DITHER) ||
              facts.drawBatches[8].shaderMiscFlags != static_cast<uint32_t>(
                  rive::gpu::ShaderMiscFlags::fixedFunctionColorOutput))))
        {
            fail(changingPathClippedCase
                     ? "changing path-clipped atlas oracle must clear the previous clip between two top-level MSAA clip updates"
                     : "path-clipped atlas oracle must execute the top-level MSAA clip update followed by one clipped atlas batch");
        }
    }
    else if (facts.interlockMode !=
            static_cast<uint32_t>(rive::gpu::InterlockMode::msaa) ||
        !facts.fixedFunctionColorOutput || facts.drawBatches.size() != 1 ||
        facts.drawBatches.front().drawType !=
            static_cast<uint32_t>(rive::gpu::DrawType::atlasBlit) ||
        facts.drawBatches.front().shaderFeatures !=
            (static_cast<uint32_t>(rive::gpu::ShaderFeatures::ENABLE_DITHER) |
             (clippedCase
                  ? static_cast<uint32_t>(
                        rive::gpu::ShaderFeatures::ENABLE_CLIP_RECT)
                  : 0u)) ||
        facts.drawBatches.front().shaderMiscFlags != static_cast<uint32_t>(
            rive::gpu::ShaderMiscFlags::fixedFunctionColorOutput))
    {
        fail("final atlas-blit oracle must execute one fixed-function MSAA atlas batch");
    }
    if (!directOutputCase && !changingPathClippedCase &&
        (facts.contentWidth != kExpectedLogicalAtlasSize ||
        facts.contentHeight != kExpectedLogicalAtlasSize ||
        !facts.pathTransformValid || facts.pathTranslateX != kAtlasPadding ||
        facts.pathTranslateY != kAtlasPadding || facts.strokeBatchCount != 1 ||
        facts.atlasBatchIsStroke == fillCase ||
        facts.strokeScissor.left != 0 || facts.strokeScissor.top != 0 ||
        facts.strokeScissor.right != kExpectedLogicalAtlasSize ||
        facts.strokeScissor.bottom != kExpectedLogicalAtlasSize))
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
    if (changingPathClippedCase &&
        (facts.contentWidth != kExpectedLogicalAtlasSize * 2 ||
         facts.contentHeight != kExpectedLogicalAtlasSize ||
         facts.strokeBatchCount != 2))
    {
        fail("changing path-clipped oracle must allocate two canonical atlas stroke regions");
    }
    uint32_t atlasWidth = 0;
    uint32_t atlasHeight = 0;
    if (!directOutputCase)
    {
        const wgpu::Texture atlas = webgpuContext->atlasMaskTextureForOracle();
        atlasWidth = atlas.GetWidth();
        atlasHeight = atlas.GetHeight();
        const uint32_t expectedAtlasWidth =
            changingPathClippedCase ? 97 : kExpectedPhysicalAtlasSize;
        if (atlasWidth != expectedAtlasWidth ||
            atlasHeight != kExpectedPhysicalAtlasSize)
        {
            std::fprintf(stderr,
                         "cpp-atlas-mask-oracle: expected physical=%ux48 frame=64x64, got physical=%ux%u\n",
                         expectedAtlasWidth,
                         atlasWidth,
                         atlasHeight);
            return 1;
        }
        if (!changingPathClippedCase)
        {
            const std::vector<uint8_t> atlasBytes =
                readTexture(instance, device, queue, atlas, kBytesPerTexel);
            writeMask(output,
                      atlasWidth,
                      atlasHeight,
                      atlasBytes.data(),
                      atlasWidth * kBytesPerTexel);
        }
    }

    const wgpu::Texture tessellation =
        webgpuContext->tessellationTextureForOracle();
    const uint32_t tessWidth = tessellation.GetWidth();
    const uint32_t tessHeight = tessellation.GetHeight();
    if (tessWidth != kExpectedTessWidth || tessHeight == 0 ||
        (directPolySharkCase && tessHeight != kExpectedPolySharkTessHeight) ||
        (changingPathClippedCase && tessHeight != 1))
    {
        std::fprintf(stderr,
                     "cpp-atlas-mask-oracle: unexpected tessellation texture dimensions %ux%u\n",
                     tessWidth,
                     tessHeight);
        return 1;
    }
    if (!changingPathClippedCase)
    {
        const std::vector<uint8_t> tessellationBytes =
            readTexture(instance, device, queue, tessellation, kTessBytesPerTexel);
        if (directGridCase)
        {
            writeDirectGridInputs(inputsOutput,
                                  facts,
                                  tessWidth,
                                  tessHeight,
                                  tessellationBytes.data(),
                                  tessWidth * kTessBytesPerTexel);
        }
        else if (directFlowerCase)
        {
            writeDirectFlowerInputs(inputsOutput,
                                    facts,
                                    tessWidth,
                                    tessHeight,
                                    tessellationBytes.data(),
                                    tessWidth * kTessBytesPerTexel);
        }
        else if (directBadSkinCase)
        {
            writeDirectBadSkinInputs(inputsOutput,
                                     facts,
                                     tessWidth,
                                     tessHeight,
                                     tessellationBytes.data(),
                                     tessWidth * kTessBytesPerTexel);
        }
        else
        {
            writeInputs(inputsOutput,
                        facts,
                        tessWidth,
                        tessHeight,
                        tessellationBytes.data(),
                        tessWidth * kTessBytesPerTexel);
        }
    }
    const std::vector<uint8_t> targetBytes =
        readTexture(instance, device, queue, targetTexture, 4);
    if (changingPathClippedCase)
    {
        const auto pixel = [&targetBytes](uint32_t x, uint32_t y) {
            return targetBytes.data() + (y * kFrameWidth + x) * 4;
        };
        const uint8_t* leftPixel = pixel(16, 32);
        const uint8_t* gapPixel = pixel(32, 32);
        const uint8_t* rightPixel = pixel(48, 32);
        for (uint32_t channel = 0; channel != 4; ++channel)
        {
            if (leftPixel[channel] != 79 || gapPixel[channel] != 0 ||
                rightPixel[channel] != 79)
            {
                fail("changing path-clipped oracle pixels must prove both clipped draws and the reset gap");
            }
        }
    }
    else if (nestedPathClippedCase)
    {
        const auto pixel = [&targetBytes](uint32_t x, uint32_t y) {
            return targetBytes.data() + (y * kFrameWidth + x) * 4;
        };
        const uint8_t* outsideInnerClip = pixel(22, 40);
        const uint8_t* insideInnerClip = pixel(24, 40);
        for (uint32_t channel = 0; channel != 4; ++channel)
        {
            if (outsideInnerClip[channel] != 0 ||
                insideInnerClip[channel] != 61)
            {
                fail("nested path-clipped oracle pixels must prove intersection with the inner non-zero clip");
            }
        }
    }
    else if (nestedEvenOddPathClippedCase)
    {
        const auto pixel = [&targetBytes](uint32_t x, uint32_t y) {
            return targetBytes.data() + (y * kFrameWidth + x) * 4;
        };
        const uint8_t* insideBothClips = pixel(18, 18);
        const uint8_t* insideEvenOddHole = pixel(24, 32);
        const uint8_t* counterclockwiseParentContour = pixel(46, 18);
        for (uint32_t channel = 0; channel != 4; ++channel)
        {
            if (insideBothClips[channel] == 0 ||
                insideEvenOddHole[channel] != 0 ||
                counterclockwiseParentContour[channel] != 0)
            {
                fail("nested even-odd path-clipped oracle pixels must prove the even-odd hole and clockwise parent winding");
            }
        }
    }
    else if (nestedClockwisePathClippedCase)
    {
        const auto pixel = [&targetBytes](uint32_t x, uint32_t y) {
            return targetBytes.data() + (y * kFrameWidth + x) * 4;
        };
        const uint8_t* clockwiseContour = pixel(20, 32);
        const uint8_t* insideEvenOddHole = pixel(28, 32);
        const uint8_t* counterclockwiseContour = pixel(44, 32);
        for (uint32_t channel = 0; channel != 4; ++channel)
        {
            if (clockwiseContour[channel] == 0 ||
                insideEvenOddHole[channel] != 0 ||
                counterclockwiseContour[channel] != 0)
            {
                fail("nested clockwise path-clipped oracle pixels must prove the even-odd parent hole and reject the oppositely wound contour");
            }
        }
    }
    writeBlit(blitOutput,
              frameWidth,
              frameHeight,
              targetBytes.data());
    if (!directOutputCase && !changingPathClippedCase)
    {
        std::printf("wrote %s: %ux%u R16Float row-packed atlas mask\\n",
                    output,
                    atlasWidth,
                    atlasHeight);
    }
    if (!changingPathClippedCase)
    {
        std::printf("wrote %s: batch=%u+%u contours=%zu interiorTriangles=%zu tessellation=%ux%u RGBA32Uint\\n",
                    inputsOutput,
                    facts.strokeBasePatch,
                    facts.strokePatchCount,
                    facts.contours.size(),
                    facts.triangles.size(),
                    tessWidth,
                    tessHeight);
    }
    std::printf("wrote %s: %ux%u tightly packed RGBA8 atlas blit\n",
                blitOutput,
                frameWidth,
                frameHeight);
    return 0;
}
