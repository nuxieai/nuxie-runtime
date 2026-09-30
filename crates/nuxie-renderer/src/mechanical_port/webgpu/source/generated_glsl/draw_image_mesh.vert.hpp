#pragma once

#include "draw_image_mesh.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_image_mesh_vert[] = R"===(#ifdef BB
c1(w3) K(0,c,QC);d1 c1(K3) K(1,c,RC);d1 c1(z1) K(L9,e,YB);K(M9,e,SB);K(N9,e,PB);K(O9,uint,ZB);K(P9,uint,AC);K(Q9,uint,BC);K(R9,uint,MC);K(Z9,e,HC);d1
#endif
l2 E0 W(0,c,T5);
#ifdef A
KB W(1,d,Z3);
#endif
#if defined(AB)&&!defined(CB)
E0 W(2,e,R0);
#endif
KB W(3,i,P1);
#ifdef O
a3 W(4,R,H1);
#endif
d2
#ifdef BB
j4 k4 S6(RB,w3,x3,K3,L3,z1,h0,G){L(G,x3,QC,c);L(G,L3,RC,c);L(r,h0,YB,e);L(r,h0,SB,e);L(r,h0,PB,e);L(r,h0,ZB,uint);L(r,h0,AC,uint);L(r,h0,BC,uint);L(r,h0,MC,uint);L(r,h0,HC,e);T(T5,c);
#ifdef A
T(Z3,d);
#endif
#if defined(AB)&&!defined(CB)
T(R0,e);
#endif
T(P1,i);
#ifdef O
T(H1,R);
#endif
c k0=K0(n1(YB),QC)+PB.xy;T5=RC*HC.zw+HC.xy;
#ifdef A
if(A){Z3=k6(AC,j.T4);}
#endif
#ifdef AB
if(AB){
#ifndef CB
R0=h8(n1(SB),PB.zw,k0 Y4);
#else
Ga(n1(SB),PB.zw,k0 Y4);
#endif
}
#endif
e I=I3(k0);
#ifdef NC
I.y=-I.y;
#endif
#ifdef CB
I.z=H8(MC,0xffu);
#endif
P1=unpackUnorm4x8(ZB);
#ifdef O
H1=O1(BC);
#endif
Z(T5);
#ifdef A
Z(Z3);
#endif
#if defined(AB)&&!defined(CB)
Z(R0);
#endif
Z(P1);
#ifdef O
Z(H1);
#endif
v1(I);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive