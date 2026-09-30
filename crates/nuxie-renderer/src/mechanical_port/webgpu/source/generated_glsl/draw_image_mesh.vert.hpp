#pragma once

#include "draw_image_mesh.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_image_mesh_vert[] = R"===(#ifdef DB
f1(j3)J(0,c,PC);g1 f1(z3)J(1,c,QC);g1 f1(m1)J(w9,f,WB);J(x9,f,SB);J(y9,f,NB);J(z9,uint,XB);J(A9,uint,YB);J(B9,uint,ZB);J(C9,uint,MC);g1
#endif
p2 H0 V(0,c,H5);
#ifdef I
MB V(1,d,L3);
#endif
#if defined(BB)&&!defined(CB)
H0 V(2,f,M0);
#endif
MB V(3,i,H1);
#ifdef AB
T2 V(4,L,A1);
#endif
h2
#ifdef DB
V3 W3 K6(FC,j3,k3,z3,A3,m1,g0,B){K(B,k3,PC,c);K(B,A3,QC,c);K(v,g0,WB,f);K(v,g0,SB,f);K(v,g0,NB,f);K(v,g0,XB,uint);K(v,g0,YB,uint);K(v,g0,ZB,uint);K(v,g0,MC,uint);T(H5,c);
#ifdef I
T(L3,d);
#endif
#if defined(BB)&&!defined(CB)
T(M0,f);
#endif
T(H1,i);
#ifdef AB
T(A1,L);
#endif
c j0=N0(I1(WB),PC)+NB.xy;H5=QC;
#ifdef I
if(I){L3=v8(YB,l.f6);}
#endif
#ifdef BB
if(BB){
#ifndef CB
M0=U7(I1(SB),NB.zw,j0 y5);
#else
Lc(I1(SB),NB.zw,j0 y5);
#endif
}
#endif
f W=N3(j0);
#ifdef SC
W.y=-W.y;
#endif
#ifdef CB
W.z=na(MC);
#endif
H1=unpackUnorm4x8(XB);
#ifdef AB
A1=Y1(ZB);
#endif
a0(H5);
#ifdef I
a0(L3);
#endif
#if defined(BB)&&!defined(CB)
a0(M0);
#endif
a0(H1);
#ifdef AB
a0(A1);
#endif
z1(W);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive