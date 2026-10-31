#pragma once

#include "draw_image_mesh.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_image_mesh_vert[] = R"===(#ifdef BB
f1(H3) K(0,c,QC);g1 f1(W3) K(1,c,RC);g1 f1(G1) K(wa,e,AC);K(xa,e,VB);K(ya,e,QB);K(za,uint,BC);K(Aa,uint,CC);K(Ba,uint,DC);K(Ca,uint,NC);K(Ka,e,IC);g1
#endif
n2 I0 W(0,c,h6);
#ifdef N
NB W(1,d,j4);
#endif
#if defined(AB)&&!defined(CB)
I0 W(2,e,Y0);
#endif
NB W(3,i,a2);
#ifdef H
m3 W(4,O,Q1);
#endif
P1
#ifdef BB
A4 B4 k7(UB,H3,I3,W3,o3,G1,j0,C){L(C,I3,QC,c);L(C,o3,RC,c);L(r,j0,AC,e);L(r,j0,VB,e);L(r,j0,QB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,NC,uint);L(r,j0,IC,e);V(h6,c);
#ifdef N
V(j4,d);
#endif
#if defined(AB)&&!defined(CB)
V(Y0,e);
#endif
V(a2,i);
#ifdef H
V(Q1,O);
#endif
c i0=F0(v1(AC),QC)+QB.xy;h6=RC*IC.zw+IC.xy;
#ifdef N
if(N){j4=n9(CC,j.z6);}
#endif
#ifdef AB
if(AB){
#ifndef CB
Y0=L8(v1(VB),QB.zw,i0 H4);
#else
tb(v1(VB),QB.zw,i0 H4);
#endif
}
#endif
e I=U3(i0);
#ifdef OC
I.y=-I.y;
#endif
#ifdef CB
I.z=o9(NC,0xffu);
#endif
a2=unpackUnorm4x8(BC);
#ifdef H
Q1=Z1(DC);
#endif
e0(h6);
#ifdef N
e0(j4);
#endif
#if defined(AB)&&!defined(CB)
e0(Y0);
#endif
e0(a2);
#ifdef H
e0(Q1);
#endif
B1(I);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive