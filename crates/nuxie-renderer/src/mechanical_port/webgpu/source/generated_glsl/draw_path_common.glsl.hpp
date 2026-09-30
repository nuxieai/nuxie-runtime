#pragma once

#include "draw_path_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_common[] = R"===(#define m7 -2.
#define kd -1.5
#define ld .25
#define F8 1e3
#define md (F8*F8)
#ifdef CB
Y3 Cc(h3,mg,JC);
#ifdef GB
l6(h3,k7,XC);
#endif
Z3 F4 N4(ed,Kg,OB);R5(Vb,qf,CD);S5(Wb,rf,PB);N4(fd,Lg,HD);G4
#endif
#if defined(GB)||defined(FB)
g4(k7,ha)
#endif
#ifdef EB
I3 e3(h3,gd,DD);
#if defined(GB)||defined(FB)
l6(h3,k7,XC);
#endif
#ifdef FB
q5(h3,hd,ED);
#endif
e3(h5,a4,GC);
#if defined(BB)&&defined(S)&&!defined(Q)
r5(XD);
#endif
J3 g4(gd,P9)
#ifdef FB
g4(hd,U9)
#endif
i5 c4(Z5)j5
#endif
#ifdef EB
e bool Y5(f P){return P.y>=.0;}e bool Y5(E P){return P.y>=.0;}
#endif
#if defined(EB)&&defined(GB)
e bool gc(f P){return P.x<kd;}e bool hc(f P){return P.y<kd;}
#endif
#ifdef CB
f nd(float Ea,c G8,float G1){c m6=(1.-G8*abs(G1))*.5;float h4,v5;if(abs(Ea-a7)<1./F8){h4=.0;v5=.0;}else{float Fa=tan(Ea);h4=sign(a7-Ea)/max(abs(Fa),1./md);v5=h4>=.0?m6.y-(1.-m6.x)*Fa:m6.y+m6.x*Fa;}f P;P.x=max(m6.x,.0)+ld;P.y=-m6.y+m7;P.z=h4;P.w=v5;return P;}
#endif
#ifdef GB
e d g8(f P L3){d h4=P.z;d v5=max(P.w,.0);d n6=h4>=.0?n5(v5):.0;if(abs(h4)<F8){d x=abs(P.x)-ld;d y=-P.y+m7;d c3=(y-v5)*0.5984134206;i t=v5+c3*E0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-h4+(y*h4+x);i Mg=E0(n5(u[0]),n5(u[1]),n5(u[2]),n5(u[3]));i od=t*5.09593080173+-2.54796540086;i Ng=exp2(-od*od);n6+=dot(Mg,Ng)*c3;}return n6*sign(P.x);}e d C4(f P L3){float n6=1.;float Og=(1.-m7)+P.x;n6-=n5(Og);float Pg=1.-P.y;n6-=n5(Pg);return n6;}
#endif
#if defined(CB)&&defined(ND)
e Z w5(int pd){return Z(pd&((1<<Tc)-1),pd>>Tc);}e float qd(e0 W0,c Qg){c m2=P0(W0,Qg);return(abs(m2.x)+abs(m2.y))*(1./dot(m2,m2));}e bool x9(f n7,f Ga,int v,c1(uint)i3,c1(c)Rg
#ifndef BB
,c1(f)S1
#else
,c1(N)o7
#endif
o6){int H8=int(n7.x);float G1=n7.y;float Ha=n7.z;int rd=floatBitsToInt(n7.w)>>2;int p7=floatBitsToInt(n7.w)&3;int Ia=min(H8,rd-1);int O4=v*rd+Ia;H4 x5=v1(JC,w5(O4));uint j0=m5(x5.w);uint I8=max(j0&ad,1u);Y Ja=L0(HD,I8-1u);c sd=uintBitsToFloat(Ja.xy);i3=Ja.z&0xffffu;uint td=Ja.w;e0 W0=L1(uintBitsToFloat(L0(OB,i3*4u)));Y P4=L0(OB,i3*4u+1u);c I2=uintBitsToFloat(P4.xy);float N2=uintBitsToFloat(P4.z);float O2=uintBitsToFloat(P4.w);uint ud=j0&K3;if(ud!=0u){H8=int(Ga.x);G1=Ga.y;Ha=Ga.z;}if(H8!=Ia){int vd=O4+H8-Ia;H4 wd=v1(JC,w5(vd));if((m5(wd.w)&(K3|0xffffu))!=(j0&(K3|0xffffu))){bool Sg=N2==.0||sd.x!=.0;if(Sg){O4=int(td);x5=v1(JC,w5(O4));}}else{O4=vd;x5=wd;}j0=(m5(x5.w)&~K3)|ud;}bool Ka=false;float f1;
#ifdef GB
float q7;float x1;if((j0&f4)==B8&&p7==E8){uint xd=m5(x5.z);float i4=float(xd&0xffffu);float n2=float(xd>>16);Z J8=Z(-i4-1.,n2-i4+1.);if((j0&K3)!=0u)J8=-J8;H4 yd=v1(JC,w5(O4+J8.x));H4 La=v1(JC,w5(O4+J8.y));if((m5(La.w)&(K3|0xffffu))!=(m5(yd.w)&(K3|0xffffu))){La=v1(JC,w5(int(td)));}q7=d6(yd.z);float zd=d6(La.z);x1=zd-q7;if(abs(x1)>H3)x1-=v8*sign(x1);float Ma=n2+1.-float(Uc);float Ad=clamp(round(abs(x1)/H3*Ma),1.,Ma-1.);float r7=Ma-Ad;if(i4<=r7){x1=-(H3*sign(x1)-x1);n2=r7;if(i4==r7)G1=-G1;}else if(i4==r7+1.){i4=.0;n2=.0;G1=.0;}else{i4-=r7+2.;n2=Ad;}if(i4==n2){f1=zd;}else{f1=q7+x1*(i4/n2);}}else
#endif
{f1=d6(x5.z);}c d3=c(sin(f1),-cos(f1));c Bd=d6(x5.xy);c K8=c(0,0);if(O2!=.0){O2=max(O2,(va/3.)/length(P0(W0,d3)));}if(N2!=.0){G1*=sign(determinant(W0));if((j0&D8)!=0u)G1=min(G1,.0);if((j0&Zc)!=0u)G1=max(G1,.0);float Q4=O2!=.0?O2:qd(W0,d3)*x4;d Cd=1.;if(Q4>N2&&O2==.0){Cd=S3(N2)/S3(Q4);N2=Q4;}c y5=d3*(N2+Q4);
#ifndef BB
float x=G1*(N2+Q4);S1.xy=(1./(Q4*2.))*(c(x,-x)+N2)+.5;S1.zw=Q6(.0);
#endif
uint Na=j0&f4;if(Na>A8){int v7=2;if((j0&wa)==0u)v7=-v7;if((j0&K3)!=0u)v7=-v7;Z Tg=w5(O4+v7);H4 Ug=v1(JC,Tg);float Vg=d6(Ug.z);float w7=abs(Vg-f1);if(w7>H3)w7=v8-w7;bool L8=(j0&wa)!=0u;bool Wg=(j0&D8)!=0u;float Dd=w7*(L8==Wg?-.5:.5)+f1;c M8=c(sin(Dd),-cos(Dd));float Oa=qd(W0,M8);float x7=cos(w7*.5);float Pa;if((Na==hg)||(Na==ig&&x7>=.25)){float Xg=(j0&C8)!=0u?1.:.25;Pa=N2*(1./max(x7,Xg));}else{Pa=N2*x7+Oa*.5;}float Qa=Pa+Oa*x4;if((j0&Yc)!=0u){float Ed=N2+Q4;float Yg=Q4*.125;if(Ed<=Qa*x7+Yg){float Zg=Ed*(1./x7);y5=M8*Zg;}else{c Ra=M8*Qa;c ah=c(dot(y5,y5),dot(Ra,Ra));y5=P0(ah,inverse(e0(y5,Ra)));}}c bh=abs(G1)*y5;float Fd=(Qa-dot(bh,M8))/(Oa*(x4*2.));
#ifndef BB
if((j0&D8)!=0u)S1.y=Fd;else S1.x=Fd;
#endif
}
#ifndef BB
S1.xy*=Cd;S1.y=max(S1.y,1e-4);if(O2!=.0){S1.x=m7-S1.x;}
#endif
K8=P0(W0,G1*y5);if(p7!=E8)Ka=true;}else{
#ifndef BB
S1=f(Ha,-1.,.0,.0);
#ifdef GB
if(O2!=.0){S1.y=m7;S1.z=md;S1.w=Ha;if((j0&f4)==B8&&p7==E8){if(x1<.0){q7+=x1;x1=-x1;}float j4=f1-q7;j4=mod(j4+a7,v8)-a7;j4=clamp(j4,.0,x1);if(j4>x1*.5){j4=x1-j4;}c G8=c(sin(j4),cos(j4));
#if 0
float T1=1.+.33*log2(a7/(H3-min(x1,H3-H3/16.)));f ch=nd(x1,G8,.5*(T1/3.));float dh=g8(ch e1);float eh=Fc(dh);float fh=(.5-eh)*(va*2.);float gh=T1/max(fh,T1);G1*=gh;
#endif
S1=nd(x1,G8,G1);}K8=P0(W0,(G1*O2)*d3);}else
#endif
{K8=sign(P0(G1*d3,inverse(W0)))*x4;}if(bool(j0&K3)!=bool(j0&jg)){S1*=f(-1.,+1.,+1.,+1.);}
#endif
if(p7==cd)Bd=sd;if((j0&Xc)!=0u&&p7!=bd){Ka=true;}}Rg=P0(W0,Bd)+K8+I2;
#ifdef BB
Y R4=L0(OB,i3*4u+2u);o7=a2(R4.x);
#else
S1.xy=mix(S1.xy,c(1.,-1.),Rf(j.hh!=0u));
#endif
return!Ka;}
#endif
#if defined(CB)&&defined(DB)
e c Pb(R p6,c1(uint)i3
#ifdef BB
,c1(N)o7
#else
,c1(d)ih
#endif
o6){i3=floatBitsToUint(p6.z)&0xffffu;
#ifdef BB
Y R4=L0(OB,i3*4u+2u);o7=a2(R4.x);
#else
ih=ia(floatBitsToInt(p6.z)>>16);
#endif
c q6=p6.xy;e0 W0=L1(uintBitsToFloat(L0(OB,i3*4u)));Y P4=L0(OB,i3*4u+1u);c I2=uintBitsToFloat(P4.xy);q6=P0(W0,q6)+I2;return q6;}
#endif
#if defined(CB)&&defined(FB)
e c Ob(R p6,c1(uint)i3,
#ifdef BB
c1(N)o7,
#endif
c1(c)jh o6){i3=floatBitsToUint(p6.z)&0xffffu;Y R4=L0(OB,i3*4u+2u);
#ifdef BB
o7=a2(R4.x);
#endif
c q6=p6.xy;R y7=uintBitsToFloat(R4.yzw);jh=(q6*y7.x+y7.yz)*j.kh;return q6;}
#endif
e d N8(d e2,d H1,d j3){return(H1-e2)/max(1.-e2*j3,v9);}
#if defined(QB)||defined(ID)
e uint O8(N0 k4,uint lh){uint Sa=(k4.y>>k6)*(lh<<k6)+((k4.x>>k6)<<(k6<<1));Sa+=((k4.x&0x1cu)<<k6)+((k4.y&0x1cu)<<2);Sa+=((k4.y&0x3u)<<2)+(k4.x&0x3u);return Sa;}
#endif
#ifdef QB
#ifdef Q
#define k5 v2
#define d4(z5) F1=z5;r3
#else
#define k5 P1
#define d4(z5) A0(m0,z5);d2;
#endif
e d Ta(uint mh){return ia(int((mh&Ba)-p5))*za;}e uint z7(d o){return uint(o*sg+.5);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive