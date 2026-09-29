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
U3 mc(d3,Of,MC);
#ifdef HB
j6(d3,h7,YC);
#endif
V3 C4 J4(Pc,mg,QB);O5(Kb,Re,BD);P5(Lb,Se,RB);J4(Qc,ng,FD);D4
#endif
#if defined(HB)||defined(FB)
c4(h7,aa)
#endif
#ifdef GB
E3 Z2(d3,Rc,ND);
#if defined(HB)||defined(FB)
j6(d3,h7,YC);
#endif
#ifdef FB
n5(d3,Sc,CD);
#endif
Z2(d5,W3,JC);
#if defined(CB)&&defined(AB)&&!defined(Q)
k7(VD);
#endif
F3 c4(Rc,Qb)
#ifdef FB
c4(Sc,Q9)
#endif
e5 X3(W5)f5
#endif
#ifdef GB
e bool V5(g P){return P.y>=.0;}e bool V5(E P){return P.y>=.0;}
#endif
#if defined(GB)&&defined(HB)
e bool Rb(g P){return P.x<Uc;}e bool Sb(g P){return P.y<Uc;}
#endif
#ifdef DB
g Xc(float va,d D8,float E1){d k6=(1.-D8*abs(E1))*.5;float d4,o5;if(abs(va-X6)<1./C8){d4=.0;o5=.0;}else{float wa=tan(va);d4=sign(X6-va)/max(abs(wa),1./Wc);o5=d4>=.0?k6.y-(1.-k6.x)*wa:k6.y+k6.x*wa;}g P;P.x=max(k6.x,.0)+Vc;P.y=-k6.y+j7;P.z=d4;P.w=o5;return P;}
#endif
#ifdef HB
e c d8(g P I3){c d4=P.z;c o5=max(P.w,.0);c l6=d4>=.0?j5(o5):.0;if(abs(d4)<C8){c x=abs(P.x)-Vc;c y=-P.y+j7;c X2=(y-o5)*0.5984134206;i t=o5+X2*C0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-d4+(y*d4+x);i og=C0(j5(u[0]),j5(u[1]),j5(u[2]),j5(u[3]));i Yc=t*5.09593080173+-2.54796540086;i pg=exp2(-Yc*Yc);l6+=dot(og,pg)*X2;}return l6*sign(P.x);}e c z4(g P I3){float l6=1.;float qg=(1.-j7)+P.x;l6-=j5(qg);float rg=1.-P.y;l6-=j5(rg);return l6;}
#endif
#if defined(DB)&&defined(LD)
e Y p5(int Zc){return Y(Zc&((1<<Ec)-1),Zc>>Ec);}e float ad(f0 U0,d sg){d j2=R0(U0,sg);return(abs(j2.x)+abs(j2.y))*(1./dot(j2,j2));}e bool q9(g l7,g xa,int v,Z0(uint)e3,Z0(d)tg
#ifndef CB
,Z0(g)P1
#else
,Z0(N)m7
#endif
m6){int E8=int(l7.x);float E1=l7.y;float ya=l7.z;int bd=floatBitsToInt(l7.w)>>2;int n7=floatBitsToInt(l7.w)&3;int za=min(E8,bd-1);int K4=v*bd+za;E4 q5=q1(MC,p5(K4));uint i0=i5(q5.w);uint F8=max(i0&Lc,1u);G Aa=J0(FD,F8-1u);d cd=uintBitsToFloat(Aa.xy);e3=Aa.z&0xffffu;uint dd=Aa.w;f0 U0=h2(uintBitsToFloat(J0(QB,e3*4u)));G L4=J0(QB,e3*4u+1u);d k3=uintBitsToFloat(L4.xy);float J2=uintBitsToFloat(L4.z);float K2=uintBitsToFloat(L4.w);uint ed=i0&G3;if(ed!=0u){E8=int(xa.x);E1=xa.y;ya=xa.z;}if(E8!=za){int fd=K4+E8-za;E4 gd=q1(MC,p5(fd));if((i5(gd.w)&(G3|0xffffu))!=(i0&(G3|0xffffu))){bool ug=J2==.0||cd.x!=.0;if(ug){K4=int(dd);q5=q1(MC,p5(K4));}}else{K4=fd;q5=gd;}i0=(i5(q5.w)&~G3)|ed;}float e1;
#ifdef HB
float o7;float v1;if((i0&a4)==y8&&n7==B8){uint hd=i5(q5.z);float e4=float(hd&0xffffu);float k2=float(hd>>16);Y G8=Y(-e4-1.,k2-e4+1.);if((i0&G3)!=0u)G8=-G8;E4 id=q1(MC,p5(K4+G8.x));E4 Ba=q1(MC,p5(K4+G8.y));if((i5(Ba.w)&(G3|0xffffu))!=(i5(id.w)&(G3|0xffffu))){Ba=q1(MC,p5(int(dd)));}o7=Z5(id.z);float jd=Z5(Ba.z);v1=jd-o7;if(abs(v1)>D3)v1-=p8*sign(v1);float Ca=k2+1.-float(Fc);float kd=clamp(round(abs(v1)/D3*Ca),1.,Ca-1.);float p7=Ca-kd;if(e4<=p7){v1=-(D3*sign(v1)-v1);k2=p7;if(e4==p7)E1=-E1;}else if(e4==p7+1.){e4=.0;k2=.0;E1=.0;}else{e4-=p7+2.;k2=kd;}if(e4==k2){e1=jd;}else{e1=o7+v1*(e4/k2);}}else
#endif
{e1=Z5(q5.z);}d Y2=d(sin(e1),-cos(e1));d ld=Z5(q5.xy);d H8=d(0,0);if(K2!=.0){K2=max(K2,(na/3.)/length(R0(U0,Y2)));}if(J2!=.0){E1*=sign(determinant(U0));if((i0&A8)!=0u)E1=min(E1,.0);if((i0&Kc)!=0u)E1=max(E1,.0);float M4=K2!=.0?K2:ad(U0,Y2)*q4;c md=1.;if(M4>J2&&K2==.0){md=W4(J2)/W4(M4);J2=M4;}d r5=Y2*(J2+M4);
#ifndef CB
float x=E1*(J2+M4);P1.xy=(1./(M4*2.))*(d(x,-x)+J2)+.5;P1.zw=N6(.0);
#endif
uint Da=i0&a4;if(Da>x8){int q7=2;if((i0&oa)==0u)q7=-q7;if((i0&G3)!=0u)q7=-q7;Y vg=p5(K4+q7);E4 wg=q1(MC,vg);float xg=Z5(wg.z);float r7=abs(xg-e1);if(r7>D3)r7=p8-r7;bool I8=(i0&oa)!=0u;bool yg=(i0&A8)!=0u;float nd=r7*(I8==yg?-.5:.5)+e1;d J8=d(sin(nd),-cos(nd));float Ea=ad(U0,J8);float v7=cos(r7*.5);float Fa;if((Da==If)||(Da==Jf&&v7>=.25)){float zg=(i0&z8)!=0u?1.:.25;Fa=J2*(1./max(v7,zg));}else{Fa=J2*v7+Ea*.5;}float Ga=Fa+Ea*q4;if((i0&Jc)!=0u){float od=J2+M4;float Ag=M4*.125;if(od<=Ga*v7+Ag){float Bg=od*(1./v7);r5=J8*Bg;}else{d Ha=J8*Ga;d Cg=d(dot(r5,r5),dot(Ha,Ha));r5=R0(Cg,inverse(f0(r5,Ha)));}}d Dg=abs(E1)*r5;float pd=(Ga-dot(Dg,J8))/(Ea*(q4*2.));
#ifndef CB
if((i0&A8)!=0u)P1.y=pd;else P1.x=pd;
#endif
}
#ifndef CB
P1.xy*=md;P1.y=max(P1.y,1e-4);if(K2!=.0){P1.x=j7-P1.x;}
#endif
H8=R0(U0,E1*r5);if(n7!=B8)return false;}else{
#ifndef CB
P1=g(ya,-1.,.0,.0);
#ifdef HB
if(K2!=.0){P1.y=j7;P1.z=Wc;P1.w=ya;if((i0&a4)==y8&&n7==B8){if(v1<.0){o7+=v1;v1=-v1;}float f4=e1-o7;f4=mod(f4+X6,p8)-X6;f4=clamp(f4,.0,v1);if(f4>v1*.5){f4=v1-f4;}d D8=d(sin(f4),cos(f4));
#if 0
float Q1=1.+.33*log2(X6/(D3-min(v1,D3-D3/16.)));g Eg=Xc(v1,D8,.5*(Q1/3.));float Fg=d8(Eg d1);float Gg=pc(Fg);float Hg=(.5-Gg)*(na*2.);float Ig=Q1/max(Hg,Q1);E1*=Ig;
#endif
P1=Xc(v1,D8,E1);}H8=R0(U0,(E1*K2)*Y2);}else
#endif
{H8=sign(R0(E1*Y2,inverse(U0)))*q4;}if(bool(i0&G3)!=bool(i0&Kf)){P1*=g(-1.,+1.,+1.,+1.);}
#endif
if(n7==Nc)ld=cd;if((i0&Ic)!=0u&&n7!=Mc){return false;}}tg=R0(U0,ld)+H8+k3;
#ifdef CB
G N4=J0(QB,e3*4u+2u);m7=X1(N4.x);
#else
P1.xy=mix(P1.xy,d(1.,-1.),sf(m.Jg!=0u));
#endif
return true;}
#endif
#if defined(DB)&&defined(EB)
e d Ib(R n6,Z0(uint)e3
#ifdef CB
,Z0(N)m7
#else
,Z0(c)Kg
#endif
m6){e3=floatBitsToUint(n6.z)&0xffffu;
#ifdef CB
G N4=J0(QB,e3*4u+2u);m7=X1(N4.x);
#else
Kg=ba(floatBitsToInt(n6.z)>>16);
#endif
d o6=n6.xy;f0 U0=h2(uintBitsToFloat(J0(QB,e3*4u)));G L4=J0(QB,e3*4u+1u);d k3=uintBitsToFloat(L4.xy);o6=R0(U0,o6)+k3;return o6;}
#endif
#if defined(DB)&&defined(FB)
e d Hb(R n6,Z0(uint)e3,
#ifdef CB
Z0(N)m7,
#endif
Z0(d)Lg m6){e3=floatBitsToUint(n6.z)&0xffffu;G N4=J0(QB,e3*4u+2u);
#ifdef CB
m7=X1(N4.x);
#endif
d o6=n6.xy;R w7=uintBitsToFloat(N4.yzw);Lg=(o6*w7.x+w7.yz)*m.Mg;return o6;}
#endif
e c K8(c a2,c F1,c v2){return(F1-a2)/max(1.-a2*v2,o9);}
#if defined(SB)||defined(GD)
e uint L8(a1 p6,uint Ng){uint Ia=(p6.y>>i6)*(Ng<<i6)+((p6.x>>i6)<<(i6<<1));Ia+=((p6.x&0x1cu)<<i6)+((p6.y&0x1cu)<<2);Ia+=((p6.y&0x3u)<<2)+(p6.x&0x3u);return Ia;}
#endif
#ifdef SB
#ifdef Q
#define g5 p2
#define Y3(v5) D1=v5;n3
#else
#define g5 M1
#define Y3(v5) y0(j0,v5);Z1;
#endif
e c Ja(uint Og){return ba(int((Og&sa)-m5))*qa;}e uint x7(c n){return uint(n*Uf+.5);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive