#pragma once

#include "common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char common[] = R"===(#define E3 3.14159265359
#define q8 6.28318530718
#define Y6 1.57079632679
#ifndef CB
#define r4 float(.5)
#else
#define r4 float(.0)
#endif
#define N3(n) p8(n,l.Gf,l.Hf)
#ifdef OF
#define wc(U,g,a) g5(U,g,a)
#define D4 f
#define da(q) q
#define Y5(q) q
#define ea(q) uintBitsToFloat(q)
#define h5(q) floatBitsToUint(q)
#else
#define wc(U,g,a) E4(U,g,a)
#define D4 X
#define da(q) floatBitsToUint(q)
#define Y5(q) uintBitsToFloat(q)
#define ea(q) q
#define h5(q) q
#endif
#define If(a,n,r8) p1(a,Y(n)+Y(-1,0))r8,p1(a,Y(n)+Y(0,0))r8,p1(a,Y(n)+Y(0,-1))r8,p1(a,Y(n)+Y(-1,-1))r8
#define i5(q) Z6(YC,fa,q,xc,float(xc),.0).x
#define zc(q) Z6(YC,fa,q,yc,float(yc),.0).x
#ifdef Ac
e d W4(float x){return x;}e d Z5(uint x){return float(x);}e d Jf(L x){return float(x);}e d ga(int x){return float(x);}e i a5(f xyzw){return xyzw;}e E S7(c xy){return xy;}e i rc(X xyzw){return vec4(xyzw);}e L a6(d x){return uint(x);}e L Y1(uint x){return x;}
#else
e d W4(float x){return(d)x;}e d Z5(uint x){return(d)x;}e d Jf(L x){return(d)x;}e d ga(int x){return(d)x;}e i a5(f xyzw){return(i)xyzw;}e E S7(c xy){return(E)xy;}e i rc(X xyzw){return(i)xyzw;}e L a6(d x){return(L)x;}e L Y1(uint x){return(L)x;}
#endif
e d I0(d x){return x;}e E C2(E xy){return xy;}e E C2(d x,d y){E S;S.x=x,S.y=y;return S;}e E C2(d x){E S;S.x=x,S.y=x;return S;}e c O6(float x){return c(x,x);}e A R0(d x,d y,d z){A S;S.x=x,S.y=y,S.z=z;return S;}e A R0(d x){A S;S.x=x,S.y=x,S.z=x;return S;}e i D0(d x,d y,d z,d w){i S;S.x=x,S.y=y,S.z=z,S.w=w;return S;}e i D0(A xyz,d w){i S;S.xyz=xyz;S.w=w;return S;}e i D0(d x){i S;S.x=x,S.y=x,S.z=x,S.w=x;return S;}e i D0(i x){return x;}e F4 Kf(bool b){return F4(b,b);}e a7 yi(A m,A b,A G1){a7 S;S[0]=m;S[1]=b;S[2]=G1;return S;}e c7 zi(A m,A b){c7 S;S[0]=m;S[1]=b;return S;}e G4 Ai(i m,i b,i G1,i Lf){G4 S;S[0]=m;S[1]=b;S[2]=G1;S[3]=Lf;return S;}e d0 I1(f x){return d0(x.xy,x.zw);}e uint dc(L x){return x;}e c c6(c m,c b,float t){return(b-m)*t+m;}e d v8(uint Bc,uint d6){return Bc==0u?.0:unpackHalf2x16((Bc+Mf)*d6).x;}e float Cc(c l2){l2=normalize(l2);float e1=acos(clamp(l2.x,-1.,1.));return l2.y>=.0?e1:-e1;}e i Bi(i j){return D0(j.xyz*j.w,j.w);}e A G6(i ha){return ha.xyz*(ha.w!=.0?1./ha.w:.0);}e d i3(E d7){return min(d7.x,d7.y);}e d i3(A Dc){return min(i3(Dc.xy),Dc.z);}e d i3(i Ec){E d7=min(Ec.xy,Ec.zw);d Nf=min(d7.x,d7.y);return Nf;}e d K5(E e7){return max(e7.x,e7.y);}e d K5(A Fc){return max(K5(Fc.xy),Fc.z);}e d K5(i Gc){E e7=max(Gc.xy,Gc.zw);d Of=max(e7.x,e7.y);return Of;}e float G9(c x){return abs(x.x)+abs(x.y);}e d ia(d x,d ja,d ka){
#if defined(PF)||defined(HD)
#ifdef HD
if(HD)
#endif
{if(x<ka)if(x>ja)return x;else return ja;else return ka;}
#endif
return clamp(x,ja,ka);}e d Hc(c r0,d D2,d n3){d Pf=fract(0.06711056*r0.x+0.00583715*r0.y);d Qf=fract(52.9829189*Pf);return(Qf*D2)+n3;}
#if 0
e d Ci(c r0,float D2,float n3){int x=int(r0.x);int y=int(r0.y);int Ic=(x^y);int b=(y>>1)&1;b|=(Ic&2);b|=(y&1)<<2;b|=(Ic&1)<<3;float Rf=float(b);d Sf=W4(Rf)/16.0;return(Sf*D2)+n3;}e d Di(c r0,float D2,float n3){r0.y*=0.5;r0.x=fract(r0.x*0.5+r0.y);r0.y=fract(r0.y);float P3=(r0.y*0.5+r0.x);return(P3*D2)+n3;}
#endif
#ifdef LB
e d la(c r0,d D2,d n3){return LB?Hc(r0,D2,n3):.0;}e A I2(A j,d f7,c r0,d D2,d n3){return(LB&&f7!=.0)?(Hc(r0,D2,n3)+j):j;}e A I2(A j,d f7,d Jc){return(LB&&f7!=.0)?(Jc+j):j;}
#else
e d la(c r0,float D2,float n3){return 0.;}e A I2(A j,d f7,c r0,d D2,d n3){return j;}e A I2(A j,d f7,d Jc){return j;}
#endif
#ifdef DB
e f p8(c Kc,float Tf,float Lc){return f(Kc.x*Tf-1.,Kc.y*Lc-sign(Lc),0.,1.);}
#ifndef CB
e f U7(d0 a4,c H4,c ma){c na=abs(a4[0])+abs(a4[1]);if(na.x!=.0&&na.y!=.0){c P=1./na;c j5=N0(a4,ma)+H4;const float Uf=.5;return f(j5,-j5)*P.xyxy+P.xyxy+Uf;}else{return H4.xyxy;}}
#else
e float oa(uint pa){return 1.-float(pa)*(2./32768.);}
#ifdef BB
e void Mc(d0 a4,c H4,c ma g7){
#ifndef SE
if(any(notEqual(f(a4),f(.0,.0,.0,.0)))){c j5=N0(a4,ma)+H4.xy;gl_ClipDistance[0]=j5.x+1.;gl_ClipDistance[1]=j5.y+1.;gl_ClipDistance[2]=1.-j5.x;gl_ClipDistance[3]=1.-j5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=H4.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef FB
#ifdef AC
e d l3(d j){return(j<=0.04045)?j/12.92:pow(abs((j+0.055)/1.055),2.4);}e A l3(A j){return R0(l3(j.x),l3(j.y),l3(j.z));}e i l3(i j){return D0(l3(j.xyz),j.w);}
#endif
#endif
#if defined(FB)&&defined(CB)&&!defined(O)
e i qa(G4 h7,int w8){if(w8==0xf){return(h7[0]+h7[1]+h7[2]+h7[3])*.25;}else{i Vf=f(notEqual(w8&e6(1,2,4,8),e6(0,0,0,0)));i S=N0(h7,Vf);int x8=(w8&5)+((w8>>1)&5);x8=(x8&3)+(x8>>2);S*=1./float(x8);return S;}}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive