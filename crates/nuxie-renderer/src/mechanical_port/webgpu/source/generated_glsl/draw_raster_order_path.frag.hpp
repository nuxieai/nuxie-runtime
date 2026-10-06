#pragma once

#include "draw_raster_order_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_raster_order_path_frag[] = R"===(#ifdef FB
S1 B0(K2,n0);o1(c3,m0);B0(o6,C4);o1(U6,Q7);T1 U1(IB){q(a1,e);
#ifdef GB
q(v1,O);
#endif
#ifdef DB
q(m1,d);
#else
q(S,G2);
#endif
q(F0,d);
#ifdef A
q(l1,C);
#endif
#ifdef AB
q(R0,e);
#endif
#ifdef N
q(Q0,d);
#endif
#if!defined(DB)
E2;
#endif
C d5=unpackHalf2x16(h1(Q7));d E9=d5.y;d w0=E9==F0?d5.x:H0(.0);
#ifdef DB
w0+=m1;a2(Q7);
#else
w0=aj(w0,S k1);j1(Q7,packHalf2x16(H2(w0,F0)));
#endif
d n;
#ifdef GE
if(GE){n=Ba(w0,H0(.0),H0(1.));}else
#endif
{n=abs(w0);
#ifdef XC
if(XC&&F0<.0){n=1.-H0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,H0(1.));}
#ifdef A
if(A&&l1.x<.0){d X0=-l1.x;
#ifdef AD
if(AD){d F4=l1.y;if(F4!=.0){C T0=unpackHalf2x16(h1(m0));d P6=T0.y;d H4;if(P6!=X0){H4=P6==F4?T0.x:.0;
#ifndef DB
y0(C4,I0(H4,.0,.0,.0));
#endif
}else{H4=N0(C4).x;
#ifndef DB
D2(C4);
#endif
}n=min(n,H4);}}
#endif
j1(m0,packHalf2x16(H2(n,X0)));D2(n0);}else
#endif
{
#ifdef A
if(A){d X0=l1.x;if(X0!=.0){C T0=unpackHalf2x16(h1(m0));d P6=T0.y;n=(P6==X0)?min(T0.x,n):H0(.0);}}
#endif
#ifdef AB
if(AB){d m5=w3(v5(R0));n=clamp(m5,H0(.0),n);}
#endif
i p=X7(
#ifdef GB
v1,
#endif
#ifdef N
k3(Q0),
#endif
a1 e3);i J1;if(E9!=F0){J1=N0(n0);
#ifndef DB
y0(C4,J1);
#endif
}else{J1=N0(C4);
#ifndef DB
D2(C4);
#endif
}bool df=false;
#ifdef GB
df=GB&&v1.z<.0;
#endif
if(df){
#ifdef GB
uint gj=uint(-v1.z-1.);d hj=Yi(p,gj);p=J1*mix(H0(1.),hj,n);y0(n0,p);a2(m0);
#endif
}else{
#ifdef N
if(N&&Q0!=j6(M4)){p.xyz=h5(p.xyz,J1,k3(Q0))*p.w;}
#endif
p*=n;d n3=p.w;p+=J1*(1.-n3);p.xyz=M2(p.xyz,n3,f0.xy,j.M3,j.N3);y0(n0,p);a2(m0);}}
#if!defined(DB)
F2;
#endif
h2;}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive