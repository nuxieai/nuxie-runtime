#pragma once

#include "draw_image_mesh.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_image_mesh_vert[] = R"===(#ifdef BB
e1(D3) K(0,c,PC);f1 e1(V3) K(1,c,QC);f1 e1(E1) K(sa,e,AC);K(ta,e,TB);K(ua,e,PB);K(va,uint,BC);K(wa,uint,CC);K(xa,uint,DC);K(ya,uint,MC);K(Ga,e,IC);f1
#endif
m2 F0 W(0,c,g6);
#ifdef N
MB W(1,d,h4);
#endif
#if defined(AB)&&!defined(CB)
F0 W(2,e,W0);
#endif
MB W(3,i,Y1);
#ifdef H
h3 W(4,P,N1);
#endif
M1
#ifdef BB
v4 w4 l7(SB,D3,E3,V3,j3,E1,j0,C){L(C,E3,PC,c);L(C,j3,QC,c);L(r,j0,AC,e);L(r,j0,TB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,IC,e);V(g6,c);
#ifdef N
V(h4,d);
#endif
#if defined(AB)&&!defined(CB)
V(W0,e);
#endif
V(Y1,i);
#ifdef H
V(N1,P);
#endif
c i0=C0(q1(AC),PC)+PB.xy;g6=QC*IC.zw+IC.xy;
#ifdef N
if(N){h4=j9(CC,j.z6);}
#endif
#ifdef AB
if(AB){
#ifndef CB
W0=G8(q1(TB),PB.zw,i0 C4);
#else
ob(q1(TB),PB.zw,i0 C4);
#endif
}
#endif
e I=T3(i0);
#ifdef NC
I.y=-I.y;
#endif
#ifdef CB
I.z=k9(MC,0xffu);
#endif
Y1=unpackUnorm4x8(BC);
#ifdef H
N1=X1(DC);
#endif
a0(g6);
#ifdef N
a0(h4);
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