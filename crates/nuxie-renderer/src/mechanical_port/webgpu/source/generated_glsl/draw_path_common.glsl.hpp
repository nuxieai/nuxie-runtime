#pragma once

#include "draw_path_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_common[] = R"===(#define m7 -2.
#define jd -1.5
#define kd .25
#define F8 1e3
#define ld (F8*F8)
#ifdef CB
Y3 Bc(h3,lg,JC);
#ifdef GB
l6(h3,k7,XC);
#endif
Z3 F4 N4(dd,Jg,OB);R5(Ub,pf,CD);S5(Vb,qf,PB);N4(ed,Kg,HD);G4
#endif
#if defined(GB)||defined(FB)
g4(k7,ha)
#endif
#ifdef EB
I3 e3(h3,fd,DD);
#if defined(GB)||defined(FB)
l6(h3,k7,XC);
#endif
#ifdef FB
q5(h3,gd,ED);
#endif
e3(h5,a4,GC);
#if defined(BB)&&defined(S)&&!defined(Q)
r5(XD);
#endif
J3 g4(fd,P9)
#ifdef FB
g4(gd,U9)
#endif
i5 c4(Z5)j5
#endif
#ifdef EB
e bool Y5(f P){return P.y>=.0;}e bool Y5(E P){return P.y>=.0;}
#endif
#if defined(EB)&&defined(GB)
e bool fc(f P){return P.x<jd;}e bool gc(f P){return P.y<jd;}
#endif
#ifdef CB
f md(float Ea,c G8,float G1){c m6=(1.-G8*abs(G1))*.5;float h4,v5;if(abs(Ea-a7)<1./F8){h4=.0;v5=.0;}else{float Fa=tan(Ea);h4=sign(a7-Ea)/max(abs(Fa),1./ld);v5=h4>=.0?m6.y-(1.-m6.x)*Fa:m6.y+m6.x*Fa;}f P;P.x=max(m6.x,.0)+kd;P.y=-m6.y+m7;P.z=h4;P.w=v5;return P;}
#endif
#ifdef GB
e d g8(f P L3){d h4=P.z;d v5=max(P.w,.0);d n6=h4>=.0?n5(v5):.0;if(abs(h4)<F8){d x=abs(P.x)-kd;d y=-P.y+m7;d c3=(y-v5)*0.5984134206;i t=v5+c3*E0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-h4+(y*h4+x);i Lg=E0(n5(u[0]),n5(u[1]),n5(u[2]),n5(u[3]));i nd=t*5.09593080173+-2.54796540086;i Mg=exp2(-nd*nd);n6+=dot(Lg,Mg)*c3;}return n6*sign(P.x);}e d C4(f P L3){float n6=1.;float Ng=(1.-m7)+P.x;n6-=n5(Ng);float Og=1.-P.y;n6-=n5(Og);return n6;}
#endif
#if defined(CB)&&defined(ND)
e Z w5(int od){return Z(od&((1<<Sc)-1),od>>Sc);}e float pd(e0 W0,c Pg){c m2=P0(W0,Pg);return(abs(m2.x)+abs(m2.y))*(1./dot(m2,m2));}e bool x9(f n7,f Ga,int v,c1(uint)i3,c1(c)Qg
#ifndef BB
,c1(f)S1
#else
,c1(N)o7
#endif
o6){int H8=int(n7.x);float G1=n7.y;float Ha=n7.z;int qd=floatBitsToInt(n7.w)>>2;int p7=floatBitsToInt(n7.w)&3;int Ia=min(H8,qd-1);int O4=v*qd+Ia;H4 x5=v1(JC,w5(O4));uint j0=m5(x5.w);uint I8=max(j0&Zc,1u);Y Ja=L0(HD,I8-1u);c rd=uintBitsToFloat(Ja.xy);i3=Ja.z&0xffffu;uint sd=Ja.w;e0 W0=L1(uintBitsToFloat(L0(OB,i3*4u)));Y P4=L0(OB,i3*4u+1u);c I2=uintBitsToFloat(P4.xy);float N2=uintBitsToFloat(P4.z);float O2=uintBitsToFloat(P4.w);uint td=j0&K3;if(td!=0u){H8=int(Ga.x);G1=Ga.y;Ha=Ga.z;}if(H8!=Ia){int ud=O4+H8-Ia;H4 vd=v1(JC,w5(ud));if((m5(vd.w)&(K3|0xffffu))!=(j0&(K3|0xffffu))){bool Rg=N2==.0||rd.x!=.0;if(Rg){O4=int(sd);x5=v1(JC,w5(O4));}}else{O4=ud;x5=vd;}j0=(m5(x5.w)&~K3)|td;}float f1;
#ifdef GB
float q7;float x1;if((j0&f4)==B8&&p7==E8){uint wd=m5(x5.z);float i4=float(wd&0xffffu);float n2=float(wd>>16);Z J8=Z(-i4-1.,n2-i4+1.);if((j0&K3)!=0u)J8=-J8;H4 xd=v1(JC,w5(O4+J8.x));H4 Ka=v1(JC,w5(O4+J8.y));if((m5(Ka.w)&(K3|0xffffu))!=(m5(xd.w)&(K3|0xffffu))){Ka=v1(JC,w5(int(sd)));}q7=d6(xd.z);float yd=d6(Ka.z);x1=yd-q7;if(abs(x1)>H3)x1-=v8*sign(x1);float La=n2+1.-float(Tc);float zd=clamp(round(abs(x1)/H3*La),1.,La-1.);float r7=La-zd;if(i4<=r7){x1=-(H3*sign(x1)-x1);n2=r7;if(i4==r7)G1=-G1;}else if(i4==r7+1.){i4=.0;n2=.0;G1=.0;}else{i4-=r7+2.;n2=zd;}if(i4==n2){f1=yd;}else{f1=q7+x1*(i4/n2);}}else
#endif
{f1=d6(x5.z);}c d3=c(sin(f1),-cos(f1));c Ad=d6(x5.xy);c K8=c(0,0);if(O2!=.0){O2=max(O2,(va/3.)/length(P0(W0,d3)));}if(N2!=.0){G1*=sign(determinant(W0));if((j0&D8)!=0u)G1=min(G1,.0);if((j0&Yc)!=0u)G1=max(G1,.0);float Q4=O2!=.0?O2:pd(W0,d3)*x4;d Bd=1.;if(Q4>N2&&O2==.0){Bd=S3(N2)/S3(Q4);N2=Q4;}c y5=d3*(N2+Q4);
#ifndef BB
float x=G1*(N2+Q4);S1.xy=(1./(Q4*2.))*(c(x,-x)+N2)+.5;S1.zw=Q6(.0);
#endif
uint Ma=j0&f4;if(Ma>A8){int v7=2;if((j0&wa)==0u)v7=-v7;if((j0&K3)!=0u)v7=-v7;Z Sg=w5(O4+v7);H4 Tg=v1(JC,Sg);float Ug=d6(Tg.z);float w7=abs(Ug-f1);if(w7>H3)w7=v8-w7;bool L8=(j0&wa)!=0u;bool Vg=(j0&D8)!=0u;float Cd=w7*(L8==Vg?-.5:.5)+f1;c M8=c(sin(Cd),-cos(Cd));float Na=pd(W0,M8);float x7=cos(w7*.5);float Oa;if((Ma==gg)||(Ma==hg&&x7>=.25)){float Wg=(j0&C8)!=0u?1.:.25;Oa=N2*(1./max(x7,Wg));}else{Oa=N2*x7+Na*.5;}float Pa=Oa+Na*x4;if((j0&Xc)!=0u){float Dd=N2+Q4;float Xg=Q4*.125;if(Dd<=Pa*x7+Xg){float Yg=Dd*(1./x7);y5=M8*Yg;}else{c Qa=M8*Pa;c Zg=c(dot(y5,y5),dot(Qa,Qa));y5=P0(Zg,inverse(e0(y5,Qa)));}}c ah=abs(G1)*y5;float Ed=(Pa-dot(ah,M8))/(Na*(x4*2.));
#ifndef BB
if((j0&D8)!=0u)S1.y=Ed;else S1.x=Ed;
#endif
}
#ifndef BB
S1.xy*=Bd;S1.y=max(S1.y,1e-4);if(O2!=.0){S1.x=m7-S1.x;}
#endif
K8=P0(W0,G1*y5);if(p7!=E8)return false;}else{
#ifndef BB
S1=f(Ha,-1.,.0,.0);
#ifdef GB
if(O2!=.0){S1.y=m7;S1.z=ld;S1.w=Ha;if((j0&f4)==B8&&p7==E8){if(x1<.0){q7+=x1;x1=-x1;}float j4=f1-q7;j4=mod(j4+a7,v8)-a7;j4=clamp(j4,.0,x1);if(j4>x1*.5){j4=x1-j4;}c G8=c(sin(j4),cos(j4));
#if 0
float T1=1.+.33*log2(a7/(H3-min(x1,H3-H3/16.)));f bh=md(x1,G8,.5*(T1/3.));float ch=g8(bh e1);float dh=Ec(ch);float eh=(.5-dh)*(va*2.);float fh=T1/max(eh,T1);G1*=fh;
#endif
S1=md(x1,G8,G1);}K8=P0(W0,(G1*O2)*d3);}else
#endif
{K8=sign(P0(G1*d3,inverse(W0)))*x4;}if(bool(j0&K3)!=bool(j0&ig)){S1*=f(-1.,+1.,+1.,+1.);}
#endif
if(p7==bd)Ad=rd;if((j0&Wc)!=0u&&p7!=ad){return false;}}Qg=P0(W0,Ad)+K8+I2;
#ifdef BB
Y R4=L0(OB,i3*4u+2u);o7=a2(R4.x);
#else
S1.xy=mix(S1.xy,c(1.,-1.),Qf(j.gh!=0u));
#endif
return true;}
#endif
#if defined(CB)&&defined(DB)
e c Ob(R p6,c1(uint)i3
#ifdef BB
,c1(N)o7
#else
,c1(d)hh
#endif
o6){i3=floatBitsToUint(p6.z)&0xffffu;
#ifdef BB
Y R4=L0(OB,i3*4u+2u);o7=a2(R4.x);
#else
hh=ia(floatBitsToInt(p6.z)>>16);
#endif
c q6=p6.xy;e0 W0=L1(uintBitsToFloat(L0(OB,i3*4u)));Y P4=L0(OB,i3*4u+1u);c I2=uintBitsToFloat(P4.xy);q6=P0(W0,q6)+I2;return q6;}
#endif
#if defined(CB)&&defined(FB)
e c Nb(R p6,c1(uint)i3,
#ifdef BB
c1(N)o7,
#endif
c1(c)ih o6){i3=floatBitsToUint(p6.z)&0xffffu;Y R4=L0(OB,i3*4u+2u);
#ifdef BB
o7=a2(R4.x);
#endif
c q6=p6.xy;R y7=uintBitsToFloat(R4.yzw);ih=(q6*y7.x+y7.yz)*j.jh;return q6;}
#endif
e d N8(d e2,d H1,d j3){return(H1-e2)/max(1.-e2*j3,v9);}
#if defined(QB)||defined(ID)
e uint O8(N0 k4,uint kh){uint Ra=(k4.y>>k6)*(kh<<k6)+((k4.x>>k6)<<(k6<<1));Ra+=((k4.x&0x1cu)<<k6)+((k4.y&0x1cu)<<2);Ra+=((k4.y&0x3u)<<2)+(k4.x&0x3u);return Ra;}
#endif
#ifdef QB
#ifdef Q
#define k5 v2
#define d4(z5) F1=z5;r3
#else
#define k5 P1
#define d4(z5) A0(m0,z5);d2;
#endif
e d Sa(uint lh){return ia(int((lh&Ba)-p5))*za;}e uint z7(d o){return uint(o*rg+.5);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive