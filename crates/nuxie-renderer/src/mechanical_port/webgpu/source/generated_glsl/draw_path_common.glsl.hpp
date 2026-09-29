#pragma once

#include "draw_path_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_common[] = R"===(#define j7 -2.
#define Uc -1.5
#define Vc .25
#define C8 1e3
#define Wc (C8*C8)
#ifdef DB
T3 nc(d3,Nf,KC);
#ifdef HB
i6(d3,h7,XC);
#endif
U3 B4 J4(Pc,lg,PB);N5(Kb,Re,AD);O5(Lb,Se,QB);J4(Qc,mg,ED);C4
#endif
#if defined(HB)||defined(FB)
a4(h7,Z9)
#endif
#ifdef GB
D3 Z2(d3,Rc,MD);
#if defined(HB)||defined(FB)
i6(d3,h7,XC);
#endif
#ifdef FB
m5(d3,Sc,BD);
#endif
Z2(d5,V3,HC);
#if defined(CB)&&defined(AB)&&!defined(N)
j6(UD);
#endif
E3 a4(Rc,Rb)
#ifdef FB
a4(Sc,O9)
#endif
e5 W3(V5)f5
#endif
#ifdef GB
e bool U5(g M){return M.y>=.0;}e bool U5(E M){return M.y>=.0;}
#endif
#if defined(GB)&&defined(HB)
e bool Sb(g M){return M.x<Uc;}e bool Tb(g M){return M.y<Uc;}
#endif
#ifdef DB
g Xc(float xa,d D8,float E1){d k6=(1.-D8*abs(E1))*.5;float c4,n5;if(abs(xa-X6)<1./C8){c4=.0;n5=.0;}else{float ya=tan(xa);c4=sign(X6-xa)/max(abs(ya),1./Wc);n5=c4>=.0?k6.y-(1.-k6.x)*ya:k6.y+k6.x*ya;}g M;M.x=max(k6.x,.0)+Vc;M.y=-k6.y+j7;M.z=c4;M.w=n5;return M;}
#endif
#ifdef HB
e c d8(g M H3){c c4=M.z;c n5=max(M.w,.0);c l6=c4>=.0?j5(n5):.0;if(abs(c4)<C8){c x=abs(M.x)-Vc;c y=-M.y+j7;c X2=(y-n5)*0.5984134206;i t=n5+X2*C0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-c4+(y*c4+x);i ng=C0(j5(u[0]),j5(u[1]),j5(u[2]),j5(u[3]));i Yc=t*5.09593080173+-2.54796540086;i og=exp2(-Yc*Yc);l6+=dot(ng,og)*X2;}return l6*sign(M.x);}e c y4(g M H3){float l6=1.;float pg=(1.-j7)+M.x;l6-=j5(pg);float qg=1.-M.y;l6-=j5(qg);return l6;}
#endif
#if defined(DB)&&defined(KD)
e Y o5(int Zc){return Y(Zc&((1<<Ec)-1),Zc>>Ec);}e float ad(f0 U0,d rg){d j2=R0(U0,rg);return(abs(j2.x)+abs(j2.y))*(1./dot(j2,j2));}e bool q9(g k7,g za,int A,Z0(uint)e3,Z0(d)sg
#ifndef CB
,Z0(g)P1
#else
,Z0(K)l7
#endif
m6){int E8=int(k7.x);float E1=k7.y;float Aa=k7.z;int bd=floatBitsToInt(k7.w)>>2;int m7=floatBitsToInt(k7.w)&3;int Ba=min(E8,bd-1);int K4=A*bd+Ba;D4 p5=q1(KC,o5(K4));uint h0=i5(p5.w);uint F8=max(h0&Lc,1u);X Ca=J0(ED,F8-1u);d cd=uintBitsToFloat(Ca.xy);e3=Ca.z&0xffffu;uint dd=Ca.w;f0 U0=h2(uintBitsToFloat(J0(PB,e3*4u)));X L4=J0(PB,e3*4u+1u);d k3=uintBitsToFloat(L4.xy);float J2=uintBitsToFloat(L4.z);float K2=uintBitsToFloat(L4.w);uint ed=h0&F3;if(ed!=0u){E8=int(za.x);E1=za.y;Aa=za.z;}if(E8!=Ba){int fd=K4+E8-Ba;D4 gd=q1(KC,o5(fd));if((i5(gd.w)&(F3|0xffffu))!=(h0&(F3|0xffffu))){bool tg=J2==.0||cd.x!=.0;if(tg){K4=int(dd);p5=q1(KC,o5(K4));}}else{K4=fd;p5=gd;}h0=(i5(p5.w)&~F3)|ed;}float e1;
#ifdef HB
float n7;float v1;if((h0&Z3)==y8&&m7==B8){uint hd=i5(p5.z);float d4=float(hd&0xffffu);float k2=float(hd>>16);Y G8=Y(-d4-1.,k2-d4+1.);if((h0&F3)!=0u)G8=-G8;D4 id=q1(KC,o5(K4+G8.x));D4 Da=q1(KC,o5(K4+G8.y));if((i5(Da.w)&(F3|0xffffu))!=(i5(id.w)&(F3|0xffffu))){Da=q1(KC,o5(int(dd)));}n7=Y5(id.z);float jd=Y5(Da.z);v1=jd-n7;if(abs(v1)>C3)v1-=p8*sign(v1);float Ea=k2+1.-float(Fc);float kd=clamp(round(abs(v1)/C3*Ea),1.,Ea-1.);float o7=Ea-kd;if(d4<=o7){v1=-(C3*sign(v1)-v1);k2=o7;if(d4==o7)E1=-E1;}else if(d4==o7+1.){d4=.0;k2=.0;E1=.0;}else{d4-=o7+2.;k2=kd;}if(d4==k2){e1=jd;}else{e1=n7+v1*(d4/k2);}}else
#endif
{e1=Y5(p5.z);}d Y2=d(sin(e1),-cos(e1));d ld=Y5(p5.xy);d H8=d(0,0);if(K2!=.0){K2=max(K2,(na/3.)/length(R0(U0,Y2)));}if(J2!=.0){E1*=sign(determinant(U0));if((h0&A8)!=0u)E1=min(E1,.0);if((h0&Kc)!=0u)E1=max(E1,.0);float M4=K2!=.0?K2:ad(U0,Y2)*q4;c md=1.;if(M4>J2&&K2==.0){md=W4(J2)/W4(M4);J2=M4;}d q5=Y2*(J2+M4);
#ifndef CB
float x=E1*(J2+M4);P1.xy=(1./(M4*2.))*(d(x,-x)+J2)+.5;P1.zw=N6(.0);
#endif
uint Fa=h0&Z3;if(Fa>x8){int p7=2;if((h0&oa)==0u)p7=-p7;if((h0&F3)!=0u)p7=-p7;Y ug=o5(K4+p7);D4 vg=q1(KC,ug);float wg=Y5(vg.z);float q7=abs(wg-e1);if(q7>C3)q7=p8-q7;bool I8=(h0&oa)!=0u;bool xg=(h0&A8)!=0u;float nd=q7*(I8==xg?-.5:.5)+e1;d J8=d(sin(nd),-cos(nd));float Ga=ad(U0,J8);float r7=cos(q7*.5);float Ha;if((Fa==If)||(Fa==Jf&&r7>=.25)){float yg=(h0&z8)!=0u?1.:.25;Ha=J2*(1./max(r7,yg));}else{Ha=J2*r7+Ga*.5;}float Ia=Ha+Ga*q4;if((h0&Jc)!=0u){float od=J2+M4;float zg=M4*.125;if(od<=Ia*r7+zg){float Ag=od*(1./r7);q5=J8*Ag;}else{d Ja=J8*Ia;d Bg=d(dot(q5,q5),dot(Ja,Ja));q5=R0(Bg,inverse(f0(q5,Ja)));}}d Cg=abs(E1)*q5;float pd=(Ia-dot(Cg,J8))/(Ga*(q4*2.));
#ifndef CB
if((h0&A8)!=0u)P1.y=pd;else P1.x=pd;
#endif
}
#ifndef CB
P1.xy*=md;P1.y=max(P1.y,1e-4);if(K2!=.0){P1.x=j7-P1.x;}
#endif
H8=R0(U0,E1*q5);if(m7!=B8)return false;}else{
#ifndef CB
P1=g(Aa,-1.,.0,.0);
#ifdef HB
if(K2!=.0){P1.y=j7;P1.z=Wc;P1.w=Aa;if((h0&Z3)==y8&&m7==B8){if(v1<.0){n7+=v1;v1=-v1;}float e4=e1-n7;e4=mod(e4+X6,p8)-X6;e4=clamp(e4,.0,v1);if(e4>v1*.5){e4=v1-e4;}d D8=d(sin(e4),cos(e4));
#if 0
float Q1=1.+.33*log2(X6/(C3-min(v1,C3-C3/16.)));g Dg=Xc(v1,D8,.5*(Q1/3.));float Eg=d8(Dg d1);float Fg=qc(Eg);float Gg=(.5-Fg)*(na*2.);float Hg=Q1/max(Gg,Q1);E1*=Hg;
#endif
P1=Xc(v1,D8,E1);}H8=R0(U0,(E1*K2)*Y2);}else
#endif
{H8=sign(R0(E1*Y2,inverse(U0)))*q4;}if(bool(h0&F3)!=bool(h0&Kf)){P1*=g(-1.,+1.,+1.,+1.);}
#endif
if(m7==Nc)ld=cd;if((h0&Ic)!=0u&&m7!=Mc){return false;}}sg=R0(U0,ld)+H8+k3;
#ifdef CB
X N4=J0(PB,e3*4u+2u);l7=X1(N4.x);
#else
P1.xy=mix(P1.xy,d(1.,-1.),sf(m.Ig!=0u));
#endif
return true;}
#endif
#if defined(DB)&&defined(EB)
e d Ib(Q n6,Z0(uint)e3
#ifdef CB
,Z0(K)l7
#else
,Z0(c)Jg
#endif
m6){e3=floatBitsToUint(n6.z)&0xffffu;
#ifdef CB
X N4=J0(PB,e3*4u+2u);l7=X1(N4.x);
#else
Jg=aa(floatBitsToInt(n6.z)>>16);
#endif
d o6=n6.xy;f0 U0=h2(uintBitsToFloat(J0(PB,e3*4u)));X L4=J0(PB,e3*4u+1u);d k3=uintBitsToFloat(L4.xy);o6=R0(U0,o6)+k3;return o6;}
#endif
#if defined(DB)&&defined(FB)
e d Hb(Q n6,Z0(uint)e3,
#ifdef CB
Z0(K)l7,
#endif
Z0(d)Kg m6){e3=floatBitsToUint(n6.z)&0xffffu;X N4=J0(PB,e3*4u+2u);
#ifdef CB
l7=X1(N4.x);
#endif
d o6=n6.xy;Q v7=uintBitsToFloat(N4.yzw);Kg=(o6*v7.x+v7.yz)*m.Lg;return o6;}
#endif
e c K8(c a2,c F1,c v2){return(F1-a2)/max(1.-a2*v2,o9);}
#if defined(RB)||defined(FD)
e uint L8(a1 f4,uint Mg){uint Ka=(f4.y>>h6)*(Mg<<h6)+((f4.x>>h6)<<(h6<<1));Ka+=((f4.x&0x1cu)<<h6)+((f4.y&0x1cu)<<2);Ka+=((f4.y&0x3u)<<2)+(f4.x&0x3u);return Ka;}
#endif
#ifdef RB
#ifdef N
#define g5 p2
#define X3(r5) D1=r5;m3
#else
#define g5 M1
#define X3(r5) y0(j0,r5);Z1;
#endif
e c La(uint Ng){return aa(int((Ng&ua)-l5))*sa;}e uint w7(c n){return uint(n*Tf+.5);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive