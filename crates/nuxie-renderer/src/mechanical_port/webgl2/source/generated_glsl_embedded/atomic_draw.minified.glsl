#ifdef ND
#ifdef BB
c1(d0) K(0,f,WB);K(1,f,XB);d1
#endif
l2
#ifdef HB
E0 W(0,f,S);
#else
E0 W(0,C,S);
#endif
a3 W(1,R,F0);e2
#ifdef BB
v1(RB,d0,D,G,r){L(G,D,WB,f);L(G,D,XB,f);
#ifdef HB
T(S,f);
#else
T(S,C);
#endif
T(F0,R);f I;uint a0;c k0;f U;if(L9(WB,XB,r,a0,k0,U H3)){
#ifdef HB
S=U;
#else
S.xy=h8(U.xy);
#endif
F0=P1(a0);I=I3(k0);}else{I=f(j.c3,j.c3,j.c3,j.c3);}Z(S);Z(F0);w1(I);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef BB
c1(d0) K(0,d4,MB);d1
#endif
l2
#ifdef FB
E0 W(0,c,K2);
#else
KB W(0,d,m1);
#endif
a3 W(1,R,F0);e2
#ifdef BB
v1(RB,d0,D,G,r){L(G,D,MB,P);
#ifdef FB
T(K2,c);
#else
T(m1,d);
#endif
T(F0,R);uint a0;c k0;
#ifdef FB
k0=lc(MB,a0,K2 H3);
#else
k0=mc(MB,a0,m1 H3);
#endif
F0=P1(a0);f I=I3(k0);
#ifdef FB
Z(K2);
#else
Z(m1);
#endif
Z(F0);w1(I);}
#endif
#endif
#ifdef CD
#ifdef BB
c1(d0) K(0,f,HC);d1 c1(A1) K(M9,f,YB);K(N9,f,SB);K(O9,f,PB);K(P9,uint,ZB);K(Q9,uint,AC);K(R9,uint,BC);K(S9,uint,MC);K(yf,f,OD);K(zf,f,PD);K(Af,f,DD);K(nc,f,PC);d1
#endif
l2 E0 W(0,c,f2);E0 W(1,d,j5);E0 W(2,f,k5);
#ifdef AB
E0 W(3,f,S0);
#endif
KB W(4,i,Q1);
#ifdef A
a3 W(5,R,J3);
#endif
#ifdef O
a3 W(6,R,H1);
#endif
e2
#ifdef BB
i8(RB,d0,D,A1,h0,G,r){L(G,D,HC,f);L(r,h0,YB,f);L(r,h0,SB,f);L(r,h0,PB,f);L(r,h0,ZB,uint);L(r,h0,AC,uint);L(r,h0,BC,uint);L(r,h0,MC,uint);L(r,h0,OD,f);L(r,h0,PD,f);L(r,h0,DD,f);L(r,h0,PC,f);T(f2,c);T(j5,d);T(k5,f);
#ifdef AB
T(S0,f);
#endif
T(Q1,i);
#ifdef A
T(J3,R);
#endif
#ifdef O
T(H1,R);
#endif
bool T9=HC.z==.0||HC.w==.0;j5=T9?.0:1.;c k0=HC.xy;Y T0=n1(YB);Y T6=transpose(inverse(T0));if(!T9){float U9=I4*V9(T6[1])/dot(T0[1],T6[1]);if(U9>=.5){k0.x=.5;j5*=e4(.5/U9);}else{k0.x+=U9*HC.z;}float W9=I4*V9(T6[0])/dot(T0[0],T6[0]);if(W9>=.5){k0.y=.5;j5*=e4(.5/W9);}else{k0.y+=W9*HC.w;}}Y Bf=n1(OD);f2=M0(Bf,k0)+DD.xy;k0=M0(T0,k0)+PB.xy;if(T9){c f4=M0(T6,HC.zw);f4*=V9(f4)/dot(f4,f4);k0+=I4*f4;}
#ifdef AB
if(AB){S0=j8(n1(SB),PB.zw,k0);}
#endif
Q1=unpackUnorm4x8(ZB);
#ifdef A
J3=P1(AC);
#endif
#ifdef O
H1=P1(BC);
#endif
f I=I3(k0);c l0=k0;
#ifdef QD
if(j.X9!=0u){l0.y=float(j.Y9)-l0.y;}
#endif
if(PC.w!=0.0){Y Cf=n1(PD);c Df=DD.zw;k5=Z9(l0,Cf,Df,PC.w,PC.xy,PC.z);}else{k5=f(.0,.0,.0,.0);}Z(f2);Z(j5);Z(k5);
#ifdef AB
Z(S0);
#endif
Z(Q1);
#ifdef A
Z(J3);
#endif
#ifdef O
Z(H1);
#endif
w1(I);}
#endif
#elif defined(NB)
#ifdef BB
c1(w3) K(0,c,QC);d1 c1(K3) K(1,c,RC);d1 c1(A1) K(M9,f,YB);K(N9,f,SB);K(O9,f,PB);K(P9,uint,ZB);K(Q9,uint,AC);K(R9,uint,BC);K(S9,uint,MC);K(aa,f,IC);d1
#endif
l2 E0 W(0,c,f2);
#ifdef AB
E0 W(1,f,S0);
#endif
KB W(3,i,Q1);
#ifdef A
a3 W(4,R,J3);
#endif
#ifdef O
a3 W(5,R,H1);
#endif
e2
#ifdef BB
U6(RB,w3,x3,K3,L3,A1,h0,G){L(G,x3,QC,c);L(G,L3,RC,c);L(r,h0,YB,f);L(r,h0,SB,f);L(r,h0,PB,f);L(r,h0,ZB,uint);L(r,h0,AC,uint);L(r,h0,BC,uint);L(r,h0,MC,uint);L(r,h0,IC,f);T(f2,c);
#ifdef AB
T(S0,f);
#endif
T(Q1,i);
#ifdef A
T(J3,R);
#endif
#ifdef O
T(H1,R);
#endif
Y T0=n1(YB);c k0=M0(T0,QC)+PB.xy;f2=RC*IC.zw+IC.xy;
#ifdef AB
if(AB){S0=j8(n1(SB),PB.zw,k0);}
#endif
Q1=unpackUnorm4x8(ZB);
#ifdef A
J3=P1(AC);
#endif
#ifdef O
H1=P1(BC);
#endif
f I=I3(k0);Z(f2);
#ifdef AB
Z(S0);
#endif
Z(Q1);
#ifdef A
Z(J3);
#endif
#ifdef O
Z(H1);
#endif
w1(I);}
#endif
#endif
#ifdef JF
#ifdef BB
c1(d0) d1
#endif
l2 e2
#ifdef BB
v1(RB,d0,D,G,r){e0 z2;z2.x=(G&1)==0?j.k8.x:j.k8.z;z2.y=(G&2)==0?j.k8.y:j.k8.w;f I=I3(c(z2));w1(I);}
#endif
#endif
#ifdef NE
#endif
#if defined(OE)&&!defined(V)
#endif
#ifdef EB
R1
#ifndef V
#ifdef PE
#define ba PE
#else
#define ba L2
#endif
#ifdef ED
J4(ba,n0);
#else
B0(ba,n0);
#endif
#endif
#ifdef WC
#define K4 i
#define ca N0
#define l8 G0(.0)
#define oc(F) ((F).w!=.0)
#ifdef A
#ifndef SC
B0(d3,m0);
#else
J4(d3,m0);
#endif
#endif
#else
#define K4 uint
#define l8 0u
#define ca h1
#define oc(F) ((F)!=0u)
#ifdef A
o1(d3,m0);
#endif
#endif
M2(V6,L4);S1 g4 a6(pc,Ff,XC);c6(qc,Gf,JB);h4 e uint Hf(float x){return uint(round(x*da+ea));}e d m8(uint x){return e4(float(x)*rc+(-ea*rc));}R n8(R a0){
#ifdef KF
a0=min(a0,j.If);
#endif
return a0;}
#ifdef A
e void sc(uint X0,K4 U0,W6(d) o){
#ifdef WC
if(all(lessThan(abs(U0.xy-unpackUnorm4x8(X0).xy),I2(.25/255.)))) o=min(o,U0.z);else o=.0;
#else
if(X0==U0>>16) o=min(o,unpackHalf2x16(U0).x);else o=.0;
#endif
}
#endif
e void o8(uint a0,d w0,i1(i) M
#if defined(A)&&!defined(SC)
,W6(K4) B1
#endif
X6 i4){O0 H0=m5(XC,a0);d o=w0;if((H0.x&(Jf|fa))!=0u){o=abs(o);
#ifdef YC
if(YC&&(H0.x&fa)!=0u){o=1.-abs(fract(o*.5)*2.+-1.);}
#endif
}o=clamp(o,I0(.0),I0(1.));
#ifdef A
if(A){uint X0=H0.x>>16u;if(X0!=0u){sc(X0,ca(m0),o);}}
#endif
#ifdef AB
if(AB&&(H0.x&Kf)!=0u){Y T0=n1(p0(JB,a0*g2+2u));f m2=p0(JB,a0*g2+3u);c Lf=M0(T0,f0)+m2.xy;C tc=h8(abs(Lf)*m2.zw-m2.zw);d n5=clamp(min(tc.x,tc.y)+.5,.0,1.);o=min(o,n5);}
#endif
uint n2=H0.x&0xfu;R y3=P1((H0.x>>4)&0xfu);
#ifdef O
bool o5=O&&y3!=M4;
#else
const bool o5=false;
#endif
if(n2<=ga){M=unpackUnorm4x8(H0.y);
#ifdef A
if(A&&n2==p5){
#ifndef SC
#ifdef WC
B1.xy=M.zw;B1.z=o;B1.w=1.;
#else
B1=H0.y|packHalf2x16(I2(o,.0));
#endif
#endif
M=G0(.0);}
#endif
}else{Y T0=n1(p0(JB,a0*g2));f m2=p0(JB,a0*g2+1u);c uc=M0(T0,f0)+m2.xy;float t=n2==vc?uc.x:length(uc);t=clamp(t,.0,1.);float x=t*m2.z+m2.w;float wc=uintBitsToFloat(H0.y);float Mf=floor(wc)*j.xc+j.yc;M=o2(FD,ia,c(x,Mf),.0);if(!o5){M.xyz*=M.w;d ja=e4(fract(wc)*(256./255.));M.w*=ja;}}
#if!defined(V)&&defined(O)
if(o5){if(M.w*o!=.0){i I1=N0(n0);M.xyz=i5(M.xyz,I1,y3);}M.xyz*=M.w;}
#endif
M*=o;
#if defined(CC)&&(defined(V)||defined(SC))
M=z3(M);
#endif
}
#if!defined(V)&&!defined(ED)
e void p8(i M i4){
#ifndef WC
if(M.x+M.y+M.z+M.w==.0) return;float Y6=1.-M.w;if(Y6!=.0) M+=N0(n0)*Y6;
#endif
y0(n0,M);}
#endif
#if defined(A)&&!defined(SC)
e void ka(K4 B1 i4){
#ifdef WC
y0(m0,B1);
#else
if(B1!=0u) j1(m0,B1);
#endif
}
#endif
#ifdef V
#define d6 A2
#define e6 A3
#else
#define d6 T1
#define e6 h2
#endif
#ifdef ND
d6(IB){
#ifdef HB
q(S,f);
#else
q(S,C);
#endif
q(F0,R);d q8;
#ifdef HB
if(HB&&zc(S)){q8=N4(S k1);}else if(HB&&Ac(S)){q8=r8(S k1);}else
#endif
{q8=min(min(I0(S.x),abs(I0(S.y))),I0(1.));}i M=G0(.0);
#ifdef A
K4 B1=l8;
#endif
uint v8=Hf(q8);uint Bc=(Cc(F0)<<f6)|v8;uint B2=q5(L4,Bc);R J1=P1(B2>>f6);J1=n8(J1);if(J1==F0){if(!g6(S)){v8+=B2-max(Bc,B2);v8-=la;r5(L4,v8);}}else{d w0=m8(B2&w8);o8(J1,w0,M
#ifdef A
,B1
#endif
e3 U1);}M.xyz=O2(M.xyz,M.w,f0.xy,j.M3,j.N3);
#ifdef V
K1=M;
#else
p8(M U1);
#endif
#ifdef A
ka(B1 U1);
#endif
e6}
#endif
#if defined(DB)||defined(FB)
d6(IB){
#ifdef FB
q(K2,c);
#else
q(m1,d);
#endif
q(F0,R);uint B2=f3(L4);R J1=P1(B2>>f6);J1=n8(J1);uint ma;
#ifndef FB
if(J1==F0){ma=B2;}else
#endif
{ma=(Cc(F0)<<f6)+la;}d o;
#ifdef FB
o=clamp(o2(GD,na,K2,.0).x,I0(.0),I0(1.));
#else
o=m1;
#endif
int Nf=int(round(o*da));g3(L4,ma+uint(Nf));i M=G0(.0);
#ifdef A
K4 B1=l8;
#endif
#ifndef FB
if(J1!=F0)
#endif
{d oa=m8(B2&w8);o8(J1,oa,M
#ifdef A
,B1
#endif
e3 U1);}M.xyz=O2(M.xyz,M.w,f0.xy,j.M3,j.N3);
#ifdef V
K1=M;
#else
p8(M U1);
#endif
#ifdef A
ka(B1 U1);
#endif
e6}
#endif
#ifdef NE
d6(IB){q(f2,c);
#ifdef CD
q(j5,d);q(k5,f);
#endif
#ifdef AB
q(S0,f);
#endif
q(Q1,i);
#ifdef A
q(J3,R);
#endif
#ifdef O
q(H1,R);
#endif
i p2=x8(DC,v5,f2);d h6=1.;
#ifdef CD
h6=min(j5,h6);
#endif
#ifdef AB
if(AB){d n5=v3(w5(S0));h6=clamp(n5,I0(.0),h6);}
#endif
uint B2=f3(L4);R J1=P1(B2>>f6);J1=n8(J1);d oa=m8(B2&w8);i M;
#ifdef A
K4 B1=l8;
#endif
o8(J1,oa,M
#ifdef A
,B1
#endif
e3 U1);
#ifdef A
if(A&&J3!=0u){K4 U0=oc(B1)?B1:ca(m0);sc(J3,U0,h6);}
#endif
#ifdef CD
if(k5.w!=0.0){c pa=Dc(k5);i qa=o2(FD,ia,pa,0.0);qa.xyz*=qa.w;p2*=qa;}
#endif
p2*=Q1;
#if!defined(V)&&defined(O)
if(O&&H1!=M4){i I1=N0(n0)*(1.-M.w)+M;p2.xyz=i5(R6(p2),I1,H1)*p2.w;}
#endif
p2*=h6;
#if defined(CC)
p2=z3(p2);
#endif
M=M*(1.-p2.w)+p2;M.xyz=O2(M.xyz,M.w,f0.xy,j.M3,j.N3);
#ifdef V
K1=M;
#else
p8(M U1);
#endif
#ifdef A
ka(B1 U1);
#endif
g3(L4,la);e6}
#endif
#ifdef OE
d6(IB){
#ifndef V
#ifdef RD
if(RD){y0(n0,unpackUnorm4x8(j.Of));}
#endif
#ifdef SD
if(SD){y0(n0,p1(DC,H));}
#endif
#ifdef LF
i l=N0(n0);y0(n0,l.zyxw);
#endif
#endif
g3(L4,j.Pf);
#ifdef A
if(A){j1(m0,0u);}
#endif
#ifdef V
discard;
#endif
e6}
#endif
#ifdef SC
#ifdef ED
A2(IB)
#else
d6(IB)
#endif
{uint B2=f3(L4);d w0=m8(B2&w8);R J1=P1(B2>>f6);J1=n8(J1);i M;o8(J1,w0,M e3 U1);
#ifdef ED
float Y6=1.-M.w;if(Y6!=.0) M+=N0(n0)*Y6;K1=M;A3
#else
M.xyz=O2(M.xyz,M.w,f0.xy,j.M3,j.N3);
#ifdef V
K1=M;
#else
p8(M U1);
#endif
e6
#endif
}
#endif
#endif
