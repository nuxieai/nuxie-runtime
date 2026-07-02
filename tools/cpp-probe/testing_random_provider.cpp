#include <cstdlib>
#include <cstddef>
#include <vector>

namespace rive
{
class RandomProvider
{
public:
    static float generateRandomFloat();
};
} // namespace rive

namespace rive_probe
{
namespace
{
bool g_countedRandomProviderEnabled = false;
std::size_t g_randomCallsCount = 0;
std::vector<float> g_randomResults;
} // namespace

void clearRandomProvider()
{
    g_countedRandomProviderEnabled = true;
    g_randomCallsCount = 0;
    g_randomResults.clear();
}

void addRandomProviderValue(float value)
{
    g_countedRandomProviderEnabled = true;
    g_randomResults.push_back(value);
}

std::size_t randomProviderTotalCalls() { return g_randomCallsCount; }
} // namespace rive_probe

float rive::RandomProvider::generateRandomFloat()
{
    if (!rive_probe::g_countedRandomProviderEnabled)
    {
        return std::rand() / static_cast<float>(RAND_MAX);
    }

    ++rive_probe::g_randomCallsCount;
    if (rive_probe::g_randomResults.empty())
    {
        return 0.0f;
    }

    float value = rive_probe::g_randomResults.front();
    rive_probe::g_randomResults.erase(rive_probe::g_randomResults.begin());
    return value;
}
