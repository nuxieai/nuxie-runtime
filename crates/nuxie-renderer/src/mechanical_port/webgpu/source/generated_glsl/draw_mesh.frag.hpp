#pragma once

#include "draw_mesh.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_mesh_frag[] = R"===(#ifdef EB
#if(defined(U)&&!defined(N))||defined(QB)
#undef Oc
#else
#define Oc
#endif
U1
#ifndef U
C0(T2,n0);
#endif
#ifndef QB
p1(j3,m0);
#ifndef U
C0(A6,I4);
#endif
p1(h7,Y0);
#else
C0(j3,m0);
#endif
V1
#ifdef NB
V3 p3(A5,v4,TB);W3 B5 w4(U4) C5 k4 l4
#endif
#ifdef U
#ifdef NB
G2(IB)
#else
G2(IB)
#endif
#else
#ifdef NB
X1(IB)
#else
X1(IB)
#endif
#endif
{
#ifdef FB
q(O0,f);
#if defined(GB)
q(U0,M);
#endif
q(S2,c);
#endif
#ifdef N
q(f4,d);
#endif
#ifdef AB
q(V0,f);
#endif
#if defined(FB)&&defined(H)
q(P0,d);
#endif
#ifdef NB
q(d6,c);q(T1,i);
#ifdef H
q(J1,P);
#endif
#endif
#ifdef FB
i l=r8(
#ifdef GB
U0,
#endif
#ifdef H
W2(P0),
#endif
O0 l3);d n=clamp(n2(HD,Pa,S2,.0).x,J0(.0),J0(1.));
#endif
#ifdef NB
i l=e8(TB,U4,d6,j.Ee);d n=1.;
#endif
#ifdef AB
if(AB){d x5=max(B3(V4(V0)),J0(.0));n=min(x5,n);}
#endif
#ifdef Oc
N2;
#endif
#if defined(N)
if(N&&f4!=.0){d P3;
#ifndef QB
D W0=unpackHalf2x16(j1(m0));d d7=W0.y;P3=max(d7==f4?W0.x:J0(.0),J0(.0));
#else
P3=Q0(m0).x;
#endif
P3=max(P3,J0(.0));n=min(n,P3);}
#endif
#ifdef NB
l*=T1;
#endif
#if!defined(U)
i z1=Q0(n0);
#ifdef H
#ifdef FB
P W1=W2(P0);
#endif
#ifdef NB
P W1=J1;
#endif
if(H&&W1!=U3){
#ifdef NB
l.xyz=i6(l);
#endif
l.xyz=N4(l.xyz,z1,W1)*l.w;}
#endif
l*=n;l.xyz=I2(l.xyz,l.w,d0.xy,j.F3,j.G3);
#ifndef QB
l=z1*(1.-l.w)+l;
#endif
y0(n0,l);
#endif
#ifndef QB
g2(m0);g2(Y0);
#else
y0(m0,H0(.0));
#endif
#ifdef Oc
O2;
#endif
#ifdef U
l=(l*n);l.xyz=I2(l.xyz,l.w,d0.xy,j.F3,j.G3);L1=l;E3
#else
o2;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive