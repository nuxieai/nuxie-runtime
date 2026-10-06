#pragma once

#include "flush_uniforms.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char flush_uniforms[] = R"===(#ifndef T2
#define T2(y4) float y4;
#endif
#ifndef X3
#define X3(y4) uint y4;
#endif
#ifndef Sd
#define Sd(y4) m6 y4;
#endif
#ifndef kb
#define kb(y4) c y4;
#endif
#ifndef Qh
#define Qh(y4) e y4;
#endif
#ifndef Td
#define Td UB
#endif
G7(V4,Td) T2(Rc) T2(Ud) T2(dg) T2(eg) X3(A6) X3(Y9) X3(Pf) X3(Qf) Sd(i8) kb(Nh) kb(Vd) X3(j2) T2(Rh) X3(U4) T2(a3) T2(Wd) X3(Jf) T2(M3) T2(N3) T2(Xd) X3(Kh) X3(X9) T2(xc) T2(yc) e9(j)
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive