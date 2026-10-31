#pragma once

#include "draw_path_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_common[] = R"===(#define W7 -2.
#define xe -1.5
#define ye .25
#define H9 1e3
#define ze (H9*H9)
#ifdef BB
x4 l5(w3,Kh,VB);
#ifdef IB
K6(w3,L7,ZC);
#endif
y4 c5 m4(Ka,kd,JB);k6(ld,vg,KB);m4(me,qi,CD);d5
#endif
#if defined(IB)||defined(FB)
d4(L7,fb)
#endif
#ifdef DB
Y3 r3(w3,ne,YC);
#if defined(IB)||defined(FB)
K6(w3,L7,ZC);
#endif
#ifdef FB
O5(w3,oe,ID);
#endif
r3(B5,z4,RB);
#if defined(CB)&&defined(H)&&!defined(Q)&&!defined(QB)
P5(LD);
#endif
Z3
#ifdef QB
F1 Q5(z2,RC);G1
#endif
L6(ne,R8)
#ifdef FB
d4(oe,Ua)
#endif
C5 A4(v4) D5
#endif
#ifdef DB
f bool o6(e T){return T.y>=.0;}f bool o6(E T){return T.y>=.0;}
#endif
#ifdef DB
f d we(i Sb,uint E3){d Ae=dot(Sb.xyz,Y0(.30,.59,.11));if(E3==Hh) return Sb.w;if(E3==Ih) return 1.-Sb.w;if(E3==Jh) return Ae;return 1.-Ae;}
#endif
#if defined(DB)&&defined(IB)
f bool pd(e T){return T.x<xe;}f bool qd(e T){return T.y<xe;}
#endif
#ifdef BB
e Be(float Tb,c I9,float J0){c M6=(1.-I9*abs(J0))*.5;float F4,R5;if(abs(Tb-B7)<1./H9){F4=.0;R5=.0;}else{float Ub=tan(Tb);F4=sign(B7-Tb)/max(abs(Ub),1./ze);R5=F4>=.0?M6.y-(1.-M6.x)*Ub:M6.y+M6.x*Ub;}e T;T.x=max(M6.x,.0)+ye;T.y=-M6.y+W7;T.z=F4;T.w=R5;return T;}
#endif
#ifdef IB
f d U8(e T e4){d F4=T.z;d R5=max(T.w,.0);d N6=F4>=.0?F5(R5):.0;if(abs(F4)<H9){d x=abs(T.x)-ye;d y=-T.y+W7;d q3=(y-R5)*0.5984134206;i t=R5+q3*E0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-F4+(y*F4+x);i ri=E0(F5(u[0]),F5(u[1]),F5(u[2]),F5(u[3]));i Ce=t*5.09593080173+-2.54796540086;i si=exp2(-Ce*Ce);N6+=dot(ri,si)*q3;}return N6*sign(T.x);}f d X4(e T e4){float N6=1.;float ti=(1.-W7)+T.x;N6-=F5(ti);float ui=1.-T.y;N6-=F5(ui);return N6;}
#endif
#ifdef BB
f f0 C4(int De){return f0(De&((1<<Zd)-1),De>>Zd);}f float E9(uint z){return float(z)*(A7/(65536.*65536.));}f float te(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(BB)&&defined(RD)
f float Ee(X Q0,c vi){c D2=D0(Q0,vi);return(abs(D2.x)+abs(D2.y))*(1./dot(D2,D2));}f bool sa(e X7,e Vb,int r,k1(uint) y3,k1(c) wi
#ifndef CB
,k1(e) g2
#else
,k1(O) Y7
#endif
O6){int J9=int(X7.x);float J0=X7.y;float Wb=X7.z;int Fe=floatBitsToInt(X7.w)>>2;int P6=floatBitsToInt(X7.w)&3;int J5=min(J9,Fe-1);int c4=r*Fe+J5;R d2=v1(VB,C4(c4));uint Z=d2.w;uint F6=max(Z&zb,1u);R K5=n0(CD,F6-1u);c P7=uintBitsToFloat(K5.xy);y3=K5.z&0xffffu;uint C9=K5.w;X Q0=p1(uintBitsToFloat(n0(JB,y3*4u)));R r2=n0(JB,y3*4u+1u);c A2=uintBitsToFloat(r2.xy);float e2=.0;float P2=.0;float Ge=0.0;if((r2.z&Gh)!=0u){R Q6=n0(JB,y3*4u+2u);e2=uintBitsToFloat(Q6.x);P2=uintBitsToFloat(Q6.y);Ge=uintBitsToFloat(Q6.z);}uint H6=Z&Z2;if(H6!=0u){J9=int(Vb.x);J0=Vb.y;Wb=Vb.z;}if(J9!=J5){int D9=c4+J9-J5;R R7=v1(VB,C4(D9));if((R7.w&(Z2|0xffffu))!=(Z&(Z2|0xffffu))){bool S7=e2==.0||P7.x!=.0;if(S7){c4=int(C9);d2=v1(VB,C4(c4));}}else{c4=D9;d2=R7;}Z=(d2.w&~Z2)|H6;}bool Xb=false;float f1;
#ifdef IB
float Z7;float L1;if((Z&v3)==p9&&P6==r9){uint He=d2.z;float G4=float(He&0xffffu);float E2=float(He>>16);f0 K9=f0(-G4-1.,E2-G4+1.);if((Z&Z2)!=0u) K9=-K9;R Ie=v1(VB,C4(c4+K9.x));R Yb=v1(VB,C4(c4+K9.y));if((Yb.w&(Z2|0xffffu))!=(Ie.w&(Z2|0xffffu))){Yb=v1(VB,C4(int(C9)));}Z7=E9(Ie.z);float Je=E9(Yb.z);L1=Je-Z7;if(abs(L1)>w4) L1-=A7*sign(L1);float Zb=E2+1.-float(de);float Ke=clamp(round(abs(L1)/w4*Zb),1.,Zb-1.);float a8=Zb-Ke;if(G4<=a8){L1=-(w4*sign(L1)-L1);E2=a8;if(G4==a8) J0=-J0;}else if(G4==a8+1.){G4=.0;E2=.0;J0=.0;}else{G4-=a8+2.;E2=Ke;}if(G4==E2){f1=Je;}else{f1=Z7+L1*(G4/E2);}}else
#endif
{f1=E9(d2.z);}c K1=c(sin(f1),-cos(f1));c I6=uintBitsToFloat(d2.xy);c c8=c(0,0);if(P2!=.0){P2=max(P2,(ub/3.)/length(D0(Q0,K1)));}if(e2!=.0){J0*=sign(determinant(Q0));if((Z&z6)!=0u) J0=min(J0,.0);if((Z&yb)!=0u) J0=max(J0,.0);float m5=P2!=.0?P2:Ee(Q0,K1)*S4;d Le=1.;if(m5>e2&&P2==.0){Le=T4(e2)/T4(m5);e2=m5;}c N3=K1*(e2+m5);
#ifndef CB
float x=J0*(e2+m5);g2.xy=(1./(m5*2.))*(c(x,-x)+e2)+.5;g2.zw=o7(.0);
#endif
uint j5=Z&v3;if(j5>K7){bool J6=(Z&wb)!=0u;bool Lb=(Z&z6)!=0u;float f2=te(d2.z);float D4=sqrt(max(1.-f2*f2,.0));if(J6==Lb) D4=-D4;X Mb=X(f2,D4,-D4,f2);c k5=D0(Mb,K1);float ac=Ee(Q0,k5);float bc;if((j5==vb)||(j5==zh&&f2>=.25)){float xi=(Z&q9)!=0u?1.:.25;bc=e2*(1./max(f2,xi));}else{bc=e2*f2+ac*.5;}float cc=bc+ac*S4;if((Z&xb)!=0u){float Me=e2+m5;float yi=m5*.125;if(Me<=cc*f2+yi){float zi=Me*(1./f2);N3=k5*zi;}else{c dc=k5*cc;c Ai=c(dot(N3,N3),dot(dc,dc));N3=D0(Ai,inverse(X(N3,dc)));}}c Bi=abs(J0)*N3;float Ne=(cc-dot(Bi,k5))/(ac*(S4*2.));
#ifndef CB
if((Z&z6)!=0u) g2.y=Ne;else g2.x=Ne;
#endif
}
#ifndef CB
g2.xy*=Le;g2.y=max(g2.y,1e-4);if(P2!=.0){g2.x=W7-g2.x;}
#endif
c8=D0(Q0,J0*N3);if(P6!=r9) Xb=true;}else{
#ifndef CB
g2=e(Wb,-1.,.0,.0);
#ifdef IB
if(P2!=.0){g2.y=W7;g2.z=ze;g2.w=Wb;if((Z&v3)==p9&&P6==r9){if(L1<.0){Z7+=L1;L1=-L1;}float H4=f1-Z7;H4=mod(H4+B7,A7)-B7;H4=clamp(H4,.0,L1);if(H4>L1*.5){H4=L1-H4;}c I9=c(sin(H4),cos(H4));
#if 0
float h2=1.+.33*log2(B7/(w4-min(L1,w4-w4/16.)));e Ci=Be(L1,I9,.5*(h2/3.));float Di=U8(Ci m1);float Ei=Md(Di);float Fi=(.5-Ei)*(ub*2.);float Gi=h2/max(Fi,h2);J0*=Gi;
#endif
g2=Be(L1,I9,J0);}c8=D0(Q0,(J0*P2)*K1);}else
#endif
{c8=sign(D0(J0*K1,inverse(Q0)))*S4;if(P6!=Ab){c8+=K1*Ge;}}if(bool(Z&Z2)!=bool(Z&Ch)){g2*=e(-1.,+1.,+1.,+1.);}
#endif
if(P6==Ab) I6=P7;if((Z&ge)!=0u&&P6!=he){Xb=true;}}wi=D0(Q0,I6)+c8+A2;
#ifdef CB
Y7=X1(r2.z>>16u);
#else
g2.xy=mix(g2.xy,c(1.,-1.),Wg(j.Hi!=0u));
#endif
return!Xb;}
#endif
#if defined(BB)&&defined(EB)
f c hd(M R6,k1(uint) y3
#ifdef CB
,k1(O) Y7
#else
,k1(d) Ii
#endif
O6){y3=floatBitsToUint(R6.z)&0xffffu;R r2=n0(JB,y3*4u+1u);
#ifdef CB
Y7=X1(r2.z>>16u);
#else
Ii=hb(floatBitsToInt(R6.z)>>16);
#endif
c S6=R6.xy;X Q0=p1(uintBitsToFloat(n0(JB,y3*4u)));c A2=uintBitsToFloat(r2.xy);S6=D0(Q0,S6)+A2;return S6;}
#endif
#if defined(BB)&&defined(FB)
f c gd(M R6,k1(uint) y3,
#ifdef CB
k1(O) Y7,
#endif
k1(c) Ji O6){y3=floatBitsToUint(R6.z)&0xffffu;
#ifdef CB
Y7=X1(n0(JB,y3*4u+1u).z>>16u);
#endif
c S6=R6.xy;M d8=uintBitsToFloat(n0(JB,y3*4u+3u).xyz);Ji=(S6*d8.x+d8.yz)*j.Ki;return S6;}
#endif
f d L9(d v2,d T1,d z3){return(T1-v2)/max(1.-v2*z3,ra);}
#if defined(SB)||defined(MD)
f uint M9(U0 x3,uint Li){uint ec=(x3.y>>C6)*(Li<<C6)+((x3.x>>C6)<<(C6<<1));ec+=((x3.x&0x1cu)<<C6)+((x3.y&0x1cu)<<2);ec+=((x3.y&0x3u)<<2)+(x3.x&0x3u);return ec;}
#endif
#ifdef SB
#ifdef Q
#define E5 I2
#define B4(S5) R1=S5;H3
#else
#define E5 I1
#define B4(S5) x0(o0,S5);P1;
#endif
f d fc(uint Mi){return hb(int((Mi&Gb)-H5))*Eb;}f uint e8(d n){return uint(n*Rh+.5);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive