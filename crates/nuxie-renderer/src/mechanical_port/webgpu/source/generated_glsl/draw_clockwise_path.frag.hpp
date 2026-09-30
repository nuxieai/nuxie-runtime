#pragma once

#include "draw_clockwise_path.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_clockwise_path_frag[] = R"===(#ifdef EB
M1
#ifndef Q
z0(G2,m0);
#endif
k1(X2,i0);
#ifndef Q
db(j6,G6);
#endif
k1(M6,S0);N1
#ifdef Q
v2(HB)
#else
P1(HB)
#endif
{r(X1,f);
#ifdef IB
r(C2,R);
#endif
#ifdef DB
r(j1,d);
#else
r(O,B2);
#endif
r(D0,d);
#ifdef I
r(Y1,E);
#endif
#ifdef AB
r(O0,f);
#endif
#ifdef S
r(g1,d);
#endif
d y0=
#ifdef DB
j1;
#else
yb(O);
#endif
i k0;d I1;
#if defined(DB)&&defined(CC)
if(!CC)
#endif
{k0=O7(
#ifdef IB
C2,
#endif
#ifdef S
g3(g1),
#endif
X1 Y2);I1=1.;
#ifdef AB
if(AB){d Db=m3(g5(O0));I1=min(Db,I1);}
#endif
}z2;
#if defined(DB)&&defined(CC)
if(CC){d1(S0,packHalf2x16(D2(y0,D0)));
#ifndef Q
y2(m0);
#endif
}else
#endif
{E V4=unpackHalf2x16(a1(S0));d m9=V4.y;d X4=m9==D0?V4.x:J0(.0);d Me=
#ifndef DB
Y5(O)?max(X4,y0):
#endif
X4+y0;
#ifdef I
if(I&&Y1.x!=.0){E Q0=unpackHalf2x16(a1(i0));d O5=Q0.y;d Eb=O5==Y1.x?Q0.x:J0(.0);I1=min(Eb,I1);}
#endif
I1=max(I1,.0);d e2=ka(X4,.0,I1);d H1=ka(Me,.0,I1);
#ifdef LB
d N5;if(LB){N5=na(d0.xy,j.F3,j.G3);}
#endif
#ifndef Q
i O1=K0(m0);
#ifdef S
if(S&&g1!=e6(B4)){if(H1!=.0){if(e2==.0){k0.xyz=Z4(k0.xyz,O1,g3(g1));
#ifndef DB
if(H1<I1){A S7=k0.xyz;
#ifdef LB
if(LB){S7+=N5*j.Ld;}
#endif
A0(G6,E0(S7,0.0));}
#endif
}else{k0.xyz=K0(G6).xyz;y2(G6);}}k0.xyz*=k0.w;}
#endif
#endif
k0*=N8(e2,H1,k0.w);
#ifdef LB
k0.xyz=K2(k0.xyz,k0.w,N5);
#endif
#ifndef DB
#ifdef S
#define Ne (!S||g1==e6(B4))&&k0.w>=1.
#else
#define Ne k0.w>=1.
#endif
be(Ne,S0,packHalf2x16(D2(Me,D0)));
#else
h2(S0);
#endif
#ifndef Q
ae(k0.x+k0.y+k0.z+k0.w==.0,m0,O1*(1.-k0.w)+k0);
#endif
}h2(i0);A2;
#ifdef Q
F1=k0;r3
#else
d2;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive