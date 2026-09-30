#pragma once

#include "draw_mesh.frag.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_mesh_frag[] = R"===(#ifdef EB
#if(defined(O)&&!defined(I))||defined(QB)
#undef Eb
#else
#define Eb
#endif
L1
#ifndef O
z0(G2,l0);
#endif
#ifndef QB
k1(V2,i0);
#ifndef O
z0(i6,o4);
#endif
k1(L6,R0);
#else
z0(V2,i0);
#endif
M1
#ifdef KB
I3 c3(g5,Z3,GC);J3 h5 a4(Y5)i5 T3 U3
#endif
#ifdef O
#ifdef KB
v2(HB)
#else
v2(HB)
#endif
#else
#ifdef KB
O1(HB)
#else
O1(HB)
#endif
#endif
{
#ifdef FB
r(X1,f);
#if defined(IB)
r(C2,R);
#endif
r(F2,c);
#endif
#ifdef I
r(O3,d);
#endif
#ifdef AB
r(N0,f);
#endif
#if defined(FB)&&defined(S)
r(g1,d);
#endif
#ifdef KB
r(K5,c);r(J1,i);
#ifdef S
r(C1,L);
#endif
#endif
#ifdef FB
i k=N7(
#ifdef IB
C2,
#endif
#ifdef S
e3(g1),
#endif
X1 W2);d o=clamp(j2(ED,T9,F2,.0).x,J0(.0),J0(1.));
#endif
#ifdef KB
i k=C7(GC,Y5,K5,j.Fd);d o=1.;
#endif
#ifdef AB
if(AB){d a5=max(k3(f5(N0)),J0(.0));o=min(a5,o);}
#endif
#ifdef Eb
z2;
#endif
#if defined(I)
if(I&&O3!=.0){d z3;
#ifndef QB
E P0=unpackHalf2x16(Z0(i0));d G6=P0.y;z3=max(G6==O3?P0.x:J0(.0),J0(.0));
#else
z3=K0(i0).x;
#endif
z3=max(z3,J0(.0));o=min(o,z3);}
#endif
#ifdef KB
k*=J1;
#endif
#if!defined(O)
i N1=K0(l0);
#ifdef S
#ifdef FB
L n3=e3(g1);
#endif
#ifdef KB
L n3=C1;
#endif
if(S&&n3!=A4){
#ifdef KB
k.xyz=H6(k);
#endif
k.xyz=X4(k.xyz,N1,n3)*k.w;}
#endif
k*=o;
#ifdef ZB
if(ZB){k=o3(k);}
#endif
k.xyz=J2(k.xyz,k.w,d0.xy,j.F3,j.G3);
#ifndef QB
k=N1*(1.-k.w)+k;
#endif
A0(l0,k);
#endif
#ifndef QB
h2(i0);h2(R0);
#else
A0(i0,E0(.0));
#endif
#ifdef Eb
A2;
#endif
#ifdef O
k=(k*o);k.xyz=J2(k.xyz,k.w,d0.xy,j.F3,j.G3);E1=k;p3
#else
d2;
#endif
}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive