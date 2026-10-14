#define U7 -2.
#define le -1.5
#define me .25
#define F9 1e3
#define ne (F9*F9)
#ifdef BB
q4 i5(r3,Ah,UB);
#ifdef HB
I6(r3,J7,YC);
#endif
r4 Y4 j5(ae,ei,KB);j6(bd,jg,VC);k6(cd,kg,JB);j5(be,fi,BD);Z4
#endif
#if defined(HB)||defined(FB)
a4(J7,ab)
#endif
#ifdef EB
V3 q3(r3,ce,XC);
#if defined(HB)||defined(FB)
I6(r3,J7,YC);
#endif
#ifdef FB
N5(r3,de,HD);
#endif
q3(B5,v4,TB);
#if defined(CB)&&defined(H)&&!defined(U)
O5(KD);
#endif
W3 J6(ce,O8)
#ifdef FB
a4(de,Pa)
#endif
C5 w4(U4) D5
#endif
#ifdef EB
e bool o6(f T){return T.y>=.0;}e bool o6(D T){return T.y>=.0;}
#endif
#if defined(EB)&&defined(HB)
e bool gd(f T){return T.x<le;}e bool hd(f T){return T.y<le;}
#endif
#ifdef BB
f oe(float Jb,c G9,float I0){c K6=(1.-G9*abs(I0))*.5;float A4,P5;if(abs(Jb-y7)<1./F9){A4=.0;P5=.0;}else{float Kb=tan(Jb);A4=sign(y7-Jb)/max(abs(Kb),1./ne);P5=A4>=.0?K6.y-(1.-K6.x)*Kb:K6.y+K6.x*Kb;}f T;T.x=max(K6.x,.0)+me;T.y=-K6.y+U7;T.z=A4;T.w=P5;return T;}
#endif
#ifdef HB
e d R8(f T c4){d A4=T.z;d P5=max(T.w,.0);d L6=A4>=.0?G5(P5):.0;if(abs(A4)<F9){d x=abs(T.x)-me;d y=-T.y+U7;d p3=(y-P5)*0.5984134206;i t=P5+p3*H0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-A4+(y*A4+x);i gi=H0(G5(u[0]),G5(u[1]),G5(u[2]),G5(u[3]));i pe=t*5.09593080173+-2.54796540086;i hi=exp2(-pe*pe);L6+=dot(gi,hi)*p3;}return L6*sign(T.x);}e d T4(f T c4){float L6=1.;float ii=(1.-U7)+T.x;L6-=G5(ii);float ji=1.-T.y;L6-=G5(ji);return L6;}
#endif
#ifdef BB
e g0 y4(int qe){return g0(qe&((1<<Od)-1),qe>>Od);}e float C9(uint z){return float(z)*(f9/(65536.*65536.));}e float ie(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(BB)&&defined(PD)
e float re(X N0,c ki){c A2=B0(N0,ki);return(abs(A2.x)+abs(A2.y))*(1./dot(A2,A2));}e bool pa(f V7,f Lb,int r,k1(uint) w3,k1(c) li
#ifndef CB
,k1(f) d2
#else
,k1(P) W7
#endif
M6){int H9=int(V7.x);float I0=V7.y;float Mb=V7.z;int se=floatBitsToInt(V7.w)>>2;int X7=floatBitsToInt(V7.w)&3;int K5=min(H9,se-1);int Y3=r*se+K5;O a2=q1(UB,y4(Y3));uint a0=a2.w;uint D6=max(a0&ub,1u);O L5=p0(BD,D6-1u);c O7=uintBitsToFloat(L5.xy);w3=L5.z&0xffffu;uint A9=L5.w;X N0=o1(uintBitsToFloat(p0(KB,w3*4u)));O L3=p0(KB,w3*4u+1u);c x2=uintBitsToFloat(L3.xy);float B2=uintBitsToFloat(L3.z);float Y2=uintBitsToFloat(L3.w);uint F6=a0&X2;if(F6!=0u){H9=int(Lb.x);I0=Lb.y;Mb=Lb.z;}if(H9!=K5){int B9=Y3+H9-K5;O Q7=q1(UB,y4(B9));if((Q7.w&(X2|0xffffu))!=(a0&(X2|0xffffu))){bool R7=B2==.0||O7.x!=.0;if(R7){Y3=int(A9);a2=q1(UB,y4(Y3));}}else{Y3=B9;a2=Q7;}a0=(a2.w&~X2)|F6;}bool Nb=false;float f1;
#ifdef HB
float Y7;float F1;if((a0&J3)==n9&&X7==p9){uint te=a2.z;float B4=float(te&0xffffu);float C2=float(te>>16);g0 I9=g0(-B4-1.,C2-B4+1.);if((a0&X2)!=0u) I9=-I9;O ue=q1(UB,y4(Y3+I9.x));O Ob=q1(UB,y4(Y3+I9.y));if((Ob.w&(X2|0xffffu))!=(ue.w&(X2|0xffffu))){Ob=q1(UB,y4(int(A9)));}Y7=C9(ue.z);float ve=C9(Ob.z);F1=ve-Y7;if(abs(F1)>p4) F1-=f9*sign(F1);float Pb=C2+1.-float(Sd);float we=clamp(round(abs(F1)/p4*Pb),1.,Pb-1.);float Z7=Pb-we;if(B4<=Z7){F1=-(p4*sign(F1)-F1);C2=Z7;if(B4==Z7) I0=-I0;}else if(B4==Z7+1.){B4=.0;C2=.0;I0=.0;}else{B4-=Z7+2.;C2=we;}if(B4==C2){f1=ve;}else{f1=Y7+F1*(B4/C2);}}else
#endif
{f1=C9(a2.z);}c N1=c(sin(f1),-cos(f1));c G6=uintBitsToFloat(a2.xy);c J9=c(0,0);if(Y2!=.0){Y2=max(Y2,(pb/3.)/length(B0(N0,N1)));}if(B2!=.0){I0*=sign(determinant(N0));if((a0&y6)!=0u) I0=min(I0,.0);if((a0&tb)!=0u) I0=max(I0,.0);float k5=Y2!=.0?Y2:re(N0,N1)*O4;d xe=1.;if(k5>B2&&Y2==.0){xe=w5(B2)/w5(k5);B2=k5;}c M3=N1*(B2+k5);
#ifndef CB
float x=I0*(B2+k5);d2.xy=(1./(k5*2.))*(c(x,-x)+B2)+.5;d2.zw=m7(.0);
#endif
uint f5=a0&J3;if(f5>I7){bool H6=(a0&rb)!=0u;bool Eb=(a0&y6)!=0u;float c2=ie(a2.z);float z4=sqrt(max(1.-c2*c2,.0));if(H6==Eb) z4=-z4;X Fb=X(c2,z4,-z4,c2);c g5=B0(Fb,N1);float Qb=re(N0,g5);float Rb;if((f5==qb)||(f5==oh&&c2>=.25)){float mi=(a0&o9)!=0u?1.:.25;Rb=B2*(1./max(c2,mi));}else{Rb=B2*c2+Qb*.5;}float Sb=Rb+Qb*O4;if((a0&sb)!=0u){float ye=B2+k5;float ni=k5*.125;if(ye<=Sb*c2+ni){float oi=ye*(1./c2);M3=g5*oi;}else{c Tb=g5*Sb;c pi=c(dot(M3,M3),dot(Tb,Tb));M3=B0(pi,inverse(X(M3,Tb)));}}c qi=abs(I0)*M3;float ze=(Sb-dot(qi,g5))/(Qb*(O4*2.));
#ifndef CB
if((a0&y6)!=0u) d2.y=ze;else d2.x=ze;
#endif
}
#ifndef CB
d2.xy*=xe;d2.y=max(d2.y,1e-4);if(Y2!=.0){d2.x=U7-d2.x;}
#endif
J9=B0(N0,I0*M3);if(X7!=p9) Nb=true;}else{
#ifndef CB
d2=f(Mb,-1.,.0,.0);
#ifdef HB
if(Y2!=.0){d2.y=U7;d2.z=ne;d2.w=Mb;if((a0&J3)==n9&&X7==p9){if(F1<.0){Y7+=F1;F1=-F1;}float C4=f1-Y7;C4=mod(C4+y7,f9)-y7;C4=clamp(C4,.0,F1);if(C4>F1*.5){C4=F1-C4;}c G9=c(sin(C4),cos(C4));
#if 0
float e2=1.+.33*log2(y7/(p4-min(F1,p4-p4/16.)));f ri=oe(F1,G9,.5*(e2/3.));float si=R8(ri m1);float ti=Cd(si);float ui=(.5-ti)*(pb*2.);float vi=e2/max(ui,e2);I0*=vi;
#endif
d2=oe(F1,G9,I0);}J9=B0(N0,(I0*Y2)*N1);}else
#endif
{J9=sign(B0(I0*N1,inverse(N0)))*O4;}if(bool(a0&X2)!=bool(a0&qh)){d2*=f(-1.,+1.,+1.,+1.);}
#endif
if(X7==Xd) G6=O7;if((a0&Vd)!=0u&&X7!=Wd){Nb=true;}}li=B0(N0,G6)+J9+x2;
#ifdef CB
O Z3=p0(KB,w3*4u+2u);W7=S1(Z3.x);
#else
d2.xy=mix(d2.xy,c(1.,-1.),Lg(j.wi!=0u));
#endif
return!Nb;}
#endif
#if defined(BB)&&defined(DB)
e c Yc(M N6,k1(uint) w3
#ifdef CB
,k1(P) W7
#else
,k1(d) xi
#endif
M6){w3=floatBitsToUint(N6.z)&0xffffu;
#ifdef CB
O Z3=p0(KB,w3*4u+2u);W7=S1(Z3.x);
#else
xi=cb(floatBitsToInt(N6.z)>>16);
#endif
c O6=N6.xy;X N0=o1(uintBitsToFloat(p0(KB,w3*4u)));O L3=p0(KB,w3*4u+1u);c x2=uintBitsToFloat(L3.xy);O6=B0(N0,O6)+x2;return O6;}
#endif
#if defined(BB)&&defined(FB)
e c Xc(M N6,k1(uint) w3,
#ifdef CB
k1(P) W7,
#endif
k1(c) yi M6){w3=floatBitsToUint(N6.z)&0xffffu;O Z3=p0(KB,w3*4u+2u);
#ifdef CB
W7=S1(Z3.x);
#endif
c O6=N6.xy;M a8=uintBitsToFloat(Z3.yzw);yi=(O6*a8.x+a8.yz)*j.zi;return O6;}
#endif
e d K9(d p2,d O1,d x3){return(O1-p2)/max(1.-p2*x3,oa);}
#if defined(QB)||defined(LD)
e uint L9(R0 v3,uint Ai){uint Ub=(v3.y>>B6)*(Ai<<B6)+((v3.x>>B6)<<(B6<<1));Ub+=((v3.x&0x1cu)<<B6)+((v3.y&0x1cu)<<2);Ub+=((v3.y&0x3u)<<2)+(v3.x&0x3u);return Ub;}
#endif
#ifdef QB
#ifdef U
#define F5 G2
#define x4(Q5) L1=Q5;E3
#else
#define F5 X1
#define x4(Q5) y0(n0,Q5);o2;
#endif
e d Vb(uint Bi){return cb(int((Bi&zb)-I5))*xb;}e uint c8(d n){return uint(n*Hh+.5);}
#endif
