#pragma once

#include "draw_path_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_common[] = R"===(#define f7 -2.
#define Qc -1.5
#define Rc .25
#define z8 1e3
#define Sc (z8*z8)
#ifdef DB
S3 ic(a3,Ff,LC);
#ifdef HB
f6(a3,d7,XC);
#endif
T3 z4 G4(Lc,bg,PB);K5(Gb,Je,AD);L5(Hb,Ke,RB);G4(Mc,cg,ED);A4
#endif
#if defined(HB)||defined(FB)
Z3(d7,aa)
#endif
#ifdef GB
C3 X2(a3,Nc,KD);
#if defined(HB)||defined(FB)
f6(a3,d7,XC);
#endif
#ifdef FB
k5(a3,Oc,BD);
#endif
X2(Z4,U3,IC);
#if defined(CB)&&defined(AB)&&!defined(Q)
g7(SD);
#endif
D3 Z3(Nc,Mb)
#ifdef FB
Z3(Oc,Q9)
#endif
a5 V3(S5)c5
#endif
#ifdef GB
e bool R5(g P){return P.y>=.0;}e bool R5(E P){return P.y>=.0;}
#endif
#if defined(GB)&&defined(HB)
e bool Nb(g P){return P.x<Qc;}e bool Ob(g P){return P.y<Qc;}
#endif
#ifdef DB
g Tc(float ua,d A8,float D1){d g6=(1.-A8*abs(D1))*.5;float a4,l5;if(abs(ua-T6)<1./z8){a4=.0;l5=.0;}else{float va=tan(ua);a4=sign(T6-ua)/max(abs(va),1./Sc);l5=a4>=.0?g6.y-(1.-g6.x)*va:g6.y+g6.x*va;}g P;P.x=max(g6.x,.0)+Rc;P.y=-g6.y+f7;P.z=a4;P.w=l5;return P;}
#endif
#ifdef HB
e c Z7(g P G3){c a4=P.z;c l5=max(P.w,.0);c h6=a4>=.0?g5(l5):.0;if(abs(a4)<z8){c x=abs(P.x)-Rc;c y=-P.y+f7;c V2=(y-l5)*0.5984134206;i t=l5+V2*C0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-a4+(y*a4+x);i dg=C0(g5(u[0]),g5(u[1]),g5(u[2]),g5(u[3]));i Uc=t*5.09593080173+-2.54796540086;i eg=exp2(-Uc*Uc);h6+=dot(dg,eg)*V2;}return h6*sign(P.x);}e c v4(g P G3){float h6=1.;float fg=(1.-f7)+P.x;h6-=g5(fg);float gg=1.-P.y;h6-=g5(gg);return h6;}
#endif
#if defined(DB)&&defined(ID)
e X m5(int Vc){return X(Vc&((1<<Ac)-1),Vc>>Ac);}e float Wc(g0 T0,d hg){d h2=U0(T0,hg);return(abs(h2.x)+abs(h2.y))*(1./dot(h2,h2));}e bool q9(g h7,g wa,int r,Z0(uint)c3,Z0(d)ig
#ifndef CB
,Z0(g)O1
#else
,Z0(N)i7
#endif
i6){int B8=int(h7.x);float D1=h7.y;float xa=h7.z;int Xc=floatBitsToInt(h7.w)>>2;int j7=floatBitsToInt(h7.w)&3;int ya=min(B8,Xc-1);int H4=r*Xc+ya;B4 n5=q1(LC,m5(H4));uint i0=f5(n5.w);uint C8=max(i0&Hc,1u);G za=N0(ED,C8-1u);d Yc=uintBitsToFloat(za.xy);c3=za.z&0xffffu;uint Zc=za.w;g0 T0=l2(uintBitsToFloat(N0(PB,c3*4u)));G I4=N0(PB,c3*4u+1u);d i3=uintBitsToFloat(I4.xy);float H2=uintBitsToFloat(I4.z);float I2=uintBitsToFloat(I4.w);uint ad=i0&E3;if(ad!=0u){B8=int(wa.x);D1=wa.y;xa=wa.z;}if(B8!=ya){int bd=H4+B8-ya;B4 cd=q1(LC,m5(bd));if((f5(cd.w)&(E3|0xffffu))!=(i0&(E3|0xffffu))){bool jg=H2==.0||Yc.x!=.0;if(jg){H4=int(Zc);n5=q1(LC,m5(H4));}}else{H4=bd;n5=cd;}i0=(f5(n5.w)&~E3)|ad;}float e1;
#ifdef HB
float k7;float v1;if((i0&Y3)==v8&&j7==y8){uint dd=f5(n5.z);float c4=float(dd&0xffffu);float i2=float(dd>>16);X D8=X(-c4-1.,i2-c4+1.);if((i0&E3)!=0u)D8=-D8;B4 ed=q1(LC,m5(H4+D8.x));B4 Aa=q1(LC,m5(H4+D8.y));if((f5(Aa.w)&(E3|0xffffu))!=(f5(ed.w)&(E3|0xffffu))){Aa=q1(LC,m5(int(Zc)));}k7=V5(ed.z);float fd=V5(Aa.z);v1=fd-k7;if(abs(v1)>B3)v1-=m8*sign(v1);float Ba=i2+1.-float(Bc);float gd=clamp(round(abs(v1)/B3*Ba),1.,Ba-1.);float l7=Ba-gd;if(c4<=l7){v1=-(B3*sign(v1)-v1);i2=l7;if(c4==l7)D1=-D1;}else if(c4==l7+1.){c4=.0;i2=.0;D1=.0;}else{c4-=l7+2.;i2=gd;}if(c4==i2){e1=fd;}else{e1=k7+v1*(c4/i2);}}else
#endif
{e1=V5(n5.z);}d W2=d(sin(e1),-cos(e1));d hd=V5(n5.xy);d E8=d(0,0);if(I2!=.0){I2=max(I2,(ma/3.)/length(U0(T0,W2)));}if(H2!=.0){D1*=sign(determinant(T0));if((i0&x8)!=0u)D1=min(D1,.0);if((i0&Gc)!=0u)D1=max(D1,.0);float J4=I2!=.0?I2:Wc(T0,W2)*n4;c id=1.;if(J4>H2&&I2==.0){id=S4(H2)/S4(J4);H2=J4;}d o5=W2*(H2+J4);
#ifndef CB
float x=D1*(H2+J4);O1.xy=(1./(J4*2.))*(d(x,-x)+H2)+.5;O1.zw=J6(.0);
#endif
uint Ca=i0&Y3;if(Ca>r8){int m7=2;if((i0&na)==0u)m7=-m7;if((i0&E3)!=0u)m7=-m7;X kg=m5(H4+m7);B4 lg=q1(LC,kg);float mg=V5(lg.z);float n7=abs(mg-e1);if(n7>B3)n7=m8-n7;bool F8=(i0&na)!=0u;bool ng=(i0&x8)!=0u;float jd=n7*(F8==ng?-.5:.5)+e1;d G8=d(sin(jd),-cos(jd));float Da=Wc(T0,G8);float o7=cos(n7*.5);float Ea;if((Ca==Af)||(Ca==Bf&&o7>=.25)){float og=(i0&w8)!=0u?1.:.25;Ea=H2*(1./max(o7,og));}else{Ea=H2*o7+Da*.5;}float Fa=Ea+Da*n4;if((i0&Fc)!=0u){float kd=H2+J4;float pg=J4*.125;if(kd<=Fa*o7+pg){float qg=kd*(1./o7);o5=G8*qg;}else{d Ga=G8*Fa;d rg=d(dot(o5,o5),dot(Ga,Ga));o5=U0(rg,inverse(g0(o5,Ga)));}}d sg=abs(D1)*o5;float ld=(Fa-dot(sg,G8))/(Da*(n4*2.));
#ifndef CB
if((i0&x8)!=0u)O1.y=ld;else O1.x=ld;
#endif
}
#ifndef CB
O1.xy*=id;O1.y=max(O1.y,1e-4);if(I2!=.0){O1.x=f7-O1.x;}
#endif
E8=U0(T0,D1*o5);if(j7!=y8)return false;}else{
#ifndef CB
O1=g(xa,-1.,.0,.0);
#ifdef HB
if(I2!=.0){O1.y=f7;O1.z=Sc;O1.w=xa;if((i0&Y3)==v8&&j7==y8){if(v1<.0){k7+=v1;v1=-v1;}float d4=e1-k7;d4=mod(d4+T6,m8)-T6;d4=clamp(d4,.0,v1);if(d4>v1*.5){d4=v1-d4;}d A8=d(sin(d4),cos(d4));
#if 0
float P1=1.+.33*log2(T6/(B3-min(v1,B3-B3/16.)));g tg=Tc(v1,A8,.5*(P1/3.));float ug=Z7(tg d1);float vg=lc(ug);float wg=(.5-vg)*(ma*2.);float xg=P1/max(wg,P1);D1*=xg;
#endif
O1=Tc(v1,A8,D1);}E8=U0(T0,(D1*I2)*W2);}else
#endif
{E8=sign(U0(D1*W2,inverse(T0)))*n4;}if(bool(i0&E3)!=bool(i0&Cf)){O1*=g(-1.,+1.,+1.,+1.);}
#endif
if(j7==Jc)hd=Yc;if((i0&Ec)!=0u&&j7!=Ic){return false;}}ig=U0(T0,hd)+E8+i3;
#ifdef CB
G K4=N0(PB,c3*4u+2u);i7=W1(K4.x);
#else
O1.xy=mix(O1.xy,d(1.,-1.),kf(n.yg!=0u));
#endif
return true;}
#endif
#if defined(DB)&&defined(EB)
e d Eb(c0 j6,Z0(uint)c3
#ifdef CB
,Z0(N)i7
#else
,Z0(c)zg
#endif
i6){c3=floatBitsToUint(j6.z)&0xffffu;
#ifdef CB
G K4=N0(PB,c3*4u+2u);i7=W1(K4.x);
#else
zg=ba(floatBitsToInt(j6.z)>>16);
#endif
d k6=j6.xy;g0 T0=l2(uintBitsToFloat(N0(PB,c3*4u)));G I4=N0(PB,c3*4u+1u);d i3=uintBitsToFloat(I4.xy);k6=U0(T0,k6)+i3;return k6;}
#endif
#if defined(DB)&&defined(FB)
e d Db(c0 j6,Z0(uint)c3,
#ifdef CB
Z0(N)i7,
#endif
Z0(d)Ag i6){c3=floatBitsToUint(j6.z)&0xffffu;G K4=N0(PB,c3*4u+2u);
#ifdef CB
i7=W1(K4.x);
#endif
d k6=j6.xy;c0 p7=uintBitsToFloat(K4.yzw);Ag=(k6*p7.x+p7.yz)*n.Bg;return k6;}
#endif
e c H8(c Z1,c E1,c r2){return(E1-Z1)/max(1.-Z1*r2,o9);}e uint I8(a1 l6,uint Cg){uint Ha=(l6.y>>e6)*(Cg<<e6)+((l6.x>>e6)<<(e6<<1));Ha+=((l6.x&0x1cu)<<e6)+((l6.y&0x1cu)<<2);Ha+=((l6.y&0x3u)<<2)+(l6.x&0x3u);return Ha;}
#ifdef SB
#ifdef Q
#define d5 o2
#define W3(p5) C1=p5;l3
#else
#define d5 L1
#define W3(p5) x0(j0,p5);Y1;
#endif
e c Ia(uint Dg){return ba(int((Dg&ra)-j5))*pa;}e uint q7(c o){return uint(o*Lf+.5);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive