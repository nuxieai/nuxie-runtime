#pragma once

#include "draw_path.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_vert[] = R"===(#undef G5
#ifdef AG
#define G5 true
#elif defined(AB)
#define G5 AB
#else
#define G5 false
#endif
#undef z2
#ifdef HB
#define z2 g
#else
#define z2 E
#endif
#ifdef DB
g1(e0)
#if defined(EB)||defined(FB)
L(0,N3,LB);
#else
L(0,g,VB);L(1,g,WB);
#endif
h1
#endif
l2 H0 X(0,g,f1);
#ifdef FB
H0 X(1,d,D2);
#elif!defined(CB)
#ifdef EB
NB X(1,c,i1);
#else
H0 X(2,z2,O);
#endif
NB X(3,c,B0);
#endif
#ifdef I
#ifdef FB
NB X(4,c,K3);
#else
NB X(4,E,U1);
#endif
#endif
#if defined(BB)&&!defined(CB)
H0 X(5,g,M0);
#endif
#ifdef AB
NB X(6,c,e2);
#endif
#ifdef TB
Q2 X(7,a1,f3);X(8,d,n4);
#endif
#ifdef KB
H0 X(9,R,A2);
#endif
f2
#ifdef DB
#ifdef HD
layout(push_constant)uniform Ui{float Lh;}Mh;
#endif
y1(HC,e0,F,B,v){
#if defined(EB)||defined(FB)
M(B,F,LB,R);
#else
M(B,F,VB,g);M(B,F,WB,g);
#endif
V(f1,g);
#if defined(KB)
V(A2,R);
#endif
#ifdef FB
V(D2,d);
#elif!defined(CB)
#ifdef EB
V(i1,c);
#else
V(O,z2);
#endif
V(B0,c);
#endif
#ifdef I
#ifdef FB
V(K3,c);
#else
V(U1,E);
#endif
#endif
#if defined(BB)&&!defined(CB)
V(M0,g);
#endif
#ifdef AB
V(e2,c);
#endif
#ifdef TB
V(f3,a1);V(n4,d);
#endif
bool ee=false;uint l0;d m0;
#ifdef CB
N g9;
#endif
#ifdef FB
m0=Fb(LB,l0,
#ifdef CB
g9,
#endif
D2 w3);
#elif defined(EB)
m0=Gb(LB,l0
#ifdef CB
,g9
#else
,i1
#endif
w3);
#else
g P;ee=!q9(VB,WB,v,l0,m0
#ifndef CB
,P
#else
,g9
#endif
w3);
#ifndef CB
#ifdef HB
O=P;
#else
O.xy=Q7(P.xy);
#endif
#endif
#endif
a1 p1=O5(BD,l0);
#if!defined(FB)&&!defined(CB)
B0=q8(l0,m.c6);if((p1.x&K9)!=0u)B0=-B0;
#endif
uint l3=p1.x&0xfu;
#ifdef I
if(I){uint Nh=(l3==Y7?p1.y:p1.x)>>16;c k1=q8(Nh,m.c6);if(l3==Y7)k1=-k1;
#ifdef FB
K3=k1;
#else
U1.x=k1;
#endif
}
#endif
#ifdef AB
if(AB){e2=float((p1.x>>4)&0xfu);}
#endif
d L0=m0;
#ifdef BG
L0.y=float(m.Lg)-L0.y;
#endif
#ifdef BB
if(BB){f0 Z3=g2(J0(RB,l0*A3+2u));g G4=J0(RB,l0*A3+3u);
#ifndef CB
M0=S7(Z3,G4.xy,L0);
#else
Ac(Z3,G4.xy,L0 w5);
#endif
}
#endif
if(l3==Nb){i j=unpackUnorm4x8(p1.y);if(G5){}else{j.xyz*=j.w;}f1=g(j);}
#if defined(I)&&!defined(FB)
else if(I&&l3==Y7){c H5=q8(p1.x>>16,m.c6);U1.y=H5;}
#endif
else{f0 mb=g2(J0(RB,l0*A3));g nb=J0(RB,l0*A3+1u);d x4=R0(mb,L0)+nb.xy;if(l3==M9||l3==Hf){f1.w=-uintBitsToFloat(p1.y);float Oh=nb.z;if(Oh>.9){f1.z=2.;}else{f1.z=nb.w;}if(l3==M9){f1.y=.0;f1.x=x4.x;}else{f1.z=-f1.z;f1.xy=x4.xy;}}}
#ifdef HD
if(HD){f1*=Mh.Lh;}
#endif
#if defined(KB)
if(KB&&(p1.x&If)!=0u){f0 mb=g2(J0(RB,l0*A3+4u));g fe=J0(RB,l0*A3+5u);d x4=R0(mb,L0)+fe.xy;A2=R(x4.x,x4.y,1.+fe.z);}else{A2=R(0.0,0.0,0.0);}
#endif
g W;if(!ee){W=M3(m0);
#ifdef SC
W.y=-W.y;
#endif
#ifdef CB
W.z=ja(g9);
#elif defined(TB)
G P4=J0(QB,l0*4u+3u);f3=P4.xy;n4=m0+uintBitsToFloat(P4.zw);
#endif
}else{W=g(m.R2,m.R2,m.R2,m.R2);}a0(f1);
#if defined(KB)
a0(A2);
#endif
#ifdef FB
a0(D2);
#elif!defined(CB)
#ifdef EB
a0(i1);
#else
a0(O);
#endif
a0(B0);
#endif
#ifdef I
#ifdef FB
a0(K3);
#else
a0(U1);
#endif
#endif
#if defined(BB)&&!defined(CB)
a0(M0);
#endif
#ifdef AB
a0(e2);
#endif
#ifdef TB
a0(f3);a0(n4);
#endif
z1(W);}
#endif
#ifdef GB
Q3 R3 e i L7(g I5,
#ifdef KB
R ob,
#endif
float n J6){i j;if(I5.w>=.0){j=Z4(I5);if(G5)j.w*=n;else j*=n;}else{float t=I5.z>.0?I5.x:length(I5.xy);t=clamp(t,.0,1.);float ge=abs(I5.z);float x=ge>1.?(1.-1./la)*t+(.5/la):(1./la)*t+ge;float Ph=-I5.w;j=n2(MD,Ob,d(x,Ph),.0);j.w*=n;if(G5){}else{j.xyz*=j.w;}}
#if defined(KB)
if(KB&&ob.z>0.0){c Qh=ob.z-1.;i G2=S6(JC,U5,ob.xy,Qh);if(G5)G2=C0(E6(G2),G2.w);j*=G2;}
#endif
return j;}
#if!defined(EB)&&!defined(FB)
e c he(z2 P I3){
#ifdef HB
if(HB&&Pb(P))return y4(P d1);else
#endif
return min(P.x,P.y);}e c ie(z2 P I3){
#if defined(HB)
if(HB&&Qb(P))return c8(P d1);else
#endif
return P.x;}e c pb(z2 P I3){if(T5(P))return he(P d1);else return ie(P d1);}e c Rh(c Q4,z2 P I3){if(T5(P)){c r0=he(P d1);return max(r0,Q4);}else{c r0=ie(P d1);return Q4+r0;}}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive