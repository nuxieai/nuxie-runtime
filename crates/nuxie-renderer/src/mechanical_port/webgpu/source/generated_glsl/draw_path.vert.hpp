#pragma once

#include "draw_path.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_vert[] = R"===(#undef G2
#ifdef HB
#define G2 e
#else
#define G2 C
#endif
#ifdef BB
c1(d0)
#if defined(DB)||defined(EB)
K(0,d4,MB);
#else
K(0,e,WB);K(1,e,XB);
#endif
d1
#endif
l2 E0 V(0,e,a1);
#ifdef EB
E0 V(1,c,J2);
#elif!defined(CB)
#ifdef DB
KB V(1,d,m1);
#else
E0 V(2,G2,S);
#endif
KB V(3,d,F0);
#endif
#ifdef A
#ifdef EB
KB V(4,d,Z3);
#else
KB V(4,C,l1);
#endif
#endif
#if defined(AB)&&!defined(CB)
E0 V(5,e,R0);
#endif
#ifdef N
KB V(6,d,Q0);
#endif
#ifdef QB
Z2 V(7,O0,r3);V(8,c,G4);
#endif
#ifdef GB
E0 V(9,O,v1);
#endif
e2
#ifdef BB
w1(RB,d0,D,G,r){
#if defined(DB)||defined(EB)
L(G,D,MB,O);
#else
L(G,D,WB,e);L(G,D,XB,e);
#endif
T(a1,e);
#if defined(GB)
T(v1,O);
#endif
#ifdef EB
T(J2,c);
#elif!defined(CB)
#ifdef DB
T(m1,d);
#else
T(S,G2);
#endif
T(F0,d);
#endif
#ifdef A
#ifdef EB
T(Z3,d);
#else
T(l1,C);
#endif
#endif
#if defined(AB)&&!defined(CB)
T(R0,e);
#endif
#ifdef N
T(Q0,d);
#endif
#ifdef QB
T(r3,O0);T(G4,c);
#endif
bool Qe=false;uint a0;c k0;
#ifdef CB
Q C9;
#endif
#ifdef EB
k0=lc(MB,a0,
#ifdef CB
C9,
#endif
J2 H3);
#elif defined(DB)
k0=mc(MB,a0
#ifdef CB
,C9
#else
,m1
#endif
H3);
#else
e U;Qe=!L9(WB,XB,r,a0,k0
#ifndef CB
,U
#else
,C9
#endif
H3);
#ifndef CB
#ifdef HB
S=U;
#else
S.xy=f8(U.xy);
#endif
#endif
#endif
O0 G0=l5(WC,a0);
#if!defined(EB)&&!defined(CB)
F0=l6(a0,j.U4);if((G0.x&fa)!=0u) F0=-F0;
#endif
uint n2=G0.x&0xfu;
#ifdef A
if(A){uint Rb=(n2==o5?G0.y:G0.x)>>16;d X0=l6(Rb,j.U4);if(n2==o5) X0=-X0;
#ifdef EB
Z3=X0;
#else
l1.x=X0;
#endif
}
#endif
#ifdef N
if(N){Q0=float((G0.x>>4)&0xfu);}
#endif
c l0=k0;
#ifdef PD
if(j.X9!=0u){l0.y=float(j.Y9)-l0.y;}
#endif
#ifdef AB
if(AB){Y C3=n1(p0(JB,a0*g2+2u));e Q3=p0(JB,a0*g2+3u);
#ifndef CB
R0=h8(C3,Q3.xy,l0);
#else
Ha(C3,Q3.xy,l0 Z4);
#endif
}
#endif
if(n2==ga){a1=e(unpackUnorm4x8(G0.y));}
#if defined(A)&&!defined(EB)
else if(A&&n2==o5){d F4=l6(G0.x>>16,j.U4);l1.y=F4;}
#endif
else{Y Sb=n1(p0(JB,a0*g2));e V7=p0(JB,a0*g2+1u);a1=Z9(l0,Sb,V7.xy,float(n2),V7.zw,uintBitsToFloat(G0.y));a1.w=-a1.w;}
#if defined(GB)
if(GB&&(G0.x&wd)!=0u){Y Tb=n1(p0(JB,a0*g2+4u));e W7=p0(JB,a0*g2+5u);c o3=M0(Tb,l0)+W7.xy;float Re=1.+W7.z;if((G0.x&Kg)!=0u){uint c4=(G0.x&Mg)>>Lg;Re=-(1.+float(c4));}v1=O(o3.x,o3.y,Re);}else{v1=O(0.0,0.0,0.0);}
#endif
e I;if(!Qe){I=I3(k0);
#ifdef MC
I.y=-I.y;
#endif
#ifdef CB
I.z=H8(C9,0xffu);
#elif defined(QB)
M d5=p0(LB,a0*4u+3u);r3=d5.xy;G4=k0+uintBitsToFloat(d5.zw);
#endif
}else{I=e(j.a3,j.a3,j.a3,j.a3);}Z(a1);
#if defined(GB)
Z(v1);
#endif
#ifdef EB
Z(J2);
#elif!defined(CB)
#ifdef DB
Z(m1);
#else
Z(S);
#endif
Z(F0);
#endif
#ifdef A
#ifdef EB
Z(Z3);
#else
Z(l1);
#endif
#endif
#if defined(AB)&&!defined(CB)
Z(R0);
#endif
#ifdef N
Z(Q0);
#endif
#ifdef QB
Z(r3);Z(G4);
#endif
x1(I);}
#endif
#ifdef FB
g4 h4 f d Yi(i Ub,uint c4){d Se=dot(Ub.xyz,W0(.30,.59,.11));if(c4==Ng) return Ub.w;if(c4==Og) return 1.-Ub.w;if(c4==Pg) return Se;return 1.-Se;}f i X7(
#ifdef GB
O Y7,
#endif
#ifdef N
Q z3,
#endif
e e5 W6){
#ifdef N
bool n5=N&&z3!=M4;
#else
const bool n5=false;
#endif
i p;if(e5.w>=.0){p=v5(e5);}else{e5.w=-e5.w;d ja=e4(fract(e5.w)*(256./255.));e5.w=floor(e5.w)*j.xc+j.yc;c pa=Dc(e5);p=o2(ED,ia,pa,.0);if(!n5){p.xyz*=p.w;p.w*=ja;}}
#if defined(GB)
if(GB&&Y7.z<0.0){return i6(CC,r5,Y7.xy,H0(.0));}if(GB&&Y7.z>0.0){d Zi=Y7.z-1.;i N2=i6(CC,r5,Y7.xy,Zi);if(n5) N2=I0(Q6(N2),N2.w);p*=N2;}
#endif
return p;}
#if!defined(DB)&&!defined(EB)
f d Te(G2 U S3){
#ifdef HB
if(HB&&zc(U)) return N4(U k1);else
#endif
return min(U.x,U.y);}f d Ue(G2 U S3){
#if defined(HB)
if(HB&&Ac(U)) return p8(U k1);else
#endif
return U.x;}f d Vb(G2 U S3){if(f6(U)) return Te(U k1);else return Ue(U k1);}f d aj(d f5,G2 U S3){if(f6(U)){d A0=Te(U k1);return max(A0,f5);}else{d A0=Ue(U k1);return f5+A0;}}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive