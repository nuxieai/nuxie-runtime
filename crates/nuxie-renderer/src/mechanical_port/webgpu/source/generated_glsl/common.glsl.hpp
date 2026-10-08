#pragma once

#include "common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char common[] = R"===(#define n4 3.14159265359
#define Y8 6.28318530718
#define r7 1.57079632679
#ifndef CB
#define M4 float(.5)
#else
#define M4 float(.0)
#endif
#define Q3(o) X8(o,j.Dg,j.Eg)
#define Fg(a,o,Z8) r1(a,g0(o)+g0(-1,0)) Z8,r1(a,g0(o)+g0(0,0)) Z8,r1(a,g0(o)+g0(0,-1)) Z8,r1(a,g0(o)+g0(-1,-1)) Z8
#define C5(E) v7(ZC,Ta,E,wd,float(wd),.0).x
#define yd(E) v7(ZC,Ta,E,xd,float(xd),.0).x
#ifdef Ua
f d i4(float x){return x;}f d D5(uint x){return float(x);}f d Gg(P x){return float(x);}f d Va(int x){return float(x);}f i T4(e xyzw){return xyzw;}f D y8(c xy){return xy;}f i sd(O xyzw){return vec4(xyzw);}f P X2(d x){return uint(x);}f P T1(uint x){return x;}
#else
f d i4(float x){return(d) x;}f d D5(uint x){return(d) x;}f d Gg(P x){return(d) x;}f d Va(int x){return(d) x;}f i T4(e xyzw){return(i) xyzw;}f D y8(c xy){return(D) xy;}f i sd(O xyzw){return(i) xyzw;}f P X2(d x){return(P) x;}f P T1(uint x){return(P) x;}
#endif
f d I0(d x){return x;}f D R2(D xy){return xy;}f D R2(d x,d y){D Y;Y.x=x,Y.y=y;return Y;}f D R2(d x){D Y;Y.x=x,Y.y=x;return Y;}f c i7(float x){return c(x,x);}f v a1(d x,d y,d z){v Y;Y.x=x,Y.y=y,Y.z=z;return Y;}f v a1(d x){v Y;Y.x=x,Y.y=x,Y.z=x;return Y;}f i H0(d x,d y,d z,d w){i Y;Y.x=x,Y.y=y,Y.z=z,Y.w=w;return Y;}f i H0(v xyz,d w){i Y;Y.xyz=xyz;Y.w=w;return Y;}f i H0(d x){i Y;Y.x=x,Y.y=x,Y.z=x,Y.w=x;return Y;}f i H0(i x){return x;}f Y4 Hg(bool b){return Y4(b,b);}f w7 Rj(v k,v b,v S1){w7 Y;Y[0]=k;Y[1]=b;Y[2]=S1;return Y;}f x7 Sj(v k,v b){x7 Y;Y[0]=k;Y[1]=b;return Y;}f Z4 Tj(i k,i b,i S1,i Ig){Z4 Y;Y[0]=k;Y[1]=b;Y[2]=S1;Y[3]=Ig;return Y;}f W p1(e x){return W(x.xy,x.zw);}f uint ed(P x){return x;}f c o6(c k,c b,float t){return(b-k)*t+k;}f d a9(uint zd,uint p6){return zd==0u?.0:unpackHalf2x16((zd+Jg)*p6).x;}f float Ad(c A2){A2=normalize(A2);float h1=acos(clamp(A2.x,-1.,1.));return A2.y>=.0?h1:-h1;}f i Uj(i n){return H0(n.xyz*n.w,n.w);}f v f6(i Wa){return Wa.xyz*(Wa.w!=.0?1./Wa.w:.0);}f d A3(D y7){return min(y7.x,y7.y);}f d A3(v Bd){return min(A3(Bd.xy),Bd.z);}f d A3(i Cd){D y7=min(Cd.xy,Cd.zw);d Kg=min(y7.x,y7.y);return Kg;}f d e6(D z7){return max(z7.x,z7.y);}f d e6(v Dd){return max(e6(Dd.xy),Dd.z);}f d e6(i Ed){D z7=max(Ed.xy,Ed.zw);d Lg=max(z7.x,z7.y);return Lg;}f float sa(c x){return abs(x.x)+abs(x.y);}f d Xa(d x,d Ya,d Za){
#if defined(PF)||defined(ID)
#ifdef ID
if(ID)
#endif
{if(x<Za) if(x>Ya) return x;else return Ya;else return Za;}
#endif
return clamp(x,Ya,Za);}f d Fd(c l0,d S2,d G3){d Mg=fract(0.06711056*l0.x+0.00583715*l0.y);d Ng=fract(52.9829189*Mg);return(Ng*S2)+G3;}
#if 0
f d Vj(c l0,float S2,float G3){int x=int(l0.x);int y=int(l0.y);int Gd=(x^y);int b=(y>>1)&1;b|=(Gd&2);b|=(y&1)<<2;b|=(Gd&1)<<3;float Og=float(b);d Pg=i4(Og)/16.0;return(Pg*S2)+G3;}f d Wj(c l0,float S2,float G3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float j4=(l0.y*0.5+l0.x);return(j4*S2)+G3;}
#endif
#ifdef OB
f d ab(c l0,d S2,d G3){return OB?Fd(l0,S2,G3):.0;}f v I2(v n,d A7,c l0,d S2,d G3){return(OB&&A7!=.0)?(Fd(l0,S2,G3)+n):n;}f v I2(v n,d A7,d Hd){return(OB&&A7!=.0)?(Hd+n):n;}
#else
f d ab(c l0,float S2,float G3){return 0.;}f v I2(v n,d A7,c l0,d S2,d G3){return n;}f v I2(v n,d A7,d Hd){return n;}
#endif
#ifdef BB
f e X8(c Id,float Qg,float Jd){return e(Id.x*Qg-1.,Id.y*Jd-sign(Jd),0.,1.);}
#ifndef CB
f e A8(W H3,c W3,c bb){c cb=abs(H3[0])+abs(H3[1]);if(cb.x!=.0&&cb.y!=.0){c R=1./cb;c E5=y0(H3,bb)+W3;const float Rg=.5;return e(E5,-E5)*R.xyxy+R.xyxy+Rg;}else{return W3.xyxy;}}
#else
f float c9(uint Sg,uint Tg){float Kd=float((Sg<<Ug)|Tg);
#if defined(Ua)&&!defined(DC)
return Kd*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return Kd*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef AB
f void db(W H3,c W3,c bb B7){
#ifndef RE
if(any(notEqual(e(H3),e(.0,.0,.0,.0)))){c E5=y0(H3,bb)+W3.xy;gl_ClipDistance[0]=E5.x+1.;gl_ClipDistance[1]=E5.y+1.;gl_ClipDistance[2]=1.-E5.x;gl_ClipDistance[3]=1.-E5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=W3.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(EB)&&defined(CB)&&!defined(U)
f i eb(Z4 C7,int d9){if(d9==0xf){return(C7[0]+C7[1]+C7[2]+C7[3])*.25;}else{i Vg=e(notEqual(d9&q6(1,2,4,8),q6(0,0,0,0)));i Y=y0(C7,Vg);int e9=(d9&5)+((d9>>1)&5);e9=(e9&3)+(e9>>2);Y*=1./float(e9);return Y;}}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive