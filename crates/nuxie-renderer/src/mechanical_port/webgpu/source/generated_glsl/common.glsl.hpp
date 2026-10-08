#pragma once

#include "common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char common[] = R"===(#define n4 3.14159265359
#define Z8 6.28318530718
#define r7 1.57079632679
#ifndef CB
#define M4 float(.5)
#else
#define M4 float(.0)
#endif
#define Q3(o) Y8(o,j.Gg,j.Hg)
#define Ig(a,o,a9) r1(a,g0(o)+g0(-1,0)) a9,r1(a,g0(o)+g0(0,0)) a9,r1(a,g0(o)+g0(0,-1)) a9,r1(a,g0(o)+g0(-1,-1)) a9
#define C5(E) v7(ZC,Va,E,yd,float(yd),.0).x
#define Ad(E) v7(ZC,Va,E,zd,float(zd),.0).x
#ifdef Wa
f d i4(float x){return x;}f d D5(uint x){return float(x);}f d Jg(P x){return float(x);}f d Xa(int x){return float(x);}f i T4(e xyzw){return xyzw;}f D z8(c xy){return xy;}f i ud(O xyzw){return vec4(xyzw);}f P X2(d x){return uint(x);}f P T1(uint x){return x;}
#else
f d i4(float x){return(d) x;}f d D5(uint x){return(d) x;}f d Jg(P x){return(d) x;}f d Xa(int x){return(d) x;}f i T4(e xyzw){return(i) xyzw;}f D z8(c xy){return(D) xy;}f i ud(O xyzw){return(i) xyzw;}f P X2(d x){return(P) x;}f P T1(uint x){return(P) x;}
#endif
f d J0(d x){return x;}f D R2(D xy){return xy;}f D R2(d x,d y){D Y;Y.x=x,Y.y=y;return Y;}f D R2(d x){D Y;Y.x=x,Y.y=x;return Y;}f c i7(float x){return c(x,x);}f v a1(d x,d y,d z){v Y;Y.x=x,Y.y=y,Y.z=z;return Y;}f v a1(d x){v Y;Y.x=x,Y.y=x,Y.z=x;return Y;}f i H0(d x,d y,d z,d w){i Y;Y.x=x,Y.y=y,Y.z=z,Y.w=w;return Y;}f i H0(v xyz,d w){i Y;Y.xyz=xyz;Y.w=w;return Y;}f i H0(d x){i Y;Y.x=x,Y.y=x,Y.z=x,Y.w=x;return Y;}f i H0(i x){return x;}f Y4 Kg(bool b){return Y4(b,b);}f w7 Wj(v k,v b,v S1){w7 Y;Y[0]=k;Y[1]=b;Y[2]=S1;return Y;}f x7 Xj(v k,v b){x7 Y;Y[0]=k;Y[1]=b;return Y;}f Z4 Yj(i k,i b,i S1,i Lg){Z4 Y;Y[0]=k;Y[1]=b;Y[2]=S1;Y[3]=Lg;return Y;}f W p1(e x){return W(x.xy,x.zw);}f uint gd(P x){return x;}f c o6(c k,c b,float t){return(b-k)*t+k;}f d c9(uint Bd,uint p6){return Bd==0u?.0:unpackHalf2x16((Bd+Mg)*p6).x;}f float Cd(c A2){A2=normalize(A2);float h1=acos(clamp(A2.x,-1.,1.));return A2.y>=.0?h1:-h1;}f i Zj(i n){return H0(n.xyz*n.w,n.w);}f v f6(i Ya){return Ya.xyz*(Ya.w!=.0?1./Ya.w:.0);}f d A3(D y7){return min(y7.x,y7.y);}f d A3(v Dd){return min(A3(Dd.xy),Dd.z);}f d A3(i Ed){D y7=min(Ed.xy,Ed.zw);d Ng=min(y7.x,y7.y);return Ng;}f d e6(D z7){return max(z7.x,z7.y);}f d e6(v Fd){return max(e6(Fd.xy),Fd.z);}f d e6(i Gd){D z7=max(Gd.xy,Gd.zw);d Og=max(z7.x,z7.y);return Og;}f float ua(c x){return abs(x.x)+abs(x.y);}f d Za(d x,d ab,d bb){
#if defined(QF)||defined(JD)
#ifdef JD
if(JD)
#endif
{if(x<bb) if(x>ab) return x;else return ab;else return bb;}
#endif
return clamp(x,ab,bb);}f d Hd(c l0,d S2,d G3){d Pg=fract(0.06711056*l0.x+0.00583715*l0.y);d Qg=fract(52.9829189*Pg);return(Qg*S2)+G3;}
#if 0
f d ak(c l0,float S2,float G3){int x=int(l0.x);int y=int(l0.y);int Id=(x^y);int b=(y>>1)&1;b|=(Id&2);b|=(y&1)<<2;b|=(Id&1)<<3;float Rg=float(b);d Sg=i4(Rg)/16.0;return(Sg*S2)+G3;}f d bk(c l0,float S2,float G3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float j4=(l0.y*0.5+l0.x);return(j4*S2)+G3;}
#endif
#ifdef OB
f d cb(c l0,d S2,d G3){return OB?Hd(l0,S2,G3):.0;}f v I2(v n,d A7,c l0,d S2,d G3){return(OB&&A7!=.0)?(Hd(l0,S2,G3)+n):n;}f v I2(v n,d A7,d Jd){return(OB&&A7!=.0)?(Jd+n):n;}
#else
f d cb(c l0,float S2,float G3){return 0.;}f v I2(v n,d A7,c l0,d S2,d G3){return n;}f v I2(v n,d A7,d Jd){return n;}
#endif
#ifdef BB
f e Y8(c Kd,float Tg,float Ld){return e(Kd.x*Tg-1.,Kd.y*Ld-sign(Ld),0.,1.);}
#ifndef CB
f e B8(W H3,c W3,c db){c eb=abs(H3[0])+abs(H3[1]);if(eb.x!=.0&&eb.y!=.0){c R=1./eb;c E5=y0(H3,db)+W3;const float Ug=.5;return e(E5,-E5)*R.xyxy+R.xyxy+Ug;}else{return W3.xyxy;}}
#else
f float d9(uint Vg,uint Wg){float Md=float((Vg<<Xg)|Wg);
#if defined(Wa)&&!defined(DC)
return Md*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return Md*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef AB
f void fb(W H3,c W3,c db B7){
#ifndef SE
if(any(notEqual(e(H3),e(.0,.0,.0,.0)))){c E5=y0(H3,db)+W3.xy;gl_ClipDistance[0]=E5.x+1.;gl_ClipDistance[1]=E5.y+1.;gl_ClipDistance[2]=1.-E5.x;gl_ClipDistance[3]=1.-E5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=W3.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(EB)&&defined(CB)&&!defined(U)
f i gb(Z4 C7,int e9){if(e9==0xf){return(C7[0]+C7[1]+C7[2]+C7[3])*.25;}else{i Yg=e(notEqual(e9&q6(1,2,4,8),q6(0,0,0,0)));i Y=y0(C7,Yg);int f9=(e9&5)+((e9>>1)&5);f9=(f9&3)+(f9>>2);Y*=1./float(f9);return Y;}}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive