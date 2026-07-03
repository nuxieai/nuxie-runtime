// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/utils/serializing_factory.cpp
#include "recording_renderer.hpp"

#include "rive/math/raw_path.hpp"
#include "rive/refcnt.hpp"

#include <algorithm>
#include <cassert>
#include <cstring>
#include <iomanip>
#include <limits>
#include <sstream>
#include <utility>
#include <vector>

namespace rive_rust::golden
{
namespace
{
std::string floatToString(float value)
{
    std::ostringstream out;
    out.imbue(std::locale::classic());
    out << std::setprecision(std::numeric_limits<float>::max_digits10)
        << value;
    return out.str();
}

std::string matToString(const rive::Mat2D& mat)
{
    std::ostringstream out;
    out << '[';
    for (int index = 0; index < 6; index++)
    {
        if (index != 0)
        {
            out << ',';
        }
        out << floatToString(mat[index]);
    }
    out << ']';
    return out.str();
}

std::string colorToString(rive::ColorInt color)
{
    std::ostringstream out;
    out << "0x" << std::hex << std::setw(8) << std::setfill('0')
        << std::nouppercase << color;
    return out.str();
}

std::string samplerToString(rive::ImageSampler sampler)
{
    std::ostringstream out;
    out << "{wrapX=" << static_cast<int>(sampler.wrapX)
        << ",wrapY=" << static_cast<int>(sampler.wrapY)
        << ",filter=" << static_cast<int>(sampler.filter)
        << ",key=" << static_cast<int>(sampler.asKey()) << '}';
    return out.str();
}

uint64_t fnv1a(rive::Span<const uint8_t> bytes)
{
    uint64_t hash = 1469598103934665603ull;
    for (uint8_t byte : bytes)
    {
        hash ^= byte;
        hash *= 1099511628211ull;
    }
    return hash;
}

std::string hexBytes(const std::vector<uint8_t>& bytes)
{
    std::ostringstream out;
    out << std::hex << std::setfill('0') << std::nouppercase;
    for (uint8_t byte : bytes)
    {
        out << std::setw(2) << static_cast<int>(byte);
    }
    return out.str();
}

std::string verbToString(rive::PathVerb verb)
{
    switch (verb)
    {
        case rive::PathVerb::move:
            return "move";
        case rive::PathVerb::line:
            return "line";
        case rive::PathVerb::quad:
            return "quad";
        case rive::PathVerb::cubic:
            return "cubic";
        case rive::PathVerb::close:
            return "close";
    }
    return "unknown";
}

std::string rawPathToString(const rive::RawPath& path)
{
    std::ostringstream out;
    out << "{verbs=[";
    bool first = true;
    for (auto verb : path.verbs())
    {
        if (!first)
        {
            out << ',';
        }
        first = false;
        out << verbToString(verb);
    }
    out << "],points=[";
    first = true;
    for (auto point : path.points())
    {
        if (!first)
        {
            out << ',';
        }
        first = false;
        out << '(' << floatToString(point.x) << ','
            << floatToString(point.y) << ')';
    }
    out << "]}";
    return out.str();
}

class RecordingRenderShader : public rive::RenderShader
{
public:
    explicit RecordingRenderShader(uint64_t id) : m_id(id) {}
    uint64_t id() const { return m_id; }

private:
    uint64_t m_id;
};

class RecordingRenderImage : public rive::RenderImage
{
public:
    RecordingRenderImage(uint64_t id, int width, int height) : m_id(id)
    {
        m_Width = width;
        m_Height = height;
    }

    uint64_t id() const { return m_id; }

private:
    uint64_t m_id;
};

class RecordingRenderPaint : public rive::RenderPaint
{
public:
    explicit RecordingRenderPaint(uint64_t id) : m_id(id) {}

    void style(rive::RenderPaintStyle style) override { m_style = style; }
    void color(rive::ColorInt value) override { m_color = value; }
    void thickness(float value) override { m_thickness = value; }
    void join(rive::StrokeJoin value) override { m_join = value; }
    void cap(rive::StrokeCap value) override { m_cap = value; }
    void feather(float value) override { m_feather = value; }
    void blendMode(rive::BlendMode value) override { m_blendMode = value; }
    void shader(rive::rcp<rive::RenderShader> shader) override
    {
        m_shader = std::move(shader);
    }
    void invalidateStroke() override {}

    uint64_t id() const { return m_id; }

    std::string snapshot() const
    {
        uint64_t shaderId = 0;
        if (m_shader != nullptr)
        {
            shaderId =
                static_cast<RecordingRenderShader*>(m_shader.get())->id();
        }

        std::ostringstream out;
        out << "{id=" << m_id << ",style="
            << (m_style == rive::RenderPaintStyle::stroke ? "stroke" : "fill")
            << ",color=" << colorToString(m_color)
            << ",thickness=" << floatToString(m_thickness)
            << ",join=" << static_cast<int>(m_join)
            << ",cap=" << static_cast<int>(m_cap)
            << ",feather=" << floatToString(m_feather)
            << ",blendMode=" << static_cast<int>(m_blendMode)
            << ",shader=" << shaderId << '}';
        return out.str();
    }

private:
    uint64_t m_id;
    rive::RenderPaintStyle m_style = rive::RenderPaintStyle::fill;
    rive::ColorInt m_color = 0xff000000;
    float m_thickness = 1.0f;
    rive::StrokeJoin m_join = rive::StrokeJoin::miter;
    rive::StrokeCap m_cap = rive::StrokeCap::butt;
    float m_feather = 0.0f;
    rive::BlendMode m_blendMode = rive::BlendMode::srcOver;
    rive::rcp<rive::RenderShader> m_shader;
};

class RecordingRenderPath : public rive::RenderPath
{
public:
    explicit RecordingRenderPath(uint64_t id) : m_id(id) {}
    RecordingRenderPath(uint64_t id,
                        const rive::RawPath& rawPath,
                        rive::FillRule fillRule) :
        m_id(id), m_rawPath(rawPath), m_fillRule(fillRule)
    {}

    void rewind() override { m_rawPath.rewind(); }
    void fillRule(rive::FillRule value) override { m_fillRule = value; }
    void addPath(rive::CommandPath* path,
                 const rive::Mat2D& transform) override
    {
        addRenderPath(path->renderPath(), transform);
    }
    void addRenderPath(const rive::RenderPath* path,
                       const rive::Mat2D& transform) override
    {
        auto recordingPath = static_cast<const RecordingRenderPath*>(path);
        m_rawPath.addPath(recordingPath->rawPath(), &transform);
    }
    void addRenderPathBackwards(const rive::RenderPath* path,
                                const rive::Mat2D& transform) override
    {
        auto recordingPath = static_cast<const RecordingRenderPath*>(path);
        m_rawPath.addPathBackwards(recordingPath->rawPath(), &transform);
    }

    void moveTo(float x, float y) override { m_rawPath.moveTo(x, y); }
    void lineTo(float x, float y) override { m_rawPath.lineTo(x, y); }
    void cubicTo(float ox,
                 float oy,
                 float ix,
                 float iy,
                 float x,
                 float y) override
    {
        m_rawPath.cubicTo(ox, oy, ix, iy, x, y);
    }
    void close() override { m_rawPath.close(); }
    void addRawPath(const rive::RawPath& path) override
    {
        m_rawPath.addPath(path);
    }

    uint64_t id() const { return m_id; }
    rive::FillRule fillRule() const { return m_fillRule; }
    const rive::RawPath& rawPath() const { return m_rawPath; }

    std::string snapshot() const
    {
        std::ostringstream out;
        out << "{id=" << m_id
            << ",fillRule=" << static_cast<int>(m_fillRule)
            << ",path=" << rawPathToString(m_rawPath) << '}';
        return out.str();
    }

private:
    uint64_t m_id;
    rive::RawPath m_rawPath;
    rive::FillRule m_fillRule = rive::FillRule::nonZero;
};

class RecordingRenderBuffer : public rive::RenderBuffer
{
public:
    RecordingRenderBuffer(RecordingStream* stream,
                          uint64_t id,
                          rive::RenderBufferType type,
                          rive::RenderBufferFlags flags,
                          size_t sizeInBytes) :
        rive::RenderBuffer(type, flags, sizeInBytes),
        m_stream(stream),
        m_id(id),
        m_bytes(sizeInBytes)
    {}

    uint64_t id() const { return m_id; }
    const std::vector<uint8_t>& bytes() const { return m_bytes; }

private:
    void* onMap() override { return m_bytes.data(); }
    void onUnmap() override
    {
        std::ostringstream out;
        out << "bufferData id=" << m_id
            << " type=" << static_cast<int>(type())
            << " size=" << m_bytes.size()
            << " data=" << hexBytes(m_bytes);
        m_stream->line(out.str());
    }

    RecordingStream* m_stream;
    uint64_t m_id;
    std::vector<uint8_t> m_bytes;
};

uint64_t imageId(const rive::RenderImage* image)
{
    if (image == nullptr)
    {
        return 0;
    }
    return static_cast<const RecordingRenderImage*>(image)->id();
}

uint64_t bufferId(const rive::rcp<rive::RenderBuffer>& buffer)
{
    if (buffer == nullptr)
    {
        return 0;
    }
    return static_cast<RecordingRenderBuffer*>(buffer.get())->id();
}
} // namespace

void RecordingStream::line(const std::string& value)
{
    m_stream << value << '\n';
}

std::string RecordingStream::str() const { return m_stream.str(); }

void RecordingStream::clear()
{
    m_stream.str("");
    m_stream.clear();
}

RecordingRenderer::RecordingRenderer(RecordingStream* stream) :
    m_stream(stream)
{}

void RecordingRenderer::save() { m_stream->line("save"); }

void RecordingRenderer::restore() { m_stream->line("restore"); }

void RecordingRenderer::transform(const rive::Mat2D& transform)
{
    m_stream->line("transform matrix=" + matToString(transform));
}

void RecordingRenderer::drawPath(rive::RenderPath* path,
                                 rive::RenderPaint* paint)
{
    auto recordingPath = static_cast<RecordingRenderPath*>(path);
    auto recordingPaint = static_cast<RecordingRenderPaint*>(paint);
    std::ostringstream out;
    out << "drawPath path=" << recordingPath->snapshot()
        << " paint=" << recordingPaint->snapshot();
    m_stream->line(out.str());
}

void RecordingRenderer::clipPath(rive::RenderPath* path)
{
    auto recordingPath = static_cast<RecordingRenderPath*>(path);
    m_stream->line("clipPath path=" + recordingPath->snapshot());
}

void RecordingRenderer::drawImage(const rive::RenderImage* image,
                                  rive::ImageSampler sampler,
                                  rive::BlendMode blendMode,
                                  float opacity)
{
    std::ostringstream out;
    out << "drawImage image=" << imageId(image)
        << " sampler=" << samplerToString(sampler)
        << " blendMode=" << static_cast<int>(blendMode)
        << " opacity=" << floatToString(opacity);
    m_stream->line(out.str());
}

void RecordingRenderer::drawImageMesh(const rive::RenderImage* image,
                                      rive::ImageSampler sampler,
                                      rive::rcp<rive::RenderBuffer> vertices,
                                      rive::rcp<rive::RenderBuffer> uvCoords,
                                      rive::rcp<rive::RenderBuffer> indices,
                                      uint32_t vertexCount,
                                      uint32_t indexCount,
                                      rive::BlendMode blendMode,
                                      float opacity)
{
    std::ostringstream out;
    out << "drawImageMesh image=" << imageId(image)
        << " sampler=" << samplerToString(sampler)
        << " vertices=" << bufferId(vertices)
        << " uvs=" << bufferId(uvCoords)
        << " indices=" << bufferId(indices)
        << " vertexCount=" << vertexCount
        << " indexCount=" << indexCount
        << " blendMode=" << static_cast<int>(blendMode)
        << " opacity=" << floatToString(opacity);
    m_stream->line(out.str());
}

void RecordingRenderer::modulateOpacity(float opacity)
{
    m_stream->line("modulateOpacity opacity=" + floatToString(opacity));
}

RecordingFactory::RecordingFactory()
{
    m_stream.line("rive-golden-stream-v1");
}

rive::rcp<rive::RenderBuffer> RecordingFactory::makeRenderBuffer(
    rive::RenderBufferType type,
    rive::RenderBufferFlags flags,
    size_t sizeInBytes)
{
    auto id = m_nextBufferId++;
    std::ostringstream out;
    out << "makeRenderBuffer id=" << id
        << " type=" << static_cast<int>(type)
        << " flags=" << static_cast<int>(flags)
        << " size=" << sizeInBytes;
    m_stream.line(out.str());
    return rive::make_rcp<RecordingRenderBuffer>(
        &m_stream, id, type, flags, sizeInBytes);
}

rive::rcp<rive::RenderShader> RecordingFactory::makeLinearGradient(
    float sx,
    float sy,
    float ex,
    float ey,
    const rive::ColorInt colors[],
    const float stops[],
    size_t count)
{
    auto id = m_nextShaderId++;
    std::ostringstream out;
    out << "makeLinearGradient id=" << id << " start=("
        << floatToString(sx) << ',' << floatToString(sy) << ") end=("
        << floatToString(ex) << ',' << floatToString(ey) << ") stops=[";
    for (size_t index = 0; index < count; index++)
    {
        if (index != 0)
        {
            out << ',';
        }
        out << "{color=" << colorToString(colors[index])
            << ",stop=" << floatToString(stops[index]) << '}';
    }
    out << ']';
    m_stream.line(out.str());
    return rive::make_rcp<RecordingRenderShader>(id);
}

rive::rcp<rive::RenderShader> RecordingFactory::makeRadialGradient(
    float cx,
    float cy,
    float radius,
    const rive::ColorInt colors[],
    const float stops[],
    size_t count)
{
    auto id = m_nextShaderId++;
    std::ostringstream out;
    out << "makeRadialGradient id=" << id << " center=("
        << floatToString(cx) << ',' << floatToString(cy)
        << ") radius=" << floatToString(radius) << " stops=[";
    for (size_t index = 0; index < count; index++)
    {
        if (index != 0)
        {
            out << ',';
        }
        out << "{color=" << colorToString(colors[index])
            << ",stop=" << floatToString(stops[index]) << '}';
    }
    out << ']';
    m_stream.line(out.str());
    return rive::make_rcp<RecordingRenderShader>(id);
}

rive::rcp<rive::RenderPath> RecordingFactory::makeRenderPath(
    rive::RawPath& rawPath,
    rive::FillRule fillRule)
{
    auto id = m_nextPathId++;
    auto path = rive::make_rcp<RecordingRenderPath>(id, rawPath, fillRule);
    m_stream.line("makeRenderPath " + path->snapshot());
    return path;
}

rive::rcp<rive::RenderPath> RecordingFactory::makeEmptyRenderPath()
{
    auto id = m_nextPathId++;
    auto path = rive::make_rcp<RecordingRenderPath>(id);
    m_stream.line("makeEmptyRenderPath " + path->snapshot());
    return path;
}

rive::rcp<rive::RenderPaint> RecordingFactory::makeRenderPaint()
{
    auto id = m_nextPaintId++;
    auto paint = rive::make_rcp<RecordingRenderPaint>(id);
    m_stream.line("makeRenderPaint " + paint->snapshot());
    return paint;
}

rive::rcp<rive::RenderImage> RecordingFactory::decodeImage(
    rive::Span<const uint8_t> data)
{
    auto id = m_nextImageId++;
    std::ostringstream out;
    out << "decodeImage id=" << id << " size=" << data.size()
        << " hash=" << fnv1a(data);
    m_stream.line(out.str());
    // TODO(golden): link rive_decoders into the full harness and record real
    // decoded image dimensions.
    return rive::make_rcp<RecordingRenderImage>(id, 0, 0);
}

std::unique_ptr<rive::Renderer> RecordingFactory::makeRenderer()
{
    return std::make_unique<RecordingRenderer>(&m_stream);
}

void RecordingFactory::addFrame() { m_stream.line("frame"); }

void RecordingFactory::frameSize(uint32_t width, uint32_t height)
{
    std::ostringstream out;
    out << "frameSize width=" << width << " height=" << height;
    m_stream.line(out.str());
}

std::string RecordingFactory::stream() const { return m_stream.str(); }

void RecordingFactory::clear()
{
    m_stream.clear();
    m_stream.line("rive-golden-stream-v1");
}
} // namespace rive_rust::golden
