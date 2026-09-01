#pragma once

#include "draw_path_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_common[] = R"===(#define h7 -2.
#define Sc -1.5
#define Tc .25
#define B8 1e3
#define Uc (B8*B8)
#ifdef DB
U3 kc(d3,Jf,MC);
#ifdef HB
h6(d3,f7,YC);
#endif
V3 B4 I4(Nc,hg,QB);M5(Ib,Me,BD);N5(Jb,Ne,RB);I4(Oc,ig,FD);C4
#endif
#if defined(HB)||defined(FB)
c4(f7,aa)
#endif
#ifdef GB
E3 Z2(d3,Pc,MD);
#if defined(HB)||defined(FB)
h6(d3,f7,YC);
#endif
#ifdef FB
l5(d3,Qc,CD);
#endif
Z2(a5,W3,JC);
#if defined(CB)&&defined(AB)&&!defined(Q)
i7(UD);
#endif
F3 c4(Pc,Ob)
#ifdef FB
c4(Qc,Q9)
#endif
c5 X3(U5)d5
#endif
#ifdef GB
e bool T5(g P){return P.y>=.0;}e bool T5(E P){return P.y>=.0;}
#endif
#if defined(GB)&&defined(HB)
e bool Pb(g P){return P.x<Sc;}e bool Qb(g P){return P.y<Sc;}
#endif
#ifdef DB
g Vc(float ua,d C8,float D1){d i6=(1.-C8*abs(D1))*.5;float d4,m5;if(abs(ua-V6)<1./B8){d4=.0;m5=.0;}else{float va=tan(ua);d4=sign(V6-ua)/max(abs(va),1./Uc);m5=d4>=.0?i6.y-(1.-i6.x)*va:i6.y+i6.x*va;}g P;P.x=max(i6.x,.0)+Tc;P.y=-i6.y+h7;P.z=d4;P.w=m5;return P;}
#endif
#ifdef HB
e c c8(g P I3){c d4=P.z;c m5=max(P.w,.0);c j6=d4>=.0?h5(m5):.0;if(abs(d4)<B8){c x=abs(P.x)-Tc;c y=-P.y+h7;c X2=(y-m5)*0.5984134206;i t=m5+X2*C0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-d4+(y*d4+x);i jg=C0(h5(u[0]),h5(u[1]),h5(u[2]),h5(u[3]));i Wc=t*5.09593080173+-2.54796540086;i kg=exp2(-Wc*Wc);j6+=dot(jg,kg)*X2;}return j6*sign(P.x);}e c y4(g P I3){float j6=1.;float lg=(1.-h7)+P.x;j6-=h5(lg);float mg=1.-P.y;j6-=h5(mg);return j6;}
#endif
#if defined(DB)&&defined(KD)
e Y n5(int Xc){return Y(Xc&((1<<Cc)-1),Xc>>Cc);}e float Yc(f0 U0,d ng){d i2=R0(U0,ng);return(abs(i2.x)+abs(i2.y))*(1./dot(i2,i2));}e bool q9(g j7,g wa,int v,Z0(uint)e3,Z0(d)og
#ifndef CB
,Z0(g)O1
#else
,Z0(N)k7
#endif
k6){int D8=int(j7.x);float D1=j7.y;float xa=j7.z;int Zc=floatBitsToInt(j7.w)>>2;int l7=floatBitsToInt(j7.w)&3;int ya=min(D8,Zc-1);int J4=v*Zc+ya;D4 o5=q1(MC,n5(J4));uint i0=g5(o5.w);uint E8=max(i0&Jc,1u);G za=J0(FD,E8-1u);d ad=uintBitsToFloat(za.xy);e3=za.z&0xffffu;uint bd=za.w;f0 U0=g2(uintBitsToFloat(J0(QB,e3*4u)));G K4=J0(QB,e3*4u+1u);d k3=uintBitsToFloat(K4.xy);float J2=uintBitsToFloat(K4.z);float K2=uintBitsToFloat(K4.w);uint cd=i0&G3;if(cd!=0u){D8=int(wa.x);D1=wa.y;xa=wa.z;}if(D8!=ya){int dd=J4+D8-ya;D4 ed=q1(MC,n5(dd));if((g5(ed.w)&(G3|0xffffu))!=(i0&(G3|0xffffu))){bool pg=J2==.0||ad.x!=.0;if(pg){J4=int(bd);o5=q1(MC,n5(J4));}}else{J4=dd;o5=ed;}i0=(g5(o5.w)&~G3)|cd;}float e1;
#ifdef HB
float m7;float v1;if((i0&a4)==x8&&l7==A8){uint fd=g5(o5.z);float e4=float(fd&0xffffu);float j2=float(fd>>16);Y F8=Y(-e4-1.,j2-e4+1.);if((i0&G3)!=0u)F8=-F8;D4 gd=q1(MC,n5(J4+F8.x));D4 Aa=q1(MC,n5(J4+F8.y));if((g5(Aa.w)&(G3|0xffffu))!=(g5(gd.w)&(G3|0xffffu))){Aa=q1(MC,n5(int(bd)));}m7=X5(gd.z);float hd=X5(Aa.z);v1=hd-m7;if(abs(v1)>D3)v1-=o8*sign(v1);float Ba=j2+1.-float(Dc);float id=clamp(round(abs(v1)/D3*Ba),1.,Ba-1.);float n7=Ba-id;if(e4<=n7){v1=-(D3*sign(v1)-v1);j2=n7;if(e4==n7)D1=-D1;}else if(e4==n7+1.){e4=.0;j2=.0;D1=.0;}else{e4-=n7+2.;j2=id;}if(e4==j2){e1=hd;}else{e1=m7+v1*(e4/j2);}}else
#endif
{e1=X5(o5.z);}d Y2=d(sin(e1),-cos(e1));d jd=X5(o5.xy);d G8=d(0,0);if(K2!=.0){K2=max(K2,(ma/3.)/length(R0(U0,Y2)));}if(J2!=.0){D1*=sign(determinant(U0));if((i0&z8)!=0u)D1=min(D1,.0);if((i0&Ic)!=0u)D1=max(D1,.0);float L4=K2!=.0?K2:Yc(U0,Y2)*p4;c kd=1.;if(L4>J2&&K2==.0){kd=U4(J2)/U4(L4);J2=L4;}d p5=Y2*(J2+L4);
#ifndef CB
float x=D1*(J2+L4);O1.xy=(1./(L4*2.))*(d(x,-x)+J2)+.5;O1.zw=L6(.0);
#endif
uint Ca=i0&a4;if(Ca>w8){int o7=2;if((i0&na)==0u)o7=-o7;if((i0&G3)!=0u)o7=-o7;Y qg=n5(J4+o7);D4 rg=q1(MC,qg);float sg=X5(rg.z);float p7=abs(sg-e1);if(p7>D3)p7=o8-p7;bool H8=(i0&na)!=0u;bool tg=(i0&z8)!=0u;float ld=p7*(H8==tg?-.5:.5)+e1;d I8=d(sin(ld),-cos(ld));float Da=Yc(U0,I8);float q7=cos(p7*.5);float Ea;if((Ca==Df)||(Ca==Ef&&q7>=.25)){float ug=(i0&y8)!=0u?1.:.25;Ea=J2*(1./max(q7,ug));}else{Ea=J2*q7+Da*.5;}float Fa=Ea+Da*p4;if((i0&Hc)!=0u){float md=J2+L4;float vg=L4*.125;if(md<=Fa*q7+vg){float wg=md*(1./q7);p5=I8*wg;}else{d Ga=I8*Fa;d xg=d(dot(p5,p5),dot(Ga,Ga));p5=R0(xg,inverse(f0(p5,Ga)));}}d yg=abs(D1)*p5;float nd=(Fa-dot(yg,I8))/(Da*(p4*2.));
#ifndef CB
if((i0&z8)!=0u)O1.y=nd;else O1.x=nd;
#endif
}
#ifndef CB
O1.xy*=kd;O1.y=max(O1.y,1e-4);if(K2!=.0){O1.x=h7-O1.x;}
#endif
G8=R0(U0,D1*p5);if(l7!=A8)return false;}else{
#ifndef CB
O1=g(xa,-1.,.0,.0);
#ifdef HB
if(K2!=.0){O1.y=h7;O1.z=Uc;O1.w=xa;if((i0&a4)==x8&&l7==A8){if(v1<.0){m7+=v1;v1=-v1;}float f4=e1-m7;f4=mod(f4+V6,o8)-V6;f4=clamp(f4,.0,v1);if(f4>v1*.5){f4=v1-f4;}d C8=d(sin(f4),cos(f4));
#if 0
float P1=1.+.33*log2(V6/(D3-min(v1,D3-D3/16.)));g zg=Vc(v1,C8,.5*(P1/3.));float Ag=c8(zg d1);float Bg=nc(Ag);float Cg=(.5-Bg)*(ma*2.);float Dg=P1/max(Cg,P1);D1*=Dg;
#endif
O1=Vc(v1,C8,D1);}G8=R0(U0,(D1*K2)*Y2);}else
#endif
{G8=sign(R0(D1*Y2,inverse(U0)))*p4;}if(bool(i0&G3)!=bool(i0&Ff)){O1*=g(-1.,+1.,+1.,+1.);}
#endif
if(l7==Lc)jd=ad;if((i0&Gc)!=0u&&l7!=Kc){return false;}}og=R0(U0,jd)+G8+k3;
#ifdef CB
G M4=J0(QB,e3*4u+2u);k7=W1(M4.x);
#else
O1.xy=mix(O1.xy,d(1.,-1.),nf(m.Eg!=0u));
#endif
return true;}
#endif
#if defined(DB)&&defined(EB)
e d Gb(R l6,Z0(uint)e3
#ifdef CB
,Z0(N)k7
#else
,Z0(c)Fg
#endif
k6){e3=floatBitsToUint(l6.z)&0xffffu;
#ifdef CB
G M4=J0(QB,e3*4u+2u);k7=W1(M4.x);
#else
Fg=ba(floatBitsToInt(l6.z)>>16);
#endif
d m6=l6.xy;f0 U0=g2(uintBitsToFloat(J0(QB,e3*4u)));G K4=J0(QB,e3*4u+1u);d k3=uintBitsToFloat(K4.xy);m6=R0(U0,m6)+k3;return m6;}
#endif
#if defined(DB)&&defined(FB)
e d Fb(R l6,Z0(uint)e3,
#ifdef CB
Z0(N)k7,
#endif
Z0(d)Gg k6){e3=floatBitsToUint(l6.z)&0xffffu;G M4=J0(QB,e3*4u+2u);
#ifdef CB
k7=W1(M4.x);
#endif
d m6=l6.xy;R r7=uintBitsToFloat(M4.yzw);Gg=(m6*r7.x+r7.yz)*m.Hg;return m6;}
#endif
e c J8(c Z1,c E1,c r2){return(E1-Z1)/max(1.-Z1*r2,o9);}e uint K8(a1 n6,uint Ig){uint Ha=(n6.y>>g6)*(Ig<<g6)+((n6.x>>g6)<<(g6<<1));Ha+=((n6.x&0x1cu)<<g6)+((n6.y&0x1cu)<<2);Ha+=((n6.y&0x3u)<<2)+(n6.x&0x3u);return Ha;}
#ifdef TB
#ifdef Q
#define e5 o2
#define Y3(q5) C1=q5;n3
#else
#define e5 L1
#define Y3(q5) x0(j0,q5);Y1;
#endif
e c Ia(uint Jg){return ba(int((Jg&ra)-k5))*pa;}e uint v7(c n){return uint(n*Pf+.5);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive