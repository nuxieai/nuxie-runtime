#pragma once

#include "draw_clockwise_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_path_frag[] = R"===(#ifdef EB
R1
#ifndef V
B0(L2,n0);
#endif
o1(d3,m0);
#ifndef V
tb(p6,P6);
#endif
o1(V6,W0);S1
#ifdef V
A2(IB)
#else
T1(IB)
#endif
{q(a1,f);
#ifdef GB
q(r1,P);
#endif
#ifdef DB
q(m1,d);
#else
q(S,H2);
#endif
q(F0,d);
#ifdef A
q(l1,C);
#endif
#ifdef AB
q(S0,f);
#endif
#ifdef O
q(Q0,d);
#endif
d A0=
#ifdef DB
m1;
#else
Vb(S);
#endif
i o0;d N1;
#if defined(DB)&&defined(FC)
if(!FC)
#endif
{o0=Z7(
#ifdef GB
r1,
#endif
#ifdef O
k3(Q0),
#endif
a1 e3);N1=1.;
#ifdef AB
if(AB){d ac=v3(w5(S0));N1=min(ac,N1);}
#endif
}F2;
#if defined(DB)&&defined(FC)
if(FC){j1(W0,packHalf2x16(I2(A0,F0)));
#ifndef V
E2(n0);
#endif
}else
#endif
{C e5=unpackHalf2x16(h1(W0));d E9=e5.y;d g5=E9==F0?e5.x:I0(.0);d af=
#ifndef DB
g6(S)?max(g5,A0):
#endif
g5+A0;
#ifdef A
if(A&&l1.x!=.0){C U0=unpackHalf2x16(h1(m0));d Y5=U0.y;d bc=Y5==l1.x?U0.x:I0(.0);N1=min(bc,N1);}
#endif
N1=max(N1,.0);d i2=Ba(g5,.0,N1);d M1=Ba(af,.0,N1);
#ifdef OB
d X5;if(OB){X5=Ea(f0.xy,j.M3,j.N3);}
#endif
#ifndef V
i I1=N0(n0);
#ifdef O
if(O&&Q0!=k6(M4)){if(M1!=.0){if(i2==.0){o0.xyz=i5(o0.xyz,I1,k3(Q0));
#ifndef DB
if(M1<N1){v f8=o0.xyz;
#ifdef OB
if(OB){f8+=X5*j.Xd;}
#endif
y0(P6,G0(f8,0.0));}
#endif
}else{o0.xyz=N0(P6).xyz;E2(P6);}}o0.xyz*=o0.w;}
#endif
#endif
o0*=d9(i2,M1,o0.w);
#ifdef OB
o0.xyz=O2(o0.xyz,o0.w,X5);
#endif
#ifndef DB
#ifdef O
#define bf (!O||Q0==k6(M4))&&o0.w>=1.
#else
#define bf o0.w>=1.
#endif
ke(bf,W0,packHalf2x16(I2(af,F0)));
#else
Z1(W0);
#endif
#ifndef V
je(o0.x+o0.y+o0.z+o0.w==.0,n0,I1*(1.-o0.w)+o0);
#endif
}Z1(m0);G2;
#ifdef V
K1=o0;A3
#else
h2;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive