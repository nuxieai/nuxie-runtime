#pragma once

#include "draw_path_common.glsl.exports.h"

namespace rive {
namespace gpu {
namespace glsl {
const char draw_path_common[] = R"===(#define X7 -2.
#define xe -1.5
#define ye .25
#define I9 1e3
#define ze (I9*I9)
#ifdef BB
x4 l5(v3,Jh,VB);
#ifdef IB
M6(v3,L7,AD);
#endif
y4 c5 m5(le,pi,KB);m6(id,ug,XC);n6(jd,vg,JB);m5(me,qi,DD);d5
#endif
#if defined(IB)||defined(FB)
e4(L7,eb)
#endif
#ifdef DB
Y3 q3(v3,ne,ZC);
#if defined(IB)||defined(FB)
M6(v3,L7,AD);
#endif
#ifdef FB
Q5(v3,oe,JD);
#endif
q3(D5,z4,RB);
#if defined(CB)&&defined(H)&&!defined(R)&&!defined(QB)
R5(MD);
#endif
Z3
#ifdef QB
F1 S5(x2,RC);G1
#endif
N6(ne,S8)
#ifdef FB
e4(oe,Ta)
#endif
E5 A4(v4) F5
#endif
#ifdef DB
f bool r6(e U){return U.y>=.0;}f bool r6(E U){return U.y>=.0;}
#endif
#ifdef DB
f d we(i Qb,uint D3){d Ae=dot(Qb.xyz,Z0(.30,.59,.11));if(D3==Gh) return Qb.w;if(D3==Hh) return 1.-Qb.w;if(D3==Ih) return Ae;return 1.-Ae;}
#endif
#if defined(DB)&&defined(IB)
f bool nd(e U){return U.x<xe;}f bool od(e U){return U.y<xe;}
#endif
#ifdef BB
e Be(float Rb,c J9,float J0){c O6=(1.-J9*abs(J0))*.5;float F4,T5;if(abs(Rb-B7)<1./I9){F4=.0;T5=.0;}else{float Sb=tan(Rb);F4=sign(B7-Rb)/max(abs(Sb),1./ze);T5=F4>=.0?O6.y-(1.-O6.x)*Sb:O6.y+O6.x*Sb;}e U;U.x=max(O6.x,.0)+ye;U.y=-O6.y+X7;U.z=F4;U.w=T5;return U;}
#endif
#ifdef IB
f d V8(e U f4){d F4=U.z;d T5=max(U.w,.0);d P6=F4>=.0?H5(T5):.0;if(abs(F4)<I9){d x=abs(U.x)-ye;d y=-U.y+X7;d p3=(y-T5)*0.5984134206;i t=T5+p3*E0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-F4+(y*F4+x);i ri=E0(H5(u[0]),H5(u[1]),H5(u[2]),H5(u[3]));i Ce=t*5.09593080173+-2.54796540086;i si=exp2(-Ce*Ce);P6+=dot(ri,si)*p3;}return P6*sign(U.x);}f d X4(e U f4){float P6=1.;float ti=(1.-X7)+U.x;P6-=H5(ti);float ui=1.-U.y;P6-=H5(ui);return P6;}
#endif
#ifdef BB
f g0 C4(int De){return g0(De&((1<<Xd)-1),De>>Xd);}f float F9(uint z){return float(z)*(A7/(65536.*65536.));}f float te(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(BB)&&defined(SD)
f float Ee(X R0,c vi){c B2=D0(R0,vi);return(abs(B2.x)+abs(B2.y))*(1./dot(B2,B2));}f bool sa(e Y7,e Tb,int r,l1(uint) x3,l1(c) wi
#ifndef CB
,l1(e) f2
#else
,l1(P) Z7
#endif
Q6){int K9=int(Y7.x);float J0=Y7.y;float Ub=Y7.z;int Fe=floatBitsToInt(Y7.w)>>2;int a8=floatBitsToInt(Y7.w)&3;int L5=min(K9,Fe-1);int c4=r*Fe+L5;O d2=v1(VB,C4(c4));uint Z=d2.w;uint H6=max(Z&yb,1u);O M5=q0(DD,H6-1u);c Q7=uintBitsToFloat(M5.xy);x3=M5.z&0xffffu;uint D9=M5.w;X R0=q1(uintBitsToFloat(q0(KB,x3*4u)));O M3=q0(KB,x3*4u+1u);c y2=uintBitsToFloat(M3.xy);float C2=uintBitsToFloat(M3.z);float Y2=uintBitsToFloat(M3.w);uint J6=Z&X2;if(J6!=0u){K9=int(Tb.x);J0=Tb.y;Ub=Tb.z;}if(K9!=L5){int E9=c4+K9-L5;O S7=v1(VB,C4(E9));if((S7.w&(X2|0xffffu))!=(Z&(X2|0xffffu))){bool T7=C2==.0||Q7.x!=.0;if(T7){c4=int(D9);d2=v1(VB,C4(c4));}}else{c4=E9;d2=S7;}Z=(d2.w&~X2)|J6;}bool Vb=false;float g1;
#ifdef IB
float c8;float K1;if((Z&r3)==q9&&a8==v9){uint Ge=d2.z;float G4=float(Ge&0xffffu);float D2=float(Ge>>16);g0 L9=g0(-G4-1.,D2-G4+1.);if((Z&X2)!=0u) L9=-L9;O He=v1(VB,C4(c4+L9.x));O Wb=v1(VB,C4(c4+L9.y));if((Wb.w&(X2|0xffffu))!=(He.w&(X2|0xffffu))){Wb=v1(VB,C4(int(D9)));}c8=F9(He.z);float Ie=F9(Wb.z);K1=Ie-c8;if(abs(K1)>w4) K1-=A7*sign(K1);float Xb=D2+1.-float(be);float Je=clamp(round(abs(K1)/w4*Xb),1.,Xb-1.);float d8=Xb-Je;if(G4<=d8){K1=-(w4*sign(K1)-K1);D2=d8;if(G4==d8) J0=-J0;}else if(G4==d8+1.){G4=.0;D2=.0;J0=.0;}else{G4-=d8+2.;D2=Je;}if(G4==D2){g1=Ie;}else{g1=c8+K1*(G4/D2);}}else
#endif
{g1=F9(d2.z);}c S1=c(sin(g1),-cos(g1));c K6=uintBitsToFloat(d2.xy);c M9=c(0,0);if(Y2!=.0){Y2=max(Y2,(tb/3.)/length(D0(R0,S1)));}if(C2!=.0){J0*=sign(determinant(R0));if((Z&C6)!=0u) J0=min(J0,.0);if((Z&xb)!=0u) J0=max(J0,.0);float n5=Y2!=.0?Y2:Ee(R0,S1)*S4;d Ke=1.;if(n5>C2&&Y2==.0){Ke=T4(C2)/T4(n5);C2=n5;}c N3=S1*(C2+n5);
#ifndef CB
float x=J0*(C2+n5);f2.xy=(1./(n5*2.))*(c(x,-x)+C2)+.5;f2.zw=o7(.0);
#endif
uint j5=Z&r3;if(j5>K7){bool L6=(Z&vb)!=0u;bool Jb=(Z&C6)!=0u;float e2=te(d2.z);float D4=sqrt(max(1.-e2*e2,.0));if(L6==Jb) D4=-D4;X Kb=X(e2,D4,-D4,e2);c k5=D0(Kb,S1);float Yb=Ee(R0,k5);float Zb;if((j5==ub)||(j5==zh&&e2>=.25)){float xi=(Z&r9)!=0u?1.:.25;Zb=C2*(1./max(e2,xi));}else{Zb=C2*e2+Yb*.5;}float ac=Zb+Yb*S4;if((Z&wb)!=0u){float Le=C2+n5;float yi=n5*.125;if(Le<=ac*e2+yi){float zi=Le*(1./e2);N3=k5*zi;}else{c bc=k5*ac;c Ai=c(dot(N3,N3),dot(bc,bc));N3=D0(Ai,inverse(X(N3,bc)));}}c Bi=abs(J0)*N3;float Me=(ac-dot(Bi,k5))/(Yb*(S4*2.));
#ifndef CB
if((Z&C6)!=0u) f2.y=Me;else f2.x=Me;
#endif
}
#ifndef CB
f2.xy*=Ke;f2.y=max(f2.y,1e-4);if(Y2!=.0){f2.x=X7-f2.x;}
#endif
M9=D0(R0,J0*N3);if(a8!=v9) Vb=true;}else{
#ifndef CB
f2=e(Ub,-1.,.0,.0);
#ifdef IB
if(Y2!=.0){f2.y=X7;f2.z=ze;f2.w=Ub;if((Z&r3)==q9&&a8==v9){if(K1<.0){c8+=K1;K1=-K1;}float H4=g1-c8;H4=mod(H4+B7,A7)-B7;H4=clamp(H4,.0,K1);if(H4>K1*.5){H4=K1-H4;}c J9=c(sin(H4),cos(H4));
#if 0
float g2=1.+.33*log2(B7/(w4-min(K1,w4-w4/16.)));e Ci=Be(K1,J9,.5*(g2/3.));float Di=V8(Ci n1);float Ei=Kd(Di);float Fi=(.5-Ei)*(tb*2.);float Gi=g2/max(Fi,g2);J0*=Gi;
#endif
f2=Be(K1,J9,J0);}M9=D0(R0,(J0*Y2)*S1);}else
#endif
{M9=sign(D0(J0*S1,inverse(R0)))*S4;}if(bool(Z&X2)!=bool(Z&Ch)){f2*=e(-1.,+1.,+1.,+1.);}
#endif
if(a8==ge) K6=Q7;if((Z&ee)!=0u&&a8!=fe){Vb=true;}}wi=D0(R0,K6)+M9+y2;
#ifdef CB
O d4=q0(KB,x3*4u+2u);Z7=X1(d4.x);
#else
f2.xy=mix(f2.xy,c(1.,-1.),Wg(j.Hi!=0u));
#endif
return!Vb;}
#endif
#if defined(BB)&&defined(EB)
f c fd(M R6,l1(uint) x3
#ifdef CB
,l1(P) Z7
#else
,l1(d) Ii
#endif
Q6){x3=floatBitsToUint(R6.z)&0xffffu;
#ifdef CB
O d4=q0(KB,x3*4u+2u);Z7=X1(d4.x);
#else
Ii=gb(floatBitsToInt(R6.z)>>16);
#endif
c S6=R6.xy;X R0=q1(uintBitsToFloat(q0(KB,x3*4u)));O M3=q0(KB,x3*4u+1u);c y2=uintBitsToFloat(M3.xy);S6=D0(R0,S6)+y2;return S6;}
#endif
#if defined(BB)&&defined(FB)
f c ed(M R6,l1(uint) x3,
#ifdef CB
l1(P) Z7,
#endif
l1(c) Ji Q6){x3=floatBitsToUint(R6.z)&0xffffu;O d4=q0(KB,x3*4u+2u);
#ifdef CB
Z7=X1(d4.x);
#endif
c S6=R6.xy;M e8=uintBitsToFloat(d4.yzw);Ji=(S6*e8.x+e8.yz)*j.Ki;return S6;}
#endif
f d N9(d q2,d T1,d y3){return(T1-q2)/max(1.-q2*y3,ra);}
#if defined(SB)||defined(ND)
f uint O9(T0 w3,uint Li){uint cc=(w3.y>>F6)*(Li<<F6)+((w3.x>>F6)<<(F6<<1));cc+=((w3.x&0x1cu)<<F6)+((w3.y&0x1cu)<<2);cc+=((w3.y&0x3u)<<2)+(w3.x&0x3u);return cc;}
#endif
#ifdef SB
#ifdef R
#define G5 H2
#define B4(U5) Q1=U5;G3
#else
#define G5 I1
#define B4(U5) x0(n0,U5);O1;
#endif
f d dc(uint Mi){return gb(int((Mi&Eb)-J5))*Cb;}f uint f8(d n){return uint(n*Qh+.5);}
#endif
)===";
} // namespace glsl
} // namespace gpu
} // namespace rive