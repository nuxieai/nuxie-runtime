#pragma once

#include "draw_clockwise_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_path_frag[] = R"===(#ifdef GB
I1
#ifndef Q
w0(S2,j0);
#endif
j1(T2,h0);
#ifndef Q
Sa(f6,C6);
#endif
j1(I6,P0);J1
#ifdef Q
o2(JB)
#else
L1(JB)
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
#ifdef I
r(U1,E);
#endif
#ifdef BB
r(M0,g);
#endif
#ifdef AB
r(e2,c);
#endif
c r0=
#ifdef EB
i1;
#else
pb(O);
#endif
i v0;c F1;
#if defined(EB)&&defined(FC)
if(!FC)
#endif
{v0=L7(f1,
#ifdef KB
A2,
#endif
1. U2);F1=1.;
#ifdef BB
if(BB){c ub=h3(Z4(M0));F1=min(ub,F1);}
#endif
}w2;
#if defined(EB)&&defined(FC)
if(FC){c1(P0,packHalf2x16(B2(r0,B0)));
#ifndef Q
v2(j0);
#endif
}else
#endif
{E P4=unpackHalf2x16(Y0(P0));c i9=P4.y;c Q4=i9==B0?P4.x:G0(.0);c pe=
#ifndef EB
T5(O)?max(Q4,r0):
#endif
Q4+r0;
#ifdef I
if(I&&U1.x!=.0){E N0=unpackHalf2x16(Y0(h0));c K5=N0.y;c vb=K5==U1.x?N0.x:G0(.0);F1=min(vb,F1);}
#endif
F1=max(F1,.0);c Z1=da(Q4,.0,F1);c E1=da(pe,.0,F1);
#ifdef MB
c J5;if(MB){J5=ga(Z.xy,m.B3,m.C3);}
#endif
#ifndef Q
i K1=I0(j0);
#ifdef AB
if(AB){if(e2!=Y5(P5)&&E1!=.0){if(Z1==.0){v0.xyz=S4(v0.xyz,K1,Z5(e2));
#ifndef EB
if(E1<F1){A O7=v0.xyz;
#ifdef MB
if(MB){O7+=J5*m.td;}
#endif
x0(C6,C0(O7,0.0));}
#endif
}else{v0.xyz=I0(C6).xyz;v2(C6);}}v0.xyz*=v0.w;}
#endif
#endif
v0*=J8(Z1,E1,v0.w);
#ifdef MB
v0.xyz=F2(v0.xyz,v0.w,J5);
#endif
#ifndef EB
#ifdef AB
#define qe (!AB||e2==Y5(P5))&&v0.w>=1.
#else
#define qe v0.w>=1.
#endif
Gd(qe,P0,packHalf2x16(B2(pe,B0)));
#else
d2(P0);
#endif
#ifndef Q
Fd(v0.w==.0,j0,K1*(1.-v0.w)+v0);
#endif
}d2(h0);x2;
#ifdef Q
C1=v0;n3
#else
Y1;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive