#include "rive/text/font_hb.hpp"
#include "rive/text_engine.hpp"

#include <hb.h>

#include <cstdint>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <iterator>
#include <string>
#include <vector>

static constexpr uint32_t kWeightTag =
    (uint32_t('w') << 24) | (uint32_t('g') << 16) | (uint32_t('h') << 8) |
    uint32_t('t');

static void emitPoint(const rive::Vec2D& point)
{
    std::cout << ',' << point.x << ',' << point.y;
}

static void emitPath(const rive::RawPath& path)
{
    std::cout << '[';
    bool first = true;
    for (auto [verb, points] : path)
    {
        if (!first)
        {
            std::cout << ',';
        }
        first = false;
        switch (verb)
        {
            case rive::PathVerb::move:
                std::cout << "[\"M\"";
                emitPoint(points[0]);
                std::cout << ']';
                break;
            case rive::PathVerb::line:
                std::cout << "[\"L\"";
                emitPoint(points[1]);
                std::cout << ']';
                break;
            case rive::PathVerb::quad:
                std::cout << "[\"Q\"";
                emitPoint(points[1]);
                emitPoint(points[2]);
                std::cout << ']';
                break;
            case rive::PathVerb::cubic:
                std::cout << "[\"C\"";
                emitPoint(points[1]);
                emitPoint(points[2]);
                emitPoint(points[3]);
                std::cout << ']';
                break;
            case rive::PathVerb::close:
                std::cout << "[\"Z\"]";
                break;
        }
    }
    std::cout << ']';
}

int main(int argc, char** argv)
{
    if (argc != 2)
    {
        std::cerr << "usage: loc013-cpp-probe FONT\n";
        return 2;
    }
    std::ifstream input(argv[1], std::ios::binary);
    if (!input)
    {
        std::cerr << "failed to read font\n";
        return 2;
    }
    std::vector<uint8_t> bytes((std::istreambuf_iterator<char>(input)),
                               std::istreambuf_iterator<char>());
    auto base = HBFont::Decode(rive::Span<const uint8_t>(bytes));
    if (base == nullptr)
    {
        std::cerr << "failed to decode font\n";
        return 2;
    }

    std::cout << std::setprecision(9);
    std::cout << "{\"font_bytes\":" << bytes.size()
              << ",\"face_index\":0,\"axis_tag\":\"wght\",\"results\":[";
    const float weights[] = {400.0f, 500.0f, 600.0f, 700.0f};
    for (size_t weightIndex = 0; weightIndex < 4; ++weightIndex)
    {
        const float weight = weights[weightIndex];
        const std::string text =
            std::to_string(static_cast<int>(weight)) + " Inter sample";
        std::vector<rive::Unichar> codepoints;
        codepoints.reserve(text.size());
        for (unsigned char byte : text)
        {
            codepoints.push_back(byte);
        }
        const rive::Font::Coord coord{kWeightTag, weight};
        auto varied = base->makeAtCoord(coord);
        rive::TextRun textRun{varied,
                              17.0f,
                              22.0f,
                              0.0f,
                              static_cast<uint32_t>(codepoints.size()),
                              static_cast<uint32_t>(HB_SCRIPT_LATIN),
                              0,
                              0};
        auto paragraphs = varied->shapeText(
            rive::Span<const rive::Unichar>(codepoints),
            rive::Span<const rive::TextRun>(&textRun, 1));

        if (weightIndex != 0)
        {
            std::cout << ',';
        }
        std::cout << "{\"weight\":" << weight
                  << ",\"axis_value\":" << varied->getAxisValue(kWeightTag)
                  << ",\"text\":\"" << text << "\",\"glyphs\":[";
        bool firstGlyph = true;
        for (const auto& paragraph : paragraphs)
        {
            for (const auto& run : paragraph.runs)
            {
                for (size_t glyphIndex = 0; glyphIndex < run.glyphs.size();
                     ++glyphIndex)
                {
                    if (!firstGlyph)
                    {
                        std::cout << ',';
                    }
                    firstGlyph = false;
                    const auto glyph = run.glyphs[glyphIndex];
                    std::cout << "{\"id\":" << glyph
                              << ",\"advance\":" << run.advances[glyphIndex]
                              << ",\"outline\":";
                    emitPath(run.font->getPath(glyph));
                    std::cout << '}';
                }
            }
        }
        std::cout << "]}";
    }
    std::cout << "]}\n";
    return 0;
}
