#ifdef MD
#ifdef BB
c1(d0) K(0,e,WB);K(1,e,XB);d1
#endif
l2
#ifdef HB
E0 V(0,e,S);
#else
E0 V(0,C,S);
#endif
Z2 V(1,Q,F0);e2
#ifdef BB
w1(RB,d0,D,G,r){L(G,D,WB,e);L(G,D,XB,e);
#ifdef HB
T(S,e);
#else
T(S,C);
#endif
T(F0,Q);e I;uint a0;c k0;e U;if(L9(WB,XB,r,a0,k0,U H3)){
#ifdef HB
S=U;
#else
S.xy=f8(U.xy);
#endif
F0=Q1(a0);I=I3(k0);}else{I=e(j.a3,j.a3,j.a3,j.a3);}Z(S);Z(F0);x1(I);}
#endif
#endif
#if defined(DB)||defined(EB)
#ifdef BB
c1(d0) K(0,d4,MB);d1
#endif
l2
#ifdef EB
E0 V(0,c,J2);
#else
KB V(0,d,m1);
#endif
Z2 V(1,Q,F0);e2
#ifdef BB
w1(RB,d0,D,G,r){L(G,D,MB,O);
#ifdef EB
T(J2,c);
#else
T(m1,d);
#endif
T(F0,Q);uint a0;c k0;
#ifdef EB
k0=lc(MB,a0,J2 H3);
#else
k0=mc(MB,a0,m1 H3);
#endif
F0=Q1(a0);e I=I3(k0);
#ifdef EB
Z(J2);
#else
Z(m1);
#endif
Z(F0);x1(I);}
#endif
#endif
#ifdef BD
#ifdef BB
c1(d0) K(0,e,GC);d1 c1(B1) K(M9,e,YB);K(N9,e,SB);K(O9,e,PB);K(P9,uint,ZB);K(Q9,uint,AC);K(R9,uint,BC);K(S9,uint,LC);K(zf,e,ND);K(Af,e,OD);K(Bf,e,CD);K(nc,e,OC);d1
#endif
l2 E0 V(0,c,f2);E0 V(1,d,i5);E0 V(2,e,j5);
#ifdef AB
E0 V(3,e,R0);
#endif
KB V(4,i,R1);
#ifdef A
Z2 V(5,Q,J3);
#endif
#ifdef N
Z2 V(6,Q,I1);
#endif
e2
#ifdef BB
g8(RB,d0,D,B1,h0,G,r){L(G,D,GC,e);L(r,h0,YB,e);L(r,h0,SB,e);L(r,h0,PB,e);L(r,h0,ZB,uint);L(r,h0,AC,uint);L(r,h0,BC,uint);L(r,h0,LC,uint);L(r,h0,ND,e);L(r,h0,OD,e);L(r,h0,CD,e);L(r,h0,OC,e);T(f2,c);T(i5,d);T(j5,e);
#ifdef AB
T(R0,e);
#endif
T(R1,i);
#ifdef A
T(J3,Q);
#endif
#ifdef N
T(I1,Q);
#endif
bool T9=GC.z==.0||GC.w==.0;i5=T9?.0:1.;c k0=GC.xy;Y S0=n1(YB);Y S6=transpose(inverse(S0));if(!T9){float U9=I4*V9(S6[1])/dot(S0[1],S6[1]);if(U9>=.5){k0.x=.5;i5*=e4(.5/U9);}else{k0.x+=U9*GC.z;}float W9=I4*V9(S6[0])/dot(S0[0],S6[0]);if(W9>=.5){k0.y=.5;i5*=e4(.5/W9);}else{k0.y+=W9*GC.w;}}Y Cf=n1(ND);f2=M0(Cf,k0)+CD.xy;k0=M0(S0,k0)+PB.xy;if(T9){c f4=M0(S6,GC.zw);f4*=V9(f4)/dot(f4,f4);k0+=I4*f4;}
#ifdef AB
if(AB){R0=h8(n1(SB),PB.zw,k0);}
#endif
R1=unpackUnorm4x8(ZB);
#ifdef A
J3=Q1(AC);
#endif
#ifdef N
I1=Q1(BC);
#endif
e I=I3(k0);c l0=k0;
#ifdef PD
if(j.X9!=0u){l0.y=float(j.Y9)-l0.y;}
#endif
if(OC.w!=0.0){Y Df=n1(OD);c Ef=CD.zw;j5=Z9(l0,Df,Ef,OC.w,OC.xy,OC.z);}else{j5=e(.0,.0,.0,.0);}Z(f2);Z(i5);Z(j5);
#ifdef AB
Z(R0);
#endif
Z(R1);
#ifdef A
Z(J3);
#endif
#ifdef N
Z(I1);
#endif
x1(I);}
#endif
#elif defined(NB)
#ifdef BB
c1(x3) K(0,c,PC);d1 c1(K3) K(1,c,QC);d1 c1(B1) K(M9,e,YB);K(N9,e,SB);K(O9,e,PB);K(P9,uint,ZB);K(Q9,uint,AC);K(R9,uint,BC);K(S9,uint,LC);K(aa,e,HC);d1
#endif
l2 E0 V(0,c,f2);
#ifdef AB
E0 V(1,e,R0);
#endif
KB V(3,i,R1);
#ifdef A
Z2 V(4,Q,J3);
#endif
#ifdef N
Z2 V(5,Q,I1);
#endif
e2
#ifdef BB
T6(RB,x3,y3,K3,L3,B1,h0,G){L(G,y3,PC,c);L(G,L3,QC,c);L(r,h0,YB,e);L(r,h0,SB,e);L(r,h0,PB,e);L(r,h0,ZB,uint);L(r,h0,AC,uint);L(r,h0,BC,uint);L(r,h0,LC,uint);L(r,h0,HC,e);T(f2,c);
#ifdef AB
T(R0,e);
#endif
T(R1,i);
#ifdef A
T(J3,Q);
#endif
#ifdef N
T(I1,Q);
#endif
Y S0=n1(YB);c k0=M0(S0,PC)+PB.xy;f2=QC*HC.zw+HC.xy;
#ifdef AB
if(AB){R0=h8(n1(SB),PB.zw,k0);}
#endif
R1=unpackUnorm4x8(ZB);
#ifdef A
J3=Q1(AC);
#endif
#ifdef N
I1=Q1(BC);
#endif
e I=I3(k0);Z(f2);
#ifdef AB
Z(R0);
#endif
Z(R1);
#ifdef A
Z(J3);
#endif
#ifdef N
Z(I1);
#endif
x1(I);}
#endif
#endif
#ifdef IF
#ifdef BB
c1(d0) d1
#endif
l2 e2
#ifdef BB
w1(RB,d0,D,G,r){e0 y2;y2.x=(G&1)==0?j.i8.x:j.i8.z;y2.y=(G&2)==0?j.i8.y:j.i8.w;e I=I3(c(y2));x1(I);}
#endif
#endif
#ifdef ME
#endif
#if defined(NE)&&!defined(W)
#endif
#ifdef FB
S1
#ifndef W
#ifdef OE
#define ba OE
#else
#define ba K2
#endif
#ifdef DD
J4(ba,n0);
#else
B0(ba,n0);
#endif
#endif
#ifdef UC
#define K4 i
#define ca N0
#define j8 I0(.0)
#define oc(F) ((F).w!=.0)
#ifdef A
#ifndef VC
B0(c3,m0);
#else
J4(c3,m0);
#endif
#endif
#else
#define K4 uint
#define j8 0u
#define ca h1
#define oc(F) ((F)!=0u)
#ifdef A
o1(c3,m0);
#endif
#endif
L2(U6,L4);T1 g4 Z5(pc,Gf,WC);a6(qc,Hf,JB);h4 f uint If(float x){return uint(round(x*da+ea));}f d k8(uint x){return e4(float(x)*rc+(-ea*rc));}Q l8(Q a0){
#ifdef JF
a0=min(a0,j.Jf);
#endif
return a0;}
#ifdef A
f void sc(uint X0,K4 T0,V6(d) n){
#ifdef UC
if(all(lessThan(abs(T0.xy-unpackUnorm4x8(X0).xy),H2(.25/255.)))) n=min(n,T0.z);else n=.0;
#else
if(X0==T0>>16) n=min(n,unpackHalf2x16(T0).x);else n=.0;
#endif
}
#endif
f void m8(uint a0,d w0,i1(i) P
#if defined(A)&&!defined(VC)
,V6(K4) C1
#endif
W6 i4){O0 G0=l5(WC,a0);d n=w0;if((G0.x&(Kf|fa))!=0u){n=abs(n);
#ifdef XC
if(XC&&(G0.x&fa)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,H0(.0),H0(1.));
#ifdef A
if(A){uint X0=G0.x>>16u;if(X0!=0u){sc(X0,ca(m0),n);}}
#endif
#ifdef AB
if(AB&&(G0.x&Lf)!=0u){Y S0=n1(p0(JB,a0*g2+2u));e m2=p0(JB,a0*g2+3u);c Mf=M0(S0,f0)+m2.xy;C tc=f8(abs(Mf)*m2.zw-m2.zw);d m5=clamp(min(tc.x,tc.y)+.5,.0,1.);n=min(n,m5);}
#endif
uint n2=G0.x&0xfu;Q z3=Q1((G0.x>>4)&0xfu);
#ifdef N
bool n5=N&&z3!=M4;
#else
const bool n5=false;
#endif
if(n2<=ga){P=unpackUnorm4x8(G0.y);
#ifdef A
if(A&&n2==o5){
#ifndef VC
#ifdef UC
C1.xy=P.zw;C1.z=n;C1.w=1.;
#else
C1=G0.y|packHalf2x16(H2(n,.0));
#endif
#endif
P=I0(.0);}
#endif
}else{Y S0=n1(p0(JB,a0*g2));e m2=p0(JB,a0*g2+1u);c uc=M0(S0,f0)+m2.xy;float t=n2==vc?uc.x:length(uc);t=clamp(t,.0,1.);float x=t*m2.z+m2.w;float wc=uintBitsToFloat(G0.y);float Nf=floor(wc)*j.xc+j.yc;P=o2(ED,ia,c(x,Nf),.0);if(!n5){P.xyz*=P.w;d ja=e4(fract(wc)*(256./255.));P.w*=ja;}}
#if!defined(W)&&defined(N)
if(n5){if(P.w*n!=.0){i J1=N0(n0);P.xyz=h5(P.xyz,J1,z3);}P.xyz*=P.w;}
#endif
P*=n;}
#if!defined(W)&&!defined(DD)
f void n8(i P i4){
#ifndef UC
if(P.x+P.y+P.z+P.w==.0) return;float X6=1.-P.w;if(X6!=.0) P+=N0(n0)*X6;
#endif
y0(n0,P);}
#endif
#if defined(A)&&!defined(VC)
f void ka(K4 C1 i4){
#ifdef UC
y0(m0,C1);
#else
if(C1!=0u) j1(m0,C1);
#endif
}
#endif
#ifdef W
#define c6 z2
#define d6 A3
#else
#define c6 U1
#define d6 h2
#endif
#ifdef MD
c6(IB){
#ifdef HB
q(S,e);
#else
q(S,C);
#endif
q(F0,Q);d o8;
#ifdef HB
if(HB&&zc(S)){o8=N4(S k1);}else if(HB&&Ac(S)){o8=p8(S k1);}else
#endif
{o8=min(min(H0(S.x),abs(H0(S.y))),H0(1.));}i P=I0(.0);
#ifdef A
K4 C1=j8;
#endif
uint q8=If(o8);uint Bc=(Cc(F0)<<e6)|q8;uint A2=p5(L4,Bc);Q K1=Q1(A2>>e6);K1=l8(K1);if(K1==F0){if(!f6(S)){q8+=A2-max(Bc,A2);q8-=la;q5(L4,q8);}}else{d w0=k8(A2&r8);m8(K1,w0,P
#ifdef A
,C1
#endif
e3 V1);}P.xyz=M2(P.xyz,P.w,f0.xy,j.M3,j.N3);
#ifdef W
L1=P;
#else
n8(P V1);
#endif
#ifdef A
ka(C1 V1);
#endif
d6}
#endif
#if defined(DB)||defined(EB)
c6(IB){
#ifdef EB
q(J2,c);
#else
q(m1,d);
#endif
q(F0,Q);uint A2=f3(L4);Q K1=Q1(A2>>e6);K1=l8(K1);uint ma;
#ifndef EB
if(K1==F0){ma=A2;}else
#endif
{ma=(Cc(F0)<<e6)+la;}d n;
#ifdef EB
n=clamp(o2(FD,na,J2,.0).x,H0(.0),H0(1.));
#else
n=m1;
#endif
int Of=int(round(n*da));g3(L4,ma+uint(Of));i P=I0(.0);
#ifdef A
K4 C1=j8;
#endif
#ifndef EB
if(K1!=F0)
#endif
{d oa=k8(A2&r8);m8(K1,oa,P
#ifdef A
,C1
#endif
e3 V1);}P.xyz=M2(P.xyz,P.w,f0.xy,j.M3,j.N3);
#ifdef W
L1=P;
#else
n8(P V1);
#endif
#ifdef A
ka(C1 V1);
#endif
d6}
#endif
#ifdef ME
c6(IB){q(f2,c);
#ifdef BD
q(i5,d);q(j5,e);
#endif
#ifdef AB
q(R0,e);
#endif
q(R1,i);
#ifdef A
q(J3,Q);
#endif
#ifdef N
q(I1,Q);
#endif
i N2=v8(CC,r5,f2);d g6=1.;
#ifdef BD
g6=min(i5,g6);
#endif
#ifdef AB
if(AB){d m5=w3(v5(R0));g6=clamp(m5,H0(.0),g6);}
#endif
uint A2=f3(L4);Q K1=Q1(A2>>e6);K1=l8(K1);d oa=k8(A2&r8);i P;
#ifdef A
K4 C1=j8;
#endif
m8(K1,oa,P
#ifdef A
,C1
#endif
e3 V1);
#ifdef A
if(A&&J3!=0u){K4 T0=oc(C1)?C1:ca(m0);sc(J3,T0,g6);}
#endif
#ifdef BD
if(j5.w!=0.0){c pa=Dc(j5);i qa=o2(ED,ia,pa,0.0);qa.xyz*=qa.w;N2*=qa;}
#endif
N2*=R1;
#if!defined(W)&&defined(N)
if(N&&I1!=M4){i J1=N0(n0)*(1.-P.w)+P;N2.xyz=h5(Q6(N2),J1,I1)*N2.w;}
#endif
N2*=g6;P=P*(1.-N2.w)+N2;P.xyz=M2(P.xyz,P.w,f0.xy,j.M3,j.N3);
#ifdef W
L1=P;
#else
n8(P V1);
#endif
#ifdef A
ka(C1 V1);
#endif
g3(L4,la);d6}
#endif
#ifdef NE
c6(IB){
#ifndef W
#ifdef QD
if(QD){y0(n0,unpackUnorm4x8(j.Pf));}
#endif
#ifdef RD
if(RD){y0(n0,p1(CC,H));}
#endif
#ifdef KF
i p=N0(n0);y0(n0,p.zyxw);
#endif
#endif
g3(L4,j.Qf);
#ifdef A
if(A){j1(m0,0u);}
#endif
#ifdef W
discard;
#endif
d6}
#endif
#ifdef VC
#ifdef DD
z2(IB)
#else
c6(IB)
#endif
{uint A2=f3(L4);d w0=k8(A2&r8);Q K1=Q1(A2>>e6);K1=l8(K1);i P;m8(K1,w0,P e3 V1);
#ifdef DD
float X6=1.-P.w;if(X6!=.0) P+=N0(n0)*X6;L1=P;A3
#else
P.xyz=M2(P.xyz,P.w,f0.xy,j.M3,j.N3);
#ifdef W
L1=P;
#else
n8(P V1);
#endif
d6
#endif
}
#endif
#endif
