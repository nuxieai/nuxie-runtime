// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/tests/goldens/goldens.cpp
#include "recording_renderer.hpp"

#include "rive/animation/state_machine_instance.hpp"
#include "rive/artboard.hpp"
#include "rive/file.hpp"
#include "rive/math/raw_path.hpp"
#include "rive/math/vec2d.hpp"
#include "rive/refcnt.hpp"
#include "rive/scene.hpp"
#include "rive/static_scene.hpp"

#include <algorithm>
#include <cerrno>
#include <cmath>
#include <cctype>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <iterator>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace
{
constexpr float kTimeEpsilon = 0.000001f;

class CliError : public std::runtime_error
{
public:
    explicit CliError(const std::string& message) : std::runtime_error(message)
    {}
};

enum class InputKind
{
    pointerDown,
    pointerMove,
    pointerUp,
    pointerExit,
};

struct InputEvent
{
    float seconds = 0.0f;
    InputKind kind = InputKind::pointerMove;
    float x = 0.0f;
    float y = 0.0f;
    int pointerId = 0;
    size_t order = 0;
};

struct Options
{
    bool smoke = false;
    bool help = false;
    std::string file;
    std::string artboard;
    std::string stateMachine;
    std::string inputScript;
    std::string viewModelScript;
    std::vector<float> samples = {0.0f};
};

std::string usage()
{
    return "usage: rive_golden_runner [--smoke]\n"
           "       rive_golden_runner --file <path> [--artboard <name>]\n"
           "           [--state-machine <name>] [--samples <t0,t1,...>]\n"
           "           [--input-script <path>]\n"
           "\n"
           "input script lines:\n"
           "  <seconds> pointerDown <x> <y> [pointerId]\n"
           "  <seconds> pointerMove <x> <y> [pointerId]\n"
           "  <seconds> pointerUp <x> <y> [pointerId]\n"
           "  <seconds> pointerExit <x> <y> [pointerId]\n";
}

std::string trim(const std::string& value)
{
    size_t start = 0;
    while (start < value.size() &&
           std::isspace(static_cast<unsigned char>(value[start])))
    {
        start++;
    }

    size_t end = value.size();
    while (end > start &&
           std::isspace(static_cast<unsigned char>(value[end - 1])))
    {
        end--;
    }

    return value.substr(start, end - start);
}

float parseFloat(const std::string& value, const std::string& context)
{
    errno = 0;
    char* end = nullptr;
    const float parsed = std::strtof(value.c_str(), &end);
    if (end == value.c_str() || *end != '\0' || errno == ERANGE)
    {
        throw CliError("invalid float for " + context + ": " + value);
    }
    return parsed;
}

int parseInt(const std::string& value, const std::string& context)
{
    errno = 0;
    char* end = nullptr;
    const long parsed = std::strtol(value.c_str(), &end, 10);
    if (end == value.c_str() || *end != '\0' || errno == ERANGE)
    {
        throw CliError("invalid integer for " + context + ": " + value);
    }
    return static_cast<int>(parsed);
}

std::vector<float> parseSamples(const std::string& value)
{
    std::vector<float> samples;
    std::stringstream parts(value);
    std::string part;
    while (std::getline(parts, part, ','))
    {
        part = trim(part);
        if (part.empty())
        {
            throw CliError("empty sample in --samples");
        }
        samples.push_back(parseFloat(part, "--samples"));
    }

    if (samples.empty())
    {
        throw CliError("--samples must contain at least one time");
    }

    for (size_t index = 0; index < samples.size(); index++)
    {
        if (samples[index] < 0.0f)
        {
            throw CliError("--samples must be non-negative");
        }
        if (index != 0 && samples[index] + kTimeEpsilon < samples[index - 1])
        {
            throw CliError("--samples must be sorted in ascending order");
        }
    }

    return samples;
}

InputKind parseInputKind(const std::string& value, size_t lineNumber)
{
    if (value == "pointerDown")
    {
        return InputKind::pointerDown;
    }
    if (value == "pointerMove")
    {
        return InputKind::pointerMove;
    }
    if (value == "pointerUp")
    {
        return InputKind::pointerUp;
    }
    if (value == "pointerExit")
    {
        return InputKind::pointerExit;
    }
    throw CliError("unknown input event on line " + std::to_string(lineNumber) +
                   ": " + value);
}

std::string inputKindName(InputKind kind)
{
    switch (kind)
    {
        case InputKind::pointerDown:
            return "pointerDown";
        case InputKind::pointerMove:
            return "pointerMove";
        case InputKind::pointerUp:
            return "pointerUp";
        case InputKind::pointerExit:
            return "pointerExit";
    }
    return "unknown";
}

std::vector<InputEvent> loadInputScript(const std::string& path)
{
    std::ifstream stream(path);
    if (!stream.good())
    {
        throw std::runtime_error("unable to read input script: " + path);
    }

    std::vector<InputEvent> events;
    std::string line;
    size_t lineNumber = 0;
    while (std::getline(stream, line))
    {
        lineNumber++;
        const auto commentStart = line.find('#');
        if (commentStart != std::string::npos)
        {
            line = line.substr(0, commentStart);
        }
        line = trim(line);
        if (line.empty())
        {
            continue;
        }

        std::istringstream words(line);
        std::vector<std::string> tokens;
        std::string token;
        while (words >> token)
        {
            tokens.push_back(token);
        }

        if (tokens.size() != 4 && tokens.size() != 5)
        {
            throw CliError("input script line " + std::to_string(lineNumber) +
                           " must be: <seconds> <event> <x> <y> [pointerId]");
        }

        InputEvent event;
        event.seconds = parseFloat(tokens[0],
                                   "input script line " +
                                       std::to_string(lineNumber) +
                                       " seconds");
        if (event.seconds < 0.0f)
        {
            throw CliError("input script line " + std::to_string(lineNumber) +
                           " has a negative time");
        }
        event.kind = parseInputKind(tokens[1], lineNumber);
        event.x =
            parseFloat(tokens[2],
                       "input script line " + std::to_string(lineNumber) + " x");
        event.y =
            parseFloat(tokens[3],
                       "input script line " + std::to_string(lineNumber) + " y");
        event.pointerId =
            tokens.size() == 5
                ? parseInt(tokens[4],
                           "input script line " + std::to_string(lineNumber) +
                               " pointerId")
                : 0;
        event.order = events.size();
        events.push_back(event);
    }

    std::stable_sort(events.begin(), events.end(), [](const auto& a,
                                                      const auto& b) {
        if (std::abs(a.seconds - b.seconds) <= kTimeEpsilon)
        {
            return a.order < b.order;
        }
        return a.seconds < b.seconds;
    });

    return events;
}

Options parseOptions(int argc, char** argv)
{
    Options options;
    bool samplesSet = false;

    for (int index = 1; index < argc; index++)
    {
        const std::string arg = argv[index];

        auto requireValue = [&](const std::string& option) -> std::string {
            if (index + 1 >= argc)
            {
                throw CliError(option + " requires a value");
            }
            index++;
            return argv[index];
        };

        if (arg == "--help" || arg == "-h")
        {
            options.help = true;
        }
        else if (arg == "--smoke")
        {
            options.smoke = true;
        }
        else if (arg == "--file")
        {
            options.file = requireValue(arg);
        }
        else if (arg == "--artboard")
        {
            options.artboard = requireValue(arg);
        }
        else if (arg == "--state-machine")
        {
            options.stateMachine = requireValue(arg);
        }
        else if (arg == "--samples")
        {
            options.samples = parseSamples(requireValue(arg));
            samplesSet = true;
        }
        else if (arg == "--sample")
        {
            const float sample = parseFloat(requireValue(arg), arg);
            if (!samplesSet)
            {
                options.samples.clear();
                samplesSet = true;
            }
            options.samples.push_back(sample);
        }
        else if (arg == "--input-script")
        {
            options.inputScript = requireValue(arg);
        }
        else if (arg == "--view-model-script")
        {
            options.viewModelScript = requireValue(arg);
        }
        else if (!arg.empty() && arg[0] == '-')
        {
            throw CliError("unknown option: " + arg);
        }
        else if (options.file.empty())
        {
            options.file = arg;
        }
        else
        {
            throw CliError("unexpected positional argument: " + arg);
        }
    }

    for (size_t index = 0; index < options.samples.size(); index++)
    {
        if (options.samples[index] < 0.0f)
        {
            throw CliError("sample times must be non-negative");
        }
        if (index != 0 &&
            options.samples[index] + kTimeEpsilon < options.samples[index - 1])
        {
            throw CliError("sample times must be sorted in ascending order");
        }
    }

    return options;
}

std::vector<uint8_t> readFile(const std::string& path)
{
    std::ifstream stream(path, std::ios::binary);
    if (!stream.good())
    {
        throw std::runtime_error("unable to read riv file: " + path);
    }
    return std::vector<uint8_t>(std::istreambuf_iterator<char>(stream), {});
}

class RIVLoader
{
public:
    RIVLoader(const std::vector<uint8_t>& rivBytes,
              const std::string& artboardName,
              const std::string& stateMachineName,
              rive::Factory* factory)
    {
        rive::ImportResult importResult = rive::ImportResult::success;
        m_file = rive::File::import(rivBytes, factory, &importResult);
        if (m_file == nullptr)
        {
            std::ostringstream out;
            out << "bad riv file; import result="
                << static_cast<int>(importResult);
            throw std::runtime_error(out.str());
        }

        if (!artboardName.empty())
        {
            m_artboard = m_file->artboardNamed(artboardName);
        }
        else
        {
            m_artboard = m_file->artboardDefault();
        }
        if (m_artboard == nullptr)
        {
            throw std::runtime_error("can't load artboard");
        }

        m_viewModelInstance = m_file->createViewModelInstance(m_artboard.get());
        m_artboard->bindViewModelInstance(m_viewModelInstance);

        if (!stateMachineName.empty())
        {
            m_scene = m_artboard->stateMachineNamed(stateMachineName);
        }
        else
        {
            m_scene = m_artboard->defaultStateMachine();
        }

        if (m_scene == nullptr)
        {
            m_scene = std::make_unique<rive::StaticScene>(m_artboard.get());
        }

        if (m_viewModelInstance != nullptr)
        {
            m_scene->bindViewModelInstance(m_viewModelInstance);
        }
    }

    rive::Scene* scene() const { return m_scene.get(); }
    const std::string& artboardName() const { return m_artboard->name(); }

private:
    rive::rcp<rive::File> m_file;
    std::unique_ptr<rive::ArtboardInstance> m_artboard;
    std::unique_ptr<rive::Scene> m_scene;
    rive::rcp<rive::ViewModelInstance> m_viewModelInstance;
};

void advanceTo(rive::Scene* scene, float targetSeconds, float& currentSeconds)
{
    if (targetSeconds + kTimeEpsilon < currentSeconds)
    {
        throw CliError("cannot move timeline backwards");
    }

    float elapsed = targetSeconds - currentSeconds;
    if (elapsed < 0.0f)
    {
        elapsed = 0.0f;
    }
    scene->advanceAndApply(elapsed);
    currentSeconds = targetSeconds;
}

void applyInput(rive::Scene* scene, const InputEvent& event)
{
    const rive::Vec2D position(event.x, event.y);
    switch (event.kind)
    {
        case InputKind::pointerDown:
            scene->pointerDown(position, event.pointerId);
            break;
        case InputKind::pointerMove:
            scene->pointerMove(position, event.seconds, event.pointerId);
            break;
        case InputKind::pointerUp:
            scene->pointerUp(position, event.pointerId);
            break;
        case InputKind::pointerExit:
            scene->pointerExit(position, event.pointerId);
            break;
    }
}

uint32_t frameDimension(float value)
{
    return static_cast<uint32_t>(std::max(1.0f, std::ceil(value)));
}

int runSmoke()
{
    rive_rust::golden::RecordingFactory factory;
    auto renderer = factory.makeRenderer();
    auto path = factory.makeEmptyRenderPath();
    auto paint = factory.makeRenderPaint();

    path->moveTo(0.0f, 0.0f);
    path->lineTo(10.0f, 0.0f);
    path->lineTo(10.0f, 10.0f);
    path->close();
    paint->color(0xff336699);

    factory.source("smoke", "", "manual");
    factory.frameSize(64, 64);
    factory.addSample(0.0f);
    renderer->save();
    renderer->drawPath(path.get(), paint.get());
    renderer->restore();
    factory.addFrame();

    std::cout << factory.stream();
    return 0;
}

int runFile(const Options& options)
{
    if (!options.viewModelScript.empty())
    {
        throw CliError(
            "unsupported: view-model scripts are not implemented in "
            "golden-runner yet");
    }
    if (options.file.empty())
    {
        throw CliError("missing --file <path>");
    }

    std::vector<InputEvent> inputEvents;
    if (!options.inputScript.empty())
    {
        inputEvents = loadInputScript(options.inputScript);
    }

    rive::File::deterministicMode = true;

    rive_rust::golden::RecordingFactory factory;
    RIVLoader loader(readFile(options.file),
                     options.artboard,
                     options.stateMachine,
                     &factory);
    rive::Scene* scene = loader.scene();
    auto renderer = factory.makeRenderer();

    factory.source(options.file, loader.artboardName(), scene->name());
    factory.frameSize(frameDimension(scene->width()),
                      frameDimension(scene->height()));

    float currentSeconds = 0.0f;
    size_t nextInput = 0;
    for (float sampleSeconds : options.samples)
    {
        while (nextInput < inputEvents.size() &&
               inputEvents[nextInput].seconds <= sampleSeconds + kTimeEpsilon)
        {
            const auto& event = inputEvents[nextInput];
            advanceTo(scene, event.seconds, currentSeconds);
            applyInput(scene, event);
            factory.addInputEvent(inputKindName(event.kind),
                                  event.seconds,
                                  event.x,
                                  event.y,
                                  event.pointerId);
            nextInput++;
        }

        advanceTo(scene, sampleSeconds, currentSeconds);
        factory.addSample(sampleSeconds);
        scene->draw(renderer.get());
        factory.addFrame();
    }

    std::cout << factory.stream();
    return 0;
}
} // namespace

int main(int argc, char** argv)
{
    try
    {
        const Options options = parseOptions(argc, argv);
        if (options.help)
        {
            std::cout << usage();
            return 0;
        }
        if (options.smoke)
        {
            return runSmoke();
        }
        return runFile(options);
    }
    catch (const CliError& error)
    {
        std::cerr << error.what() << "\n\n" << usage();
        return 2;
    }
    catch (const std::exception& error)
    {
        std::cerr << "golden-runner error: " << error.what() << '\n';
        return 1;
    }
}
