#pragma once

#include "draw_image_mesh.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_image_mesh_vert[] = R"===(#ifdef CB
h1(n3) I(0,c,PC);i1 h1(C3) I(1,c,QC);i1 h1(p1) I(v9,f,XB);I(w9,f,RB);I(x9,f,NB);I(y9,uint,YB);I(z9,uint,ZB);I(A9,uint,AC);I(B9,uint,MC);I(G9,f,GC);i1
#endif
q2 I0 W(0,c,J5);
#ifdef K
MB W(1,d,O3);
#endif
#if defined(AB)&&!defined(BB)
I0 W(2,f,O0);
#endif
MB W(3,i,K1);
#ifdef T
V2 W(4,N,D1);
#endif
i2
#ifdef CB
Y3 Z3 I6(EC,n3,o3,C3,D3,p1,g0,A){J(A,o3,PC,c);J(A,D3,QC,c);J(q,g0,XB,f);J(q,g0,RB,f);J(q,g0,NB,f);J(q,g0,YB,uint);J(q,g0,ZB,uint);J(q,g0,AC,uint);J(q,g0,MC,uint);J(q,g0,GC,f);V(J5,c);
#ifdef K
V(O3,d);
#endif
#if defined(AB)&&!defined(BB)
V(O0,f);
#endif
V(K1,i);
#ifdef T
V(D1,N);
#endif
c l0=P0(L1(XB),PC)+NB.xy;J5=QC*GC.zw+GC.xy;
#ifdef K
if(K){O3=r8(ZB,j.c6);}
#endif
#ifdef AB
if(AB){
#ifndef BB
O0=T7(L1(RB),NB.zw,l0 A5);
#else
Nc(L1(RB),NB.zw,l0 A5);
#endif
}
#endif
f X=Q3(l0);
#ifdef SC
X.y=-X.y;
#endif
#ifdef BB
X.z=na(MC,0xffu);
#endif
K1=unpackUnorm4x8(YB);
#ifdef T
D1=a2(AC);
#endif
c0(J5);
#ifdef K
c0(O3);
#endif
#if defined(AB)&&!defined(BB)
c0(O0);
#endif
c0(K1);
#ifdef T
c0(D1);
#endif
C1(X);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive