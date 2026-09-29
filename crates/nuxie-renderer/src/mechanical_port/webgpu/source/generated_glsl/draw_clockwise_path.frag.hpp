#pragma once

#include "draw_clockwise_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_path_frag[] = R"===(#ifdef GB
J1
#ifndef Q
x0(S2,j0);
#endif
j1(T2,h0);
#ifndef Q
Va(h6,F6);
#endif
j1(L6,P0);K1
#ifdef Q
p2(JB)
#else
M1(JB)
#endif
{r(f1,g);
#ifdef KB
r(A2,R);
#endif
#ifdef EB
r(i1,c);
#else
r(O,z2);
#endif
r(B0,c);
#ifdef J
r(V1,E);
#endif
#ifdef BB
r(M0,g);
#endif
#ifdef AB
r(f2,c);
#endif
c v0=
#ifdef EB
i1;
#else
sb(O);
#endif
i w0;c G1;
#if defined(EB)&&defined(FC)
if(!FC)
#endif
{w0=N7(f1,
#ifdef KB
A2,
#endif
1. U2);G1=1.;
#ifdef BB
if(BB){c xb=h3(d5(M0));G1=min(xb,G1);}
#endif
}x2;
#if defined(EB)&&defined(FC)
if(FC){c1(P0,packHalf2x16(B2(v0,B0)));
#ifndef Q
w2(j0);
#endif
}else
#endif
{E S4=unpackHalf2x16(Y0(P0));c j9=S4.y;c T4=j9==B0?S4.x:G0(.0);c ue=
#ifndef EB
V5(O)?max(T4,v0):
#endif
T4+v0;
#ifdef J
if(J&&V1.x!=.0){E N0=unpackHalf2x16(Y0(h0));c M5=N0.y;c yb=M5==V1.x?N0.x:G0(.0);G1=min(yb,G1);}
#endif
G1=max(G1,.0);c a2=ea(T4,.0,G1);c F1=ea(ue,.0,G1);
#ifdef MB
c L5;if(MB){L5=ha(a0.xy,m.B3,m.C3);}
#endif
#ifndef Q
i L1=I0(j0);
#ifdef AB
if(AB){if(f2!=a6(R5)&&F1!=.0){if(a2==.0){w0.xyz=V4(w0.xyz,L1,c6(f2));
#ifndef EB
if(F1<G1){A Q7=w0.xyz;
#ifdef MB
if(MB){Q7+=L5*m.vd;}
#endif
y0(F6,C0(Q7,0.0));}
#endif
}else{w0.xyz=I0(F6).xyz;w2(F6);}}w0.xyz*=w0.w;}
#endif
#endif
w0*=L8(a2,F1,w0.w);
#ifdef MB
w0.xyz=F2(w0.xyz,w0.w,L5);
#endif
#ifndef EB
#ifdef AB
#define ve (!AB||f2==a6(R5))&&w0.w>=1.
#else
#define ve w0.w>=1.
#endif
Ld(ve,P0,packHalf2x16(B2(ue,B0)));
#else
e2(P0);
#endif
#ifndef Q
Kd(w0.w==.0,j0,L1*(1.-w0.w)+w0);
#endif
}e2(h0);y2;
#ifdef Q
D1=w0;n3
#else
Z1;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive