#ifdef ND
#ifdef CB
h1(g0)K(0,f,UB);K(1,f,VB);i1
#endif
q2
#ifdef GB
I0 W(0,f,O);
#else
I0 W(0,E,O);
#endif
V2 W(1,N,D0);i2
#ifdef CB
B1(EC,g0,F,B,v){L(B,F,UB,f);L(B,F,VB,f);
#ifdef GB
U(O,f);
#else
U(O,E);
#endif
U(D0,N);f X;uint o0;c l0;f P;if(x9(UB,VB,v,o0,l0,P A3)){
#ifdef GB
O=P;
#else
O.xy=U7(P.xy);
#endif
D0=a2(o0);X=Q3(l0);}else{X=f(j.W2,j.W2,j.W2,j.W2);}c0(O);c0(D0);C1(X);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef CB
h1(g0)K(0,R3,JB);i1
#endif
q2
#ifdef FB
I0 W(0,c,F2);
#else
MB W(0,d,j1);
#endif
V2 W(1,N,D0);i2
#ifdef CB
B1(EC,g0,F,B,v){L(B,F,JB,R);
#ifdef FB
U(F2,c);
#else
U(j1,d);
#endif
U(D0,N);uint o0;c l0;
#ifdef FB
l0=Nb(JB,o0,F2 A3);
#else
l0=Ob(JB,o0,j1 A3);
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
#ifdef ZC
#ifdef CB
h1(g0)K(0,f,FC);i1 h1(p1)K(y9,f,WB);K(z9,f,RB);K(A9,f,NB);K(B9,uint,XB);K(C9,uint,YB);K(D9,uint,ZB);K(E9,uint,LC);K(hf,f,OD);K(jf,f,PD);K(kf,f,AD);K(Pb,f,NC);i1
#endif
q2 I0 W(0,c,c2);I0 W(1,d,a5);I0 W(2,f,Q5);
#ifdef AB
I0 W(3,f,O0);
#endif
MB W(4,i,K1);
#ifdef I
V2 W(5,N,B3);
#endif
#ifdef S
V2 W(6,N,D1);
#endif
i2
#ifdef CB
V7(EC,g0,F,p1,h0,B,v){L(B,F,FC,f);L(v,h0,WB,f);L(v,h0,RB,f);L(v,h0,NB,f);L(v,h0,XB,uint);L(v,h0,YB,uint);L(v,h0,ZB,uint);L(v,h0,LC,uint);L(v,h0,OD,f);L(v,h0,PD,f);L(v,h0,AD,f);L(v,h0,NC,f);U(c2,c);U(a5,d);U(Q5,f);
#ifdef AB
U(O0,f);
#endif
U(K1,i);
#ifdef I
U(B3,N);
#endif
#ifdef S
U(D1,N);
#endif
bool F9=FC.z==.0||FC.w==.0;a5=F9?.0:1.;c l0=FC.xy;e0 W0=L1(WB);e0 K6=transpose(inverse(W0));if(!F9){float G9=x4*H9(K6[1])/dot(W0[1],K6[1]);if(G9>=.5){l0.x=.5;a5*=S3(.5/G9);}else{l0.x+=G9*FC.z;}float I9=x4*H9(K6[0])/dot(W0[0],K6[0]);if(I9>=.5){l0.y=.5;a5*=S3(.5/I9);}else{l0.y+=I9*FC.w;}}e0 lf=L1(OD);c2=P0(lf,l0)+AD.xy;l0=P0(W0,l0)+NB.xy;if(F9){c T3=P0(K6,FC.zw);T3*=H9(T3)/dot(T3,T3);l0+=x4*T3;}
#ifdef AB
if(AB){O0=W7(L1(RB),NB.zw,l0);}
#endif
K1=unpackUnorm4x8(XB);
#ifdef I
B3=a2(YB);
#endif
#ifdef S
D1=a2(ZB);
#endif
f X=Q3(l0);c v0=l0;
#ifdef ME
if(j.Qb!=0u){v0.y=float(j.Rb)-v0.y;}
#endif
if(NC.w!=0.0){e0 mf=L1(PD);c nf=AD.zw;Q5=Sb(v0,mf,nf,NC.w,NC.xy,NC.z);}c0(c2);c0(a5);c0(Q5);
#ifdef AB
c0(O0);
#endif
c0(K1);
#ifdef I
c0(B3);
#endif
#ifdef S
c0(D1);
#endif
C1(X);}
#endif
#elif defined(KB)
#ifdef CB
h1(n3)K(0,c,OC);i1 h1(C3)K(1,c,PC);i1 h1(p1)K(y9,f,WB);K(z9,f,RB);K(A9,f,NB);K(B9,uint,XB);K(C9,uint,YB);K(D9,uint,ZB);K(E9,uint,LC);i1
#endif
q2 I0 W(0,c,c2);
#ifdef AB
I0 W(1,f,O0);
#endif
MB W(3,i,K1);
#ifdef I
V2 W(4,N,B3);
#endif
#ifdef S
V2 W(5,N,D1);
#endif
i2
#ifdef CB
L6(EC,n3,o3,C3,D3,p1,h0,B){L(B,o3,OC,c);L(B,D3,PC,c);L(v,h0,WB,f);L(v,h0,RB,f);L(v,h0,NB,f);L(v,h0,XB,uint);L(v,h0,YB,uint);L(v,h0,ZB,uint);L(v,h0,LC,uint);U(c2,c);
#ifdef AB
U(O0,f);
#endif
U(K1,i);
#ifdef I
U(B3,N);
#endif
#ifdef S
U(D1,N);
#endif
e0 W0=L1(WB);c l0=P0(W0,OC)+NB.xy;c2=PC;
#ifdef AB
if(AB){O0=W7(L1(RB),NB.zw,l0);}
#endif
K1=unpackUnorm4x8(XB);
#ifdef I
B3=a2(YB);
#endif
#ifdef S
D1=a2(ZB);
#endif
f X=Q3(l0);c0(c2);
#ifdef AB
c0(O0);
#endif
c0(K1);
#ifdef I
c0(B3);
#endif
#ifdef S
c0(D1);
#endif
C1(X);}
#endif
#endif
#ifdef HF
#ifdef CB
h1(g0)i1
#endif
q2 i2
#ifdef CB
B1(EC,g0,F,B,v){Z r2;r2.x=(B&1)==0?j.X7.x:j.X7.z;r2.y=(B&2)==0?j.X7.y:j.X7.w;f X=Q3(c(r2));C1(X);}
#endif
#endif
#ifdef NE
#endif
#if defined(OE)&&!defined(Q)
#endif
#ifdef EB
M1
#ifndef Q
#ifdef PE
#define J9 PE
#else
#define J9 G2
#endif
#ifdef BD
y4(J9,m0);
#else
z0(J9,m0);
#endif
#endif
#ifdef VC
#define z4 i
#define K9 K0
#define Y7 E0(.0)
#define Tb(q) ((q).w!=.0)
#ifdef I
#ifndef QC
z0(X2,i0);
#else
y4(X2,i0);
#endif
#endif
#else
#define z4 uint
#define Y7 0u
#define K9 a1
#define Tb(q) ((q)!=0u)
#ifdef I
k1(X2,i0);
#endif
#endif
H2(M6,A4);N1 U3 R5(Ub,pf,CD);S5(Vb,qf,PB);V3 e uint rf(float x){return uint(round(x*L9+M9));}e d Z7(uint x){return S3(float(x)*Wb+(-M9*Wb));}N a8(N o0){
#ifdef IF
o0=min(o0,j.sf);
#endif
return o0;}
#ifdef I
e void Xb(uint m1,z4 Q0,N6(d)o){
#ifdef VC
if(all(lessThan(abs(Q0.xy-unpackUnorm4x8(m1).xy),D2(.25/255.))))o=min(o,Q0.z);else o=.0;
#else
if(m1==Q0>>16)o=min(o,unpackHalf2x16(Q0).x);else o=.0;
#endif
}
#endif
e void c8(uint o0,d r0,c1(i)J
#if defined(I)&&!defined(QC)
,N6(z4)q1
#endif
O6 W3){N0 r1=U5(CD,o0);d o=r0;if((r1.x&(tf|N9))!=0u){o=abs(o);
#ifdef WC
if(WC&&(r1.x&N9)!=0u){o=1.-abs(fract(o*.5)*2.+-1.);}
#endif
}o=clamp(o,J0(.0),J0(1.));
#ifdef I
if(I){uint m1=r1.x>>16u;if(m1!=0u){Xb(m1,K9(i0),o);}}
#endif
#ifdef AB
if(AB&&(r1.x&uf)!=0u){e0 W0=L1(L0(PB,o0*E3+2u));f I2=L0(PB,o0*E3+3u);c vf=P0(W0,d0)+I2.xy;E Yb=U7(abs(vf)*I2.zw-I2.zw);d c5=clamp(min(Yb.x,Yb.y)+.5,.0,1.);o=min(o,c5);}
#endif
uint X3=r1.x&0xfu;N p3=a2((r1.x>>4)&0xfu);
#ifdef S
bool d5=S&&p3!=B4;
#else
const bool d5=false;
#endif
if(X3<=Zb){J=unpackUnorm4x8(r1.y);
#ifdef I
if(I&&X3==d8){
#ifndef QC
#ifdef VC
q1.xy=J.zw;q1.z=o;q1.w=1.;
#else
q1=r1.y|packHalf2x16(D2(o,.0));
#endif
#endif
J=E0(.0);}
#endif
}else{e0 W0=L1(L0(PB,o0*E3));f I2=L0(PB,o0*E3+1u);c ac=P0(W0,d0)+I2.xy;float t=X3==bc?ac.x:length(ac);t=clamp(t,.0,1.);float x=t*I2.z+I2.w;float cc=uintBitsToFloat(r1.y);float wf=floor(cc)*j.dc+j.ec;J=j2(DD,P9,c(x,wf),.0);if(!d5){J.xyz*=J.w;d Q9=S3(fract(cc)*(256./255.));J.w*=Q9;}}
#if!defined(Q)&&defined(S)
if(d5){if(J.w*o!=.0){i O1=K0(m0);J.xyz=Z4(J.xyz,O1,p3);}J.xyz*=J.w;}
#endif
J*=o;
#if defined(AC)&&(defined(Q)||defined(QC))
J=q3(J);
#endif
}
#if!defined(Q)&&!defined(BD)
e void e8(i J W3){
#ifndef VC
if(J.x+J.y+J.z+J.w==.0)return;float P6=1.-J.w;if(P6!=.0)J+=K0(m0)*P6;
#endif
A0(m0,J);}
#endif
#if defined(I)&&!defined(QC)
e void R9(z4 q1 W3){
#ifdef VC
A0(i0,q1);
#else
if(q1!=0u)d1(i0,q1);
#endif
}
#endif
#ifdef Q
#define V5 v2
#define W5 r3
#else
#define V5 P1
#define W5 d2
#endif
#ifdef ND
V5(HB){
#ifdef GB
r(O,f);
#else
r(O,E);
#endif
r(D0,N);d f8;
#ifdef GB
if(GB&&fc(O)){f8=C4(O e1);}else if(GB&&gc(O)){f8=g8(O e1);}else
#endif
{f8=min(min(J0(O.x),abs(J0(O.y))),J0(1.));}i J=E0(.0);
#ifdef I
z4 q1=Y7;
#endif
uint h8=rf(f8);uint hc=(ic(D0)<<X5)|h8;uint w2=e5(A4,hc);N E1=a2(w2>>X5);E1=a8(E1);if(E1==D0){if(!Y5(O)){h8+=w2-max(hc,w2);h8-=S9;f5(A4,h8);}}else{d r0=Z7(w2&i8);c8(E1,r0,J
#ifdef I
,q1
#endif
Y2 Q1);}J.xyz=K2(J.xyz,J.w,d0.xy,j.F3,j.G3);
#ifdef Q
F1=J;
#else
e8(J Q1);
#endif
#ifdef I
R9(q1 Q1);
#endif
W5}
#endif
#if defined(DB)||defined(FB)
V5(HB){
#ifdef FB
r(F2,c);
#else
r(j1,d);
#endif
r(D0,N);uint w2=Z2(A4);N E1=a2(w2>>X5);E1=a8(E1);uint T9;
#ifndef FB
if(E1==D0){T9=w2;}else
#endif
{T9=(ic(D0)<<X5)+S9;}d o;
#ifdef FB
o=clamp(j2(ED,U9,F2,.0).x,J0(.0),J0(1.));
#else
o=j1;
#endif
int xf=int(round(o*L9));a3(A4,T9+uint(xf));i J=E0(.0);
#ifdef I
z4 q1=Y7;
#endif
#ifndef FB
if(E1!=D0)
#endif
{d V9=Z7(w2&i8);c8(E1,V9,J
#ifdef I
,q1
#endif
Y2 Q1);}J.xyz=K2(J.xyz,J.w,d0.xy,j.F3,j.G3);
#ifdef Q
F1=J;
#else
e8(J Q1);
#endif
#ifdef I
R9(q1 Q1);
#endif
W5}
#endif
#ifdef NE
V5(HB){r(c2,c);
#ifdef ZC
r(a5,d);r(Q5,f);
#endif
#ifdef AB
r(O0,f);
#endif
r(K1,i);
#ifdef I
r(B3,N);
#endif
#ifdef S
r(D1,N);
#endif
i k2=j8(GC,Z5,c2);d a6=1.;
#ifdef ZC
a6=min(a5,a6);
#endif
#ifdef AB
if(AB){d c5=m3(g5(O0));a6=clamp(c5,J0(.0),a6);}
#endif
uint w2=Z2(A4);N E1=a2(w2>>X5);E1=a8(E1);d V9=Z7(w2&i8);i J;
#ifdef I
z4 q1=Y7;
#endif
c8(E1,V9,J
#ifdef I
,q1
#endif
Y2 Q1);
#ifdef I
if(I&&B3!=0u){z4 Q0=Tb(q1)?q1:K9(i0);Xb(B3,Q0,a6);}
#endif
#ifdef ZC
if(Q5.w!=0.0){c W9=jc(Q5);i X9=j2(DD,P9,W9,0.0);X9.xyz*=X9.w;k2*=X9;}
#endif
k2*=K1;
#if!defined(Q)&&defined(S)
if(S&&D1!=B4){i O1=K0(m0)*(1.-J.w)+J;k2.xyz=Z4(I6(k2),O1,D1)*k2.w;}
#endif
k2*=a6;
#if defined(AC)
k2=q3(k2);
#endif
J=J*(1.-k2.w)+k2;J.xyz=K2(J.xyz,J.w,d0.xy,j.F3,j.G3);
#ifdef Q
F1=J;
#else
e8(J Q1);
#endif
#ifdef I
R9(q1 Q1);
#endif
a3(A4,S9);W5}
#endif
#ifdef OE
V5(HB){
#ifndef Q
#ifdef QD
if(QD){A0(m0,unpackUnorm4x8(j.yf));}
#endif
#ifdef RD
if(RD){A0(m0,v1(GC,G));}
#endif
#ifdef JF
i k=K0(m0);A0(m0,k.zyxw);
#endif
#endif
a3(A4,j.zf);
#ifdef I
if(I){d1(i0,0u);}
#endif
#ifdef Q
discard;
#endif
W5}
#endif
#ifdef QC
#ifdef BD
v2(HB)
#else
V5(HB)
#endif
{uint w2=Z2(A4);d r0=Z7(w2&i8);N E1=a2(w2>>X5);E1=a8(E1);i J;c8(E1,r0,J Y2 Q1);
#ifdef BD
float P6=1.-J.w;if(P6!=.0)J+=K0(m0)*P6;F1=J;r3
#else
J.xyz=K2(J.xyz,J.w,d0.xy,j.F3,j.G3);
#ifdef Q
F1=J;
#else
e8(J Q1);
#endif
W5
#endif
}
#endif
#endif
