#ifdef ND
#ifdef CB
h1(g0)J(0,f,TB);J(1,f,UB);i1
#endif
q2
#ifdef GB
I0 W(0,f,M);
#else
I0 W(0,E,M);
#endif
T2 W(1,L,D0);i2
#ifdef CB
A1(EC,g0,F,B,v){K(B,F,TB,f);K(B,F,UB,f);
#ifdef GB
U(M,f);
#else
U(M,E);
#endif
U(D0,L);f X;uint n0;c k0;f N;if(x9(TB,UB,v,n0,k0,N A3)){
#ifdef GB
M=N;
#else
M.xy=U7(N.xy);
#endif
D0=a2(n0);X=Q3(k0);}else{X=f(j.U2,j.U2,j.U2,j.U2);}c0(M);c0(D0);B1(X);}
#endif
#endif
#if defined(DB)||defined(FB)
#ifdef CB
h1(g0)J(0,R3,JB);i1
#endif
q2
#ifdef FB
I0 W(0,c,F2);
#else
MB W(0,d,j1);
#endif
T2 W(1,L,D0);i2
#ifdef CB
A1(EC,g0,F,B,v){K(B,F,JB,R);
#ifdef FB
U(F2,c);
#else
U(j1,d);
#endif
U(D0,L);uint n0;c k0;
#ifdef FB
k0=Mb(JB,n0,F2 A3);
#else
k0=Nb(JB,n0,j1 A3);
#endif
D0=a2(n0);f X=Q3(k0);
#ifdef FB
c0(F2);
#else
c0(j1);
#endif
c0(D0);B1(X);}
#endif
#endif
#ifdef ZC
#ifdef CB
h1(g0)J(0,f,FC);i1 h1(o1)J(y9,f,VB);J(z9,f,RB);J(A9,f,NB);J(B9,uint,WB);J(C9,uint,XB);J(D9,uint,YB);J(E9,uint,LC);J(df,f,OD);J(ef,f,PD);J(ff,f,AD);J(Ob,f,NC);i1
#endif
q2 I0 W(0,c,c2);I0 W(1,d,Y4);I0 W(2,f,P5);
#ifdef AB
I0 W(3,f,N0);
#endif
MB W(4,i,J1);
#ifdef I
T2 W(5,L,B3);
#endif
#ifdef S
T2 W(6,L,C1);
#endif
i2
#ifdef CB
V7(EC,g0,F,o1,h0,B,v){K(B,F,FC,f);K(v,h0,VB,f);K(v,h0,RB,f);K(v,h0,NB,f);K(v,h0,WB,uint);K(v,h0,XB,uint);K(v,h0,YB,uint);K(v,h0,LC,uint);K(v,h0,OD,f);K(v,h0,PD,f);K(v,h0,AD,f);K(v,h0,NC,f);U(c2,c);U(Y4,d);U(P5,f);
#ifdef AB
U(N0,f);
#endif
U(J1,i);
#ifdef I
U(B3,L);
#endif
#ifdef S
U(C1,L);
#endif
bool F9=FC.z==.0||FC.w==.0;Y4=F9?.0:1.;c k0=FC.xy;e0 V0=K1(VB);e0 J6=transpose(inverse(V0));if(!F9){float G9=w4*H9(J6[1])/dot(V0[1],J6[1]);if(G9>=.5){k0.x=.5;Y4*=Z4(.5/G9);}else{k0.x+=G9*FC.z;}float I9=w4*H9(J6[0])/dot(V0[0],J6[0]);if(I9>=.5){k0.y=.5;Y4*=Z4(.5/I9);}else{k0.y+=I9*FC.w;}}e0 gf=K1(OD);c2=O0(gf,k0)+AD.xy;k0=O0(V0,k0)+NB.xy;if(F9){c S3=O0(J6,FC.zw);S3*=H9(S3)/dot(S3,S3);k0+=w4*S3;}
#ifdef AB
if(AB){N0=W7(K1(RB),NB.zw,k0);}
#endif
J1=unpackUnorm4x8(WB);
#ifdef I
B3=a2(XB);
#endif
#ifdef S
C1=a2(YB);
#endif
f X=Q3(k0);c v0=k0;
#ifdef ME
if(j.Pb!=0u){v0.y=float(j.Qb)-v0.y;}
#endif
if(NC.w!=0.0){e0 hf=K1(PD);c jf=AD.zw;P5=Rb(v0,hf,jf,NC.w,NC.xy,NC.z);}c0(c2);c0(Y4);c0(P5);
#ifdef AB
c0(N0);
#endif
c0(J1);
#ifdef I
c0(B3);
#endif
#ifdef S
c0(C1);
#endif
B1(X);}
#endif
#elif defined(KB)
#ifdef CB
h1(l3)J(0,c,OC);i1 h1(C3)J(1,c,PC);i1 h1(o1)J(y9,f,VB);J(z9,f,RB);J(A9,f,NB);J(B9,uint,WB);J(C9,uint,XB);J(D9,uint,YB);J(E9,uint,LC);i1
#endif
q2 I0 W(0,c,c2);
#ifdef AB
I0 W(1,f,N0);
#endif
MB W(3,i,J1);
#ifdef I
T2 W(4,L,B3);
#endif
#ifdef S
T2 W(5,L,C1);
#endif
i2
#ifdef CB
K6(EC,l3,m3,C3,D3,o1,h0,B){K(B,m3,OC,c);K(B,D3,PC,c);K(v,h0,VB,f);K(v,h0,RB,f);K(v,h0,NB,f);K(v,h0,WB,uint);K(v,h0,XB,uint);K(v,h0,YB,uint);K(v,h0,LC,uint);U(c2,c);
#ifdef AB
U(N0,f);
#endif
U(J1,i);
#ifdef I
U(B3,L);
#endif
#ifdef S
U(C1,L);
#endif
e0 V0=K1(VB);c k0=O0(V0,OC)+NB.xy;c2=PC;
#ifdef AB
if(AB){N0=W7(K1(RB),NB.zw,k0);}
#endif
J1=unpackUnorm4x8(WB);
#ifdef I
B3=a2(XB);
#endif
#ifdef S
C1=a2(YB);
#endif
f X=Q3(k0);c0(c2);
#ifdef AB
c0(N0);
#endif
c0(J1);
#ifdef I
c0(B3);
#endif
#ifdef S
c0(C1);
#endif
B1(X);}
#endif
#endif
#ifdef HF
#ifdef CB
h1(g0)i1
#endif
q2 i2
#ifdef CB
A1(EC,g0,F,B,v){Z r2;r2.x=(B&1)==0?j.X7.x:j.X7.z;r2.y=(B&2)==0?j.X7.y:j.X7.w;f X=Q3(c(r2));B1(X);}
#endif
#endif
#ifdef NE
#endif
#if defined(OE)&&!defined(O)
#endif
#ifdef EB
L1
#ifndef O
#ifdef PE
#define J9 PE
#else
#define J9 G2
#endif
#ifdef BD
x4(J9,l0);
#else
z0(J9,l0);
#endif
#endif
#ifdef VC
#define y4 i
#define K9 K0
#define Y7 E0(.0)
#define Sb(q) ((q).w!=.0)
#ifdef I
#ifndef QC
z0(V2,i0);
#else
x4(V2,i0);
#endif
#endif
#else
#define y4 uint
#define Y7 0u
#define K9 Z0
#define Sb(q) ((q)!=0u)
#ifdef I
k1(V2,i0);
#endif
#endif
H2(L6,z4);M1 T3 Q5(Tb,lf,CD);R5(Ub,mf,PB);U3 e uint nf(float x){return uint(round(x*L9+M9));}e d Z7(uint x){return Z4(float(x)*Vb+(-M9*Vb));}L a8(L n0){
#ifdef IF
n0=min(n0,j.of);
#endif
return n0;}
#ifdef I
e void Wb(uint l1,y4 P0,M6(d)o){
#ifdef VC
if(all(lessThan(abs(P0.xy-unpackUnorm4x8(l1).xy),D2(.25/255.))))o=min(o,P0.z);else o=.0;
#else
if(l1==P0>>16)o=min(o,unpackHalf2x16(P0).x);else o=.0;
#endif
}
#endif
e void c8(uint n0,d r0,a1(i)Q
#if defined(I)&&!defined(QC)
,M6(y4)p1
#endif
N6 V3){c1 q1=T5(CD,n0);d o=r0;if((q1.x&(pf|N9))!=0u){o=abs(o);
#ifdef WC
if(WC&&(q1.x&N9)!=0u){o=1.-abs(fract(o*.5)*2.+-1.);}
#endif
}o=clamp(o,J0(.0),J0(1.));
#ifdef I
if(I){uint l1=q1.x>>16u;if(l1!=0u){Wb(l1,K9(i0),o);}}
#endif
#ifdef AB
if(AB&&(q1.x&qf)!=0u){e0 V0=K1(L0(PB,n0*E3+2u));f I2=L0(PB,n0*E3+3u);c rf=O0(V0,d0)+I2.xy;E Xb=U7(abs(rf)*I2.zw-I2.zw);d a5=clamp(min(Xb.x,Xb.y)+.5,.0,1.);o=min(o,a5);}
#endif
uint W3=q1.x&0xfu;L n3=a2((q1.x>>4)&0xfu);
#ifdef S
bool c5=S&&n3!=A4;
#else
const bool c5=false;
#endif
if(W3<=Yb){Q=unpackUnorm4x8(q1.y);
#ifdef I
if(I&&W3==d8){
#ifndef QC
#ifdef VC
p1.xy=Q.zw;p1.z=o;p1.w=1.;
#else
p1=q1.y|packHalf2x16(D2(o,.0));
#endif
#endif
Q=E0(.0);}
#endif
}else{e0 V0=K1(L0(PB,n0*E3));f I2=L0(PB,n0*E3+1u);c Zb=O0(V0,d0)+I2.xy;float t=W3==ac?Zb.x:length(Zb);t=clamp(t,.0,1.);float x=t*I2.z+I2.w;float y=uintBitsToFloat(q1.y);Q=j2(DD,P9,c(x,y),.0);if(!c5)Q.xyz*=Q.w;}
#if!defined(O)&&defined(S)
if(c5){if(Q.w*o!=.0){i N1=K0(l0);Q.xyz=X4(Q.xyz,N1,n3);}Q.xyz*=Q.w;}
#endif
Q*=o;
#if defined(ZB)&&(defined(O)||defined(QC))
Q=o3(Q);
#endif
}
#if!defined(O)&&!defined(BD)
e void e8(i Q V3){
#ifndef VC
if(Q.w==.0)return;float O6=1.-Q.w;if(O6!=.0)Q+=K0(l0)*O6;
#endif
A0(l0,Q);}
#endif
#if defined(I)&&!defined(QC)
e void Q9(y4 p1 V3){
#ifdef VC
A0(i0,p1);
#else
if(p1!=0u)d1(i0,p1);
#endif
}
#endif
#ifdef O
#define U5 v2
#define V5 p3
#else
#define U5 O1
#define V5 d2
#endif
#ifdef ND
U5(HB){
#ifdef GB
r(M,f);
#else
r(M,E);
#endif
r(D0,L);d f8;
#ifdef GB
if(GB&&bc(M)){f8=B4(M e1);}else if(GB&&cc(M)){f8=g8(M e1);}else
#endif
{f8=min(min(J0(M.x),abs(J0(M.y))),J0(1.));}i Q=E0(.0);
#ifdef I
y4 p1=Y7;
#endif
uint h8=nf(f8);uint dc=(ec(D0)<<W5)|h8;uint w2=d5(z4,dc);L D1=a2(w2>>W5);D1=a8(D1);if(D1==D0){if(!X5(M)){h8+=w2-max(dc,w2);h8-=R9;e5(z4,h8);}}else{d r0=Z7(w2&i8);c8(D1,r0,Q
#ifdef I
,p1
#endif
W2 P1);}Q.xyz=J2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef O
E1=Q;
#else
e8(Q P1);
#endif
#ifdef I
Q9(p1 P1);
#endif
V5}
#endif
#if defined(DB)||defined(FB)
U5(HB){
#ifdef FB
r(F2,c);
#else
r(j1,d);
#endif
r(D0,L);uint w2=X2(z4);L D1=a2(w2>>W5);D1=a8(D1);uint S9;
#ifndef FB
if(D1==D0){S9=w2;}else
#endif
{S9=(ec(D0)<<W5)+R9;}d o;
#ifdef FB
o=clamp(j2(ED,T9,F2,.0).x,J0(.0),J0(1.));
#else
o=j1;
#endif
int sf=int(round(o*L9));Y2(z4,S9+uint(sf));i Q=E0(.0);
#ifdef I
y4 p1=Y7;
#endif
#ifndef FB
if(D1!=D0)
#endif
{d U9=Z7(w2&i8);c8(D1,U9,Q
#ifdef I
,p1
#endif
W2 P1);}Q.xyz=J2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef O
E1=Q;
#else
e8(Q P1);
#endif
#ifdef I
Q9(p1 P1);
#endif
V5}
#endif
#ifdef NE
U5(HB){r(c2,c);
#ifdef ZC
r(Y4,d);r(P5,f);
#endif
#ifdef AB
r(N0,f);
#endif
r(J1,i);
#ifdef I
r(B3,L);
#endif
#ifdef S
r(C1,L);
#endif
i k2=j8(GC,Y5,c2);d Z5=1.;
#ifdef ZC
Z5=min(Y4,Z5);
#endif
#ifdef AB
if(AB){d a5=k3(f5(N0));Z5=clamp(a5,J0(.0),Z5);}
#endif
uint w2=X2(z4);L D1=a2(w2>>W5);D1=a8(D1);d U9=Z7(w2&i8);i Q;
#ifdef I
y4 p1=Y7;
#endif
c8(D1,U9,Q
#ifdef I
,p1
#endif
W2 P1);
#ifdef I
if(I&&B3!=0u){y4 P0=Sb(p1)?p1:K9(i0);Wb(B3,P0,Z5);}
#endif
#ifdef ZC
if(P5.w!=0.0){c V9=fc(P5);i W9=j2(DD,P9,V9,0.0);W9.xyz*=W9.w;k2*=W9;}
#endif
k2*=J1;
#if!defined(O)&&defined(S)
if(S&&C1!=A4){i N1=K0(l0)*(1.-Q.w)+Q;k2.xyz=X4(H6(k2),N1,C1)*k2.w;}
#endif
k2*=Z5;
#if defined(ZB)
k2=o3(k2);
#endif
Q=Q*(1.-k2.w)+k2;Q.xyz=J2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef O
E1=Q;
#else
e8(Q P1);
#endif
#ifdef I
Q9(p1 P1);
#endif
Y2(z4,R9);V5}
#endif
#ifdef OE
U5(HB){
#ifndef O
#ifdef QD
if(QD){A0(l0,unpackUnorm4x8(j.tf));}
#endif
#ifdef RD
if(RD){A0(l0,r1(GC,G));}
#endif
#ifdef JF
i k=K0(l0);A0(l0,k.zyxw);
#endif
#endif
Y2(z4,j.uf);
#ifdef I
if(I){d1(i0,0u);}
#endif
#ifdef O
discard;
#endif
V5}
#endif
#ifdef QC
#ifdef BD
v2(HB)
#else
U5(HB)
#endif
{uint w2=X2(z4);d r0=Z7(w2&i8);L D1=a2(w2>>W5);D1=a8(D1);i Q;c8(D1,r0,Q W2 P1);
#ifdef BD
float O6=1.-Q.w;if(O6!=.0)Q+=K0(l0)*O6;E1=Q;p3
#else
Q.xyz=J2(Q.xyz,Q.w,d0.xy,j.F3,j.G3);
#ifdef O
E1=Q;
#else
e8(Q P1);
#endif
V5
#endif
}
#endif
#endif
