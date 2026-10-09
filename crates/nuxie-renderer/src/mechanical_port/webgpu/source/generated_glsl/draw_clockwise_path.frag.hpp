#pragma once

#include "draw_clockwise_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_path_frag[] = R"===(#ifdef EB
U1
#ifndef U
C0(T2,n0);
#endif
p1(j3,m0);
#ifndef U
ec(A6,c7);
#endif
p1(h7,Y0);V1
#ifdef U
G2(IB)
#else
X1(IB)
#endif
{q(O0,f);
#ifdef GB
q(U0,M);
#endif
#ifdef DB
q(n1,d);
#else
q(S,P2);
#endif
q(G0,d);
#ifdef N
q(i2,D);
#endif
#ifdef AB
q(V0,f);
#endif
#ifdef H
q(P0,d);
#endif
d A0=
#ifdef DB
n1;
#else
Gc(S);
#endif
i o0;d Q1;
#if defined(DB)&&defined(EC)
if(!EC)
#endif
{o0=r8(
#ifdef GB
U0,
#endif
#ifdef H
W2(P0),
#endif
O0 l3);Q1=1.;
#ifdef AB
if(AB){d Lc=B3(V4(V0));Q1=min(Lc,Q1);}
#endif
}N2;
#if defined(DB)&&defined(EC)
if(EC){l1(Y0,packHalf2x16(Q2(A0,G0)));
#ifndef U
M2(n0);
#endif
}else
#endif
{D n5=unpackHalf2x16(j1(Y0));d ia=n5.y;d o5=ia==G0?n5.x:J0(.0);d Ff=
#ifndef DB
o6(S)?max(o5,A0):
#endif
o5+A0;
#ifdef N
if(N&&i2.x!=.0){D W0=unpackHalf2x16(j1(m0));d g6=W0.y;d Mc=g6==i2.x?W0.x:J0(.0);Q1=min(Mc,Q1);}
#endif
Q1=max(Q1,.0);d p2=eb(o5,.0,Q1);d O1=eb(Ff,.0,Q1);
#ifdef OB
d f6;if(OB){f6=hb(d0.xy,j.F3,j.G3);}
#endif
#ifndef U
i z1=Q0(n0);
#ifdef H
if(H&&P0!=r6(U3)){if(O1!=.0){if(p2==.0){o0.xyz=N4(o0.xyz,z1,W2(P0));
#ifndef DB
if(O1<Q1){v z8=o0.xyz;
#ifdef OB
if(OB){z8+=f6*j.Fe;}
#endif
y0(c7,H0(z8,0.0));}
#endif
}else{o0.xyz=Q0(c7).xyz;M2(c7);}}o0.xyz*=o0.w;}
#endif
#endif
o0*=J9(p2,O1,o0.w);
#ifdef OB
o0.xyz=I2(o0.xyz,o0.w,f6);
#endif
#ifndef DB
#ifdef H
#define Gf (!H||P0==r6(U3))&&o0.w>=1.
#else
#define Gf o0.w>=1.
#endif
Se(Gf,Y0,packHalf2x16(Q2(Ff,G0)));
#else
g2(Y0);
#endif
#ifndef U
Re(o0.x+o0.y+o0.z+o0.w==.0,n0,z1*(1.-o0.w)+o0);
#endif
}g2(m0);O2;
#ifdef U
L1=o0;E3
#else
o2;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive