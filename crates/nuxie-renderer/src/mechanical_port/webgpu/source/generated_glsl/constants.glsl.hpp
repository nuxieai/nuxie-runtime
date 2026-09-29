#pragma once

#include "constants.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char constants[] = R"===(#define Ef float(2048)
#define Ec 11
#define la 16u
#define ma float(512)
#define kc float(0.001953125)
#define na float(3)
#define nc 0
#define oc 1
#define Fc 3u
#define Ff (Fc+1u)
#define Gf float(1.0)
#define Gc 8
#define Hc 0xffu
#define jc 0x80000000u
#define lc 0x40000000u
#define X9 0x20000000u
#define nf (jc|lc|X9)
#define Ic (1u<<31u)
#define Hf (1u<<29u)
#define a4 (7u<<26u)
#define If (5u<<26u)
#define Jf (4u<<26u)
#define x8 (2u<<26u)
#define y8 (1u<<26u)
#define z8 (1u<<25u)
#define Kf (1u<<24u)
#define G3 (1u<<23u)
#define oa (1u<<22u)
#define Jc (1u<<21u)
#define A8 (1u<<20u)
#define Kc (1u<<19u)
#define Lc 0xffffu
#define Lf .0
#define B8 0
#define Mc 1
#define Nc 2
#define B8 0
#define Mc 1
#define Nc 2
#define Z7 0u
#define Pb 1u
#define M9 2u
#define Mf 3u
#define Ve 0x100u
#define K9 0x200u
#define We 0x400u
#define Nf 0x800u
#define d3 0
#define d5 1
#define I4 0
#define Oc 1
#define Pc 2
#define Kb 3
#define Lb 4
#define Qc 5
#define pa 6
#define Of 7
#define Rc 8
#define h7 9
#define Sc 10
#define W3 11
#define Pf 12
#define g6 13
#define Qf 13
#define O1(f) (3+f)
#define H3 2
#define Rf 3
#define S2 0
#define T2 1
#define h6 2
#define K6 3
#define Sf 2
#define r9 2
#define v9 3
#define w9 4
#define B9 5
#define Tf 5
#define x9 5
#define y9 6
#define z9 7
#define A9 8
#define uf 1023u
#define o9 6.2e-5
#define R5 0u
#define Be 1u
#define Ce 2u
#define De 3u
#define Ee 4u
#define Fe 5u
#define He 6u
#define Ie 7u
#define Je 8u
#define Ke 9u
#define Le 10u
#define Ae 11u
#define Me 12u
#define Ne 13u
#define Oe 14u
#define Pe 15u
#define I9 float(2048)
#define Mb float(0.00048828125)
#define J9 float(1<<16)
#define O9 (1u<<16)
#define U5 17u
#define f8 0x1ffffu
#define Uf float(1024)
#define qa float(0.0009765625)
#define ra 19u
#define m5 (1u<<(ra-1u))
#define sa ((1u<<ra)-1u)
#define i7 (1u<<ra)
#define Vf 0
#define Wf 1
#define Xf 2
#define Yf 3
#define Zf 4
#define ag 5
#define bg 6
#define cg 7
#define dg 8
#define eg 9
#define fg 10
#define gg 11
#define hg 12
#define ig 13
#define jg 14
#define kg 15
#define Tc 65536.
#define ta 8.
#define ua 32u
#define i6 5u
#define A3 8u
#ifdef lg
#if lg>=201703
Hi(ua==1u<<i6);
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive