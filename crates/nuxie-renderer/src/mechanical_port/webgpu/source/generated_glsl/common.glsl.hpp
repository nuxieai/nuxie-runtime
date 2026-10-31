#pragma once

#include "common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char common[] = R"===(#define w4 3.14159265359
#define A7 6.28318530718
#define B7 1.57079632679
#ifndef CB
#define S4 float(.5)
#else
#define S4 float(.0)
#endif
#define U3(o) h9(o,j.Sg,j.Tg)
#define Ug(a,o,i9) v1(a,f0(o)+f0(-1,0)) i9,v1(a,f0(o)+f0(0,0)) i9,v1(a,f0(o)+f0(0,-1)) i9,v1(a,f0(o)+f0(-1,-1)) i9
#define F5(F) C7(ZC,fb,F,Kd,float(Kd),.0).x
#define Md(F) C7(ZC,fb,F,Ld,float(Ld),.0).x
#ifdef gb
f d T4(float x){return x;}f d r6(uint x){return float(x);}f d Vg(O x){return float(x);}f d hb(int x){return float(x);}f i Y4(e xyzw){return xyzw;}f E F8(c xy){return xy;}f i Gd(R xyzw){return vec4(xyzw);}f O Y2(d x){return uint(x);}f O X1(uint x){return x;}
#else
f d T4(float x){return(d) x;}f d r6(uint x){return(d) x;}f d Vg(O x){return(d) x;}f d hb(int x){return(d) x;}f i Y4(e xyzw){return(i) xyzw;}f E F8(c xy){return(E) xy;}f i Gd(R xyzw){return(i) xyzw;}f O Y2(d x){return(O) x;}f O X1(uint x){return(O) x;}
#endif
f d C0(d x){return x;}f E U2(E xy){return xy;}f E U2(d x,d y){E Y;Y.x=x,Y.y=y;return Y;}f E U2(d x){E Y;Y.x=x,Y.y=x;return Y;}f c o7(float x){return c(x,x);}f v Y0(d x,d y,d z){v Y;Y.x=x,Y.y=y,Y.z=z;return Y;}f v Y0(d x){v Y;Y.x=x,Y.y=x,Y.z=x;return Y;}f i E0(d x,d y,d z,d w){i Y;Y.x=x,Y.y=y,Y.z=z,Y.w=w;return Y;}f i E0(v xyz,d w){i Y;Y.xyz=xyz;Y.w=w;return Y;}f i E0(d x){i Y;Y.x=x,Y.y=x,Y.z=x,Y.w=x;return Y;}f i E0(i x){return x;}f g5 Wg(bool b){return g5(b,b);}f D7 gk(v l,v b,v W1){D7 Y;Y[0]=l;Y[1]=b;Y[2]=W1;return Y;}f E7 hk(v l,v b){E7 Y;Y[0]=l;Y[1]=b;return Y;}f h5 ik(i l,i b,i W1,i Xg){h5 Y;Y[0]=l;Y[1]=b;Y[2]=W1;Y[3]=Xg;return Y;}f X p1(e x){return X(x.xy,x.zw);}f uint sd(O x){return x;}f c v6(c l,c b,float t){return(b-l)*t+l;}f d j9(uint Nd,uint w6){return Nd==0u?.0:unpackHalf2x16((Nd+Yg)*w6).x;}f float ib(c D2){D2=normalize(D2);float f1=acos(clamp(D2.x,-1.,1.));return D2.y>=.0?f1:-f1;}f i jk(i k){return E0(k.xyz*k.w,k.w);}f v j6(i jb){return jb.xyz*(jb.w!=.0?1./jb.w:.0);}f d D3(E F7){return min(F7.x,F7.y);}f d D3(v Od){return min(D3(Od.xy),Od.z);}f d D3(i Pd){E F7=min(Pd.xy,Pd.zw);d Zg=min(F7.x,F7.y);return Zg;}f d i6(E G7){return max(G7.x,G7.y);}f d i6(v Qd){return max(i6(Qd.xy),Qd.z);}f d i6(i Rd){E G7=max(Rd.xy,Rd.zw);d ah=max(G7.x,G7.y);return ah;}f float Ca(c x){return abs(x.x)+abs(x.y);}f d kb(d x,d lb,d mb){
#if defined(SF)||defined(KD)
#ifdef KD
if(KD)
#endif
{if(x<mb) if(x>lb) return x;else return lb;else return mb;}
#endif
return clamp(x,lb,mb);}f d Sd(c l0,d V2,d K3){d bh=fract(0.06711056*l0.x+0.00583715*l0.y);d ch=fract(52.9829189*bh);return(ch*V2)+K3;}
#if 0
f d kk(c l0,float V2,float K3){int x=int(l0.x);int y=int(l0.y);int Td=(x^y);int b=(y>>1)&1;b|=(Td&2);b|=(y&1)<<2;b|=(Td&1)<<3;float dh=float(b);d eh=T4(dh)/16.0;return(eh*V2)+K3;}f d lk(c l0,float V2,float K3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float k4=(l0.y*0.5+l0.x);return(k4*V2)+K3;}
#endif
#ifdef OB
f d nb(c l0,d V2,d K3){return OB?Sd(l0,V2,K3):.0;}f v K2(v k,d H7,c l0,d V2,d K3){return(OB&&H7!=.0)?(Sd(l0,V2,K3)+k):k;}f v K2(v k,d H7,d Ud){return(OB&&H7!=.0)?(Ud+k):k;}
#else
f d nb(c l0,float V2,float K3){return 0.;}f v K2(v k,d H7,c l0,d V2,d K3){return k;}f v K2(v k,d H7,d Ud){return k;}
#endif
#ifdef BB
f e h9(c Vd,float fh,float Wd){return e(Vd.x*fh-1.,Vd.y*Wd-sign(Wd),0.,1.);}
#ifndef CB
f e H8(X L3,c a4,c ob){c pb=abs(L3[0])+abs(L3[1]);if(pb.x!=.0&&pb.y!=.0){c U=1./pb;c G5=D0(L3,ob)+a4;const float gh=.5;return e(G5,-G5)*U.xyxy+U.xyxy+gh;}else{return a4.xyxy;}}
#else
f float k9(uint hh,uint ih){float Xd=float((hh<<jh)|ih);
#if defined(gb)&&!defined(EC)
return Xd*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return Xd*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef AB
f void qb(X L3,c a4,c ob x6){
#ifndef WE
if(any(notEqual(e(L3),e(.0,.0,.0,.0)))){c G5=D0(L3,ob)+a4.xy;gl_ClipDistance[0]=G5.x+1.;gl_ClipDistance[1]=G5.y+1.;gl_ClipDistance[2]=1.-G5.x;gl_ClipDistance[3]=1.-G5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=a4.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(DB)&&defined(CB)&&!defined(Q)
f i rb(h5 I7,int l9){if(l9==int(0xf)){return(I7[0]+I7[1]+I7[2]+I7[3])*.25;}else{i kh=e(notEqual(l9&y6(1,2,4,8),y6(0,0,0,0)));i Y=D0(I7,kh);int m9=(l9&5)+((l9>>1)&5);m9=(m9&3)+(m9>>2);Y*=1./float(m9);return Y;}}
#endif
#ifdef DB
f i Qa(i Yd,i z1){return Yd+z1*(1.-Yd.w);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive