#define W7 -2.
#define qe -1.5
#define re .25
#define H9 1e3
#define se (H9*H9)
#ifdef BB
v4 j5(v3,Gh,VB);
#ifdef IB
M6(v3,L7,AD);
#endif
w4 a5 k5(fe,ki,KB);m6(fd,og,XC);n6(gd,pg,JB);k5(ge,li,DD);c5
#endif
#if defined(IB)||defined(FB)
d4(L7,db)
#endif
#ifdef EB
X3 q3(v3,he,ZC);
#if defined(IB)||defined(FB)
M6(v3,L7,AD);
#endif
#ifdef FB
Q5(v3,ie,JD);
#endif
q3(C5,x4,UB);
#if defined(CB)&&defined(H)&&!defined(R)&&!defined(QB)
R5(MD);
#endif
Y3
#ifdef QB
F1 S5(x2,RC);G1
#endif
N6(he,Q8)
#ifdef FB
d4(ie,Sa)
#endif
D5 y4(W4) E5
#endif
#ifdef EB
f bool r6(e U){return U.y>=.0;}f bool r6(E U){return U.y>=.0;}
#endif
#if defined(EB)&&defined(IB)
f bool kd(e U){return U.x<qe;}f bool ld(e U){return U.y<qe;}
#endif
#ifdef BB
e te(float Nb,c I9,float I0){c O6=(1.-I9*abs(I0))*.5;float D4,T5;if(abs(Nb-B7)<1./H9){D4=.0;T5=.0;}else{float Ob=tan(Nb);D4=sign(B7-Nb)/max(abs(Ob),1./se);T5=D4>=.0?O6.y-(1.-O6.x)*Ob:O6.y+O6.x*Ob;}e U;U.x=max(O6.x,.0)+re;U.y=-O6.y+W7;U.z=D4;U.w=T5;return U;}
#endif
#ifdef IB
f d T8(e U e4){d D4=U.z;d T5=max(U.w,.0);d P6=D4>=.0?H5(T5):.0;if(abs(D4)<H9){d x=abs(U.x)-re;d y=-U.y+W7;d p3=(y-T5)*0.5984134206;i t=T5+p3*H0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-D4+(y*D4+x);i mi=H0(H5(u[0]),H5(u[1]),H5(u[2]),H5(u[3]));i ue=t*5.09593080173+-2.54796540086;i ni=exp2(-ue*ue);P6+=dot(mi,ni)*p3;}return P6*sign(U.x);}f d V4(e U e4){float P6=1.;float oi=(1.-W7)+U.x;P6-=H5(oi);float pi=1.-U.y;P6-=H5(pi);return P6;}
#endif
#ifdef BB
f g0 A4(int ve){return g0(ve&((1<<Ud)-1),ve>>Ud);}f float E9(uint z){return float(z)*(h9/(65536.*65536.));}f float ne(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(BB)&&defined(SD)
f float we(X O0,c qi){c B2=C0(O0,qi);return(abs(B2.x)+abs(B2.y))*(1./dot(B2,B2));}f bool ra(e X7,e Pb,int r,l1(uint) x3,l1(c) ri
#ifndef CB
,l1(e) f2
#else
,l1(P) Y7
#endif
Q6){int J9=int(X7.x);float I0=X7.y;float Qb=X7.z;int xe=floatBitsToInt(X7.w)>>2;int Z7=floatBitsToInt(X7.w)&3;int L5=min(J9,xe-1);int a4=r*xe+L5;O d2=v1(VB,A4(a4));uint Z=d2.w;uint H6=max(Z&xb,1u);O M5=q0(DD,H6-1u);c Q7=uintBitsToFloat(M5.xy);x3=M5.z&0xffffu;uint C9=M5.w;X O0=q1(uintBitsToFloat(q0(KB,x3*4u)));O L3=q0(KB,x3*4u+1u);c y2=uintBitsToFloat(L3.xy);float C2=uintBitsToFloat(L3.z);float Y2=uintBitsToFloat(L3.w);uint J6=Z&X2;if(J6!=0u){J9=int(Pb.x);I0=Pb.y;Qb=Pb.z;}if(J9!=L5){int D9=a4+J9-L5;O S7=v1(VB,A4(D9));if((S7.w&(X2|0xffffu))!=(Z&(X2|0xffffu))){bool T7=C2==.0||Q7.x!=.0;if(T7){a4=int(C9);d2=v1(VB,A4(a4));}}else{a4=D9;d2=S7;}Z=(d2.w&~X2)|J6;}bool Rb=false;float g1;
#ifdef IB
float a8;float K1;if((Z&r3)==p9&&Z7==r9){uint ye=d2.z;float E4=float(ye&0xffffu);float D2=float(ye>>16);g0 K9=g0(-E4-1.,D2-E4+1.);if((Z&X2)!=0u) K9=-K9;O ze=v1(VB,A4(a4+K9.x));O Sb=v1(VB,A4(a4+K9.y));if((Sb.w&(X2|0xffffu))!=(ze.w&(X2|0xffffu))){Sb=v1(VB,A4(int(C9)));}a8=E9(ze.z);float Ae=E9(Sb.z);K1=Ae-a8;if(abs(K1)>r4) K1-=h9*sign(K1);float Tb=D2+1.-float(Yd);float Be=clamp(round(abs(K1)/r4*Tb),1.,Tb-1.);float c8=Tb-Be;if(E4<=c8){K1=-(r4*sign(K1)-K1);D2=c8;if(E4==c8) I0=-I0;}else if(E4==c8+1.){E4=.0;D2=.0;I0=.0;}else{E4-=c8+2.;D2=Be;}if(E4==D2){g1=Ae;}else{g1=a8+K1*(E4/D2);}}else
#endif
{g1=E9(d2.z);}c S1=c(sin(g1),-cos(g1));c K6=uintBitsToFloat(d2.xy);c L9=c(0,0);if(Y2!=.0){Y2=max(Y2,(sb/3.)/length(C0(O0,S1)));}if(C2!=.0){I0*=sign(determinant(O0));if((Z&C6)!=0u) I0=min(I0,.0);if((Z&wb)!=0u) I0=max(I0,.0);float l5=Y2!=.0?Y2:we(O0,S1)*Q4;d Ce=1.;if(l5>C2&&Y2==.0){Ce=x5(C2)/x5(l5);C2=l5;}c M3=S1*(C2+l5);
#ifndef CB
float x=I0*(C2+l5);f2.xy=(1./(l5*2.))*(c(x,-x)+C2)+.5;f2.zw=p7(.0);
#endif
uint h5=Z&r3;if(h5>K7){bool L6=(Z&ub)!=0u;bool Ib=(Z&C6)!=0u;float e2=ne(d2.z);float B4=sqrt(max(1.-e2*e2,.0));if(L6==Ib) B4=-B4;X Jb=X(e2,B4,-B4,e2);c i5=C0(Jb,S1);float Ub=we(O0,i5);float Vb;if((h5==tb)||(h5==th&&e2>=.25)){float si=(Z&q9)!=0u?1.:.25;Vb=C2*(1./max(e2,si));}else{Vb=C2*e2+Ub*.5;}float Wb=Vb+Ub*Q4;if((Z&vb)!=0u){float De=C2+l5;float ti=l5*.125;if(De<=Wb*e2+ti){float ui=De*(1./e2);M3=i5*ui;}else{c Xb=i5*Wb;c vi=c(dot(M3,M3),dot(Xb,Xb));M3=C0(vi,inverse(X(M3,Xb)));}}c wi=abs(I0)*M3;float Ee=(Wb-dot(wi,i5))/(Ub*(Q4*2.));
#ifndef CB
if((Z&C6)!=0u) f2.y=Ee;else f2.x=Ee;
#endif
}
#ifndef CB
f2.xy*=Ce;f2.y=max(f2.y,1e-4);if(Y2!=.0){f2.x=W7-f2.x;}
#endif
L9=C0(O0,I0*M3);if(Z7!=r9) Rb=true;}else{
#ifndef CB
f2=e(Qb,-1.,.0,.0);
#ifdef IB
if(Y2!=.0){f2.y=W7;f2.z=se;f2.w=Qb;if((Z&r3)==p9&&Z7==r9){if(K1<.0){a8+=K1;K1=-K1;}float F4=g1-a8;F4=mod(F4+B7,h9)-B7;F4=clamp(F4,.0,K1);if(F4>K1*.5){F4=K1-F4;}c I9=c(sin(F4),cos(F4));
#if 0
float g2=1.+.33*log2(B7/(r4-min(K1,r4-r4/16.)));e xi=te(K1,I9,.5*(g2/3.));float yi=T8(xi n1);float zi=Hd(yi);float Ai=(.5-zi)*(sb*2.);float Bi=g2/max(Ai,g2);I0*=Bi;
#endif
f2=te(K1,I9,I0);}L9=C0(O0,(I0*Y2)*S1);}else
#endif
{L9=sign(C0(I0*S1,inverse(O0)))*Q4;}if(bool(Z&X2)!=bool(Z&wh)){f2*=e(-1.,+1.,+1.,+1.);}
#endif
if(Z7==de) K6=Q7;if((Z&be)!=0u&&Z7!=ce){Rb=true;}}ri=C0(O0,K6)+L9+y2;
#ifdef CB
O c4=q0(KB,x3*4u+2u);Y7=X1(c4.x);
#else
f2.xy=mix(f2.xy,c(1.,-1.),Qg(j.Ci!=0u));
#endif
return!Rb;}
#endif
#if defined(BB)&&defined(DB)
f c cd(M R6,l1(uint) x3
#ifdef CB
,l1(P) Y7
#else
,l1(d) Di
#endif
Q6){x3=floatBitsToUint(R6.z)&0xffffu;
#ifdef CB
O c4=q0(KB,x3*4u+2u);Y7=X1(c4.x);
#else
Di=fb(floatBitsToInt(R6.z)>>16);
#endif
c S6=R6.xy;X O0=q1(uintBitsToFloat(q0(KB,x3*4u)));O L3=q0(KB,x3*4u+1u);c y2=uintBitsToFloat(L3.xy);S6=C0(O0,S6)+y2;return S6;}
#endif
#if defined(BB)&&defined(FB)
f c bd(M R6,l1(uint) x3,
#ifdef CB
l1(P) Y7,
#endif
l1(c) Ei Q6){x3=floatBitsToUint(R6.z)&0xffffu;O c4=q0(KB,x3*4u+2u);
#ifdef CB
Y7=X1(c4.x);
#endif
c S6=R6.xy;M d8=uintBitsToFloat(c4.yzw);Ei=(S6*d8.x+d8.yz)*j.Fi;return S6;}
#endif
f d M9(d q2,d T1,d y3){return(T1-q2)/max(1.-q2*y3,qa);}
#if defined(RB)||defined(ND)
f uint N9(R0 w3,uint Gi){uint Yb=(w3.y>>F6)*(Gi<<F6)+((w3.x>>F6)<<(F6<<1));Yb+=((w3.x&0x1cu)<<F6)+((w3.y&0x1cu)<<2);Yb+=((w3.y&0x3u)<<2)+(w3.x&0x3u);return Yb;}
#endif
#ifdef RB
#ifdef R
#define G5 H2
#define z4(U5) Q1=U5;F3
#else
#define G5 I1
#define z4(U5) x0(n0,U5);O1;
#endif
f d Zb(uint Hi){return fb(int((Hi&Db)-J5))*Bb;}f uint e8(d n){return uint(n*Nh+.5);}
#endif
