// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/utils/serializing_factory.cpp
#ifndef RIVE_RUST_GOLDEN_RECORDING_RENDERER_HPP
#define RIVE_RUST_GOLDEN_RECORDING_RENDERER_HPP

#include "rive/factory.hpp"
#include "rive/renderer.hpp"

#include <memory>
#include <sstream>
#include <string>
#include <vector>

namespace rive
{
struct SemanticsDiff;
}

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

class NullRenderer : public rive::Renderer
{
public:
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
};

// One typed custom property attached to a reported event, pre-extraction;
// formatting happens inside RecordingFactory so the stream float/string
// rules stay in one place (docs/side-channel-format.md).
struct SideChannelEventProperty
{
    enum class Kind
    {
        number,
        boolean,
        string,
        color,
        uintValue,
    };
    Kind kind = Kind::number;
    std::string name;
    float numberValue = 0.0f;
    bool boolValue = false;
    std::string stringValue;
    uint32_t colorValue = 0;
    uint64_t uintValue = 0;
};

struct SideChannelEvent
{
    uint32_t coreType = 0;
    std::string name;
    float delay = 0.0f;
    bool hasUrl = false;
    std::string url;
    std::string target;
    std::vector<SideChannelEventProperty> properties;
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
    void addSetInputBoolean(float seconds,
                            const std::string& name,
                            bool value);
    void addSetInputNumber(float seconds,
                           const std::string& name,
                           float value);
    void addSetInputTrigger(float seconds, const std::string& name);
    void addViewModelBoolean(float seconds,
                             const std::string& property,
                             bool value);
    void addViewModelNumber(float seconds,
                            const std::string& property,
                            float value);
    void addViewModelString(float seconds,
                            const std::string& property,
                            const std::string& value);
    void addViewModelEnum(float seconds,
                          const std::string& property,
                          uint32_t value);
    void addViewModelColor(float seconds,
                           const std::string& property,
                           uint32_t value);
    void addViewModelTrigger(float seconds, const std::string& property);
    void addResize(float seconds,
                   float width,
                   float height,
                   float dpr,
                   uint32_t pixelWidth,
                   uint32_t pixelHeight);
    void addAdvance(float seconds, bool settled);
    void addAdvanceWithStates(float seconds, bool settled, size_t statesChanged);
    void addSideChannelEvent(const SideChannelEvent& event);
    void addSemanticsDiff(const rive::SemanticsDiff& diff);
    void addSemanticAction(float seconds,
                           uint32_t nodeId,
                           const std::string& action,
                           bool dispatched);
    void addSemanticFocus(float seconds, uint32_t nodeId, bool focused);
    void addHitResult(const std::string& result);
    void addFrame();
    void frameSize(uint32_t width, uint32_t height);
    void clearColor(rive::ColorInt color);
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

class NullFactory : public rive::Factory
{
public:
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
};
} // namespace rive_rust::golden

#endif
