#pragma once

#include "draw_mesh.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_mesh_frag[] = R"===(#ifdef FB
#if(defined(O)&&!defined(I))||defined(RB)
#undef Cb
#else
#define Cb
#endif
J1
#ifndef O
y0(G2,k0);
#endif
#ifndef RB
i1(V2,h0);
#ifndef O
y0(i6,m4);
#endif
i1(L6,Q0);
#else
y0(V2,h0);
#endif
K1
#ifdef OB
F3 c3(d5,X3,HC);G3 e5 Y3(X5)f5 Q3 R3
#endif
#ifdef O
#ifdef OB
r2(IB)
#else
r2(IB)
#endif
#else
#ifdef OB
M1(IB)
#else
M1(IB)
#endif
#endif
{
#ifdef GB
r(V1,f);
#if defined(JB)
r(C2,Q);
#endif
r(F2,c);
#endif
#ifdef I
r(L3,d);
#endif
#ifdef BB
r(M0,f);
#endif
#if defined(GB)&&defined(AB)
r(g2,d);
#endif
#ifdef OB
r(H5,c);r(H1,i);
#ifdef AB
r(A1,L);
#endif
#endif
#ifdef GB
i j=M7(V1,
#ifdef JB
C2,
#endif
1. W2);d o=clamp(i2(FD,R9,F2,.0).x,I0(.0),I0(1.));
#endif
#ifdef OB
i j=B7(HC,X5,H5,l.Dd);d o=1.;
#endif
#ifdef BB
if(BB){d Y4=max(i3(c5(M0)),I0(.0));o=min(Y4,o);}
#endif
#ifdef Cb
z2;
#endif
#if defined(I)
if(I&&L3!=.0){d w3;
#ifndef RB
E O0=unpackHalf2x16(Y0(h0));d G6=O0.y;w3=max(G6==L3?O0.x:I0(.0),I0(.0));
#else
w3=J0(h0).x;
#endif
w3=max(w3,I0(.0));o=min(o,w3);}
#endif
#ifdef OB
j*=H1;
#endif
#if!defined(O)
i L1=J0(k0);
#ifdef AB
if(AB){
#ifdef GB
L U3=d6(g2);
#endif
#ifdef OB
j.xyz=H6(j);L U3=A1;
#endif
if(U3!=S5){j.xyz=U4(j.xyz,L1,U3);}j.w*=o;j.xyz*=j.w;}else
#endif
{j*=o;}
#ifdef AC
if(AC){j=l3(j);}
#endif
j.xyz=J2(j.xyz,j.w,c0.xy,l.C3,l.D3);
#ifndef RB
j=L1*(1.-j.w)+j;
#endif
z0(k0,j);
#endif
#ifndef RB
f2(h0);f2(Q0);
#else
z0(h0,D0(.0));
#endif
#ifdef Cb
A2;
#endif
#ifdef O
j=(j*o);j.xyz=J2(j.xyz,j.w,c0.xy,l.C3,l.D3);C1=j;m3
#else
a2;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive