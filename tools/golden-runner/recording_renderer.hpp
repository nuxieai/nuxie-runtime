// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/utils/serializing_factory.cpp
#ifndef RIVE_RUST_GOLDEN_RECORDING_RENDERER_HPP
#define RIVE_RUST_GOLDEN_RECORDING_RENDERER_HPP

#include "rive/factory.hpp"
#include "rive/renderer.hpp"

#include <memory>
#include <sstream>
#include <string>

namespace rive_rust::golden
{
class RecordingStream
{
public:
    void line(const std::string& value);
    std::string str() const;
    void clear();

private:
    std::ostringstream m_stream;
};

class RecordingRenderer : public rive::Renderer
{
public:
    explicit RecordingRenderer(RecordingStream* stream);

    void save() override;
    void restore() override;
    void transform(const rive::Mat2D& transform) override;
    void drawPath(rive::RenderPath* path, rive::RenderPaint* paint) override;
    void clipPath(rive::RenderPath* path) override;
    void drawImage(const rive::RenderImage* image,
                   rive::ImageSampler sampler,
                   rive::BlendMode blendMode,
                   float opacity) override;
    void drawImageMesh(const rive::RenderImage* image,
                       rive::ImageSampler sampler,
                       rive::rcp<rive::RenderBuffer> vertices,
                       rive::rcp<rive::RenderBuffer> uvCoords,
                       rive::rcp<rive::RenderBuffer> indices,
                       uint32_t vertexCount,
                       uint32_t indexCount,
                       rive::BlendMode blendMode,
                       float opacity) override;
    void modulateOpacity(float opacity) override;

private:
    RecordingStream* m_stream;
};

class RecordingFactory : public rive::Factory
{
public:
    RecordingFactory();

    rive::rcp<rive::RenderBuffer> makeRenderBuffer(rive::RenderBufferType,
                                                   rive::RenderBufferFlags,
                                                   size_t sizeInBytes) override;
    rive::rcp<rive::RenderShader> makeLinearGradient(
        float sx,
        float sy,
        float ex,
        float ey,
        const rive::ColorInt colors[],
        const float stops[],
        size_t count) override;
    rive::rcp<rive::RenderShader> makeRadialGradient(
        float cx,
        float cy,
        float radius,
        const rive::ColorInt colors[],
        const float stops[],
        size_t count) override;
    rive::rcp<rive::RenderPath> makeRenderPath(rive::RawPath&,
                                               rive::FillRule) override;
    rive::rcp<rive::RenderPath> makeEmptyRenderPath() override;
    rive::rcp<rive::RenderPaint> makeRenderPaint() override;
    rive::rcp<rive::RenderImage> decodeImage(
        rive::Span<const uint8_t>) override;

    std::unique_ptr<rive::Renderer> makeRenderer();
    void source(const std::string& file,
                const std::string& artboard,
                const std::string& scene);
    void addSample(float seconds);
    void addInputEvent(const std::string& kind,
                       float seconds,
                       float x,
                       float y,
                       int pointerId);
    void addFrame();
    void frameSize(uint32_t width, uint32_t height);
    std::string stream() const;
    void clear();

private:
    RecordingStream m_stream;
    uint64_t m_nextImageId = 1;
    uint64_t m_nextPaintId = 1;
    uint64_t m_nextPathId = 1;
    uint64_t m_nextBufferId = 1;
    uint64_t m_nextShaderId = 1;
};
} // namespace rive_rust::golden

#endif
