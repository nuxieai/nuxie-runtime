#pragma once

#include "flush_uniforms.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char flush_uniforms[] = R"===(#ifndef Z2
#define Z2(D4) float D4;
#endif
#ifndef d4
#define d4(D4) uint D4;
#endif
#ifndef Ae
#define Ae(D4) x6 D4;
#endif
#ifndef Vb
#define Vb(D4) c D4;
#endif
#ifndef Bi
#define Bi(D4) f D4;
#endif
#ifndef Be
#define Be VB
#endif
c8(e5,Be) Z2(wd) Z2(Ce) Z2(Hg) Z2(Ig) d4(P6) d4(Ca) d4(tg) d4(ug) Ae(E8) Vb(yi) Vb(De) d4(q2) Z2(Ci) d4(w6) Z2(h3) Z2(Ee) d4(mg) Z2(F3) Z2(G3) Z2(Fe) d4(vi) d4(Ba) Z2(L8) Z2(M8) L9(j)
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive