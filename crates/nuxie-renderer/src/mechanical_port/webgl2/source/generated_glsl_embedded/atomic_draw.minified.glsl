#ifdef OD
#ifdef CB
h1(h0) I(0,f,VB);I(1,f,WB);i1
#endif
q2
#ifdef GB
I0 W(0,f,O);
#else
I0 W(0,D,O);
#endif
V2 W(1,N,D0);i2
#ifdef CB
B1(EC,h0,F,A,q){J(A,F,VB,f);J(A,F,WB,f);
#ifdef GB
V(O,f);
#else
V(O,D);
#endif
V(D0,N);f X;uint o0;c l0;f P;if(r9(VB,WB,q,o0,l0,P A3)){
#ifdef GB
O=P;
#else
O.xy=R7(P.xy);
#endif
D0=a2(o0);X=Q3(l0);}else{X=f(j.W2,j.W2,j.W2,j.W2);}c0(O);c0(D0);C1(X);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef CB
h1(h0) I(0,R3,JB);i1
#endif
q2
#ifdef FB
I0 W(0,c,F2);
#else
MB W(0,d,j1);
#endif
V2 W(1,N,D0);i2
#ifdef CB
B1(EC,h0,F,A,q){J(A,F,JB,S);
#ifdef FB
V(F2,c);
#else
V(j1,d);
#endif
V(D0,N);uint o0;c l0;
#ifdef FB
l0=Kb(JB,o0,F2 A3);
#else
l0=Lb(JB,o0,j1 A3);
#endif
D0=a2(o0);f X=Q3(l0);
#ifdef FB
c0(F2);
#else
c0(j1);
#endif
c0(D0);C1(X);}
#endif
#endif
#ifdef AD
#ifdef CB
h1(h0) I(0,f,FC);i1 h1(p1) I(v9,f,XB);I(w9,f,RB);I(x9,f,NB);I(y9,uint,YB);I(z9,uint,ZB);I(A9,uint,AC);I(B9,uint,MC);I(cf,f,PD);I(df,f,QD);I(ef,f,BD);I(Mb,f,OC);i1
#endif
q2 I0 W(0,c,c2);I0 W(1,d,Z4);I0 W(2,f,a5);
#ifdef AB
I0 W(3,f,O0);
#endif
MB W(4,i,K1);
#ifdef K
V2 W(5,N,B3);
#endif
#ifdef T
V2 W(6,N,D1);
#endif
i2
#ifdef CB
S7(EC,h0,F,p1,g0,A,q){J(A,F,FC,f);J(q,g0,XB,f);J(q,g0,RB,f);J(q,g0,NB,f);J(q,g0,YB,uint);J(q,g0,ZB,uint);J(q,g0,AC,uint);J(q,g0,MC,uint);J(q,g0,PD,f);J(q,g0,QD,f);J(q,g0,BD,f);J(q,g0,OC,f);V(c2,c);V(Z4,d);V(a5,f);
#ifdef AB
V(O0,f);
#endif
V(K1,i);
#ifdef K
V(B3,N);
#endif
#ifdef T
V(D1,N);
#endif
bool C9=FC.z==.0||FC.w==.0;Z4=C9?.0:1.;c l0=FC.xy;e0 W0=L1(XB);e0 H6=transpose(inverse(W0));if(!C9){float D9=x4*E9(H6[1])/dot(W0[1],H6[1]);if(D9>=.5){l0.x=.5;Z4*=S3(.5/D9);}else{l0.x+=D9*FC.z;}float F9=x4*E9(H6[0])/dot(W0[0],H6[0]);if(F9>=.5){l0.y=.5;Z4*=S3(.5/F9);}else{l0.y+=F9*FC.w;}}e0 ff=L1(PD);c2=P0(ff,l0)+BD.xy;l0=P0(W0,l0)+NB.xy;if(C9){c T3=P0(H6,FC.zw);T3*=E9(T3)/dot(T3,T3);l0+=x4*T3;}
#ifdef AB
if(AB){O0=T7(L1(RB),NB.zw,l0);}
#endif
K1=unpackUnorm4x8(YB);
#ifdef K
B3=a2(ZB);
#endif
#ifdef T
D1=a2(AC);
#endif
f X=Q3(l0);c v0=l0;
#ifdef NE
if(j.Nb!=0u){v0.y=float(j.Ob)-v0.y;}
#endif
if(OC.w!=0.0){e0 gf=L1(QD);c hf=BD.zw;a5=Pb(v0,gf,hf,OC.w,OC.xy,OC.z);}else{a5=f(.0,.0,.0,.0);}c0(c2);c0(Z4);c0(a5);
#ifdef AB
c0(O0);
#endif
c0(K1);
#ifdef K
c0(B3);
#endif
#ifdef T
c0(D1);
#endif
C1(X);}
#endif
#elif defined(KB)
#ifdef CB
h1(n3) I(0,c,PC);i1 h1(C3) I(1,c,QC);i1 h1(p1) I(v9,f,XB);I(w9,f,RB);I(x9,f,NB);I(y9,uint,YB);I(z9,uint,ZB);I(A9,uint,AC);I(B9,uint,MC);I(G9,f,GC);i1
#endif
q2 I0 W(0,c,c2);
#ifdef AB
I0 W(1,f,O0);
#endif
MB W(3,i,K1);
#ifdef K
V2 W(4,N,B3);
#endif
#ifdef T
V2 W(5,N,D1);
#endif
i2
#ifdef CB
I6(EC,n3,o3,C3,D3,p1,g0,A){J(A,o3,PC,c);J(A,D3,QC,c);J(q,g0,XB,f);J(q,g0,RB,f);J(q,g0,NB,f);J(q,g0,YB,uint);J(q,g0,ZB,uint);J(q,g0,AC,uint);J(q,g0,MC,uint);J(q,g0,GC,f);V(c2,c);
#ifdef AB
V(O0,f);
#endif
V(K1,i);
#ifdef K
V(B3,N);
#endif
#ifdef T
V(D1,N);
#endif
e0 W0=L1(XB);c l0=P0(W0,PC)+NB.xy;c2=QC*GC.zw+GC.xy;
#ifdef AB
if(AB){O0=T7(L1(RB),NB.zw,l0);}
#endif
K1=unpackUnorm4x8(YB);
#ifdef K
B3=a2(ZB);
#endif
#ifdef T
D1=a2(AC);
#endif
f X=Q3(l0);c0(c2);
#ifdef AB
c0(O0);
#endif
c0(K1);
#ifdef K
c0(B3);
#endif
#ifdef T
c0(D1);
#endif
C1(X);}
#endif
#endif
#ifdef IF
#ifdef CB
h1(h0) i1
#endif
q2 i2
#ifdef CB
B1(EC,h0,F,A,q){Y r2;r2.x=(A&1)==0?j.U7.x:j.U7.z;r2.y=(A&2)==0?j.U7.y:j.U7.w;f X=Q3(c(r2));C1(X);}
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
#define H9 G2
#endif
#ifdef CD
y4(H9,m0);
#else
z0(H9,m0);
#endif
#endif
#ifdef WC
#define z4 i
#define I9 K0
#define V7 E0(.0)
#define Qb(E) ((E).w!=.0)
#ifdef K
#ifndef RC
z0(X2,i0);
#else
y4(X2,i0);
#endif
#endif
#else
#define z4 uint
#define V7 0u
#define I9 a1
#define Qb(E) ((E)!=0u)
#ifdef K
k1(X2,i0);
#endif
#endif
H2(J6,A4);N1 U3 O5(Rb,kf,DD);P5(Sb,lf,PB);V3 e uint mf(float x){return uint(round(x*J9+K9));}e d W7(uint x){return S3(float(x)*Tb+(-K9*Tb));}N X7(N o0){
#ifdef JF
o0=min(o0,j.nf);
#endif
return o0;}
#ifdef K
e void Ub(uint m1,z4 Q0,K6(d) o){
#ifdef WC
if(all(lessThan(abs(Q0.xy-unpackUnorm4x8(m1).xy),D2(.25/255.)))) o=min(o,Q0.z);else o=.0;
#else
if(m1==Q0>>16) o=min(o,unpackHalf2x16(Q0).x);else o=.0;
#endif
}
#endif
e void Y7(uint o0,d r0,c1(i) L
#if defined(K)&&!defined(RC)
,K6(z4) q1
#endif
L6 W3){N0 r1=R5(DD,o0);d o=r0;if((r1.x&(of|L9))!=0u){o=abs(o);
#ifdef XC
if(XC&&(r1.x&L9)!=0u){o=1.-abs(fract(o*.5)*2.+-1.);}
#endif
}o=clamp(o,J0(.0),J0(1.));
#ifdef K
if(K){uint m1=r1.x>>16u;if(m1!=0u){Ub(m1,I9(i0),o);}}
#endif
#ifdef AB
if(AB&&(r1.x&pf)!=0u){e0 W0=L1(L0(PB,o0*E3+2u));f I2=L0(PB,o0*E3+3u);c qf=P0(W0,d0)+I2.xy;D Vb=R7(abs(qf)*I2.zw-I2.zw);d c5=clamp(min(Vb.x,Vb.y)+.5,.0,1.);o=min(o,c5);}
#endif
uint X3=r1.x&0xfu;N p3=a2((r1.x>>4)&0xfu);
#ifdef T
bool d5=T&&p3!=B4;
#else
const bool d5=false;
#endif
if(X3<=Wb){L=unpackUnorm4x8(r1.y);
#ifdef K
if(K&&X3==Z7){
#ifndef RC
#ifdef WC
q1.xy=L.zw;q1.z=o;q1.w=1.;
#else
q1=r1.y|packHalf2x16(D2(o,.0));
#endif
#endif
L=E0(.0);}
#endif
}else{e0 W0=L1(L0(PB,o0*E3));f I2=L0(PB,o0*E3+1u);c Xb=P0(W0,d0)+I2.xy;float t=X3==Yb?Xb.x:length(Xb);t=clamp(t,.0,1.);float x=t*I2.z+I2.w;float Zb=uintBitsToFloat(r1.y);float rf=floor(Zb)*j.ac+j.bc;L=j2(ED,N9,c(x,rf),.0);if(!d5){L.xyz*=L.w;d O9=S3(fract(Zb)*(256./255.));L.w*=O9;}}
#if!defined(Q)&&defined(T)
if(d5){if(L.w*o!=.0){i O1=K0(m0);L.xyz=Y4(L.xyz,O1,p3);}L.xyz*=L.w;}
#endif
L*=o;
#if defined(BC)&&(defined(Q)||defined(RC))
L=q3(L);
#endif
}
#if!defined(Q)&&!defined(CD)
e void a8(i L W3){
#ifndef WC
if(L.x+L.y+L.z+L.w==.0) return;float M6=1.-L.w;if(M6!=.0) L+=K0(m0)*M6;
#endif
A0(m0,L);}
#endif
#if defined(K)&&!defined(RC)
e void P9(z4 q1 W3){
#ifdef WC
A0(i0,q1);
#else
if(q1!=0u) d1(i0,q1);
#endif
}
#endif
#ifdef Q
#define S5 v2
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
r(D0,N);d c8;
#ifdef GB
if(GB&&cc(O)){c8=C4(O e1);}else if(GB&&dc(O)){c8=d8(O e1);}else
#endif
{c8=min(min(J0(O.x),abs(J0(O.y))),J0(1.));}i L=E0(.0);
#ifdef K
z4 q1=V7;
#endif
uint e8=mf(c8);uint ec=(fc(D0)<<U5)|e8;uint w2=e5(A4,ec);N E1=a2(w2>>U5);E1=X7(E1);if(E1==D0){if(!V5(O)){e8+=w2-max(ec,w2);e8-=Q9;f5(A4,e8);}}else{d r0=W7(w2&f8);Y7(E1,r0,L
#ifdef K
,q1
#endif
Y2 Q1);}L.xyz=K2(L.xyz,L.w,d0.xy,j.F3,j.G3);
#ifdef Q
F1=L;
#else
a8(L Q1);
#endif
#ifdef K
P9(q1 Q1);
#endif
T5}
#endif
#if defined(DB)||defined(FB)
S5(HB){
#ifdef FB
r(F2,c);
#else
r(j1,d);
#endif
r(D0,N);uint w2=Z2(A4);N E1=a2(w2>>U5);E1=X7(E1);uint R9;
#ifndef FB
if(E1==D0){R9=w2;}else
#endif
{R9=(fc(D0)<<U5)+Q9;}d o;
#ifdef FB
o=clamp(j2(FD,S9,F2,.0).x,J0(.0),J0(1.));
#else
o=j1;
#endif
int sf=int(round(o*J9));a3(A4,R9+uint(sf));i L=E0(.0);
#ifdef K
z4 q1=V7;
#endif
#ifndef FB
if(E1!=D0)
#endif
{d T9=W7(w2&f8);Y7(E1,T9,L
#ifdef K
,q1
#endif
Y2 Q1);}L.xyz=K2(L.xyz,L.w,d0.xy,j.F3,j.G3);
#ifdef Q
F1=L;
#else
a8(L Q1);
#endif
#ifdef K
P9(q1 Q1);
#endif
T5}
#endif
#ifdef OE
S5(HB){r(c2,c);
#ifdef AD
r(Z4,d);r(a5,f);
#endif
#ifdef AB
r(O0,f);
#endif
r(K1,i);
#ifdef K
r(B3,N);
#endif
#ifdef T
r(D1,N);
#endif
i k2=g8(HC,W5,c2);d X5=1.;
#ifdef AD
X5=min(Z4,X5);
#endif
#ifdef AB
if(AB){d c5=m3(g5(O0));X5=clamp(c5,J0(.0),X5);}
#endif
uint w2=Z2(A4);N E1=a2(w2>>U5);E1=X7(E1);d T9=W7(w2&f8);i L;
#ifdef K
z4 q1=V7;
#endif
Y7(E1,T9,L
#ifdef K
,q1
#endif
Y2 Q1);
#ifdef K
if(K&&B3!=0u){z4 Q0=Qb(q1)?q1:I9(i0);Ub(B3,Q0,X5);}
#endif
#ifdef AD
if(a5.w!=0.0){c U9=gc(a5);i V9=j2(ED,N9,U9,0.0);V9.xyz*=V9.w;k2*=V9;}
#endif
k2*=K1;
#if!defined(Q)&&defined(T)
if(T&&D1!=B4){i O1=K0(m0)*(1.-L.w)+L;k2.xyz=Y4(F6(k2),O1,D1)*k2.w;}
#endif
k2*=X5;
#if defined(BC)
k2=q3(k2);
#endif
L=L*(1.-k2.w)+k2;L.xyz=K2(L.xyz,L.w,d0.xy,j.F3,j.G3);
#ifdef Q
F1=L;
#else
a8(L Q1);
#endif
#ifdef K
P9(q1 Q1);
#endif
a3(A4,Q9);T5}
#endif
#ifdef PE
S5(HB){
#ifndef Q
#ifdef RD
if(RD){A0(m0,unpackUnorm4x8(j.tf));}
#endif
#ifdef SD
if(SD){A0(m0,v1(HC,G));}
#endif
#ifdef KF
i k=K0(m0);A0(m0,k.zyxw);
#endif
#endif
a3(A4,j.uf);
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
v2(HB)
#else
S5(HB)
#endif
{uint w2=Z2(A4);d r0=W7(w2&f8);N E1=a2(w2>>U5);E1=X7(E1);i L;Y7(E1,r0,L Y2 Q1);
#ifdef CD
float M6=1.-L.w;if(M6!=.0) L+=K0(m0)*M6;F1=L;r3
#else
L.xyz=K2(L.xyz,L.w,d0.xy,j.F3,j.G3);
#ifdef Q
F1=L;
#else
a8(L Q1);
#endif
T5
#endif
}
#endif
#endif
