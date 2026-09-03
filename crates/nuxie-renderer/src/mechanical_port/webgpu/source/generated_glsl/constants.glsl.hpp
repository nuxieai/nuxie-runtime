#pragma once

#include "constants.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char constants[] = R"===(#define Af float(2048)
#define Dc 11
#define la 16u
#define ma float(512)
#define jc float(0.001953125)
#define na float(3)
#define mc 0
#define nc 1
#define Ec 3u
#define Bf (Ec+1u)
#define Cf float(1.0)
#define Fc 8
#define Gc 0xffu
#define ic 0x80000000u
#define kc 0x40000000u
#define X9 0x20000000u
#define jf (ic|kc|X9)
#define Hc (1u<<31u)
#define Df (1u<<29u)
#define a4 (7u<<26u)
#define Ef (5u<<26u)
#define Ff (4u<<26u)
#define x8 (2u<<26u)
#define y8 (1u<<26u)
#define z8 (1u<<25u)
#define Gf (1u<<24u)
#define G3 (1u<<23u)
#define oa (1u<<22u)
#define Ic (1u<<21u)
#define A8 (1u<<20u)
#define Jc (1u<<19u)
#define Kc 0xffffu
#define Hf .0
#define B8 0
#define Lc 1
#define Mc 2
#define B8 0
#define Lc 1
#define Mc 2
#define Z7 0u
#define Ob 1u
#define M9 2u
#define If 3u
#define Re 0x100u
#define K9 0x200u
#define Se 0x400u
#define Jf 0x800u
#define d3 0
#define c5 1
#define H4 0
#define Nc 1
#define Oc 2
#define Jb 3
#define Kb 4
#define Pc 5
#define pa 6
#define Kf 7
#define Qc 8
#define g7 9
#define Rc 10
#define W3 11
#define Lf 12
#define f6 13
#define Mf 13
#define N1(f) (3+f)
#define H3 2
#define Nf 3
#define S2 0
#define T2 1
#define g6 2
#define J6 3
#define Of 2
#define r9 2
#define v9 3
#define w9 4
#define B9 5
#define Pf 5
#define x9 5
#define y9 6
#define z9 7
#define A9 8
#define qf 1023u
#define o9 6.2e-5
#define Q5 0u
#define xe 1u
#define ye 2u
#define ze 3u
#define Ae 4u
#define Be 5u
#define De 6u
#define Ee 7u
#define Fe 8u
#define Ge 9u
#define He 10u
#define we 11u
#define Ie 12u
#define Je 13u
#define Ke 14u
#define Le 15u
#define I9 float(2048)
#define Lb float(0.00048828125)
#define J9 float(1<<16)
#define O9 (1u<<16)
#define T5 17u
#define f8 0x1ffffu
#define Qf float(1024)
#define qa float(0.0009765625)
#define ra 19u
#define l5 (1u<<(ra-1u))
#define sa ((1u<<ra)-1u)
#define h7 (1u<<ra)
#define Rf 0
#define Sf 1
#define Tf 2
#define Uf 3
#define Vf 4
#define Wf 5
#define Xf 6
#define Yf 7
#define Zf 8
#define ag 9
#define bg 10
#define cg 11
#define dg 12
#define eg 13
#define fg 14
#define gg 15
#define Sc 65536.
#define ta 8.
#define ua 32u
#define h6 5u
#define A3 8u
#ifdef hg
#if hg>=201703
Ci(ua==1u<<h6);
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive