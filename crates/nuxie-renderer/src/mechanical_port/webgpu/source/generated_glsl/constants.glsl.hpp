#pragma once

#include "constants.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char constants[] = R"===(#define zf float(2048)
#define Cc 11
#define la float(512)
#define ic float(0.001953125)
#define ma float(3)
#define lc 0
#define mc 1
#define Dc 3u
#define Af (Dc+1u)
#define Bf float(1.0)
#define Ec 8
#define Fc 0xffu
#define hc 0x80000000u
#define jc 0x40000000u
#define X9 0x20000000u
#define hf (hc|jc|X9)
#define Gc (1u<<31u)
#define Cf (1u<<29u)
#define a4 (7u<<26u)
#define Df (5u<<26u)
#define Ef (4u<<26u)
#define w8 (2u<<26u)
#define x8 (1u<<26u)
#define y8 (1u<<25u)
#define Ff (1u<<24u)
#define G3 (1u<<23u)
#define na (1u<<22u)
#define Hc (1u<<21u)
#define z8 (1u<<20u)
#define Ic (1u<<19u)
#define Jc 0xffffu
#define Gf .0
#define A8 0
#define Kc 1
#define Lc 2
#define A8 0
#define Kc 1
#define Lc 2
#define Y7 0u
#define Nb 1u
#define M9 2u
#define Hf 3u
#define Qe 0x100u
#define K9 0x200u
#define Re 0x400u
#define If 0x800u
#define d3 0
#define a5 1
#define H4 0
#define Mc 1
#define Nc 2
#define Ib 3
#define Jb 4
#define Oc 5
#define oa 6
#define Jf 7
#define Pc 8
#define f7 9
#define Qc 10
#define W3 11
#define Kf 12
#define e6 13
#define Lf 13
#define N1(f) (3+f)
#define H3 2
#define Mf 3
#define S2 0
#define T2 1
#define f6 2
#define I6 3
#define Nf 2
#define r9 2
#define v9 3
#define w9 4
#define B9 5
#define Of 5
#define x9 5
#define y9 6
#define z9 7
#define A9 8
#define pf 1023u
#define o9 6.2e-5
#define P5 0u
#define we 1u
#define xe 2u
#define ye 3u
#define ze 4u
#define Ae 5u
#define Ce 6u
#define De 7u
#define Ee 8u
#define Fe 9u
#define Ge 10u
#define ve 11u
#define He 12u
#define Ie 13u
#define Je 14u
#define Ke 15u
#define I9 float(2048)
#define Kb float(0.00048828125)
#define J9 float(1<<16)
#define O9 (1u<<16)
#define S5 17u
#define e8 0x1ffffu
#define Pf float(1024)
#define pa float(0.0009765625)
#define qa 19u
#define k5 (1u<<(qa-1u))
#define ra ((1u<<qa)-1u)
#define g7 (1u<<qa)
#define Qf 0
#define Rf 1
#define Sf 2
#define Tf 3
#define Uf 4
#define Vf 5
#define Wf 6
#define Xf 7
#define Yf 8
#define Zf 9
#define ag 10
#define bg 11
#define cg 12
#define dg 13
#define eg 14
#define fg 15
#define Rc 65536.
#define sa 8.
#define ta 32u
#define g6 5u
#define A3 8u
#ifdef gg
#if gg>=201703
Ai(ta==1u<<g6);
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive