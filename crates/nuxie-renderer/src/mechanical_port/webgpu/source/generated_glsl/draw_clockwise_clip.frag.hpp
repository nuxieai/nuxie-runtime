#pragma once

#include "draw_clockwise_clip.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_clip_frag[] = R"===(#ifdef EB
Q1
#ifndef V
A0(L2,o0);
#endif
o1(d3,m0);
#ifndef V
sb(n6,B4);
#endif
o1(T6,V0);R1 T1(IB){q(l1,C);d X0=-l1.x;
#ifdef DB
q(m1,d);d z0=m1;
#else
q(S,H2);d z0=S.x;
#endif
F2;C T0;d V5,G3;
#if defined(DB)&&defined(EC)
if(EC){G3=z0;}else
#endif
{T0=unpackHalf2x16(h1(m0));V5=T0.y;d f5=V5==X0?T0.x:M0(.0);G3=f5+z0;}
#ifdef BD
d E4=l1.y;if(BD&&E4!=.0){d G4=.0;
#if defined(DB)&&defined(EC)
if(EC){T0=unpackHalf2x16(h1(m0));V5=T0.y;}
#endif
if(V5!=X0){G4=V5==E4?T0.x:.0;j1(V0,packHalf2x16(I2(G4,Eg)));}else{G4=unpackHalf2x16(h1(V0)).x;k2(V0);}G3=min(G3,G4);}else
#endif
{k2(V0);}j1(m0,packHalf2x16(I2(G3,X0)));
#ifndef V
E2(o0);
#endif
G2;g2;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive