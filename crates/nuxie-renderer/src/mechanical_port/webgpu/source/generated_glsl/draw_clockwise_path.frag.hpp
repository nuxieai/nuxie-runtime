#pragma once

#include "draw_clockwise_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_path_frag[] = R"===(#ifdef EB
F1
#ifndef R
y0(x2,n0);
#endif
r1(k3,m0);
#ifndef R
gc(E6,h7);
#endif
r1(m7,Y0);G1
#ifdef R
H2(HB)
#else
I1(HB)
#endif
{q(P0,e);
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
d B0=
#ifdef DB
p1;
#else
Lc(T);
#endif
i p0;d V1;
#if defined(DB)&&defined(FC)
if(!FC)
#endif
{p0=y8(
#ifdef GB
V0,
#endif
#ifdef H
W2(Q0),
#endif
P0 m3);V1=1.;
#ifdef AB
if(AB){d Qc=C3(X4(W0));V1=min(Qc,V1);}
#endif
}P2;
#if defined(DB)&&defined(FC)
if(FC){m1(Y0,packHalf2x16(S2(B0,G0)));
#ifndef R
O2(n0);
#endif
}else
#endif
{E p5=unpackHalf2x16(k1(Y0));d ka=p5.y;d q5=ka==G0?p5.x:J0(.0);d Kf=
#ifndef DB
r6(T)?max(q5,B0):
#endif
q5+B0;
#ifdef N
if(N&&k2.x!=.0){E X0=unpackHalf2x16(k1(m0));d j6=X0.y;d Rc=j6==k2.x?X0.x:J0(.0);V1=min(Rc,V1);}
#endif
V1=max(V1,.0);d q2=ib(q5,.0,V1);d T1=ib(Kf,.0,V1);
#ifdef OB
d i6;if(OB){i6=lb(c0.xy,j.G3,j.H3);}
#endif
#ifndef R
i a1=K0(n0);
#ifdef H
if(H&&Q0!=x6(W3)){if(T1!=.0){if(q2==.0){p0.xyz=P4(p0.xyz,a1,W2(Q0));
#ifndef DB
if(T1<V1){v D8=p0.xyz;
#ifdef OB
if(OB){D8+=i6*j.Ke;}
#endif
x0(h7,H0(D8,0.0));}
#endif
}else{p0.xyz=K0(h7).xyz;O2(h7);}}p0.xyz*=p0.w;}
#endif
#endif
p0*=M9(q2,T1,p0.w);
#ifdef OB
p0.xyz=J2(p0.xyz,p0.w,i6);
#endif
#ifndef DB
#ifdef H
#define Lf (!H||Q0==x6(W3))&&p0.w>=1.
#else
#define Lf p0.w>=1.
#endif
Xe(Lf,Y0,packHalf2x16(S2(Kf,G0)));
#else
i2(Y0);
#endif
#ifndef R
We(p0.x+p0.y+p0.z+p0.w==.0,n0,a1*(1.-p0.w)+p0);
#endif
}i2(m0);Q2;
#ifdef R
Q1=p0;F3
#else
O1;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive