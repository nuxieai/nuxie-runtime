#ifdef SD
#ifdef BB
e1(e0) K(0,e,YB);K(1,e,ZB);f1
#endif
m2
#ifdef IB
F0 W(0,e,T);
#else
F0 W(0,E,T);
#endif
h3 W(1,P,G0);M1
#ifdef BB
o1(SB,e0,A,C,r){L(C,A,YB,e);L(C,A,ZB,e);
#ifdef IB
V(T,e);
#else
V(T,E);
#endif
V(G0,P);e I;uint f0;c i0;e U;if(ra(YB,ZB,r,f0,i0,U S3)){
#ifdef IB
T=U;
#else
T.xy=E8(U.xy);
#endif
G0=X1(f0);I=T3(i0);}else{I=e(j.i3,j.i3,j.i3,j.i3);}a0(T);a0(G0);A1(I);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef BB
e1(e0) K(0,k4,LB);f1
#endif
m2
#ifdef FB
F0 W(0,c,U2);
#else
MB W(0,d,p1);
#endif
h3 W(1,P,G0);M1
#ifdef BB
o1(SB,e0,A,C,r){L(C,A,LB,M);
#ifdef FB
V(U2,c);
#else
V(p1,d);
#endif
V(G0,P);uint f0;c i0;
#ifdef FB
i0=bd(LB,f0,U2 S3);
#else
i0=cd(LB,f0,p1 S3);
#endif
G0=X1(f0);e I=T3(i0);
#ifdef FB
a0(U2);
#else
a0(p1);
#endif
a0(G0);A1(I);}
#endif
#endif
#ifdef FD
#ifdef BB
e1(e0) K(0,e,HC);f1 e1(E1) K(sa,e,AC);K(ta,e,TB);K(ua,e,PB);K(va,uint,BC);K(wa,uint,CC);K(xa,uint,DC);K(ya,uint,MC);K(hg,e,TD);K(ig,e,UD);K(jg,e,GD);K(dd,e,HD);f1
#endif
m2 F0 W(0,c,n2);F0 W(1,d,v5);F0 W(2,e,w5);
#ifdef AB
F0 W(3,e,W0);
#endif
MB W(4,i,Y1);
#ifdef N
h3 W(5,P,U3);
#endif
#ifdef H
h3 W(6,P,N1);
#endif
M1
#ifdef BB
F8(SB,e0,A,E1,j0,C,r){L(C,A,HC,e);L(r,j0,AC,e);L(r,j0,TB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,TD,e);L(r,j0,UD,e);L(r,j0,GD,e);L(r,j0,HD,e);V(n2,c);V(v5,d);V(w5,e);
#ifdef AB
V(W0,e);
#endif
V(Y1,i);
#ifdef N
V(U3,P);
#endif
#ifdef H
V(N1,P);
#endif
bool za=HC.z==.0||HC.w==.0;v5=za?.0:1.;c i0=HC.xy;X O0=q1(AC);X k7=transpose(inverse(O0));if(!za){float Aa=Q4*Ba(k7[1])/dot(O0[1],k7[1]);if(Aa>=.5){i0.x=.5;v5*=x5(.5/Aa);}else{i0.x+=Aa*HC.z;}float Ca=Q4*Ba(k7[0])/dot(O0[0],k7[0]);if(Ca>=.5){i0.y=.5;v5*=x5(.5/Ca);}else{i0.y+=Ca*HC.w;}}X kg=q1(TD);n2=C0(kg,i0)+GD.xy;i0=C0(O0,i0)+PB.xy;if(za){c l4=C0(k7,HC.zw);l4*=Ba(l4)/dot(l4,l4);i0+=Q4*l4;}
#ifdef AB
if(AB){W0=G8(q1(TB),PB.zw,i0);}
#endif
Y1=unpackUnorm4x8(BC);
#ifdef N
U3=X1(CC);
#endif
#ifdef H
N1=X1(DC);
#endif
e I=T3(i0);c l0=i0;
#ifdef VD
if(j.Da!=0u){l0.y=float(j.Ea)-l0.y;}
#endif
uint w2=floatBitsToUint(HD.z);if(w2!=0){X lg=q1(UD);c mg=GD.zw;w5=Fa(l0,lg,mg,HD.xy,w2,1.0,1.0);}else{w5=e(.0,.0,.0,.0);}a0(n2);a0(v5);a0(w5);
#ifdef AB
a0(W0);
#endif
a0(Y1);
#ifdef N
a0(U3);
#endif
#ifdef H
a0(N1);
#endif
A1(I);}
#endif
#elif defined(NB)
#ifdef BB
e1(D3) K(0,c,PC);f1 e1(V3) K(1,c,QC);f1 e1(E1) K(sa,e,AC);K(ta,e,TB);K(ua,e,PB);K(va,uint,BC);K(wa,uint,CC);K(xa,uint,DC);K(ya,uint,MC);K(Ga,e,IC);f1
#endif
m2 F0 W(0,c,n2);
#ifdef AB
F0 W(1,e,W0);
#endif
MB W(3,i,Y1);
#ifdef N
h3 W(4,P,U3);
#endif
#ifdef H
h3 W(5,P,N1);
#endif
M1
#ifdef BB
l7(SB,D3,E3,V3,j3,E1,j0,C){L(C,E3,PC,c);L(C,j3,QC,c);L(r,j0,AC,e);L(r,j0,TB,e);L(r,j0,PB,e);L(r,j0,BC,uint);L(r,j0,CC,uint);L(r,j0,DC,uint);L(r,j0,MC,uint);L(r,j0,IC,e);V(n2,c);
#ifdef AB
V(W0,e);
#endif
V(Y1,i);
#ifdef N
V(U3,P);
#endif
#ifdef H
V(N1,P);
#endif
X O0=q1(AC);c i0=C0(O0,PC)+PB.xy;n2=QC*IC.zw+IC.xy;
#ifdef AB
if(AB){W0=G8(q1(TB),PB.zw,i0);}
#endif
Y1=unpackUnorm4x8(BC);
#ifdef N
U3=X1(CC);
#endif
#ifdef H
N1=X1(DC);
#endif
e I=T3(i0);a0(n2);
#ifdef AB
a0(W0);
#endif
a0(Y1);
#ifdef N
a0(U3);
#endif
#ifdef H
a0(N1);
#endif
A1(I);}
#endif
#endif
#ifdef MF
#ifdef BB
e1(e0) f1
#endif
m2 M1
#ifdef BB
o1(SB,e0,A,C,r){g0 F2;F2.x=(C&1)==0?j.H8.x:j.H8.z;F2.y=(C&2)==0?j.H8.y:j.H8.w;e I=T3(c(F2));A1(I);}
#endif
#endif
#ifdef SE
#endif
#if defined(TE)&&!defined(R)
#endif
#ifdef EB
F1
#ifndef R
#ifdef UE
#define Ha UE
#else
#define Ha x2
#endif
#ifdef ID
R4(Ha,n0);
#else
y0(Ha,n0);
#endif
#endif
#ifdef VC
#define S4 i
#define Ia K0
#define I8 H0(.0)
#define ed(F) ((F).w!=.0)
#ifdef N
#ifndef WC
y0(k3,m0);
#else
R4(k3,m0);
#endif
#endif
#else
#define S4 uint
#define I8 0u
#define Ia k1
#define ed(F) ((F)!=0u)
#ifdef N
r1(k3,m0);
#endif
#endif
V2(m7,T4);G1 m4 m6(fd,og,XC);n6(gd,pg,JB);n4 f uint qg(float x){return uint(round(x*Ja+Ka));}f d J8(uint x){return x5(float(x)*hd+(-Ka*hd));}P K8(P f0){
#ifdef NF
f0=min(f0,j.rg);
#endif
return f0;}
#ifdef N
f void id(uint B1,S4 X0,n7(d) n){
#ifdef VC
if(all(lessThan(abs(X0.xy-unpackUnorm4x8(B1).xy),S2(.25/255.)))) n=min(n,X0.z);else n=.0;
#else
if(B1==X0>>16) n=min(n,unpackHalf2x16(X0).x);else n=.0;
#endif
}
#endif
f void L8(uint f0,d w0,l1(i) Q
#if defined(N)&&!defined(WC)
,n7(S4) H1
#endif
o7 o4){R0 S0=y5(XC,f0);d n=w0;if((S0.x&(sg|La))!=0u){n=abs(n);
#ifdef YC
if(YC&&(S0.x&La)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,J0(.0),J0(1.));
#ifdef N
if(N){uint B1=S0.x>>16u;if(B1!=0u){id(B1,Ia(m0),n);}}
#endif
#ifdef AB
if(AB&&(S0.x&tg)!=0u){X O0=q1(q0(JB,f0*o2+2u));e y2=q0(JB,f0*o2+3u);c ug=C0(O0,c0)+y2.xy;E jd=E8(abs(ug)*y2.zw-y2.zw);d z5=clamp(min(jd.x,jd.y)+.5,.0,1.);n=min(n,z5);}
#endif
uint w2=S0.x&0xfu;P Z1=X1((S0.x>>4)&0xfu);
#ifdef H
bool G2=H&&Z1!=W3;
#else
const bool G2=false;
#endif
if(w2<=Ma){Q=unpackUnorm4x8(S0.y);
#ifdef N
if(N&&w2==M8){
#ifndef WC
#ifdef VC
H1.xy=Q.zw;H1.z=n;H1.w=1.;
#else
H1=S0.y|packHalf2x16(S2(n,.0));
#endif
#endif
Q=H0(.0);}
#endif
}else{X O0=q1(q0(JB,f0*o2));e y2=q0(JB,f0*o2+1u);c vg=C0(O0,c0)+y2.xy;c N8=y2.zw;bool p4=N8.x<0.0;N8.x=max(0.0,N8.x);c j3=wg(float(w2),vg,N8,p4,j.O8,j.P8);Q=p2(ZC,Q8,j3,0.0);if(!G2){Q.xyz*=Q.w;float q4=uintBitsToFloat(S0.y);Q.w*=abs(q4);}}
#if!defined(R)&&defined(H)
if(G2){if(Q.w*n!=.0){i a1=K0(n0);Q.xyz=P4(Q.xyz,a1,Z1);}Q.xyz*=Q.w;}
#endif
Q*=n;}
#if!defined(R)&&!defined(ID)
f void R8(i Q o4){
#ifndef VC
if(Q.x+Q.y+Q.z+Q.w==.0) return;if(Q.w!=1.) Q=Oa(Q,K0(n0));
#endif
x0(n0,Q);}
#endif
#if defined(N)&&!defined(WC)
f void Pa(S4 H1 o4){
#ifdef VC
x0(m0,H1);
#else
if(H1!=0u) m1(m0,H1);
#endif
}
#endif
#ifdef R
#define o6 H2
#define p6 F3
#else
#define o6 I1
#define p6 O1
#endif
#ifdef SD
o6(HB){
#ifdef IB
q(T,e);
#else
q(T,E);
#endif
q(G0,P);d S8;
#ifdef IB
if(IB&&kd(T)){S8=V4(T n1);}else if(IB&&ld(T)){S8=T8(T n1);}else
#endif
{S8=min(min(J0(T.x),abs(J0(T.y))),J0(1.));}i Q=H0(.0);
#ifdef N
S4 H1=I8;
#endif
uint U8=qg(S8);uint md=(nd(G0)<<q6)|U8;uint I2=A5(T4,md);P P1=X1(I2>>q6);P1=K8(P1);if(P1==G0){if(!r6(T)){U8+=I2-max(md,I2);U8-=Qa;B5(T4,U8);}}else{d w0=J8(I2&V8);L8(P1,w0,Q
#ifdef N
,H1
#endif
m3 a2);}Q.xyz=J2(Q.xyz,Q.w,c0.xy,j.G3,j.H3);
#ifdef R
Q1=Q;
#else
R8(Q a2);
#endif
#ifdef N
Pa(H1 a2);
#endif
p6}
#endif
#if defined(DB)||defined(FB)
o6(HB){
#ifdef FB
q(U2,c);
#else
q(p1,d);
#endif
q(G0,P);uint I2=n3(T4);P P1=X1(I2>>q6);P1=K8(P1);uint Ra;
#ifndef FB
if(P1==G0){Ra=I2;}else
#endif
{Ra=(nd(G0)<<q6)+Qa;}d n;
#ifdef FB
n=clamp(p2(JD,Sa,U2,.0).x,J0(.0),J0(1.));
#else
n=p1;
#endif
int xg=int(round(n*Ja));o3(T4,Ra+uint(xg));i Q=H0(.0);
#ifdef N
S4 H1=I8;
#endif
#ifndef FB
if(P1!=G0)
#endif
{d Ta=J8(I2&V8);L8(P1,Ta,Q
#ifdef N
,H1
#endif
m3 a2);}Q.xyz=J2(Q.xyz,Q.w,c0.xy,j.G3,j.H3);
#ifdef R
Q1=Q;
#else
R8(Q a2);
#endif
#ifdef N
Pa(H1 a2);
#endif
p6}
#endif
#ifdef SE
o6(HB){q(n2,c);
#ifdef FD
q(v5,d);q(w5,e);
#endif
#ifdef AB
q(W0,e);
#endif
q(Y1,i);
#ifdef N
q(U3,P);
#endif
#ifdef H
q(N1,P);
#endif
i R1=W8(UB,W4,n2);d v6=1.;
#ifdef FD
v6=min(v5,v6);
#endif
#ifdef AB
if(AB){d z5=C3(X4(W0));v6=clamp(z5,J0(.0),v6);}
#endif
uint I2=n3(T4);P P1=X1(I2>>q6);P1=K8(P1);d Ta=J8(I2&V8);i Q;
#ifdef N
S4 H1=I8;
#endif
L8(P1,Ta,Q
#ifdef N
,H1
#endif
m3 a2);
#ifdef N
if(N&&U3!=0u){S4 X0=ed(H1)?H1:Ia(m0);id(U3,X0,v6);}
#endif
#ifdef FD
if(w5.w!=0.0){c Ua=Va(w5,j.O8,j.P8);i Wa=p2(ZC,Q8,Ua,0.0);Wa.xyz*=Wa.w;R1*=Wa;}
#endif
R1*=Y1;
#if!defined(R)&&defined(H)
if(H&&N1!=W3){i a1=K0(n0)*(1.-Q.w)+Q;R1.xyz=P4(l6(R1),a1,N1)*R1.w;}
#endif
R1*=v6;Q=Q*(1.-R1.w)+R1;Q.xyz=J2(Q.xyz,Q.w,c0.xy,j.G3,j.H3);
#ifdef R
Q1=Q;
#else
R8(Q a2);
#endif
#ifdef N
Pa(H1 a2);
#endif
o3(T4,Qa);p6}
#endif
#ifdef TE
o6(HB){
#ifndef R
#ifdef WD
if(WD){x0(n0,unpackUnorm4x8(j.yg));}
#endif
#ifdef XD
if(XD){x0(n0,v1(UB,G));}
#endif
#ifdef OF
i k=K0(n0);x0(n0,k.zyxw);
#endif
#endif
o3(T4,j.zg);
#ifdef N
if(N){m1(m0,0u);}
#endif
#ifdef R
discard;
#endif
p6}
#endif
#ifdef WC
#ifdef ID
H2(HB)
#else
o6(HB)
#endif
{uint I2=n3(T4);d w0=J8(I2&V8);P P1=X1(I2>>q6);P1=K8(P1);i Q;L8(P1,w0,Q m3 a2);
#ifdef ID
float od=1.-Q.w;if(od!=.0) Q+=K0(n0)*od;Q1=Q;F3
#else
Q.xyz=J2(Q.xyz,Q.w,c0.xy,j.G3,j.H3);
#ifdef R
Q1=Q;
#else
R8(Q a2);
#endif
p6
#endif
}
#endif
#endif
