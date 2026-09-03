#pragma once

#include "draw_path.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_vert[] = R"===(#undef H5
#ifdef AG
#define H5 true
#elif defined(AB)
#define H5 AB
#else
#define H5 false
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
m2 H0 X(0,g,f1);
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
NB X(4,E,V1);
#endif
#endif
#if defined(BB)&&!defined(CB)
H0 X(5,g,M0);
#endif
#ifdef AB
NB X(6,c,f2);
#endif
#ifdef TB
Q2 X(7,a1,f3);X(8,d,n4);
#endif
#ifdef KB
H0 X(9,R,A2);
#endif
g2
#ifdef DB
#ifdef HD
layout(push_constant)uniform Wi{float Mh;}Nh;
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
V(V1,E);
#endif
#endif
#if defined(BB)&&!defined(CB)
V(M0,g);
#endif
#ifdef AB
V(f2,c);
#endif
#ifdef TB
V(f3,a1);V(n4,d);
#endif
bool fe=false;uint l0;d m0;
#ifdef CB
N g9;
#endif
#ifdef FB
m0=Gb(LB,l0,
#ifdef CB
g9,
#endif
D2 w3);
#elif defined(EB)
m0=Hb(LB,l0
#ifdef CB
,g9
#else
,i1
#endif
w3);
#else
g P;fe=!q9(VB,WB,v,l0,m0
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
O.xy=R7(P.xy);
#endif
#endif
#endif
a1 p1=P5(BD,l0);
#if!defined(FB)&&!defined(CB)
B0=r8(l0,m.d6);if((p1.x&K9)!=0u)B0=-B0;
#endif
uint l3=p1.x&0xfu;
#ifdef I
if(I){uint Oh=(l3==Z7?p1.y:p1.x)>>16;c k1=r8(Oh,m.d6);if(l3==Z7)k1=-k1;
#ifdef FB
K3=k1;
#else
V1.x=k1;
#endif
}
#endif
#ifdef AB
if(AB){f2=float((p1.x>>4)&0xfu);}
#endif
d L0=m0;
#ifdef BG
L0.y=float(m.Mg)-L0.y;
#endif
#ifdef BB
if(BB){f0 Z3=h2(J0(RB,l0*A3+2u));g G4=J0(RB,l0*A3+3u);
#ifndef CB
M0=T7(Z3,G4.xy,L0);
#else
Bc(Z3,G4.xy,L0 x5);
#endif
}
#endif
if(l3==Ob){i j=unpackUnorm4x8(p1.y);if(H5){}else{j.xyz*=j.w;}f1=g(j);}
#if defined(I)&&!defined(FB)
else if(I&&l3==Z7){c I5=r8(p1.x>>16,m.d6);V1.y=I5;}
#endif
else{f0 nb=h2(J0(RB,l0*A3));g ob=J0(RB,l0*A3+1u);d x4=R0(nb,L0)+ob.xy;if(l3==M9||l3==If){f1.w=-uintBitsToFloat(p1.y);float Ph=ob.z;if(Ph>.9){f1.z=2.;}else{f1.z=ob.w;}if(l3==M9){f1.y=.0;f1.x=x4.x;}else{f1.z=-f1.z;f1.xy=x4.xy;}}}
#ifdef HD
if(HD){f1*=Nh.Mh;}
#endif
#if defined(KB)
if(KB&&(p1.x&Jf)!=0u){f0 nb=h2(J0(RB,l0*A3+4u));g ge=J0(RB,l0*A3+5u);d x4=R0(nb,L0)+ge.xy;A2=R(x4.x,x4.y,1.+ge.z);}else{A2=R(0.0,0.0,0.0);}
#endif
g W;if(!fe){W=M3(m0);
#ifdef SC
W.y=-W.y;
#endif
#ifdef CB
W.z=ja(g9);
#elif defined(TB)
G Q4=J0(QB,l0*4u+3u);f3=Q4.xy;n4=m0+uintBitsToFloat(Q4.zw);
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
a0(V1);
#endif
#endif
#if defined(BB)&&!defined(CB)
a0(M0);
#endif
#ifdef AB
a0(f2);
#endif
#ifdef TB
a0(f3);a0(n4);
#endif
z1(W);}
#endif
#ifdef GB
Q3 R3 e i M7(g J5,
#ifdef KB
R pb,
#endif
float n K6){i j;if(J5.w>=.0){j=a5(J5);if(H5)j.w*=n;else j*=n;}else{float t=J5.z>.0?J5.x:length(J5.xy);t=clamp(t,.0,1.);float he=abs(J5.z);float x=he>1.?(1.-1./ma)*t+(.5/ma):(1./ma)*t+he;float Qh=-J5.w;j=o2(MD,Pb,d(x,Qh),.0);j.w*=n;if(H5){}else{j.xyz*=j.w;}}
#if defined(KB)
if(KB&&pb.z>0.0){c Rh=pb.z-1.;i G2=T6(JC,V5,pb.xy,Rh);if(H5)G2=C0(F6(G2),G2.w);j*=G2;}
#endif
return j;}
#if!defined(EB)&&!defined(FB)
e c ie(z2 P I3){
#ifdef HB
if(HB&&Qb(P))return y4(P d1);else
#endif
return min(P.x,P.y);}e c je(z2 P I3){
#if defined(HB)
if(HB&&Rb(P))return d8(P d1);else
#endif
return P.x;}e c qb(z2 P I3){if(U5(P))return ie(P d1);else return je(P d1);}e c Sh(c R4,z2 P I3){if(U5(P)){c v0=ie(P d1);return max(v0,R4);}else{c v0=je(P d1);return R4+v0;}}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive