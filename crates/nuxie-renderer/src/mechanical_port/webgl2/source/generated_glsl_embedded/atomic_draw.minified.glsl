#ifdef OD
#ifdef DB
f1(f0)J(0,f,UB);J(1,f,VB);g1
#endif
p2
#ifdef HB
H0 V(0,f,M);
#else
H0 V(0,E,M);
#endif
S2 V(1,L,C0);h2
#ifdef DB
y1(FC,f0,F,B,v){K(B,F,UB,f);K(B,F,VB,f);
#ifdef HB
T(M,f);
#else
T(M,E);
#endif
T(C0,L);f W;uint m0;c j0;f N;if(v9(UB,VB,v,m0,j0,N w3)){
#ifdef HB
M=N;
#else
M.xy=S7(N.xy);
#endif
C0=Y1(m0);W=M3(j0);}else{W=f(n.T2,n.T2,n.T2,n.T2);}a0(M);a0(C0);z1(W);}
#endif
#endif
#if defined(EB)||defined(GB)
#ifdef DB
f1(f0)J(0,N3,KB);g1
#endif
p2
#ifdef GB
H0 V(0,c,F2);
#else
MB V(0,d,h1);
#endif
S2 V(1,L,C0);h2
#ifdef DB
y1(FC,f0,F,B,v){K(B,F,KB,Q);
#ifdef GB
T(F2,c);
#else
T(h1,d);
#endif
T(C0,L);uint m0;c j0;
#ifdef GB
j0=Kb(KB,m0,F2 w3);
#else
j0=Lb(KB,m0,h1 w3);
#endif
C0=Y1(m0);f W=M3(j0);
#ifdef GB
a0(F2);
#else
a0(h1);
#endif
a0(C0);z1(W);}
#endif
#endif
#ifdef AD
#ifdef DB
f1(f0)J(0,f,GC);g1 f1(m1)J(w9,f,WB);J(x9,f,SB);J(y9,f,NB);J(z9,uint,XB);J(A9,uint,YB);J(B9,uint,ZB);J(C9,uint,MC);J(af,f,PD);J(bf,f,QD);J(cf,f,BD);J(Mb,f,OC);g1
#endif
p2 H0 V(0,c,Z1);H0 V(1,d,V4);H0 V(2,f,M5);
#ifdef BB
H0 V(3,f,M0);
#endif
MB V(4,i,H1);
#ifdef I
S2 V(5,L,x3);
#endif
#ifdef AB
S2 V(6,L,A1);
#endif
h2
#ifdef DB
T7(FC,f0,F,m1,g0,B,v){K(B,F,GC,f);K(v,g0,WB,f);K(v,g0,SB,f);K(v,g0,NB,f);K(v,g0,XB,uint);K(v,g0,YB,uint);K(v,g0,ZB,uint);K(v,g0,MC,uint);K(v,g0,PD,f);K(v,g0,QD,f);K(v,g0,BD,f);K(v,g0,OC,f);T(Z1,c);T(V4,d);T(M5,f);
#ifdef BB
T(M0,f);
#endif
T(H1,i);
#ifdef I
T(x3,L);
#endif
#ifdef AB
T(A1,L);
#endif
bool D9=GC.z==.0||GC.w==.0;V4=D9?.0:1.;c j0=GC.xy;d0 U0=I1(WB);d0 I6=transpose(inverse(U0));if(!D9){float E9=r4*F9(I6[1])/dot(U0[1],I6[1]);if(E9>=.5){j0.x=.5;V4*=W4(.5/E9);}else{j0.x+=E9*GC.z;}float G9=r4*F9(I6[0])/dot(U0[0],I6[0]);if(G9>=.5){j0.y=.5;V4*=W4(.5/G9);}else{j0.y+=G9*GC.w;}}d0 df=I1(PD);Z1=N0(df,j0)+BD.xy;j0=N0(U0,j0)+NB.xy;if(D9){c O3=N0(I6,GC.zw);O3*=F9(O3)/dot(O3,O3);j0+=r4*O3;}
#ifdef BB
if(BB){M0=U7(I1(SB),NB.zw,j0);}
#endif
H1=unpackUnorm4x8(XB);
#ifdef I
x3=Y1(YB);
#endif
#ifdef AB
A1=Y1(ZB);
#endif
f W=M3(j0);c q0=j0;
#ifdef NE
q0.y=float(n.Nb)-q0.y;
#endif
if(OC.w!=0.0){d0 ef=I1(QD);c ff=BD.zw;M5=Ob(q0,ef,ff,OC.w,OC.xy,OC.z);}a0(Z1);a0(V4);a0(M5);
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
#elif defined(OB)
#ifdef DB
f1(j3)J(0,c,PC);g1 f1(y3)J(1,c,QC);g1 f1(m1)J(w9,f,WB);J(x9,f,SB);J(y9,f,NB);J(z9,uint,XB);J(A9,uint,YB);J(B9,uint,ZB);J(C9,uint,MC);g1
#endif
p2 H0 V(0,c,Z1);
#ifdef BB
H0 V(1,f,M0);
#endif
MB V(3,i,H1);
#ifdef I
S2 V(4,L,x3);
#endif
#ifdef AB
S2 V(5,L,A1);
#endif
h2
#ifdef DB
J6(FC,j3,k3,y3,z3,m1,g0,B){K(B,k3,PC,c);K(B,z3,QC,c);K(v,g0,WB,f);K(v,g0,SB,f);K(v,g0,NB,f);K(v,g0,XB,uint);K(v,g0,YB,uint);K(v,g0,ZB,uint);K(v,g0,MC,uint);T(Z1,c);
#ifdef BB
T(M0,f);
#endif
T(H1,i);
#ifdef I
T(x3,L);
#endif
#ifdef AB
T(A1,L);
#endif
d0 U0=I1(WB);c j0=N0(U0,PC)+NB.xy;Z1=QC;
#ifdef BB
if(BB){M0=U7(I1(SB),NB.zw,j0);}
#endif
H1=unpackUnorm4x8(XB);
#ifdef I
x3=Y1(YB);
#endif
#ifdef AB
A1=Y1(ZB);
#endif
f W=M3(j0);a0(Z1);
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
#ifdef HF
#ifdef DB
f1(f0)g1
#endif
p2 h2
#ifdef DB
y1(FC,f0,F,B,v){Y q2;q2.x=(B&1)==0?n.V7.x:n.V7.z;q2.y=(B&2)==0?n.V7.y:n.V7.w;f W=M3(c(q2));z1(W);}
#endif
#endif
#ifdef OE
#endif
#if defined(PE)&&!defined(O)
#endif
#ifdef FB
J1
#ifndef O
#ifdef QE
#define H9 QE
#else
#define H9 U2
#endif
#ifdef CD
v4(H9,k0);
#else
y0(H9,k0);
#endif
#endif
#ifdef WC
#define w4 i
#define I9 J0
#define W7 D0(.0)
#define Pb(q) ((q).w!=.0)
#ifdef I
#ifndef RC
y0(V2,h0);
#else
v4(V2,h0);
#endif
#endif
#else
#define w4 uint
#define W7 0u
#define I9 Y0
#define Pb(q) ((q)!=0u)
#ifdef I
i1(V2,h0);
#endif
#endif
G2(K6,x4);K1 P3 N5(Qb,hf,DD);O5(Rb,jf,QB);Q3 e uint kf(float x){return uint(round(x*J9+K9));}e d X7(uint x){return W4(float(x)*Sb+(-K9*Sb));}L Y7(L m0){
#ifdef IF
m0=min(m0,n.lf);
#endif
return m0;}
#ifdef I
e void Tb(uint j1,w4 O0,X4(d)o){
#ifdef WC
if(all(lessThan(abs(O0.xy-unpackUnorm4x8(j1).xy),D2(.25/255.))))o=min(o,O0.z);else o=.0;
#else
if(j1==O0>>16)o=min(o,unpackHalf2x16(O0).x);else o=.0;
#endif
}
#endif
e void Z7(uint m0,d p0,Z0(i)R
#if defined(I)&&!defined(RC)
,X4(w4)n1
#endif
L6 R3){a1 o1=Q5(DD,m0);d o=p0;if((o1.x&(mf|L9))!=0u){o=abs(o);
#ifdef XC
if(XC&&(o1.x&L9)!=0u){o=1.-abs(fract(o*.5)*2.+-1.);}
#endif
}o=clamp(o,I0(.0),I0(1.));
#ifdef I
if(I){uint j1=o1.x>>16u;if(j1!=0u){Tb(j1,I9(h0),o);}}
#endif
#ifdef BB
if(BB&&(o1.x&nf)!=0u){d0 U0=I1(K0(QB,m0*A3+2u));f H2=K0(QB,m0*A3+3u);c of=N0(U0,c0)+H2.xy;E Ub=S7(abs(of)*H2.zw-H2.zw);d Y4=clamp(min(Ub.x,Ub.y)+.5,.0,1.);o=min(o,Y4);}
#endif
uint S3=o1.x&0xfu;if(S3<=Vb){R=unpackUnorm4x8(o1.y);
#ifdef I
if(I&&S3==a8){
#ifndef RC
#ifdef WC
n1.xy=R.zw;n1.z=o;n1.w=1.;
#else
n1=o1.y|packHalf2x16(D2(o,.0));
#endif
#endif
R=D0(.0);}
#endif
}else{d0 U0=I1(K0(QB,m0*A3));f H2=K0(QB,m0*A3+1u);c Wb=N0(U0,c0)+H2.xy;float t=S3==Xb?Wb.x:length(Wb);t=clamp(t,.0,1.);float x=t*H2.z+H2.w;float y=uintBitsToFloat(o1.y);R=i2(ED,N9,c(x,y),.0);}R.w*=o;
#if!defined(O)&&defined(AB)
L T3;if(AB&&R.w!=.0&&(T3=Y1((o1.x>>4)&0xfu))!=R5){i L1=J0(k0);R.xyz=U4(R.xyz,L1,T3);}
#endif
#if defined(AC)&&(defined(O)||defined(RC))
R=l3(R);
#endif
R.xyz*=R.w;}
#if!defined(O)&&!defined(CD)
e void c8(i R R3){
#ifndef WC
if(R.w==.0)return;float M6=1.-R.w;if(M6!=.0)R+=J0(k0)*M6;
#endif
z0(k0,R);}
#endif
#if defined(I)&&!defined(RC)
e void O9(w4 n1 R3){
#ifdef WC
z0(h0,n1);
#else
if(n1!=0u)c1(h0,n1);
#endif
}
#endif
#ifdef O
#define S5 r2
#define T5 m3
#else
#define S5 M1
#define T5 a2
#endif
#ifdef OD
S5(IB){
#ifdef HB
r(M,f);
#else
r(M,E);
#endif
r(C0,L);d d8;
#ifdef HB
if(HB&&Yb(M)){d8=y4(M d1);}else if(HB&&Zb(M)){d8=e8(M d1);}else
#endif
{d8=min(min(I0(M.x),abs(I0(M.y))),I0(1.));}i R=D0(.0);
#ifdef I
w4 n1=W7;
#endif
uint f8=kf(d8);uint ac=(bc(C0)<<U5)|f8;uint v2=Z4(x4,ac);L B1=Y1(v2>>U5);B1=Y7(B1);if(B1==C0){if(!V5(M)){f8+=v2-max(ac,v2);f8-=P9;a5(x4,f8);}}else{d p0=X7(v2&g8);Z7(B1,p0,R
#ifdef I
,n1
#endif
W2 N1);}R.xyz=I2(R.xyz,R.w,c0.xy,n.B3,n.C3);
#ifdef O
C1=R;
#else
c8(R N1);
#endif
#ifdef I
O9(n1 N1);
#endif
T5}
#endif
#if defined(EB)||defined(GB)
S5(IB){
#ifdef GB
r(F2,c);
#else
r(h1,d);
#endif
r(C0,L);uint v2=X2(x4);L B1=Y1(v2>>U5);B1=Y7(B1);uint Q9;
#ifndef GB
if(B1==C0){Q9=v2;}else
#endif
{Q9=(bc(C0)<<U5)+P9;}d o;
#ifdef GB
o=clamp(i2(FD,R9,F2,.0).x,I0(.0),I0(1.));
#else
o=h1;
#endif
int pf=int(round(o*J9));Y2(x4,Q9+uint(pf));i R=D0(.0);
#ifdef I
w4 n1=W7;
#endif
#ifndef GB
if(B1!=C0)
#endif
{d S9=X7(v2&g8);Z7(B1,S9,R
#ifdef I
,n1
#endif
W2 N1);}R.xyz=I2(R.xyz,R.w,c0.xy,n.B3,n.C3);
#ifdef O
C1=R;
#else
c8(R N1);
#endif
#ifdef I
O9(n1 N1);
#endif
T5}
#endif
#ifdef OE
S5(IB){r(Z1,c);
#ifdef AD
r(V4,d);r(M5,f);
#endif
#ifdef BB
r(M0,f);
#endif
r(H1,i);
#ifdef I
r(x3,L);
#endif
#ifdef AB
r(A1,L);
#endif
i j2=h8(HC,W5,Z1);d X5=1.;
#ifdef AD
X5=min(V4,X5);
#endif
#ifdef BB
if(BB){d Y4=i3(c5(M0));X5=clamp(Y4,I0(.0),X5);}
#endif
uint v2=X2(x4);L B1=Y1(v2>>U5);B1=Y7(B1);d S9=X7(v2&g8);i R;
#ifdef I
w4 n1=W7;
#endif
Z7(B1,S9,R
#ifdef I
,n1
#endif
W2 N1);
#ifdef I
if(I&&x3!=0u){w4 O0=Pb(n1)?n1:I9(h0);Tb(x3,O0,X5);}
#endif
#ifdef AD
if(M5.w!=0.0){c T9=cc(M5);i U9=i2(ED,N9,T9,0.0);U9.xyz*=U9.w;j2*=U9;}
#endif
j2*=H1;
#if!defined(O)&&defined(AB)
if(AB&&A1!=R5){i L1=J0(k0)*(1.-R.w)+R;j2.xyz=U4(G6(j2),L1,A1)*j2.w;}
#endif
j2*=X5;
#if defined(AC)
j2=l3(j2);
#endif
R=R*(1.-j2.w)+j2;R.xyz=I2(R.xyz,R.w,c0.xy,n.B3,n.C3);
#ifdef O
C1=R;
#else
c8(R N1);
#endif
#ifdef I
O9(n1 N1);
#endif
Y2(x4,P9);T5}
#endif
#ifdef PE
S5(IB){
#ifndef O
#ifdef RD
if(RD){z0(k0,unpackUnorm4x8(n.qf));}
#endif
#ifdef SD
if(SD){z0(k0,p1(HC,G));}
#endif
#ifdef JF
i j=J0(k0);z0(k0,j.zyxw);
#endif
#endif
Y2(x4,n.rf);
#ifdef I
if(I){c1(h0,0u);}
#endif
#ifdef O
discard;
#endif
T5}
#endif
#ifdef RC
#ifdef CD
r2(IB)
#else
S5(IB)
#endif
{uint v2=X2(x4);d p0=X7(v2&g8);L B1=Y1(v2>>U5);B1=Y7(B1);i R;Z7(B1,p0,R W2 N1);
#ifdef CD
float M6=1.-R.w;if(M6!=.0)R+=J0(k0)*M6;C1=R;m3
#else
R.xyz=I2(R.xyz,R.w,c0.xy,n.B3,n.C3);
#ifdef O
C1=R;
#else
c8(R N1);
#endif
T5
#endif
}
#endif
#endif
