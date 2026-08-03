// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/tests/goldens/goldens.cpp
#include "recording_renderer.hpp"

#include "rive/animation/state_machine_instance.hpp"
#include "rive/animation/state_machine_input_instance.hpp"
#include "rive/animation/semantic_listener_group.hpp"
#include "rive/artboard.hpp"
#include "rive/custom_property_boolean.hpp"
#include "rive/custom_property_color.hpp"
#include "rive/custom_property_enum.hpp"
#include "rive/custom_property_number.hpp"
#include "rive/custom_property_string.hpp"
#include "rive/custom_property_trigger.hpp"
#include "rive/event.hpp"
#include "rive/event_report.hpp"
#include "rive/open_url_event.hpp"
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
#include "rive/semantic/semantic_manager.hpp"
#include "rive/semantic/semantic_node.hpp"
#include "rive/static_scene.hpp"
#include "rive/viewmodel/viewmodel_instance.hpp"
#include "rive/viewmodel/viewmodel_instance_boolean.hpp"
#include "rive/viewmodel/viewmodel_instance_number.hpp"
#include "rive/viewmodel/viewmodel_instance_trigger.hpp"

#include <algorithm>
#include <atomic>
#include <cerrno>
#include <chrono>
#include <cmath>
#include <cctype>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <iterator>
#include <limits>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

#if defined(RIVE_GOLDEN_COVERAGE_TRACE)
namespace
{
std::atomic<bool> g_countFrameLoopAllocations = false;
std::atomic<uint64_t> g_frameLoopAllocations = 0;
}

void* operator new(std::size_t size)
{
    if (g_countFrameLoopAllocations.load(std::memory_order_relaxed))
    {
        g_frameLoopAllocations.fetch_add(1, std::memory_order_relaxed);
    }
    if (void* value = std::malloc(size))
    {
        return value;
    }
    throw std::bad_alloc();
}

void* operator new[](std::size_t size)
{
    return ::operator new(size);
}
#endif

namespace
{
#if defined(RIVE_GOLDEN_COVERAGE_TRACE)
extern "C" int __llvm_profile_write_file(void);
extern "C" void __llvm_profile_reset_counters(void);
#endif

void flushCoverageProfileIfRequested()
{
#if defined(RIVE_GOLDEN_COVERAGE_TRACE)
    if (std::getenv("RIVE_GOLDEN_COVERAGE_FLUSH") != nullptr)
    {
        __llvm_profile_write_file();
    }
#endif
}

void resetCoverageProfileForFrameLoopIfRequested()
{
#if defined(RIVE_GOLDEN_COVERAGE_TRACE)
    if (std::getenv("RIVE_GOLDEN_COVERAGE_FRAME_ONLY") != nullptr)
    {
        __llvm_profile_reset_counters();
    }
#endif
}

void resetCoverageProfileForOccurrenceIfRequested()
{
#if defined(RIVE_GOLDEN_COVERAGE_TRACE)
    if (std::getenv("RIVE_GOLDEN_COVERAGE_OCCURRENCE_ONLY") != nullptr)
    {
        __llvm_profile_reset_counters();
    }
#endif
}

void resetFrameLoopAllocationCounterIfRequested()
{
#if defined(RIVE_GOLDEN_COVERAGE_TRACE)
    if (std::getenv("RIVE_GOLDEN_ALLOCATION_COUNTER") != nullptr)
    {
        g_frameLoopAllocations.store(0, std::memory_order_relaxed);
        g_countFrameLoopAllocations.store(true, std::memory_order_relaxed);
    }
#endif
}

uint64_t stopFrameLoopAllocationCounter()
{
#if defined(RIVE_GOLDEN_COVERAGE_TRACE)
    g_countFrameLoopAllocations.store(false, std::memory_order_relaxed);
    return g_frameLoopAllocations.load(std::memory_order_relaxed);
#else
    return 0;
#endif
}

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
    semanticAction,
    semanticFocus,
    setInput,
    resize,
};

enum class ScriptValueKind
{
    boolean,
    number,
    trigger,
};

enum class ViewModelKind
{
    setBoolean,
    setNumber,
    fireTrigger,
};

struct InputEvent
{
    float seconds = 0.0f;
    InputKind kind = InputKind::pointerMove;
    float x = 0.0f;
    float y = 0.0f;
    int pointerId = 0;
    uint32_t semanticNodeId = 0;
    rive::SemanticActionType semanticAction = rive::SemanticActionType::tap;
    std::string name;
    ScriptValueKind valueKind = ScriptValueKind::boolean;
    bool boolValue = false;
    float numberValue = 0.0f;
    float width = 0.0f;
    float height = 0.0f;
    float dpr = 1.0f;
    size_t order = 0;
};

struct ViewModelEvent
{
    float seconds = 0.0f;
    ViewModelKind kind = ViewModelKind::setNumber;
    std::string property;
    bool boolValue = false;
    float numberValue = 0.0f;
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
    std::string animation;
    std::string inputScript;
    std::string viewModelScript;
    std::vector<float> samples = {0.0f};
    size_t benchmarkRepeat = 1;
    bool sideChannel = false;
    bool semanticDefaultViewModel = false;
    bool semanticSideChannelOnly = false;
};

void validateTraceOptions(const Options& options)
{
    const bool frameOnly =
        std::getenv("RIVE_GOLDEN_COVERAGE_FRAME_ONLY") != nullptr;
    const bool allocations =
        std::getenv("RIVE_GOLDEN_ALLOCATION_COUNTER") != nullptr;
    const bool steadyOnly =
        std::getenv("RIVE_GOLDEN_COVERAGE_STEADY_ONLY") != nullptr;
    const bool occurrenceOnly =
        std::getenv("RIVE_GOLDEN_COVERAGE_OCCURRENCE_ONLY") != nullptr;
    const bool mechanismInput =
        std::getenv("RIVE_GOLDEN_COVERAGE_MECHANISM_INPUT") != nullptr;
#if !defined(RIVE_GOLDEN_COVERAGE_TRACE)
    const bool flush =
        std::getenv("RIVE_GOLDEN_COVERAGE_FLUSH") != nullptr;
    if (frameOnly || flush || occurrenceOnly || mechanismInput)
    {
        throw CliError(
            "coverage tracing requires RIVE_GOLDEN_COVERAGE_TRACE and LLVM "
            "coverage instrumentation at build time");
    }
#endif
    if (mechanismInput &&
        (!frameOnly || occurrenceOnly || steadyOnly ||
         options.inputScript.empty() || options.benchmarkRepeat != 1))
    {
        throw CliError(
            "mechanism input coverage requires frame-only coverage, an "
            "input script, --benchmark-repeat 1, and "
            "non-occurrence/non-steady mode");
    }

#if !defined(RIVE_GOLDEN_COVERAGE_TRACE)
    if (allocations)
    {
        throw CliError(
            "RIVE_GOLDEN_ALLOCATION_COUNTER requires "
            "RIVE_GOLDEN_COVERAGE_TRACE at build time");
    }
#endif

    if (options.benchmarkRepeat > 1 && (frameOnly || allocations))
    {
        throw CliError(
            "frame-only coverage and allocation tracing require "
            "--benchmark-repeat 1");
    }
    if (steadyOnly &&
        (!frameOnly || options.samples.size() != 1 ||
         options.benchmarkRepeat != 1 || !options.inputScript.empty() ||
         !options.viewModelScript.empty()))
    {
        throw CliError(
            "steady-only coverage requires frame-only coverage, one sample, "
            "--benchmark-repeat 1, and no input script");
    }
}

std::string usage()
{
    return "usage: rive_golden_runner [--smoke]\n"
           "       rive_golden_runner --file <path> [--artboard <name>]\n"
           "           [--state-machine <name> | --animation <name>]\n"
           "           [--samples <t0,t1,...>]\n"
           "           [--input-script <path>]\n"
           "           [--view-model-script <path>] [--side-channel]\n"
           "           [--semantic-default-view-model]\n"
           "           [--semantic-side-channel-only]\n"
           "           [--benchmark] [--benchmark-repeat N]\n"
           "\n"
           "input script lines:\n"
           "  <seconds> pointerDown <x> <y> [pointerId]\n"
           "  <seconds> pointerMove <x> <y> [pointerId]\n"
           "  <seconds> pointerUp <x> <y> [pointerId]\n"
           "  <seconds> pointerExit <x> <y> [pointerId]\n"
           "  <seconds> semanticAction <nodeId> <tap|increase|decrease>\n"
           "  <seconds> semanticFocus <nodeId>\n"
           "  <seconds> setInput <name> bool <true|false>\n"
           "  <seconds> setInput <name> number <value>\n"
           "  <seconds> setInput <name> trigger\n"
           "  <seconds> resize <width> <height> <dpr>\n"
           "\n"
           "view-model script lines:\n"
           "  <seconds> setBoolean <property> <true|false>\n"
           "  <seconds> setNumber <property> <value>\n"
           "  <seconds> fireTrigger <property>\n";
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

float parseFiniteFloat(const std::string& value, const std::string& context)
{
    const float parsed = parseFloat(value, context);
    if (!std::isfinite(parsed))
    {
        throw CliError(context + " must be finite");
    }
    return parsed;
}

bool parseBool(const std::string& value, const std::string& context)
{
    if (value == "true")
    {
        return true;
    }
    if (value == "false")
    {
        return false;
    }
    throw CliError("invalid boolean for " + context + ": " + value);
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

uint32_t parseUint32(const std::string& value, const std::string& context)
{
    errno = 0;
    char* end = nullptr;
    const unsigned long parsed = std::strtoul(value.c_str(), &end, 10);
    if (end == value.c_str() || *end != '\0' || errno == ERANGE ||
        parsed > std::numeric_limits<uint32_t>::max())
    {
        throw CliError("invalid unsigned integer for " + context + ": " +
                       value);
    }
    return static_cast<uint32_t>(parsed);
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
    if (value == "semanticAction")
    {
        return InputKind::semanticAction;
    }
    if (value == "semanticFocus")
    {
        return InputKind::semanticFocus;
    }
    if (value == "setInput")
    {
        return InputKind::setInput;
    }
    if (value == "resize")
    {
        return InputKind::resize;
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
        case InputKind::semanticAction:
            return "semanticAction";
        case InputKind::semanticFocus:
            return "semanticFocus";
        case InputKind::setInput:
            return "setInput";
        case InputKind::resize:
            return "resize";
    }
    return "unknown";
}

rive::SemanticActionType parseSemanticAction(const std::string& value,
                                             size_t lineNumber)
{
    if (value == "tap")
    {
        return rive::SemanticActionType::tap;
    }
    if (value == "increase")
    {
        return rive::SemanticActionType::increase;
    }
    if (value == "decrease")
    {
        return rive::SemanticActionType::decrease;
    }
    throw CliError("unknown semantic action on line " +
                   std::to_string(lineNumber) + ": " + value);
}

std::string semanticActionName(rive::SemanticActionType action)
{
    switch (action)
    {
        case rive::SemanticActionType::tap:
            return "tap";
        case rive::SemanticActionType::increase:
            return "increase";
        case rive::SemanticActionType::decrease:
            return "decrease";
    }
    return "tap";
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

        if (tokens.size() < 2)
        {
            throw CliError("input script line " + std::to_string(lineNumber) +
                           " must start with: <seconds> <event>");
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
        if ((event.kind == InputKind::setInput ||
             event.kind == InputKind::resize) &&
            !std::isfinite(event.seconds))
        {
            throw CliError("input script line " +
                           std::to_string(lineNumber) +
                           " seconds must be finite");
        }
        const std::string lineContext =
            "input script line " + std::to_string(lineNumber);
        if (event.kind == InputKind::semanticAction)
        {
            if (tokens.size() != 4)
            {
                throw CliError(lineContext +
                               " must be: <seconds> semanticAction <nodeId> "
                               "<tap|increase|decrease>");
            }
            event.semanticNodeId = parseUint32(tokens[2], lineContext + " nodeId");
            event.semanticAction = parseSemanticAction(tokens[3], lineNumber);
        }
        else if (event.kind == InputKind::semanticFocus)
        {
            if (tokens.size() != 3)
            {
                throw CliError(lineContext +
                               " must be: <seconds> semanticFocus <nodeId>");
            }
            event.semanticNodeId = parseUint32(tokens[2], lineContext + " nodeId");
        }
        else if (event.kind == InputKind::setInput)
        {
            if (tokens.size() < 4)
            {
                throw CliError(
                    lineContext +
                    " must be: <seconds> setInput <name> "
                    "<bool|number|trigger> [value]");
            }
            event.name = tokens[2];
            if (tokens[3] == "bool")
            {
                if (tokens.size() != 5)
                {
                    throw CliError(lineContext +
                                   " bool input requires one value");
                }
                event.valueKind = ScriptValueKind::boolean;
                event.boolValue = parseBool(tokens[4], lineContext + " value");
            }
            else if (tokens[3] == "number")
            {
                if (tokens.size() != 5)
                {
                    throw CliError(lineContext +
                                   " number input requires one value");
                }
                event.valueKind = ScriptValueKind::number;
                event.numberValue =
                    parseFiniteFloat(tokens[4], lineContext + " value");
            }
            else if (tokens[3] == "trigger")
            {
                if (tokens.size() != 4)
                {
                    throw CliError(lineContext +
                                   " trigger input takes no value");
                }
                event.valueKind = ScriptValueKind::trigger;
            }
            else
            {
                throw CliError("unknown setInput type on line " +
                               std::to_string(lineNumber) + ": " + tokens[3]);
            }
        }
        else if (event.kind == InputKind::resize)
        {
            if (tokens.size() != 5)
            {
                throw CliError(lineContext +
                               " must be: <seconds> resize <width> <height> "
                               "<dpr>");
            }
            event.width = parseFiniteFloat(tokens[2], lineContext + " width");
            event.height =
                parseFiniteFloat(tokens[3], lineContext + " height");
            event.dpr = parseFiniteFloat(tokens[4], lineContext + " dpr");
            if (event.width <= 0.0f || event.height <= 0.0f ||
                event.dpr <= 0.0f)
            {
                throw CliError(lineContext +
                               " resize width, height, and dpr must be greater "
                               "than 0");
            }
        }
        else
        {
            if (tokens.size() != 4 && tokens.size() != 5)
            {
                throw CliError(lineContext +
                               " must be: <seconds> <pointer-event> <x> <y> "
                               "[pointerId]");
            }
            event.x = parseFloat(tokens[2], lineContext + " x");
            event.y = parseFloat(tokens[3], lineContext + " y");
            event.pointerId =
                tokens.size() == 5
                    ? parseInt(tokens[4], lineContext + " pointerId")
                    : 0;
        }
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

ViewModelKind parseViewModelKind(const std::string& value, size_t lineNumber)
{
    if (value == "setBoolean")
    {
        return ViewModelKind::setBoolean;
    }
    if (value == "setNumber")
    {
        return ViewModelKind::setNumber;
    }
    if (value == "fireTrigger")
    {
        return ViewModelKind::fireTrigger;
    }
    throw CliError("unknown view-model event on line " +
                   std::to_string(lineNumber) + ": " + value);
}

std::vector<ViewModelEvent> loadViewModelScript(const std::string& path)
{
    std::ifstream stream(path);
    if (!stream.good())
    {
        throw std::runtime_error("unable to read view-model script: " + path);
    }

    std::vector<ViewModelEvent> events;
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
        if (tokens.size() < 2)
        {
            throw CliError("view-model script line " +
                           std::to_string(lineNumber) +
                           " must start with: <seconds> <event>");
        }

        const std::string lineContext =
            "view-model script line " + std::to_string(lineNumber);
        ViewModelEvent event;
        event.seconds = parseFiniteFloat(tokens[0], lineContext + " seconds");
        if (event.seconds < 0.0f)
        {
            throw CliError(lineContext + " has a negative time");
        }
        event.kind = parseViewModelKind(tokens[1], lineNumber);
        if (event.kind == ViewModelKind::fireTrigger)
        {
            if (tokens.size() != 3)
            {
                throw CliError(lineContext +
                               " must be: <seconds> fireTrigger <property>");
            }
        }
        else if (tokens.size() != 4)
        {
            throw CliError(lineContext +
                           " must be: <seconds> <setBoolean|setNumber> "
                           "<property> <value>");
        }
        event.property = tokens[2];
        if (event.kind == ViewModelKind::setBoolean)
        {
            event.boolValue = parseBool(tokens[3], lineContext + " value");
        }
        else if (event.kind == ViewModelKind::setNumber)
        {
            event.numberValue =
                parseFiniteFloat(tokens[3], lineContext + " value");
        }
        event.order = events.size();
        events.push_back(std::move(event));
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
        else if (arg == "--animation")
        {
            options.animation = requireValue(arg);
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
        else if (arg == "--side-channel")
        {
            options.sideChannel = true;
        }
        else if (arg == "--semantic-default-view-model")
        {
            options.semanticDefaultViewModel = true;
        }
        else if (arg == "--semantic-side-channel-only")
        {
            options.semanticSideChannelOnly = true;
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

    if (!options.stateMachine.empty() && !options.animation.empty())
    {
        throw CliError("--state-machine and --animation are mutually exclusive");
    }
    if (options.sideChannel && options.benchmark)
    {
        throw CliError("--side-channel cannot be combined with --benchmark");
    }
    if (options.semanticDefaultViewModel && !options.sideChannel)
    {
        throw CliError(
            "--semantic-default-view-model requires --side-channel");
    }
    if (options.semanticSideChannelOnly && !options.sideChannel)
    {
        throw CliError(
            "--semantic-side-channel-only requires --side-channel");
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
        if (!options.inputScript.empty() || !options.viewModelScript.empty())
        {
            throw CliError(
                "--benchmark-repeat cannot be combined with scripts");
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
              const std::string& animationName,
              bool semanticDefaultViewModel,
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
        // Construction evidence compares one live Artboard occurrence. The
        // source Artboard import above is definition construction, not the
        // instance owner graph measured by FL-A.
        resetCoverageProfileForOccurrenceIfRequested();

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
        m_viewModelInstance =
            semanticDefaultViewModel
                ? m_file->createDefaultViewModelInstance(m_artboard.get())
                : m_file->createViewModelInstance(m_artboard.get());
#endif
        m_artboard->bindViewModelInstance(m_viewModelInstance);

        if (!stateMachineName.empty())
        {
            auto stateMachine = m_artboard->stateMachineNamed(stateMachineName);
            if (stateMachine == nullptr)
            {
                throw std::runtime_error("state machine '" + stateMachineName +
                                         "' was not found");
            }
            m_stateMachine = stateMachine.get();
            m_scene = std::move(stateMachine);
        }
        else if (!animationName.empty())
        {
            m_scene = m_artboard->animationNamed(animationName);
            if (m_scene == nullptr)
            {
                throw std::runtime_error("linear animation '" + animationName +
                                         "' was not found");
            }
        }
        else
        {
            auto stateMachine = m_artboard->defaultStateMachine();
            m_stateMachine = stateMachine.get();
            m_scene = std::move(stateMachine);
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
    // Non-null only when the selected scene is a state machine; the no-RTTI
    // reference build cannot recover this via dynamic_cast at emit time.
    rive::StateMachineInstance* stateMachine() const { return m_stateMachine; }
    rive::ArtboardInstance* artboard() const { return m_artboard.get(); }
    rive::ViewModelInstance* viewModelInstance() const
    {
        return m_viewModelInstance.get();
    }
    const std::string& artboardName() const { return m_artboard->name(); }

private:
    rive::rcp<rive::File> m_file;
    std::unique_ptr<rive::ArtboardInstance> m_artboard;
    std::unique_ptr<rive::Scene> m_scene;
    rive::StateMachineInstance* m_stateMachine = nullptr;
    rive::rcp<rive::ViewModelInstance> m_viewModelInstance;
};

// Returns Scene::advanceAndApply's raw keep-going/needs-frame bool
// (state_machine_instance.cpp:2601-2665; static_scene.cpp:22-28).
bool advanceTo(rive::Scene* scene, float targetSeconds, float& currentSeconds)
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
    const bool keepGoing = scene->advanceAndApply(elapsed);
    currentSeconds = targetSeconds;
    return keepGoing;
}

rive::HitResult applyInput(rive::Scene* scene, const InputEvent& event)
{
    const rive::Vec2D position(event.x, event.y);
    switch (event.kind)
    {
        case InputKind::pointerDown:
            return scene->pointerDown(position, event.pointerId);
        case InputKind::pointerMove:
            return scene->pointerMove(position, event.seconds, event.pointerId);
        case InputKind::pointerUp:
            return scene->pointerUp(position, event.pointerId);
        case InputKind::pointerExit:
            return scene->pointerExit(position, event.pointerId);
        case InputKind::semanticAction:
        case InputKind::semanticFocus:
        case InputKind::setInput:
        case InputKind::resize:
            return rive::HitResult::none;
    }
    return rive::HitResult::none;
}

uint32_t resizePixelDimension(float logical, float dpr)
{
    const double pixels =
        std::ceil(static_cast<double>(logical) * static_cast<double>(dpr));
    if (!std::isfinite(pixels) || pixels < 1.0 ||
        pixels > std::numeric_limits<uint32_t>::max())
    {
        throw CliError("resize physical extent is outside the u32 range");
    }
    return static_cast<uint32_t>(pixels);
}

void applySetInput(rive_rust::golden::RecordingFactory& factory,
                   rive::StateMachineInstance* stateMachine,
                   const InputEvent& event,
                   bool record)
{
    if (stateMachine == nullptr)
    {
        throw CliError("setInput requires a state-machine scene");
    }
    switch (event.valueKind)
    {
        case ScriptValueKind::boolean:
        {
            auto* input = stateMachine->getBool(event.name);
            if (input == nullptr)
            {
                throw CliError("state-machine input '" + event.name +
                               "' was not found as bool");
            }
            input->value(event.boolValue);
            if (record)
            {
                factory.addSetInputBoolean(event.seconds,
                                           event.name,
                                           event.boolValue);
            }
            return;
        }
        case ScriptValueKind::number:
        {
            auto* input = stateMachine->getNumber(event.name);
            if (input == nullptr)
            {
                throw CliError("state-machine input '" + event.name +
                               "' was not found as number");
            }
            input->value(event.numberValue);
            if (record)
            {
                factory.addSetInputNumber(event.seconds,
                                          event.name,
                                          event.numberValue);
            }
            return;
        }
        case ScriptValueKind::trigger:
        {
            auto* input = stateMachine->getTrigger(event.name);
            if (input == nullptr)
            {
                throw CliError("state-machine input '" + event.name +
                               "' was not found as trigger");
            }
            input->fire();
            if (record)
            {
                factory.addSetInputTrigger(event.seconds, event.name);
            }
            return;
        }
    }
}

void applyViewModelEvent(rive_rust::golden::RecordingFactory& factory,
                         rive::ViewModelInstance* viewModel,
                         const ViewModelEvent& event,
                         bool record)
{
    if (viewModel == nullptr)
    {
        throw CliError("view-model script requires a bound main view model");
    }
    rive::ViewModelInstanceValue* property =
        viewModel->propertyValue(event.property);
    switch (event.kind)
    {
        case ViewModelKind::setBoolean:
            if (property == nullptr ||
                !property->is<rive::ViewModelInstanceBoolean>())
            {
                throw CliError("view-model property '" + event.property +
                               "' was not found as bool");
            }
            property->as<rive::ViewModelInstanceBoolean>()->propertyValue(
                event.boolValue);
            if (record)
            {
                factory.addViewModelBoolean(event.seconds,
                                            event.property,
                                            event.boolValue);
            }
            return;
        case ViewModelKind::setNumber:
            if (property == nullptr ||
                !property->is<rive::ViewModelInstanceNumber>())
            {
                throw CliError("view-model property '" + event.property +
                               "' was not found as number");
            }
            property->as<rive::ViewModelInstanceNumber>()->propertyValue(
                event.numberValue);
            if (record)
            {
                factory.addViewModelNumber(event.seconds,
                                           event.property,
                                           event.numberValue);
            }
            return;
        case ViewModelKind::fireTrigger:
            if (property == nullptr ||
                !property->is<rive::ViewModelInstanceTrigger>())
            {
                throw CliError("view-model property '" + event.property +
                               "' was not found as trigger");
            }
            property->as<rive::ViewModelInstanceTrigger>()->trigger();
            if (record)
            {
                factory.addViewModelTrigger(event.seconds, event.property);
            }
            return;
    }
}

void applyResize(rive_rust::golden::RecordingFactory& factory,
                 rive::ArtboardInstance* artboard,
                 const InputEvent& event,
                 bool record)
{
    artboard->width(event.width);
    artboard->height(event.height);
    if (record)
    {
        factory.addResize(event.seconds,
                          event.width,
                          event.height,
                          event.dpr,
                          resizePixelDimension(event.width, event.dpr),
                          resizePixelDimension(event.height, event.dpr));
    }
}

std::string hitResultName(rive::HitResult result)
{
    switch (result)
    {
        case rive::HitResult::none:
            return "none";
        case rive::HitResult::hit:
            return "hit";
        case rive::HitResult::hitOpaque:
            return "hitOpaque";
    }
    return "none";
}

// docs/side-channel-format.md target mapping (OpenUrlEvent targetValue).
std::string openUrlTargetName(uint32_t value)
{
    switch (value)
    {
        case 0:
            return "_blank";
        case 1:
            return "_parent";
        case 2:
            return "_self";
        case 3:
            return "_top";
    }
    return "";
}

rive_rust::golden::SideChannelEvent describeReportedEvent(
    const rive::EventReport& report)
{
    rive_rust::golden::SideChannelEvent out;
    rive::Event* event = report.event();
    if (event == nullptr)
    {
        return out;
    }
    out.coreType = static_cast<uint32_t>(event->coreType());
    out.name = event->name();
    out.delay = report.secondsDelay();
    if (event->is<rive::OpenUrlEvent>())
    {
        auto openUrl = event->as<rive::OpenUrlEvent>();
        out.hasUrl = true;
        out.url = openUrl->url();
        out.target = openUrlTargetName(openUrl->targetValue());
    }
    for (rive::Component* child : event->children())
    {
        rive_rust::golden::SideChannelEventProperty property;
        property.name = child->name();
        if (child->is<rive::CustomPropertyNumber>())
        {
            property.kind =
                rive_rust::golden::SideChannelEventProperty::Kind::number;
            property.numberValue =
                child->as<rive::CustomPropertyNumber>()->propertyValue();
        }
        else if (child->is<rive::CustomPropertyBoolean>())
        {
            property.kind =
                rive_rust::golden::SideChannelEventProperty::Kind::boolean;
            property.boolValue =
                child->as<rive::CustomPropertyBoolean>()->propertyValue();
        }
        else if (child->is<rive::CustomPropertyString>())
        {
            property.kind =
                rive_rust::golden::SideChannelEventProperty::Kind::string;
            property.stringValue =
                child->as<rive::CustomPropertyString>()->propertyValue();
        }
        else if (child->is<rive::CustomPropertyColor>())
        {
            property.kind =
                rive_rust::golden::SideChannelEventProperty::Kind::color;
            property.colorValue = static_cast<uint32_t>(
                child->as<rive::CustomPropertyColor>()->propertyValue());
        }
        else if (child->is<rive::CustomPropertyEnum>())
        {
            property.kind =
                rive_rust::golden::SideChannelEventProperty::Kind::uintValue;
            property.uintValue =
                child->as<rive::CustomPropertyEnum>()->propertyValue();
        }
        else if (child->is<rive::CustomPropertyTrigger>())
        {
            property.kind =
                rive_rust::golden::SideChannelEventProperty::Kind::uintValue;
            property.uintValue =
                child->as<rive::CustomPropertyTrigger>()->propertyValue();
        }
        else
        {
            continue;
        }
        out.properties.push_back(std::move(property));
    }
    return out;
}

// Emits the advance line (settled = !advanceAndApply return) plus one event
// line per event the state machine reported during that advance.
void recordAdvanceSideChannel(rive_rust::golden::RecordingFactory& factory,
                              rive::StateMachineInstance* stateMachine,
                              float targetSeconds,
                              bool keepGoing)
{
    if (stateMachine == nullptr)
    {
        factory.addAdvance(targetSeconds, !keepGoing);
        return;
    }
    factory.addAdvanceWithStates(targetSeconds,
                                 !keepGoing,
                                 stateMachine->stateChangedCount());
    const size_t reportedCount = stateMachine->reportedEventCount();
    for (size_t index = 0; index < reportedCount; index++)
    {
        factory.addSideChannelEvent(
            describeReportedEvent(stateMachine->reportedEventAt(index)));
    }
    auto* semanticManager = stateMachine->semanticManager();
    if (semanticManager != nullptr)
    {
        factory.addSemanticsDiff(semanticManager->drainDiff());
    }
}

void applySemanticInput(
    rive_rust::golden::RecordingFactory& factory,
    rive::StateMachineInstance* stateMachine,
    const InputEvent& event,
    bool recordSideChannel)
{
    auto* semanticManager =
        stateMachine == nullptr ? nullptr : stateMachine->semanticManager();
    if (event.kind == InputKind::semanticAction)
    {
        auto* node = semanticManager == nullptr
                         ? nullptr
                         : semanticManager->nodeById(event.semanticNodeId);
        const bool dispatched =
            node != nullptr && node->semanticData() != nullptr;
        if (dispatched)
        {
            stateMachine->fireSemanticAction(event.semanticNodeId,
                                             event.semanticAction);
        }
        if (recordSideChannel)
        {
            factory.addSemanticAction(event.seconds,
                                      event.semanticNodeId,
                                      semanticActionName(event.semanticAction),
                                      dispatched);
        }
        return;
    }

    const bool focused = semanticManager != nullptr &&
                         semanticManager->requestFocus(event.semanticNodeId);
    if (recordSideChannel)
    {
        factory.addSemanticFocus(event.seconds, event.semanticNodeId, focused);
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
                     options.animation,
                     false,
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
    resetCoverageProfileForFrameLoopIfRequested();
    resetFrameLoopAllocationCounterIfRequested();
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
    validateTraceOptions(options);
    if (options.file.empty())
    {
        throw CliError("missing --file <path>");
    }

    std::vector<InputEvent> inputEvents;
    if (!options.inputScript.empty())
    {
        inputEvents = loadInputScript(options.inputScript);
    }
    std::vector<ViewModelEvent> viewModelEvents;
    if (!options.viewModelScript.empty())
    {
        viewModelEvents = loadViewModelScript(options.viewModelScript);
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
        flushCoverageProfileIfRequested();
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
                     options.animation,
                     options.semanticDefaultViewModel,
                     factory);
    rive::Scene* scene = loader.scene();
    rive::StateMachineInstance* sceneStateMachine = loader.stateMachine();
    rive::ArtboardInstance* sceneArtboard = loader.artboard();
    rive::ViewModelInstance* sceneViewModel = loader.viewModelInstance();
    if (options.sideChannel && sceneStateMachine != nullptr)
    {
        sceneStateMachine->enableSemantics();
    }
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

    float currentSeconds = 0.0f;
    if (std::getenv("RIVE_GOLDEN_COVERAGE_STEADY_ONLY") != nullptr)
    {
        const bool keepGoing =
            advanceTo(scene, options.samples.front(), currentSeconds);
        if (options.sideChannel)
        {
            recordAdvanceSideChannel(recordingFactory,
                                     sceneStateMachine,
                                     options.samples.front(),
                                     keepGoing);
        }
        scene->draw(renderer.get());
    }
    resetCoverageProfileForFrameLoopIfRequested();
    resetFrameLoopAllocationCounterIfRequested();
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
    size_t nextInput = 0;
    size_t nextViewModel = 0;
    for (size_t repeat = 0; repeat < options.benchmarkRepeat; repeat++)
    {
        for (float sampleSeconds : options.samples)
        {
            while (true)
            {
                const bool inputDue =
                    nextInput < inputEvents.size() &&
                    inputEvents[nextInput].seconds <=
                        sampleSeconds + kTimeEpsilon;
                const bool viewModelDue =
                    nextViewModel < viewModelEvents.size() &&
                    viewModelEvents[nextViewModel].seconds <=
                        sampleSeconds + kTimeEpsilon;
                if (!inputDue && !viewModelDue)
                {
                    break;
                }
                const bool useInput =
                    inputDue &&
                    (!viewModelDue ||
                     inputEvents[nextInput].seconds <=
                         viewModelEvents[nextViewModel].seconds);
                const float eventSeconds =
                    useInput ? inputEvents[nextInput].seconds
                             : viewModelEvents[nextViewModel].seconds;
                timedStage(advanceElapsed, [&] {
                    const bool keepGoing =
                        advanceTo(scene, eventSeconds, currentSeconds);
                    if (options.sideChannel && !options.benchmark)
                    {
                        recordAdvanceSideChannel(recordingFactory,
                                                 sceneStateMachine,
                                                 eventSeconds,
                                                 keepGoing);
                    }
                });
                timedStage(inputElapsed, [&] {
                    if (!useInput)
                    {
                        applyViewModelEvent(recordingFactory,
                                            sceneViewModel,
                                            viewModelEvents[nextViewModel],
                                            !options.benchmark);
                        return;
                    }
                    const auto& event = inputEvents[nextInput];
                    if (event.kind == InputKind::semanticAction ||
                        event.kind == InputKind::semanticFocus)
                    {
                        applySemanticInput(recordingFactory,
                                           sceneStateMachine,
                                           event,
                                           options.sideChannel);
                        return;
                    }
                    if (event.kind == InputKind::setInput)
                    {
                        applySetInput(recordingFactory,
                                      sceneStateMachine,
                                      event,
                                      !options.benchmark);
                        return;
                    }
                    if (event.kind == InputKind::resize)
                    {
                        applyResize(recordingFactory,
                                    sceneArtboard,
                                    event,
                                    !options.benchmark);
                        return;
                    }
                    const rive::HitResult hitResult = applyInput(scene, event);
                    if (!options.benchmark)
                    {
                        recordingFactory.addInputEvent(
                            inputKindName(event.kind),
                            event.seconds,
                            event.x,
                            event.y,
                            event.pointerId);
                        if (options.sideChannel)
                        {
                            recordingFactory.addHitResult(
                                hitResultName(hitResult));
                        }
                    }
                });
                if (useInput)
                {
                    nextInput++;
                }
                else
                {
                    nextViewModel++;
                }
            }

            timedStage(advanceElapsed, [&] {
                const bool keepGoing =
                    advanceTo(scene, sampleSeconds, currentSeconds);
                if (options.sideChannel && !options.benchmark)
                {
                    recordAdvanceSideChannel(recordingFactory,
                                             sceneStateMachine,
                                             sampleSeconds,
                                             keepGoing);
                }
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
    const uint64_t frameLoopAllocations = stopFrameLoopAllocationCounter();
    if (std::getenv("RIVE_GOLDEN_ALLOCATION_COUNTER") != nullptr)
    {
        std::cerr << "frame_loop_allocations=" << frameLoopAllocations << "\n";
    }

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
    flushCoverageProfileIfRequested();
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
