#pragma once

#include "draw_path_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_common[] = R"===(#define k7 -2.
#define dd -1.5
#define ed .25
#define D8 1e3
#define fd (D8*D8)
#ifdef DB
V3 vc(e3,eg,KC);
#ifdef HB
k6(e3,i7,YC);
#endif
W3 B4 J4(Xc,Cg,PB);O5(Rb,jf,DD);P5(Sb,kf,QB);J4(Yc,Dg,ID);C4
#endif
#if defined(HB)||defined(GB)
d4(i7,ea)
#endif
#ifdef FB
F3 c3(e3,Zc,ED);
#if defined(HB)||defined(GB)
k6(e3,i7,YC);
#endif
#ifdef GB
m5(e3,ad,FD);
#endif
c3(d5,X3,HC);
#if defined(CB)&&defined(AB)&&!defined(O)
n5(YD);
#endif
G3 d4(Zc,N9)
#ifdef GB
d4(ad,R9)
#endif
e5 Y3(X5)f5
#endif
#ifdef FB
e bool W5(f N){return N.y>=.0;}e bool W5(E N){return N.y>=.0;}
#endif
#if defined(FB)&&defined(HB)
e bool Zb(f N){return N.x<dd;}e bool ac(f N){return N.y<dd;}
#endif
#ifdef DB
f gd(float Ba,c E8,float D1){c l6=(1.-E8*abs(D1))*.5;float e4,o5;if(abs(Ba-Y6)<1./D8){e4=.0;o5=.0;}else{float Ca=tan(Ba);e4=sign(Y6-Ba)/max(abs(Ca),1./fd);o5=e4>=.0?l6.y-(1.-l6.x)*Ca:l6.y+l6.x*Ca;}f N;N.x=max(l6.x,.0)+ed;N.y=-l6.y+k7;N.z=e4;N.w=o5;return N;}
#endif
#ifdef HB
e d e8(f N I3){d e4=N.z;d o5=max(N.w,.0);d m6=e4>=.0?j5(o5):.0;if(abs(e4)<D8){d x=abs(N.x)-ed;d y=-N.y+k7;d Z2=(y-o5)*0.5984134206;i t=o5+Z2*D0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-e4+(y*e4+x);i Eg=D0(j5(u[0]),j5(u[1]),j5(u[2]),j5(u[3]));i hd=t*5.09593080173+-2.54796540086;i Fg=exp2(-hd*hd);m6+=dot(Eg,Fg)*Z2;}return m6*sign(N.x);}e d y4(f N I3){float m6=1.;float Gg=(1.-k7)+N.x;m6-=j5(Gg);float Hg=1.-N.y;m6-=j5(Hg);return m6;}
#endif
#if defined(DB)&&defined(OD)
e Y p5(int id){return Y(id&((1<<Mc)-1),id>>Mc);}e float jd(d0 U0,c Ig){c l2=N0(U0,Ig);return(abs(l2.x)+abs(l2.y))*(1./dot(l2,l2));}e bool v9(f l7,f Da,int v,Z0(uint)f3,Z0(c)Jg
#ifndef CB
,Z0(f)P1
#else
,Z0(L)m7
#endif
n6){int F8=int(l7.x);float D1=l7.y;float Ea=l7.z;int kd=floatBitsToInt(l7.w)>>2;int n7=floatBitsToInt(l7.w)&3;int Fa=min(F8,kd-1);int K4=v*kd+Fa;D4 q5=p1(KC,p5(K4));uint i0=i5(q5.w);uint G8=max(i0&Tc,1u);X Ga=K0(ID,G8-1u);c ld=uintBitsToFloat(Ga.xy);f3=Ga.z&0xffffu;uint md=Ga.w;d0 U0=I1(uintBitsToFloat(K0(PB,f3*4u)));X L4=K0(PB,f3*4u+1u);c I2=uintBitsToFloat(L4.xy);float M2=uintBitsToFloat(L4.z);float N2=uintBitsToFloat(L4.w);uint nd=i0&H3;if(nd!=0u){F8=int(Da.x);D1=Da.y;Ea=Da.z;}if(F8!=Fa){int od=K4+F8-Fa;D4 pd=p1(KC,p5(od));if((i5(pd.w)&(H3|0xffffu))!=(i0&(H3|0xffffu))){bool Kg=M2==.0||ld.x!=.0;if(Kg){K4=int(md);q5=p1(KC,p5(K4));}}else{K4=od;q5=pd;}i0=(i5(q5.w)&~H3)|nd;}float e1;
#ifdef HB
float o7;float r1;if((i0&c4)==z8&&n7==C8){uint qd=i5(q5.z);float f4=float(qd&0xffffu);float m2=float(qd>>16);Y H8=Y(-f4-1.,m2-f4+1.);if((i0&H3)!=0u)H8=-H8;D4 rd=p1(KC,p5(K4+H8.x));D4 Ha=p1(KC,p5(K4+H8.y));if((i5(Ha.w)&(H3|0xffffu))!=(i5(rd.w)&(H3|0xffffu))){Ha=p1(KC,p5(int(md)));}o7=a6(rd.z);float sd=a6(Ha.z);r1=sd-o7;if(abs(r1)>E3)r1-=q8*sign(r1);float Ia=m2+1.-float(Nc);float td=clamp(round(abs(r1)/E3*Ia),1.,Ia-1.);float p7=Ia-td;if(f4<=p7){r1=-(E3*sign(r1)-r1);m2=p7;if(f4==p7)D1=-D1;}else if(f4==p7+1.){f4=.0;m2=.0;D1=.0;}else{f4-=p7+2.;m2=td;}if(f4==m2){e1=sd;}else{e1=o7+r1*(f4/m2);}}else
#endif
{e1=a6(q5.z);}c a3=c(sin(e1),-cos(e1));c ud=a6(q5.xy);c I8=c(0,0);if(N2!=.0){N2=max(N2,(sa/3.)/length(N0(U0,a3)));}if(M2!=.0){D1*=sign(determinant(U0));if((i0&B8)!=0u)D1=min(D1,.0);if((i0&Sc)!=0u)D1=max(D1,.0);float M4=N2!=.0?N2:jd(U0,a3)*r4;d vd=1.;if(M4>M2&&N2==.0){vd=W4(M2)/W4(M4);M2=M4;}c r5=a3*(M2+M4);
#ifndef CB
float x=D1*(M2+M4);P1.xy=(1./(M4*2.))*(c(x,-x)+M2)+.5;P1.zw=O6(.0);
#endif
uint Ja=i0&c4;if(Ja>y8){int q7=2;if((i0&ta)==0u)q7=-q7;if((i0&H3)!=0u)q7=-q7;Y Lg=p5(K4+q7);D4 Mg=p1(KC,Lg);float Ng=a6(Mg.z);float r7=abs(Ng-e1);if(r7>E3)r7=q8-r7;bool J8=(i0&ta)!=0u;bool Og=(i0&B8)!=0u;float wd=r7*(J8==Og?-.5:.5)+e1;c K8=c(sin(wd),-cos(wd));float Ka=jd(U0,K8);float v7=cos(r7*.5);float La;if((Ja==Zf)||(Ja==ag&&v7>=.25)){float Pg=(i0&A8)!=0u?1.:.25;La=M2*(1./max(v7,Pg));}else{La=M2*v7+Ka*.5;}float Ma=La+Ka*r4;if((i0&Rc)!=0u){float xd=M2+M4;float Qg=M4*.125;if(xd<=Ma*v7+Qg){float Rg=xd*(1./v7);r5=K8*Rg;}else{c Na=K8*Ma;c Sg=c(dot(r5,r5),dot(Na,Na));r5=N0(Sg,inverse(d0(r5,Na)));}}c Tg=abs(D1)*r5;float yd=(Ma-dot(Tg,K8))/(Ka*(r4*2.));
#ifndef CB
if((i0&B8)!=0u)P1.y=yd;else P1.x=yd;
#endif
}
#ifndef CB
P1.xy*=vd;P1.y=max(P1.y,1e-4);if(N2!=.0){P1.x=k7-P1.x;}
#endif
I8=N0(U0,D1*r5);if(n7!=C8)return false;}else{
#ifndef CB
P1=f(Ea,-1.,.0,.0);
#ifdef HB
if(N2!=.0){P1.y=k7;P1.z=fd;P1.w=Ea;if((i0&c4)==z8&&n7==C8){if(r1<.0){o7+=r1;r1=-r1;}float g4=e1-o7;g4=mod(g4+Y6,q8)-Y6;g4=clamp(g4,.0,r1);if(g4>r1*.5){g4=r1-g4;}c E8=c(sin(g4),cos(g4));
#if 0
float Q1=1.+.33*log2(Y6/(E3-min(r1,E3-E3/16.)));f Ug=gd(r1,E8,.5*(Q1/3.));float Vg=e8(Ug d1);float Wg=yc(Vg);float Xg=(.5-Wg)*(sa*2.);float Yg=Q1/max(Xg,Q1);D1*=Yg;
#endif
P1=gd(r1,E8,D1);}I8=N0(U0,(D1*N2)*a3);}else
#endif
{I8=sign(N0(D1*a3,inverse(U0)))*r4;}if(bool(i0&H3)!=bool(i0&bg)){P1*=f(-1.,+1.,+1.,+1.);}
#endif
if(n7==Vc)ud=ld;if((i0&Qc)!=0u&&n7!=Uc){return false;}}Jg=N0(U0,ud)+I8+I2;
#ifdef CB
X N4=K0(PB,f3*4u+2u);m7=Y1(N4.x);
#else
P1.xy=mix(P1.xy,c(1.,-1.),Jf(l.Zg!=0u));
#endif
return true;}
#endif
#if defined(DB)&&defined(EB)
e c Lb(Q o6,Z0(uint)f3
#ifdef CB
,Z0(L)m7
#else
,Z0(d)ah
#endif
n6){f3=floatBitsToUint(o6.z)&0xffffu;
#ifdef CB
X N4=K0(PB,f3*4u+2u);m7=Y1(N4.x);
#else
ah=fa(floatBitsToInt(o6.z)>>16);
#endif
c p6=o6.xy;d0 U0=I1(uintBitsToFloat(K0(PB,f3*4u)));X L4=K0(PB,f3*4u+1u);c I2=uintBitsToFloat(L4.xy);p6=N0(U0,p6)+I2;return p6;}
#endif
#if defined(DB)&&defined(GB)
e c Kb(Q o6,Z0(uint)f3,
#ifdef CB
Z0(L)m7,
#endif
Z0(c)bh n6){f3=floatBitsToUint(o6.z)&0xffffu;X N4=K0(PB,f3*4u+2u);
#ifdef CB
m7=Y1(N4.x);
#endif
c p6=o6.xy;Q w7=uintBitsToFloat(N4.yzw);bh=(p6*w7.x+w7.yz)*l.ch;return p6;}
#endif
e d L8(d c2,d E1,d x2){return(E1-c2)/max(1.-c2*x2,q9);}
#if defined(RB)||defined(JD)
e uint M8(a1 h4,uint dh){uint Oa=(h4.y>>j6)*(dh<<j6)+((h4.x>>j6)<<(j6<<1));Oa+=((h4.x&0x1cu)<<j6)+((h4.y&0x1cu)<<2);Oa+=((h4.y&0x3u)<<2)+(h4.x&0x3u);return Oa;}
#endif
#ifdef RB
#ifdef O
#define g5 r2
#define Z3(v5) C1=v5;m3
#else
#define g5 M1
#define Z3(v5) z0(k0,v5);a2;
#endif
e d Pa(uint eh){return fa(int((eh&ya)-l5))*wa;}e uint x7(d o){return uint(o*kg+.5);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive