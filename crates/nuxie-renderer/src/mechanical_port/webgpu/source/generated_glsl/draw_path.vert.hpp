#pragma once

#include "draw_path.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_vert[] = R"===(#undef H2
#ifdef HB
#define H2 f
#else
#define H2 C
#endif
#ifdef BB
c1(d0)
#if defined(DB)||defined(FB)
K(0,d4,MB);
#else
K(0,f,WB);K(1,f,XB);
#endif
d1
#endif
l2 E0 W(0,f,a1);
#ifdef FB
E0 W(1,c,K2);
#elif!defined(CB)
#ifdef DB
KB W(1,d,m1);
#else
E0 W(2,H2,S);
#endif
KB W(3,d,F0);
#endif
#ifdef A
#ifdef FB
KB W(4,d,Z3);
#else
KB W(4,C,l1);
#endif
#endif
#if defined(AB)&&!defined(CB)
E0 W(5,f,S0);
#endif
#ifdef O
KB W(6,d,Q0);
#endif
#ifdef QB
a3 W(7,O0,q3);W(8,c,G4);
#endif
#ifdef GB
E0 W(9,P,r1);
#endif
e2
#ifdef BB
v1(RB,d0,D,G,r){
#if defined(DB)||defined(FB)
L(G,D,MB,P);
#else
L(G,D,WB,f);L(G,D,XB,f);
#endif
T(a1,f);
#if defined(GB)
T(r1,P);
#endif
#ifdef FB
T(K2,c);
#elif!defined(CB)
#ifdef DB
T(m1,d);
#else
T(S,H2);
#endif
T(F0,d);
#endif
#ifdef A
#ifdef FB
T(Z3,d);
#else
T(l1,C);
#endif
#endif
#if defined(AB)&&!defined(CB)
T(S0,f);
#endif
#ifdef O
T(Q0,d);
#endif
#ifdef QB
T(q3,O0);T(G4,c);
#endif
bool Pe=false;uint a0;c k0;
#ifdef CB
R C9;
#endif
#ifdef FB
k0=lc(MB,a0,
#ifdef CB
C9,
#endif
K2 H3);
#elif defined(DB)
k0=mc(MB,a0
#ifdef CB
,C9
#else
,m1
#endif
H3);
#else
f U;Pe=!L9(WB,XB,r,a0,k0
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
S.xy=h8(U.xy);
#endif
#endif
#endif
O0 H0=m5(XC,a0);
#if!defined(FB)&&!defined(CB)
F0=m6(a0,j.U4);if((H0.x&fa)!=0u) F0=-F0;
#endif
uint n2=H0.x&0xfu;
#ifdef A
if(A){uint Rb=(n2==p5?H0.y:H0.x)>>16;d X0=m6(Rb,j.U4);if(n2==p5) X0=-X0;
#ifdef FB
Z3=X0;
#else
l1.x=X0;
#endif
}
#endif
#ifdef O
if(O){Q0=float((H0.x>>4)&0xfu);}
#endif
c l0=k0;
#ifdef QD
if(j.X9!=0u){l0.y=float(j.Y9)-l0.y;}
#endif
#ifdef AB
if(AB){Y C3=n1(p0(JB,a0*g2+2u));f Q3=p0(JB,a0*g2+3u);
#ifndef CB
S0=j8(C3,Q3.xy,l0);
#else
Ha(C3,Q3.xy,l0 Z4);
#endif
}
#endif
if(n2==ga){a1=f(unpackUnorm4x8(H0.y));}
#if defined(A)&&!defined(FB)
else if(A&&n2==p5){d F4=m6(H0.x>>16,j.U4);l1.y=F4;}
#endif
else{Y Sb=n1(p0(JB,a0*g2));f X7=p0(JB,a0*g2+1u);a1=Z9(l0,Sb,X7.xy,float(n2),X7.zw,uintBitsToFloat(H0.y));a1.w=-a1.w;}
#if defined(GB)
if(GB&&(H0.x&wd)!=0u){Y Tb=n1(p0(JB,a0*g2+4u));f Y7=p0(JB,a0*g2+5u);c o3=M0(Tb,l0)+Y7.xy;float Qe=1.+Y7.z;if((H0.x&Jg)!=0u){uint c4=(H0.x&Lg)>>Kg;Qe=-(1.+float(c4));}r1=P(o3.x,o3.y,Qe);}else{r1=P(0.0,0.0,0.0);}
#endif
f I;if(!Pe){I=I3(k0);
#ifdef NC
I.y=-I.y;
#endif
#ifdef CB
I.z=J8(C9,0xffu);
#elif defined(QB)
N e5=p0(LB,a0*4u+3u);q3=e5.xy;G4=k0+uintBitsToFloat(e5.zw);
#endif
}else{I=f(j.c3,j.c3,j.c3,j.c3);}Z(a1);
#if defined(GB)
Z(r1);
#endif
#ifdef FB
Z(K2);
#elif!defined(CB)
#ifdef DB
Z(m1);
#else
Z(S);
#endif
Z(F0);
#endif
#ifdef A
#ifdef FB
Z(Z3);
#else
Z(l1);
#endif
#endif
#if defined(AB)&&!defined(CB)
Z(S0);
#endif
#ifdef O
Z(Q0);
#endif
#ifdef QB
Z(q3);Z(G4);
#endif
w1(I);}
#endif
#ifdef EB
g4 h4 e d Wi(i Ub,uint c4){d Re=dot(Ub.xyz,R0(.30,.59,.11));if(c4==Mg) return Ub.w;if(c4==Ng) return 1.-Ub.w;if(c4==Og) return Re;return 1.-Re;}e i Z7(
#ifdef GB
P a8,
#endif
#ifdef O
R y3,
#endif
f f5 X6){
#ifdef O
bool o5=O&&y3!=M4;
#else
const bool o5=false;
#endif
i l;if(f5.w>=.0){l=w5(f5);}else{f5.w=-f5.w;d ja=e4(fract(f5.w)*(256./255.));f5.w=floor(f5.w)*j.xc+j.yc;c pa=Dc(f5);l=o2(FD,ia,pa,.0);if(!o5){l.xyz*=l.w;l.w*=ja;}}
#if defined(GB)
if(GB&&a8.z<0.0){return j6(DC,v5,a8.xy,I0(.0));}if(GB&&a8.z>0.0){d Xi=a8.z-1.;i p2=j6(DC,v5,a8.xy,Xi);if(o5) p2=G0(R6(p2),p2.w);l*=p2;}
#endif
return l;}
#if!defined(DB)&&!defined(FB)
e d Se(H2 U S3){
#ifdef HB
if(HB&&zc(U)) return N4(U k1);else
#endif
return min(U.x,U.y);}e d Te(H2 U S3){
#if defined(HB)
if(HB&&Ac(U)) return r8(U k1);else
#endif
return U.x;}e d Vb(H2 U S3){if(g6(U)) return Se(U k1);else return Te(U k1);}e d Yi(d g5,H2 U S3){if(g6(U)){d A0=Se(U k1);return max(A0,g5);}else{d A0=Te(U k1);return g5+A0;}}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive