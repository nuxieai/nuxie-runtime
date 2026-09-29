#pragma once

#include "common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char common[] = R"===(#define D3 3.14159265359
#define p8 6.28318530718
#define X6 1.57079632679
#ifndef CB
#define q4 float(.5)
#else
#define q4 float(.0)
#endif
#define M3(l) o8(l,m.of,m.pf)
#ifdef JF
#define mc(U,f,a) h5(U,f,a)
#define E4 g
#define Y9(q) q
#define Z5(q) q
#define Z9(q) uintBitsToFloat(q)
#define i5(q) floatBitsToUint(q)
#else
#define mc(U,f,a) F4(U,f,a)
#define E4 G
#define Y9(q) floatBitsToUint(q)
#define Z5(q) uintBitsToFloat(q)
#define Z9(q) q
#define i5(q) q
#endif
#define qf(a,l,q8) q1(a,Y(l)+Y(-1,0))q8,q1(a,Y(l)+Y(0,0))q8,q1(a,Y(l)+Y(0,-1))q8,q1(a,Y(l)+Y(-1,-1))q8
#define j5(q) Y6(YC,aa,q,nc,float(nc),.0).x
#define pc(q) Y6(YC,aa,q,oc,float(oc),.0).x
#ifdef qc
e c W4(float x){return x;}e c a6(uint x){return float(x);}e c rf(N x){return float(x);}e c ba(int x){return float(x);}e i c5(g xyzw){return xyzw;}e E R7(d xy){return xy;}e i hc(G xyzw){return vec4(xyzw);}e N c6(c x){return uint(x);}e N X1(uint x){return x;}
#else
e c W4(float x){return(c)x;}e c a6(uint x){return(c)x;}e c rf(N x){return(c)x;}e c ba(int x){return(c)x;}e i c5(g xyzw){return(i)xyzw;}e E R7(d xy){return(E)xy;}e i hc(G xyzw){return(i)xyzw;}e N c6(c x){return(N)x;}e N X1(uint x){return(N)x;}
#endif
e c G0(c x){return x;}e E B2(E xy){return xy;}e E B2(c x,c y){E T;T.x=x,T.y=y;return T;}e E B2(c x){E T;T.x=x,T.y=x;return T;}e d N6(float x){return d(x,x);}e A Q0(c x,c y,c z){A T;T.x=x,T.y=y,T.z=z;return T;}e A Q0(c x){A T;T.x=x,T.y=x,T.z=x;return T;}e i C0(c x,c y,c z,c w){i T;T.x=x,T.y=y,T.z=z,T.w=w;return T;}e i C0(A xyz,c w){i T;T.xyz=xyz;T.w=w;return T;}e i C0(c x){i T;T.x=x,T.y=x,T.z=x,T.w=x;return T;}e i C0(i x){return x;}e G4 sf(bool b){return G4(b,b);}e Z6 fi(A o,A b,A G1){Z6 T;T[0]=o;T[1]=b;T[2]=G1;return T;}e a7 gi(A o,A b){a7 T;T[0]=o;T[1]=b;return T;}e k5 hi(i o,i b,i G1,i tf){k5 T;T[0]=o;T[1]=b;T[2]=G1;T[3]=tf;return T;}e f0 h2(g x){return f0(x.xy,x.zw);}e uint Ub(N x){return x;}e d d6(d o,d b,float t){return(b-o)*t+o;}e c r8(uint rc,uint e6){return rc==0u?.0:unpackHalf2x16((rc+uf)*e6).x;}e float sc(d j2){j2=normalize(j2);float e1=acos(clamp(j2.x,-1.,1.));return j2.y>=.0?e1:-e1;}e i ii(i j){return C0(j.xyz*j.w,j.w);}e A G6(i ca){return ca.xyz*(ca.w!=.0?1./ca.w:.0);}e c h3(E c7){return min(c7.x,c7.y);}e c h3(A tc){return min(h3(tc.xy),tc.z);}e c h3(i uc){E c7=min(uc.xy,uc.zw);c vf=min(c7.x,c7.y);return vf;}e c N5(E d7){return max(d7.x,d7.y);}e c N5(A vc){return max(N5(vc.xy),vc.z);}e c N5(i wc){E d7=max(wc.xy,wc.zw);c wf=max(d7.x,d7.y);return wf;}e float E9(d x){return abs(x.x)+abs(x.y);}e c da(c x,c ea,c fa){
#if defined(KF)||defined(ED)
#ifdef ED
if(ED)
#endif
{if(x<fa)if(x>ea)return x;else return ea;else return fa;}
#endif
return clamp(x,ea,fa);}e c xc(d L0,c C2,c o3){c xf=fract(0.06711056*L0.x+0.00583715*L0.y);c yf=fract(52.9829189*xf);return(yf*C2)+o3;}
#if 0
e c ji(d L0,float C2,float o3){int x=int(L0.x);int y=int(L0.y);int yc=(x^y);int b=(y>>1)&1;b|=(yc&2);b|=(y&1)<<2;b|=(yc&1)<<3;float zf=float(b);c Af=W4(zf)/16.0;return(Af*C2)+o3;}e c ki(d L0,float C2,float o3){L0.y*=0.5;L0.x=fract(L0.x*0.5+L0.y);L0.y=fract(L0.y);float P3=(L0.y*0.5+L0.x);return(P3*C2)+o3;}
#endif
#ifdef MB
e c ga(d L0,c C2,c o3){return MB?xc(L0,C2,o3):.0;}e A F2(A j,c e7,d L0,c C2,c o3){return(MB&&e7!=.0)?(xc(L0,C2,o3)+j):j;}e A F2(A j,c e7,c zc){return(MB&&e7!=.0)?(zc+j):j;}
#else
e c ga(d L0,float C2,float o3){return 0.;}e A F2(A j,c e7,d L0,c C2,c o3){return j;}e A F2(A j,c e7,c zc){return j;}
#endif
#ifdef DB
e g o8(d Ac,float Bf,float Bc){return g(Ac.x*Bf-1.,Ac.y*Bc-sign(Bc),0.,1.);}
#ifndef CB
e g T7(f0 Z3,d H4,d ha){d ia=abs(Z3[0])+abs(Z3[1]);if(ia.x!=.0&&ia.y!=.0){d K=1./ia;d l5=R0(Z3,ha)+H4;const float Cf=.5;return g(l5,-l5)*K.xyxy+K.xyxy+Cf;}else{return H4.xyxy;}}
#else
e float ja(uint ka){return 1.-float(ka)*(2./32768.);}
#ifdef BB
e void Cc(f0 Z3,d H4,d ha f7){
#ifndef OE
if(any(notEqual(g(Z3),g(.0,.0,.0,.0)))){d l5=R0(Z3,ha)+H4.xy;gl_ClipDistance[0]=l5.x+1.;gl_ClipDistance[1]=l5.y+1.;gl_ClipDistance[2]=1.-l5.x;gl_ClipDistance[3]=1.-l5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=H4.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef GB
#ifdef CC
e c m3(c j){return(j<=0.04045)?j/12.92:pow(abs((j+0.055)/1.055),2.4);}e A m3(A j){return Q0(m3(j.x),m3(j.y),m3(j.z));}e i m3(i j){return C0(m3(j.xyz),j.w);}
#endif
#endif
#if defined(GB)&&defined(CB)&&!defined(Q)
e i Dc(k5 g7,int v8){if(v8==0xf){return(g7[0]+g7[1]+g7[2]+g7[3])*.25;}else{i Df=g(notEqual(v8&f6(1,2,4,8),f6(0,0,0,0)));i T=R0(g7,Df);int w8=(v8&5)+((v8>>1)&5);w8=(w8&3)+(w8>>2);T*=1./float(w8);return T;}}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive