#define U7 -2.
#define xe -1.5
#define ye .25
#define G9 1e3
#define ze (G9*G9)
#ifdef BB
x4 l5(w3,Jh,VB);
#ifdef IB
K6(w3,J7,ZC);
#endif
y4 c5 m4(Ka,jd,JB);k6(kd,ug,KB);m4(me,pi,CD);d5
#endif
#if defined(IB)||defined(FB)
d4(J7,fb)
#endif
#ifdef DB
Y3 r3(w3,ne,YC);
#if defined(IB)||defined(FB)
K6(w3,J7,ZC);
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
L6(ne,Q8)
#ifdef FB
d4(oe,Ua)
#endif
C5 A4(v4) D5
#endif
#ifdef DB
f bool o6(e T){return T.y>=.0;}f bool o6(E T){return T.y>=.0;}
#endif
#ifdef DB
f d we(i Rb,uint E3){d Ae=dot(Rb.xyz,Y0(.30,.59,.11));if(E3==Gh) return Rb.w;if(E3==Hh) return 1.-Rb.w;if(E3==Ih) return Ae;return 1.-Ae;}
#endif
#if defined(DB)&&defined(IB)
f bool od(e T){return T.x<xe;}f bool pd(e T){return T.y<xe;}
#endif
#ifdef BB
e Be(float Sb,c H9,float J0){c M6=(1.-H9*abs(J0))*.5;float F4,R5;if(abs(Sb-z7)<1./G9){F4=.0;R5=.0;}else{float Tb=tan(Sb);F4=sign(z7-Sb)/max(abs(Tb),1./ze);R5=F4>=.0?M6.y-(1.-M6.x)*Tb:M6.y+M6.x*Tb;}e T;T.x=max(M6.x,.0)+ye;T.y=-M6.y+U7;T.z=F4;T.w=R5;return T;}
#endif
#ifdef IB
f d T8(e T e4){d F4=T.z;d R5=max(T.w,.0);d N6=F4>=.0?F5(R5):.0;if(abs(F4)<G9){d x=abs(T.x)-ye;d y=-T.y+U7;d q3=(y-R5)*0.5984134206;i t=R5+q3*E0(0.20888568955,0.62665706865,1.04442844776,1.46219982687);i u=t*-F4+(y*F4+x);i qi=E0(F5(u[0]),F5(u[1]),F5(u[2]),F5(u[3]));i Ce=t*5.09593080173+-2.54796540086;i ri=exp2(-Ce*Ce);N6+=dot(qi,ri)*q3;}return N6*sign(T.x);}f d X4(e T e4){float N6=1.;float si=(1.-U7)+T.x;N6-=F5(si);float ti=1.-T.y;N6-=F5(ti);return N6;}
#endif
#ifdef BB
f f0 C4(int De){return f0(De&((1<<Yd)-1),De>>Yd);}f float D9(uint z){return float(z)*(y7/(65536.*65536.));}f float te(uint z){return float(z&0xffffu)*(1./65535.);}
#endif
#if defined(BB)&&defined(RD)
f float Ee(X Q0,c ui){c D2=D0(Q0,ui);return(abs(D2.x)+abs(D2.y))*(1./dot(D2,D2));}f bool sa(e V7,e Ub,int r,k1(uint) y3,k1(c) vi
#ifndef CB
,k1(e) g2
#else
,k1(O) W7
#endif
O6){int I9=int(V7.x);float J0=V7.y;float Vb=V7.z;int Fe=floatBitsToInt(V7.w)>>2;int X7=floatBitsToInt(V7.w)&3;int J5=min(I9,Fe-1);int c4=r*Fe+J5;R d2=v1(VB,C4(c4));uint Z=d2.w;uint F6=max(Z&zb,1u);R K5=n0(CD,F6-1u);c N7=uintBitsToFloat(K5.xy);y3=K5.z&0xffffu;uint B9=K5.w;X Q0=p1(uintBitsToFloat(n0(JB,y3*4u)));R r2=n0(JB,y3*4u+1u);c A2=uintBitsToFloat(r2.xy);float e2=.0;float P2=.0;if((r2.z&Fh)!=0u){R Y7=n0(JB,y3*4u+2u);e2=uintBitsToFloat(Y7.x);P2=uintBitsToFloat(Y7.y);}uint H6=Z&Z2;if(H6!=0u){I9=int(Ub.x);J0=Ub.y;Vb=Ub.z;}if(I9!=J5){int C9=c4+I9-J5;R P7=v1(VB,C4(C9));if((P7.w&(Z2|0xffffu))!=(Z&(Z2|0xffffu))){bool Q7=e2==.0||N7.x!=.0;if(Q7){c4=int(B9);d2=v1(VB,C4(c4));}}else{c4=C9;d2=P7;}Z=(d2.w&~Z2)|H6;}bool Wb=false;float f1;
#ifdef IB
float Z7;float K1;if((Z&v3)==o9&&X7==q9){uint Ge=d2.z;float G4=float(Ge&0xffffu);float E2=float(Ge>>16);f0 J9=f0(-G4-1.,E2-G4+1.);if((Z&Z2)!=0u) J9=-J9;R He=v1(VB,C4(c4+J9.x));R Xb=v1(VB,C4(c4+J9.y));if((Xb.w&(Z2|0xffffu))!=(He.w&(Z2|0xffffu))){Xb=v1(VB,C4(int(B9)));}Z7=D9(He.z);float Ie=D9(Xb.z);K1=Ie-Z7;if(abs(K1)>w4) K1-=y7*sign(K1);float Yb=E2+1.-float(ce);float Je=clamp(round(abs(K1)/w4*Yb),1.,Yb-1.);float a8=Yb-Je;if(G4<=a8){K1=-(w4*sign(K1)-K1);E2=a8;if(G4==a8) J0=-J0;}else if(G4==a8+1.){G4=.0;E2=.0;J0=.0;}else{G4-=a8+2.;E2=Je;}if(G4==E2){f1=Ie;}else{f1=Z7+K1*(G4/E2);}}else
#endif
{f1=D9(d2.z);}c S1=c(sin(f1),-cos(f1));c I6=uintBitsToFloat(d2.xy);c K9=c(0,0);if(P2!=.0){P2=max(P2,(ub/3.)/length(D0(Q0,S1)));}if(e2!=.0){J0*=sign(determinant(Q0));if((Z&z6)!=0u) J0=min(J0,.0);if((Z&yb)!=0u) J0=max(J0,.0);float m5=P2!=.0?P2:Ee(Q0,S1)*S4;d Ke=1.;if(m5>e2&&P2==.0){Ke=T4(e2)/T4(m5);e2=m5;}c N3=S1*(e2+m5);
#ifndef CB
float x=J0*(e2+m5);g2.xy=(1./(m5*2.))*(c(x,-x)+e2)+.5;g2.zw=m7(.0);
#endif
uint j5=Z&v3;if(j5>I7){bool J6=(Z&wb)!=0u;bool Kb=(Z&z6)!=0u;float f2=te(d2.z);float D4=sqrt(max(1.-f2*f2,.0));if(J6==Kb) D4=-D4;X Lb=X(f2,D4,-D4,f2);c k5=D0(Lb,S1);float Zb=Ee(Q0,k5);float ac;if((j5==vb)||(j5==yh&&f2>=.25)){float wi=(Z&p9)!=0u?1.:.25;ac=e2*(1./max(f2,wi));}else{ac=e2*f2+Zb*.5;}float bc=ac+Zb*S4;if((Z&xb)!=0u){float Le=e2+m5;float xi=m5*.125;if(Le<=bc*f2+xi){float yi=Le*(1./f2);N3=k5*yi;}else{c cc=k5*bc;c zi=c(dot(N3,N3),dot(cc,cc));N3=D0(zi,inverse(X(N3,cc)));}}c Ai=abs(J0)*N3;float Me=(bc-dot(Ai,k5))/(Zb*(S4*2.));
#ifndef CB
if((Z&z6)!=0u) g2.y=Me;else g2.x=Me;
#endif
}
#ifndef CB
g2.xy*=Ke;g2.y=max(g2.y,1e-4);if(P2!=.0){g2.x=U7-g2.x;}
#endif
K9=D0(Q0,J0*N3);if(X7!=q9) Wb=true;}else{
#ifndef CB
g2=e(Vb,-1.,.0,.0);
#ifdef IB
if(P2!=.0){g2.y=U7;g2.z=ze;g2.w=Vb;if((Z&v3)==o9&&X7==q9){if(K1<.0){Z7+=K1;K1=-K1;}float H4=f1-Z7;H4=mod(H4+z7,y7)-z7;H4=clamp(H4,.0,K1);if(H4>K1*.5){H4=K1-H4;}c H9=c(sin(H4),cos(H4));
#if 0
float h2=1.+.33*log2(z7/(w4-min(K1,w4-w4/16.)));e Bi=Be(K1,H9,.5*(h2/3.));float Ci=T8(Bi m1);float Di=Ld(Ci);float Ei=(.5-Di)*(ub*2.);float Fi=h2/max(Ei,h2);J0*=Fi;
#endif
g2=Be(K1,H9,J0);}K9=D0(Q0,(J0*P2)*S1);}else
#endif
{K9=sign(D0(J0*S1,inverse(Q0)))*S4;}if(bool(Z&Z2)!=bool(Z&Bh)){g2*=e(-1.,+1.,+1.,+1.);}
#endif
if(X7==he) I6=N7;if((Z&fe)!=0u&&X7!=ge){Wb=true;}}vi=D0(Q0,I6)+K9+A2;
#ifdef CB
W7=X1(r2.z>>16u);
#else
g2.xy=mix(g2.xy,c(1.,-1.),Vg(j.Gi!=0u));
#endif
return!Wb;}
#endif
#if defined(BB)&&defined(EB)
f c gd(M P6,k1(uint) y3
#ifdef CB
,k1(O) W7
#else
,k1(d) Hi
#endif
O6){y3=floatBitsToUint(P6.z)&0xffffu;R r2=n0(JB,y3*4u+1u);
#ifdef CB
W7=X1(r2.z>>16u);
#else
Hi=hb(floatBitsToInt(P6.z)>>16);
#endif
c Q6=P6.xy;X Q0=p1(uintBitsToFloat(n0(JB,y3*4u)));c A2=uintBitsToFloat(r2.xy);Q6=D0(Q0,Q6)+A2;return Q6;}
#endif
#if defined(BB)&&defined(FB)
f c fd(M P6,k1(uint) y3,
#ifdef CB
k1(O) W7,
#endif
k1(c) Ii O6){y3=floatBitsToUint(P6.z)&0xffffu;
#ifdef CB
W7=X1(n0(JB,y3*4u+1u).z>>16u);
#endif
c Q6=P6.xy;M c8=uintBitsToFloat(n0(JB,y3*4u+3u).xyz);Ii=(Q6*c8.x+c8.yz)*j.Ji;return Q6;}
#endif
f d L9(d v2,d T1,d z3){return(T1-v2)/max(1.-v2*z3,ra);}
#if defined(SB)||defined(MD)
f uint M9(U0 x3,uint Ki){uint dc=(x3.y>>C6)*(Ki<<C6)+((x3.x>>C6)<<(C6<<1));dc+=((x3.x&0x1cu)<<C6)+((x3.y&0x1cu)<<2);dc+=((x3.y&0x3u)<<2)+(x3.x&0x3u);return dc;}
#endif
#ifdef SB
#ifdef Q
#define E5 I2
#define B4(S5) Q1=S5;H3
#else
#define E5 I1
#define B4(S5) x0(o0,S5);O1;
#endif
f d ec(uint Li){return hb(int((Li&Fb)-H5))*Db;}f uint d8(d n){return uint(n*Qh+.5);}
#endif
