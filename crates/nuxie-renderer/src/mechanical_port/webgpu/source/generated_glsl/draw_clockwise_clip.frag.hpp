#pragma once

#include "draw_clockwise_clip.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_clip_frag[] = R"===(#ifdef EB
F1
#ifndef R
y0(x2,n0);
#endif
r1(k3,m0);
#ifndef R
gc(E6,K4);
#endif
r1(m7,Y0);G1 I1(HB){q(k2,E);d B1=-k2.x;
#ifdef DB
q(p1,d);d B0=p1;
#else
q(T,R2);d B0=T.x;
#endif
P2;E X0;d j6,R3;
#if defined(DB)&&defined(FC)
if(FC){R3=B0;}else
#endif
{X0=unpackHalf2x16(k1(m0));j6=X0.y;d q5=j6==B1?X0.x:J0(.0);R3=q5+B0;}
#ifdef ED
d h6=k2.y;if(ED&&h6!=.0){d O4=.0;
#if defined(DB)&&defined(FC)
if(FC){X0=unpackHalf2x16(k1(m0));j6=X0.y;}
#endif
if(j6!=B1){O4=j6==h6?X0.x:.0;m1(Y0,packHalf2x16(S2(O4,xh)));}else{O4=unpackHalf2x16(k1(Y0)).x;i2(Y0);}R3=min(R3,O4);}else
#endif
{i2(Y0);}m1(m0,packHalf2x16(S2(R3,B1)));
#ifndef R
O2(n0);
#endif
Q2;O1;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive