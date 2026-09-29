#pragma once

#include "draw_path.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_vert[] = R"===(#undef I5
#ifdef DG
#define I5 true
#elif defined(AB)
#define I5 AB
#else
#define I5 false
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
#ifdef J
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
#ifdef SB
Q2 X(7,a1,f3);X(8,d,o4);
#endif
#ifdef KB
H0 X(9,R,A2);
#endif
g2
#ifdef DB
#ifdef ID
Bd(gh)Cd(float,Rh)Dd(Sh)
#endif
z1(HC,e0,F,B,v){
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
#ifdef J
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
#ifdef SB
V(f3,a1);V(o4,d);
#endif
bool je=false;uint l0;d m0;
#ifdef CB
N h9;
#endif
#ifdef FB
m0=Ib(LB,l0,
#ifdef CB
h9,
#endif
D2 w3);
#elif defined(EB)
m0=Jb(LB,l0
#ifdef CB
,h9
#else
,i1
#endif
w3);
#else
g P;je=!r9(VB,WB,v,l0,m0
#ifndef CB
,P
#else
,h9
#endif
w3);
#ifndef CB
#ifdef HB
O=P;
#else
O.xy=S7(P.xy);
#endif
#endif
#endif
a1 p1=Q5(BD,l0);
#if!defined(FB)&&!defined(CB)
B0=v8(l0,m.e6);if((p1.x&L9)!=0u)B0=-B0;
#endif
uint l3=p1.x&0xfu;
#ifdef J
if(J){uint Th=(l3==a8?p1.y:p1.x)>>16;c k1=v8(Th,m.e6);if(l3==a8)k1=-k1;
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
#ifdef EG
L0.y=float(m.Qg)-L0.y;
#endif
#ifdef BB
if(BB){f0 Z3=h2(J0(RB,l0*A3+2u));g I4=J0(RB,l0*A3+3u);
#ifndef CB
M0=U7(Z3,I4.xy,L0);
#else
Dc(Z3,I4.xy,L0 y5);
#endif
}
#endif
if(l3==Qb){i j=unpackUnorm4x8(p1.y);if(I5){}else{j.xyz*=j.w;}f1=g(j);}
#if defined(J)&&!defined(FB)
else if(J&&l3==a8){c J5=v8(p1.x>>16,m.e6);V1.y=J5;}
#endif
else{f0 pb=h2(J0(RB,l0*A3));g qb=J0(RB,l0*A3+1u);d y4=R0(pb,L0)+qb.xy;if(l3==N9||l3==Mf){f1.w=-uintBitsToFloat(p1.y);float Uh=qb.z;if(Uh>.9){f1.z=2.;}else{f1.z=qb.w;}if(l3==N9){f1.y=.0;f1.x=y4.x;}else{f1.z=-f1.z;f1.xy=y4.xy;}}}
#ifdef ID
if(ID){f1*=Sh.Rh;}
#endif
#if defined(KB)
if(KB&&(p1.x&Nf)!=0u){f0 pb=h2(J0(RB,l0*A3+4u));g ke=J0(RB,l0*A3+5u);d y4=R0(pb,L0)+ke.xy;A2=R(y4.x,y4.y,1.+ke.z);}else{A2=R(0.0,0.0,0.0);}
#endif
g W;if(!je){W=M3(m0);
#ifdef SC
W.y=-W.y;
#endif
#ifdef CB
W.z=ka(h9);
#elif defined(SB)
H S4=J0(QB,l0*4u+3u);f3=S4.xy;o4=m0+uintBitsToFloat(S4.zw);
#endif
}else{W=g(m.R2,m.R2,m.R2,m.R2);}c0(f1);
#if defined(KB)
c0(A2);
#endif
#ifdef FB
c0(D2);
#elif!defined(CB)
#ifdef EB
c0(i1);
#else
c0(O);
#endif
c0(B0);
#endif
#ifdef J
#ifdef FB
c0(K3);
#else
c0(V1);
#endif
#endif
#if defined(BB)&&!defined(CB)
c0(M0);
#endif
#ifdef AB
c0(f2);
#endif
#ifdef SB
c0(f3);c0(o4);
#endif
A1(W);}
#endif
#ifdef GB
Q3 R3 e i N7(g K5,
#ifdef KB
R rb,
#endif
float n M6){i j;if(K5.w>=.0){j=d5(K5);if(I5)j.w*=n;else j*=n;}else{float t=K5.z>.0?K5.x:length(K5.xy);t=clamp(t,.0,1.);float le=abs(K5.z);float x=le>1.?(1.-1./oa)*t+(.5/oa):(1./oa)*t+le;float Vh=-K5.w;j=o2(ND,Rb,d(x,Vh),.0);j.w*=n;if(I5){}else{j.xyz*=j.w;}}
#if defined(KB)
if(KB&&rb.z>0.0){c Wh=rb.z-1.;i G2=V6(JC,W5,rb.xy,Wh);if(I5)G2=C0(H6(G2),G2.w);j*=G2;}
#endif
return j;}
#if!defined(EB)&&!defined(FB)
e c me(z2 P I3){
#ifdef HB
if(HB&&Sb(P))return z4(P d1);else
#endif
return min(P.x,P.y);}e c ne(z2 P I3){
#if defined(HB)
if(HB&&Tb(P))return e8(P d1);else
#endif
return P.x;}e c sb(z2 P I3){if(V5(P))return me(P d1);else return ne(P d1);}e c Xh(c T4,z2 P I3){if(V5(P)){c v0=me(P d1);return max(v0,T4);}else{c v0=ne(P d1);return T4+v0;}}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive