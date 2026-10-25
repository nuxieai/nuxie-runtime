#pragma once

#include "draw_raster_order_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_raster_order_path_frag[] = R"===(#ifdef EB
F1 y0(x2,n0);r1(k3,m0);y0(E6,K4);r1(m7,q8);G1 I1(HB){q(P0,e);
#ifdef GB
q(V0,M);
#endif
#ifdef DB
q(p1,d);
#else
q(T,R2);
#endif
q(G0,d);
#ifdef N
q(k2,E);
#endif
#ifdef AB
q(W0,e);
#endif
#ifdef H
q(Q0,d);
#endif
#if!defined(DB)
P2;
#endif
E p5=unpackHalf2x16(k1(q8));d ka=p5.y;d w0=ka==G0?p5.x:J0(.0);
#ifdef DB
w0+=p1;i2(q8);
#else
w0=Uj(w0,T n1);m1(q8,packHalf2x16(S2(w0,G0)));
#endif
d n;
#ifdef NE
if(NE){n=ib(w0,J0(.0),J0(1.));}else
#endif
{n=abs(w0);
#ifdef YC
if(YC&&G0<.0){n=1.-J0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,J0(1.));}
#ifdef N
if(N&&k2.x<.0){d B1=-k2.x;
#ifdef ED
if(ED){d h6=k2.y;if(h6!=.0){E X0=unpackHalf2x16(k1(m0));d i7=X0.y;d O4;if(i7!=B1){O4=i7==h6?X0.x:.0;
#ifndef DB
x0(K4,H0(O4,.0,.0,.0));
#endif
}else{O4=K0(K4).x;
#ifndef DB
O2(K4);
#endif
}n=min(n,O4);}}
#endif
m1(m0,packHalf2x16(S2(n,B1)));O2(n0);}else
#endif
{
#ifdef N
if(N){d B1=k2.x;if(B1!=.0){E X0=unpackHalf2x16(k1(m0));d i7=X0.y;n=(i7==B1)?min(X0.x,n):J0(.0);}}
#endif
#ifdef AB
if(AB){d z5=C3(X4(W0));n=clamp(z5,J0(.0),n);}
#endif
i k=y8(
#ifdef GB
V0,
#endif
#ifdef H
W2(Q0),
#endif
P0 m3);i a1;if(ka!=G0){a1=K0(n0);
#ifndef DB
x0(K4,a1);
#endif
}else{a1=K0(K4);
#ifndef DB
O2(K4);
#endif
}bool Mf=false;
#ifdef GB
Mf=GB&&V0.z<.0;
#endif
if(Mf){
#ifdef GB
uint ak=uint(-V0.z-1.);d bk=Tj(k,ak);k=a1*mix(J0(1.),bk,n);x0(n0,k);i2(m0);
#endif
}else{
#ifdef H
if(H&&Q0!=x6(W3)){k.xyz=P4(k.xyz,a1,W2(Q0))*k.w;}
#endif
k*=n;d y3=k.w;k+=a1*(1.-y3);k.xyz=J2(k.xyz,y3,c0.xy,j.G3,j.H3);x0(n0,k);i2(m0);}}
#if!defined(DB)
Q2;
#endif
O1;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive