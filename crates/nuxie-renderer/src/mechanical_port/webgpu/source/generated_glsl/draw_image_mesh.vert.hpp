#pragma once

#include "draw_image_mesh.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_image_mesh_vert[] = R"===(#ifdef DB
g1(i3)L(0,d,PC);h1 g1(y3)L(1,d,QC);h1 g1(n1)L(r9,g,XB);L(v9,g,TB);L(w9,g,OB);
#ifdef O3
L(x9,uint,YB);L(y9,uint,ZB);L(z9,uint,AC);L(A9,uint,BC);
#else
L(B9,G,IB);
#endif
h1
#endif
m2 H0 X(0,d,H5);
#ifdef I
NB X(1,c,K3);
#endif
#if defined(BB)&&!defined(CB)
H0 X(2,g,M0);
#endif
NB X(3,c,I1);
#ifdef AB
Q2 X(4,N,B1);
#endif
g2
#ifdef DB
U3 V3 J6(HC,i3,j3,y3,z3,n1,g0,B){M(B,j3,PC,d);M(B,z3,QC,d);M(v,g0,XB,g);M(v,g0,TB,g);M(v,g0,OB,g);
#ifdef O3
M(v,g0,YB,uint);M(v,g0,ZB,uint);M(v,g0,AC,uint);M(v,g0,BC,uint);G IB=G(YB,ZB,AC,BC);
#else
M(v,g0,IB,G);
#endif
V(H5,d);
#ifdef I
V(K3,c);
#endif
#if defined(BB)&&!defined(CB)
V(M0,g);
#endif
V(I1,c);
#ifdef AB
V(B1,N);
#endif
d m0=R0(h2(XB),PC)+OB.xy;H5=QC;
#ifdef I
if(I){K3=r8(IB.y,m.e6);}
#endif
#ifdef BB
if(BB){
#ifndef CB
M0=T7(h2(TB),OB.zw,m0 y5);
#else
Cc(h2(TB),OB.zw,m0 y5);
#endif
}
#endif
g W=M3(m0);
#ifdef SC
W.y=-W.y;
#endif
#ifdef CB
W.z=ja(IB.w);
#endif
I1=uintBitsToFloat(IB.x);
#ifdef AB
B1=X1(IB.z);
#endif
c0(H5);
#ifdef I
c0(K3);
#endif
#if defined(BB)&&!defined(CB)
c0(M0);
#endif
c0(I1);
#ifdef AB
c0(B1);
#endif
A1(W);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive