#pragma once

#include "draw_image_mesh.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_image_mesh_vert[] = R"===(#ifdef BB
e1(E3) K(0,c,PC);f1 e1(W3) K(1,c,QC);f1 e1(E1) K(ta,e,AC);K(ua,e,UB);K(va,e,PB);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,DC);K(za,uint,MC);K(Ha,e,IC);f1
#endif
m2 H0 W(0,c,g6);
#ifdef N
MB W(1,d,i4);
#endif
#if defined(AB)&&!defined(CB)
H0 W(2,e,W0);
#endif
MB W(3,i,Y1);
#ifdef H
h3 W(4,P,N1);
#endif
M1
#ifdef BB
x4 y4 k7(TB,E3,F3,W3,j3,E1,j0,C){L(C,F3,PC,c);L(C,j3,QC,c);L(r,j0,AC,e);L(r,j0,UB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,IC,e);V(g6,c);
#ifdef N
V(i4,d);
#endif
#if defined(AB)&&!defined(CB)
V(W0,e);
#endif
V(Y1,i);
#ifdef H
V(N1,P);
#endif
c i0=D0(q1(AC),PC)+PB.xy;g6=QC*IC.zw+IC.xy;
#ifdef N
if(N){i4=k9(CC,j.z6);}
#endif
#ifdef AB
if(AB){
#ifndef CB
W0=I8(q1(UB),PB.zw,i0 E4);
#else
pb(q1(UB),PB.zw,i0 E4);
#endif
}
#endif
e I=U3(i0);
#ifdef NC
I.y=-I.y;
#endif
#ifdef CB
I.z=l9(MC,0xffu);
#endif
Y1=unpackUnorm4x8(BC);
#ifdef H
N1=X1(DC);
#endif
a0(g6);
#ifdef N
a0(i4);
#endif
#if defined(AB)&&!defined(CB)
a0(W0);
#endif
a0(Y1);
#ifdef H
a0(N1);
#endif
A1(I);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive