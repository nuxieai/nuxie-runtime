#ifdef KD
#ifdef DB
g1(e0)O(0,g,UB);O(1,g,VB);h1
#endif
m2
#ifdef HB
H0 W(0,g,L);
#else
H0 W(0,E,L);
#endif
Q2 W(1,K,B0);g2
#ifdef DB
z1(FC,e0,F,B,A){P(B,F,UB,g);P(B,F,VB,g);
#ifdef HB
U(L,g);
#else
U(L,E);
#endif
U(B0,K);g V;uint l0;d m0;g M;if(q9(UB,VB,A,l0,m0,M v3)){
#ifdef HB
L=M;
#else
L.xy=R7(M.xy);
#endif
B0=X1(l0);V=L3(m0);}else{V=g(m.R2,m.R2,m.R2,m.R2);}c0(L);c0(B0);A1(V);}
#endif
#endif
#if defined(EB)||defined(FB)
#ifdef DB
g1(e0)O(0,M3,KB);h1
#endif
m2
#ifdef FB
H0 W(0,d,D2);
#else
MB W(0,c,i1);
#endif
Q2 W(1,K,B0);g2
#ifdef DB
z1(FC,e0,F,B,A){P(B,F,KB,Q);
#ifdef FB
U(D2,d);
#else
U(i1,c);
#endif
U(B0,K);uint l0;d m0;
#ifdef FB
m0=Hb(KB,l0,D2 v3);
#else
m0=Ib(KB,l0,i1 v3);
#endif
B0=X1(l0);g V=L3(m0);
#ifdef FB
c0(D2);
#else
c0(i1);
#endif
c0(B0);A1(V);}
#endif
#endif
#ifdef LD
#ifdef DB
g1(e0)O(0,g,GC);h1 g1(n1)O(r9,g,WB);O(v9,g,SB);O(w9,g,NB);O(x9,float,XB);O(y9,uint,YB);O(z9,uint,ZB);O(A9,uint,MC);h1
#endif
m2 H0 W(0,d,Y1);H0 W(1,c,V4);
#ifdef BB
H0 W(2,g,M0);
#endif
MB W(3,c,I1);
#ifdef I
Q2 W(4,K,w3);
#endif
#ifdef AB
Q2 W(5,K,B1);
#endif
g2
#ifdef DB
S7(FC,e0,F,n1,i0,B,A){P(B,F,GC,g);P(A,i0,WB,g);P(A,i0,SB,g);P(A,i0,NB,g);P(A,i0,XB,float);P(A,i0,YB,uint);P(A,i0,ZB,uint);P(A,i0,MC,uint);U(Y1,d);U(V4,c);
#ifdef BB
U(M0,g);
#endif
U(I1,c);
#ifdef I
U(w3,K);
#endif
#ifdef AB
U(B1,K);
#endif
bool B9=GC.z==.0||GC.w==.0;V4=B9?.0:1.;d m0=GC.xy;f0 U0=h2(WB);f0 H6=transpose(inverse(U0));if(!B9){float C9=q4*D9(H6[1])/dot(U0[1],H6[1]);if(C9>=.5){m0.x=.5;V4*=W4(.5/C9);}else{m0.x+=C9*GC.z;}float E9=q4*D9(H6[0])/dot(U0[0],H6[0]);if(E9>=.5){m0.y=.5;V4*=W4(.5/E9);}else{m0.y+=E9*GC.w;}}Y1=m0;m0=R0(U0,m0)+NB.xy;if(B9){d N3=R0(H6,GC.zw);N3*=D9(N3)/dot(N3,N3);m0+=q4*N3;}
#ifdef BB
if(BB){M0=T7(h2(SB),NB.zw,m0);}
#endif
I1=XB;
#ifdef I
w3=X1(YB);
#endif
#ifdef AB
B1=X1(ZB);
#endif
g V=L3(m0);c0(Y1);c0(V4);
#ifdef BB
c0(M0);
#endif
c0(I1);
#ifdef I
c0(w3);
#endif
#ifdef AB
c0(B1);
#endif
A1(V);}
#endif
#elif defined(OB)
#ifdef DB
g1(i3)O(0,d,OC);h1 g1(x3)O(1,d,PC);h1 g1(n1)O(r9,g,WB);O(v9,g,SB);O(w9,g,NB);O(x9,float,XB);O(y9,uint,YB);O(z9,uint,ZB);O(A9,uint,MC);h1
#endif
m2 H0 W(0,d,Y1);
#ifdef BB
H0 W(1,g,M0);
#endif
MB W(3,c,I1);
#ifdef I
Q2 W(4,K,w3);
#endif
#ifdef AB
Q2 W(5,K,B1);
#endif
g2
#ifdef DB
I6(FC,i3,j3,x3,y3,n1,i0,B){P(B,j3,OC,d);P(B,y3,PC,d);P(A,i0,WB,g);P(A,i0,SB,g);P(A,i0,NB,g);P(A,i0,XB,float);P(A,i0,YB,uint);P(A,i0,ZB,uint);P(A,i0,MC,uint);U(Y1,d);
#ifdef BB
U(M0,g);
#endif
U(I1,c);
#ifdef I
U(w3,K);
#endif
#ifdef AB
U(B1,K);
#endif
f0 U0=h2(WB);d m0=R0(U0,OC)+NB.xy;Y1=PC;
#ifdef BB
if(BB){M0=T7(h2(SB),NB.zw,m0);}
#endif
I1=XB;
#ifdef I
w3=X1(YB);
#endif
#ifdef AB
B1=X1(ZB);
#endif
g V=L3(m0);c0(Y1);
#ifdef BB
c0(M0);
#endif
c0(I1);
#ifdef I
c0(w3);
#endif
#ifdef AB
c0(B1);
#endif
A1(V);}
#endif
#endif
#ifdef CF
#ifdef DB
g1(e0)h1
#endif
m2 g2
#ifdef DB
z1(FC,e0,F,B,A){Y n2;n2.x=(B&1)==0?m.U7.x:m.U7.z;n2.y=(B&2)==0?m.U7.y:m.U7.w;g V=L3(d(n2));A1(V);}
#endif
#endif
#ifdef JE
#endif
#if defined(KE)&&!defined(N)
#endif
#ifdef GB
J1
#ifndef N
#ifdef LE
#define F9 LE
#else
#define F9 S2
#endif
#ifdef ZC
r4(F9,j0);
#else
x0(F9,j0);
#endif
#endif
#ifdef VC
#define v4 i
#define G9 I0
#define V7 C0(.0)
#define Jb(q) ((q).w!=.0)
#ifdef I
#ifndef QC
x0(T2,g0);
#else
r4(T2,g0);
#endif
#endif
#else
#define v4 uint
#define V7 0u
#define G9 Y0
#define Jb(q) ((q)!=0u)
#ifdef I
j1(T2,g0);
#endif
#endif
E2(J6,w4);K1 O3 N5(Kb,Re,AD);O5(Lb,Se,QB);P3 e uint Te(float x){return uint(round(x*H9+I9));}e c W7(uint x){return W4(float(x)*Mb+(-I9*Mb));}K X7(K l0){
#ifdef DF
l0=min(l0,m.Ue);
#endif
return l0;}
#ifdef I
e void Nb(uint k1,v4 N0,X4(c)n){
#ifdef VC
if(all(lessThan(abs(N0.xy-unpackUnorm4x8(k1).xy),B2(.25/255.))))n=min(n,N0.z);else n=.0;
#else
if(k1==N0>>16)n=min(n,unpackHalf2x16(N0).x);else n=.0;
#endif
}
#endif
e void Y7(uint l0,c p0,Z0(i)R
#if defined(I)&&!defined(QC)
,X4(v4)o1
#endif
K6 Q3){a1 p1=P5(AD,l0);c n=p0;if((p1.x&(Ve|J9))!=0u){n=abs(n);
#ifdef WC
if(WC&&(p1.x&J9)!=0u){n=1.-abs(fract(n*.5)*2.+-1.);}
#endif
}n=clamp(n,G0(.0),G0(1.));
#ifdef I
if(I){uint k1=p1.x>>16u;if(k1!=0u){Nb(k1,G9(g0),n);}}
#endif
#ifdef BB
if(BB&&(p1.x&We)!=0u){f0 U0=h2(J0(QB,l0*z3+2u));g k3=J0(QB,l0*z3+3u);d Xe=R0(U0,a0)+k3.xy;E Ob=R7(abs(Xe)*k3.zw-k3.zw);c Y4=clamp(min(Ob.x,Ob.y)+.5,.0,1.);n=min(n,Y4);}
#endif
uint R3=p1.x&0xfu;if(R3<=Pb){R=unpackUnorm4x8(p1.y);
#ifdef I
if(I&&R3==Z7){
#ifndef QC
#ifdef VC
o1.xy=R.zw;o1.z=n;o1.w=1.;
#else
o1=p1.y|packHalf2x16(B2(n,.0));
#endif
#endif
R=C0(.0);}
#endif
}else{f0 U0=h2(J0(QB,l0*z3));g k3=J0(QB,l0*z3+1u);d L6=R0(U0,a0)+k3.xy;float t=R3==Qb?L6.x:length(L6);t=clamp(t,.0,1.);float x=t*k3.z+k3.w;float y=uintBitsToFloat(p1.y);R=o2(MD,Rb,d(x,y),.0);}R.w*=n;
#if!defined(N)&&defined(AB)
K S3;if(AB&&R.w!=.0&&(S3=X1((p1.x>>4)&0xfu))!=Q5){i L1=I0(j0);R.xyz=U4(R.xyz,L1,S3);}
#endif
#if defined(AC)&&(defined(N)||defined(QC))
R=l3(R);
#endif
R.xyz*=R.w;}
#if!defined(N)&&!defined(ZC)
e void a8(i R Q3){
#ifndef VC
if(R.w==.0)return;float M6=1.-R.w;if(M6!=.0)R+=I0(j0)*M6;
#endif
y0(j0,R);}
#endif
#if defined(I)&&!defined(QC)
e void L9(v4 o1 Q3){
#ifdef VC
y0(g0,o1);
#else
if(o1!=0u)c1(g0,o1);
#endif
}
#endif
#ifdef N
#define R5 p2
#define S5 m3
#else
#define R5 M1
#define S5 Z1
#endif
#ifdef KD
R5(IB){
#ifdef HB
r(L,g);
#else
r(L,E);
#endif
r(B0,K);c c8;
#ifdef HB
if(HB&&Sb(L)){c8=y4(L d1);}else if(HB&&Tb(L)){c8=d8(L d1);}else
#endif
{c8=min(min(G0(L.x),abs(G0(L.y))),G0(1.));}i R=C0(.0);
#ifdef I
v4 o1=V7;
#endif
uint e8=Te(c8);uint Ub=(Vb(B0)<<T5)|e8;uint q2=Z4(w4,Ub);K C1=X1(q2>>T5);C1=X7(C1);if(C1==B0){if(!U5(L)){e8+=q2-max(Ub,q2);e8-=M9;a5(w4,e8);}}else{c p0=W7(q2&f8);Y7(C1,p0,R
#ifdef I
,o1
#endif
U2 N1);}R.xyz=F2(R.xyz,R.w,a0.xy,m.A3,m.B3);
#ifdef N
D1=R;
#else
a8(R N1);
#endif
#ifdef I
L9(o1 N1);
#endif
S5}
#endif
#if defined(EB)||defined(FB)
R5(IB){
#ifdef FB
r(D2,d);
#else
r(i1,c);
#endif
r(B0,K);uint q2=V2(w4);K C1=X1(q2>>T5);C1=X7(C1);uint N9;
#ifndef FB
if(C1==B0){N9=q2;}else
#endif
{N9=(Vb(B0)<<T5)+M9;}c n;
#ifdef FB
n=clamp(o2(BD,O9,D2,.0).x,G0(.0),G0(1.));
#else
n=i1;
#endif
int Ye=int(round(n*H9));W2(w4,N9+uint(Ye));i R=C0(.0);
#ifdef I
v4 o1=V7;
#endif
#ifndef FB
if(C1!=B0)
#endif
{c P9=W7(q2&f8);Y7(C1,P9,R
#ifdef I
,o1
#endif
U2 N1);}R.xyz=F2(R.xyz,R.w,a0.xy,m.A3,m.B3);
#ifdef N
D1=R;
#else
a8(R N1);
#endif
#ifdef I
L9(o1 N1);
#endif
S5}
#endif
#ifdef JE
R5(IB){r(Y1,d);
#ifdef LD
r(V4,c);
#endif
#ifdef BB
r(M0,g);
#endif
r(I1,c);
#ifdef I
r(w3,K);
#endif
#ifdef AB
r(B1,K);
#endif
i G2=g8(HC,V5,Y1);c W5=1.;
#ifdef LD
W5=min(V4,W5);
#endif
#ifdef BB
if(BB){c Y4=h3(c5(M0));W5=clamp(Y4,G0(.0),W5);}
#endif
uint q2=V2(w4);K C1=X1(q2>>T5);C1=X7(C1);c P9=W7(q2&f8);i R;
#ifdef I
v4 o1=V7;
#endif
Y7(C1,P9,R
#ifdef I
,o1
#endif
U2 N1);
#ifdef I
if(I&&w3!=0u){v4 N0=Jb(o1)?o1:G9(g0);Nb(w3,N0,W5);}
#endif
#if!defined(N)&&defined(AB)
if(AB&&B1!=Q5){i L1=I0(j0)*(1.-R.w)+R;G2.xyz=U4(F6(G2),L1,B1)*G2.w;}
#endif
G2*=W5*I1;
#if defined(AC)
G2=l3(G2);
#endif
R=R*(1.-G2.w)+G2;R.xyz=F2(R.xyz,R.w,a0.xy,m.A3,m.B3);
#ifdef N
D1=R;
#else
a8(R N1);
#endif
#ifdef I
L9(o1 N1);
#endif
W2(w4,M9);S5}
#endif
#ifdef KE
R5(IB){
#ifndef N
#ifdef ND
if(ND){y0(j0,unpackUnorm4x8(m.Ze));}
#endif
#ifdef OD
if(OD){y0(j0,q1(HC,G));}
#endif
#ifdef EF
i j=I0(j0);y0(j0,j.zyxw);
#endif
#endif
W2(w4,m.af);
#ifdef I
if(I){c1(g0,0u);}
#endif
#ifdef N
discard;
#endif
S5}
#endif
#ifdef QC
#ifdef ZC
p2(IB)
#else
R5(IB)
#endif
{uint q2=V2(w4);c p0=W7(q2&f8);K C1=X1(q2>>T5);C1=X7(C1);i R;Y7(C1,p0,R U2 N1);
#ifdef ZC
float M6=1.-R.w;if(M6!=.0)R+=I0(j0)*M6;D1=R;m3
#else
R.xyz=F2(R.xyz,R.w,a0.xy,m.A3,m.B3);
#ifdef N
D1=R;
#else
a8(R N1);
#endif
S5
#endif
}
#endif
#endif
