#pragma once

#include "draw_clockwise_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_path_frag[] = R"===(#ifdef EB
H1
#ifndef Q
y0(B2,p0);
#endif
w1(p3,n0);
#ifndef Q
oc(D6,g7);
#endif
w1(l7,c1);I1
#ifdef Q
K2(HB)
#else
D1(HB)
#endif
{q(N0,e);
#ifdef GB
q(O0,M);
#endif
#ifdef DB
q(r1,d);
#else
q(S,T2);
#endif
q(J0,d);
#ifdef N
q(l2,F);
#endif
#ifdef AB
q(Y0,e);
#endif
#ifdef H
q(S0,d);
#endif
d E0=
#ifdef DB
r1;
#else
Tc(S);
#endif
i q0;d X1;
#if defined(DB)&&defined(FC)
if(!FC)
#endif
{q0=C8(
#ifdef GB
O0,
#endif
#ifdef H
Z2(S0),
#endif
N0 r3);X1=1.;
#ifdef AB
if(AB){d Yc=F3(y4(Y0));X1=min(Yc,X1);}
#endif
}R2;
#if defined(DB)&&defined(FC)
if(FC){o1(c1,packHalf2x16(U2(E0,J0)));
#ifndef Q
Q2(p0);
#endif
}else
#endif
{F A5=unpackHalf2x16(m1(c1));d oa=A5.y;d B5=oa==J0?A5.x:A0(.0);d Yf=
#ifndef DB
r6(S)?max(B5,E0):
#endif
B5+E0;
#ifdef N
if(N&&l2.x!=.0){F Z0=unpackHalf2x16(m1(n0));d k6=Z0.y;d Zc=k6==l2.x?Z0.x:A0(.0);X1=min(Zc,X1);}
#endif
X1=max(X1,.0);d x2=nb(B5,.0,X1);d U1=nb(Yf,.0,X1);
#ifdef PB
d j6;if(PB){j6=qb(Y.xy,j.K3,j.L3);}
#endif
#ifndef Q
i X0=L0(p0);
#ifdef H
if(H&&S0!=x6(X3)){if(U1!=.0){if(x2==.0){q0.xyz=X4(q0.xyz,X0,Z2(S0));
#ifndef DB
if(U1<X1){v H8=q0.xyz;
#ifdef PB
if(PB){H8+=j6*j.Xe;}
#endif
x0(g7,z0(H8,0.0));}
#endif
}else{q0.xyz=L0(g7).xyz;Q2(g7);}}q0.xyz*=q0.w;}
#endif
#endif
q0*=P9(x2,U1,q0.w);
#ifdef PB
q0.xyz=M2(q0.xyz,q0.w,j6);
#endif
#ifndef DB
#ifdef H
#define Zf (!H||S0==x6(X3))&&q0.w>=1.
#else
#define Zf q0.w>=1.
#endif
lf(Zf,c1,packHalf2x16(U2(Yf,J0)));
#else
k2(c1);
#endif
#ifndef Q
kf(q0.x+q0.y+q0.z+q0.w==.0,p0,X0*(1.-q0.w)+q0);
#endif
}k2(n0);S2;
#ifdef Q
S1=q0;J3
#else
K1;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive