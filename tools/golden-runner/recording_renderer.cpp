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

std::string quotedString(const std::string& value)
{
    std::ostringstream out;
    out << '"';
    for (char ch : value)
    {
        switch (ch)
        {
            case '\\':
                out << "\\\\";
                break;
            case '"':
                out << "\\\"";
                break;
            case '\n':
                out << "\\n";
                break;
            case '\r':
                out << "\\r";
                break;
            case '\t':
                out << "\\t";
                break;
            default:
                out << ch;
                break;
        }
    }
    out << '"';
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

uint32_t readBigEndian32(rive::Span<const uint8_t> bytes, size_t offset)
{
    return (static_cast<uint32_t>(bytes[offset]) << 24) |
           (static_cast<uint32_t>(bytes[offset + 1]) << 16) |
           (static_cast<uint32_t>(bytes[offset + 2]) << 8) |
           static_cast<uint32_t>(bytes[offset + 3]);
}

uint16_t readBigEndian16(rive::Span<const uint8_t> bytes, size_t offset)
{
    return static_cast<uint16_t>((static_cast<uint16_t>(bytes[offset]) << 8) |
                                 static_cast<uint16_t>(bytes[offset + 1]));
}

uint32_t readLittleEndian24(rive::Span<const uint8_t> bytes, size_t offset)
{
    return static_cast<uint32_t>(bytes[offset]) |
           (static_cast<uint32_t>(bytes[offset + 1]) << 8) |
           (static_cast<uint32_t>(bytes[offset + 2]) << 16);
}

uint16_t readLittleEndian16(rive::Span<const uint8_t> bytes, size_t offset)
{
    return static_cast<uint16_t>(static_cast<uint16_t>(bytes[offset]) |
                                 (static_cast<uint16_t>(bytes[offset + 1])
                                  << 8));
}

bool bytesEqual(rive::Span<const uint8_t> bytes,
                size_t offset,
                const char* literal,
                size_t length)
{
    return bytes.size() >= offset + length &&
           std::memcmp(bytes.data() + offset, literal, length) == 0;
}

std::pair<uint32_t, uint32_t> pngDimensions(rive::Span<const uint8_t> bytes)
{
    static constexpr uint8_t pngSignature[] = {
        0x89, 'P', 'N', 'G', '\r', '\n', 0x1a, '\n'};
    if (bytes.size() < 24 ||
        std::memcmp(bytes.data(), pngSignature, sizeof(pngSignature)) != 0 ||
        !bytesEqual(bytes, 12, "IHDR", 4))
    {
        return {0, 0};
    }
    return {readBigEndian32(bytes, 16), readBigEndian32(bytes, 20)};
}

bool jpegStartOfFrame(uint8_t marker)
{
    return (marker >= 0xc0 && marker <= 0xc3) ||
           (marker >= 0xc5 && marker <= 0xc7) ||
           (marker >= 0xc9 && marker <= 0xcb) ||
           (marker >= 0xcd && marker <= 0xcf);
}

std::pair<uint32_t, uint32_t> jpegDimensions(rive::Span<const uint8_t> bytes)
{
    if (bytes.size() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8)
    {
        return {0, 0};
    }

    size_t offset = 2;
    while (offset + 4 <= bytes.size())
    {
        while (offset < bytes.size() && bytes[offset] != 0xff)
        {
            offset++;
        }
        while (offset < bytes.size() && bytes[offset] == 0xff)
        {
            offset++;
        }
        if (offset >= bytes.size())
        {
            break;
        }

        uint8_t marker = bytes[offset++];
        if (marker == 0xd9 || marker == 0xda)
        {
            break;
        }
        if (marker >= 0xd0 && marker <= 0xd7)
        {
            continue;
        }
        if (offset + 2 > bytes.size())
        {
            break;
        }

        uint16_t segmentLength = readBigEndian16(bytes, offset);
        if (segmentLength < 2 || offset + segmentLength > bytes.size())
        {
            break;
        }
        if (jpegStartOfFrame(marker) && segmentLength >= 7)
        {
            uint32_t height = readBigEndian16(bytes, offset + 3);
            uint32_t width = readBigEndian16(bytes, offset + 5);
            return {width, height};
        }
        offset += segmentLength;
    }

    return {0, 0};
}

std::pair<uint32_t, uint32_t> webpDimensions(rive::Span<const uint8_t> bytes)
{
    if (bytes.size() < 20 || !bytesEqual(bytes, 0, "RIFF", 4) ||
        !bytesEqual(bytes, 8, "WEBP", 4))
    {
        return {0, 0};
    }

    size_t offset = 12;
    while (offset + 8 <= bytes.size())
    {
        const size_t chunkData = offset + 8;
        uint32_t chunkSize = static_cast<uint32_t>(bytes[offset + 4]) |
                             (static_cast<uint32_t>(bytes[offset + 5]) << 8) |
                             (static_cast<uint32_t>(bytes[offset + 6]) << 16) |
                             (static_cast<uint32_t>(bytes[offset + 7]) << 24);
        if (chunkData + chunkSize > bytes.size())
        {
            break;
        }

        if (bytesEqual(bytes, offset, "VP8X", 4) && chunkSize >= 10)
        {
            return {readLittleEndian24(bytes, chunkData + 4) + 1,
                    readLittleEndian24(bytes, chunkData + 7) + 1};
        }
        if (bytesEqual(bytes, offset, "VP8L", 4) && chunkSize >= 5 &&
            bytes[chunkData] == 0x2f)
        {
            uint32_t width = 1 + static_cast<uint32_t>(
                                     bytes[chunkData + 1] |
                                     ((bytes[chunkData + 2] & 0x3f) << 8));
            uint32_t height =
                1 + static_cast<uint32_t>((bytes[chunkData + 2] >> 6) |
                                          (bytes[chunkData + 3] << 2) |
                                          ((bytes[chunkData + 4] & 0x0f)
                                           << 10));
            return {width, height};
        }
        if (bytesEqual(bytes, offset, "VP8 ", 4) && chunkSize >= 10 &&
            bytesEqual(bytes, chunkData + 3, "\x9d\x01\x2a", 3))
        {
            return {readLittleEndian16(bytes, chunkData + 6) & 0x3fff,
                    readLittleEndian16(bytes, chunkData + 8) & 0x3fff};
        }

        offset = chunkData + chunkSize + (chunkSize & 1);
    }

    return {0, 0};
}

std::pair<uint32_t, uint32_t> encodedImageDimensions(
    rive::Span<const uint8_t> bytes)
{
    auto dimensions = pngDimensions(bytes);
    if (dimensions.first != 0 || dimensions.second != 0)
    {
        return dimensions;
    }
    dimensions = jpegDimensions(bytes);
    if (dimensions.first != 0 || dimensions.second != 0)
    {
        return dimensions;
    }
    return webpDimensions(bytes);
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

class NullRenderShader : public rive::RenderShader
{};

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

class NullRenderImage : public rive::RenderImage
{
public:
    NullRenderImage(int width, int height)
    {
        m_Width = width;
        m_Height = height;
    }
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

class NullRenderPaint : public rive::RenderPaint
{
public:
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

private:
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

class NullRenderPath : public rive::RenderPath
{
public:
    NullRenderPath() = default;
    NullRenderPath(const rive::RawPath& rawPath, rive::FillRule fillRule) :
        m_rawPath(rawPath), m_fillRule(fillRule)
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
        auto nullPath = static_cast<const NullRenderPath*>(path);
        m_rawPath.addPath(nullPath->rawPath(), &transform);
    }
    void addRenderPathBackwards(const rive::RenderPath* path,
                                const rive::Mat2D& transform) override
    {
        auto nullPath = static_cast<const NullRenderPath*>(path);
        m_rawPath.addPathBackwards(nullPath->rawPath(), &transform);
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

    const rive::RawPath& rawPath() const { return m_rawPath; }

private:
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

class NullRenderBuffer : public rive::RenderBuffer
{
public:
    NullRenderBuffer(rive::RenderBufferType type,
                     rive::RenderBufferFlags flags,
                     size_t sizeInBytes) :
        rive::RenderBuffer(type, flags, sizeInBytes),
        m_bytes(sizeInBytes)
    {}

private:
    void* onMap() override { return m_bytes.data(); }
    void onUnmap() override {}

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

void NullRenderer::save() {}

void NullRenderer::restore() {}

void NullRenderer::transform(const rive::Mat2D&) {}

void NullRenderer::drawPath(rive::RenderPath*, rive::RenderPaint*) {}

void NullRenderer::clipPath(rive::RenderPath*) {}

void NullRenderer::drawImage(const rive::RenderImage*,
                             rive::ImageSampler,
                             rive::BlendMode,
                             float)
{}

void NullRenderer::drawImageMesh(const rive::RenderImage*,
                                 rive::ImageSampler,
                                 rive::rcp<rive::RenderBuffer>,
                                 rive::rcp<rive::RenderBuffer>,
                                 rive::rcp<rive::RenderBuffer>,
                                 uint32_t,
                                 uint32_t,
                                 rive::BlendMode,
                                 float)
{}

void NullRenderer::modulateOpacity(float) {}

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
    auto dimensions = encodedImageDimensions(data);
    std::ostringstream out;
    out << "decodeImage id=" << id << " width=" << dimensions.first
        << " height=" << dimensions.second << " data="
        << hexBytes(std::vector<uint8_t>(data.begin(), data.end()));
    m_stream.line(out.str());
    return rive::make_rcp<RecordingRenderImage>(id,
                                                dimensions.first,
                                                dimensions.second);
}

std::unique_ptr<rive::Renderer> RecordingFactory::makeRenderer()
{
    return std::make_unique<RecordingRenderer>(&m_stream);
}

void RecordingFactory::source(const std::string& file,
                              const std::string& artboard,
                              const std::string& scene)
{
    std::ostringstream out;
    out << "source file=" << quotedString(file)
        << " artboard=" << quotedString(artboard)
        << " scene=" << quotedString(scene);
    m_stream.line(out.str());
}

void RecordingFactory::addSample(float seconds)
{
    m_stream.line("sample seconds=" + floatToString(seconds));
}

void RecordingFactory::addInputEvent(const std::string& kind,
                                     float seconds,
                                     float x,
                                     float y,
                                     int pointerId)
{
    std::ostringstream out;
    out << "input kind=" << kind << " seconds=" << floatToString(seconds)
        << " position=(" << floatToString(x) << ',' << floatToString(y)
        << ") pointerId=" << pointerId;
    m_stream.line(out.str());
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

rive::rcp<rive::RenderBuffer> NullFactory::makeRenderBuffer(
    rive::RenderBufferType type,
    rive::RenderBufferFlags flags,
    size_t sizeInBytes)
{
    return rive::make_rcp<NullRenderBuffer>(type, flags, sizeInBytes);
}

rive::rcp<rive::RenderShader> NullFactory::makeLinearGradient(
    float,
    float,
    float,
    float,
    const rive::ColorInt[],
    const float[],
    size_t)
{
    return rive::make_rcp<NullRenderShader>();
}

rive::rcp<rive::RenderShader> NullFactory::makeRadialGradient(
    float,
    float,
    float,
    const rive::ColorInt[],
    const float[],
    size_t)
{
    return rive::make_rcp<NullRenderShader>();
}

rive::rcp<rive::RenderPath> NullFactory::makeRenderPath(
    rive::RawPath& rawPath,
    rive::FillRule fillRule)
{
    return rive::make_rcp<NullRenderPath>(rawPath, fillRule);
}

rive::rcp<rive::RenderPath> NullFactory::makeEmptyRenderPath()
{
    return rive::make_rcp<NullRenderPath>();
}

rive::rcp<rive::RenderPaint> NullFactory::makeRenderPaint()
{
    return rive::make_rcp<NullRenderPaint>();
}

rive::rcp<rive::RenderImage> NullFactory::decodeImage(
    rive::Span<const uint8_t> data)
{
    auto dimensions = encodedImageDimensions(data);
    return rive::make_rcp<NullRenderImage>(dimensions.first,
                                           dimensions.second);
}

std::unique_ptr<rive::Renderer> NullFactory::makeRenderer()
{
    return std::make_unique<NullRenderer>();
}
} // namespace rive_rust::golden
