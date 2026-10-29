#pragma once

#include "draw_clockwise_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_path_frag[] = R"===(#ifdef DB
F1
#ifndef Q
y0(z2,o0);
#endif
q1(l3,m0);
#ifndef Q
lc(B6,e7);
#endif
q1(j7,X0);G1
#ifdef Q
I2(HB)
#else
I1(HB)
#endif
{q(M0,e);
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
d B0=
#ifdef EB
o1;
#else
Pc(S);
#endif
i q0;d V1;
#if defined(EB)&&defined(FC)
if(!FC)
#endif
{q0=x8(
#ifdef GB
N0,
#endif
#ifdef H
Y2(R0),
#endif
M0 n3);V1=1.;
#ifdef AB
if(AB){d Uc=D3(Y4(V0));V1=min(Uc,V1);}
#endif
}R2;
#if defined(EB)&&defined(FC)
if(FC){l1(X0,packHalf2x16(U2(B0,I0)));
#ifndef Q
Q2(o0);
#endif
}else
#endif
{E q5=unpackHalf2x16(j1(X0));d la=q5.y;d r5=la==I0?q5.x:C0(.0);d Qf=
#ifndef EB
o6(S)?max(r5,B0):
#endif
r5+B0;
#ifdef N
if(N&&l2.x!=.0){E W0=unpackHalf2x16(j1(m0));d h6=W0.y;d Vc=h6==l2.x?W0.x:C0(.0);V1=min(Vc,V1);}
#endif
V1=max(V1,.0);d v2=kb(r5,.0,V1);d T1=kb(Qf,.0,V1);
#ifdef OB
d g6;if(OB){g6=nb(c0.xy,j.I3,j.J3);}
#endif
#ifndef Q
i Z0=K0(o0);
#ifdef H
if(H&&R0!=r6(X3)){if(T1!=.0){if(v2==.0){q0.xyz=R4(q0.xyz,Z0,Y2(R0));
#ifndef EB
if(T1<V1){v C8=q0.xyz;
#ifdef OB
if(OB){C8+=g6*j.Se;}
#endif
x0(e7,E0(C8,0.0));}
#endif
}else{q0.xyz=K0(e7).xyz;Q2(e7);}}q0.xyz*=q0.w;}
#endif
#endif
q0*=L9(v2,T1,q0.w);
#ifdef OB
q0.xyz=K2(q0.xyz,q0.w,g6);
#endif
#ifndef EB
#ifdef H
#define Rf (!H||R0==r6(X3))&&q0.w>=1.
#else
#define Rf q0.w>=1.
#endif
ff(Rf,X0,packHalf2x16(U2(Qf,I0)));
#else
j2(X0);
#endif
#ifndef Q
ef(q0.x+q0.y+q0.z+q0.w==.0,o0,Z0*(1.-q0.w)+q0);
#endif
}j2(m0);S2;
#ifdef Q
Q1=q0;H3
#else
O1;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive