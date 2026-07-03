#include "recording_renderer.hpp"

#include "rive/math/raw_path.hpp"

#include <iostream>
#include <string>

int main(int argc, char** argv)
{
    if (argc > 1 && std::string(argv[1]) != "--smoke")
    {
        std::cerr << "golden-runner CLI is not implemented yet; use --smoke\n";
        return 2;
    }

    rive_rust::golden::RecordingFactory factory;
    auto renderer = factory.makeRenderer();
    auto path = factory.makeEmptyRenderPath();
    auto paint = factory.makeRenderPaint();

    path->moveTo(0.0f, 0.0f);
    path->lineTo(10.0f, 0.0f);
    path->lineTo(10.0f, 10.0f);
    path->close();
    paint->color(0xff336699);

    factory.frameSize(64, 64);
    renderer->save();
    renderer->drawPath(path.get(), paint.get());
    renderer->restore();
    factory.addFrame();

    std::cout << factory.stream();
    return 0;
}
