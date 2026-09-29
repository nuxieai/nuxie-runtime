#ifdef LD
#ifdef DB
g1(e0)L(0,g,VB);L(1,g,WB);h1
#endif
m2
#ifdef HB
H0 X(0,g,O);
#else
H0 X(0,E,O);
#endif
Q2 X(1,N,B0);g2
#ifdef DB
z1(HC,e0,F,B,v){M(B,F,VB,g);M(B,F,WB,g);
#ifdef HB
V(O,g);
#else
V(O,E);
#endif
V(B0,N);g W;uint l0;d m0;g P;if(r9(VB,WB,v,l0,m0,P w3)){
#ifdef HB
O=P;
#else
O.xy=S7(P.xy);
#endif
B0=X1(l0);W=M3(m0);}else{W=g(m.R2,m.R2,m.R2,m.R2);}c0(O);c0(B0);A1(W);}
#endif
#endif
#if defined(EB)||defined(FB)
#ifdef DB
g1(e0)L(0,N3,LB);h1
#endif
m2
#ifdef FB
H0 X(0,d,D2);
#else
NB X(0,c,i1);
#endif
Q2 X(1,N,B0);g2
#ifdef DB
z1(HC,e0,F,B,v){M(B,F,LB,R);
#ifdef FB
V(D2,d);
#else
V(i1,c);
#endif
V(B0,N);uint l0;d m0;
#ifdef FB
m0=Ib(LB,l0,D2 w3);
#else
m0=Jb(LB,l0,i1 w3);
#endif
B0=X1(l0);g W=M3(m0);
#ifdef FB
c0(D2);
#else
c0(i1);
#endif
c0(B0);A1(W);}
#endif
#endif
#ifdef MD
#ifdef DB
g1(e0)L(0,g,IC);h1 g1(n1)L(v9,g,XB);L(w9,g,TB);L(x9,g,OB);
#ifdef O3
L(y9,uint,YB);L(z9,uint,ZB);L(A9,uint,AC);L(B9,uint,BC);
#else
L(C9,H,IB);
#endif
h1
#endif
m2 H0 X(0,d,Y1);H0 X(1,c,W4);
#ifdef BB
H0 X(2,g,M0);
#endif
NB X(3,c,I1);
#ifdef J
Q2 X(4,N,x3);
#endif
#ifdef AB
Q2 X(5,N,B1);
#endif
g2
#ifdef DB
T7(HC,e0,F,n1,g0,B,v){M(B,F,IC,g);M(v,g0,XB,g);M(v,g0,TB,g);M(v,g0,OB,g);
#ifdef O3
M(v,g0,YB,uint);M(v,g0,ZB,uint);M(v,g0,AC,uint);M(v,g0,BC,uint);H IB=H(YB,ZB,AC,BC);
#else
M(v,g0,IB,H);
#endif
V(Y1,d);V(W4,c);
#ifdef BB
V(M0,g);
#endif
V(I1,c);
#ifdef J
V(x3,N);
#endif
#ifdef AB
V(B1,N);
#endif
bool D9=IC.z==.0||IC.w==.0;W4=D9?.0:1.;d m0=IC.xy;f0 U0=h2(XB);f0 J6=transpose(inverse(U0));if(!D9){float E9=q4*F9(J6[1])/dot(U0[1],J6[1]);if(E9>=.5){m0.x=.5;W4*=X4(.5/E9);}else{m0.x+=E9*IC.z;}float G9=q4*F9(J6[0])/dot(U0[0],J6[0]);if(G9>=.5){m0.y=.5;W4*=X4(.5/G9);}else{m0.y+=G9*IC.w;}}Y1=m0;m0=R0(U0,m0)+OB.xy;if(D9){d P3=R0(J6,IC.zw);P3*=F9(P3)/dot(P3,P3);m0+=q4*P3;}
#ifdef BB
if(BB){M0=U7(h2(TB),OB.zw,m0);}
#endif
I1=uintBitsToFloat(IB.x);
#ifdef J
x3=X1(IB.y);
#endif
#ifdef AB
B1=X1(IB.z);
#endif
g W=M3(m0);c0(Y1);c0(W4);
#ifdef BB
c0(M0);
#endif
c0(I1);
#ifdef J
c0(x3);
#endif
#ifdef AB
c0(B1);
#endif
A1(W);}
#endif
#elif defined(PB)
#ifdef DB
g1(i3)L(0,d,PC);h1 g1(y3)L(1,d,QC);h1 g1(n1)L(v9,g,XB);L(w9,g,TB);L(x9,g,OB);
#ifdef O3
L(y9,uint,YB);L(z9,uint,ZB);L(A9,uint,AC);L(B9,uint,BC);
#else
L(C9,H,IB);
#endif
h1
#endif
m2 H0 X(0,d,Y1);
#ifdef BB
H0 X(1,g,M0);
#endif
NB X(3,c,I1);
#ifdef J
Q2 X(4,N,x3);
#endif
#ifdef AB
Q2 X(5,N,B1);
#endif
g2
#ifdef DB
K6(HC,i3,j3,y3,z3,n1,g0,B){M(B,j3,PC,d);M(B,z3,QC,d);M(v,g0,XB,g);M(v,g0,TB,g);M(v,g0,OB,g);
#ifdef O3
M(v,g0,YB,uint);M(v,g0,ZB,uint);M(v,g0,AC,uint);M(v,g0,BC,uint);H IB=H(YB,ZB,AC,BC);
#else
M(v,g0,IB,H);
#endif
V(Y1,d);
#ifdef BB
V(M0,g);
#endif
V(I1,c);
#ifdef J
V(x3,N);
#endif
#ifdef AB
V(B1,N);
#endif
f0 U0=h2(XB);d m0=R0(U0,PC)+OB.xy;Y1=QC;
#ifdef BB
if(BB){M0=U7(h2(TB),OB.zw,m0);}
#endif
I1=uintBitsToFloat(IB.x);
#ifdef J
x3=X1(IB.y);
#endif
#ifdef AB
B1=X1(IB.z);
#endif
g W=M3(m0);c0(Y1);
#ifdef BB
c0(M0);
#endif
c0(I1);
#ifdef J
c0(x3);
#endif
#ifdef AB
c0(B1);
#endif
A1(W);}
#endif
#endif
#ifdef DF
#ifdef DB
g1(e0)h1
#endif
m2 g2
#ifdef DB
z1(HC,e0,F,B,v){Y n2;n2.x=(B&1)==0?m.V7.x:m.V7.z;n2.y=(B&2)==0?m.V7.y:m.V7.w;g W=M3(d(n2));A1(W);}
#endif
#endif
#ifdef KE
#endif
#if defined(LE)&&!defined(Q)
#endif
#ifdef GB
J1
#ifndef Q
#ifdef ME
#define H9 ME
#else
#define H9 S2
#endif
#ifdef AD
r4(H9,j0);
#else
x0(H9,j0);
#endif
#endif
#ifdef WC
#define v4 i
#define I9 I0
#define W7 C0(.0)
#define Kb(q) ((q).w!=.0)
#ifdef J
#ifndef RC
x0(T2,h0);
#else
r4(T2,h0);
#endif
#endif
#else
#define v4 uint
#define W7 0u
#define I9 Y0
#define Kb(q) ((q)!=0u)
#ifdef J
j1(T2,h0);
#endif
#endif
E2(L6,w4);K1 Q3 O5(Lb,Re,BD);P5(Mb,Se,RB);R3 e uint Te(float x){return uint(round(x*J9+K9));}e c X7(uint x){return X4(float(x)*Nb+(-K9*Nb));}N Y7(N l0){
#ifdef EF
l0=min(l0,m.Ue);
#endif
return l0;}
#ifdef J
e void Ob(uint k1,v4 N0,Y4(c)n){
#ifdef WC
if(all(lessThan(abs(N0.xy-unpackUnorm4x8(k1).xy),B2(.25/255.))))n=min(n,N0.z);else n=.0;
#else
if(k1==N0>>16)n=min(n,unpackHalf2x16(N0).x);else n=.0;
#endif
}
#endif
e void Z7(uint l0,c p0,Z0(i)S
#if defined(J)&&!defined(RC)
,Y4(v4)o1
#endif
M6 S3){a1 p1=Q5(BD,l0);c n=p0;if((p1.x&(Ve|L9))!=0u){n=abs(n);
#ifdef XC
if(XC&&(p1.x&L9)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,G0(.0),G0(1.));
#ifdef J
if(J){uint k1=p1.x>>16u;if(k1!=0u){Ob(k1,I9(h0),n);}}
#endif
#ifdef BB
if(BB&&(p1.x&We)!=0u){f0 U0=h2(J0(RB,l0*A3+2u));g k3=J0(RB,l0*A3+3u);d Xe=R0(U0,a0)+k3.xy;E Pb=S7(abs(Xe)*k3.zw-k3.zw);c Z4=clamp(min(Pb.x,Pb.y)+.5,.0,1.);n=min(n,Z4);}
#endif
uint l3=p1.x&0xfu;if(l3<=Qb){S=unpackUnorm4x8(p1.y);
#ifdef J
if(J&&l3==a8){
#ifndef RC
#ifdef WC
o1.xy=S.zw;o1.z=n;o1.w=1.;
#else
o1=p1.y|packHalf2x16(B2(n,.0));
#endif
#endif
S=C0(.0);}
#endif
}else{f0 U0=h2(J0(RB,l0*A3));g k3=J0(RB,l0*A3+1u);d y4=R0(U0,a0)+k3.xy;float t=l3==N9?y4.x:length(y4);t=clamp(t,.0,1.);float x=t*k3.z+k3.w;float y=uintBitsToFloat(p1.y);S=o2(ND,Rb,d(x,y),.0);}S.w*=n;
#if!defined(Q)&&defined(AB)
N T3;if(AB&&S.w!=.0&&(T3=X1((p1.x>>4)&0xfu))!=R5){i L1=I0(j0);S.xyz=V4(S.xyz,L1,T3);}
#endif
#if defined(CC)&&(defined(Q)||defined(RC))
S=m3(S);
#endif
S.xyz*=S.w;}
#if!defined(Q)&&!defined(AD)
e void c8(i S S3){
#ifndef WC
if(S.w==.0)return;float N6=1.-S.w;if(N6!=.0)S+=I0(j0)*N6;
#endif
y0(j0,S);}
#endif
#if defined(J)&&!defined(RC)
e void O9(v4 o1 S3){
#ifdef WC
y0(h0,o1);
#else
if(o1!=0u)c1(h0,o1);
#endif
}
#endif
#ifdef Q
#define S5 p2
#define T5 n3
#else
#define S5 M1
#define T5 Z1
#endif
#ifdef LD
S5(JB){
#ifdef HB
r(O,g);
#else
r(O,E);
#endif
r(B0,N);c d8;
#ifdef HB
if(HB&&Sb(O)){d8=z4(O d1);}else if(HB&&Tb(O)){d8=e8(O d1);}else
#endif
{d8=min(min(G0(O.x),abs(G0(O.y))),G0(1.));}i S=C0(.0);
#ifdef J
v4 o1=W7;
#endif
uint f8=Te(d8);uint Ub=(Vb(B0)<<U5)|f8;uint q2=a5(w4,Ub);N C1=X1(q2>>U5);C1=Y7(C1);if(C1==B0){if(!V5(O)){f8+=q2-max(Ub,q2);f8-=P9;c5(w4,f8);}}else{c p0=X7(q2&g8);Z7(C1,p0,S
#ifdef J
,o1
#endif
U2 N1);}S.xyz=F2(S.xyz,S.w,a0.xy,m.B3,m.C3);
#ifdef Q
D1=S;
#else
c8(S N1);
#endif
#ifdef J
O9(o1 N1);
#endif
T5}
#endif
#if defined(EB)||defined(FB)
S5(JB){
#ifdef FB
r(D2,d);
#else
r(i1,c);
#endif
r(B0,N);uint q2=V2(w4);N C1=X1(q2>>U5);C1=Y7(C1);uint Q9;
#ifndef FB
if(C1==B0){Q9=q2;}else
#endif
{Q9=(Vb(B0)<<U5)+P9;}c n;
#ifdef FB
n=clamp(o2(CD,R9,D2,.0).x,G0(.0),G0(1.));
#else
n=i1;
#endif
int Ye=int(round(n*J9));W2(w4,Q9+uint(Ye));i S=C0(.0);
#ifdef J
v4 o1=W7;
#endif
#ifndef FB
if(C1!=B0)
#endif
{c S9=X7(q2&g8);Z7(C1,S9,S
#ifdef J
,o1
#endif
U2 N1);}S.xyz=F2(S.xyz,S.w,a0.xy,m.B3,m.C3);
#ifdef Q
D1=S;
#else
c8(S N1);
#endif
#ifdef J
O9(o1 N1);
#endif
T5}
#endif
#ifdef KE
S5(JB){r(Y1,d);
#ifdef MD
r(W4,c);
#endif
#ifdef BB
r(M0,g);
#endif
r(I1,c);
#ifdef J
r(x3,N);
#endif
#ifdef AB
r(B1,N);
#endif
i G2=h8(JC,W5,Y1);c X5=1.;
#ifdef MD
X5=min(W4,X5);
#endif
#ifdef BB
if(BB){c Z4=h3(d5(M0));X5=clamp(Z4,G0(.0),X5);}
#endif
uint q2=V2(w4);N C1=X1(q2>>U5);C1=Y7(C1);c S9=X7(q2&g8);i S;
#ifdef J
v4 o1=W7;
#endif
Z7(C1,S9,S
#ifdef J
,o1
#endif
U2 N1);
#ifdef J
if(J&&x3!=0u){v4 N0=Kb(o1)?o1:I9(h0);Ob(x3,N0,X5);}
#endif
#if!defined(Q)&&defined(AB)
if(AB&&B1!=R5){i L1=I0(j0)*(1.-S.w)+S;G2.xyz=V4(H6(G2),L1,B1)*G2.w;}
#endif
G2*=X5*I1;
#if defined(CC)
G2=m3(G2);
#endif
S=S*(1.-G2.w)+G2;S.xyz=F2(S.xyz,S.w,a0.xy,m.B3,m.C3);
#ifdef Q
D1=S;
#else
c8(S N1);
#endif
#ifdef J
O9(o1 N1);
#endif
W2(w4,P9);T5}
#endif
#ifdef LE
S5(JB){
#ifndef Q
#ifdef OD
if(OD){y0(j0,unpackUnorm4x8(m.Ze));}
#endif
#ifdef PD
if(PD){y0(j0,q1(JC,G));}
#endif
#ifdef FF
i j=I0(j0);y0(j0,j.zyxw);
#endif
#endif
W2(w4,m.af);
#ifdef J
if(J){c1(h0,0u);}
#endif
#ifdef Q
discard;
#endif
T5}
#endif
#ifdef RC
#ifdef AD
p2(JB)
#else
S5(JB)
#endif
{uint q2=V2(w4);c p0=X7(q2&g8);N C1=X1(q2>>U5);C1=Y7(C1);i S;Z7(C1,p0,S U2 N1);
#ifdef AD
float N6=1.-S.w;if(N6!=.0)S+=I0(j0)*N6;D1=S;n3
#else
S.xyz=F2(S.xyz,S.w,a0.xy,m.B3,m.C3);
#ifdef Q
D1=S;
#else
c8(S N1);
#endif
T5
#endif
}
#endif
#endif
