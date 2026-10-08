#pragma once

#include "draw_path.vert.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_vert[] = R"===(#undef Q2
#ifdef HB
#define Q2 e
#else
#define Q2 D
#endif
#ifdef BB
f1(f0)
#if defined(DB)||defined(FB)
K(0,h4,LB);
#else
K(0,e,XB);K(1,e,YB);
#endif
g1
#endif
w2 F0 X(0,e,O0);
#ifdef FB
F0 X(1,c,T2);
#elif!defined(CB)
#ifdef DB
MB X(1,d,o1);
#else
F0 X(2,Q2,S);
#endif
MB X(3,d,G0);
#endif
#ifdef N
#ifdef FB
MB X(4,d,e4);
#else
MB X(4,D,j2);
#endif
#endif
#if defined(AB)&&!defined(CB)
F0 X(5,e,W0);
#endif
#ifdef H
MB X(6,d,P0);
#endif
#ifdef QB
g3 X(7,S0,y3);X(8,c,J4);
#endif
#ifdef GB
F0 X(9,M,V0);
#endif
l2
#ifdef BB
x1(RB,f0,B,F,r){
#if defined(DB)||defined(FB)
L(F,B,LB,M);
#else
L(F,B,XB,e);L(F,B,YB,e);
#endif
V(O0,e);
#if defined(GB)
V(V0,M);
#endif
#ifdef FB
V(T2,c);
#elif!defined(CB)
#ifdef DB
V(o1,d);
#else
V(S,Q2);
#endif
V(G0,d);
#endif
#ifdef N
#ifdef FB
V(e4,d);
#else
V(j2,D);
#endif
#endif
#if defined(AB)&&!defined(CB)
V(W0,e);
#endif
#ifdef H
V(P0,d);
#endif
#ifdef QB
V(y3,S0);V(J4,c);
#endif
bool rf=false;uint c0;c i0;
#ifdef CB
P C6;
#endif
#ifdef FB
i0=Oc(LB,c0,
#ifdef CB
C6,
#endif
T2 P3);
#elif defined(DB)
i0=Pc(LB,c0
#ifdef CB
,C6
#else
,o1
#endif
P3);
#else
e T;rf=!ia(XB,YB,r,c0,i0
#ifndef CB
,T
#else
,C6
#endif
P3);
#ifndef CB
#ifdef HB
S=T;
#else
S.xy=y8(T.xy);
#endif
#endif
#endif
S0 T0=q5(WC,c0);
#if!defined(FB)&&!defined(CB)
G0=a9(c0,j.p6);if((T0.x&Ba)!=0u) G0=-G0;
#endif
uint j3=T0.x&0xfu;
#ifdef N
if(N){uint Fj=(j3==G8?T0.y:T0.x)>>16;d z1=a9(Fj,j.p6);if(j3==G8) z1=-z1;
#ifdef FB
e4=z1;
#else
j2.x=z1;
#endif
}
#endif
#ifdef H
if(H){P0=float((T0.x>>4)&0xfu);}
#endif
c l0=i0;
#ifdef RD
if(j.ua!=0u){l0.y=float(j.va)-l0.y;}
#endif
#ifdef AB
if(AB){W H3=p1(p0(JB,c0*n2+2u));e W3=p0(JB,c0*n2+3u);
#ifndef CB
W0=A8(H3,W3.xy,l0);
#else
db(H3,W3.xy,l0 e5);
#endif
}
#endif
if(j3==Ca){O0=e(unpackUnorm4x8(T0.y));}
#if defined(N)&&!defined(FB)
else if(N&&j3==G8){d a6=a9(T0.x>>16,j.p6);j2.y=a6;}
#endif
else{W zb=p1(p0(JB,c0*n2));e O7=p0(JB,c0*n2+1u);O0=Rc(l0,zb,O7.xy,float(j3),O7.zw,uintBitsToFloat(T0.y));O0.w=-O0.w;}
#if defined(GB)
if(GB&&(T0.x&Vd)!=0u){W Ab=p1(p0(JB,c0*n2+4u));e P7=p0(JB,c0*n2+5u);c r3=y0(Ab,l0)+P7.xy;float sf=1.+P7.z;if((T0.x&oh)!=0u){uint g4=(T0.x&qh)>>ph;sf=-(1.+float(g4));}V0=M(r3.x,r3.y,sf);}else{V0=M(0.0,0.0,0.0);}
#endif
e I;if(!rf){I=Q3(i0);
#ifdef MC
I.y=-I.y;
#endif
#ifdef CB
I.z=c9(C6,0xffu);
#elif defined(QB)
O k5=p0(KB,c0*4u+3u);y3=k5.xy;J4=i0+uintBitsToFloat(k5.zw);
#endif
}else{I=e(j.h3,j.h3,j.h3,j.h3);}Z(O0);
#if defined(GB)
Z(V0);
#endif
#ifdef FB
Z(T2);
#elif!defined(CB)
#ifdef DB
Z(o1);
#else
Z(S);
#endif
Z(G0);
#endif
#ifdef N
#ifdef FB
Z(e4);
#else
Z(j2);
#endif
#endif
#if defined(AB)&&!defined(CB)
Z(W0);
#endif
#ifdef H
Z(P0);
#endif
#ifdef QB
Z(y3);Z(J4);
#endif
y1(I);}
#endif
#ifdef EB
k4 l4 f d Gj(i xc,uint g4){d tf=dot(xc.xyz,a1(.30,.59,.11));if(g4==rh) return xc.w;if(g4==sh) return 1.-xc.w;if(g4==th) return tf;return 1.-tf;}f i o8(
#ifdef GB
M p8,
#endif
#ifdef H
P X1,
#endif
e l5 f7){
#ifdef H
bool F2=H&&X1!=T3;
#else
const bool F2=false;
#endif
i n;if(l5.w>=.0){n=T4(l5);}else{l5.w=-l5.w;d Fa=i4(fract(l5.w)*(256./255.));l5.w=floor(l5.w)*j.ad+j.g7;c La=fd(l5);n=o2(YC,H8,La,.0);if(!F2){n.xyz*=n.w;n.w*=Fa;}}
#if defined(GB)
if(GB&&p8.z<0.0){return A5(TB,S4,p8.xy,I0(.0));}if(GB&&p8.z>0.0){d Db=p8.z-1.;i O1=A5(TB,S4,p8.xy,Db);if(F2) O1=H0(f6(O1),O1.w);n*=O1;}
#endif
return n;}
#if!defined(DB)&&!defined(FB)
f d uf(Q2 T a4){
#ifdef HB
if(HB&&bd(T)) return R4(T n1);else
#endif
return min(T.x,T.y);}f d vf(Q2 T a4){
#if defined(HB)
if(HB&&cd(T)) return K8(T n1);else
#endif
return T.x;}f d yc(Q2 T a4){if(l6(T)) return uf(T n1);else return vf(T n1);}f d Hj(d m5,Q2 T a4){if(l6(T)){d B0=uf(T n1);return max(B0,m5);}else{d B0=vf(T n1);return m5+B0;}}
#endif
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive