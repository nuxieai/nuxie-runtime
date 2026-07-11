#include "recording_renderer.hpp"

#include "common/testing_window.hpp"
#include "gm/gm.hpp"

#include <algorithm>
#include <fstream>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>

using rive_rust::golden::RecordingFactory;
using rive_rust::golden::RecordingRenderer;

namespace
{
TestingWindow* g_testingWindow = nullptr;

class RecordingWindow final : public TestingWindow
{
public:
    rive::Factory* factory() override { return &m_factory; }

    std::unique_ptr<rive::Renderer> beginFrame(const FrameOptions&) override
    {
        return m_factory.makeRenderer();
    }

    void endFrame(std::vector<uint8_t>*) override { m_factory.addFrame(); }

    RecordingFactory& recordingFactory() { return m_factory; }

private:
    RecordingFactory m_factory;
};
} // namespace

TestingWindow::Backend TestingWindow::s_Backend = TestingWindow::Backend::null;

TestingWindow* TestingWindow::Get() { return g_testingWindow; }

void TestingWindow::Set(TestingWindow* window) { g_testingWindow = window; }

void TestingWindow::Destroy()
{
    delete g_testingWindow;
    g_testingWindow = nullptr;
}

#define GM_ENTRY(name) extern "C" rivegm::GM* make_##name();
#include "generated_registry.inc"
#undef GM_ENTRY

namespace
{
struct GMEntry
{
    const char* name;
    rivegm::GM* (*make)();
};

constexpr GMEntry registry[] = {
#define GM_ENTRY(name) {#name, make_##name},
#include "generated_registry.inc"
#undef GM_ENTRY
};
} // namespace

int main(int argc, const char* argv[])
{
    try
    {
        if (argc != 3)
        {
            throw std::runtime_error("usage: gm_stream_capture <gm> <output>");
        }
        const std::string requestedName = argv[1];
        const auto* entry = std::find_if(
            std::begin(registry), std::end(registry), [&](const GMEntry& entry) {
                return requestedName == entry.name;
            });
        if (entry == std::end(registry))
        {
            throw std::runtime_error("unknown or gated GM: " + requestedName);
        }
        RecordingWindow window;
        TestingWindow::Set(&window);
        std::unique_ptr<rivegm::GM> gm(entry->make());
        gm->onceBeforeDraw();

        auto& factory = window.recordingFactory();
        factory.source("gm:" + requestedName, "", requestedName);
        factory.frameSize(gm->width(), gm->height());
        factory.clearColor(gm->clearColor());
        auto renderer = factory.makeRenderer();
        gm->draw(renderer.get());
        factory.addFrame();

        std::ofstream output(argv[2]);
        output << factory.stream();
        TestingWindow::Set(nullptr);
        return output.good() ? 0 : 1;
    }
    catch (const std::exception& error)
    {
        TestingWindow::Set(nullptr);
        std::cerr << error.what() << '\n';
        return 1;
    }
}
