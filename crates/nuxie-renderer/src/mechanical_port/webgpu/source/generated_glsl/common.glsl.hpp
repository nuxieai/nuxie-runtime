#pragma once

#include "common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char common[] = R"===(#define H3 3.14159265359
#define v8 6.28318530718
#define Z6 1.57079632679
#ifndef BB
#define w4 float(.5)
#else
#define w4 float(.0)
#endif
#define Q3(l) r8(l,j.Hf,j.If)
#ifdef NF
#define xc(V,g,a) k5(V,g,a)
#define G4 f
#define ea(q) q
#define c6(q) q
#define fa(q) uintBitsToFloat(q)
#define l5(q) floatBitsToUint(q)
#else
#define xc(V,g,a) H4(V,g,a)
#define G4 Y
#define ea(q) floatBitsToUint(q)
#define c6(q) uintBitsToFloat(q)
#define fa(q) q
#define l5(q) q
#endif
#define Jf(a,l,w8) r1(a,Z(l)+Z(-1,0))w8,r1(a,Z(l)+Z(0,0))w8,r1(a,Z(l)+Z(0,-1))w8,r1(a,Z(l)+Z(-1,-1))w8
#define m5(q) a7(XC,ga,q,yc,float(yc),.0).x
#define Ac(q) a7(XC,ga,q,zc,float(zc),.0).x
#ifdef Bc
e d Z4(float x){return x;}e d d6(uint x){return float(x);}e d Kf(L x){return float(x);}e d ha(int x){return float(x);}e i f5(f xyzw){return xyzw;}e E U7(c xy){return xy;}e i sc(Y xyzw){return vec4(xyzw);}e L e3(d x){return uint(x);}e L a2(uint x){return x;}
#else
e d Z4(float x){return(d)x;}e d d6(uint x){return(d)x;}e d Kf(L x){return(d)x;}e d ha(int x){return(d)x;}e i f5(f xyzw){return(i)xyzw;}e E U7(c xy){return(E)xy;}e i sc(Y xyzw){return(i)xyzw;}e L e3(d x){return(L)x;}e L a2(uint x){return(L)x;}
#endif
e d J0(d x){return x;}e E D2(E xy){return xy;}e E D2(d x,d y){E T;T.x=x,T.y=y;return T;}e E D2(d x){E T;T.x=x,T.y=x;return T;}e c P6(float x){return c(x,x);}e A S0(d x,d y,d z){A T;T.x=x,T.y=y,T.z=z;return T;}e A S0(d x){A T;T.x=x,T.y=x,T.z=x;return T;}e i E0(d x,d y,d z,d w){i T;T.x=x,T.y=y,T.z=z,T.w=w;return T;}e i E0(A xyz,d w){i T;T.xyz=xyz;T.w=w;return T;}e i E0(d x){i T;T.x=x,T.y=x,T.z=x,T.w=x;return T;}e i E0(i x){return x;}e I4 Lf(bool b){return I4(b,b);}e c7 zi(A n,A b,A I1){c7 T;T[0]=n;T[1]=b;T[2]=I1;return T;}e d7 Ai(A n,A b){d7 T;T[0]=n;T[1]=b;return T;}e J4 Bi(i n,i b,i I1,i Mf){J4 T;T[0]=n;T[1]=b;T[2]=I1;T[3]=Mf;return T;}e e0 K1(f x){return e0(x.xy,x.zw);}e uint ec(L x){return x;}e c e6(c n,c b,float t){return(b-n)*t+n;}e d x8(uint Cc,uint f6){return Cc==0u?.0:unpackHalf2x16((Cc+Nf)*f6).x;}e float Dc(c m2){m2=normalize(m2);float f1=acos(clamp(m2.x,-1.,1.));return m2.y>=.0?f1:-f1;}e i Ci(i k){return E0(k.xyz*k.w,k.w);}e A H6(i ia){return ia.xyz*(ia.w!=.0?1./ia.w:.0);}e d k3(E e7){return min(e7.x,e7.y);}e d k3(A Ec){return min(k3(Ec.xy),Ec.z);}e d k3(i Fc){E e7=min(Fc.xy,Fc.zw);d Of=min(e7.x,e7.y);return Of;}e d O5(E f7){return max(f7.x,f7.y);}e d O5(A Gc){return max(O5(Gc.xy),Gc.z);}e d O5(i Hc){E f7=max(Hc.xy,Hc.zw);d Pf=max(f7.x,f7.y);return Pf;}e float H9(c x){return abs(x.x)+abs(x.y);}e d ja(d x,d ka,d la){
#if defined(OF)||defined(GD)
#ifdef GD
if(GD)
#endif
{if(x<la)if(x>ka)return x;else return ka;else return la;}
#endif
return clamp(x,ka,la);}e d Ic(c v0,d E2,d q3){d Qf=fract(0.06711056*v0.x+0.00583715*v0.y);d Rf=fract(52.9829189*Qf);return(Rf*E2)+q3;}
#if 0
e d Di(c v0,float E2,float q3){int x=int(v0.x);int y=int(v0.y);int Jc=(x^y);int b=(y>>1)&1;b|=(Jc&2);b|=(y&1)<<2;b|=(Jc&1)<<3;float Sf=float(b);d Tf=Z4(Sf)/16.0;return(Tf*E2)+q3;}e d Ei(c v0,float E2,float q3){v0.y*=0.5;v0.x=fract(v0.x*0.5+v0.y);v0.y=fract(v0.y);float S3=(v0.y*0.5+v0.x);return(S3*E2)+q3;}
#endif
#ifdef LB
e d ma(c v0,d E2,d q3){return LB?Ic(v0,E2,q3):.0;}e A J2(A k,d g7,c v0,d E2,d q3){return(LB&&g7!=.0)?(Ic(v0,E2,q3)+k):k;}e A J2(A k,d g7,d Kc){return(LB&&g7!=.0)?(Kc+k):k;}
#else
e d ma(c v0,float E2,float q3){return 0.;}e A J2(A k,d g7,c v0,d E2,d q3){return k;}e A J2(A k,d g7,d Kc){return k;}
#endif
#ifdef CB
e f r8(c Lc,float Uf,float Mc){return f(Lc.x*Uf-1.,Lc.y*Mc-sign(Mc),0.,1.);}
#ifndef BB
e f W7(e0 d4,c K4,c na){c oa=abs(d4[0])+abs(d4[1]);if(oa.x!=.0&&oa.y!=.0){c P=1./oa;c n5=O0(d4,na)+K4;const float Vf=.5;return f(n5,-n5)*P.xyxy+P.xyxy+Vf;}else{return K4.xyxy;}}
#else
e float pa(uint qa){return 1.-float(qa)*(2./32768.);}
#ifdef AB
e void Nc(e0 d4,c K4,c na h7){
#ifndef RE
if(any(notEqual(f(d4),f(.0,.0,.0,.0)))){c n5=O0(d4,na)+K4.xy;gl_ClipDistance[0]=n5.x+1.;gl_ClipDistance[1]=n5.y+1.;gl_ClipDistance[2]=1.-n5.x;gl_ClipDistance[3]=1.-n5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=K4.x-.5;}
#endif
}
#endif
#endif
#endif
#ifdef EB
#ifdef ZB
e d o3(d k){return(k<=0.04045)?k/12.92:pow(abs((k+0.055)/1.055),2.4);}e A o3(A k){return S0(o3(k.x),o3(k.y),o3(k.z));}e i o3(i k){return E0(o3(k.xyz),k.w);}
#endif
#endif
#if defined(EB)&&defined(BB)&&!defined(O)
e i ra(J4 i7,int y8){if(y8==0xf){return(i7[0]+i7[1]+i7[2]+i7[3])*.25;}else{i Wf=f(notEqual(y8&g6(1,2,4,8),g6(0,0,0,0)));i T=O0(i7,Wf);int z8=(y8&5)+((y8>>1)&5);z8=(z8&3)+(z8>>2);T*=1./float(z8);return T;}}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive