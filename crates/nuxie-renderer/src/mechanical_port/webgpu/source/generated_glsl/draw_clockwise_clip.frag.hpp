#pragma once

#include "draw_clockwise_clip.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_clip_frag[] = R"===(#ifdef FB
J1
#ifndef O
y0(F2,k0);
#endif
i1(U2,h0);
#ifndef O
ab(g6,m4);
#endif
i1(K6,Q0);K1 M1(IB){r(W1,E);d j1=-W1.x;
#ifdef EB
r(h1,d);d x0=h1;
#else
r(M,A2);d x0=M.x;
#endif
y2;E O0;d J5,w3;
#if defined(EB)&&defined(DC)
if(DC){w3=x0;}else
#endif
{O0=unpackHalf2x16(Y0(h0));J5=O0.y;d S4=J5==j1?O0.x:I0(.0);w3=S4+x0;}
#ifdef ZC
d H5=W1.y;if(ZC&&H5!=.0){d q4=.0;
#if defined(EB)&&defined(DC)
if(DC){O0=unpackHalf2x16(Y0(h0));J5=O0.y;}
#endif
if(J5!=j1){q4=J5==H5?O0.x:.0;c1(Q0,packHalf2x16(C2(q4,dg)));}else{q4=unpackHalf2x16(Y0(Q0)).x;f2(Q0);}w3=min(w3,q4);}else
#endif
{f2(Q0);}c1(h0,packHalf2x16(C2(w3,j1)));
#ifndef O
x2(k0);
#endif
z2;a2;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive