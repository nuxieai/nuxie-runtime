#pragma once

#include "common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char common[] = R"===(#define C3 3.14159265359
#define p8 6.28318530718
#define X6 1.57079632679
#ifndef CB
#define q4 float(.5)
#else
#define q4 float(.0)
#endif
#define L3(l) o8(l,m.of,m.pf)
#ifdef IF
#define nc(T,f,a) h5(T,f,a)
#define D4 g
#define X9(q) q
#define Y5(q) q
#define Y9(q) uintBitsToFloat(q)
#define i5(q) floatBitsToUint(q)
#else
#define nc(T,f,a) E4(T,f,a)
#define D4 X
#define X9(q) floatBitsToUint(q)
#define Y5(q) uintBitsToFloat(q)
#define Y9(q) q
#define i5(q) q
#endif
#define qf(a,l,q8) q1(a,Y(l)+Y(-1,0))q8,q1(a,Y(l)+Y(0,0))q8,q1(a,Y(l)+Y(0,-1))q8,q1(a,Y(l)+Y(-1,-1))q8
#define j5(q) Y6(XC,Z9,q,oc,float(oc),.0).x
#define qc(q) Y6(XC,Z9,q,pc,float(pc),.0).x
#ifdef rc
e c W4(float x){return x;}e c Z5(uint x){return float(x);}e c rf(K x){return float(x);}e c aa(int x){return float(x);}e i c5(g xyzw){return xyzw;}e E R7(d xy){return xy;}e i ic(X xyzw){return vec4(xyzw);}e K a6(c x){return uint(x);}e K X1(uint x){return x;}
#else
e c W4(float x){return(c)x;}e c Z5(uint x){return(c)x;}e c rf(K x){return(c)x;}e c aa(int x){return(c)x;}e i c5(g xyzw){return(i)xyzw;}e E R7(d xy){return(E)xy;}e i ic(X xyzw){return(i)xyzw;}e K a6(c x){return(K)x;}e K X1(uint x){return(K)x;}
#endif
e c G0(c x){return x;}e E B2(E xy){return xy;}e E B2(c x,c y){E S;S.x=x,S.y=y;return S;}e E B2(c x){E S;S.x=x,S.y=x;return S;}e d N6(float x){return d(x,x);}e v Q0(c x,c y,c z){v S;S.x=x,S.y=y,S.z=z;return S;}e v Q0(c x){v S;S.x=x,S.y=x,S.z=x;return S;}e i C0(c x,c y,c z,c w){i S;S.x=x,S.y=y,S.z=z,S.w=w;return S;}e i C0(v xyz,c w){i S;S.xyz=xyz;S.w=w;return S;}e i C0(c x){i S;S.x=x,S.y=x,S.z=x,S.w=x;return S;}e i C0(i x){return x;}e F4 sf(bool b){return F4(b,b);}e Z6 gi(v o,v b,v H1){Z6 S;S[0]=o;S[1]=b;S[2]=H1;return S;}e a7 hi(v o,v b){a7 S;S[0]=o;S[1]=b;return S;}e G4 ii(i o,i b,i H1,i tf){G4 S;S[0]=o;S[1]=b;S[2]=H1;S[3]=tf;return S;}e f0 h2(g x){return f0(x.xy,x.zw);}e uint Vb(K x){return x;}e d c6(d o,d b,float t){return(b-o)*t+o;}e c r8(uint sc,uint d6){return sc==0u?.0:unpackHalf2x16((sc+uf)*d6).x;}e float tc(d j2){j2=normalize(j2);float e1=acos(clamp(j2.x,-1.,1.));return j2.y>=.0?e1:-e1;}e i ji(i j){return C0(j.xyz*j.w,j.w);}e v F6(i ba){return ba.xyz*(ba.w!=.0?1./ba.w:.0);}e c h3(E c7){return min(c7.x,c7.y);}e c h3(v uc){return min(h3(uc.xy),uc.z);}e c h3(i vc){E c7=min(vc.xy,vc.zw);c vf=min(c7.x,c7.y);return vf;}e c M5(E d7){return max(d7.x,d7.y);}e c M5(v wc){return max(M5(wc.xy),wc.z);}e c M5(i xc){E d7=max(xc.xy,xc.zw);c wf=max(d7.x,d7.y);return wf;}e float D9(d x){return abs(x.x)+abs(x.y);}e c ca(c x,c da,c ea){
#if defined(JF)||defined(DD)
#ifdef DD
if(DD)
#endif
{if(x<ea)if(x>da)return x;else return da;else return ea;}
#endif
return clamp(x,da,ea);}e c yc(d L0,c C2,c n3){c xf=fract(0.06711056*L0.x+0.00583715*L0.y);c yf=fract(52.9829189*xf);return(yf*C2)+n3;}
#if 0
e c ki(d L0,float C2,float n3){int x=int(L0.x);int y=int(L0.y);int zc=(x^y);int b=(y>>1)&1;b|=(zc&2);b|=(y&1)<<2;b|=(zc&1)<<3;float zf=float(b);c Af=W4(zf)/16.0;return(Af*C2)+n3;}e c li(d L0,float C2,float n3){L0.y*=0.5;L0.x=fract(L0.x*0.5+L0.y);L0.y=fract(L0.y);float N3=(L0.y*0.5+L0.x);return(N3*C2)+n3;}
#endif
#ifdef LB
e c fa(d L0,c C2,c n3){return LB?yc(L0,C2,n3):.0;}e v F2(v j,c e7,d L0,c C2,c n3){return(LB&&e7!=.0)?(yc(L0,C2,n3)+j):j;}e v F2(v j,c e7,c Ac){return(LB&&e7!=.0)?(Ac+j):j;}
#else
e c fa(d L0,float C2,float n3){return 0.;}e v F2(v j,c e7,d L0,c C2,c n3){return j;}e v F2(v j,c e7,c Ac){return j;}
#endif
#ifdef DB
e g o8(d Bc,float Bf,float Cc){return g(Bc.x*Bf-1.,Bc.y*Cc-sign(Cc),0.,1.);}
#ifndef CB
e g T7(f0 Y3,d H4,d ga){d ha=abs(Y3[0])+abs(Y3[1]);if(ha.x!=.0&&ha.y!=.0){d J=1./ha;d k5=R0(Y3,ga)+H4;const float Cf=.5;return g(k5,-k5)*J.xyxy+J.xyxy+Cf;}else{return H4.xyxy;}}
#else
e float ia(uint ja){return 1.-float(ja)*(2./32768.);}
#ifdef BB
e void Dc(f0 Y3,d H4,d ga f7){
#ifndef NE
if(any(notEqual(g(Y3),g(.0,.0,.0,.0)))){d k5=R0(Y3,ga)+H4.xy;gl_ClipDistance[0]=k5.x+1.;gl_ClipDistance[1]=k5.y+1.;gl_ClipDistance[2]=1.-k5.x;gl_ClipDistance[3]=1.-k5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=H4.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef GB
#ifdef AC
e c l3(c j){return(j<=0.04045)?j/12.92:pow(abs((j+0.055)/1.055),2.4);}e v l3(v j){return Q0(l3(j.x),l3(j.y),l3(j.z));}e i l3(i j){return C0(l3(j.xyz),j.w);}
#endif
#endif
#if defined(GB)&&defined(CB)&&!defined(N)
e i ka(G4 g7,int v8){if(v8==0xf){return(g7[0]+g7[1]+g7[2]+g7[3])*.25;}else{i Df=g(notEqual(v8&e6(1,2,4,8),e6(0,0,0,0)));i S=R0(g7,Df);int w8=(v8&5)+((v8>>1)&5);w8=(w8&3)+(w8>>2);S*=1./float(w8);return S;}}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive