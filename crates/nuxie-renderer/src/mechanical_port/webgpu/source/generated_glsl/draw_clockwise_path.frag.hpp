#pragma once

#include "draw_clockwise_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_path_frag[] = R"===(#ifdef FB
S1
#ifndef W
B0(K2,n0);
#endif
o1(c3,m0);
#ifndef W
tb(o6,O6);
#endif
o1(U6,V0);T1
#ifdef W
z2(IB)
#else
U1(IB)
#endif
{q(a1,e);
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
d A0=
#ifdef DB
m1;
#else
Vb(S);
#endif
i o0;d O1;
#if defined(DB)&&defined(EC)
if(!EC)
#endif
{o0=X7(
#ifdef GB
v1,
#endif
#ifdef N
k3(Q0),
#endif
a1 e3);O1=1.;
#ifdef AB
if(AB){d ac=w3(v5(R0));O1=min(ac,O1);}
#endif
}E2;
#if defined(DB)&&defined(EC)
if(EC){j1(V0,packHalf2x16(H2(A0,F0)));
#ifndef W
D2(n0);
#endif
}else
#endif
{C d5=unpackHalf2x16(h1(V0));d E9=d5.y;d f5=E9==F0?d5.x:H0(.0);d bf=
#ifndef DB
f6(S)?max(f5,A0):
#endif
f5+A0;
#ifdef A
if(A&&l1.x!=.0){C T0=unpackHalf2x16(h1(m0));d X5=T0.y;d bc=X5==l1.x?T0.x:H0(.0);O1=min(bc,O1);}
#endif
O1=max(O1,.0);d i2=Ba(f5,.0,O1);d N1=Ba(bf,.0,O1);
#ifdef OB
d W5;if(OB){W5=Ea(f0.xy,j.M3,j.N3);}
#endif
#ifndef W
i J1=N0(n0);
#ifdef N
if(N&&Q0!=j6(M4)){if(N1!=.0){if(i2==.0){o0.xyz=h5(o0.xyz,J1,k3(Q0));
#ifndef DB
if(N1<O1){v d8=o0.xyz;
#ifdef OB
if(OB){d8+=W5*j.Xd;}
#endif
y0(O6,I0(d8,0.0));}
#endif
}else{o0.xyz=N0(O6).xyz;D2(O6);}}o0.xyz*=o0.w;}
#endif
#endif
o0*=c9(i2,N1,o0.w);
#ifdef OB
o0.xyz=M2(o0.xyz,o0.w,W5);
#endif
#ifndef DB
#ifdef N
#define cf (!N||Q0==j6(M4))&&o0.w>=1.
#else
#define cf o0.w>=1.
#endif
ke(cf,V0,packHalf2x16(H2(bf,F0)));
#else
a2(V0);
#endif
#ifndef W
je(o0.x+o0.y+o0.z+o0.w==.0,n0,J1*(1.-o0.w)+o0);
#endif
}a2(m0);F2;
#ifdef W
L1=o0;A3
#else
h2;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive