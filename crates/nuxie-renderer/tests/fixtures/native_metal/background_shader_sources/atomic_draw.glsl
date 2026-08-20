#ifdef ID
#ifdef DB
g1(e0)L(0,g,UB);L(1,g,VB);h1
#endif
k2
#ifdef HB
J0 W(0,g,O);
#else
J0 W(0,E,O);
#endif
O2 W(1,N,B0);f2
#ifdef DB
y1(GC,e0,F,B,r){M(B,F,UB,g);M(B,F,VB,g);
#ifdef HB
V(O,g);
#else
V(O,E);
#endif
V(B0,N);g U;uint o0;d l0;g P;if(q9(UB,VB,r,o0,l0,P v3)){
#ifdef HB
O=P;
#else
O.xy=O7(P.xy);
#endif
B0=W1(o0);U=K3(l0);}else{U=g(n.P2,n.P2,n.P2,n.P2);}a0(O);a0(B0);z1(U);}
#endif
#endif
#if defined(EB)||defined(FB)
#ifdef DB
g1(e0)L(0,L3,KB);h1
#endif
k2
#ifdef FB
J0 W(0,d,C2);
#else
MB W(0,c,i1);
#endif
O2 W(1,N,B0);f2
#ifdef DB
y1(GC,e0,F,B,r){M(B,F,KB,c0);
#ifdef FB
V(C2,d);
#else
V(i1,c);
#endif
V(B0,N);uint o0;d l0;
#ifdef FB
l0=Db(KB,o0,C2 v3);
#else
l0=Eb(KB,o0,i1 v3);
#endif
B0=W1(o0);g U=K3(l0);
#ifdef FB
a0(C2);
#else
a0(i1);
#endif
a0(B0);z1(U);}
#endif
#endif
#ifdef JD
#ifdef DB
g1(e0)L(0,g,HC);h1 g1(n1)L(r9,g,WB);L(v9,g,QB);L(w9,g,NB);
#ifdef M3
L(x9,uint,XB);L(y9,uint,YB);L(z9,uint,ZB);L(A9,uint,AC);
#else
L(B9,G,IB);
#endif
h1
#endif
k2 J0 W(0,d,X1);J0 W(1,c,R4);
#ifdef BB
J0 W(2,g,L0);
#endif
MB W(3,c,H1);
#ifdef I
O2 W(4,N,w3);
#endif
#ifdef AB
O2 W(5,N,A1);
#endif
f2
#ifdef DB
P7(GC,e0,F,n1,f0,B,r){M(B,F,HC,g);M(r,f0,WB,g);M(r,f0,QB,g);M(r,f0,NB,g);
#ifdef M3
M(r,f0,XB,uint);M(r,f0,YB,uint);M(r,f0,ZB,uint);M(r,f0,AC,uint);G IB=G(XB,YB,ZB,AC);
#else
M(r,f0,IB,G);
#endif
V(X1,d);V(R4,c);
#ifdef BB
V(L0,g);
#endif
V(H1,c);
#ifdef I
V(w3,N);
#endif
#ifdef AB
V(A1,N);
#endif
bool C9=HC.z==.0||HC.w==.0;R4=C9?.0:1.;d l0=HC.xy;g0 T0=l2(WB);g0 E6=transpose(inverse(T0));if(!C9){float D9=n4*E9(E6[1])/dot(T0[1],E6[1]);if(D9>=.5){l0.x=.5;R4*=S4(.5/D9);}else{l0.x+=D9*HC.z;}float F9=n4*E9(E6[0])/dot(T0[0],E6[0]);if(F9>=.5){l0.y=.5;R4*=S4(.5/F9);}else{l0.y+=F9*HC.w;}}X1=l0;l0=U0(T0,l0)+NB.xy;if(C9){d N3=U0(E6,HC.zw);N3*=E9(N3)/dot(N3,N3);l0+=n4*N3;}
#ifdef BB
if(BB){L0=Q7(l2(QB),NB.zw,l0);}
#endif
H1=uintBitsToFloat(IB.x);
#ifdef I
w3=W1(IB.y);
#endif
#ifdef AB
A1=W1(IB.z);
#endif
g U=K3(l0);a0(X1);a0(R4);
#ifdef BB
a0(L0);
#endif
a0(H1);
#ifdef I
a0(w3);
#endif
#ifdef AB
a0(A1);
#endif
z1(U);}
#endif
#elif defined(OB)
#ifdef DB
g1(g3)L(0,d,OC);h1 g1(x3)L(1,d,PC);h1 g1(n1)L(r9,g,WB);L(v9,g,QB);L(w9,g,NB);
#ifdef M3
L(x9,uint,XB);L(y9,uint,YB);L(z9,uint,ZB);L(A9,uint,AC);
#else
L(B9,G,IB);
#endif
h1
#endif
k2 J0 W(0,d,X1);
#ifdef BB
J0 W(1,g,L0);
#endif
MB W(3,c,H1);
#ifdef I
O2 W(4,N,w3);
#endif
#ifdef AB
O2 W(5,N,A1);
#endif
f2
#ifdef DB
F6(GC,g3,h3,x3,y3,n1,f0,B){M(B,h3,OC,d);M(B,y3,PC,d);M(r,f0,WB,g);M(r,f0,QB,g);M(r,f0,NB,g);
#ifdef M3
M(r,f0,XB,uint);M(r,f0,YB,uint);M(r,f0,ZB,uint);M(r,f0,AC,uint);G IB=G(XB,YB,ZB,AC);
#else
M(r,f0,IB,G);
#endif
V(X1,d);
#ifdef BB
V(L0,g);
#endif
V(H1,c);
#ifdef I
V(w3,N);
#endif
#ifdef AB
V(A1,N);
#endif
g0 T0=l2(WB);d l0=U0(T0,OC)+NB.xy;X1=PC;
#ifdef BB
if(BB){L0=Q7(l2(QB),NB.zw,l0);}
#endif
H1=uintBitsToFloat(IB.x);
#ifdef I
w3=W1(IB.y);
#endif
#ifdef AB
A1=W1(IB.z);
#endif
g U=K3(l0);a0(X1);
#ifdef BB
a0(L0);
#endif
a0(H1);
#ifdef I
a0(w3);
#endif
#ifdef AB
a0(A1);
#endif
z1(U);}
#endif
#endif
#ifdef AF
#ifdef DB
g1(e0)h1
#endif
k2 f2
#ifdef DB
y1(GC,e0,F,B,r){X m2;m2.x=(B&1)==0?n.R7.x:n.R7.z;m2.y=(B&2)==0?n.R7.y:n.R7.w;g U=K3(d(m2));z1(U);}
#endif
#endif
#ifdef HE
#endif
#if defined(IE)&&!defined(Q)
#endif
#ifdef GB
I1
#ifndef Q
#ifdef JE
#define G9 JE
#else
#define G9 Q2
#endif
#ifdef ZC
o4(G9,j0);
#else
w0(G9,j0);
#endif
#endif
#ifdef VC
#define p4 i
#define H9 H0
#define S7 C0(.0)
#define Fb(q) ((q).w!=.0)
#ifdef I
#ifndef QC
w0(R2,h0);
#else
o4(R2,h0);
#endif
#endif
#else
#define p4 uint
#define S7 0u
#define H9 Y0
#define Fb(q) ((q)!=0u)
#ifdef I
j1(R2,h0);
#endif
#endif
D2(G6,q4);J1 O3 K5(Gb,Je,AD);L5(Hb,Ke,RB);P3 e uint Le(float x){return uint(round(x*I9+J9));}e c T7(uint x){return S4(float(x)*Ib+(-J9*Ib));}N U7(N o0){
#ifdef BF
o0=min(o0,n.Me);
#endif
return o0;}
#ifdef I
e void Jb(uint k1,p4 M0,T4(c)o){
#ifdef VC
if(all(lessThan(abs(M0.xy-unpackUnorm4x8(k1).xy),A2(.25/255.))))o=min(o,M0.z);else o=.0;
#else
if(k1==M0>>16)o=min(o,unpackHalf2x16(M0).x);else o=.0;
#endif
}
#endif
e void V7(uint o0,c p0,Z0(i)R
#if defined(I)&&!defined(QC)
,T4(p4)o1
#endif
H6 Q3){a1 p1=M5(AD,o0);c o=p0;if((p1.x&(Ne|K9))!=0u){o=abs(o);
#ifdef WC
if(WC&&(p1.x&K9)!=0u){o=1.-abs(fract(o*.5)*2.+-1.);}
#endif
}o=clamp(o,G0(.0),G0(1.));
#ifdef I
if(I){uint k1=p1.x>>16u;if(k1!=0u){Jb(k1,H9(h0),o);}}
#endif
#ifdef BB
if(BB&&(p1.x&Oe)!=0u){g0 T0=l2(N0(RB,o0*4u+2u));g i3=N0(RB,o0*4u+3u);d Pe=U0(T0,Y)+i3.xy;E Kb=O7(abs(Pe)*i3.zw-i3.zw);c U4=clamp(min(Kb.x,Kb.y)+.5,.0,1.);o=min(o,U4);}
#endif
uint j3=p1.x&0xfu;if(j3<=Lb){R=unpackUnorm4x8(p1.y);
#ifdef I
if(I&&j3==W7){
#ifndef QC
#ifdef VC
o1.xy=R.zw;o1.z=o;o1.w=1.;
#else
o1=p1.y|packHalf2x16(A2(o,.0));
#endif
#endif
R=C0(.0);}
#endif
}else{g0 T0=l2(N0(RB,o0*4u));g i3=N0(RB,o0*4u+1u);d V4=U0(T0,Y)+i3.xy;float t=j3==M9?V4.x:length(V4);t=clamp(t,.0,1.);float x=t*i3.z+i3.w;float y=uintBitsToFloat(p1.y);R=n2(KD,Mb,d(x,y),.0);}R.w*=o;
#if!defined(Q)&&defined(AB)
N R3;if(AB&&R.w!=.0&&(R3=W1((p1.x>>4)&0xfu))!=N5){i K1=H0(j0);R.xyz=Q4(R.xyz,K1,R3);}
#endif
#if defined(BC)&&(defined(Q)||defined(QC))
R=k3(R);
#endif
R.xyz*=R.w;}
#if!defined(Q)&&!defined(ZC)
e void X7(i R Q3){
#ifndef VC
if(R.w==.0)return;float I6=1.-R.w;if(I6!=.0)R+=H0(j0)*I6;
#endif
x0(j0,R);}
#endif
#if defined(I)&&!defined(QC)
e void N9(p4 o1 Q3){
#ifdef VC
x0(h0,o1);
#else
if(o1!=0u)c1(h0,o1);
#endif
}
#endif
#ifdef Q
#define O5 o2
#define P5 l3
#else
#define O5 L1
#define P5 Y1
#endif
#ifdef ID
O5(JB){
#ifdef HB
A(O,g);
#else
A(O,E);
#endif
A(B0,N);c Y7;
#ifdef HB
if(HB&&Nb(O)){Y7=v4(O d1);}else if(HB&&Ob(O)){Y7=Z7(O d1);}else
#endif
{Y7=min(min(G0(O.x),abs(G0(O.y))),G0(1.));}i R=C0(.0);
#ifdef I
p4 o1=S7;
#endif
uint a8=Le(Y7);uint Pb=(Qb(B0)<<Q5)|a8;uint p2=W4(q4,Pb);N B1=W1(p2>>Q5);B1=U7(B1);if(B1==B0){if(!R5(O)){a8+=p2-max(Pb,p2);a8-=O9;X4(q4,a8);}}else{c p0=T7(p2&c8);V7(B1,p0,R
#ifdef I
,o1
#endif
S2 M1);}R.xyz=E2(R.xyz,R.w,Y.xy,n.z3,n.A3);
#ifdef Q
C1=R;
#else
X7(R M1);
#endif
#ifdef I
N9(o1 M1);
#endif
P5}
#endif
#if defined(EB)||defined(FB)
O5(JB){
#ifdef FB
A(C2,d);
#else
A(i1,c);
#endif
A(B0,N);uint p2=T2(q4);N B1=W1(p2>>Q5);B1=U7(B1);uint P9;
#ifndef FB
if(B1==B0){P9=p2;}else
#endif
{P9=(Qb(B0)<<Q5)+O9;}c o;
#ifdef FB
o=clamp(n2(BD,Q9,C2,.0).x,G0(.0),G0(1.));
#else
o=i1;
#endif
int Qe=int(round(o*I9));U2(q4,P9+uint(Qe));i R=C0(.0);
#ifdef I
p4 o1=S7;
#endif
#ifndef FB
if(B1!=B0)
#endif
{c R9=T7(p2&c8);V7(B1,R9,R
#ifdef I
,o1
#endif
S2 M1);}R.xyz=E2(R.xyz,R.w,Y.xy,n.z3,n.A3);
#ifdef Q
C1=R;
#else
X7(R M1);
#endif
#ifdef I
N9(o1 M1);
#endif
P5}
#endif
#ifdef HE
O5(JB){A(X1,d);
#ifdef JD
A(R4,c);
#endif
#ifdef BB
A(L0,g);
#endif
A(H1,c);
#ifdef I
A(w3,N);
#endif
#ifdef AB
A(A1,N);
#endif
i w4=d8(IC,S5,X1);c T5=1.;
#ifdef JD
T5=min(R4,T5);
#endif
#ifdef BB
if(BB){c U4=f3(Y4(L0));T5=clamp(U4,G0(.0),T5);}
#endif
uint p2=T2(q4);N B1=W1(p2>>Q5);B1=U7(B1);c R9=T7(p2&c8);i R;
#ifdef I
p4 o1=S7;
#endif
V7(B1,R9,R
#ifdef I
,o1
#endif
S2 M1);
#ifdef I
if(I&&w3!=0u){p4 M0=Fb(o1)?o1:H9(h0);Jb(w3,M0,T5);}
#endif
#if!defined(Q)&&defined(AB)
if(AB&&A1!=N5){i K1=H0(j0)*(1.-R.w)+R;w4.xyz=Q4(C6(w4),K1,A1)*w4.w;}
#endif
w4*=T5*H1;
#if defined(BC)
w4=k3(w4);
#endif
R=R*(1.-w4.w)+w4;R.xyz=E2(R.xyz,R.w,Y.xy,n.z3,n.A3);
#ifdef Q
C1=R;
#else
X7(R M1);
#endif
#ifdef I
N9(o1 M1);
#endif
U2(q4,O9);P5}
#endif
#ifdef IE
O5(JB){
#ifndef Q
#ifdef LD
if(LD){x0(j0,unpackUnorm4x8(n.Re));}
#endif
#ifdef MD
if(MD){x0(j0,q1(IC,J));}
#endif
#ifdef CF
i j=H0(j0);x0(j0,j.zyxw);
#endif
#endif
U2(q4,n.Se);
#ifdef I
if(I){c1(h0,0u);}
#endif
#ifdef Q
discard;
#endif
P5}
#endif
#ifdef QC
#ifdef ZC
o2(JB)
#else
O5(JB)
#endif
{uint p2=T2(q4);c p0=T7(p2&c8);N B1=W1(p2>>Q5);B1=U7(B1);i R;V7(B1,p0,R S2 M1);
#ifdef ZC
float I6=1.-R.w;if(I6!=.0)R+=H0(j0)*I6;C1=R;l3
#else
R.xyz=E2(R.xyz,R.w,Y.xy,n.z3,n.A3);
#ifdef Q
C1=R;
#else
X7(R M1);
#endif
P5
#endif
}
#endif
#endif
