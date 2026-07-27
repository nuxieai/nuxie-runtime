#include <algorithm>
#include <array>
#include <iostream>
#include <string_view>

namespace
{
constexpr std::string_view kPinnedRiveRef =
    "d788e8ec6e8b598526607d6a1e8818e8b637b60c";

constexpr std::array<std::string_view, 6> kRegisteredFixtures = {
    "fed.rt_ed_007.nested_transition_duration",
    "fed.loc_001.extended_view_model_owner",
    "fed.loc_002.current_source_relation",
    "fed.loc_005.shared_view_model_owner",
    "fed.loc_007.parametric_path_dirt",
    "fed.loc_008.text_measurement_facade",
};

void usage(std::string_view executable)
{
    std::cerr << "usage: " << executable << " --list | --fixture ID\n";
}
} // namespace

int main(int argc, char** argv)
{
    if (argc == 2 && std::string_view(argv[1]) == "--list")
    {
        for (const auto fixture : kRegisteredFixtures)
        {
            std::cout << fixture << '\n';
        }
        return 0;
    }

    if (argc == 3 && std::string_view(argv[1]) == "--fixture")
    {
        const std::string_view requested = argv[2];
        const auto found = std::find(kRegisteredFixtures.begin(),
                                     kRegisteredFixtures.end(),
                                     requested);
        if (found == kRegisteredFixtures.end())
        {
            std::cerr << "unregistered fixture: " << requested << '\n';
            return 2;
        }
        std::cout << "fixture=" << requested << " status=registered"
                  << " upstream_ref=" << kPinnedRiveRef << '\n';
        return 0;
    }

    usage(argv[0]);
    return 64;
}
