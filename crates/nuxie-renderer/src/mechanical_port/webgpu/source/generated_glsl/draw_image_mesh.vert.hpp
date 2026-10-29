#pragma once

#include "draw_image_mesh.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_image_mesh_vert[] = R"===(#ifdef BB
d1(F3) K(0,c,PC);e1 d1(W3) K(1,c,QC);e1 d1(E1) K(ta,e,AC);K(ua,e,UB);K(va,e,PB);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,DC);K(za,uint,MC);K(Ha,e,IC);e1
#endif
n2 H0 W(0,c,e6);
#ifdef N
MB W(1,d,h4);
#endif
#if defined(AB)&&!defined(CB)
H0 W(2,e,V0);
#endif
MB W(3,i,Y1);
#ifdef H
i3 W(4,O,N1);
#endif
M1
#ifdef BB
x4 y4 i7(TB,F3,G3,W3,k3,E1,j0,C){L(C,G3,PC,c);L(C,k3,QC,c);L(r,j0,AC,e);L(r,j0,UB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,IC,e);V(e6,c);
#ifdef N
V(h4,d);
#endif
#if defined(AB)&&!defined(CB)
V(V0,e);
#endif
V(Y1,i);
#ifdef H
V(N1,O);
#endif
c i0=D0(p1(AC),PC)+PB.xy;e6=QC*IC.zw+IC.xy;
#ifdef N
if(N){h4=i9(CC,j.w6);}
#endif
#ifdef AB
if(AB){
#ifndef CB
V0=G8(p1(UB),PB.zw,i0 E4);
#else
qb(p1(UB),PB.zw,i0 E4);
#endif
}
#endif
e I=U3(i0);
#ifdef NC
I.y=-I.y;
#endif
#ifdef CB
I.z=j9(MC,0xffu);
#endif
Y1=unpackUnorm4x8(BC);
#ifdef H
N1=X1(DC);
#endif
a0(e6);
#ifdef N
a0(h4);
#endif
#if defined(AB)&&!defined(CB)
a0(V0);
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