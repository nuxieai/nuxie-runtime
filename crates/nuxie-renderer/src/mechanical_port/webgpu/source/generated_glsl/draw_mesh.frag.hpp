#pragma once

#include "draw_mesh.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_mesh_frag[] = R"===(#ifdef EB
#if(defined(V)&&!defined(A))||defined(QB)
#undef dc
#else
#define dc
#endif
R1
#ifndef V
B0(L2,n0);
#endif
#ifndef QB
o1(d3,m0);
#ifndef V
B0(p6,C4);
#endif
o1(V6,W0);
#else
B0(d3,m0);
#endif
S1
#ifdef NB
O3 i3(x5,m4,DC);P3 y5 n4(v5) z5 g4 h4
#endif
#ifdef V
#ifdef NB
A2(IB)
#else
A2(IB)
#endif
#else
#ifdef NB
T1(IB)
#else
T1(IB)
#endif
#endif
{
#ifdef FB
q(a1,f);
#if defined(GB)
q(r1,P);
#endif
q(K2,c);
#endif
#ifdef A
q(Z3,d);
#endif
#ifdef AB
q(S0,f);
#endif
#if defined(FB)&&defined(O)
q(Q0,d);
#endif
#ifdef NB
q(W5,c);q(Q1,i);
#ifdef O
q(H1,R);
#endif
#endif
#ifdef FB
i l=Z7(
#ifdef GB
r1,
#endif
#ifdef O
k3(Q0),
#endif
a1 e3);d o=clamp(o2(GD,na,K2,.0).x,I0(.0),I0(1.));
#endif
#ifdef NB
i l=L7(DC,v5,W5,j.Wd);d o=1.;
#endif
#ifdef AB
if(AB){d n5=max(v3(w5(S0)),I0(.0));o=min(n5,o);}
#endif
#ifdef dc
F2;
#endif
#if defined(A)
if(A&&Z3!=.0){d G3;
#ifndef QB
C U0=unpackHalf2x16(h1(m0));d Q6=U0.y;G3=max(Q6==Z3?U0.x:I0(.0),I0(.0));
#else
G3=N0(m0).x;
#endif
G3=max(G3,I0(.0));o=min(o,G3);}
#endif
#ifdef NB
l*=Q1;
#endif
#if!defined(V)
i I1=N0(n0);
#ifdef O
#ifdef FB
R y3=k3(Q0);
#endif
#ifdef NB
R y3=H1;
#endif
if(O&&y3!=M4){
#ifdef NB
l.xyz=R6(l);
#endif
l.xyz=i5(l.xyz,I1,y3)*l.w;}
#endif
l*=o;
#ifdef CC
if(CC){l=z3(l);}
#endif
l.xyz=O2(l.xyz,l.w,f0.xy,j.M3,j.N3);
#ifndef QB
l=I1*(1.-l.w)+l;
#endif
y0(n0,l);
#endif
#ifndef QB
Z1(m0);Z1(W0);
#else
y0(m0,G0(.0));
#endif
#ifdef dc
G2;
#endif
#ifdef V
l=(l*o);l.xyz=O2(l.xyz,l.w,f0.xy,j.M3,j.N3);K1=l;A3
#else
h2;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive