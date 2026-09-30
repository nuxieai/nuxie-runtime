#ifdef OD
#ifdef CB
g1(h0) I(0,f,VB);I(1,f,WB);h1
#endif
r2
#ifdef GB
I0 W(0,f,O);
#else
I0 W(0,D,O);
#endif
W2 W(1,N,D0);i2
#ifdef CB
A1(EC,h0,F,A,q){J(A,F,VB,f);J(A,F,WB,f);
#ifdef GB
V(O,f);
#else
V(O,D);
#endif
V(D0,N);f X;uint o0;c k0;f P;if(r9(VB,WB,q,o0,k0,P A3)){
#ifdef GB
O=P;
#else
O.xy=Q7(P.xy);
#endif
D0=a2(o0);X=P3(k0);}else{X=f(j.X2,j.X2,j.X2,j.X2);}c0(O);c0(D0);B1(X);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef CB
g1(h0) I(0,Q3,JB);h1
#endif
r2
#ifdef FB
I0 W(0,c,G2);
#else
MB W(0,d,i1);
#endif
W2 W(1,N,D0);i2
#ifdef CB
A1(EC,h0,F,A,q){J(A,F,JB,S);
#ifdef FB
V(G2,c);
#else
V(i1,d);
#endif
V(D0,N);uint o0;c k0;
#ifdef FB
k0=Jb(JB,o0,G2 A3);
#else
k0=Kb(JB,o0,i1 A3);
#endif
D0=a2(o0);f X=P3(k0);
#ifdef FB
c0(G2);
#else
c0(i1);
#endif
c0(D0);B1(X);}
#endif
#endif
#ifdef AD
#ifdef CB
g1(h0) I(0,f,FC);h1 g1(p1) I(v9,f,XB);I(w9,f,RB);I(x9,f,NB);I(y9,uint,YB);I(z9,uint,ZB);I(A9,uint,AC);I(B9,uint,LC);I(cf,f,PD);I(df,f,QD);I(ef,f,BD);I(Lb,f,OC);h1
#endif
r2 I0 W(0,c,c2);I0 W(1,d,a5);I0 W(2,f,c5);
#ifdef AB
I0 W(3,f,P0);
#endif
MB W(4,i,K1);
#ifdef K
W2 W(5,N,B3);
#endif
#ifdef T
W2 W(6,N,C1);
#endif
i2
#ifdef CB
R7(EC,h0,F,p1,g0,A,q){J(A,F,FC,f);J(q,g0,XB,f);J(q,g0,RB,f);J(q,g0,NB,f);J(q,g0,YB,uint);J(q,g0,ZB,uint);J(q,g0,AC,uint);J(q,g0,LC,uint);J(q,g0,PD,f);J(q,g0,QD,f);J(q,g0,BD,f);J(q,g0,OC,f);V(c2,c);V(a5,d);V(c5,f);
#ifdef AB
V(P0,f);
#endif
V(K1,i);
#ifdef K
V(B3,N);
#endif
#ifdef T
V(C1,N);
#endif
bool C9=FC.z==.0||FC.w==.0;a5=C9?.0:1.;c k0=FC.xy;Y W0=L1(XB);Y I6=transpose(inverse(W0));if(!C9){float D9=y4*E9(I6[1])/dot(W0[1],I6[1]);if(D9>=.5){k0.x=.5;a5*=R3(.5/D9);}else{k0.x+=D9*FC.z;}float F9=y4*E9(I6[0])/dot(W0[0],I6[0]);if(F9>=.5){k0.y=.5;a5*=R3(.5/F9);}else{k0.y+=F9*FC.w;}}Y ff=L1(PD);c2=N0(ff,k0)+BD.xy;k0=N0(W0,k0)+NB.xy;if(C9){c S3=N0(I6,FC.zw);S3*=E9(S3)/dot(S3,S3);k0+=y4*S3;}
#ifdef AB
if(AB){P0=S7(L1(RB),NB.zw,k0);}
#endif
K1=unpackUnorm4x8(YB);
#ifdef K
B3=a2(ZB);
#endif
#ifdef T
C1=a2(AC);
#endif
f X=P3(k0);c v0=k0;
#ifdef NE
if(j.Mb!=0u){v0.y=float(j.Nb)-v0.y;}
#endif
if(OC.w!=0.0){Y gf=L1(QD);c hf=BD.zw;c5=Ob(v0,gf,hf,OC.w,OC.xy,OC.z);}else{c5=f(.0,.0,.0,.0);}c0(c2);c0(a5);c0(c5);
#ifdef AB
c0(P0);
#endif
c0(K1);
#ifdef K
c0(B3);
#endif
#ifdef T
c0(C1);
#endif
B1(X);}
#endif
#elif defined(KB)
#ifdef CB
g1(n3) I(0,c,PC);h1 g1(C3) I(1,c,QC);h1 g1(p1) I(v9,f,XB);I(w9,f,RB);I(x9,f,NB);I(y9,uint,YB);I(z9,uint,ZB);I(A9,uint,AC);I(B9,uint,LC);I(G9,f,GC);h1
#endif
r2 I0 W(0,c,c2);
#ifdef AB
I0 W(1,f,P0);
#endif
MB W(3,i,K1);
#ifdef K
W2 W(4,N,B3);
#endif
#ifdef T
W2 W(5,N,C1);
#endif
i2
#ifdef CB
J6(EC,n3,o3,C3,D3,p1,g0,A){J(A,o3,PC,c);J(A,D3,QC,c);J(q,g0,XB,f);J(q,g0,RB,f);J(q,g0,NB,f);J(q,g0,YB,uint);J(q,g0,ZB,uint);J(q,g0,AC,uint);J(q,g0,LC,uint);J(q,g0,GC,f);V(c2,c);
#ifdef AB
V(P0,f);
#endif
V(K1,i);
#ifdef K
V(B3,N);
#endif
#ifdef T
V(C1,N);
#endif
Y W0=L1(XB);c k0=N0(W0,PC)+NB.xy;c2=QC*GC.zw+GC.xy;
#ifdef AB
if(AB){P0=S7(L1(RB),NB.zw,k0);}
#endif
K1=unpackUnorm4x8(YB);
#ifdef K
B3=a2(ZB);
#endif
#ifdef T
C1=a2(AC);
#endif
f X=P3(k0);c0(c2);
#ifdef AB
c0(P0);
#endif
c0(K1);
#ifdef K
c0(B3);
#endif
#ifdef T
c0(C1);
#endif
B1(X);}
#endif
#endif
#ifdef IF
#ifdef CB
g1(h0) h1
#endif
r2 i2
#ifdef CB
A1(EC,h0,F,A,q){d0 v2;v2.x=(A&1)==0?j.T7.x:j.T7.z;v2.y=(A&2)==0?j.T7.y:j.T7.w;f X=P3(c(v2));B1(X);}
#endif
#endif
#ifdef OE
#endif
#if defined(PE)&&!defined(Q)
#endif
#ifdef EB
M1
#ifndef Q
#ifdef QE
#define H9 QE
#else
#define H9 H2
#endif
#ifdef CD
z4(H9,l0);
#else
z0(H9,l0);
#endif
#endif
#ifdef WC
#define A4 i
#define I9 K0
#define U7 E0(.0)
#define Pb(E) ((E).w!=.0)
#ifdef K
#ifndef RC
z0(Y2,i0);
#else
z4(Y2,i0);
#endif
#endif
#else
#define A4 uint
#define U7 0u
#define I9 a1
#define Pb(E) ((E)!=0u)
#ifdef K
j1(Y2,i0);
#endif
#endif
I2(K6,B4);N1 T3 O5(Qb,kf,DD);P5(Rb,lf,PB);U3 e uint mf(float x){return uint(round(x*J9+K9));}e d V7(uint x){return R3(float(x)*Sb+(-K9*Sb));}N W7(N o0){
#ifdef JF
o0=min(o0,j.nf);
#endif
return o0;}
#ifdef K
e void Tb(uint l1,A4 Q0,L6(d) o){
#ifdef WC
if(all(lessThan(abs(Q0.xy-unpackUnorm4x8(l1).xy),E2(.25/255.)))) o=min(o,Q0.z);else o=.0;
#else
if(l1==Q0>>16) o=min(o,unpackHalf2x16(Q0).x);else o=.0;
#endif
}
#endif
e void X7(uint o0,d r0,c1(i) L
#if defined(K)&&!defined(RC)
,L6(A4) q1
#endif
M6 V3){O0 r1=R5(DD,o0);d o=r0;if((r1.x&(of|L9))!=0u){o=abs(o);
#ifdef XC
if(XC&&(r1.x&L9)!=0u){o=1.-abs(fract(o*.5)*2.+-1.);}
#endif
}o=clamp(o,J0(.0),J0(1.));
#ifdef K
if(K){uint l1=r1.x>>16u;if(l1!=0u){Tb(l1,I9(i0),o);}}
#endif
#ifdef AB
if(AB&&(r1.x&pf)!=0u){Y W0=L1(L0(PB,o0*E3+2u));f J2=L0(PB,o0*E3+3u);c qf=N0(W0,e0)+J2.xy;D Ub=Q7(abs(qf)*J2.zw-J2.zw);d d5=clamp(min(Ub.x,Ub.y)+.5,.0,1.);o=min(o,d5);}
#endif
uint W3=r1.x&0xfu;N p3=a2((r1.x>>4)&0xfu);
#ifdef T
bool e5=T&&p3!=C4;
#else
const bool e5=false;
#endif
if(W3<=Vb){L=unpackUnorm4x8(r1.y);
#ifdef K
if(K&&W3==Y7){
#ifndef RC
#ifdef WC
q1.xy=L.zw;q1.z=o;q1.w=1.;
#else
q1=r1.y|packHalf2x16(E2(o,.0));
#endif
#endif
L=E0(.0);}
#endif
}else{Y W0=L1(L0(PB,o0*E3));f J2=L0(PB,o0*E3+1u);c Wb=N0(W0,e0)+J2.xy;float t=W3==Xb?Wb.x:length(Wb);t=clamp(t,.0,1.);float x=t*J2.z+J2.w;float Yb=uintBitsToFloat(r1.y);float rf=floor(Yb)*j.Zb+j.ac;L=j2(ED,N9,c(x,rf),.0);if(!e5){L.xyz*=L.w;d O9=R3(fract(Yb)*(256./255.));L.w*=O9;}}
#if!defined(Q)&&defined(T)
if(e5){if(L.w*o!=.0){i O1=K0(l0);L.xyz=Z4(L.xyz,O1,p3);}L.xyz*=L.w;}
#endif
L*=o;
#if defined(BC)&&(defined(Q)||defined(RC))
L=q3(L);
#endif
}
#if!defined(Q)&&!defined(CD)
e void Z7(i L V3){
#ifndef WC
if(L.x+L.y+L.z+L.w==.0) return;float N6=1.-L.w;if(N6!=.0) L+=K0(l0)*N6;
#endif
A0(l0,L);}
#endif
#if defined(K)&&!defined(RC)
e void P9(A4 q1 V3){
#ifdef WC
A0(i0,q1);
#else
if(q1!=0u) d1(i0,q1);
#endif
}
#endif
#ifdef Q
#define S5 w2
#define T5 r3
#else
#define S5 P1
#define T5 d2
#endif
#ifdef OD
S5(HB){
#ifdef GB
r(O,f);
#else
r(O,D);
#endif
r(D0,N);d a8;
#ifdef GB
if(GB&&bc(O)){a8=D4(O e1);}else if(GB&&cc(O)){a8=c8(O e1);}else
#endif
{a8=min(min(J0(O.x),abs(J0(O.y))),J0(1.));}i L=E0(.0);
#ifdef K
A4 q1=U7;
#endif
uint d8=mf(a8);uint dc=(ec(D0)<<U5)|d8;uint x2=f5(B4,dc);N D1=a2(x2>>U5);D1=W7(D1);if(D1==D0){if(!V5(O)){d8+=x2-max(dc,x2);d8-=Q9;g5(B4,d8);}}else{d r0=V7(x2&e8);X7(D1,r0,L
#ifdef K
,q1
#endif
Z2 Q1);}L.xyz=L2(L.xyz,L.w,e0.xy,j.F3,j.G3);
#ifdef Q
E1=L;
#else
Z7(L Q1);
#endif
#ifdef K
P9(q1 Q1);
#endif
T5}
#endif
#if defined(DB)||defined(FB)
S5(HB){
#ifdef FB
r(G2,c);
#else
r(i1,d);
#endif
r(D0,N);uint x2=a3(B4);N D1=a2(x2>>U5);D1=W7(D1);uint R9;
#ifndef FB
if(D1==D0){R9=x2;}else
#endif
{R9=(ec(D0)<<U5)+Q9;}d o;
#ifdef FB
o=clamp(j2(FD,S9,G2,.0).x,J0(.0),J0(1.));
#else
o=i1;
#endif
int sf=int(round(o*J9));c3(B4,R9+uint(sf));i L=E0(.0);
#ifdef K
A4 q1=U7;
#endif
#ifndef FB
if(D1!=D0)
#endif
{d T9=V7(x2&e8);X7(D1,T9,L
#ifdef K
,q1
#endif
Z2 Q1);}L.xyz=L2(L.xyz,L.w,e0.xy,j.F3,j.G3);
#ifdef Q
E1=L;
#else
Z7(L Q1);
#endif
#ifdef K
P9(q1 Q1);
#endif
T5}
#endif
#ifdef OE
S5(HB){r(c2,c);
#ifdef AD
r(a5,d);r(c5,f);
#endif
#ifdef AB
r(P0,f);
#endif
r(K1,i);
#ifdef K
r(B3,N);
#endif
#ifdef T
r(C1,N);
#endif
i k2=f8(HC,W5,c2);d X5=1.;
#ifdef AD
X5=min(a5,X5);
#endif
#ifdef AB
if(AB){d d5=m3(h5(P0));X5=clamp(d5,J0(.0),X5);}
#endif
uint x2=a3(B4);N D1=a2(x2>>U5);D1=W7(D1);d T9=V7(x2&e8);i L;
#ifdef K
A4 q1=U7;
#endif
X7(D1,T9,L
#ifdef K
,q1
#endif
Z2 Q1);
#ifdef K
if(K&&B3!=0u){A4 Q0=Pb(q1)?q1:I9(i0);Tb(B3,Q0,X5);}
#endif
#ifdef AD
if(c5.w!=0.0){c U9=fc(c5);i V9=j2(ED,N9,U9,0.0);V9.xyz*=V9.w;k2*=V9;}
#endif
k2*=K1;
#if!defined(Q)&&defined(T)
if(T&&C1!=C4){i O1=K0(l0)*(1.-L.w)+L;k2.xyz=Z4(G6(k2),O1,C1)*k2.w;}
#endif
k2*=X5;
#if defined(BC)
k2=q3(k2);
#endif
L=L*(1.-k2.w)+k2;L.xyz=L2(L.xyz,L.w,e0.xy,j.F3,j.G3);
#ifdef Q
E1=L;
#else
Z7(L Q1);
#endif
#ifdef K
P9(q1 Q1);
#endif
c3(B4,Q9);T5}
#endif
#ifdef PE
S5(HB){
#ifndef Q
#ifdef RD
if(RD){A0(l0,unpackUnorm4x8(j.tf));}
#endif
#ifdef SD
if(SD){A0(l0,F1(HC,G));}
#endif
#ifdef KF
i k=K0(l0);A0(l0,k.zyxw);
#endif
#endif
c3(B4,j.uf);
#ifdef K
if(K){d1(i0,0u);}
#endif
#ifdef Q
discard;
#endif
T5}
#endif
#ifdef RC
#ifdef CD
w2(HB)
#else
S5(HB)
#endif
{uint x2=a3(B4);d r0=V7(x2&e8);N D1=a2(x2>>U5);D1=W7(D1);i L;X7(D1,r0,L Z2 Q1);
#ifdef CD
float N6=1.-L.w;if(N6!=.0) L+=K0(l0)*N6;E1=L;r3
#else
L.xyz=L2(L.xyz,L.w,e0.xy,j.F3,j.G3);
#ifdef Q
E1=L;
#else
Z7(L Q1);
#endif
T5
#endif
}
#endif
#endif
