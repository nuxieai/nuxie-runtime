#ifdef KD
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
y1(HC,e0,F,B,v){M(B,F,VB,g);M(B,F,WB,g);
#ifdef HB
V(O,g);
#else
V(O,E);
#endif
V(B0,N);g W;uint l0;d m0;g P;if(q9(VB,WB,v,l0,m0,P w3)){
#ifdef HB
O=P;
#else
O.xy=R7(P.xy);
#endif
B0=X1(l0);W=M3(m0);}else{W=g(m.R2,m.R2,m.R2,m.R2);}a0(O);a0(B0);z1(W);}
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
y1(HC,e0,F,B,v){M(B,F,LB,R);
#ifdef FB
V(D2,d);
#else
V(i1,c);
#endif
V(B0,N);uint l0;d m0;
#ifdef FB
m0=Gb(LB,l0,D2 w3);
#else
m0=Hb(LB,l0,i1 w3);
#endif
B0=X1(l0);g W=M3(m0);
#ifdef FB
a0(D2);
#else
a0(i1);
#endif
a0(B0);z1(W);}
#endif
#endif
#ifdef LD
#ifdef DB
g1(e0)L(0,g,IC);h1 g1(n1)L(r9,g,XB);L(v9,g,SB);L(w9,g,OB);
#ifdef O3
L(x9,uint,YB);L(y9,uint,ZB);L(z9,uint,AC);L(A9,uint,BC);
#else
L(B9,G,IB);
#endif
h1
#endif
m2 H0 X(0,d,Y1);H0 X(1,c,U4);
#ifdef BB
H0 X(2,g,M0);
#endif
NB X(3,c,H1);
#ifdef I
Q2 X(4,N,x3);
#endif
#ifdef AB
Q2 X(5,N,A1);
#endif
g2
#ifdef DB
S7(HC,e0,F,n1,g0,B,v){M(B,F,IC,g);M(v,g0,XB,g);M(v,g0,SB,g);M(v,g0,OB,g);
#ifdef O3
M(v,g0,YB,uint);M(v,g0,ZB,uint);M(v,g0,AC,uint);M(v,g0,BC,uint);G IB=G(YB,ZB,AC,BC);
#else
M(v,g0,IB,G);
#endif
V(Y1,d);V(U4,c);
#ifdef BB
V(M0,g);
#endif
V(H1,c);
#ifdef I
V(x3,N);
#endif
#ifdef AB
V(A1,N);
#endif
bool C9=IC.z==.0||IC.w==.0;U4=C9?.0:1.;d m0=IC.xy;f0 U0=h2(XB);f0 H6=transpose(inverse(U0));if(!C9){float D9=p4*E9(H6[1])/dot(U0[1],H6[1]);if(D9>=.5){m0.x=.5;U4*=V4(.5/D9);}else{m0.x+=D9*IC.z;}float F9=p4*E9(H6[0])/dot(U0[0],H6[0]);if(F9>=.5){m0.y=.5;U4*=V4(.5/F9);}else{m0.y+=F9*IC.w;}}Y1=m0;m0=R0(U0,m0)+OB.xy;if(C9){d P3=R0(H6,IC.zw);P3*=E9(P3)/dot(P3,P3);m0+=p4*P3;}
#ifdef BB
if(BB){M0=T7(h2(SB),OB.zw,m0);}
#endif
H1=uintBitsToFloat(IB.x);
#ifdef I
x3=X1(IB.y);
#endif
#ifdef AB
A1=X1(IB.z);
#endif
g W=M3(m0);a0(Y1);a0(U4);
#ifdef BB
a0(M0);
#endif
a0(H1);
#ifdef I
a0(x3);
#endif
#ifdef AB
a0(A1);
#endif
z1(W);}
#endif
#elif defined(PB)
#ifdef DB
g1(i3)L(0,d,PC);h1 g1(y3)L(1,d,QC);h1 g1(n1)L(r9,g,XB);L(v9,g,SB);L(w9,g,OB);
#ifdef O3
L(x9,uint,YB);L(y9,uint,ZB);L(z9,uint,AC);L(A9,uint,BC);
#else
L(B9,G,IB);
#endif
h1
#endif
m2 H0 X(0,d,Y1);
#ifdef BB
H0 X(1,g,M0);
#endif
NB X(3,c,H1);
#ifdef I
Q2 X(4,N,x3);
#endif
#ifdef AB
Q2 X(5,N,A1);
#endif
g2
#ifdef DB
I6(HC,i3,j3,y3,z3,n1,g0,B){M(B,j3,PC,d);M(B,z3,QC,d);M(v,g0,XB,g);M(v,g0,SB,g);M(v,g0,OB,g);
#ifdef O3
M(v,g0,YB,uint);M(v,g0,ZB,uint);M(v,g0,AC,uint);M(v,g0,BC,uint);G IB=G(YB,ZB,AC,BC);
#else
M(v,g0,IB,G);
#endif
V(Y1,d);
#ifdef BB
V(M0,g);
#endif
V(H1,c);
#ifdef I
V(x3,N);
#endif
#ifdef AB
V(A1,N);
#endif
f0 U0=h2(XB);d m0=R0(U0,PC)+OB.xy;Y1=QC;
#ifdef BB
if(BB){M0=T7(h2(SB),OB.zw,m0);}
#endif
H1=uintBitsToFloat(IB.x);
#ifdef I
x3=X1(IB.y);
#endif
#ifdef AB
A1=X1(IB.z);
#endif
g W=M3(m0);a0(Y1);
#ifdef BB
a0(M0);
#endif
a0(H1);
#ifdef I
a0(x3);
#endif
#ifdef AB
a0(A1);
#endif
z1(W);}
#endif
#endif
#ifdef CF
#ifdef DB
g1(e0)h1
#endif
m2 g2
#ifdef DB
y1(HC,e0,F,B,v){Y n2;n2.x=(B&1)==0?m.U7.x:m.U7.z;n2.y=(B&2)==0?m.U7.y:m.U7.w;g W=M3(d(n2));z1(W);}
#endif
#endif
#ifdef KE
#endif
#if defined(LE)&&!defined(Q)
#endif
#ifdef GB
I1
#ifndef Q
#ifdef ME
#define G9 ME
#else
#define G9 S2
#endif
#ifdef AD
q4(G9,j0);
#else
x0(G9,j0);
#endif
#endif
#ifdef WC
#define r4 i
#define H9 I0
#define V7 C0(.0)
#define Ib(q) ((q).w!=.0)
#ifdef I
#ifndef RC
x0(T2,h0);
#else
q4(T2,h0);
#endif
#endif
#else
#define r4 uint
#define V7 0u
#define H9 Y0
#define Ib(q) ((q)!=0u)
#ifdef I
j1(T2,h0);
#endif
#endif
E2(J6,v4);J1 Q3 N5(Jb,Ne,BD);O5(Kb,Oe,RB);R3 e uint Pe(float x){return uint(round(x*I9+J9));}e c W7(uint x){return V4(float(x)*Lb+(-J9*Lb));}N X7(N l0){
#ifdef DF
l0=min(l0,m.Qe);
#endif
return l0;}
#ifdef I
e void Mb(uint k1,r4 N0,W4(c)n){
#ifdef WC
if(all(lessThan(abs(N0.xy-unpackUnorm4x8(k1).xy),B2(.25/255.))))n=min(n,N0.z);else n=.0;
#else
if(k1==N0>>16)n=min(n,unpackHalf2x16(N0).x);else n=.0;
#endif
}
#endif
e void Y7(uint l0,c p0,Z0(i)S
#if defined(I)&&!defined(RC)
,W4(r4)o1
#endif
K6 S3){a1 p1=P5(BD,l0);c n=p0;if((p1.x&(Re|K9))!=0u){n=abs(n);
#ifdef XC
if(XC&&(p1.x&K9)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,G0(.0),G0(1.));
#ifdef I
if(I){uint k1=p1.x>>16u;if(k1!=0u){Mb(k1,H9(h0),n);}}
#endif
#ifdef BB
if(BB&&(p1.x&Se)!=0u){f0 U0=h2(J0(RB,l0*A3+2u));g k3=J0(RB,l0*A3+3u);d Te=R0(U0,Z)+k3.xy;E Nb=R7(abs(Te)*k3.zw-k3.zw);c X4=clamp(min(Nb.x,Nb.y)+.5,.0,1.);n=min(n,X4);}
#endif
uint l3=p1.x&0xfu;if(l3<=Ob){S=unpackUnorm4x8(p1.y);
#ifdef I
if(I&&l3==Z7){
#ifndef RC
#ifdef WC
o1.xy=S.zw;o1.z=n;o1.w=1.;
#else
o1=p1.y|packHalf2x16(B2(n,.0));
#endif
#endif
S=C0(.0);}
#endif
}else{f0 U0=h2(J0(RB,l0*A3));g k3=J0(RB,l0*A3+1u);d x4=R0(U0,Z)+k3.xy;float t=l3==M9?x4.x:length(x4);t=clamp(t,.0,1.);float x=t*k3.z+k3.w;float y=uintBitsToFloat(p1.y);S=o2(MD,Pb,d(x,y),.0);}S.w*=n;
#if!defined(Q)&&defined(AB)
N T3;if(AB&&S.w!=.0&&(T3=X1((p1.x>>4)&0xfu))!=Q5){i K1=I0(j0);S.xyz=T4(S.xyz,K1,T3);}
#endif
#if defined(CC)&&(defined(Q)||defined(RC))
S=m3(S);
#endif
S.xyz*=S.w;}
#if!defined(Q)&&!defined(AD)
e void a8(i S S3){
#ifndef WC
if(S.w==.0)return;float L6=1.-S.w;if(L6!=.0)S+=I0(j0)*L6;
#endif
y0(j0,S);}
#endif
#if defined(I)&&!defined(RC)
e void N9(r4 o1 S3){
#ifdef WC
y0(h0,o1);
#else
if(o1!=0u)c1(h0,o1);
#endif
}
#endif
#ifdef Q
#define R5 p2
#define S5 n3
#else
#define R5 L1
#define S5 Z1
#endif
#ifdef KD
R5(JB){
#ifdef HB
r(O,g);
#else
r(O,E);
#endif
r(B0,N);c c8;
#ifdef HB
if(HB&&Qb(O)){c8=y4(O d1);}else if(HB&&Rb(O)){c8=d8(O d1);}else
#endif
{c8=min(min(G0(O.x),abs(G0(O.y))),G0(1.));}i S=C0(.0);
#ifdef I
r4 o1=V7;
#endif
uint e8=Pe(c8);uint Sb=(Tb(B0)<<T5)|e8;uint q2=Y4(v4,Sb);N B1=X1(q2>>T5);B1=X7(B1);if(B1==B0){if(!U5(O)){e8+=q2-max(Sb,q2);e8-=O9;Z4(v4,e8);}}else{c p0=W7(q2&f8);Y7(B1,p0,S
#ifdef I
,o1
#endif
U2 M1);}S.xyz=F2(S.xyz,S.w,Z.xy,m.B3,m.C3);
#ifdef Q
C1=S;
#else
a8(S M1);
#endif
#ifdef I
N9(o1 M1);
#endif
S5}
#endif
#if defined(EB)||defined(FB)
R5(JB){
#ifdef FB
r(D2,d);
#else
r(i1,c);
#endif
r(B0,N);uint q2=V2(v4);N B1=X1(q2>>T5);B1=X7(B1);uint P9;
#ifndef FB
if(B1==B0){P9=q2;}else
#endif
{P9=(Tb(B0)<<T5)+O9;}c n;
#ifdef FB
n=clamp(o2(CD,Q9,D2,.0).x,G0(.0),G0(1.));
#else
n=i1;
#endif
int Ue=int(round(n*I9));W2(v4,P9+uint(Ue));i S=C0(.0);
#ifdef I
r4 o1=V7;
#endif
#ifndef FB
if(B1!=B0)
#endif
{c R9=W7(q2&f8);Y7(B1,R9,S
#ifdef I
,o1
#endif
U2 M1);}S.xyz=F2(S.xyz,S.w,Z.xy,m.B3,m.C3);
#ifdef Q
C1=S;
#else
a8(S M1);
#endif
#ifdef I
N9(o1 M1);
#endif
S5}
#endif
#ifdef KE
R5(JB){r(Y1,d);
#ifdef LD
r(U4,c);
#endif
#ifdef BB
r(M0,g);
#endif
r(H1,c);
#ifdef I
r(x3,N);
#endif
#ifdef AB
r(A1,N);
#endif
i G2=g8(JC,V5,Y1);c W5=1.;
#ifdef LD
W5=min(U4,W5);
#endif
#ifdef BB
if(BB){c X4=h3(a5(M0));W5=clamp(X4,G0(.0),W5);}
#endif
uint q2=V2(v4);N B1=X1(q2>>T5);B1=X7(B1);c R9=W7(q2&f8);i S;
#ifdef I
r4 o1=V7;
#endif
Y7(B1,R9,S
#ifdef I
,o1
#endif
U2 M1);
#ifdef I
if(I&&x3!=0u){r4 N0=Ib(o1)?o1:H9(h0);Mb(x3,N0,W5);}
#endif
#if!defined(Q)&&defined(AB)
if(AB&&A1!=Q5){i K1=I0(j0)*(1.-S.w)+S;G2.xyz=T4(F6(G2),K1,A1)*G2.w;}
#endif
G2*=W5*H1;
#if defined(CC)
G2=m3(G2);
#endif
S=S*(1.-G2.w)+G2;S.xyz=F2(S.xyz,S.w,Z.xy,m.B3,m.C3);
#ifdef Q
C1=S;
#else
a8(S M1);
#endif
#ifdef I
N9(o1 M1);
#endif
W2(v4,O9);S5}
#endif
#ifdef LE
R5(JB){
#ifndef Q
#ifdef ND
if(ND){y0(j0,unpackUnorm4x8(m.Ve));}
#endif
#ifdef OD
if(OD){y0(j0,q1(JC,J));}
#endif
#ifdef EF
i j=I0(j0);y0(j0,j.zyxw);
#endif
#endif
W2(v4,m.We);
#ifdef I
if(I){c1(h0,0u);}
#endif
#ifdef Q
discard;
#endif
S5}
#endif
#ifdef RC
#ifdef AD
p2(JB)
#else
R5(JB)
#endif
{uint q2=V2(v4);c p0=W7(q2&f8);N B1=X1(q2>>T5);B1=X7(B1);i S;Y7(B1,p0,S U2 M1);
#ifdef AD
float L6=1.-S.w;if(L6!=.0)S+=I0(j0)*L6;C1=S;n3
#else
S.xyz=F2(S.xyz,S.w,Z.xy,m.B3,m.C3);
#ifdef Q
C1=S;
#else
a8(S M1);
#endif
S5
#endif
}
#endif
#endif
