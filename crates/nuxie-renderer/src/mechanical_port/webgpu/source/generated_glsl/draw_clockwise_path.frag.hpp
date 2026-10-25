#pragma once

#include "draw_clockwise_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_path_frag[] = R"===(#ifdef DB
F1
#ifndef R
y0(x2,n0);
#endif
r1(k3,m0);
#ifndef R
kc(E6,h7);
#endif
r1(m7,Y0);G1
#ifdef R
H2(HB)
#else
I1(HB)
#endif
{q(N0,e);
#ifdef GB
q(O0,M);
#endif
#ifdef EB
q(p1,d);
#else
q(T,R2);
#endif
q(I0,d);
#ifdef N
q(k2,E);
#endif
#ifdef AB
q(W0,e);
#endif
#ifdef H
q(S0,d);
#endif
d B0=
#ifdef EB
p1;
#else
Oc(T);
#endif
i p0;d V1;
#if defined(EB)&&defined(FC)
if(!FC)
#endif
{p0=z8(
#ifdef GB
O0,
#endif
#ifdef H
W2(S0),
#endif
N0 m3);V1=1.;
#ifdef AB
if(AB){d Tc=C3(Y4(W0));V1=min(Tc,V1);}
#endif
}P2;
#if defined(EB)&&defined(FC)
if(FC){m1(Y0,packHalf2x16(S2(B0,I0)));
#ifndef R
O2(n0);
#endif
}else
#endif
{E r5=unpackHalf2x16(k1(Y0));d la=r5.y;d v5=la==I0?r5.x:C0(.0);d Qf=
#ifndef EB
r6(T)?max(v5,B0):
#endif
v5+B0;
#ifdef N
if(N&&k2.x!=.0){E X0=unpackHalf2x16(k1(m0));d j6=X0.y;d Uc=j6==k2.x?X0.x:C0(.0);V1=min(Uc,V1);}
#endif
V1=max(V1,.0);d q2=jb(v5,.0,V1);d T1=jb(Qf,.0,V1);
#ifdef OB
d i6;if(OB){i6=mb(c0.xy,j.H3,j.I3);}
#endif
#ifndef R
i a1=K0(n0);
#ifdef H
if(H&&S0!=x6(X3)){if(T1!=.0){if(q2==.0){p0.xyz=R4(p0.xyz,a1,W2(S0));
#ifndef EB
if(T1<V1){v E8=p0.xyz;
#ifdef OB
if(OB){E8+=i6*j.Se;}
#endif
x0(h7,E0(E8,0.0));}
#endif
}else{p0.xyz=K0(h7).xyz;O2(h7);}}p0.xyz*=p0.w;}
#endif
#endif
p0*=N9(q2,T1,p0.w);
#ifdef OB
p0.xyz=J2(p0.xyz,p0.w,i6);
#endif
#ifndef EB
#ifdef H
#define Rf (!H||S0==x6(X3))&&p0.w>=1.
#else
#define Rf p0.w>=1.
#endif
ff(Rf,Y0,packHalf2x16(S2(Qf,I0)));
#else
i2(Y0);
#endif
#ifndef R
ef(p0.x+p0.y+p0.z+p0.w==.0,n0,a1*(1.-p0.w)+p0);
#endif
}i2(m0);Q2;
#ifdef R
Q1=p0;G3
#else
O1;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive