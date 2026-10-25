#pragma once

#include "draw_mesh.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_mesh_frag[] = R"===(#ifdef EB
#if(defined(R)&&!defined(N))||defined(RB)
#undef Tc
#else
#define Tc
#endif
F1
#ifndef R
y0(x2,n0);
#endif
#ifndef RB
r1(k3,m0);
#ifndef R
y0(E6,K4);
#endif
r1(m7,Y0);
#else
y0(k3,m0);
#endif
G1
#ifdef NB
X3 q3(C5,x4,UB);Y3 D5 y4(W4) E5 m4 n4
#endif
#ifdef R
#ifdef NB
H2(HB)
#else
H2(HB)
#endif
#else
#ifdef NB
I1(HB)
#else
I1(HB)
#endif
#endif
{
#ifdef FB
q(P0,e);
#if defined(GB)
q(V0,M);
#endif
q(U2,c);
#endif
#ifdef N
q(h4,d);
#endif
#ifdef AB
q(W0,e);
#endif
#if defined(FB)&&defined(H)
q(Q0,d);
#endif
#ifdef NB
q(g6,c);q(Y1,i);
#ifdef H
q(N1,P);
#endif
#endif
#ifdef FB
i k=x8(
#ifdef GB
V0,
#endif
#ifdef H
W2(Q0),
#endif
P0 m3);d n=clamp(p2(JD,Sa,U2,.0).x,J0(.0),J0(1.));
#endif
#ifdef NB
i k=h8(UB,W4,g6,j.Je);d n=1.;
#endif
#ifdef AB
if(AB){d z5=max(C3(X4(W0)),J0(.0));n=min(z5,n);}
#endif
#ifdef Tc
P2;
#endif
#if defined(N)
if(N&&h4!=.0){d R3;
#ifndef RB
E X0=unpackHalf2x16(k1(m0));d i7=X0.y;R3=max(i7==h4?X0.x:J0(.0),J0(.0));
#else
R3=K0(m0).x;
#endif
R3=max(R3,J0(.0));n=min(n,R3);}
#endif
#ifdef NB
k*=Y1;
#endif
#if!defined(R)
i a1=K0(n0);
#ifdef H
#ifdef FB
P Z1=W2(Q0);
#endif
#ifdef NB
P Z1=N1;
#endif
if(H&&Z1!=W3){
#ifdef NB
k.xyz=l6(k);
#endif
k.xyz=P4(k.xyz,a1,Z1)*k.w;}
#endif
k*=n;k.xyz=J2(k.xyz,k.w,c0.xy,j.G3,j.H3);
#ifndef RB
k=a1*(1.-k.w)+k;
#endif
x0(n0,k);
#endif
#ifndef RB
i2(m0);i2(Y0);
#else
x0(m0,H0(.0));
#endif
#ifdef Tc
Q2;
#endif
#ifdef R
k=(k*n);k.xyz=J2(k.xyz,k.w,c0.xy,j.G3,j.H3);Q1=k;F3
#else
O1;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive