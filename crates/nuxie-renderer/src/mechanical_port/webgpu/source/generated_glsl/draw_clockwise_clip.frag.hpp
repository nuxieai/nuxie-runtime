#pragma once

#include "draw_clockwise_clip.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_clip_frag[] = R"===(#ifdef EB
V1
#ifndef U
C0(U2,n0);
#endif
q1(i3,m0);
#ifndef U
ac(w6,G4);
#endif
q1(d7,Z0);W1 Y1(IB){q(j2,D);d z1=-j2.x;
#ifdef DB
q(o1,d);d B0=o1;
#else
q(S,Q2);d B0=S.x;
#endif
O2;D X0;d d6,O3;
#if defined(DB)&&defined(EC)
if(EC){O3=B0;}else
#endif
{X0=unpackHalf2x16(l1(m0));d6=X0.y;d m5=d6==z1?X0.x:I0(.0);O3=m5+B0;}
#ifdef CD
d a6=j2.y;if(CD&&a6!=.0){d K4=.0;
#if defined(DB)&&defined(EC)
if(EC){X0=unpackHalf2x16(l1(m0));d6=X0.y;}
#endif
if(d6!=z1){K4=d6==a6?X0.x:.0;m1(Z0,packHalf2x16(R2(K4,nh)));}else{K4=unpackHalf2x16(l1(Z0)).x;h2(Z0);}O3=min(O3,K4);}else
#endif
{h2(Z0);}m1(m0,packHalf2x16(R2(O3,z1)));
#ifndef U
N2(n0);
#endif
P2;p2;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive