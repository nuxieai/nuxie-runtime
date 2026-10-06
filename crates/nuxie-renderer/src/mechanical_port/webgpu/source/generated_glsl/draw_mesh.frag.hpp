#pragma once

#include "draw_mesh.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_mesh_frag[] = R"===(#ifdef FB
#if(defined(W)&&!defined(A))||defined(QB)
#undef dc
#else
#define dc
#endif
S1
#ifndef W
B0(K2,n0);
#endif
#ifndef QB
o1(c3,m0);
#ifndef W
B0(o6,C4);
#endif
o1(U6,V0);
#else
B0(c3,m0);
#endif
T1
#ifdef NB
O3 i3(w5,m4,CC);P3 x5 n4(r5) y5 g4 h4
#endif
#ifdef W
#ifdef NB
z2(IB)
#else
z2(IB)
#endif
#else
#ifdef NB
U1(IB)
#else
U1(IB)
#endif
#endif
{
#ifdef EB
q(a1,e);
#if defined(GB)
q(v1,O);
#endif
q(J2,c);
#endif
#ifdef A
q(Z3,d);
#endif
#ifdef AB
q(R0,e);
#endif
#if defined(EB)&&defined(N)
q(Q0,d);
#endif
#ifdef NB
q(V5,c);q(R1,i);
#ifdef N
q(I1,Q);
#endif
#endif
#ifdef EB
i p=X7(
#ifdef GB
v1,
#endif
#ifdef N
k3(Q0),
#endif
a1 e3);d n=clamp(o2(FD,na,J2,.0).x,H0(.0),H0(1.));
#endif
#ifdef NB
i p=J7(CC,r5,V5,j.Wd);d n=1.;
#endif
#ifdef AB
if(AB){d m5=max(w3(v5(R0)),H0(.0));n=min(m5,n);}
#endif
#ifdef dc
E2;
#endif
#if defined(A)
if(A&&Z3!=.0){d G3;
#ifndef QB
C T0=unpackHalf2x16(h1(m0));d P6=T0.y;G3=max(P6==Z3?T0.x:H0(.0),H0(.0));
#else
G3=N0(m0).x;
#endif
G3=max(G3,H0(.0));n=min(n,G3);}
#endif
#ifdef NB
p*=R1;
#endif
#if!defined(W)
i J1=N0(n0);
#ifdef N
#ifdef EB
Q z3=k3(Q0);
#endif
#ifdef NB
Q z3=I1;
#endif
if(N&&z3!=M4){
#ifdef NB
p.xyz=Q6(p);
#endif
p.xyz=h5(p.xyz,J1,z3)*p.w;}
#endif
p*=n;p.xyz=M2(p.xyz,p.w,f0.xy,j.M3,j.N3);
#ifndef QB
p=J1*(1.-p.w)+p;
#endif
y0(n0,p);
#endif
#ifndef QB
a2(m0);a2(V0);
#else
y0(m0,I0(.0));
#endif
#ifdef dc
F2;
#endif
#ifdef W
p=(p*n);p.xyz=M2(p.xyz,p.w,f0.xy,j.M3,j.N3);L1=p;A3
#else
h2;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive