#include "rive/file.hpp"
#include "rive/artboard.hpp"
#include "rive/animation/nested_state_machine.hpp"
#include "rive/animation/state_machine_instance.hpp"
#include "rive/animation/state_machine_input_instance.hpp"
#include "utils/no_op_factory.hpp"
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>
#include <vector>

int main(int argc, char** argv)
{
    rive::NoOpFactory factory;
    bool all = argc > 1;
    for (int i = 1; i < argc; ++i)
    {
        const std::string path = argv[i];
        std::ifstream input(path, std::ios::binary);
        std::vector<uint8_t> bytes((std::istreambuf_iterator<char>(input)), {});
        rive::ImportResult result;
        auto file = rive::File::import(
            rive::Span<const uint8_t>(bytes.data(), bytes.size()), &factory, &result);
        if (!file)
        {
            std::cerr << "import failed: " << path << "\n";
            return 2;
        }
        auto artboard = file->artboardAt(0);
        auto machine = artboard->stateMachineAt(0);
        machine->advanceAndApply(0);
        auto observed = machine.get();
        if (path.find("nested-disabled") != std::string::npos)
        {
            const auto nested = artboard->find<rive::NestedStateMachine>();
            if (nested.size() != 1 || !nested[0]->stateMachineInstance())
            {
                std::cerr << "missing nested machine: " << path << "\n";
                return 3;
            }
            observed = nested[0]->stateMachineInstance();
        }
        auto handled = observed->getBool("handled");
        if (!handled)
        {
            std::cerr << "missing handled: " << path << "\n";
            return 4;
        }
        const bool before = handled->value();
        machine->pointerDown(rive::Vec2D(50, 50), 3932, rive::PointerButton::primary);
        machine->advanceAndApply(0);
        const bool down = handled->value();
        machine->pointerUp(rive::Vec2D(50, 50), 3932, rive::PointerButton::primary);
        machine->advanceAndApply(0);
        const bool up = handled->value();
        std::cout << "{\"file\":\"" << path << "\",\"before\":"
                  << (before ? "true" : "false") << ",\"afterDown\":"
                  << (down ? "true" : "false") << ",\"afterUp\":"
                  << (up ? "true" : "false") << "}\n";
        const bool expectedDown = path.find("down-disabled") != std::string::npos;
        all = all && !before && down == expectedDown && up;
    }
    return all ? 0 : 1;
}
