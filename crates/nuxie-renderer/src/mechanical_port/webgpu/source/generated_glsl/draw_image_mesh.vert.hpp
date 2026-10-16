#pragma once

#include "draw_image_mesh.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_image_mesh_vert[] = R"===(#ifdef BB
e1(D3) K(0,c,PC);f1 e1(T3) K(1,c,QC);f1 e1(D1) K(sa,f,ZB);K(ta,f,SB);K(ua,f,PB);K(va,uint,AC);K(wa,uint,BC);K(xa,uint,CC);K(ya,uint,MC);K(Ga,f,HC);f1
#endif
v2 F0 W(0,c,d6);
#ifdef N
MB W(1,d,f4);
#endif
#if defined(AB)&&!defined(CB)
F0 W(2,f,V0);
#endif
MB W(3,i,V1);
#ifdef H
h3 W(4,P,L1);
#endif
k2
#ifdef BB
q4 r4 k7(RB,D3,E3,T3,j3,D1,j0,F){L(F,E3,PC,c);L(F,j3,QC,c);L(r,j0,ZB,f);L(r,j0,SB,f);L(r,j0,PB,f);L(r,j0,AC,uint);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,MC,uint);L(r,j0,HC,f);V(d6,c);
#ifdef N
V(f4,d);
#endif
#if defined(AB)&&!defined(CB)
V(V0,f);
#endif
V(V1,i);
#ifdef H
V(L1,P);
#endif
c i0=C0(p1(ZB),PC)+PB.xy;d6=QC*HC.zw+HC.xy;
#ifdef N
if(N){f4=j9(BC,j.w6);}
#endif
#ifdef AB
if(AB){
#ifndef CB
V0=G8(p1(SB),PB.zw,i0 h5);
#else
ob(p1(SB),PB.zw,i0 h5);
#endif
}
#endif
f I=R3(i0);
#ifdef NC
I.y=-I.y;
#endif
#ifdef CB
I.z=k9(MC,0xffu);
#endif
V1=unpackUnorm4x8(AC);
#ifdef H
L1=U1(CC);
#endif
a0(d6);
#ifdef N
a0(f4);
#endif
#if defined(AB)&&!defined(CB)
a0(V0);
#endif
a0(V1);
#ifdef H
a0(L1);
#endif
z1(I);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive