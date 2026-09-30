#pragma once

#include "draw_clockwise_clip.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_clip_frag[] = R"===(#ifdef EB
R1
#ifndef V
B0(L2,n0);
#endif
o1(d3,m0);
#ifndef V
tb(p6,C4);
#endif
o1(V6,W0);S1 T1(IB){q(l1,C);d X0=-l1.x;
#ifdef DB
q(m1,d);d A0=m1;
#else
q(S,H2);d A0=S.x;
#endif
F2;C U0;d Y5,G3;
#if defined(DB)&&defined(FC)
if(FC){G3=A0;}else
#endif
{U0=unpackHalf2x16(h1(m0));Y5=U0.y;d g5=Y5==X0?U0.x:I0(.0);G3=g5+A0;}
#ifdef BD
d F4=l1.y;if(BD&&F4!=.0){d H4=.0;
#if defined(DB)&&defined(FC)
if(FC){U0=unpackHalf2x16(h1(m0));Y5=U0.y;}
#endif
if(Y5!=X0){H4=Y5==F4?U0.x:.0;j1(W0,packHalf2x16(I2(H4,Ig)));}else{H4=unpackHalf2x16(h1(W0)).x;Z1(W0);}G3=min(G3,H4);}else
#endif
{Z1(W0);}j1(m0,packHalf2x16(I2(G3,X0)));
#ifndef V
E2(n0);
#endif
G2;h2;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive