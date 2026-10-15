#pragma once

#include "common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char common[] = R"===(#define p4 3.14159265359
#define h9 6.28318530718
#define A7 1.57079632679
#ifndef CB
#define O4 float(.5)
#else
#define O4 float(.0)
#endif
#define R3(o) g9(o,j.Jg,j.Kg)
#define Lg(a,o,i9) r1(a,g0(o)+g0(-1,0)) i9,r1(a,g0(o)+g0(0,0)) i9,r1(a,g0(o)+g0(0,-1)) i9,r1(a,g0(o)+g0(-1,-1)) i9
#define G5(E) B7(AD,db,E,Bd,float(Bd),.0).x
#define Dd(E) B7(AD,db,E,Cd,float(Cd),.0).x
#ifdef eb
e d w5(float x){return x;}e d r6(uint x){return float(x);}e d Mg(P x){return float(x);}e d fb(int x){return float(x);}e i V4(f xyzw){return xyzw;}e D E8(c xy){return xy;}e i xd(O xyzw){return vec4(xyzw);}e P W2(d x){return uint(x);}e P U1(uint x){return x;}
#else
e d w5(float x){return(d) x;}e d r6(uint x){return(d) x;}e d Mg(P x){return(d) x;}e d fb(int x){return(d) x;}e i V4(f xyzw){return(i) xyzw;}e D E8(c xy){return(D) xy;}e i xd(O xyzw){return(i) xyzw;}e P W2(d x){return(P) x;}e P U1(uint x){return(P) x;}
#endif
e d J0(d x){return x;}e D R2(D xy){return xy;}e D R2(d x,d y){D Y;Y.x=x,Y.y=y;return Y;}e D R2(d x){D Y;Y.x=x,Y.y=x;return Y;}e c o7(float x){return c(x,x);}e v Z0(d x,d y,d z){v Y;Y.x=x,Y.y=y,Y.z=z;return Y;}e v Z0(d x){v Y;Y.x=x,Y.y=x,Y.z=x;return Y;}e i H0(d x,d y,d z,d w){i Y;Y.x=x,Y.y=y,Y.z=z,Y.w=w;return Y;}e i H0(v xyz,d w){i Y;Y.xyz=xyz;Y.w=w;return Y;}e i H0(d x){i Y;Y.x=x,Y.y=x,Y.z=x,Y.w=x;return Y;}e i H0(i x){return x;}e c5 Ng(bool b){return c5(b,b);}e C7 Xj(v l,v b,v T1){C7 Y;Y[0]=l;Y[1]=b;Y[2]=T1;return Y;}e D7 Yj(v l,v b){D7 Y;Y[0]=l;Y[1]=b;return Y;}e d5 Zj(i l,i b,i T1,i Og){d5 Y;Y[0]=l;Y[1]=b;Y[2]=T1;Y[3]=Og;return Y;}e X p1(f x){return X(x.xy,x.zw);}e uint jd(P x){return x;}e c v6(c l,c b,float t){return(b-l)*t+l;}e d j9(uint Ed,uint w6){return Ed==0u?.0:unpackHalf2x16((Ed+Pg)*w6).x;}e float gb(c B2){B2=normalize(B2);float g1=acos(clamp(B2.x,-1.,1.));return B2.y>=.0?g1:-g1;}e i ak(i k){return H0(k.xyz*k.w,k.w);}e v i6(i hb){return hb.xyz*(hb.w!=.0?1./hb.w:.0);}e d B3(D E7){return min(E7.x,E7.y);}e d B3(v Fd){return min(B3(Fd.xy),Fd.z);}e d B3(i Gd){D E7=min(Gd.xy,Gd.zw);d Qg=min(E7.x,E7.y);return Qg;}e d h6(D F7){return max(F7.x,F7.y);}e d h6(v Hd){return max(h6(Hd.xy),Hd.z);}e d h6(i Id){D F7=max(Id.xy,Id.zw);d Rg=max(F7.x,F7.y);return Rg;}e float Ba(c x){return abs(x.x)+abs(x.y);}e d ib(d x,d jb,d kb){
#if defined(SF)||defined(LD)
#ifdef LD
if(LD)
#endif
{if(x<kb) if(x>jb) return x;else return jb;else return kb;}
#endif
return clamp(x,jb,kb);}e d Jd(c l0,d S2,d H3){d Sg=fract(0.06711056*l0.x+0.00583715*l0.y);d Tg=fract(52.9829189*Sg);return(Tg*S2)+H3;}
#if 0
e d bk(c l0,float S2,float H3){int x=int(l0.x);int y=int(l0.y);int Kd=(x^y);int b=(y>>1)&1;b|=(Kd&2);b|=(y&1)<<2;b|=(Kd&1)<<3;float Ug=float(b);d Vg=w5(Ug)/16.0;return(Vg*S2)+H3;}e d ck(c l0,float S2,float H3){l0.y*=0.5;l0.x=fract(l0.x*0.5+l0.y);l0.y=fract(l0.y);float j4=(l0.y*0.5+l0.x);return(j4*S2)+H3;}
#endif
#ifdef OB
e d lb(c l0,d S2,d H3){return OB?Jd(l0,S2,H3):.0;}e v J2(v k,d G7,c l0,d S2,d H3){return(OB&&G7!=.0)?(Jd(l0,S2,H3)+k):k;}e v J2(v k,d G7,d Ld){return(OB&&G7!=.0)?(Ld+k):k;}
#else
e d lb(c l0,float S2,float H3){return 0.;}e v J2(v k,d G7,c l0,d S2,d H3){return k;}e v J2(v k,d G7,d Ld){return k;}
#endif
#ifdef BB
e f g9(c Md,float Wg,float Nd){return f(Md.x*Wg-1.,Md.y*Nd-sign(Nd),0.,1.);}
#ifndef CB
e f G8(X I3,c X3,c mb){c nb=abs(I3[0])+abs(I3[1]);if(nb.x!=.0&&nb.y!=.0){c R=1./nb;c H5=C0(I3,mb)+X3;const float Xg=.5;return f(H5,-H5)*R.xyxy+R.xyxy+Xg;}else{return X3.xyxy;}}
#else
e float k9(uint Yg,uint Zg){float Od=float((Yg<<ah)|Zg);
#if defined(eb)&&!defined(DC)
return Od*uintBitsToFloat(0x34000000u)+uintBitsToFloat(0xbf7fffffu);
#else
return Od*uintBitsToFloat(0x33800000u)+uintBitsToFloat(0x33000000u);
#endif
}
#ifdef AB
e void ob(X I3,c X3,c mb H7){
#ifndef UE
if(any(notEqual(f(I3),f(.0,.0,.0,.0)))){c H5=C0(I3,mb)+X3.xy;gl_ClipDistance[0]=H5.x+1.;gl_ClipDistance[1]=H5.y+1.;gl_ClipDistance[2]=1.-H5.x;gl_ClipDistance[3]=1.-H5.y;}else{gl_ClipDistance[0]=gl_ClipDistance[1]=gl_ClipDistance[2]=gl_ClipDistance[3]=X3.x-.5;}
#endif
}
#endif
#endif
#endif
#if defined(EB)&&defined(CB)&&!defined(U)
e i pb(d5 I7,int l9){if(l9==0xf){return(I7[0]+I7[1]+I7[2]+I7[3])*.25;}else{i bh=f(notEqual(l9&x6(1,2,4,8),x6(0,0,0,0)));i Y=C0(I7,bh);int m9=(l9&5)+((l9>>1)&5);m9=(m9&3)+(m9>>2);Y*=1./float(m9);return Y;}}
#endif
#ifdef EB
e i Oa(i Pd,i x1){return Pd+x1*(1.-Pd.w);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive