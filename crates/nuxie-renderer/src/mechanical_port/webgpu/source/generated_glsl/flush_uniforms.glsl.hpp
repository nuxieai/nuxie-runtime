#pragma once

#include "flush_uniforms.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char flush_uniforms[] = R"===(#ifndef Z2
#define Z2(I4) float I4;
#endif
#ifndef g4
#define g4(I4) uint I4;
#endif
#ifndef Ne
#define Ne(I4) B6 I4;
#endif
#ifndef ec
#define ec(I4) c I4;
#endif
#ifndef Ni
#define Ni(I4) e I4;
#endif
#ifndef Oe
#define Oe WB
#endif
h8(i5,Oe) Z2(Fd) Z2(Pe) Z2(Sg) Z2(Tg) g4(T6) g4(Fa) g4(Eg) g4(Fg) Ne(J8) ec(Ki) ec(Qe) g4(r2) Z2(Oi) g4(z6) Z2(i3) Z2(Re) g4(xg) Z2(H3) Z2(I3) Z2(Se) g4(Hi) g4(Ea) Z2(Q8) Z2(R8) P9(j)
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive