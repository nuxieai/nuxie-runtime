// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/tests/goldens/goldens.cpp
#include "recording_renderer.hpp"

#include "rive/animation/state_machine_instance.hpp"
#include "rive/artboard.hpp"
#include "rive/assets/audio_asset.hpp"
#include "rive/assets/file_asset_contents.hpp"
#include "rive/assets/font_asset.hpp"
#include "rive/assets/image_asset.hpp"
#include "rive/core/binary_reader.hpp"
#include "rive/file.hpp"
#include "rive/generated/assets/blob_asset_base.hpp"
#include "rive/generated/assets/manifest_asset_base.hpp"
#include "rive/generated/assets/script_asset_base.hpp"
#include "rive/generated/assets/shader_asset_base.hpp"
#include "rive/generated/core_registry.hpp"
#include "rive/math/raw_path.hpp"
#include "rive/math/vec2d.hpp"
#include "rive/refcnt.hpp"
#include "rive/runtime_header.hpp"
#include "rive/scene.hpp"
#include "rive/static_scene.hpp"

#include <algorithm>
#include <cerrno>
#include <chrono>
#include <cmath>
#include <cctype>
#include <cstdio>
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
    bool benchmark = false;
    std::string file;
    std::string artboard;
    std::string stateMachine;
    std::string inputScript;
    std::string viewModelScript;
    std::vector<float> samples = {0.0f};
    size_t benchmarkRepeat = 1;
};

std::string usage()
{
    return "usage: rive_golden_runner [--smoke]\n"
           "       rive_golden_runner --file <path> [--artboard <name>]\n"
           "           [--state-machine <name>] [--samples <t0,t1,...>]\n"
           "           [--input-script <path>] [--benchmark]\n"
           "           [--benchmark-repeat N]\n"
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

size_t parsePositiveSize(const std::string& value, const std::string& context)
{
    const int parsed = parseInt(value, context);
    if (parsed <= 0)
    {
        throw CliError(context + " must be greater than 0");
    }
    return static_cast<size_t>(parsed);
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
        else if (arg == "--benchmark")
        {
            options.benchmark = true;
        }
        else if (arg == "--benchmark-repeat")
        {
            options.benchmarkRepeat = parsePositiveSize(requireValue(arg), arg);
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
    if (options.benchmarkRepeat > 1)
    {
        if (!options.benchmark)
        {
            throw CliError("--benchmark-repeat requires --benchmark");
        }
        if (!options.inputScript.empty())
        {
            throw CliError(
                "--benchmark-repeat cannot be combined with --input-script");
        }
        if (options.samples.size() != 1)
        {
            throw CliError("--benchmark-repeat requires exactly one sample");
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

#ifndef WITH_RIVE_SCRIPTING
// Byte-exact mirror of the static readRuntimeObject in
// $RIVE_RUNTIME_DIR/src/file.cpp, used to walk the object stream without
// importing it. Consumes exactly the bytes File::read would consume for one
// object.
rive::Core* walkRuntimeObject(rive::BinaryReader& reader,
                              const rive::RuntimeHeader& header)
{
    auto coreObjectKey = reader.readVarUintAs<int>();
    auto object = rive::CoreRegistry::makeCoreInstance(coreObjectKey);
    while (true)
    {
        auto propertyKey = reader.readVarUintAs<uint16_t>();
        if (propertyKey == 0)
        {
            break;
        }
        if (reader.hasError())
        {
            delete object;
            return nullptr;
        }
        if (object == nullptr || !object->deserialize(propertyKey, reader))
        {
            int id = rive::CoreRegistry::propertyFieldId(propertyKey);
            if (id == -1)
            {
                id = header.propertyFieldId(propertyKey);
            }
            if (id == -1)
            {
                delete object;
                return nullptr;
            }
            switch (id)
            {
                case rive::CoreUintType::id:
                    rive::CoreUintType::deserialize(reader);
                    break;
                case rive::CoreStringType::id:
                    rive::CoreStringType::deserialize(reader);
                    break;
                case rive::CoreDoubleType::id:
                    rive::CoreDoubleType::deserialize(reader);
                    break;
                case rive::CoreColorType::id:
                    rive::CoreColorType::deserialize(reader);
                    break;
            }
        }
    }
    return object;
}
#endif

// The reference librive we link is built without WITH_RIVE_SCRIPTING. In that
// configuration File::read pushes no FileAssetImporter for ScriptAsset /
// ShaderAsset objects, so an in-band FileAssetContents that belongs to a
// script asset is routed to the previous file asset's importer instead. When
// that importer already holds its own in-band contents, the debug build
// aborts on assert(!m_content) in FileAssetImporter::onFileAssetContents
// before any stream is produced.
//
// To keep those corpus files runnable we pre-strip exactly the
// FileAssetContents objects that would trip that assert, by simulating the
// same import-stack routing File::read performs. Files that import without
// aborting today are untouched: any object we would strip is one that aborts
// the process, so a currently-passing file can never contain one.
std::vector<uint8_t> stripAbortingAssetContents(std::vector<uint8_t> bytes)
{
#ifdef WITH_RIVE_SCRIPTING
    return bytes;
#else
    rive::BinaryReader reader(
        rive::Span<const uint8_t>(bytes.data(), bytes.size()));
    rive::RuntimeHeader header;
    if (!rive::RuntimeHeader::read(reader, header) ||
        header.majorVersion() != rive::File::majorVersion)
    {
        // Malformed or unsupported: let File::import produce its usual error.
        return bytes;
    }

    struct ByteRange
    {
        size_t begin;
        size_t end;
    };
    std::vector<ByteRange> drops;

    const uint8_t* base = bytes.data();
    // Mirrors the FileAssetImporter the import stack would hold: whether one
    // exists, and whether onFileAssetContents was already called on it.
    bool importerExists = false;
    bool importerHasContent = false;
    while (!reader.reachedEnd())
    {
        const uint8_t* objectStart = reader.position();
        rive::Core* object = walkRuntimeObject(reader, header);
        if (reader.hasError())
        {
            // Unreadable tail: keep the original bytes untouched.
            delete object;
            return bytes;
        }
        const uint8_t* objectEnd = reader.position();
        if (object == nullptr)
        {
            continue;
        }
        switch (object->coreType())
        {
            // The asset types File::read pushes a FileAssetImporter for.
            case rive::ImageAsset::typeKey:
            case rive::FontAsset::typeKey:
            case rive::AudioAsset::typeKey:
            case rive::BlobAssetBase::typeKey:
            case rive::ManifestAssetBase::typeKey:
                importerExists = true;
                importerHasContent = false;
                break;
            // No importer is pushed for these without WITH_RIVE_SCRIPTING.
            case rive::ScriptAssetBase::typeKey:
            case rive::ShaderAssetBase::typeKey:
                break;
            case rive::FileAssetContents::typeKey:
                if (importerExists && importerHasContent)
                {
                    // This delivery would abort on assert(!m_content).
                    drops.push_back({static_cast<size_t>(objectStart - base),
                                     static_cast<size_t>(objectEnd - base)});
                }
                else if (importerExists)
                {
                    importerHasContent = true;
                }
                break;
            default:
                break;
        }
        delete object;
    }

    if (drops.empty())
    {
        return bytes;
    }

    std::vector<uint8_t> stripped;
    stripped.reserve(bytes.size());
    size_t copyFrom = 0;
    for (const ByteRange& drop : drops)
    {
        stripped.insert(stripped.end(),
                        bytes.begin() + copyFrom,
                        bytes.begin() + drop.begin);
        copyFrom = drop.end;
    }
    stripped.insert(stripped.end(), bytes.begin() + copyFrom, bytes.end());
    return stripped;
#endif
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

#ifdef WITH_RIVE_SCRIPTING
        const int viewModelId = m_artboard->viewModelId();
        m_viewModelInstance =
            viewModelId == -1
                ? m_file->createViewModelInstance(m_artboard.get())
                : m_file->createViewModelInstance(viewModelId, 0);
#else
        m_viewModelInstance = m_file->createViewModelInstance(m_artboard.get());
#endif
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

double durationMillis(std::chrono::steady_clock::duration duration)
{
    return std::chrono::duration<double, std::milli>(duration).count();
}

struct BenchmarkTimings
{
    std::chrono::steady_clock::duration elapsed{};
    std::chrono::steady_clock::duration advance{};
    std::chrono::steady_clock::duration input{};
    std::chrono::steady_clock::duration draw{};
};

BenchmarkTimings runBenchmarkPass(const Options& options, bool collectPhases)
{
    rive_rust::golden::NullFactory nullFactory;
    RIVLoader loader(stripAbortingAssetContents(readFile(options.file)),
                     options.artboard,
                     options.stateMachine,
                     &nullFactory);
    rive::Scene* scene = loader.scene();
    auto renderer = nullFactory.makeRenderer();

    BenchmarkTimings timings;
    auto timedStage = [&](auto& elapsed, auto&& action) {
        if (!collectPhases)
        {
            action();
            return;
        }
        const auto stageStart = std::chrono::steady_clock::now();
        action();
        elapsed += std::chrono::steady_clock::now() - stageStart;
    };

    float currentSeconds = 0.0f;
    const auto benchmarkStart = std::chrono::steady_clock::now();
    for (size_t repeat = 0; repeat < options.benchmarkRepeat; repeat++)
    {
        for (float sampleSeconds : options.samples)
        {
            timedStage(timings.advance, [&] {
                advanceTo(scene, sampleSeconds, currentSeconds);
            });
            timedStage(timings.draw, [&] { scene->draw(renderer.get()); });
        }
    }
    timings.elapsed = std::chrono::steady_clock::now() - benchmarkStart;
    return timings;
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

    if (options.benchmark && options.benchmarkRepeat > 1)
    {
        const auto totalTimings = runBenchmarkPass(options, false);
        const auto phaseTimings = runBenchmarkPass(options, true);
        const auto phaseBookkeepingElapsed = phaseTimings.elapsed -
                                             phaseTimings.advance -
                                             phaseTimings.input -
                                             phaseTimings.draw;
        std::cout << "rive-golden-benchmark-v1\n"
                  << "elapsed_ms=" << durationMillis(totalTimings.elapsed)
                  << "\n"
                  << "total_ms=" << durationMillis(totalTimings.elapsed)
                  << "\n"
                  << "advance_ms=" << durationMillis(phaseTimings.advance)
                  << "\n"
                  << "input_ms=" << durationMillis(phaseTimings.input)
                  << "\n"
                  << "prepare_ms=0\n"
                  << "draw_ms=" << durationMillis(phaseTimings.draw) << "\n"
                  << "bookkeeping_ms="
                  << durationMillis(phaseBookkeepingElapsed) << "\n"
                  << "segments="
                  << options.samples.size() * options.benchmarkRepeat << "\n";

        std::cout.flush();
        std::fflush(nullptr);
#ifndef WITH_RIVE_SCRIPTING
        std::_Exit(0);
#endif
        return 0;
    }

    rive_rust::golden::RecordingFactory recordingFactory;
    rive_rust::golden::NullFactory nullFactory;
    rive::Factory* factory = options.benchmark ? static_cast<rive::Factory*>(&nullFactory)
                                               : static_cast<rive::Factory*>(&recordingFactory);
    RIVLoader loader(stripAbortingAssetContents(readFile(options.file)),
                     options.artboard,
                     options.stateMachine,
                     factory);
    rive::Scene* scene = loader.scene();
    auto renderer = options.benchmark ? nullFactory.makeRenderer()
                                      : recordingFactory.makeRenderer();

    if (!options.benchmark)
    {
        recordingFactory.source(options.file,
                                loader.artboardName(),
                                scene->name());
        recordingFactory.frameSize(frameDimension(scene->width()),
                                   frameDimension(scene->height()));
    }

    const auto benchmarkStart = std::chrono::steady_clock::now();
    std::chrono::steady_clock::duration advanceElapsed{};
    std::chrono::steady_clock::duration inputElapsed{};
    std::chrono::steady_clock::duration drawElapsed{};
    auto timedStage = [&](auto& elapsed, auto&& action) {
        if (!options.benchmark)
        {
            action();
            return;
        }
        const auto stageStart = std::chrono::steady_clock::now();
        action();
        elapsed += std::chrono::steady_clock::now() - stageStart;
    };
    float currentSeconds = 0.0f;
    size_t nextInput = 0;
    for (size_t repeat = 0; repeat < options.benchmarkRepeat; repeat++)
    {
        for (float sampleSeconds : options.samples)
        {
            while (nextInput < inputEvents.size() &&
                   inputEvents[nextInput].seconds <=
                       sampleSeconds + kTimeEpsilon)
            {
                const auto& event = inputEvents[nextInput];
                timedStage(advanceElapsed, [&] {
                    advanceTo(scene, event.seconds, currentSeconds);
                });
                timedStage(inputElapsed, [&] {
                    applyInput(scene, event);
                    if (!options.benchmark)
                    {
                        recordingFactory.addInputEvent(
                            inputKindName(event.kind),
                            event.seconds,
                            event.x,
                            event.y,
                            event.pointerId);
                    }
                });
                nextInput++;
            }

            timedStage(advanceElapsed, [&] {
                advanceTo(scene, sampleSeconds, currentSeconds);
            });
            if (!options.benchmark)
            {
                recordingFactory.addSample(sampleSeconds);
            }
            timedStage(drawElapsed, [&] { scene->draw(renderer.get()); });
            if (!options.benchmark)
            {
                recordingFactory.addFrame();
            }
        }
    }
    const auto benchmarkElapsed =
        std::chrono::steady_clock::now() - benchmarkStart;

    if (options.benchmark)
    {
        const auto bookkeepingElapsed =
            benchmarkElapsed - advanceElapsed - inputElapsed - drawElapsed;
        std::cout << "rive-golden-benchmark-v1\n"
                  << "elapsed_ms=" << durationMillis(benchmarkElapsed) << "\n"
                  << "advance_ms=" << durationMillis(advanceElapsed) << "\n"
                  << "input_ms=" << durationMillis(inputElapsed) << "\n"
                  << "prepare_ms=0\n"
                  << "draw_ms=" << durationMillis(drawElapsed) << "\n"
                  << "bookkeeping_ms=" << durationMillis(bookkeepingElapsed)
                  << "\n"
                  << "segments="
                  << options.samples.size() * options.benchmarkRepeat << "\n";
    }
    else
    {
        std::cout << recordingFactory.stream();
    }

    // In the default unscripted reference build, skip destructors after the
    // stream is complete. Files with script objects can otherwise segfault
    // during teardown after emitting the stream.
    std::cout.flush();
    std::fflush(nullptr);
#ifndef WITH_RIVE_SCRIPTING
    std::_Exit(0);
#endif
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
