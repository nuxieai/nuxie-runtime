#pragma once

#include "draw_raster_order_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_raster_order_path_frag[] = R"===(#ifdef DB
F1 y0(z2,o0);q1(l3,m0);y0(B6,M4);q1(l7,q8);G1 I1(HB){q(M0,e);
#ifdef GB
q(N0,M);
#endif
#ifdef EB
q(o1,d);
#else
q(S,T2);
#endif
q(I0,d);
#ifdef N
q(l2,E);
#endif
#ifdef AB
q(V0,e);
#endif
#ifdef H
q(R0,d);
#endif
#if!defined(EB)
R2;
#endif
E q5=unpackHalf2x16(j1(q8));d la=q5.y;d w0=la==I0?q5.x:C0(.0);
#ifdef EB
w0+=o1;j2(q8);
#else
w0=Yj(w0,S m1);l1(q8,packHalf2x16(U2(w0,I0)));
#endif
d n;
#ifdef ME
if(ME){n=kb(w0,C0(.0),C0(1.));}else
#endif
{n=abs(w0);
#ifdef XC
if(XC&&I0<.0){n=1.-C0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,C0(1.));}
#ifdef N
if(N&&l2.x<.0){d B1=-l2.x;
#ifdef DD
if(DD){d f6=l2.y;if(f6!=.0){E W0=unpackHalf2x16(j1(m0));d h7=W0.y;d Q4;if(h7!=B1){Q4=h7==f6?W0.x:.0;
#ifndef EB
x0(M4,E0(Q4,.0,.0,.0));
#endif
}else{Q4=K0(M4).x;
#ifndef EB
Q2(M4);
#endif
}n=min(n,Q4);}}
#endif
l1(m0,packHalf2x16(U2(n,B1)));Q2(o0);}else
#endif
{
#ifdef N
if(N){d B1=l2.x;if(B1!=.0){E W0=unpackHalf2x16(j1(m0));d h7=W0.y;n=(h7==B1)?min(W0.x,n):C0(.0);}}
#endif
#ifdef AB
if(AB){d y5=D3(Y4(V0));n=clamp(y5,C0(.0),n);}
#endif
i k=y8(
#ifdef GB
N0,
#endif
#ifdef H
Y2(R0),
#endif
M0 n3);i Z0;if(la!=I0){Z0=K0(o0);
#ifndef EB
x0(M4,Z0);
#endif
}else{Z0=K0(M4);
#ifndef EB
Q2(M4);
#endif
}bool Tf=false;
#ifdef GB
Tf=GB&&N0.z<.0;
#endif
if(Tf){
#ifdef GB
uint Qb=uint(-N0.z-1.);d Rb=we(k,Qb);k=Z0*mix(C0(1.),Rb,n);x0(o0,k);j2(m0);
#endif
}else{
#ifdef H
if(H&&R0!=r6(X3)){k.xyz=R4(k.xyz,Z0,Y2(R0))*k.w;}
#endif
k*=n;d z3=k.w;k+=Z0*(1.-z3);k.xyz=K2(k.xyz,z3,c0.xy,j.I3,j.J3);x0(o0,k);j2(m0);}}
#if!defined(EB)
S2;
#endif
P1;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive