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
#define N3(m) p8(m,l.Ff,l.Gf)
#ifdef OF
#define vc(U,g,a) h5(U,g,a)
#define D4 f
#define ca(q) q
#define a6(q) q
#define da(q) uintBitsToFloat(q)
#define i5(q) floatBitsToUint(q)
#else
#define vc(U,g,a) E4(U,g,a)
#define D4 X
#define ca(q) floatBitsToUint(q)
#define a6(q) uintBitsToFloat(q)
#define da(q) q
#define i5(q) q
#endif
#define Hf(a,m,r8) p1(a,Y(m)+Y(-1,0))r8,p1(a,Y(m)+Y(0,0))r8,p1(a,Y(m)+Y(0,-1))r8,p1(a,Y(m)+Y(-1,-1))r8
#define j5(q) Z6(YC,ea,q,wc,float(wc),.0).x
#define yc(q) Z6(YC,ea,q,xc,float(xc),.0).x
#ifdef zc
e d W4(float x){return x;}e d c6(uint x){return float(x);}e d If(L x){return float(x);}e d fa(int x){return float(x);}e i c5(f xyzw){return xyzw;}e E S7(c xy){return xy;}e i qc(X xyzw){return vec4(xyzw);}e L d6(d x){return uint(x);}e L Y1(uint x){return x;}
#else
e d W4(float x){return(d)x;}e d c6(uint x){return(d)x;}e d If(L x){return(d)x;}e d fa(int x){return(d)x;}e i c5(f xyzw){return(i)xyzw;}e E S7(c xy){return(E)xy;}e i qc(X xyzw){return(i)xyzw;}e L d6(d x){return(L)x;}e L Y1(uint x){return(L)x;}
#endif
e d I0(d x){return x;}e E D2(E xy){return xy;}e E D2(d x,d y){E S;S.x=x,S.y=y;return S;}e E D2(d x){E S;S.x=x,S.y=x;return S;}e c O6(float x){return c(x,x);}e A R0(d x,d y,d z){A S;S.x=x,S.y=y,S.z=z;return S;}e A R0(d x){A S;S.x=x,S.y=x,S.z=x;return S;}e i D0(d x,d y,d z,d w){i S;S.x=x,S.y=y,S.z=z,S.w=w;return S;}e i D0(A xyz,d w){i S;S.xyz=xyz;S.w=w;return S;}e i D0(d x){i S;S.x=x,S.y=x,S.z=x,S.w=x;return S;}e i D0(i x){return x;}e F4 Jf(bool b){return F4(b,b);}e a7 xi(A n,A b,A G1){a7 S;S[0]=n;S[1]=b;S[2]=G1;return S;}e c7 yi(A n,A b){c7 S;S[0]=n;S[1]=b;return S;}e G4 zi(i n,i b,i G1,i Kf){G4 S;S[0]=n;S[1]=b;S[2]=G1;S[3]=Kf;return S;}e d0 I1(f x){return d0(x.xy,x.zw);}e uint cc(L x){return x;}e c e6(c n,c b,float t){return(b-n)*t+n;}e d v8(uint Ac,uint f6){return Ac==0u?.0:unpackHalf2x16((Ac+Lf)*f6).x;}e float Bc(c l2){l2=normalize(l2);float e1=acos(clamp(l2.x,-1.,1.));return l2.y>=.0?e1:-e1;}e i Ai(i j){return D0(j.xyz*j.w,j.w);}e A H6(i ga){return ga.xyz*(ga.w!=.0?1./ga.w:.0);}e d i3(E d7){return min(d7.x,d7.y);}e d i3(A Cc){return min(i3(Cc.xy),Cc.z);}e d i3(i Dc){E d7=min(Dc.xy,Dc.zw);d Mf=min(d7.x,d7.y);return Mf;}e d M5(E e7){return max(e7.x,e7.y);}e d M5(A Ec){return max(M5(Ec.xy),Ec.z);}e d M5(i Fc){E e7=max(Fc.xy,Fc.zw);d Nf=max(e7.x,e7.y);return Nf;}e float F9(c x){return abs(x.x)+abs(x.y);}e d ha(d x,d ia,d ja){
#if defined(PF)||defined(HD)
#ifdef HD
if(HD)
#endif
{if(x<ja)if(x>ia)return x;else return ia;else return ja;}
#endif
return clamp(x,ia,ja);}e d Gc(c q0,d E2,d n3){d Of=fract(0.06711056*q0.x+0.00583715*q0.y);d Pf=fract(52.9829189*Of);return(Pf*E2)+n3;}
#if 0
e d Bi(c q0,float E2,float n3){int x=int(q0.x);int y=int(q0.y);int Hc=(x^y);int b=(y>>1)&1;b|=(Hc&2);b|=(y&1)<<2;b|=(Hc&1)<<3;float Qf=float(b);d Rf=W4(Qf)/16.0;return(Rf*E2)+n3;}e d Ci(c q0,float E2,float n3){q0.y*=0.5;q0.x=fract(q0.x*0.5+q0.y);q0.y=fract(q0.y);float P3=(q0.y*0.5+q0.x);return(P3*E2)+n3;}
#endif
#ifdef LB
e d ka(c q0,d E2,d n3){return LB?Gc(q0,E2,n3):.0;}e A J2(A j,d f7,c q0,d E2,d n3){return(LB&&f7!=.0)?(Gc(q0,E2,n3)+j):j;}e A J2(A j,d f7,d Ic){return(LB&&f7!=.0)?(Ic+j):j;}
#else
e d ka(c q0,float E2,float n3){return 0.;}e A J2(A j,d f7,c q0,d E2,d n3){return j;}e A J2(A j,d f7,d Ic){return j;}
#endif
#ifdef DB
e f p8(c Jc,float Sf,float Kc){return f(Jc.x*Sf-1.,Jc.y*Kc-sign(Kc),0.,1.);}
#ifndef CB
e f U7(d0 a4,c H4,c la){c ma=abs(a4[0])+abs(a4[1]);if(ma.x!=.0&&ma.y!=.0){c P=1./ma;c k5=N0(a4,la)+H4;const float Tf=.5;return f(k5,-k5)*P.xyxy+P.xyxy+Tf;}else{return H4.xyxy;}}
#else
e float na(uint oa){return 1.-float(oa)*(2./32768.);}
#ifdef BB
e void Lc(d0 a4,c H4,c la g7){
#ifndef SE
if(any(notEqual(f(a4),f(.0,.0,.0,.0)))){c k5=N0(a4,la)+H4.xy;gl_ClipDistance[0]=k5.x+1.;gl_ClipDistance[1]=k5.y+1.;gl_ClipDistance[2]=1.-k5.x;gl_ClipDistance[3]=1.-k5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=H4.x-.5;}
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
e i pa(G4 h7,int w8){if(w8==0xf){return(h7[0]+h7[1]+h7[2]+h7[3])*.25;}else{i Uf=f(notEqual(w8&g6(1,2,4,8),g6(0,0,0,0)));i S=N0(h7,Uf);int x8=(w8&5)+((w8>>1)&5);x8=(x8&3)+(x8>>2);S*=1./float(x8);return S;}}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive