#pragma once

#include "draw_mesh.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_mesh_frag[] = R"===(#ifdef GB
#if(defined(Q)&&!defined(I))||defined(TB)
#undef xb
#else
#define xb
#endif
I1
#ifndef Q
w0(S2,j0);
#endif
#ifndef TB
j1(T2,h0);
#ifndef Q
w0(f6,k4);
#endif
j1(I6,P0);
#else
w0(T2,h0);
#endif
J1
#ifdef PB
E3 Z2(a5,W3,JC);F3 c5 X3(U5)d5 Q3 R3
#endif
#ifdef Q
#ifdef PB
o2(JB)
#else
o2(JB)
#endif
#else
#ifdef PB
L1(JB)
#else
L1(JB)
#endif
#endif
{
#ifdef FB
r(f1,g);
#if defined(KB)
r(A2,R);
#endif
r(D2,d);
#endif
#ifdef I
r(K3,c);
#endif
#ifdef BB
r(M0,g);
#endif
#if defined(FB)&&defined(AB)
r(e2,c);
#endif
#ifdef PB
r(F5,d);r(H1,c);
#ifdef AB
r(A1,N);
#endif
#endif
#ifdef FB
i j=L7(f1,
#ifdef KB
A2,
#endif
1. U2);c n=clamp(n2(CD,Q9,D2,.0).x,G0(.0),G0(1.));
#endif
#ifdef PB
i j=z7(JC,U5,F5,m.sd);c n=1.;
#endif
#ifdef BB
if(BB){c W4=max(h3(Z4(M0)),G0(.0));n=min(W4,n);}
#endif
#ifdef xb
w2;
#endif
#if defined(I)
if(I&&K3!=.0){c v3;
#ifndef TB
E N0=unpackHalf2x16(Y0(h0));c D6=N0.y;v3=max(D6==K3?N0.x:G0(.0),G0(.0));
#else
v3=I0(h0).x;
#endif
v3=max(v3,G0(.0));n=min(n,v3);}
#endif
#ifdef PB
n*=H1;
#endif
#if!defined(Q)
i K1=I0(j0);
#ifdef AB
if(AB){
#ifdef FB
N T3=Z5(e2);
#endif
#ifdef PB
j.xyz=E6(j);N T3=A1;
#endif
if(T3!=P5){j.xyz=S4(j.xyz,K1,T3);}j.w*=n;j.xyz*=j.w;}else
#endif
{j*=n;}
#ifdef CC
if(CC){j=m3(j);}
#endif
j.xyz=F2(j.xyz,j.w,Z.xy,m.B3,m.C3);
#ifndef TB
j=K1*(1.-j.w)+j;
#endif
x0(j0,j);
#endif
#ifndef TB
d2(h0);d2(P0);
#else
x0(h0,C0(.0));
#endif
#ifdef xb
x2;
#endif
#ifdef Q
j=(j*n);j.xyz=F2(j.xyz,j.w,Z.xy,m.B3,m.C3);C1=j;n3
#else
Y1;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive