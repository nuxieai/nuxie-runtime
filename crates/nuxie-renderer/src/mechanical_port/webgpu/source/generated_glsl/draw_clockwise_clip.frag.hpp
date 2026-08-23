#pragma once

#include "draw_clockwise_clip.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_clip_frag[] = R"===(#ifdef GB
I1
#ifndef Q
w0(Q2,j0);
#endif
j1(R2,h0);
#ifndef Q
Sa(d6,i4);
#endif
j1(G6,P0);J1 L1(JB){A(U1,E);c k1=-U1.x;
#ifdef EB
A(i1,c);c r0=i1;
#else
A(O,z2);c r0=O.x;
#endif
w2;E M0;c I5,r3;
#if defined(EB)&&defined(EC)
if(EC){r3=r0;}else
#endif
{M0=unpackHalf2x16(Y0(h0));I5=M0.y;c O4=I5==k1?M0.x:G0(.0);r3=O4+r0;}
#ifdef YC
c G5=U1.y;if(YC&&G5!=.0){c m4=.0;
#if defined(EB)&&defined(EC)
if(EC){M0=unpackHalf2x16(Y0(h0));I5=M0.y;}
#endif
if(I5!=k1){m4=I5==G5?M0.x:.0;c1(P0,packHalf2x16(A2(m4,Df)));}else{m4=unpackHalf2x16(Y0(P0)).x;d2(P0);}r3=min(r3,m4);}else
#endif
{d2(P0);}c1(h0,packHalf2x16(A2(r3,k1)));
#ifndef Q
v2(j0);
#endif
x2;Y1;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive