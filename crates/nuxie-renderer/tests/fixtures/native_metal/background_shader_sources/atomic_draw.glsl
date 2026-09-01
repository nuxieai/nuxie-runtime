#ifdef KD
#ifdef DB
g1(e0)L(0,g,VB);L(1,g,WB);h1
#endif
l2
#ifdef HB
H0 X(0,g,O);
#else
H0 X(0,E,O);
#endif
Q2 X(1,N,B0);f2
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
O.xy=Q7(P.xy);
#endif
B0=W1(l0);W=M3(m0);}else{W=g(m.R2,m.R2,m.R2,m.R2);}a0(O);a0(B0);z1(W);}
#endif
#endif
#if defined(EB)||defined(FB)
#ifdef DB
g1(e0)L(0,N3,LB);h1
#endif
l2
#ifdef FB
H0 X(0,d,D2);
#else
NB X(0,c,i1);
#endif
Q2 X(1,N,B0);f2
#ifdef DB
y1(HC,e0,F,B,v){M(B,F,LB,R);
#ifdef FB
V(D2,d);
#else
V(i1,c);
#endif
V(B0,N);uint l0;d m0;
#ifdef FB
m0=Fb(LB,l0,D2 w3);
#else
m0=Gb(LB,l0,i1 w3);
#endif
B0=W1(l0);g W=M3(m0);
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
l2 H0 X(0,d,X1);H0 X(1,c,T4);
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
f2
#ifdef DB
R7(HC,e0,F,n1,g0,B,v){M(B,F,IC,g);M(v,g0,XB,g);M(v,g0,SB,g);M(v,g0,OB,g);
#ifdef O3
M(v,g0,YB,uint);M(v,g0,ZB,uint);M(v,g0,AC,uint);M(v,g0,BC,uint);G IB=G(YB,ZB,AC,BC);
#else
M(v,g0,IB,G);
#endif
V(X1,d);V(T4,c);
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
bool C9=IC.z==.0||IC.w==.0;T4=C9?.0:1.;d m0=IC.xy;f0 U0=g2(XB);f0 G6=transpose(inverse(U0));if(!C9){float D9=p4*E9(G6[1])/dot(U0[1],G6[1]);if(D9>=.5){m0.x=.5;T4*=U4(.5/D9);}else{m0.x+=D9*IC.z;}float F9=p4*E9(G6[0])/dot(U0[0],G6[0]);if(F9>=.5){m0.y=.5;T4*=U4(.5/F9);}else{m0.y+=F9*IC.w;}}X1=m0;m0=R0(U0,m0)+OB.xy;if(C9){d P3=R0(G6,IC.zw);P3*=E9(P3)/dot(P3,P3);m0+=p4*P3;}
#ifdef BB
if(BB){M0=S7(g2(SB),OB.zw,m0);}
#endif
H1=uintBitsToFloat(IB.x);
#ifdef I
x3=W1(IB.y);
#endif
#ifdef AB
A1=W1(IB.z);
#endif
g W=M3(m0);a0(X1);a0(T4);
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
l2 H0 X(0,d,X1);
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
f2
#ifdef DB
H6(HC,i3,j3,y3,z3,n1,g0,B){M(B,j3,PC,d);M(B,z3,QC,d);M(v,g0,XB,g);M(v,g0,SB,g);M(v,g0,OB,g);
#ifdef O3
M(v,g0,YB,uint);M(v,g0,ZB,uint);M(v,g0,AC,uint);M(v,g0,BC,uint);G IB=G(YB,ZB,AC,BC);
#else
M(v,g0,IB,G);
#endif
V(X1,d);
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
f0 U0=g2(XB);d m0=R0(U0,PC)+OB.xy;X1=QC;
#ifdef BB
if(BB){M0=S7(g2(SB),OB.zw,m0);}
#endif
H1=uintBitsToFloat(IB.x);
#ifdef I
x3=W1(IB.y);
#endif
#ifdef AB
A1=W1(IB.z);
#endif
g W=M3(m0);a0(X1);
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
l2 f2
#ifdef DB
y1(HC,e0,F,B,v){Y m2;m2.x=(B&1)==0?m.T7.x:m.T7.z;m2.y=(B&2)==0?m.T7.y:m.T7.w;g W=M3(d(m2));z1(W);}
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
w0(G9,j0);
#endif
#endif
#ifdef WC
#define r4 i
#define H9 I0
#define U7 C0(.0)
#define Hb(q) ((q).w!=.0)
#ifdef I
#ifndef RC
w0(T2,h0);
#else
q4(T2,h0);
#endif
#endif
#else
#define r4 uint
#define U7 0u
#define H9 Y0
#define Hb(q) ((q)!=0u)
#ifdef I
j1(T2,h0);
#endif
#endif
E2(I6,v4);J1 Q3 M5(Ib,Me,BD);N5(Jb,Ne,RB);R3 e uint Oe(float x){return uint(round(x*I9+J9));}e c V7(uint x){return U4(float(x)*Kb+(-J9*Kb));}N W7(N l0){
#ifdef DF
l0=min(l0,m.Pe);
#endif
return l0;}
#ifdef I
e void Lb(uint k1,r4 N0,V4(c)n){
#ifdef WC
if(all(lessThan(abs(N0.xy-unpackUnorm4x8(k1).xy),B2(.25/255.))))n=min(n,N0.z);else n=.0;
#else
if(k1==N0>>16)n=min(n,unpackHalf2x16(N0).x);else n=.0;
#endif
}
#endif
e void X7(uint l0,c p0,Z0(i)S
#if defined(I)&&!defined(RC)
,V4(r4)o1
#endif
J6 S3){a1 p1=O5(BD,l0);c n=p0;if((p1.x&(Qe|K9))!=0u){n=abs(n);
#ifdef XC
if(XC&&(p1.x&K9)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,G0(.0),G0(1.));
#ifdef I
if(I){uint k1=p1.x>>16u;if(k1!=0u){Lb(k1,H9(h0),n);}}
#endif
#ifdef BB
if(BB&&(p1.x&Re)!=0u){f0 U0=g2(J0(RB,l0*A3+2u));g k3=J0(RB,l0*A3+3u);d Se=R0(U0,Z)+k3.xy;E Mb=Q7(abs(Se)*k3.zw-k3.zw);c W4=clamp(min(Mb.x,Mb.y)+.5,.0,1.);n=min(n,W4);}
#endif
uint l3=p1.x&0xfu;if(l3<=Nb){S=unpackUnorm4x8(p1.y);
#ifdef I
if(I&&l3==Y7){
#ifndef RC
#ifdef WC
o1.xy=S.zw;o1.z=n;o1.w=1.;
#else
o1=p1.y|packHalf2x16(B2(n,.0));
#endif
#endif
S=C0(.0);}
#endif
}else{f0 U0=g2(J0(RB,l0*A3));g k3=J0(RB,l0*A3+1u);d x4=R0(U0,Z)+k3.xy;float t=l3==M9?x4.x:length(x4);t=clamp(t,.0,1.);float x=t*k3.z+k3.w;float y=uintBitsToFloat(p1.y);S=n2(MD,Ob,d(x,y),.0);}S.w*=n;
#if!defined(Q)&&defined(AB)
N T3;if(AB&&S.w!=.0&&(T3=W1((p1.x>>4)&0xfu))!=P5){i K1=I0(j0);S.xyz=S4(S.xyz,K1,T3);}
#endif
#if defined(CC)&&(defined(Q)||defined(RC))
S=m3(S);
#endif
S.xyz*=S.w;}
#if!defined(Q)&&!defined(AD)
e void Z7(i S S3){
#ifndef WC
if(S.w==.0)return;float K6=1.-S.w;if(K6!=.0)S+=I0(j0)*K6;
#endif
x0(j0,S);}
#endif
#if defined(I)&&!defined(RC)
e void N9(r4 o1 S3){
#ifdef WC
x0(h0,o1);
#else
if(o1!=0u)c1(h0,o1);
#endif
}
#endif
#ifdef Q
#define Q5 o2
#define R5 n3
#else
#define Q5 L1
#define R5 Y1
#endif
#ifdef KD
Q5(JB){
#ifdef HB
r(O,g);
#else
r(O,E);
#endif
r(B0,N);c a8;
#ifdef HB
if(HB&&Pb(O)){a8=y4(O d1);}else if(HB&&Qb(O)){a8=c8(O d1);}else
#endif
{a8=min(min(G0(O.x),abs(G0(O.y))),G0(1.));}i S=C0(.0);
#ifdef I
r4 o1=U7;
#endif
uint d8=Oe(a8);uint Rb=(Sb(B0)<<S5)|d8;uint p2=X4(v4,Rb);N B1=W1(p2>>S5);B1=W7(B1);if(B1==B0){if(!T5(O)){d8+=p2-max(Rb,p2);d8-=O9;Y4(v4,d8);}}else{c p0=V7(p2&e8);X7(B1,p0,S
#ifdef I
,o1
#endif
U2 M1);}S.xyz=F2(S.xyz,S.w,Z.xy,m.B3,m.C3);
#ifdef Q
C1=S;
#else
Z7(S M1);
#endif
#ifdef I
N9(o1 M1);
#endif
R5}
#endif
#if defined(EB)||defined(FB)
Q5(JB){
#ifdef FB
r(D2,d);
#else
r(i1,c);
#endif
r(B0,N);uint p2=V2(v4);N B1=W1(p2>>S5);B1=W7(B1);uint P9;
#ifndef FB
if(B1==B0){P9=p2;}else
#endif
{P9=(Sb(B0)<<S5)+O9;}c n;
#ifdef FB
n=clamp(n2(CD,Q9,D2,.0).x,G0(.0),G0(1.));
#else
n=i1;
#endif
int Te=int(round(n*I9));W2(v4,P9+uint(Te));i S=C0(.0);
#ifdef I
r4 o1=U7;
#endif
#ifndef FB
if(B1!=B0)
#endif
{c R9=V7(p2&e8);X7(B1,R9,S
#ifdef I
,o1
#endif
U2 M1);}S.xyz=F2(S.xyz,S.w,Z.xy,m.B3,m.C3);
#ifdef Q
C1=S;
#else
Z7(S M1);
#endif
#ifdef I
N9(o1 M1);
#endif
R5}
#endif
#ifdef KE
Q5(JB){r(X1,d);
#ifdef LD
r(T4,c);
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
i G2=f8(JC,U5,X1);c V5=1.;
#ifdef LD
V5=min(T4,V5);
#endif
#ifdef BB
if(BB){c W4=h3(Z4(M0));V5=clamp(W4,G0(.0),V5);}
#endif
uint p2=V2(v4);N B1=W1(p2>>S5);B1=W7(B1);c R9=V7(p2&e8);i S;
#ifdef I
r4 o1=U7;
#endif
X7(B1,R9,S
#ifdef I
,o1
#endif
U2 M1);
#ifdef I
if(I&&x3!=0u){r4 N0=Hb(o1)?o1:H9(h0);Lb(x3,N0,V5);}
#endif
#if!defined(Q)&&defined(AB)
if(AB&&A1!=P5){i K1=I0(j0)*(1.-S.w)+S;G2.xyz=S4(E6(G2),K1,A1)*G2.w;}
#endif
G2*=V5*H1;
#if defined(CC)
G2=m3(G2);
#endif
S=S*(1.-G2.w)+G2;S.xyz=F2(S.xyz,S.w,Z.xy,m.B3,m.C3);
#ifdef Q
C1=S;
#else
Z7(S M1);
#endif
#ifdef I
N9(o1 M1);
#endif
W2(v4,O9);R5}
#endif
#ifdef LE
Q5(JB){
#ifndef Q
#ifdef ND
if(ND){x0(j0,unpackUnorm4x8(m.Ue));}
#endif
#ifdef OD
if(OD){x0(j0,q1(JC,J));}
#endif
#ifdef EF
i j=I0(j0);x0(j0,j.zyxw);
#endif
#endif
W2(v4,m.Ve);
#ifdef I
if(I){c1(h0,0u);}
#endif
#ifdef Q
discard;
#endif
R5}
#endif
#ifdef RC
#ifdef AD
o2(JB)
#else
Q5(JB)
#endif
{uint p2=V2(v4);c p0=V7(p2&e8);N B1=W1(p2>>S5);B1=W7(B1);i S;X7(B1,p0,S U2 M1);
#ifdef AD
float K6=1.-S.w;if(K6!=.0)S+=I0(j0)*K6;C1=S;n3
#else
S.xyz=F2(S.xyz,S.w,Z.xy,m.B3,m.C3);
#ifdef Q
C1=S;
#else
Z7(S M1);
#endif
R5
#endif
}
#endif
#endif
