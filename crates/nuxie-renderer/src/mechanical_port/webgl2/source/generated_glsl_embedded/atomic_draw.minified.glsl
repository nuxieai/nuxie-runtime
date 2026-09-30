#ifdef ND
#ifdef BB
c1(d0) K(0,e,WB);K(1,e,XB);d1
#endif
l2
#ifdef HB
E0 W(0,e,S);
#else
E0 W(0,C,S);
#endif
a3 W(1,R,F0);d2
#ifdef BB
r1(RB,d0,D,G,r){L(G,D,WB,e);L(G,D,XB,e);
#ifdef HB
T(S,e);
#else
T(S,C);
#endif
T(F0,R);e I;uint a0;c k0;e U;if(K9(WB,XB,r,a0,k0,U H3)){
#ifdef HB
S=U;
#else
S.xy=f8(U.xy);
#endif
F0=O1(a0);I=I3(k0);}else{I=e(j.c3,j.c3,j.c3,j.c3);}Z(S);Z(F0);v1(I);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef BB
c1(d0) K(0,c4,MB);d1
#endif
l2
#ifdef FB
E0 W(0,c,K2);
#else
KB W(0,d,m1);
#endif
a3 W(1,R,F0);d2
#ifdef BB
r1(RB,d0,D,G,r){L(G,D,MB,P);
#ifdef FB
T(K2,c);
#else
T(m1,d);
#endif
T(F0,R);uint a0;c k0;
#ifdef FB
k0=kc(MB,a0,K2 H3);
#else
k0=lc(MB,a0,m1 H3);
#endif
F0=O1(a0);e I=I3(k0);
#ifdef FB
Z(K2);
#else
Z(m1);
#endif
Z(F0);v1(I);}
#endif
#endif
#ifdef CD
#ifdef BB
c1(d0) K(0,e,GC);d1 c1(z1) K(L9,e,YB);K(M9,e,SB);K(N9,e,PB);K(O9,uint,ZB);K(P9,uint,AC);K(Q9,uint,BC);K(R9,uint,MC);K(uf,e,OD);K(vf,e,PD);K(wf,e,DD);K(mc,e,PC);d1
#endif
l2 E0 W(0,c,e2);E0 W(1,d,i5);E0 W(2,e,j5);
#ifdef AB
E0 W(3,e,R0);
#endif
KB W(4,i,P1);
#ifdef A
a3 W(5,R,J3);
#endif
#ifdef O
a3 W(6,R,H1);
#endif
d2
#ifdef BB
g8(RB,d0,D,z1,h0,G,r){L(G,D,GC,e);L(r,h0,YB,e);L(r,h0,SB,e);L(r,h0,PB,e);L(r,h0,ZB,uint);L(r,h0,AC,uint);L(r,h0,BC,uint);L(r,h0,MC,uint);L(r,h0,OD,e);L(r,h0,PD,e);L(r,h0,DD,e);L(r,h0,PC,e);T(e2,c);T(i5,d);T(j5,e);
#ifdef AB
T(R0,e);
#endif
T(P1,i);
#ifdef A
T(J3,R);
#endif
#ifdef O
T(H1,R);
#endif
bool S9=GC.z==.0||GC.w==.0;i5=S9?.0:1.;c k0=GC.xy;Y S0=n1(YB);Y R6=transpose(inverse(S0));if(!S9){float T9=H4*U9(R6[1])/dot(S0[1],R6[1]);if(T9>=.5){k0.x=.5;i5*=d4(.5/T9);}else{k0.x+=T9*GC.z;}float V9=H4*U9(R6[0])/dot(S0[0],R6[0]);if(V9>=.5){k0.y=.5;i5*=d4(.5/V9);}else{k0.y+=V9*GC.w;}}Y xf=n1(OD);e2=K0(xf,k0)+DD.xy;k0=K0(S0,k0)+PB.xy;if(S9){c e4=K0(R6,GC.zw);e4*=U9(e4)/dot(e4,e4);k0+=H4*e4;}
#ifdef AB
if(AB){R0=h8(n1(SB),PB.zw,k0);}
#endif
P1=unpackUnorm4x8(ZB);
#ifdef A
J3=O1(AC);
#endif
#ifdef O
H1=O1(BC);
#endif
e I=I3(k0);c l0=k0;
#ifdef QD
if(j.W9!=0u){l0.y=float(j.X9)-l0.y;}
#endif
if(PC.w!=0.0){Y yf=n1(PD);c zf=DD.zw;j5=Y9(l0,yf,zf,PC.w,PC.xy,PC.z);}else{j5=e(.0,.0,.0,.0);}Z(e2);Z(i5);Z(j5);
#ifdef AB
Z(R0);
#endif
Z(P1);
#ifdef A
Z(J3);
#endif
#ifdef O
Z(H1);
#endif
v1(I);}
#endif
#elif defined(NB)
#ifdef BB
c1(w3) K(0,c,QC);d1 c1(K3) K(1,c,RC);d1 c1(z1) K(L9,e,YB);K(M9,e,SB);K(N9,e,PB);K(O9,uint,ZB);K(P9,uint,AC);K(Q9,uint,BC);K(R9,uint,MC);K(Z9,e,HC);d1
#endif
l2 E0 W(0,c,e2);
#ifdef AB
E0 W(1,e,R0);
#endif
KB W(3,i,P1);
#ifdef A
a3 W(4,R,J3);
#endif
#ifdef O
a3 W(5,R,H1);
#endif
d2
#ifdef BB
S6(RB,w3,x3,K3,L3,z1,h0,G){L(G,x3,QC,c);L(G,L3,RC,c);L(r,h0,YB,e);L(r,h0,SB,e);L(r,h0,PB,e);L(r,h0,ZB,uint);L(r,h0,AC,uint);L(r,h0,BC,uint);L(r,h0,MC,uint);L(r,h0,HC,e);T(e2,c);
#ifdef AB
T(R0,e);
#endif
T(P1,i);
#ifdef A
T(J3,R);
#endif
#ifdef O
T(H1,R);
#endif
Y S0=n1(YB);c k0=K0(S0,QC)+PB.xy;e2=RC*HC.zw+HC.xy;
#ifdef AB
if(AB){R0=h8(n1(SB),PB.zw,k0);}
#endif
P1=unpackUnorm4x8(ZB);
#ifdef A
J3=O1(AC);
#endif
#ifdef O
H1=O1(BC);
#endif
e I=I3(k0);Z(e2);
#ifdef AB
Z(R0);
#endif
Z(P1);
#ifdef A
Z(J3);
#endif
#ifdef O
Z(H1);
#endif
v1(I);}
#endif
#endif
#ifdef JF
#ifdef BB
c1(d0) d1
#endif
l2 d2
#ifdef BB
r1(RB,d0,D,G,r){e0 z2;z2.x=(G&1)==0?j.i8.x:j.i8.z;z2.y=(G&2)==0?j.i8.y:j.i8.w;e I=I3(c(z2));v1(I);}
#endif
#endif
#ifdef NE
#endif
#if defined(OE)&&!defined(V)
#endif
#ifdef EB
Q1
#ifndef V
#ifdef PE
#define aa PE
#else
#define aa L2
#endif
#ifdef ED
I4(aa,o0);
#else
A0(aa,o0);
#endif
#endif
#ifdef WC
#define J4 i
#define ba N0
#define j8 G0(.0)
#define nc(F) ((F).w!=.0)
#ifdef A
#ifndef SC
A0(d3,m0);
#else
I4(d3,m0);
#endif
#endif
#else
#define J4 uint
#define j8 0u
#define ba h1
#define nc(F) ((F)!=0u)
#ifdef A
o1(d3,m0);
#endif
#endif
M2(T6,K4);R1 f4 X5(oc,Bf,XC);Y5(pc,Cf,JB);g4 f uint Df(float x){return uint(round(x*ca+da));}f d k8(uint x){return d4(float(x)*qc+(-da*qc));}R l8(R a0){
#ifdef KF
a0=min(a0,j.Ef);
#endif
return a0;}
#ifdef A
f void rc(uint X0,J4 T0,U6(d) o){
#ifdef WC
if(all(lessThan(abs(T0.xy-unpackUnorm4x8(X0).xy),I2(.25/255.)))) o=min(o,T0.z);else o=.0;
#else
if(X0==T0>>16) o=min(o,unpackHalf2x16(T0).x);else o=.0;
#endif
}
#endif
f void m8(uint a0,d w0,i1(i) M
#if defined(A)&&!defined(SC)
,U6(J4) A1
#endif
V6 h4){O0 L0=k5(XC,a0);d o=w0;if((L0.x&(Ff|ea))!=0u){o=abs(o);
#ifdef YC
if(YC&&(L0.x&ea)!=0u){o=1.-abs(fract(o*.5)*2.+-1.);}
#endif
}o=clamp(o,M0(.0),M0(1.));
#ifdef A
if(A){uint X0=L0.x>>16u;if(X0!=0u){rc(X0,ba(m0),o);}}
#endif
#ifdef AB
if(AB&&(L0.x&Gf)!=0u){Y S0=n1(p0(JB,a0*f2+2u));e m2=p0(JB,a0*f2+3u);c Hf=K0(S0,f0)+m2.xy;C sc=f8(abs(Hf)*m2.zw-m2.zw);d l5=clamp(min(sc.x,sc.y)+.5,.0,1.);o=min(o,l5);}
#endif
uint n2=L0.x&0xfu;R y3=O1((L0.x>>4)&0xfu);
#ifdef O
bool m5=O&&y3!=L4;
#else
const bool m5=false;
#endif
if(n2<=fa){M=unpackUnorm4x8(L0.y);
#ifdef A
if(A&&n2==n5){
#ifndef SC
#ifdef WC
A1.xy=M.zw;A1.z=o;A1.w=1.;
#else
A1=L0.y|packHalf2x16(I2(o,.0));
#endif
#endif
M=G0(.0);}
#endif
}else{Y S0=n1(p0(JB,a0*f2));e m2=p0(JB,a0*f2+1u);c tc=K0(S0,f0)+m2.xy;float t=n2==uc?tc.x:length(tc);t=clamp(t,.0,1.);float x=t*m2.z+m2.w;float vc=uintBitsToFloat(L0.y);float If=floor(vc)*j.wc+j.xc;M=o2(FD,ha,c(x,If),.0);if(!m5){M.xyz*=M.w;d ia=d4(fract(vc)*(256./255.));M.w*=ia;}}
#if!defined(V)&&defined(O)
if(m5){if(M.w*o!=.0){i S1=N0(o0);M.xyz=h5(M.xyz,S1,y3);}M.xyz*=M.w;}
#endif
M*=o;
#if defined(CC)&&(defined(V)||defined(SC))
M=z3(M);
#endif
}
#if!defined(V)&&!defined(ED)
f void n8(i M h4){
#ifndef WC
if(M.x+M.y+M.z+M.w==.0) return;float W6=1.-M.w;if(W6!=.0) M+=N0(o0)*W6;
#endif
B0(o0,M);}
#endif
#if defined(A)&&!defined(SC)
f void ja(J4 A1 h4){
#ifdef WC
B0(m0,A1);
#else
if(A1!=0u) j1(m0,A1);
#endif
}
#endif
#ifdef V
#define a6 A2
#define c6 A3
#else
#define a6 T1
#define c6 g2
#endif
#ifdef ND
a6(IB){
#ifdef HB
q(S,e);
#else
q(S,C);
#endif
q(F0,R);d o8;
#ifdef HB
if(HB&&yc(S)){o8=M4(S k1);}else if(HB&&zc(S)){o8=p8(S k1);}else
#endif
{o8=min(min(M0(S.x),abs(M0(S.y))),M0(1.));}i M=G0(.0);
#ifdef A
J4 A1=j8;
#endif
uint q8=Df(o8);uint Ac=(Bc(F0)<<d6)|q8;uint B2=o5(K4,Ac);R I1=O1(B2>>d6);I1=l8(I1);if(I1==F0){if(!e6(S)){q8+=B2-max(Ac,B2);q8-=ka;p5(K4,q8);}}else{d w0=k8(B2&r8);m8(I1,w0,M
#ifdef A
,A1
#endif
e3 U1);}M.xyz=O2(M.xyz,M.w,f0.xy,j.M3,j.N3);
#ifdef V
J1=M;
#else
n8(M U1);
#endif
#ifdef A
ja(A1 U1);
#endif
c6}
#endif
#if defined(DB)||defined(FB)
a6(IB){
#ifdef FB
q(K2,c);
#else
q(m1,d);
#endif
q(F0,R);uint B2=f3(K4);R I1=O1(B2>>d6);I1=l8(I1);uint la;
#ifndef FB
if(I1==F0){la=B2;}else
#endif
{la=(Bc(F0)<<d6)+ka;}d o;
#ifdef FB
o=clamp(o2(GD,ma,K2,.0).x,M0(.0),M0(1.));
#else
o=m1;
#endif
int Jf=int(round(o*ca));g3(K4,la+uint(Jf));i M=G0(.0);
#ifdef A
J4 A1=j8;
#endif
#ifndef FB
if(I1!=F0)
#endif
{d na=k8(B2&r8);m8(I1,na,M
#ifdef A
,A1
#endif
e3 U1);}M.xyz=O2(M.xyz,M.w,f0.xy,j.M3,j.N3);
#ifdef V
J1=M;
#else
n8(M U1);
#endif
#ifdef A
ja(A1 U1);
#endif
c6}
#endif
#ifdef NE
a6(IB){q(e2,c);
#ifdef CD
q(i5,d);q(j5,e);
#endif
#ifdef AB
q(R0,e);
#endif
q(P1,i);
#ifdef A
q(J3,R);
#endif
#ifdef O
q(H1,R);
#endif
i p2=v8(IC,f6,e2);d g6=1.;
#ifdef CD
g6=min(i5,g6);
#endif
#ifdef AB
if(AB){d l5=v3(q5(R0));g6=clamp(l5,M0(.0),g6);}
#endif
uint B2=f3(K4);R I1=O1(B2>>d6);I1=l8(I1);d na=k8(B2&r8);i M;
#ifdef A
J4 A1=j8;
#endif
m8(I1,na,M
#ifdef A
,A1
#endif
e3 U1);
#ifdef A
if(A&&J3!=0u){J4 T0=nc(A1)?A1:ba(m0);rc(J3,T0,g6);}
#endif
#ifdef CD
if(j5.w!=0.0){c oa=Cc(j5);i pa=o2(FD,ha,oa,0.0);pa.xyz*=pa.w;p2*=pa;}
#endif
p2*=P1;
#if!defined(V)&&defined(O)
if(O&&H1!=L4){i S1=N0(o0)*(1.-M.w)+M;p2.xyz=h5(P6(p2),S1,H1)*p2.w;}
#endif
p2*=g6;
#if defined(CC)
p2=z3(p2);
#endif
M=M*(1.-p2.w)+p2;M.xyz=O2(M.xyz,M.w,f0.xy,j.M3,j.N3);
#ifdef V
J1=M;
#else
n8(M U1);
#endif
#ifdef A
ja(A1 U1);
#endif
g3(K4,ka);c6}
#endif
#ifdef OE
a6(IB){
#ifndef V
#ifdef RD
if(RD){B0(o0,unpackUnorm4x8(j.Kf));}
#endif
#ifdef SD
if(SD){B0(o0,p1(IC,H));}
#endif
#ifdef LF
i l=N0(o0);B0(o0,l.zyxw);
#endif
#endif
g3(K4,j.Lf);
#ifdef A
if(A){j1(m0,0u);}
#endif
#ifdef V
discard;
#endif
c6}
#endif
#ifdef SC
#ifdef ED
A2(IB)
#else
a6(IB)
#endif
{uint B2=f3(K4);d w0=k8(B2&r8);R I1=O1(B2>>d6);I1=l8(I1);i M;m8(I1,w0,M e3 U1);
#ifdef ED
float W6=1.-M.w;if(W6!=.0) M+=N0(o0)*W6;J1=M;A3
#else
M.xyz=O2(M.xyz,M.w,f0.xy,j.M3,j.N3);
#ifdef V
J1=M;
#else
n8(M U1);
#endif
c6
#endif
}
#endif
#endif
